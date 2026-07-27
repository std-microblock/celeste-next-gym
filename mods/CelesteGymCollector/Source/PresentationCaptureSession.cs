using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;
using System.Diagnostics;
using System.Security.Cryptography;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Celeste.Mod.CelesteGymCollector;

internal sealed class PresentationCaptureSession {
    public const int OutputWidth = 320;
    public const int OutputHeight = 180;
    public const int OutputByteLength = OutputWidth * OutputHeight * 4;
    public const int OutputFrameRate = 60;
    public const int TailPresentationFrames = OutputFrameRate;

    private readonly string recordingRoot;
    private readonly string sessionDirectory;
    private readonly string framesDirectory;
    private readonly string runNonce;
    private readonly int processId;
    private readonly string token;
    private readonly string scenarioId;
    private readonly TimeSpan timeout;
    private readonly long startedTimestamp = Stopwatch.GetTimestamp();
    private readonly DateTimeOffset startedAt = DateTimeOffset.UtcNow;
    private readonly RecordingTimeline timeline;
    private readonly JsonSerializerOptions json = new(JsonSerializerDefaults.Web) { WriteIndented = true };
    private Color[] sourcePixels = [];
    private byte[] outputPixels = new byte[OutputByteLength];
    private bool boundToSimulation;
    private int latestStateIndex = -1;
    private string state = "active";
    private string? reason;
    private string? manifestPath;

    public PresentationCaptureSession(
        string recordingRoot,
        string runNonce,
        int processId,
        CaptureStartRequest request
    ) {
        this.recordingRoot = RecordingSecurity.ResolveRecordingRoot(recordingRoot);
        this.runNonce = runNonce;
        this.processId = processId;
        token = RecordingSecurity.ValidateToken(request.CaptureToken);
        scenarioId = RecordingSecurity.ValidateScenarioId(request.ScenarioId);
        if (request.StartStateIndex < 0 || request.EndStateIndex < request.StartStateIndex) {
            throw new InvalidOperationException("recording state range is invalid");
        }
        if (request.TimeoutMilliseconds is < 1_000 or > 600_000) {
            throw new InvalidOperationException("recording timeout must be between 1000 and 600000 ms");
        }
        timeout = TimeSpan.FromMilliseconds(request.TimeoutMilliseconds);
        timeline = new RecordingTimeline(request.StartStateIndex, request.EndStateIndex);
        sessionDirectory = RecordingSecurity.CreateSessionDirectory(this.recordingRoot, scenarioId, token);
        framesDirectory = Path.Combine(sessionDirectory, "frames");
        Directory.CreateDirectory(framesDirectory);
        RecordingSecurity.EnsureContained(this.recordingRoot, framesDirectory);
    }

    public string Token => token;
    public bool IsCapturing => state == "active";
    public bool CanFinalize => state is "ready" or "stopped" or "timed_out" or "faulted" or "finalized";

    public void BindToSimulation(int finalStateIndex) {
        CheckTimeout();
        if (!IsCapturing) throw new InvalidOperationException("capture session is not active");
        if (boundToSimulation) throw new InvalidOperationException("capture session is already bound to a simulation");
        if (finalStateIndex != timeline.EndStateIndex) {
            throw new InvalidOperationException(
                "recording end_state_index must equal the simulation final state index"
            );
        }
        boundToSimulation = true;
    }

    public void UpdateLatestStateIndex(int stateIndex) {
        if (!boundToSimulation || !IsCapturing) return;
        if (stateIndex < latestStateIndex) {
            Fail("simulation state index moved backwards");
            return;
        }
        latestStateIndex = stateIndex;
        CheckTimeout();
    }

