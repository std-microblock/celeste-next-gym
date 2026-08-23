using Microsoft.Xna.Framework;
using Monocle;

namespace Celeste.Mod.MicroblocksQolUtils;

[Tracked]
public sealed class QolHud : Entity {
    public QolHud() {
        Tag = Tags.HUD | Tags.Global | Tags.PauseUpdate | Tags.TransitionUpdate;
        Depth = -1_000_000;
    }

    public override void Update() {
        base.Update();
        if (!MicroblocksQolUtilsModule.Settings.Enabled) return;
        MiaoNetBridge.Update(Scene as Level);
    }

    public override void Render() {
        base.Render();
        QolSettings settings = MicroblocksQolUtilsModule.Settings;
        if (!settings.Enabled) return;

        if (Scene is Level level) MiniMapRenderer.Render(level);

        if (settings.ShowFps) {
            string text = $"{FrameProfiler.FramesPerSecond,3:0} FPS  {FrameProfiler.LastFrameMilliseconds,5:0.0} ms";
            SystemTtfFont.Draw(text, new Vector2(18f, 16f), Vector2.Zero, 0.43f, Color.White, 1.5f);
        }
    }
}
