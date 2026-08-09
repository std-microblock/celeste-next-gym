namespace Celeste.Mod.CelesteGymCollector;

internal static class GymFastLoopPolicy {
    public const int MaximumBatchFrames = 4096;
    public const int MaximumUpdatesPerOuterTick = 256;
    public const int SynchronousStepBridgeMilliseconds = 8;
    public const int MaximumBridgedStepsPerOuterTick = 64;

    public static int SelectFrameCount(
        bool fastMode,
        bool stepAlreadyActive,
        string activeEpisodeId,
        CollectorRequest? request
    ) {
        if (!fastMode || stepAlreadyActive || request is null) return 0;
        if (!string.Equals(request.Command, "gym_step", StringComparison.Ordinal)) return 0;
        if (!string.Equals(request.EpisodeId, activeEpisodeId, StringComparison.Ordinal)) return 0;
        return request.Inputs.Count is >= 1 and <= MaximumBatchFrames
            ? Math.Min(request.Inputs.Count, MaximumUpdatesPerOuterTick)
            : 0;
    }

    public static int SelectActiveStepFrameCount(bool fastMode, int remainingFrames) {
        if (!fastMode || remainingFrames <= 0) return 0;
        return Math.Min(remainingFrames, MaximumUpdatesPerOuterTick);
    }

    public static int SelectResetFrameCount(bool fastMode, CollectorRequest? request = null) {
        if (!fastMode) return 0;
        if (request is not null
            && !string.Equals(request.Command, "gym_reset", StringComparison.Ordinal)) return 0;
        return MaximumUpdatesPerOuterTick;
    }

    public static bool ShouldBridgeSynchronousRequest(
        bool accelerated,
        bool requestActive,
        int bridgedSteps
    ) => accelerated
        && !requestActive
        && bridgedSteps < MaximumBridgedStepsPerOuterTick;

    public static bool ConsumeOuterTickService(bool accelerated, ref bool consumed) {
        if (!accelerated) return true;
        if (consumed) return false;
        consumed = true;
        return true;
    }

    public static bool ShouldRunAutoSplitter(
        bool gymActive,
        bool accelerated,
        ref bool consumed
    ) {
        // AutoSplitter reads SaveData.LevelSetStats through AreaKey.LevelSet.
        // Gym reset intentionally swaps Session while the current Level is
        // reloading; allowing that global observer to run in this window has
        // produced both duplicate-level-set-key and null-reference crashes.
        // A headless Gym episode has no autosplitting use, so isolate it for
        // the complete episode rather than merely reducing its update rate.
        if (gymActive) return false;
        return ConsumeOuterTickService(accelerated, ref consumed);
    }

    public static bool ShouldRunRumble(bool gymActive) => !gymActive;
}