    public void CapturePresentation(GraphicsDevice graphicsDevice, Rectangle viewport) {
        CheckTimeout();
        if (!IsCapturing || !boundToSimulation || latestStateIndex < timeline.StartStateIndex) return;
        if (latestStateIndex > timeline.EndStateIndex) {
            Fail("simulation advanced beyond the declared final recording state");
            return;
        }
        try {
            if (viewport.Width <= 0 || viewport.Height <= 0) {
                throw new InvalidOperationException("render viewport is empty");
            }
            int sourceLength = checked(viewport.Width * viewport.Height);
            if (sourcePixels.Length != sourceLength) sourcePixels = new Color[sourceLength];
            graphicsDevice.GetBackBufferData(viewport, sourcePixels, 0, sourceLength);
            DownsampleBgra(sourcePixels, viewport.Width, viewport.Height, outputPixels);

            int renderIndex = timeline.Frames.Count;
            string fileName = $"{renderIndex:D6}.bgra";
            string finalPath = Path.Combine(framesDirectory, fileName);
            string partialPath = finalPath + ".partial";
            RecordingSecurity.EnsureContained(recordingRoot, finalPath);
            using (FileStream stream = new(
                partialPath,
                FileMode.CreateNew,
                FileAccess.Write,
                FileShare.None,
                64 * 1024,
                FileOptions.WriteThrough
            )) {
                stream.Write(outputPixels);
                stream.Flush(flushToDisk: true);
            }
            File.Move(partialPath, finalPath, overwrite: false);
            string hash = Convert.ToHexString(SHA256.HashData(outputPixels)).ToLowerInvariant();
            string relativePath = Path.GetRelativePath(sessionDirectory, finalPath);
            timeline.AddFrame(
                latestStateIndex,
                ElapsedNanoseconds(),
                relativePath,
                hash,
                outputPixels.LongLength
            );
            if (timeline.HasPresentedFinalStateTail(TailPresentationFrames)) {
                state = "ready";
                reason = "final_state_tail_presented";
            }
        } catch (Exception error) {
            Fail(error.ToString());
        }
    }

    public void Stop(string stopReason) {
        CheckTimeout();
        if (state is "finalized") return;
        if (state == "active") {
            state = "stopped";
            reason = string.IsNullOrWhiteSpace(stopReason) ? "explicit_stop" : stopReason;
            timeline.CompleteUnpresentedRanges(latestStateIndex);
        }
    }

    public CaptureStatus FinalizeManifest() {
        CheckTimeout();
        if (!CanFinalize) {
            throw new InvalidOperationException(
                "capture is still active; wait for final_state_presented or stop it explicitly"
            );
        }
        if (state == "finalized") return GetStatus();

        timeline.CompleteUnpresentedRanges(latestStateIndex);
        RecordingManifest manifest = new() {
            ScenarioId = scenarioId,
            RunNonce = runNonce,
            ProcessId = processId,
            CaptureTokenSha256 = Convert.ToHexString(
                SHA256.HashData(System.Text.Encoding.UTF8.GetBytes(token))
            ).ToLowerInvariant(),
            StartedAt = startedAt,
            FinalizedAt = DateTimeOffset.UtcNow,
            Outcome = state,
            Reason = reason,
            StartStateIndex = timeline.StartStateIndex,
            EndStateIndex = timeline.EndStateIndex,
            LatestStateIndex = latestStateIndex,
            FinalStatePresented = timeline.FinalStatePresented,
            RepeatedPresentationCount = timeline.RepeatedPresentationCount,
            UnpresentedUpdateRanges = timeline.UnpresentedUpdateRanges,
            Frames = timeline.Frames
        };
        string finalPath = Path.Combine(sessionDirectory, "manifest.json");
        string partialPath = Path.Combine(sessionDirectory, "manifest.partial.json");
        RecordingSecurity.EnsureContained(recordingRoot, finalPath);
        byte[] bytes = JsonSerializer.SerializeToUtf8Bytes(manifest, json);
        using (FileStream stream = new(
            partialPath,
            FileMode.CreateNew,
            FileAccess.Write,
            FileShare.None,
            64 * 1024,
            FileOptions.WriteThrough
        )) {
            stream.Write(bytes);
            stream.Flush(flushToDisk: true);
        }
        File.Move(partialPath, finalPath, overwrite: false);
        manifestPath = Path.GetRelativePath(recordingRoot, finalPath).Replace('\\', '/');
        state = "finalized";
        return GetStatus();
    }

