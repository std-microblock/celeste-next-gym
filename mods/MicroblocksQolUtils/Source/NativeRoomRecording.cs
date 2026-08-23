namespace Celeste.Mod.MicroblocksQolUtils;

internal sealed class NativeRoomRecording {
    private readonly NativeCaptureSession capture;
    private int stopped;
    private double lastMediaTime;

    public string Path { get; }

    private NativeRoomRecording(NativeCaptureSession capture, string path) {
        this.capture = capture;
        Path = path;
    }

    public double MediaTimeSeconds {
        get {
            try {
                double value = capture.Statistics.MediaTimeSeconds;
                if (value > lastMediaTime) lastMediaTime = value;
            } catch (Exception exception) {
                Logger.Log(LogLevel.Warn, "MicroblocksQolUtils/Recorder", $"Cannot read native media clock: {exception.Message}");
            }
            return lastMediaTime;
        }
    }

    public static NativeRoomRecording? Start(string output) {
        QolSettings settings = MicroblocksQolUtilsModule.Settings;
        try {
            NativeCaptureSession capture = NativeCaptureBridge.StartRecording(
                settings.RecordingWindowTitle,
                settings.RecordingFrameRate,
                output,
                settings.RecordingEncoder,
                settings.RecordingBitrateKbps
            );
            return new NativeRoomRecording(capture, output);
        } catch (Exception exception) {
            Logger.Log(LogLevel.Error, "MicroblocksQolUtils/Recorder", $"Cannot start native recording: {exception.Message}");
            return null;
        }
    }

    public Task StopAsync() {
        if (Interlocked.Exchange(ref stopped, 1) != 0) return Task.CompletedTask;
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
