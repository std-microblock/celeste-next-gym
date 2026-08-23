using System.Diagnostics;

namespace Celeste.Mod.MicroblocksQolUtils;

internal sealed class NativeRecordingSegment {
    private readonly NativeCaptureSession capture;
    private readonly Stopwatch elapsed = Stopwatch.StartNew();
    private int stopped;

    public string Path { get; }
    public MusicPosition MusicStart { get; }
    public double ElapsedSeconds => elapsed.Elapsed.TotalSeconds;

    private NativeRecordingSegment(NativeCaptureSession capture, string path, MusicPosition musicStart) {
        this.capture = capture;
        Path = path;
        MusicStart = musicStart;
    }

    public static NativeRecordingSegment? Start(string output) {
        QolSettings settings = MicroblocksQolUtilsModule.Settings;
        try {
            NativeCaptureSession capture = NativeCaptureBridge.StartRecording(
                settings.RecordingWindowTitle,
                settings.RecordingFrameRate,
                output,
                settings.RecordingEncoder,
                settings.RecordingBitrateKbps
            );
            return new NativeRecordingSegment(capture, output, MusicPosition.Read());
        } catch (Exception exception) {
            Logger.Log(LogLevel.Error, "MicroblocksQolUtils/Recorder", $"Cannot start native recording: {exception.Message}");
            return null;
        }
    }

    public Task StopAsync() {
        if (Interlocked.Exchange(ref stopped, 1) != 0) return Task.CompletedTask;
        elapsed.Stop();
        return Task.Run(capture.Dispose);
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
