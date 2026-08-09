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
}
