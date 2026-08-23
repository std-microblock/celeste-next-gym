using Monocle;

namespace Celeste.Mod.MicroblocksQolUtils;

public static class NativeCaptureCommands {
    private static NativeCaptureSession? probe;

    [Command("qol_capture_probe_start", "Start the native scap/WGC capture probe")]
    public static void Start() {
        probe?.Dispose();
        probe = NativeCaptureBridge.Start(
            MicroblocksQolUtilsModule.Settings.RecordingWindowTitle,
            MicroblocksQolUtilsModule.Settings.RecordingFrameRate
        );
        Engine.Commands.Log("Native scap capture probe started.");
    }

    [Command("qol_capture_probe_stats", "Show native scap/WGC capture statistics")]
    public static void Stats() {
        if (probe is null) {
            Engine.Commands.Log("Native capture probe is not running.");
            return;
        }
        CaptureStatistics stats = probe.Statistics;
        Engine.Commands.Log(
            $"running={stats.Running} size={stats.Width}x{stats.Height} "
            + $"captured={stats.FramesCaptured} consumed={stats.FramesConsumed} "
            + $"dropped={stats.FramesDropped} queue={stats.QueueDepth} "
            + $"bytes={stats.BytesCaptured}"
        );
    }

    [Command("qol_capture_probe_stop", "Stop the native scap/WGC capture probe")]
    public static void Stop() {
        probe?.Dispose();
        probe = null;
        Engine.Commands.Log("Native scap capture probe stopped.");
    }

    public static void Unload() {
        probe?.Dispose();
        probe = null;
    }
}

