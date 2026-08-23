using Microsoft.Xna.Framework;
using Monocle;
using System.Runtime.CompilerServices;

namespace Celeste.Mod.MicroblocksQolUtils;

public static class MiniMapRenderer {
    private const float ScreenWidth = 1920f;
    private const float Margin = 22f;
    private static readonly ConditionalWeakTable<SolidTiles, SolidPointCache> SolidPoints = new();

    public static void Render(Level level) {
        QolSettings settings = MicroblocksQolUtilsModule.Settings;
        if (!settings.MiniMapEnabled) return;
        Player? player = level.Tracker.GetEntity<Player>();
        SolidTiles? solids = level.Tracker.GetEntity<SolidTiles>();
        if (player is null || solids is null) return;

        float size = settings.MiniMapSize;
        float radius = size / 2f;
        Vector2 center = new(ScreenWidth - Margin - radius, Margin + radius);
        float pixelsPerWorld = 0.24f * settings.MiniMapZoom;
        Color levelBackground = level.BackgroundColor;
        MaterialPalette palette = MaterialPalette.FromSeed(
            settings.MiniMapAdaptiveColors
                ? levelBackground
                : AreaData.Get(level.Session.Area)?.TitleBaseColor ?? new Color(126, 99, 184)
        );
        Color background = settings.MaterialYouInterface
            ? palette.SurfaceHigh * (settings.MiniMapBackgroundOpacity / 10f)
            : Color.Black * (settings.MiniMapBackgroundOpacity / 10f);
        if (settings.MiniMapBackground) {
            if (settings.MiniMapShape == MiniMapShape.Circle) FillCircle(center, radius, background);
            else if (settings.MaterialYouInterface)
                MaterialUi.RoundedRect(center.X - radius, center.Y - radius, size, size, 24f, background);
            else Draw.Rect(center.X - radius, center.Y - radius, size, size, background);
        }

        Color mapBackdrop = settings.MiniMapBackground
            ? CompositeOver(background, levelBackground)
            : levelBackground;
        Color terrainColor = settings.MiniMapAdaptiveColors
            ? AdaptiveForeground(mapBackdrop) * 0.9f
            : Color.SlateGray * 0.9f;
        DrawSolids(solids, player.Center, center, radius, pixelsPerWorld, settings.MiniMapShape, terrainColor);
        foreach (RemotePlayer remote in MiaoNetBridge.Players) {
            if (!settings.ShowMiaoNetPlayers) break;
            DrawRemote(remote, player.Center, center, radius, pixelsPerWorld, settings);
        }
        DrawLocalPlayer(center);

        Color border = settings.MiniMapAdaptiveColors
            ? AdaptiveForeground(mapBackdrop) * 0.8f
            : settings.MaterialYouInterface ? palette.Outline : Color.White * 0.8f;
        if (settings.MiniMapShape == MiniMapShape.Circle) Draw.Circle(center, radius, border, 64);
        else if (settings.MaterialYouInterface)
            MaterialUi.RoundedOutline(center.X - radius, center.Y - radius, size, size, 24f, 2f, border);
        else Draw.HollowRect(center.X - radius, center.Y - radius, size, size, border);

        List<string> data = [];
        if (settings.ShowRoomsRemaining) {
            int? rooms = RoomRouteCache.RoomsToGoal(level);
            data.Add(rooms is int count ? $"目标 {count} 面" : "目标 ? 面");
        }
        if (settings.ShowMapPlayerCount) data.Add($"{MiaoNetBridge.PlayersInMap} 人");
        if (settings.ShowClock) data.Add(DateTime.Now.ToString("HH:mm:ss"));
        if (data.Count > 0) {
            string text = string.Join("  ·  ", data);
            Vector2 textPosition = center + new Vector2(0f, radius + 10f);
            Color textColor = Color.White;
            float textOutline = 1.25f;
            if (settings.MaterialYouInterface) {
                Vector2 measured = SystemTtfFont.Measure(text, 0.42f);
                MaterialUi.AcrylicSurface(
                    textPosition.X - measured.X / 2f - 12f,
                    textPosition.Y - 5f,
                    measured.X + 24f,
                    measured.Y + 10f,
                    14f,
                    palette.SurfaceHigh * 0.92f,
                    palette.Outline
                );
                textColor = palette.OnSurface;
                textOutline = 0f;
            }
            SystemTtfFont.Draw(
                text,
                textPosition,
                new Vector2(0.5f, 0f),
                0.42f,
                textColor,
                textOutline
            );
        }
    }

