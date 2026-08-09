namespace Celeste.Mod.CelesteGymCollector;

internal static class GymOverlayPolicy {
    public const int MaximumLines = 8;
    public const int MaximumCharactersPerLine = 160;

    public static string[] Normalize(IReadOnlyList<string>? lines) {
        if (lines is null) return [];
        if (lines.Count > MaximumLines) {
            throw new InvalidOperationException($"overlay_lines may contain at most {MaximumLines} lines");
        }
        string[] normalized = new string[lines.Count];
        for (int index = 0; index < lines.Count; index++) {
            string line = lines[index] ?? "";
            if (line.Length > MaximumCharactersPerLine) {
                throw new InvalidOperationException(
                    $"overlay_lines[{index}] may contain at most {MaximumCharactersPerLine} characters"
                );
            }
            if (line.Contains('\r') || line.Contains('\n')) {
                throw new InvalidOperationException($"overlay_lines[{index}] must be a single line");
            }
            normalized[index] = line;
        }
        return normalized;
    }

    public static bool ShouldRender(bool headlessActor, IReadOnlyList<string> lines) =>
        !headlessActor && lines.Count > 0;
}
