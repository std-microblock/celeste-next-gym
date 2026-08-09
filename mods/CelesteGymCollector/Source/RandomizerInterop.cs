using System.Reflection;
using Microsoft.Xna.Framework;

namespace Celeste.Mod.CelesteGymCollector;

internal sealed record RandomizerGenerationOptions(
    string Seed,
    string Length = "Short",
    string Difficulty = "Normal"
);

internal sealed record RandomizerRoomManifest(string Name, int[] Bounds);

internal sealed record RandomizerAreaManifest(
    int AreaId,
    int AreaMode,
    string AreaSid,
    string StartRoom,
    IReadOnlyList<RandomizerRoomManifest> Rooms
);

internal static class RandomizerRequestPolicy {
    public const int MaxSeedLength = 64;

    private static readonly HashSet<string> AllowedLengths = new(StringComparer.Ordinal) {
        "Short",
        "Medium",
        "Long",
        "Enormous"
    };

    private static readonly HashSet<string> AllowedDifficulties = new(StringComparer.Ordinal) {
        "Easy",
        "Normal",
        "Hard",
        "Expert",
        "Master",
        "Perfect"
    };

    public static RandomizerGenerationOptions Validate(
        string? seed,
        string? length,
        string? difficulty
    ) {
        string normalizedSeed = seed?.Trim() ?? "";
        if (normalizedSeed.Length is < 1 or > MaxSeedLength) {
            throw new InvalidOperationException(
                $"randomizer_seed must contain 1 to {MaxSeedLength} characters"
            );
        }
        foreach (char character in normalizedSeed) {
            if (!char.IsAsciiLetterOrDigit(character) && character is not '-' and not '_') {
                throw new InvalidOperationException(
                    "randomizer_seed may only contain ASCII letters, digits, '-' and '_'"
                );
            }
        }

        string normalizedLength = string.IsNullOrWhiteSpace(length) ? "Short" : length.Trim();
        if (!AllowedLengths.Contains(normalizedLength)) {
            throw new InvalidOperationException(
                "randomizer_length must be Short, Medium, Long or Enormous"
            );
        }

        string normalizedDifficulty = string.IsNullOrWhiteSpace(difficulty)
            ? "Normal"
            : difficulty.Trim();
        if (!AllowedDifficulties.Contains(normalizedDifficulty)) {
            throw new InvalidOperationException(
                "randomizer_difficulty must be Easy, Normal, Hard, Expert, Master or Perfect"
            );
        }

        return new RandomizerGenerationOptions(
            normalizedSeed,
            normalizedLength,
            normalizedDifficulty
        );
    }
}

internal sealed class RandomizerReflectionApi {
    private const string SettingsTypeName =
        "Celeste.Mod.Randomizer.Interoperability.SettingsInterop";
    private const string GenerationTypeName =
        "Celeste.Mod.Randomizer.Interoperability.GenerationInterop";
    private const string ModuleTypeName = "Celeste.Mod.Randomizer.RandoModule";

    private readonly Type settingsType;
    private readonly Type generationType;
    private readonly FieldInfo areaHandoffField;
    private readonly FieldInfo mapBuilderField;
    private readonly MethodInfo ingestNewAreaMethod;

    internal RandomizerReflectionApi(Type settingsType, Type generationType, Type moduleType) {
        this.settingsType = settingsType;
        this.generationType = generationType;
        areaHandoffField = RequireField(moduleType, "AreaHandoff");
        mapBuilderField = RequireField(moduleType, "MapBuilder");
        ingestNewAreaMethod = moduleType.GetMethod(
            "IngestNewArea",
            BindingFlags.Static | BindingFlags.Public | BindingFlags.NonPublic
        ) ?? throw new InvalidOperationException(
            $"Randomizer interop contract is missing {moduleType.FullName}.IngestNewArea"
        );
        ValidateContract();
    }

    public static RandomizerReflectionApi Discover() {
        Type? settings = null;
        Type? generation = null;
        Type? module = null;
        foreach (Assembly assembly in AppDomain.CurrentDomain.GetAssemblies()) {
            settings ??= assembly.GetType(SettingsTypeName, throwOnError: false);
            generation ??= assembly.GetType(GenerationTypeName, throwOnError: false);
            module ??= assembly.GetType(ModuleTypeName, throwOnError: false);
            if (settings is not null && generation is not null && module is not null) break;
        }
        if (settings is null || generation is null || module is null) {
            throw new InvalidOperationException(
                "Randomizer is unavailable; install and enable Randomizer 0.8.4 or a compatible release"
            );
        }
        return new RandomizerReflectionApi(settings, generation, module);
    }

    public void Start(RandomizerGenerationOptions options) {
        object settings = Invoke(settingsType, "GetSettingsObject")
            ?? throw new InvalidOperationException("Randomizer returned a null settings object");
        Invoke(settingsType, "EnableVanillaMaps", settings);
        RequireTrue(settingsType, "SetSeedType", settings, "Custom");
        Invoke(settingsType, "SetSeed", settings, options.Seed);
        RequireTrue(settingsType, "SetAlgorithm", settings, "Pathway");
        RequireTrue(settingsType, "SetDashes", settings, "One");
        RequireTrue(settingsType, "SetLength", settings, options.Length);
        RequireTrue(settingsType, "SetDifficulty", settings, options.Difficulty);
        if (Invoke(generationType, "Generate", settings) is not true) {
            throw new InvalidOperationException(
                "Randomizer rejected generation because another build is active"
            );
        }
    }

