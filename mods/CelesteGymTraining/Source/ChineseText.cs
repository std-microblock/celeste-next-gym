using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.Drawing.Text;
using System.Runtime.InteropServices;
using Microsoft.Xna.Framework;
using Microsoft.Xna.Framework.Graphics;
using Monocle;
using DrawingBitmap = System.Drawing.Bitmap;
using DrawingBrushes = System.Drawing.Brushes;
using DrawingFont = System.Drawing.Font;
using DrawingFontStyle = System.Drawing.FontStyle;
using DrawingGraphics = System.Drawing.Graphics;
using DrawingGraphicsUnit = System.Drawing.GraphicsUnit;
using DrawingPointF = System.Drawing.PointF;
using DrawingStringFormat = System.Drawing.StringFormat;
using DrawingStringFormatFlags = System.Drawing.StringFormatFlags;

#pragma warning disable CA1416 // Every System.Drawing entry point is guarded by Prepare's Windows check.

namespace Celeste.Mod.CelesteGymTraining;

/// <summary>
/// Draws UI text from the Noto Sans SC TTF shipped with this mod. Celeste's
/// built-in chinese.fnt only contains the characters used by the vanilla
/// dialog and silently drops any other glyph (for example, “擎”).
/// </summary>
public static class ChineseText {
    private const string FontAsset = "CelesteGymTraining/Fonts/NotoSansSC-Medium.ttf";
    private const float FontPixelSize = 48f;
    private const float LineHeight = 64f;
    private static readonly Dictionary<(char Character, int PixelSize, int ViewWidth), Glyph> Glyphs = [];
    private static readonly Dictionary<int, DrawingFont> Fonts = [];
    private static PrivateFontCollection? fontCollection;
    private static DrawingFont? drawingFont;
    private static DrawingStringFormat? stringFormat;
    private static byte[]? fontBytes;
    private static GCHandle pinnedFontBytes;
    private static bool prepared;

    public static void Prepare() {
        if (prepared) return;
        if (!OperatingSystem.IsWindows()) {
            throw new PlatformNotSupportedException("Noto Sans SC TTF rasterization currently requires Windows.");
        }

        ModAsset? asset = Everest.Content.Get(FontAsset)
            ?? Everest.Content.Get($"Content/{FontAsset}")
            ?? throw new FileNotFoundException($"missing bundled font asset: {FontAsset}");
        using (Stream stream = asset.Stream) {
            using MemoryStream copy = new();
            stream.CopyTo(copy);
            fontBytes = copy.ToArray();
        }
        if (fontBytes.Length == 0) throw new InvalidDataException($"bundled font asset is empty: {FontAsset}");

        pinnedFontBytes = GCHandle.Alloc(fontBytes, GCHandleType.Pinned);
        try {
            fontCollection = new PrivateFontCollection();
            fontCollection.AddMemoryFont(pinnedFontBytes.AddrOfPinnedObject(), fontBytes.Length);
            if (fontCollection.Families.Length == 0) throw new InvalidDataException("Noto Sans SC contains no font family.");
            drawingFont = new DrawingFont(
                fontCollection.Families[0],
                FontPixelSize,
                DrawingFontStyle.Regular,
                DrawingGraphicsUnit.Pixel
            );
            Fonts[(int) FontPixelSize] = drawingFont;
            stringFormat = (DrawingStringFormat) DrawingStringFormat.GenericTypographic.Clone();
            stringFormat.FormatFlags |= DrawingStringFormatFlags.MeasureTrailingSpaces;
            prepared = true;
            Logger.Log(LogLevel.Info, "CelesteGymTraining", $"Loaded bundled UI font {fontCollection.Families[0].Name} ({fontBytes.Length} bytes)");
        } catch {
            Dispose();
            throw;
        }
    }

    public static Vector2 Measure(string text, float scale) {
        Prepare();
        return MeasureAtScale(text, scale);
    }

    public static void Draw(
        string text,
        Vector2 position,
        Vector2 justify,
        float scale,
        Color color,
        float stroke = 2f
    ) {
        if (string.IsNullOrEmpty(text)) return;
        Prepare();
        Vector2 measured = MeasureAtScale(text, scale);
        Vector2 origin = measured * justify;
        Vector2 cursor = Vector2.Zero;
        float lineHeight = ScaledLineHeight(scale);

        foreach (char character in text) {
            if (character == '\n') {
                cursor.X = 0f;
                cursor.Y += lineHeight;
                continue;
            }

            Glyph glyph = GetGlyph(character, scale);
            Vector2 drawAt = position + cursor + glyph.Offset - origin;
            if (glyph.Texture is not null) {
                drawAt = new Vector2(
                    MathF.Round(drawAt.X * glyph.OutputScale) / glyph.OutputScale,
                    MathF.Round(drawAt.Y * glyph.OutputScale) / glyph.OutputScale
                );
                Monocle.Draw.SpriteBatch.Draw(
                    glyph.Texture,
                    drawAt,
                    null,
                    color,
                    0f,
                    Vector2.Zero,
                    glyph.TextureScale,
                    SpriteEffects.None,
                    0f
                );
            }
            cursor.X += glyph.Advance;
        }
    }

    public static void Dispose() {
        foreach (Glyph glyph in Glyphs.Values) glyph.Texture?.Dispose();
        Glyphs.Clear();
        foreach (DrawingFont font in Fonts.Values.Distinct()) font.Dispose();
        Fonts.Clear();
        stringFormat?.Dispose();
        stringFormat = null;
        drawingFont = null;
        fontCollection?.Dispose();
        fontCollection = null;
        if (pinnedFontBytes.IsAllocated) pinnedFontBytes.Free();
        fontBytes = null;
        prepared = false;
    }

