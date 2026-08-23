using Monocle;

namespace Celeste.Mod.MicroblocksQolUtils;

public sealed class MicroblocksQolUtilsModule : EverestModule {
    public static MicroblocksQolUtilsModule Instance { get; private set; } = null!;
    public static QolSettings Settings => (QolSettings)Instance._Settings;

    public override Type SettingsType => typeof(QolSettings);

    public MicroblocksQolUtilsModule() {
        Instance = this;
    }

    public override void Load() {
        Everest.Events.Level.OnLoadLevel += OnLoadLevel;
        On.Monocle.Engine.Update += EngineUpdate;
    }

    public override void Unload() {
        On.Monocle.Engine.Update -= EngineUpdate;
        Everest.Events.Level.OnLoadLevel -= OnLoadLevel;
        MiaoNetBridge.Unload();
        SystemTtfFont.Dispose();
    }

    private static void OnLoadLevel(Level level, Player.IntroTypes intro, bool fromLoader) {
        if (level.Tracker.GetEntity<QolHud>() is null) level.Add(new QolHud());
    }

    private static void EngineUpdate(On.Monocle.Engine.orig_Update orig, Engine self, Microsoft.Xna.Framework.GameTime gameTime) {
        FrameProfiler.BeginFrame();
        try {
            orig(self, gameTime);
        } finally {
            FrameProfiler.EndFrame();
        }
    }
}
