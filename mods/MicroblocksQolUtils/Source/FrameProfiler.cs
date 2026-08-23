using System.Diagnostics;

namespace Celeste.Mod.MicroblocksQolUtils;

public static class FrameProfiler {
    private static readonly Stopwatch Stopwatch = new();
    private static double smoothedMilliseconds = 16.6667;

    public static double LastFrameMilliseconds { get; private set; } = 16.6667;
    public static double FramesPerSecond => smoothedMilliseconds <= 0.0001 ? 0 : 1000.0 / smoothedMilliseconds;

    public static void BeginFrame() => Stopwatch.Restart();

    public static void EndFrame() {
        Stopwatch.Stop();
        LastFrameMilliseconds = Stopwatch.Elapsed.TotalMilliseconds;
        smoothedMilliseconds += (LastFrameMilliseconds - smoothedMilliseconds) * 0.08;
    }
}