    private static Vector2 MeasureAtScale(string text, float scale) {
        if (string.IsNullOrEmpty(text)) return Vector2.Zero;
        float width = 0f;
        float lineWidth = 0f;
        float lineHeight = ScaledLineHeight(scale);
        float height = lineHeight;
        foreach (char character in text) {
            if (character == '\n') {
                width = Math.Max(width, lineWidth);
                lineWidth = 0f;
                height += lineHeight;
            } else {
                lineWidth += GetGlyph(character, scale).Advance;
            }
        }
        return new Vector2(Math.Max(width, lineWidth), height);
    }

    private static Glyph GetGlyph(char character, float scale) {
        int viewWidth = Math.Max(1, Engine.ViewWidth);
        float outputScale = Math.Max(0.01f, viewWidth / 1920f);
        float textureScale = 1f / outputScale;
        int pixelSize = Math.Max(8, (int) MathF.Round(FontPixelSize * Math.Max(0.01f, scale) * outputScale));
        (char Character, int PixelSize, int ViewWidth) key = (character, pixelSize, viewWidth);
        if (Glyphs.TryGetValue(key, out Glyph? cached)) return cached;
        if (fontCollection is null || stringFormat is null) throw new InvalidOperationException("Chinese UI font was not prepared.");
        DrawingFont font = GetFont(pixelSize);

        if (character == '\t') {
            return Glyphs[key] = new Glyph(null, pixelSize * 2f * textureScale, Vector2.Zero, textureScale, outputScale);
        }
        string text = character.ToString();
        float advance;
        using (DrawingBitmap measurement = new(1, 1, PixelFormat.Format32bppArgb))
        using (DrawingGraphics graphics = DrawingGraphics.FromImage(measurement)) {
            graphics.PageUnit = DrawingGraphicsUnit.Pixel;
            advance = Math.Max(1f, graphics.MeasureString(text, font, DrawingPointF.Empty, stringFormat).Width);
        }
        if (char.IsWhiteSpace(character)) {
            return Glyphs[key] = new Glyph(null, advance * textureScale, Vector2.Zero, textureScale, outputScale);
        }

        int padding = Math.Max(2, (int) MathF.Ceiling(pixelSize / 12f));
        int width = Math.Max(1, (int) Math.Ceiling(advance) + padding * 2);
        int height = (int) MathF.Ceiling(ScaledLineHeight(pixelSize / FontPixelSize)) + padding * 2;
        using DrawingBitmap bitmap = new(width, height, PixelFormat.Format32bppArgb);
        bitmap.SetResolution(96f, 96f);
        using (DrawingGraphics graphics = DrawingGraphics.FromImage(bitmap)) {
            graphics.Clear(System.Drawing.Color.Black);
            graphics.PageUnit = DrawingGraphicsUnit.Pixel;
            graphics.CompositingMode = CompositingMode.SourceOver;
            graphics.CompositingQuality = CompositingQuality.HighQuality;
            graphics.PixelOffsetMode = PixelOffsetMode.HighQuality;
            graphics.SmoothingMode = SmoothingMode.HighQuality;
            graphics.TextRenderingHint = TextRenderingHint.ClearTypeGridFit;
            graphics.DrawString(text, font, DrawingBrushes.White, new DrawingPointF(padding, padding), stringFormat);
        }

        Texture2D texture = CreateClearTypeTexture(bitmap);
        Glyph result = new(
            texture,
            advance * textureScale,
            new Vector2(-padding * textureScale),
            textureScale,
            outputScale
        );
        Glyphs.Add(key, result);
        return result;
    }

    private static DrawingFont GetFont(int pixelSize) {
        if (Fonts.TryGetValue(pixelSize, out DrawingFont? font)) return font;
        if (fontCollection is null) throw new InvalidOperationException("Chinese UI font was not prepared.");
        font = new DrawingFont(fontCollection.Families[0], pixelSize, DrawingFontStyle.Regular, DrawingGraphicsUnit.Pixel);
        Fonts.Add(pixelSize, font);
        return font;
    }

    private static float ScaledLineHeight(float scale) => Math.Max(1f, LineHeight * scale);

    private static Texture2D CreateClearTypeTexture(DrawingBitmap bitmap) {
        System.Drawing.Rectangle bounds = new(0, 0, bitmap.Width, bitmap.Height);
        BitmapData data = bitmap.LockBits(bounds, ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
        try {
            int stride = Math.Abs(data.Stride);
            byte[] pixels = new byte[stride * bitmap.Height];
            Marshal.Copy(data.Scan0, pixels, 0, pixels.Length);
            Color[] colors = new Color[bitmap.Width * bitmap.Height];
            for (int y = 0; y < bitmap.Height; y++) {
                int sourceY = data.Stride < 0 ? bitmap.Height - 1 - y : y;
                int row = sourceY * stride;
                for (int x = 0; x < bitmap.Width; x++) {
                    int source = row + x * 4;
                    byte blue = pixels[source];
                    byte green = pixels[source + 1];
                    byte red = pixels[source + 2];
                    byte coverage = Median(red, green, blue);
                    colors[y * bitmap.Width + x] = new Color(coverage, coverage, coverage, coverage);
                }
            }

            Texture2D texture = new(Engine.Graphics.GraphicsDevice, bitmap.Width, bitmap.Height);
            texture.SetData(colors);
            return texture;
        } finally {
            bitmap.UnlockBits(data);
        }
    }

    private static byte Median(byte first, byte second, byte third) =>
        (byte) (first + second + third - Math.Min(first, Math.Min(second, third)) - Math.Max(first, Math.Max(second, third)));

    private sealed record Glyph(
        Texture2D? Texture,
        float Advance,
        Vector2 Offset,
        float TextureScale,
        float OutputScale
    );
}