    private static void DrawSolids(
        SolidTiles solids,
        Vector2 player,
        Vector2 center,
        float radius,
        float scale,
        MiniMapShape shape,
        Color color
    ) {
        float tileSize = Math.Max(1f, 8f * scale);
        foreach (Vector2 world in SolidPoints.GetValue(solids, static value => new SolidPointCache(value)).Points) {
            Vector2 point = center + (world - player) * scale;
            if (!Inside(point, center, radius - tileSize, shape)) continue;
            Draw.Rect(point.X - tileSize / 2f, point.Y - tileSize / 2f, tileSize + 0.5f, tileSize + 0.5f, color);
        }
    }

    private static void DrawRemote(
        RemotePlayer remote,
        Vector2 player,
        Vector2 center,
        float radius,
        float scale,
        QolSettings settings
    ) {
        Vector2 point = center + (remote.Position - player) * scale;
        if (!Inside(point, center, radius - 12f, settings.MiniMapShape)) return;
        Draw.Circle(point, 11f, Color.Black * 0.85f, 20);
        if (!MiaoNetBridge.TryDrawAvatar(remote.Id, point, 20f, Color.White)) {
            Draw.Circle(point, 8f, remote.Color, 18);
            string initial = remote.Name.Length == 0 ? "?" : remote.Name[..1];
            SystemTtfFont.Draw(initial, point + new Vector2(0f, -1f), new Vector2(0.5f), 0.27f, Color.White, 1f);
        }

        bool showName = settings.MiniMapNames == MiniMapNameMode.Everyone
            || settings.MiniMapNames == MiniMapNameMode.WatchedOnly && WatchList.Contains(remote.Name);
        if (showName) {
            SystemTtfFont.Draw(remote.Name, point + new Vector2(0f, 13f), new Vector2(0.5f, 0f), 0.25f, Color.White, 1f);
        }
    }

    private static void DrawLocalPlayer(Vector2 center) {
        Color color = Color.Cyan;
        Draw.Line(center + new Vector2(0f, -9f), center + new Vector2(-7f, 7f), color, 3f);
        Draw.Line(center + new Vector2(-7f, 7f), center + new Vector2(0f, 4f), color, 3f);
        Draw.Line(center + new Vector2(0f, 4f), center + new Vector2(7f, 7f), color, 3f);
        Draw.Line(center + new Vector2(7f, 7f), center + new Vector2(0f, -9f), color, 3f);
    }

    private static bool Inside(Vector2 point, Vector2 center, float radius, MiniMapShape shape) {
        Vector2 delta = point - center;
        return shape == MiniMapShape.Circle
            ? delta.LengthSquared() <= radius * radius
            : Math.Abs(delta.X) <= radius && Math.Abs(delta.Y) <= radius;
    }

    private static Color CompositeOver(Color source, Color destination) {
        float inverseAlpha = 1f - source.A / 255f;
        return new Color(
            (byte)Math.Clamp(source.R + destination.R * inverseAlpha, 0f, 255f),
            (byte)Math.Clamp(source.G + destination.G * inverseAlpha, 0f, 255f),
            (byte)Math.Clamp(source.B + destination.B * inverseAlpha, 0f, 255f),
            255
        );
    }

    private static Color AdaptiveForeground(Color background) {
        float luminance = RelativeLuminance(background);
        float whiteContrast = 1.05f / (luminance + 0.05f);
        float darkContrast = (luminance + 0.05f) / 0.05f;
        return whiteContrast >= darkContrast
            ? new Color(238, 244, 248)
            : new Color(28, 34, 39);
    }

    private static float RelativeLuminance(Color color) =>
        0.2126f * LinearChannel(color.R)
        + 0.7152f * LinearChannel(color.G)
        + 0.0722f * LinearChannel(color.B);

    private static float LinearChannel(byte channel) {
        float value = channel / 255f;
        return value <= 0.04045f
            ? value / 12.92f
            : MathF.Pow((value + 0.055f) / 1.055f, 2.4f);
    }

    private static void FillCircle(Vector2 center, float radius, Color color) {
        int rows = (int)MathF.Ceiling(radius);
        for (int y = -rows; y <= rows; y++) {
            float halfWidth = MathF.Sqrt(Math.Max(0f, radius * radius - y * y));
            Draw.Rect(center.X - halfWidth, center.Y + y, halfWidth * 2f, 1.5f, color);
        }
    }

    private sealed class SolidPointCache {
        public List<Vector2> Points { get; } = [];

        public SolidPointCache(SolidTiles solids) {
            for (int y = 0; y < solids.Grid.CellsY; y++) {
                for (int x = 0; x < solids.Grid.CellsX; x++) {
                    if (solids.Grid[x, y]) Points.Add(solids.Position + new Vector2(x * 8f + 4f, y * 8f + 4f));
                }
            }
        }
    }
}
