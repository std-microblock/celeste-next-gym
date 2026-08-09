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

    public int Start(int preferredPort = 32270, bool allowFallback = false) {
        CollectorListenerBinding binding = CollectorListenerBinder.Bind(
            preferredPort,
            allowFallback
        );
        listener = binding.Listener;
        acceptLoop = Task.Run(AcceptLoopAsync);
        if (binding.FellBack) {
            Logger.Log(
                LogLevel.Warn,
                "CelesteGymCollector",
                $"Default collector port 127.0.0.1:{preferredPort} failed with " +
                $"{binding.PreferredPortFailure}; using 127.0.0.1:{binding.Port}"
            );
        }
        Logger.Log(
            LogLevel.Info,
            "CelesteGymCollector",
            $"Collector TCP server listening on 127.0.0.1:{binding.Port}"
        );
        return binding.Port;
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
        using (StreamWriter writer = new(stream, new UTF8Encoding(false), leaveOpen: true)) {
            try {
                await CollectorConnectionProtocol.RunAsync(
                    reader,
                    writer,
                    async (line, cancellationToken) => {
                        CollectorResponse response;
                        try {
                            CollectorRequest? request =
                                JsonSerializer.Deserialize<CollectorRequest>(line, json);
                            if (request is null) {
                                response = new CollectorResponse {
                                    Success = false,
                                    Error = "invalid request"
                                };
                            } else {
                                PendingRequest pendingRequest = new(request);
                                pending.Enqueue(pendingRequest);
                                response = await pendingRequest.Completion.Task.WaitAsync(
                                    TimeSpan.FromSeconds(60),
                                    cancellationToken
                                );
                            }
                        } catch (OperationCanceledException) {
                            throw;
                        } catch (Exception error) {
                            response = new CollectorResponse {
                                Success = false,
                                Error = error.Message
                            };
                        }
                        return JsonSerializer.Serialize(response, json);
                    },
                    cancellation.Token
                );
            } catch (OperationCanceledException) {
            } catch (IOException) {
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
