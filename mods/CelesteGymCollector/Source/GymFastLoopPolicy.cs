namespace Celeste.Mod.CelesteGymCollector;

internal static class GymFastLoopPolicy {
    public const int MaximumBatchFrames = 4096;
    public const int MaximumUpdatesPerOuterTick = 256;

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

    public static bool ConsumeOuterTickService(bool accelerated, ref bool consumed) {
        if (!accelerated) return true;
        if (consumed) return false;
        consumed = true;
        return true;
    }
}
