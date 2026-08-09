namespace Celeste.Mod.CelesteGymCollector;

internal static class GymIdlePolicy {
    public static bool ShouldPark(
        bool episodeActive,
        bool resetActive,
        bool stepActive,
        bool simulationActive
    ) => episodeActive && !resetActive && !stepActive && !simulationActive;
}
