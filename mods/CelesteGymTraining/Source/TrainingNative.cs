using System.Reflection;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

namespace Celeste.Mod.CelesteGymTraining;

/// <summary>
/// Managed owner for the stable C ABI exported by celeste-gym-native.
/// This is the same physics/Fuzz implementation used by celeste-wasm, not a
/// second C# approximation of candidate enumeration or Rhai checks.
/// </summary>
public static class TrainingNative {
    private const string LibraryName = "celeste_gym_native";
    private static bool resolverInstalled;
    private static IntPtr nativeHandle;
    private static string? nativeLoadError;
    private static string? configuredNativeDirectory;

    public static void Initialize(string? nativeDirectory = null) {
        if (resolverInstalled) return;
        configuredNativeDirectory = string.IsNullOrWhiteSpace(nativeDirectory)
            ? null
            : Path.GetFullPath(nativeDirectory);
        resolverInstalled = true;
        NativeLibrary.SetDllImportResolver(typeof(TrainingNative).Assembly, ResolveLibrary);
        string? candidate = NativeLibraryPath(typeof(TrainingNative).Assembly);
        if (candidate is null) {
            nativeLoadError = $"managed assembly has no usable location: {typeof(TrainingNative).Assembly.Location}";
            LogError(nativeLoadError);
            return;
        }
        try {
            nativeHandle = NativeLibrary.Load(candidate);
            LogInfo($"Loaded native training bridge: {candidate}");
        } catch (Exception error) {
            nativeLoadError = $"failed to load native training bridge at {candidate}: {error}";
            LogError(nativeLoadError);
        }
    }

    public static JsonElement CacheMap(JsonElement map) =>
        Invoke(Encoding.UTF8.GetBytes(map.GetRawText()), null, NativeOperation.CacheMap);

    public static JsonElement FuzzSearch(JsonElement snapshot, JsonElement fuzz) =>
        Invoke(
            Encoding.UTF8.GetBytes(snapshot.GetRawText()),
            Encoding.UTF8.GetBytes(fuzz.GetRawText()),
            NativeOperation.FuzzSearch
        );

    public static bool EvaluateEntryChecks(JsonElement snapshot, JsonElement checks) {
        JsonElement result = Invoke(
            Encoding.UTF8.GetBytes(snapshot.GetRawText()),
            Encoding.UTF8.GetBytes(checks.GetRawText()),
            NativeOperation.EntryCheck
        );
        return result.ValueKind == JsonValueKind.True;
    }

    private static JsonElement Invoke(byte[] first, byte[]? second, NativeOperation operation) {
        Initialize();
        if (nativeHandle == IntPtr.Zero && nativeLoadError is not null) {
            throw new DllNotFoundException(nativeLoadError);
        }
        NativeBuffer buffer = operation switch {
            NativeOperation.CacheMap => CacheMapJson(first, (nuint) first.Length),
            NativeOperation.FuzzSearch => FuzzSearchJson(
                first,
                (nuint) first.Length,
                second ?? [],
                (nuint) (second?.Length ?? 0)
            ),
            NativeOperation.EntryCheck => TrainingEntryCheckJson(
                first,
                (nuint) first.Length,
                second ?? [],
                (nuint) (second?.Length ?? 0)
            ),
            _ => throw new ArgumentOutOfRangeException(nameof(operation))
        };
        try {
            if (buffer.Data == IntPtr.Zero || buffer.Length == 0) {
                throw new InvalidOperationException("native training bridge returned an empty buffer");
            }
            if (buffer.Length > int.MaxValue) {
                throw new InvalidOperationException("native training bridge returned an oversized buffer");
            }
            byte[] bytes = new byte[(int) buffer.Length];
            Marshal.Copy(buffer.Data, bytes, 0, bytes.Length);
            using JsonDocument response = JsonDocument.Parse(bytes);
            JsonElement root = response.RootElement;
            if (!root.TryGetProperty("success", out JsonElement success) || !success.GetBoolean()) {
                string message = root.TryGetProperty("error", out JsonElement error)
                    ? error.GetString() ?? "unknown native error"
                    : "malformed native response";
                throw new InvalidOperationException($"native training bridge failed: {message}");
            }
            return root.TryGetProperty("result", out JsonElement result)
                ? result.Clone()
                : default;
        } finally {
            FreeBuffer(buffer);
        }
    }

    private static IntPtr ResolveLibrary(string libraryName, Assembly assembly, DllImportSearchPath? searchPath) {
        if (!string.Equals(libraryName, LibraryName, StringComparison.Ordinal)) return IntPtr.Zero;
        if (nativeHandle != IntPtr.Zero) return nativeHandle;
        string? candidate = NativeLibraryPath(assembly);
        if (candidate is null) return IntPtr.Zero;
        try {
            nativeHandle = NativeLibrary.Load(candidate);
            return nativeHandle;
        } catch (Exception error) {
            nativeLoadError = $"resolver failed to load native training bridge at {candidate}: {error}";
            LogError(nativeLoadError);
            return IntPtr.Zero;
        }
    }

    private static string? NativeLibraryPath(Assembly assembly) {
        string? codeDirectory = configuredNativeDirectory ?? Path.GetDirectoryName(assembly.Location);
        if (string.IsNullOrWhiteSpace(codeDirectory)) return null;
        string candidate = Path.Combine(codeDirectory, RuntimeInformation.IsOSPlatform(OSPlatform.Windows)
            ? "celeste_gym_native.dll"
            : RuntimeInformation.IsOSPlatform(OSPlatform.OSX)
                ? "libceleste_gym_native.dylib"
                : "libceleste_gym_native.so");
        return File.Exists(candidate) ? Path.GetFullPath(candidate) : null;
    }

    private static void LogInfo(string message) {
#if CELESTE_GYM_TEST
        System.Diagnostics.Trace.WriteLine(message);
#else
        Logger.Log(LogLevel.Info, "CelesteGymTraining", message);
#endif
    }

    private static void LogError(string message) {
#if CELESTE_GYM_TEST
        System.Diagnostics.Trace.WriteLine(message);
#else
        Logger.Log(LogLevel.Error, "CelesteGymTraining", message);
#endif
    }

    [StructLayout(LayoutKind.Sequential)]
    private readonly struct NativeBuffer {
        public readonly IntPtr Data;
        public readonly nuint Length;
        public readonly nuint Capacity;
    }

    private enum NativeOperation {
        CacheMap,
        FuzzSearch,
        EntryCheck
    }

    [DllImport(LibraryName, EntryPoint = "celeste_gym_cache_map_json", CallingConvention = CallingConvention.Cdecl)]
    private static extern NativeBuffer CacheMapJson(byte[] json, nuint jsonLength);

    [DllImport(LibraryName, EntryPoint = "celeste_gym_fuzz_search_json", CallingConvention = CallingConvention.Cdecl)]
    private static extern NativeBuffer FuzzSearchJson(
        byte[] snapshotJson,
        nuint snapshotLength,
        byte[] fuzzJson,
        nuint fuzzLength
    );

    [DllImport(LibraryName, EntryPoint = "celeste_gym_training_entry_check_json", CallingConvention = CallingConvention.Cdecl)]
    private static extern NativeBuffer TrainingEntryCheckJson(
        byte[] snapshotJson,
        nuint snapshotLength,
        byte[] checksJson,
        nuint checksLength
    );

    [DllImport(LibraryName, EntryPoint = "celeste_gym_buffer_free", CallingConvention = CallingConvention.Cdecl)]
    private static extern void FreeBuffer(NativeBuffer buffer);
}
