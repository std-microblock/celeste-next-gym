using System.Net;
using System.Net.Sockets;

namespace Celeste.Mod.CelesteGymCollector;

internal sealed class CollectorListenerBinding(
    TcpListener listener,
    int port,
    SocketError? preferredPortFailure
) : IDisposable {
    public TcpListener Listener { get; } = listener;
    public int Port { get; } = port;
    public SocketError? PreferredPortFailure { get; } = preferredPortFailure;
    public bool FellBack => PreferredPortFailure is not null;

    public void Dispose() => Listener.Stop();
}

internal static class CollectorListenerBinder {
    public static CollectorListenerBinding Bind(int preferredPort, bool allowFallback) {
        TcpListener preferred = new(IPAddress.Loopback, preferredPort);
        try {
            preferred.Start();
            return new CollectorListenerBinding(preferred, BoundPort(preferred), null);
        } catch (SocketException error) {
            preferred.Stop();
            if (!allowFallback || !IsRecoverableDefaultPortFailure(error.SocketErrorCode)) {
                throw BindFailure(preferredPort, error);
            }
            TcpListener fallback = new(IPAddress.Loopback, 0);
            try {
                fallback.Start();
            } catch (SocketException fallbackError) {
                fallback.Stop();
                throw BindFailure(0, fallbackError);
            }
            return new CollectorListenerBinding(
                fallback,
                BoundPort(fallback),
                error.SocketErrorCode
            );
        }
    }

    private static bool IsRecoverableDefaultPortFailure(SocketError error) =>
        error is SocketError.AccessDenied or SocketError.AddressAlreadyInUse;

    private static int BoundPort(TcpListener listener) =>
        ((IPEndPoint) listener.LocalEndpoint).Port;

    private static InvalidOperationException BindFailure(int port, SocketException error) => new(
        $"Collector TCP server could not bind 127.0.0.1:{port}: " +
        $"{error.SocketErrorCode} (native {error.NativeErrorCode})",
        error
    );
}