    public bool GenerationInProgress() =>
        Invoke(generationType, "GenerationInProgress") is true;

    public bool ReadyToLaunch() => Invoke(generationType, "ReadyToLaunch") is true;

    public AreaKey GetGeneratedArea() {
        object? result = Invoke(generationType, "GetGeneratedArea");
        return result is AreaKey area
            ? area
            : throw new InvalidOperationException("Randomizer did not return a generated AreaKey");
    }

    public void PumpMainThreadHandoff() {
        object? handoff = areaHandoffField.GetValue(null);
        if (handoff is not null) {
            try {
                ingestNewAreaMethod.Invoke(null, [handoff]);
                areaHandoffField.SetValue(null, null);
            } catch (TargetInvocationException error) when (error.InnerException is not null) {
                throw new InvalidOperationException(
                    $"Randomizer area handoff failed: {error.InnerException.Message}",
                    error.InnerException
                );
            }
        }

        object? builder = mapBuilderField.GetValue(null);
        if (builder is null) return;
        MethodInfo check = RequireInstanceMethod(builder.GetType(), "Check", 0);
        try {
            if (check.Invoke(builder, null) is true) {
                if (builder is IDisposable disposable) disposable.Dispose();
                mapBuilderField.SetValue(null, null);
            }
        } catch (TargetInvocationException error) when (error.InnerException is not null) {
            throw new InvalidOperationException(
                $"Randomizer builder handoff failed: {error.InnerException.Message}",
                error.InnerException
            );
        }
    }

    internal static MethodInfo RequireMethod(Type type, string name, int parameterCount) {
        MethodInfo? method = type.GetMethods(BindingFlags.Static | BindingFlags.Public)
            .SingleOrDefault(candidate =>
                string.Equals(candidate.Name, name, StringComparison.Ordinal)
                && candidate.GetParameters().Length == parameterCount
            );
        return method ?? throw new InvalidOperationException(
            $"Randomizer interop contract is missing {type.FullName}.{name}/{parameterCount}"
        );
    }

    internal static FieldInfo RequireField(Type type, string name) =>
        type.GetField(name, BindingFlags.Static | BindingFlags.Public | BindingFlags.NonPublic)
        ?? throw new InvalidOperationException(
            $"Randomizer interop contract is missing {type.FullName}.{name}"
        );

    private static MethodInfo RequireInstanceMethod(Type type, string name, int parameterCount) {
        MethodInfo? method = type.GetMethods(BindingFlags.Instance | BindingFlags.Public)
            .SingleOrDefault(candidate =>
                string.Equals(candidate.Name, name, StringComparison.Ordinal)
                && candidate.GetParameters().Length == parameterCount
            );
        return method ?? throw new InvalidOperationException(
            $"Randomizer interop contract is missing {type.FullName}.{name}/{parameterCount}"
        );
    }

    private void ValidateContract() {
        RequireMethod(settingsType, "GetSettingsObject", 0);
        RequireMethod(settingsType, "EnableVanillaMaps", 1);
        RequireMethod(settingsType, "SetSeed", 2);
        RequireMethod(settingsType, "SetSeedType", 2);
        RequireMethod(settingsType, "SetAlgorithm", 2);
        RequireMethod(settingsType, "SetDashes", 2);
        RequireMethod(settingsType, "SetDifficulty", 2);
        RequireMethod(settingsType, "SetLength", 2);
        RequireMethod(generationType, "Generate", 1);
        RequireMethod(generationType, "GenerationInProgress", 0);
        RequireMethod(generationType, "ReadyToLaunch", 0);
        RequireMethod(generationType, "GetGeneratedArea", 0);
    }

    private static void RequireTrue(Type type, string method, params object?[] arguments) {
        if (Invoke(type, method, arguments) is not true) {
            throw new InvalidOperationException(
                $"Randomizer rejected {method}({string.Join(", ", arguments.Skip(1))})"
            );
        }
    }

    private static object? Invoke(Type type, string method, params object?[] arguments) {
        try {
            return RequireMethod(type, method, arguments.Length).Invoke(null, arguments);
        } catch (TargetInvocationException error) when (error.InnerException is not null) {
            throw new InvalidOperationException(
                $"Randomizer {method} failed: {error.InnerException.Message}",
                error.InnerException
            );
        }
    }
}

internal static class RandomizerManifestCapture {
    public static RandomizerAreaManifest Capture(AreaKey area) {
        AreaData areaData = AreaData.Get(area)
            ?? throw new InvalidOperationException($"generated randomizer area {area.ID} is unavailable");
        MapData map = areaData.Mode[(int) area.Mode]?.MapData
            ?? throw new InvalidOperationException($"generated randomizer area {area.ID} has no map data");
        LevelData start = map.StartLevel()
            ?? throw new InvalidOperationException($"generated randomizer area {area.ID} has no start room");
        List<RandomizerRoomManifest> rooms = map.Levels
            .Where(level => !level.Dummy)
            .OrderBy(level => level.Name, StringComparer.Ordinal)
            .Select(level => new RandomizerRoomManifest(
                level.Name,
                Bounds(level.Bounds)
            ))
            .ToList();
        return new RandomizerAreaManifest(
            area.ID,
            (int) area.Mode,
            areaData.SID,
            start.Name,
            rooms
        );
    }

    private static int[] Bounds(Rectangle bounds) =>
        [bounds.X, bounds.Y, bounds.Width, bounds.Height];
}