    public CaptureStatus GetStatus() {
        CheckTimeout();
        return new CaptureStatus {
            State = state,
            ScenarioId = scenarioId,
            StartStateIndex = timeline.StartStateIndex,
            EndStateIndex = timeline.EndStateIndex,
            LatestStateIndex = latestStateIndex,
            RenderFrameCount = timeline.Frames.Count,
            FinalStatePresented = timeline.FinalStatePresented,
            RepeatedPresentationCount = timeline.RepeatedPresentationCount,
            UnpresentedUpdateRanges = [.. timeline.UnpresentedUpdateRanges],
            ManifestPath = manifestPath,
            Reason = reason
        };
    }

    private void CheckTimeout() {
        if (state != "active" || Stopwatch.GetElapsedTime(startedTimestamp) <= timeout) return;
        state = "timed_out";
        reason = $"capture exceeded {timeout.TotalMilliseconds:0} ms";
        timeline.CompleteUnpresentedRanges(latestStateIndex);
    }

    private void Fail(string failure) {
        state = "faulted";
        reason = failure;
        timeline.CompleteUnpresentedRanges(latestStateIndex);
    }

    private long ElapsedNanoseconds() => checked(Stopwatch.GetElapsedTime(startedTimestamp).Ticks * 100L);

    internal static void DownsampleBgra(
        Color[] source,
        int sourceWidth,
        int sourceHeight,
        byte[] destination
    ) {
        if (source.Length != checked(sourceWidth * sourceHeight)) {
            throw new ArgumentException("source pixel count does not match dimensions", nameof(source));
        }
        if (destination.Length != OutputByteLength) {
            throw new ArgumentException("destination must be exactly 320x180 BGRA", nameof(destination));
        }
        for (int y = 0; y < OutputHeight; y++) {
            int sourceY = y * sourceHeight / OutputHeight;
            for (int x = 0; x < OutputWidth; x++) {
                int sourceX = x * sourceWidth / OutputWidth;
                Color color = source[sourceY * sourceWidth + sourceX];
                int offset = (y * OutputWidth + x) * 4;
                destination[offset] = color.B;
                destination[offset + 1] = color.G;
                destination[offset + 2] = color.R;
                destination[offset + 3] = color.A;
            }
        }
    }
}

internal sealed class RecordingManifest {
    [JsonPropertyName("schema_version")]
    public int SchemaVersion { get; set; } = 1;

    [JsonPropertyName("capture_semantics")]
    public string CaptureSemantics { get; set; } = "presentation_frames";

    [JsonPropertyName("scenario_id")]
    public string ScenarioId { get; set; } = "";

    [JsonPropertyName("run_nonce")]
    public string RunNonce { get; set; } = "";

    [JsonPropertyName("process_id")]
    public int ProcessId { get; set; }

    [JsonPropertyName("capture_token_sha256")]
    public string CaptureTokenSha256 { get; set; } = "";

    [JsonPropertyName("width")]
    public int Width { get; set; } = PresentationCaptureSession.OutputWidth;

    [JsonPropertyName("height")]
    public int Height { get; set; } = PresentationCaptureSession.OutputHeight;

    [JsonPropertyName("pixel_format")]
    public string PixelFormat { get; set; } = "bgra";

    [JsonPropertyName("encoding_frame_rate")]
    public int EncodingFrameRate { get; set; } = PresentationCaptureSession.OutputFrameRate;

    [JsonPropertyName("started_at")]
    public DateTimeOffset StartedAt { get; set; }

    [JsonPropertyName("finalized_at")]
    public DateTimeOffset FinalizedAt { get; set; }

    [JsonPropertyName("outcome")]
    public string Outcome { get; set; } = "";

    [JsonPropertyName("reason")]
    public string? Reason { get; set; }

    [JsonPropertyName("start_state_index")]
    public int StartStateIndex { get; set; }

    [JsonPropertyName("end_state_index")]
    public int EndStateIndex { get; set; }

    [JsonPropertyName("latest_state_index")]
    public int LatestStateIndex { get; set; }

    [JsonPropertyName("final_state_presented")]
    public bool FinalStatePresented { get; set; }

    [JsonPropertyName("repeated_presentation_count")]
    public int RepeatedPresentationCount { get; set; }

    [JsonPropertyName("unpresented_update_ranges")]
    public List<StateIndexRange> UnpresentedUpdateRanges { get; set; } = [];

    [JsonPropertyName("frames")]
    public List<RecordingFrameManifest> Frames { get; set; } = [];
}
