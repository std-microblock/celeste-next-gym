using System.Collections.Concurrent;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Text.Json;

namespace Celeste.Mod.CelesteGymCollector;

internal sealed class PendingRequest(CollectorRequest request) {
    public CollectorRequest Request { get; } = request;
    public TaskCompletionSource<CollectorResponse> Completion { get; } =
        new(TaskCreationOptions.RunContinuationsAsynchronously);
}

internal sealed class CollectorServer : IDisposable {
    private readonly ConcurrentQueue<PendingRequest> pending = new();
    private readonly CancellationTokenSource cancellation = new();
    private readonly JsonSerializerOptions json = new(JsonSerializerDefaults.Web);
    private TcpListener? listener;
    private Task? acceptLoop;

    public bool TryDequeue(out PendingRequest? request) => pending.TryDequeue(out request);
    public bool TryPeek(out PendingRequest? request) => pending.TryPeek(out request);

    public void Start(int port = 32270) {
        listener = new TcpListener(IPAddress.Loopback, port);
        listener.Start();
        acceptLoop = Task.Run(AcceptLoopAsync);
        Logger.Log(LogLevel.Info, "CelesteGymCollector", $"Collector TCP server listening on 127.0.0.1:{port}");
    }

    private async Task AcceptLoopAsync() {
        while (!cancellation.IsCancellationRequested && listener is not null) {
            try {
                TcpClient client = await listener.AcceptTcpClientAsync(cancellation.Token);
                _ = Task.Run(() => HandleClientAsync(client));
            } catch (OperationCanceledException) {
                break;
            } catch (Exception error) {
                Logger.Log(LogLevel.Error, "CelesteGymCollector", error.ToString());
            }
        }
    }

    private async Task HandleClientAsync(TcpClient client) {
        using (client)
        using (NetworkStream stream = client.GetStream())
        using (StreamReader reader = new(stream, Encoding.UTF8, leaveOpen: true))
        using (StreamWriter writer = new(stream, new UTF8Encoding(false), leaveOpen: true) { AutoFlush = true }) {
            try {
                string? line = await reader.ReadLineAsync(cancellation.Token);
                CollectorRequest? request = line is null ? null : JsonSerializer.Deserialize<CollectorRequest>(line, json);
                if (request is null) {
                    await writer.WriteLineAsync(JsonSerializer.Serialize(new CollectorResponse { Success = false, Error = "invalid request" }, json));
                    return;
                }
                PendingRequest pendingRequest = new(request);
                pending.Enqueue(pendingRequest);
                CollectorResponse response = await pendingRequest.Completion.Task.WaitAsync(TimeSpan.FromSeconds(60), cancellation.Token);
                await writer.WriteLineAsync(JsonSerializer.Serialize(response, json));
            } catch (Exception error) {
                await writer.WriteLineAsync(JsonSerializer.Serialize(new CollectorResponse { Success = false, Error = error.Message }, json));
            }
        }
    }

    public void Dispose() {
        cancellation.Cancel();
        listener?.Stop();
        try { acceptLoop?.Wait(TimeSpan.FromSeconds(2)); } catch { }
        cancellation.Dispose();
    }
}
