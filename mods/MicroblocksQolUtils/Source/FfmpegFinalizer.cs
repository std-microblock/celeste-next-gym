using System.Diagnostics;
using System.Globalization;
using System.Text.Json;

namespace Celeste.Mod.MicroblocksQolUtils;

internal static class FfmpegFinalizer {
    public static async Task FinishAsync(
        string ffmpeg,
        IReadOnlyList<RecordingClip> clips,
        IReadOnlyList<Task> pendingStops,
        IReadOnlyCollection<string> temporaryFiles,
        string output,
        bool hasCapturedAudio,
        BgmRecordingMode bgmMode,
        string bgmMapFile
    ) {
        try {
            await Task.WhenAll(pendingStops).ConfigureAwait(false);
            if (clips.Count == 0 || clips.Any(clip => !File.Exists(clip.Source))) return;
            Directory.CreateDirectory(Path.GetDirectoryName(output)!);
            await ConcatAsync(ffmpeg, clips, output, hasCapturedAudio).ConfigureAwait(false);
            await File.WriteAllTextAsync(
                output + ".timeline.json",
                JsonSerializer.Serialize(new { clips }, new JsonSerializerOptions { WriteIndented = true })
            ).ConfigureAwait(false);
            if (bgmMode == BgmRecordingMode.SfxOnlyWithPostMix && File.Exists(bgmMapFile)) {
                await MixBgmAsync(ffmpeg, clips, output, hasCapturedAudio, bgmMapFile).ConfigureAwait(false);
            }
            Logger.Log(LogLevel.Info, "MicroblocksQolUtils/Recorder", $"Saved successful room recording: {output}");
        } catch (Exception exception) {
            Logger.LogDetailed(exception, "MicroblocksQolUtils/Recorder");
        } finally {
            foreach (string file in temporaryFiles) {
                try { File.Delete(file); } catch { }
            }
        }
    }

    private static async Task ConcatAsync(string ffmpeg, IReadOnlyList<RecordingClip> clips, string output, bool audio) {
        ProcessStartInfo start = NewFfmpeg(ffmpeg);
        foreach (RecordingClip clip in clips) Add(start, "-i", clip.Source);
        List<string> filters = [];
        for (int i = 0; i < clips.Count; i++) {
            string duration = clips[i].DurationSeconds.ToString("0.######", CultureInfo.InvariantCulture);
            filters.Add($"[{i}:v]trim=start=0:duration={duration},setpts=PTS-STARTPTS[v{i}]");
            if (audio) filters.Add($"[{i}:a]atrim=start=0:duration={duration},asetpts=PTS-STARTPTS[a{i}]");
        }
        string inputs = string.Concat(Enumerable.Range(0, clips.Count).Select(i => audio ? $"[v{i}][a{i}]" : $"[v{i}]"));
        filters.Add($"{inputs}concat=n={clips.Count}:v=1:a={(audio ? 1 : 0)}[video]{(audio ? "[audio]" : "")}");
        Add(start, "-filter_complex", string.Join(';', filters), "-map", "[video]");
        if (audio) Add(start, "-map", "[audio]");
        Add(start, "-c:v", "libx264", "-preset", "veryfast", "-crf", "18", "-pix_fmt", "yuv420p");
        if (audio) Add(start, "-c:a", "aac", "-b:a", "192k");
        Add(start, output);
        await RunAsync(start).ConfigureAwait(false);
    }

    private static async Task MixBgmAsync(
        string ffmpeg,
        IReadOnlyList<RecordingClip> clips,
        string output,
        bool capturedAudio,
        string mapFile
    ) {
        Dictionary<string, string>? map = JsonSerializer.Deserialize<Dictionary<string, string>>(
            await File.ReadAllTextAsync(mapFile).ConfigureAwait(false)
        );
        if (map is null) return;
        string[] musicFiles = clips.Select(clip => map.GetValueOrDefault(clip.MusicEvent) ?? "").ToArray();
        if (musicFiles.Any(file => file.Length == 0 || !File.Exists(file))) {
            Logger.Log(LogLevel.Warn, "MicroblocksQolUtils/Recorder", "BGM post-mix skipped: timeline event is absent from the BGM map.");
            return;
        }

        string mixed = output + ".mixed.mp4";
        ProcessStartInfo start = NewFfmpeg(ffmpeg);
        Add(start, "-i", output);
        foreach (string file in musicFiles) Add(start, "-stream_loop", "-1", "-i", file);
        List<string> filters = [];
        for (int i = 0; i < clips.Count; i++) {
            string begin = (clips[i].MusicTimelineMilliseconds / 1000.0).ToString("0.######", CultureInfo.InvariantCulture);
            string duration = clips[i].DurationSeconds.ToString("0.######", CultureInfo.InvariantCulture);
            filters.Add($"[{i + 1}:a]atrim=start={begin}:duration={duration},asetpts=PTS-STARTPTS[bgm{i}]");
        }
        string inputs = string.Concat(Enumerable.Range(0, clips.Count).Select(i => $"[bgm{i}]"));
        filters.Add($"{inputs}concat=n={clips.Count}:v=0:a=1[bgm]");
        if (capturedAudio) filters.Add("[0:a][bgm]amix=inputs=2:duration=first:normalize=0[audio]");
        Add(start, "-filter_complex", string.Join(';', filters), "-map", "0:v:0", "-map", capturedAudio ? "[audio]" : "[bgm]");
        Add(start, "-c:v", "copy", "-c:a", "aac", "-b:a", "256k", mixed);
        await RunAsync(start).ConfigureAwait(false);
        File.Move(mixed, output, overwrite: true);
    }

    private static ProcessStartInfo NewFfmpeg(string ffmpeg) {
        ProcessStartInfo start = new() { FileName = ffmpeg, UseShellExecute = false, CreateNoWindow = true };
        Add(start, "-hide_banner", "-loglevel", "warning", "-y");
        return start;
    }

    private static async Task RunAsync(ProcessStartInfo start) {
        using Process process = Process.Start(start) ?? throw new InvalidOperationException("FFmpeg finalizer did not start.");
        await process.WaitForExitAsync().ConfigureAwait(false);
        if (process.ExitCode != 0) throw new InvalidOperationException($"FFmpeg exited with code {process.ExitCode}.");
    }

    private static void Add(ProcessStartInfo start, params string[] values) {
        foreach (string value in values) start.ArgumentList.Add(value);
    }
}

