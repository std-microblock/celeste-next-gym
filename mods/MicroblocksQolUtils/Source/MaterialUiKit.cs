using Microsoft.Xna.Framework;

namespace Celeste.Mod.MicroblocksQolUtils;

internal enum MaterialTextRole {
    Display,
    Title,
    Section,
    Body,
    Label,
    Caption
}

internal readonly record struct MaterialRect(float X, float Y, float Width, float Height) {
    public Vector2 Center => new(X + Width / 2f, Y + Height / 2f);
    public bool Contains(Vector2 point) => MaterialUi.Contains(point, X, Y, Width, Height);
}

/// <summary>
/// Shared Material You widget primitives. Chapter select and the in-level settings
/// overlay use the same typography hierarchy, surfaces, focus state and AA geometry.
/// </summary>
internal static class MaterialUiKit {
    public static void Surface(
        MaterialRect rect,
        float radius,
        MaterialPalette palette,
        float alpha = 1f,
        bool elevated = true
    ) {
        if (elevated) {
            MaterialUi.AcrylicSurface(
                rect.X, rect.Y, rect.Width, rect.Height, radius,
                palette.SurfaceHigh * alpha, palette.Outline * alpha
            );
        } else {
            MaterialUi.RoundedRect(rect.X, rect.Y, rect.Width, rect.Height, radius,
                palette.Surface * alpha);
        }
    }

    public static void Card(
        MaterialRect rect,
        MaterialPalette palette,
        bool selected,
        float alpha = 1f
    ) {
        Color fill = selected ? palette.SurfaceHighest : palette.SurfaceHigh * 0.90f;
        Color outline = selected ? palette.Primary : palette.Outline;
        MaterialUi.RoundedRect(rect.X, rect.Y + 6f, rect.Width, rect.Height, 30f,
            Color.Black * 0.18f * alpha);
        MaterialUi.RoundedRect(rect.X, rect.Y, rect.Width, rect.Height, 30f, fill * alpha);
        MaterialUi.RoundedOutline(rect.X, rect.Y, rect.Width, rect.Height, 30f,
            selected ? 3f : 1f, outline * alpha);
    }

    public static void NavigationPill(
        MaterialRect rect,
        MaterialPalette palette,
        bool selected,
        float alpha = 1f
    ) {
        if (!selected) return;
        MaterialUi.RoundedRect(rect.X, rect.Y, rect.Width, rect.Height, rect.Height / 2f,
            palette.Primary * 0.92f * alpha);
    }

    public static void Chip(
        string text,
        Vector2 rightTop,
        MaterialPalette palette,
        bool selected,
        float alpha = 1f
    ) {
        const float scale = 0.27f;
        float width = Math.Max(82f, SystemTtfFont.Measure(text, scale, UiFontWeight.Bold).X + 30f);
        MaterialUi.RoundedRect(rightTop.X - width, rightTop.Y, width, 32f, 16f,
            (selected ? palette.Primary : palette.SurfaceHighest) * alpha);
        Text(text, new Vector2(rightTop.X - width / 2f, rightTop.Y + 5f),
            new Vector2(0.5f, 0f), MaterialTextRole.Label,
            selected ? palette.OnPrimary : palette.OnSurfaceVariant, alpha, scaleOverride: scale);
    }

    public static void Text(
        string text,
        Vector2 position,
        Vector2 justify,
        MaterialTextRole role,
        Color color,
        float alpha = 1f,
        float? scaleOverride = null
    ) {
        (float scale, UiFontWeight weight) = role switch {
            MaterialTextRole.Display => (0.88f, UiFontWeight.Bold),
            MaterialTextRole.Title => (0.48f, UiFontWeight.Bold),
            MaterialTextRole.Section => (0.46f, UiFontWeight.Bold),
            MaterialTextRole.Body => (0.38f, UiFontWeight.Regular),
            MaterialTextRole.Label => (0.31f, UiFontWeight.Bold),
            _ => (0.31f, UiFontWeight.Regular)
        };
        SystemTtfFont.Draw(text, position, justify, scaleOverride ?? scale, color * alpha, weight: weight);
    }

    public static void Cursor(Vector2 position, MaterialPalette palette, float alpha) {
        MaterialUi.Circle(position, 10f, Color.Black * 0.55f * alpha);
        MaterialUi.Circle(position, 7f, palette.Primary * alpha);
    }
}
