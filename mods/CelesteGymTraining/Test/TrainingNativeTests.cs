namespace Celeste.Mod.CelesteGymTraining.Test;

using System.Text.Json;
using Xunit;

public sealed class TrainingNativeTests {
    [Fact]
    public void ManagedBridgeCallsNativeFuzzAndEntryCheck() {
        using JsonDocument map = JsonDocument.Parse("""
            {
              "bounds": { "x": 0, "y": 0, "width": 320, "height": 184 },
              "spawn": { "x": 8, "y": 152 },
              "solids": [{ "x": 0, "y": 160, "width": 320, "height": 24 }],
              "entities": []
            }
            """);
        using JsonDocument snapshot = JsonDocument.Parse("""
            {
              "pos": { "x": 8, "y": 152 },
              "on_ground": true,
              "player_on_ground": true,
              "player_on_ground_initialized": true
            }
            """);
        using JsonDocument fuzz = JsonDocument.Parse("""
            {
              "version": 1,
              "inputs": [{ "id": "jump", "keys": ["jump"], "at": 0, "verify": true }],
              "variables": [],
              "observe_until": 2,
              "success": ["!current.dead"],
              "objectives": [],
              "search": { "bindings": {}, "output": ["best", "candidates"] }
            }
            """);
        using JsonDocument checks = JsonDocument.Parse("[\"!current.dead\"]");

        TrainingNative.CacheMap(map.RootElement);
        JsonElement result = TrainingNative.FuzzSearch(snapshot.RootElement, fuzz.RootElement);

        Assert.Equal(JsonValueKind.Object, result.GetProperty("best").ValueKind);
        Assert.NotEmpty(result.GetProperty("candidates").EnumerateArray());
        Assert.True(TrainingNative.EvaluateEntryChecks(snapshot.RootElement, checks.RootElement));
    }

    [Fact]
    public void EveryGeneratedWorkspaceLessonHasNativeCandidates() {
        string root = AppContext.BaseDirectory;
        while (!File.Exists(Path.Combine(root, "Cargo.toml"))) {
            root = Directory.GetParent(root)?.FullName
                ?? throw new InvalidOperationException("repository root was not found");
        }
        string catalogPath = Path.Combine(
            root,
            "mods",
            "CelesteGymTraining",
            "Content",
            "CelesteGymTraining",
            "training-catalog.json"
        );
        using JsonDocument catalog = JsonDocument.Parse(File.ReadAllText(catalogPath));
        foreach (JsonElement project in catalog.RootElement.GetProperty("projects").EnumerateArray()) {
            TrainingNative.CacheMap(project.GetProperty("map"));
            foreach (JsonElement module in project.GetProperty("training").GetProperty("modules").EnumerateArray()) {
                JsonElement result = TrainingNative.FuzzSearch(
                    module.GetProperty("validation").GetProperty("initial_state"),
                    module.GetProperty("tutorial").GetProperty("fuzz")
                );
                JsonElement best = result.GetProperty("best");
                JsonElement reference = best.ValueKind == JsonValueKind.Object
                    ? best
                    : result.GetProperty("evaluations").EnumerateArray().FirstOrDefault();
                Assert.Equal(JsonValueKind.Object, reference.ValueKind);
                Assert.NotEmpty(reference.GetProperty("verified_inputs").EnumerateArray());
            }
        }
    }
}
