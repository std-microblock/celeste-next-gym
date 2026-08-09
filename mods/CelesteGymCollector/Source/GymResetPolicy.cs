using System.Reflection;
using Monocle;

namespace Celeste.Mod.CelesteGymCollector;

internal static class GymResetPolicy {
    private static readonly FieldInfo LevelTransitionField = typeof(Level).GetField(
        "transition",
        BindingFlags.Instance | BindingFlags.NonPublic
    ) ?? throw new MissingFieldException(typeof(Level).FullName, "transition");

    public static void ClearEngineUpdateBlockers() {
        // A gym reset can interrupt dash/hit-stop before vanilla's next
        // Engine.Update has drained the global freeze timer. Engine still calls
        // the collector hooks while frozen, but skips Scene.Update entirely,
        // so a following gym_step would consume inputs without Player.Update.
        Engine.FreezeTimer = 0f;
        Engine.DashAssistFreeze = false;
        Engine.DashAssistFreezePress = false;
    }

    public static bool UseSessionLoaderForSameArea(string areaSid) =>
        areaSid.StartsWith("randomizer/", StringComparison.OrdinalIgnoreCase);

    public static void PrepareInPlaceLevel(Level level) {
        // A gym episode terminates as soon as Session.Level changes. At that
        // point Level's private room-transition coroutine is deliberately still
        // active and Level.Update services only that coroutine, not Entities.
        // Reusing the Level without cancelling it leaves the freshly reloaded
        // Player active in the tracker but permanently outside Player.Update.
        LevelTransitionField.SetValue(level, null);
        level.Paused = false;
    }
}
