using Microsoft.Xna.Framework;
using Monocle;

namespace Celeste.Mod.CelesteGymTraining;

public static class ChineseText {
    private static Language ChineseLanguage => Dialog.Languages["schinese"];
    private static float FontSize => ChineseLanguage.FontFaceSize;
    private static PixelFont? font;
    private static PixelFont Font => font ??= Fonts.Load(ChineseLanguage.FontFace)
        ?? throw new InvalidOperationException($"unable to load Chinese font face {ChineseLanguage.FontFace}");

    public static void Prepare() => _ = Font;

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
