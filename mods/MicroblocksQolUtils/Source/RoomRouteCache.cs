using System.Runtime.CompilerServices;
using Microsoft.Xna.Framework;

namespace Celeste.Mod.MicroblocksQolUtils;

public static class RoomRouteCache {
    private static readonly ConditionalWeakTable<MapData, RoomGraph> Graphs = new();

    public static int? RoomsToGoal(Level level) {
        MapData? map = level.Session.MapData;
        if (map is null || string.IsNullOrEmpty(level.Session.Level)) return null;
        return Graphs.GetValue(map, static value => new RoomGraph(value)).DistanceFrom(level.Session.Level);
    }

    private sealed class RoomGraph {
        private readonly Dictionary<string, int> distance = new(StringComparer.Ordinal);

        public RoomGraph(MapData map) {
            List<LevelData> levels = map.Levels.Where(level => !level.Dummy).ToList();
            if (levels.Count == 0) return;

            List<LevelData> goals = levels.Where(level => level.HasHeartGem).ToList();
            if (goals.Count == 0) goals.Add(levels[^1]);

            Dictionary<string, List<string>> reverseEdges = levels.ToDictionary(
                level => level.Name,
                _ => new List<string>(),
                StringComparer.Ordinal
            );
            for (int i = 0; i < levels.Count; i++) {
                for (int j = i + 1; j < levels.Count; j++) {
                    if (!TouchAlongEdge(levels[i].Bounds, levels[j].Bounds)) continue;
                    reverseEdges[levels[i].Name].Add(levels[j].Name);
                    reverseEdges[levels[j].Name].Add(levels[i].Name);
                }
            }

            Queue<string> queue = new();
            foreach (LevelData goal in goals) {
                distance[goal.Name] = 0;
                queue.Enqueue(goal.Name);
            }
            while (queue.Count > 0) {
                string current = queue.Dequeue();
                int nextDistance = distance[current] + 1;
                foreach (string next in reverseEdges[current]) {
                    if (distance.ContainsKey(next)) continue;
                    distance[next] = nextDistance;
                    queue.Enqueue(next);
                }
            }
        }

        public int? DistanceFrom(string room) => distance.TryGetValue(room, out int value) ? value : null;

        private static bool TouchAlongEdge(Rectangle first, Rectangle second) {
            bool horizontal = (first.Right == second.Left || second.Right == first.Left)
                && Math.Min(first.Bottom, second.Bottom) > Math.Max(first.Top, second.Top);
            bool vertical = (first.Bottom == second.Top || second.Bottom == first.Top)
                && Math.Min(first.Right, second.Right) > Math.Max(first.Left, second.Left);
            return horizontal || vertical;
        }
    }
}

