using System.Text.Json;

namespace Celeste.Mod.MicroblocksQolUtils;

internal static class NativeRecordingFinalizer {
    public static async Task FinishAsync(
        IReadOnlyList<RecordingClip> clips,
        IReadOnlyList<Task> pendingStops,
        IReadOnlyCollection<string> temporaryFiles,
        string output
    ) {
        try {
            await Task.WhenAll(pendingStops).ConfigureAwait(false);
            if (clips.Count == 0 || clips.Any(clip => !File.Exists(clip.Source))) return;
            Directory.CreateDirectory(Path.GetDirectoryName(output)!);
            await NativeCaptureBridge.FinalizeRecordingAsync(clips, output).ConfigureAwait(false);
            await File.WriteAllTextAsync(
                output + ".timeline.json",
                JsonSerializer.Serialize(new { clips }, new JsonSerializerOptions { WriteIndented = true })
            ).ConfigureAwait(false);
            Logger.Log(LogLevel.Info, "MicroblocksQolUtils/Recorder", $"Saved successful room recording: {output}");
        } catch (Exception exception) {
            Logger.LogDetailed(exception, "MicroblocksQolUtils/Recorder");
        } finally {
            foreach (string file in temporaryFiles) {
                try { File.Delete(file); } catch { }
            }
        }
    }
}
