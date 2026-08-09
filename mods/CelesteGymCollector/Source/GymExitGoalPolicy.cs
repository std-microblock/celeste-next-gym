using Microsoft.Xna.Framework;

namespace Celeste.Mod.CelesteGymCollector;

internal static class GymExitGoalPolicy {
    private const float BoundaryBand = 24f;

    public static string? NormalizeBoundary(string? boundary) {
        if (string.IsNullOrWhiteSpace(boundary)) return null;
        string normalized = boundary.Trim().ToLowerInvariant();
        if (normalized is not ("up" or "down" or "left" or "right")) {
            throw new InvalidOperationException(
                "goal_boundary must be one of up, down, left, or right"
            );
        }
        return normalized;
    }

    public static float[]? NormalizePair(float[]? pair, string name) {
        if (pair is null) return null;
        if (pair.Length != 2 || !float.IsFinite(pair[0]) || !float.IsFinite(pair[1])) {
            throw new InvalidOperationException($"{name} must contain two finite numbers");
        }
        if (name == "goal_aperture" && pair[1] <= pair[0]) {
            throw new InvalidOperationException("goal_aperture maximum must exceed minimum");
        }
        return [pair[0], pair[1]];
    }

    public static bool Matches(
        string? goalBoundary,
        float[]? goalAperture,
        float[]? goalWorld,
        Rectangle roomBounds,
        PlayerFrame player
    ) {
        if (goalBoundary is null) return true;
        if (player.Pos.Length < 2 || player.Speed.Length < 2) return false;

        float x = player.Pos[0];
        float y = player.Pos[1];
        string? exitedBoundary = InferBoundary(roomBounds, x, y, player.Speed[0], player.Speed[1]);
        if (!string.Equals(exitedBoundary, goalBoundary, StringComparison.Ordinal)) return false;

        float axis = goalBoundary is "up" or "down" ? x : y;
        if (goalAperture is { Length: 2 }) {
            return axis >= goalAperture[0] && axis < goalAperture[1];
        }
        if (goalWorld is { Length: 2 }) {
            float targetAxis = goalBoundary is "up" or "down" ? goalWorld[0] : goalWorld[1];
            return axis >= targetAxis - 4f && axis < targetAxis + 4f;
        }
        return true;
    }

    private static string? InferBoundary(
        Rectangle bounds,
        float x,
        float y,
        float speedX,
        float speedY
    ) {
        (string Side, float Distance, float OutwardSpeed)[] candidates = [
            ("up", MathF.Abs(y - bounds.Top), -speedY),
            ("down", MathF.Abs(y - bounds.Bottom), speedY),
            ("left", MathF.Abs(x - bounds.Left), -speedX),
            ("right", MathF.Abs(x - bounds.Right), speedX)
        ];
        float nearest = candidates.Min(candidate => candidate.Distance);
        if (nearest > BoundaryBand) return null;
        (string Side, float Distance, float OutwardSpeed)[] nearestCandidates = candidates
            .Where(candidate => candidate.Distance <= nearest + 1f)
            .ToArray();
        return nearestCandidates
            .OrderByDescending(candidate => candidate.OutwardSpeed)
            .ThenBy(candidate => candidate.Distance)
            .First()
            .Side;
    }
}
