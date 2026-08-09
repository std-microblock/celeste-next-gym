namespace Celeste.Mod.CelesteGymCollector;

internal static class GymFastLoopPolicy {
    public const int MaximumBatchFrames = 4096;

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
            ? request.Inputs.Count
            : 0;
    }
}
