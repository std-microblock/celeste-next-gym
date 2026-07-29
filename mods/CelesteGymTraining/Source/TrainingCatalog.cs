using System.Text.Json;

namespace Celeste.Mod.CelesteGymTraining;

public static class TrainingCatalog {
    private static readonly IReadOnlyList<TrainingProjectOption> fallback = [
        new("untitled-room", "未命名房间", "一张可实时测试的训练地图。", "untitled-room", "入门")
    ];
    private static IReadOnlyList<TrainingProjectOption>? projects;

    public static IReadOnlyList<TrainingProjectOption> Projects => projects ??= Load();

    internal static void Reload() => projects = null;

    private static IReadOnlyList<TrainingProjectOption> Load() {
        ModAsset? asset = Everest.Content.Get("CelesteGymTraining/training-catalog.json")
            ?? Everest.Content.Get("Content/CelesteGymTraining/training-catalog.json");
        if (asset is null) return fallback;
        try {
            using Stream stream = asset.Stream;
            CatalogDocument? document = JsonSerializer.Deserialize<CatalogDocument>(
                stream,
                new JsonSerializerOptions { PropertyNameCaseInsensitive = true }
            );
            if (document?.Version != 1 || document.Projects is not { Count: > 0 }) return fallback;
            return document.Projects
                .Where(project => !string.IsNullOrWhiteSpace(project.Id) && !string.IsNullOrWhiteSpace(project.Room))
                .Select(project => new TrainingProjectOption(
                    project.Id,
                    project.Title,
                    project.Summary,
                    project.Room,
                    string.IsNullOrWhiteSpace(project.Difficulty) ? "入门" : project.Difficulty,
                    project.Thumbnail
                ))
                .ToArray();
        } catch {
            return fallback;
        }
    }

    private sealed class CatalogDocument {
        public int Version { get; init; }
        public List<CatalogProject> Projects { get; init; } = [];
    }

    private sealed class CatalogProject {
        public string Id { get; init; } = "";
        public string Title { get; init; } = "";
        public string Summary { get; init; } = "";
        public string Room { get; init; } = "";
        public string Difficulty { get; init; } = "入门";
        public string? Thumbnail { get; init; }
    }
}
