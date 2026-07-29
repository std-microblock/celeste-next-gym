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
    private static readonly Dictionary<char, Glyph> Glyphs = [];
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
        Vector2 size = MeasureUnscaled(text);
        return size * scale;
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
        Vector2 measured = MeasureUnscaled(text);
        Vector2 origin = measured * justify;
        Vector2 cursor = Vector2.Zero;

        foreach (char character in text) {
            if (character == '\n') {
                cursor.X = 0f;
                cursor.Y += LineHeight;
                continue;
            }

            Glyph glyph = GetGlyph(character);
            Vector2 drawAt = position + (cursor + glyph.Offset) * scale - origin * scale;
            if (glyph.Texture is not null) {
                if (stroke > 0f) DrawOutline(glyph.Texture, drawAt, scale, stroke, Color.Black);
                Monocle.Draw.SpriteBatch.Draw(glyph.Texture, drawAt, null, color, 0f, Vector2.Zero, scale, SpriteEffects.None, 0f);
            }
            cursor.X += glyph.Advance;
        }
    }

    public static void Dispose() {
        foreach (Glyph glyph in Glyphs.Values) glyph.Texture?.Dispose();
        Glyphs.Clear();
        stringFormat?.Dispose();
        stringFormat = null;
        drawingFont?.Dispose();
        drawingFont = null;
        fontCollection?.Dispose();
        fontCollection = null;
        if (pinnedFontBytes.IsAllocated) pinnedFontBytes.Free();
        fontBytes = null;
        prepared = false;
    }

    private static Vector2 MeasureUnscaled(string text) {
        if (string.IsNullOrEmpty(text)) return Vector2.Zero;
        float width = 0f;
        float lineWidth = 0f;
        float height = LineHeight;
        foreach (char character in text) {
            if (character == '\n') {
                width = Math.Max(width, lineWidth);
                lineWidth = 0f;
                height += LineHeight;
            } else {
                lineWidth += GetGlyph(character).Advance;
            }
        }
        return new Vector2(Math.Max(width, lineWidth), height);
    }

    private static Glyph GetGlyph(char character) {
        if (Glyphs.TryGetValue(character, out Glyph? cached)) return cached;
        if (drawingFont is null || stringFormat is null) throw new InvalidOperationException("Chinese UI font was not prepared.");

        if (character == '\t') return Glyphs[character] = new Glyph(null, FontPixelSize * 2f, Vector2.Zero);
        string text = character.ToString();
        float advance;
        using (DrawingBitmap measurement = new(1, 1, PixelFormat.Format32bppArgb))
        using (DrawingGraphics graphics = DrawingGraphics.FromImage(measurement)) {
            graphics.PageUnit = DrawingGraphicsUnit.Pixel;
            advance = Math.Max(1f, graphics.MeasureString(text, drawingFont, DrawingPointF.Empty, stringFormat).Width);
        }
        if (char.IsWhiteSpace(character)) return Glyphs[character] = new Glyph(null, advance, Vector2.Zero);

        const int padding = 4;
        int width = Math.Max(1, (int) Math.Ceiling(advance) + padding * 2);
        int height = (int) LineHeight + padding * 2;
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
            graphics.DrawString(text, drawingFont, DrawingBrushes.White, new DrawingPointF(padding, padding), stringFormat);
        }

        using MemoryStream png = new();
        bitmap.Save(png, ImageFormat.Png);
        png.Position = 0;
        Texture2D texture = Texture2D.FromStream(Engine.Graphics.GraphicsDevice, png);
        Glyph result = new(texture, advance, new Vector2(-padding, -padding));
        Glyphs.Add(character, result);
        return result;
    }

    private static void DrawOutline(Texture2D texture, Vector2 position, float scale, float stroke, Color color) {
        for (int y = -1; y <= 1; y++) {
            for (int x = -1; x <= 1; x++) {
                if (x == 0 && y == 0) continue;
                Monocle.Draw.SpriteBatch.Draw(
                    texture,
                    position + new Vector2(x, y) * stroke,
                    null,
                    color,
                    0f,
                    Vector2.Zero,
                    scale,
                    SpriteEffects.None,
                    0f
                );
            }
        }
    }

    private sealed record Glyph(Texture2D? Texture, float Advance, Vector2 Offset);
}
