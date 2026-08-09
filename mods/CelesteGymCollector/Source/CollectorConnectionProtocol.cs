namespace Celeste.Mod.CelesteGymCollector;

internal static class CollectorConnectionProtocol {
    public static async Task RunAsync(
        TextReader reader,
        TextWriter writer,
        Func<string, CancellationToken, Task<string>> handleLine,
        CancellationToken cancellationToken
    ) {
        while (!cancellationToken.IsCancellationRequested) {
            string? line;
            try {
                line = await reader.ReadLineAsync(cancellationToken);
            } catch (OperationCanceledException) {
                return;
            }
            if (line is null) return;
            string response = await handleLine(line, cancellationToken);
            await writer.WriteLineAsync(response);
            await writer.FlushAsync(cancellationToken);
        }
    }
}
