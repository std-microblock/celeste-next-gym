using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;
using Monocle;

namespace Celeste.Mod.MicroblocksQolUtils;

internal readonly record struct MaterialPalette(
    Color Scrim,
    Color Surface,
    Color SurfaceHigh,
    Color SurfaceHighest,
    Color Primary,
    Color OnPrimary,
    Color OnSurface,
    Color OnSurfaceVariant,
    Color Outline
) {
    public static MaterialPalette FromSeed(Color seed) {
        Color primary = Mix(seed, Color.White, 0.24f);
        Color surface = Mix(new Color(20, 18, 24), seed, 0.11f);
        return new MaterialPalette(
            new Color(8, 7, 11) * 0.48f,
            surface * 0.90f,
            Mix(surface, Color.White, 0.08f) * 0.94f,
            Mix(surface, Color.White, 0.14f) * 0.97f,
            primary,
            Luminance(primary) > 0.58f ? new Color(28, 24, 31) : Color.White,
            new Color(245, 239, 249),
            new Color(205, 195, 211),
            new Color(146, 136, 151) * 0.72f
        );
    }

    private static Color Mix(Color from, Color to, float amount) => new(
        (byte)MathHelper.Lerp(from.R, to.R, amount),
        (byte)MathHelper.Lerp(from.G, to.G, amount),
        (byte)MathHelper.Lerp(from.B, to.B, amount),
        (byte)MathHelper.Lerp(from.A, to.A, amount)
    );

    private static float Luminance(Color value) =>
        (value.R * 0.2126f + value.G * 0.7152f + value.B * 0.0722f) / 255f;
}

internal static class MaterialUi {
    private static readonly Dictionary<(int Width, int Height, int Radius), Texture2D> RoundedMasks = [];
    private static readonly Dictionary<(int Width, int Height, int Radius, int Thickness), Texture2D> OutlineMasks = [];
    private static Texture2D? noise;

    public static void RoundedRect(float x, float y, float width, float height, float radius, Color color) {
        int pixelWidth = Math.Max(1, (int)MathF.Round(width));
        int pixelHeight = Math.Max(1, (int)MathF.Round(height));
        int pixelRadius = Math.Clamp((int)MathF.Round(radius), 0, Math.Min(pixelWidth, pixelHeight) / 2);
        Texture2D mask = GetRoundedMask(pixelWidth, pixelHeight, pixelRadius);
        Draw.SpriteBatch.Draw(mask, new Rectangle((int)x, (int)y, pixelWidth, pixelHeight), color);
    }

    public static void RoundedOutline(
        float x,
        float y,
        float width,
        float height,
        float radius,
        float thickness,
        Color color
    ) {
        int pixelWidth = Math.Max(1, (int)MathF.Round(width));
        int pixelHeight = Math.Max(1, (int)MathF.Round(height));
        int pixelRadius = Math.Clamp((int)MathF.Round(radius), 0, Math.Min(pixelWidth, pixelHeight) / 2);
        int pixelThickness = Math.Clamp((int)MathF.Round(thickness), 1, Math.Min(pixelWidth, pixelHeight) / 2);
        Texture2D mask = GetOutlineMask(pixelWidth, pixelHeight, pixelRadius, pixelThickness);
        Draw.SpriteBatch.Draw(mask, new Rectangle((int)x, (int)y, pixelWidth, pixelHeight), color);
    }

    public static void AcrylicSurface(
        float x,
        float y,
        float width,
        float height,
        float radius,
        Color tint,
        Color outline,
        bool grain = true
    ) {
        RoundedRect(x, y + 8f, width, height, radius, Color.Black * 0.24f);
        RoundedRect(x, y, width, height, radius, tint);
        if (grain) DrawNoise(x, y, width, height, radius);
        RoundedOutline(x, y, width, height, radius, 2f, outline);
    }

    public static bool Contains(Vector2 point, float x, float y, float width, float height) =>
        point.X >= x && point.X <= x + width && point.Y >= y && point.Y <= y + height;

    public static void Dispose() {
        foreach (Texture2D texture in RoundedMasks.Values) texture.Dispose();
        RoundedMasks.Clear();
        foreach (Texture2D texture in OutlineMasks.Values) texture.Dispose();
        OutlineMasks.Clear();
        noise?.Dispose();
        noise = null;
    }

    private static Texture2D GetRoundedMask(int width, int height, int radius) {
        var key = (width, height, radius);
        if (RoundedMasks.TryGetValue(key, out Texture2D? texture)) return texture;
        Color[] pixels = new Color[width * height];
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                pixels[y * width + x] = InsideRounded(x, y, width, height, radius)
                    ? Color.White
                    : Color.Transparent;
            }
        }
        texture = new Texture2D(Engine.Graphics.GraphicsDevice, width, height);
        texture.SetData(pixels);
        RoundedMasks[key] = texture;
        return texture;
    }

    private static Texture2D GetOutlineMask(int width, int height, int radius, int thickness) {
        var key = (width, height, radius, thickness);
        if (OutlineMasks.TryGetValue(key, out Texture2D? texture)) return texture;
        Color[] pixels = new Color[width * height];
        int innerWidth = Math.Max(0, width - thickness * 2);
        int innerHeight = Math.Max(0, height - thickness * 2);
        int innerRadius = Math.Max(0, radius - thickness);
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                bool outer = InsideRounded(x, y, width, height, radius);
                bool inner = innerWidth > 0 && innerHeight > 0
                    && InsideRounded(x - thickness, y - thickness, innerWidth, innerHeight, innerRadius);
                pixels[y * width + x] = outer && !inner ? Color.White : Color.Transparent;
            }
        }
        texture = new Texture2D(Engine.Graphics.GraphicsDevice, width, height);
        texture.SetData(pixels);
        OutlineMasks[key] = texture;
        return texture;
    }

    private static bool InsideRounded(int x, int y, int width, int height, int radius) {
        if (x < 0 || y < 0 || x >= width || y >= height) return false;
        if (radius <= 0) return true;
        float left = radius - 0.5f;
        float right = width - radius - 0.5f;
        float top = radius - 0.5f;
        float bottom = height - radius - 0.5f;
        float nearestX = Math.Clamp(x, left, right);
        float nearestY = Math.Clamp(y, top, bottom);
        float dx = x - nearestX;
        float dy = y - nearestY;
        return dx * dx + dy * dy <= radius * radius;
    }

    private static void DrawNoise(float x, float y, float width, float height, float radius) {
        noise ??= CreateNoise();
        // The rounded tint beneath it provides the actual clip silhouette; very low alpha keeps
        // the tiled grain from making the corners visibly rectangular.
        Draw.SpriteBatch.Draw(
            noise,
            new Rectangle((int)(x + radius / 2f), (int)(y + radius / 2f),
                Math.Max(1, (int)(width - radius)), Math.Max(1, (int)(height - radius))),
            new Rectangle(0, 0, noise.Width, noise.Height),
            Color.White * 0.035f
        );
    }

    private static Texture2D CreateNoise() {
        const int size = 96;
        Color[] pixels = new Color[size * size];
        uint state = 0xA341316Cu;
        for (int index = 0; index < pixels.Length; index++) {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            byte value = (byte)(100 + state % 156);
            pixels[index] = new Color(value, value, value, 255);
        }
        Texture2D texture = new(Engine.Graphics.GraphicsDevice, size, size);
        texture.SetData(pixels);
        return texture;
    }
}
