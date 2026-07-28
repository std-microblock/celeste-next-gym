namespace Celeste.Mod.CelesteGymCollector;

/// <summary>
/// Decides whether the fresh LevelEnter wipe must be removed before a
/// scripted simulation installs its initial snapshot. A live presentation
/// capture owns the simulation from state zero, so letting the wipe run would
/// advance entities and scene clocks outside the trace; skip-transitions has
/// the same requirement even without a capture.
/// </summary>
internal static class RecordingLifecycle {
    public static bool ShouldCancelInitialEntryWipe(bool captureActive, bool skipTransitions) {
        return captureActive || skipTransitions;
    }
}
