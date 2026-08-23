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
            string text = $"{Engine.FPS,3} FPS  {FrameProfiler.LastFrameMilliseconds,5:0.0} ms CPU";
            SystemTtfFont.Draw(text, new Vector2(18f, 16f), Vector2.Zero, 0.43f, Color.White, 1.5f);
            if (settings.EnableFrameProfiler) FrameProfiler.RenderHud(new Vector2(18f, 48f));
        }
    }
}
