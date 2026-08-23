namespace Celeste.Mod.MicroblocksQolUtils;

internal static class NativeCaptureSmoke {
    private const string OutputVariable = "MICROBLOCKS_QOL_CAPTURE_SMOKE_OUTPUT";
    private static CancellationTokenSource? cancellation;
    private static NativeCaptureSession? active;

    public static void Load() {
        string? configured = Environment.GetEnvironmentVariable(OutputVariable);
        if (string.IsNullOrWhiteSpace(configured) || !OperatingSystem.IsWindows()) return;
        string output = Path.GetFullPath(configured);
        cancellation = new CancellationTokenSource();
        _ = RunAsync(output, cancellation.Token);
    }

    public static void Unload() {
        cancellation?.Cancel();
        cancellation?.Dispose();
        cancellation = null;
        Interlocked.Exchange(ref active, null)?.Dispose();
    }

    private static async Task RunAsync(string output, CancellationToken token) {
        try {
            Directory.CreateDirectory(Path.GetDirectoryName(output)!);
            await Task.Delay(3_000, token).ConfigureAwait(false);
            NativeCaptureSession capture = NativeCaptureBridge.StartRecording(
                "Celeste",
                30,
                output,
                "libopenh264",
                2_000
            );
            Interlocked.Exchange(ref active, capture)?.Dispose();
            await Task.Delay(2_000, token).ConfigureAwait(false);
            CaptureStatistics statistics = capture.Statistics;
            Interlocked.Exchange(ref active, null)?.Dispose();
            FileInfo file = new(output);
            long length = file.Exists ? file.Length : 0;
            if (length < 1_000) {
                throw new InvalidDataException(
                    $"native capture output is missing or too small: {length} bytes; "
                    + $"running={statistics.Running} captured={statistics.FramesCaptured} "
                    + $"consumed={statistics.FramesConsumed} dropped={statistics.FramesDropped}; "
                    + $"nativeError={NativeCaptureBridge.LastError()}"
                );
            }
            Logger.Log(LogLevel.Info, "MicroblocksQolUtils/Recorder", $"QOL_CAPTURE_SMOKE_PASSED {output}");
        } catch (OperationCanceledException) {
        } catch (Exception exception) {
            Logger.LogDetailed(exception, "MicroblocksQolUtils/Recorder/CaptureSmoke");
        } finally {
            Interlocked.Exchange(ref active, null)?.Dispose();
        }
    }
}
