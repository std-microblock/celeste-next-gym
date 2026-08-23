using System.Diagnostics;

namespace Celeste.Mod.MicroblocksQolUtils;

internal sealed class FfmpegCaptureSession {
    private readonly Process process;
    private readonly Stopwatch elapsed = Stopwatch.StartNew();
    private int stopped;

    public string Path { get; }
    public MusicPosition MusicStart { get; }
    public double ElapsedSeconds => elapsed.Elapsed.TotalSeconds;

    private FfmpegCaptureSession(Process process, string path, MusicPosition musicStart) {
        this.process = process;
        Path = path;
        MusicStart = musicStart;
    }

    public static FfmpegCaptureSession? Start(string ffmpeg, string output) {
        QolSettings settings = MicroblocksQolUtilsModule.Settings;
        try {
            ProcessStartInfo start = new() {
                FileName = ffmpeg,
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardInput = true
            };
            Add(start, "-hide_banner", "-loglevel", "warning", "-y");
            Add(start, "-f", "gdigrab", "-framerate", settings.RecordingFrameRate.ToString(), "-draw_mouse", "0");
            Add(start, "-rtbufsize", "64M", "-i", $"title={settings.RecordingWindowTitle}");
            bool hasAudio = !string.IsNullOrWhiteSpace(settings.RecordingAudioDevice);
            if (hasAudio) Add(start, "-f", "dshow", "-i", $"audio={settings.RecordingAudioDevice}");
            Add(start, "-map", "0:v:0");
            if (hasAudio) Add(start, "-map", "1:a:0");
            else Add(start, "-an");
            Add(start,
                "-c:v", string.IsNullOrWhiteSpace(settings.RecordingEncoder) ? "libx264" : settings.RecordingEncoder,
                "-b:v", $"{settings.RecordingBitrateKbps}k",
                "-g", settings.RecordingFrameRate.ToString(),
                "-pix_fmt", "yuv420p"
            );
            if (hasAudio) Add(start, "-c:a", "aac", "-b:a", "192k");
            Add(start, output);
            Process process = Process.Start(start) ?? throw new InvalidOperationException("FFmpeg did not start.");
            return new FfmpegCaptureSession(process, output, MusicPosition.Read());
        } catch (Exception exception) {
            Logger.Log(LogLevel.Error, "MicroblocksQolUtils/Recorder", $"Cannot start FFmpeg: {exception.Message}");
            return null;
        }
    }

    public Task StopAsync() {
        if (Interlocked.Exchange(ref stopped, 1) != 0) return Task.CompletedTask;
        elapsed.Stop();
        return Task.Run(() => {
            try {
                if (!process.HasExited) {
                    process.StandardInput.WriteLine("q");
                    process.StandardInput.Flush();
                    if (!process.WaitForExit(5000)) process.Kill(entireProcessTree: true);
                }
            } catch {
                try { if (!process.HasExited) process.Kill(entireProcessTree: true); } catch { }
            } finally {
                process.Dispose();
            }
        });
    }

    private static void Add(ProcessStartInfo start, params string[] values) {
        foreach (string value in values) start.ArgumentList.Add(value);
    }
}

public readonly record struct MusicPosition(string Event, int TimelineMilliseconds) {
    public static MusicPosition Read() {
        try {
            FMOD.Studio.EventInstance instance = Audio.CurrentMusicEventInstance;
            string name = Audio.GetEventName(instance) ?? "";
            instance.getTimelinePosition(out int position);
            return new MusicPosition(name, Math.Max(0, position));
        } catch {
            return new MusicPosition("", 0);
        }
    }
}

