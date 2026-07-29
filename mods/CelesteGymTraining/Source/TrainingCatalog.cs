namespace Celeste.Mod.CelesteGymTraining;

public static class TrainingCatalog {
    private static readonly IReadOnlyList<TrainingProjectOption> fallback = [
        new("playground", "训练项目", "进入训练房间，按提示完成动作。", "playground")
    ];

    public static IReadOnlyList<TrainingProjectOption> Projects => fallback;
}
