namespace Celeste.Mod.CelesteGymCollector;

internal static class GymAreaIdentity {
    public static AreaMode ResolveMode(int mode) {
        if (mode is < 0 or > 2) {
            throw new InvalidOperationException("area_mode must be 0 (A), 1 (B), or 2 (C)");
        }
        return (AreaMode) mode;
    }

    public static AreaKey CreateKey(int areaId, int mode) => new(areaId, ResolveMode(mode));

    public static bool Matches(AreaKey actual, int expectedId, int expectedMode) =>
        actual.ID == expectedId && actual.Mode == ResolveMode(expectedMode);

    public static string Sid(AreaKey area) =>
        AreaData.Get(area)?.SID
        ?? area.SID
        ?? throw new InvalidOperationException(
            $"area {area.ID}/{(int) area.Mode} has no registered SID"
        );

    public static bool Matches(
        AreaKey actual,
        int expectedId,
        int expectedMode,
        string expectedSid
    ) => Matches(actual.ID, (int) actual.Mode, Sid(actual), expectedId, expectedMode, expectedSid);

    public static bool CanResetInPlace(
        AreaKey active,
        string? activeSid,
        AreaKey target,
        string targetSid
    ) => activeSid is not null
        && Matches(
            active.ID,
            (int) active.Mode,
            activeSid,
            target.ID,
            (int) target.Mode,
            targetSid
        );

    internal static bool Matches(
        int actualId,
        int actualMode,
        string actualSid,
        int expectedId,
        int expectedMode,
        string expectedSid
    ) => actualId == expectedId
        && actualMode == (int) ResolveMode(expectedMode)
        && string.Equals(actualSid, expectedSid, StringComparison.Ordinal);
}
