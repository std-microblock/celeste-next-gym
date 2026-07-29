using Microsoft.Xna.Framework;
using Monocle;

namespace Celeste.Mod.CelesteGymTraining;

public static class ChineseText {
    private const float FontSize = 64f;
    private static PixelFont? font;

    private static PixelFont Font => font ??= Fonts.Load("chinese");

    public static Vector2 Measure(string text, float scale) => Font.Get(FontSize).Measure(text) * scale;

    public static void Draw(
        string text,
        Vector2 position,
        Vector2 justify,
        float scale,
        Color color,
        float stroke = 2f
    ) {
        Font.DrawOutline(
            FontSize,
            text,
            position,
            justify,
            Vector2.One * scale,
            color,
            stroke,
            Color.Black
        );
    }
}
