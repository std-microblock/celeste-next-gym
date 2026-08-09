using System.Reflection;
using Monocle;

namespace Celeste.Mod.CelesteGymCollector;

internal static class GymResetPolicy {
    public const int GarbageCollectionInterval = 256;

    private static readonly FieldInfo LevelTransitionField = typeof(Level).GetField(
        "transition",
        BindingFlags.Instance | BindingFlags.NonPublic
    ) ?? throw new MissingFieldException(typeof(Level).FullName, "transition");
    private static int inPlaceReloadCount;

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

    public static void ReloadInPlace(Level level) {
        if (level.Completed) return;
        Session session = level.Session;
        if (session.FirstLevel
            && session.Strawberries.Count <= 0
            && !session.Cassette
            && !session.HeartGem
            && !session.HitCheckpoint) {
            session.Time = 0L;
            session.Deaths = 0;
            level.TimerStarted = false;
        }
        session.Dashes = session.DashesAtLevelStart;
        Glitch.Value = 0f;
        Engine.TimeRate = 1f;
        Distort.Anxiety = 0f;
        Distort.GameRate = 1f;
        Audio.SetMusicParam("fade", 1f);
        level.ParticlesBG.Clear();
        level.Particles.Clear();
        level.ParticlesFG.Clear();
        TrailManager.Clear();
        level.UnloadLevel();

        // Vanilla Level.Reload forces a full blocking collection and waits for
        // every finalizer on every death. That is useful for an interactive
        // game loading at human speed, but dominates short RL episodes. Keep
        // the authoritative unload/load path and amortize that maintenance so
        // stale entity graphs are still reclaimed in long-lived actors.
        int completedReloads = Interlocked.Increment(ref inPlaceReloadCount);
        if (ShouldCollectGarbage(completedReloads)) {
            GC.Collect();
            GC.WaitForPendingFinalizers();
        }

        level.LoadLevel(Player.IntroTypes.Respawn);
        level.strawberriesDisplay.DrawLerp = 0f;
        WindController? windController = level.Entities.FindFirst<WindController>();
        if (windController is not null) windController.SnapWind();
        else level.Wind = Microsoft.Xna.Framework.Vector2.Zero;
    }

    public static bool ShouldCollectGarbage(int completedReloads) =>
        completedReloads > 0 && completedReloads % GarbageCollectionInterval == 0;
}
