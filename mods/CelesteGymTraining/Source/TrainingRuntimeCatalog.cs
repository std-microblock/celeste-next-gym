using System.Text.Json;

namespace Celeste.Mod.CelesteGymTraining;

public sealed record TrainingTeachingStep(
    string Prompt,
    string OrderErrorTitle,
    string OrderErrorBody,
    string WindowErrorTitle,
    string WindowErrorBody
);

public sealed record TrainingFuzzInput(
    int Index,
    string Id,
    IReadOnlyList<string> Keys,
    JsonElement At,
    bool Verify,
    JsonElement HeldTime
);

public sealed record TrainingLessonDefinition(
    string Id,
    string Title,
    string Summary,
    string EntryInputId,
    string EntryHint,
    string EntryFailureTitle,
    string EntryFailureBody,
    JsonElement EntryChecks,
    JsonElement Fuzz,
    JsonElement InitialSnapshot,
    IReadOnlyList<TrainingFuzzInput> Inputs,
    IReadOnlyList<TrainingTeachingStep> Steps,
    IReadOnlyList<string> ObjectiveExpressions,
    bool SlowdownEnabled,
    int SlowdownRadiusFrames,
    float MinimumTimeMultiplier
) {
    public int EntryInputIndex => Inputs.FirstOrDefault(input =>
        input.Verify && string.Equals(input.Id, EntryInputId, StringComparison.Ordinal))?.Index ?? -1;
}

public sealed record TrainingRuntimeProject(
    string Id,
    string Room,
    JsonElement Map,
    IReadOnlyDictionary<string, TrainingLessonDefinition> Lessons,
    bool RequireAllModules
);

public static class TrainingRuntimeCatalog {
    private static IReadOnlyDictionary<string, TrainingRuntimeProject>? byRoom;

    public static TrainingRuntimeProject? ForRoom(string room) {
        byRoom ??= Load();
        return byRoom.GetValueOrDefault(room);
    }

    internal static void Reload() => byRoom = null;

    private static IReadOnlyDictionary<string, TrainingRuntimeProject> Load() {
        ModAsset? asset = Everest.Content.Get("CelesteGymTraining/training-catalog.json")
            ?? Everest.Content.Get("Content/CelesteGymTraining/training-catalog.json");
        if (asset is null) return new Dictionary<string, TrainingRuntimeProject>();
        try {
            using Stream stream = asset.Stream;
            using JsonDocument document = JsonDocument.Parse(stream);
            Dictionary<string, TrainingRuntimeProject> projects = new(StringComparer.Ordinal);
            foreach (JsonElement project in document.RootElement.GetProperty("projects").EnumerateArray()) {
                string id = project.GetProperty("id").GetString() ?? "";
                string room = project.GetProperty("room").GetString() ?? "";
                JsonElement training = project.GetProperty("training");
                Dictionary<string, TrainingLessonDefinition> lessons = new(StringComparer.Ordinal);
                foreach (JsonElement module in training.GetProperty("modules").EnumerateArray()) {
                    TrainingLessonDefinition lesson = ParseLesson(module);
                    lessons.Add(lesson.Id, lesson);
                }
                bool requireAll = training.GetProperty("finish")
                    .TryGetProperty("require_all_modules", out JsonElement require)
                    && require.GetBoolean();
                projects.Add(room, new TrainingRuntimeProject(
                    id,
                    room,
                    project.GetProperty("map").Clone(),
                    lessons,
                    requireAll
                ));
            }
            return projects;
        } catch (Exception error) {
            Logger.Log(LogLevel.Error, "CelesteGymTraining", $"Failed to parse runtime training catalog: {error}");
            return new Dictionary<string, TrainingRuntimeProject>();
        }
    }

