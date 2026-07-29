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
    private static readonly Dictionary<(char Character, int PixelSize), Glyph> Glyphs = [];
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
                if (stroke > 0f) DrawShadow(glyph.Texture, drawAt, stroke);
                Monocle.Draw.SpriteBatch.Draw(glyph.Texture, drawAt, null, color, 0f, Vector2.Zero, 1f, SpriteEffects.None, 0f);
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
        int pixelSize = Math.Max(8, (int) MathF.Round(FontPixelSize * Math.Max(0.01f, scale)));
        (char Character, int PixelSize) key = (character, pixelSize);
        if (Glyphs.TryGetValue(key, out Glyph? cached)) return cached;
        if (fontCollection is null || stringFormat is null) throw new InvalidOperationException("Chinese UI font was not prepared.");
        DrawingFont font = GetFont(pixelSize);

        if (character == '\t') return Glyphs[key] = new Glyph(null, pixelSize * 2f, Vector2.Zero);
        string text = character.ToString();
        float advance;
        using (DrawingBitmap measurement = new(1, 1, PixelFormat.Format32bppArgb))
        using (DrawingGraphics graphics = DrawingGraphics.FromImage(measurement)) {
            graphics.PageUnit = DrawingGraphicsUnit.Pixel;
            advance = Math.Max(1f, graphics.MeasureString(text, font, DrawingPointF.Empty, stringFormat).Width);
        }
        if (char.IsWhiteSpace(character)) return Glyphs[key] = new Glyph(null, advance, Vector2.Zero);

        int padding = Math.Max(2, (int) MathF.Ceiling(pixelSize / 12f));
        int width = Math.Max(1, (int) Math.Ceiling(advance) + padding * 2);
        int height = (int) MathF.Ceiling(ScaledLineHeight(pixelSize / FontPixelSize)) + padding * 2;
        using DrawingBitmap bitmap = new(width, height, PixelFormat.Format32bppArgb);
        bitmap.SetResolution(96f, 96f);
        using (DrawingGraphics graphics = DrawingGraphics.FromImage(bitmap)) {
            graphics.Clear(System.Drawing.Color.Transparent);
            graphics.PageUnit = DrawingGraphicsUnit.Pixel;
            graphics.CompositingMode = CompositingMode.SourceCopy;
            graphics.CompositingQuality = CompositingQuality.HighQuality;
            graphics.InterpolationMode = InterpolationMode.HighQualityBicubic;
            graphics.PixelOffsetMode = PixelOffsetMode.HighQuality;
            graphics.SmoothingMode = SmoothingMode.HighQuality;
            graphics.TextRenderingHint = TextRenderingHint.AntiAliasGridFit;
            graphics.DrawString(text, font, DrawingBrushes.White, new DrawingPointF(padding, padding), stringFormat);
        }

        using MemoryStream png = new();
        bitmap.Save(png, ImageFormat.Png);
        png.Position = 0;
        Texture2D texture = Texture2D.FromStream(Engine.Graphics.GraphicsDevice, png);
        Glyph result = new(texture, advance, new Vector2(-padding, -padding));
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

    private static void DrawShadow(Texture2D texture, Vector2 position, float strength) {
        // Noto is already antialiased. The old eight-direction opaque outline
        // was intended for Celeste's pixel font and turns these glyphs into
        // heavy black blobs. Keep only one soft, sub-pixel drop shadow.
        float offset = Math.Min(1.4f, 0.55f + strength * 0.18f);
        Monocle.Draw.SpriteBatch.Draw(
            texture,
            position + new Vector2(offset),
            null,
            Color.Black * 0.32f,
            0f,
            Vector2.Zero,
            1f,
            SpriteEffects.None,
            0f
        );
    }

    private sealed record Glyph(Texture2D? Texture, float Advance, Vector2 Offset);
}
