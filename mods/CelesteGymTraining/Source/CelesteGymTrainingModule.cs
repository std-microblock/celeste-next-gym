namespace Celeste.Mod.CelesteGymTraining;

public sealed class CelesteGymTrainingModule : EverestModule {
    public const string AreaSid = "CelesteGymTraining/Training";
    public const string TrainingActiveFlag = "celeste_gym_training_active";

    public override void Load() {
        Everest.Events.Level.OnLoadLevel += OnLoadLevel;
    }

    public override void Unload() {
        Everest.Events.Level.OnLoadLevel -= OnLoadLevel;
    }

    private static void OnLoadLevel(Level level, Player.IntroTypes playerIntro, bool isFromLoader) {
        if (!string.Equals(level.Session.Area.SID, AreaSid, StringComparison.Ordinal)) return;
        if (level.Session.GetFlag(TrainingActiveFlag)) return;
        if (level.Tracker.GetEntity<TrainingProjectMenu>() is not null) return;
        level.Add(new TrainingProjectMenu(level, TrainingCatalog.Projects));
    }
}