    private static TrainingLessonDefinition ParseLesson(JsonElement module) {
        JsonElement tutorial = module.GetProperty("tutorial");
        JsonElement entry = tutorial.GetProperty("entry");
        JsonElement failure = entry.GetProperty("failure");
        JsonElement fuzz = tutorial.GetProperty("fuzz");
        List<TrainingFuzzInput> inputs = [];
        int inputIndex = 0;
        foreach (JsonElement input in fuzz.GetProperty("inputs").EnumerateArray()) {
            inputs.Add(new TrainingFuzzInput(
                inputIndex++,
                input.GetProperty("id").GetString() ?? "",
                input.GetProperty("keys").EnumerateArray()
                    .Select(key => key.GetString() ?? "")
                    .ToArray(),
                input.GetProperty("at").Clone(),
                !input.TryGetProperty("verify", out JsonElement verify) || verify.GetBoolean(),
                input.TryGetProperty("held_time", out JsonElement heldTime)
                    ? heldTime.Clone()
                    : default
            ));
        }
        List<TrainingTeachingStep> steps = [];
        foreach (JsonElement step in tutorial.GetProperty("teaching").GetProperty("steps").EnumerateArray()) {
            JsonElement order = step.GetProperty("order_error");
            JsonElement window = step.GetProperty("window_error");
            steps.Add(new TrainingTeachingStep(
                step.GetProperty("prompt").GetString() ?? "",
                order.GetProperty("title").GetString() ?? "动作顺序不正确",
                order.GetProperty("body").GetString() ?? "请按提示操作。",
                window.GetProperty("title").GetString() ?? "错过输入窗口",
                window.GetProperty("body").GetString() ?? "请调整输入时机。"
            ));
        }
        JsonElement autoSlowdown = tutorial.GetProperty("assist").GetProperty("auto_slowdown");
        List<string> objectiveExpressions = [];
        if (fuzz.TryGetProperty("checkpoints", out JsonElement checkpoints)) {
            foreach (JsonElement checkpoint in checkpoints.EnumerateArray()) {
                if (!checkpoint.TryGetProperty("objectives", out JsonElement objectives)) continue;
                objectiveExpressions.AddRange(objectives.EnumerateArray()
                    .Select(objective => objective.GetProperty("expression").GetString() ?? "")
                    .Where(expression => expression.Length > 0));
            }
        }
        if (fuzz.TryGetProperty("objectives", out JsonElement globalObjectives)) {
            objectiveExpressions.AddRange(globalObjectives.EnumerateArray()
                .Select(objective => objective.GetProperty("expression").GetString() ?? "")
                .Where(expression => expression.Length > 0));
        }
        return new TrainingLessonDefinition(
            module.GetProperty("id").GetString() ?? "",
            tutorial.GetProperty("title").GetString() ?? "训练",
            tutorial.GetProperty("summary").GetString() ?? "",
            entry.GetProperty("input_id").GetString() ?? "",
            entry.GetProperty("hint").GetString() ?? "按提示开始。",
            failure.GetProperty("title").GetString() ?? "动作不正确",
            failure.GetProperty("body").GetString() ?? "请按提示操作。",
            entry.GetProperty("check").Clone(),
            fuzz.Clone(),
            module.GetProperty("validation").GetProperty("initial_state").Clone(),
            inputs,
            steps,
            objectiveExpressions,
            autoSlowdown.GetProperty("enabled_by_default").GetBoolean(),
            autoSlowdown.GetProperty("radius_frames").GetInt32(),
            autoSlowdown.GetProperty("minimum_multiplier").GetSingle()
        );
    }
}

public sealed record TrainingVerifiedInput(int InputIndex, int Frame, IReadOnlyList<string> Keys);

public sealed record TrainingFuzzCandidate(
    IReadOnlyDictionary<string, int> Bindings,
    IReadOnlyList<TrainingVerifiedInput> VerifiedInputs,
    IReadOnlyList<double> ObjectiveValues,
    bool Successful
);

public sealed record TrainingFuzzResult(
    TrainingFuzzCandidate Best,
    IReadOnlyList<TrainingFuzzCandidate> Candidates,
    IReadOnlyList<TrainingFuzzCandidate> Evaluations,
    JsonElement ExactWindows,
    JsonElement CoverageReport,
    bool UsesFallback
) {
    public static TrainingFuzzResult Parse(JsonElement result) {
        TrainingFuzzCandidate[] candidates = result.GetProperty("candidates")
            .EnumerateArray()
            .Select(ParseCandidate)
            .Where(candidate => candidate is not null)
            .Cast<TrainingFuzzCandidate>()
            .ToArray();
        TrainingFuzzCandidate[] evaluations = result.TryGetProperty("evaluations", out JsonElement rawEvaluations)
            ? rawEvaluations.EnumerateArray()
                .Select(ParseCandidate)
                .Where(candidate => candidate is not null)
                .Cast<TrainingFuzzCandidate>()
                .ToArray()
            : [];
        TrainingFuzzCandidate? best = ParseCandidate(result.GetProperty("best"));
        bool usesFallback = best is null || candidates.Length == 0;
        best ??= evaluations
            .OrderByDescending(candidate => candidate.ObjectiveValues.FirstOrDefault(double.NegativeInfinity))
            .FirstOrDefault();
        if (best is null) throw new InvalidOperationException("Fuzz 没有返回候选或评估轨迹。 ");
        if (candidates.Length == 0) candidates = [best];
        return new TrainingFuzzResult(
            best,
            candidates,
            evaluations,
            result.GetProperty("exact_windows").Clone(),
            result.TryGetProperty("coverage_report", out JsonElement coverage)
                ? coverage.Clone()
                : default,
            usesFallback
        );
    }

    private static TrainingFuzzCandidate? ParseCandidate(JsonElement element) {
        if (element.ValueKind is JsonValueKind.Null or JsonValueKind.Undefined) return null;
        Dictionary<string, int> bindings = element.GetProperty("bindings")
            .EnumerateObject()
            .ToDictionary(property => property.Name, property => property.Value.GetInt32(), StringComparer.Ordinal);
        TrainingVerifiedInput[] inputs = element.GetProperty("verified_inputs")
            .EnumerateArray()
            .Select(input => new TrainingVerifiedInput(
                input.GetProperty("input_index").GetInt32(),
                input.GetProperty("frame").GetInt32(),
                input.GetProperty("keys").EnumerateArray().Select(key => key.GetString() ?? "").ToArray()
            ))
            .ToArray();
        return new TrainingFuzzCandidate(
            bindings,
            inputs,
            element.GetProperty("objective_values").EnumerateArray().Select(value => value.GetDouble()).ToArray(),
            element.GetProperty("successful").GetBoolean()
        );
    }
}
