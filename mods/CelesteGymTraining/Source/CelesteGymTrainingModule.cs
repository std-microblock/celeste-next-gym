using Monocle;

namespace Celeste.Mod.CelesteGymTraining;

public sealed class CelesteGymTrainingModule : EverestModule {
    public const string AreaSid = "CelesteGymTraining/Training";
    public const string TrainingActiveFlag = "celeste_gym_training_active";
    private const string PreviewFullscreenEnvironment = "CELESTE_GYM_PREVIEW_FULLSCREEN";
    private static bool previewDisplayApplied;

    public override void Load() {
        TrainingNative.Initialize(Path.GetDirectoryName(Metadata.DLL));
        On.Celeste.Player.Update += PlayerUpdate;
        On.Celeste.Input.GetAimVector += GetAimVector;
        Everest.Events.Level.OnLoadLevel += OnLoadLevel;
    }

    public override void Unload() {
        Everest.Events.Level.OnLoadLevel -= OnLoadLevel;
        On.Celeste.Player.Update -= PlayerUpdate;
        On.Celeste.Input.GetAimVector -= GetAimVector;
        ChineseText.Dispose();
    }

    private static void PlayerUpdate(On.Celeste.Player.orig_Update orig, Player player) {
        TrainingRuntimeController? controller = player.Scene?.Tracker.GetEntity<TrainingRuntimeController>();
        controller?.BeforePlayerUpdate(player);
        orig(player);
        controller?.AfterPlayerUpdate(player);
    }

    private static Microsoft.Xna.Framework.Vector2 GetAimVector(
        On.Celeste.Input.orig_GetAimVector orig,
        Facings defaultFacing
    ) {
        if (Engine.Scene is Level level
            && level.Tracker.GetEntity<TrainingRuntimeController>() is { } controller
            && controller.TryGetDemoAim(defaultFacing, out Microsoft.Xna.Framework.Vector2 aim)) {
            return aim;
        }
        return orig(defaultFacing);
    }

    private static void OnLoadLevel(Level level, Player.IntroTypes playerIntro, bool isFromLoader) {
        if (!string.Equals(level.Session.Area.SID, AreaSid, StringComparison.Ordinal)) return;
        ApplyPreviewDisplayMode();
        TrainingBindings.ApplyDefaultsIfVanilla();
        if (level.Session.GetFlag(TrainingActiveFlag)) {
            if (level.Tracker.GetEntity<TrainingRuntimeController>() is null
                && TrainingRuntimeCatalog.ForRoom(level.Session.Level) is { } project) {
                level.Add(new TrainingRuntimeController(level, project));
            }
            return;
        }
        if (level.Tracker.GetEntity<TrainingProjectMenu>() is not null) return;
        level.Add(new TrainingProjectMenu(level, TrainingCatalog.Projects));
    }

    private static void ApplyPreviewDisplayMode() {
        if (previewDisplayApplied
            || !string.Equals(Environment.GetEnvironmentVariable(PreviewFullscreenEnvironment), "1", StringComparison.Ordinal)) {
            return;
        }

        previewDisplayApplied = true;
        Settings.Instance.Fullscreen = true;
        Settings.Instance.ViewportPadding = 0;
        Engine.ViewPadding = 0;
        Microsoft.Xna.Framework.Graphics.DisplayMode display =
            Engine.Graphics.GraphicsDevice.Adapter.CurrentDisplayMode;
        Logger.Log(LogLevel.Info, "CelesteGymTraining", $"Sizing preview to fullscreen {display.Width}x{display.Height}");
        Engine.SetFullscreen();
    }
}
