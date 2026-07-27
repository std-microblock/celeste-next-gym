using System.Text.RegularExpressions;

namespace Celeste.Mod.CelesteGymCollector;

internal static partial class RecordingSecurity {
    private const int MinimumTokenLength = 32;
    private const int MaximumTokenLength = 128;

    public static void Authenticate(
        string expectedNonce,
        int expectedProcessId,
        string? requestNonce,
        int? requestProcessId
    ) {
        if (string.IsNullOrEmpty(expectedNonce)
            || !string.Equals(expectedNonce, requestNonce, StringComparison.Ordinal)) {
            throw new UnauthorizedAccessException("recording run nonce does not match this collector process");
        }
        if (requestProcessId != expectedProcessId) {
            throw new UnauthorizedAccessException("recording process id does not match this collector process");
        }
    }

    public static string ValidateToken(string? token) {
        if (string.IsNullOrWhiteSpace(token)
            || token.Length is < MinimumTokenLength or > MaximumTokenLength
            || !CaptureTokenPattern().IsMatch(token)) {
            throw new InvalidOperationException(
                $"capture token must be {MinimumTokenLength}-{MaximumTokenLength} URL-safe characters"
            );
        }
        return token;
    }

    public static string ValidateScenarioId(string? scenarioId) {
        if (string.IsNullOrWhiteSpace(scenarioId)
            || scenarioId.Length > 128
            || !ScenarioIdPattern().IsMatch(scenarioId)) {
            throw new InvalidOperationException(
                "scenario id must contain only ASCII letters, digits, dot, underscore, and dash"
            );
        }
        return scenarioId;
    }

    public static string ResolveRecordingRoot(string? configuredRoot) {
        if (string.IsNullOrWhiteSpace(configuredRoot) || !Path.IsPathFullyQualified(configuredRoot)) {
            throw new InvalidOperationException(
                "CELESTE_GYM_RECORDING_ROOT must be an absolute per-run directory"
            );
        }
        string root = Path.GetFullPath(configuredRoot);
        Directory.CreateDirectory(root);
        RejectReparsePath(root);
        return root;
    }

    public static string CreateSessionDirectory(string recordingRoot, string scenarioId, string token) {
        string safeScenario = ValidateScenarioId(scenarioId);
        string safeToken = ValidateToken(token);
        string root = Path.GetFullPath(recordingRoot);
        string candidate = Path.GetFullPath(Path.Combine(root, "scenarios", safeScenario, safeToken));
        EnsureContained(root, candidate);
        if (Directory.Exists(candidate) || File.Exists(candidate)) {
            throw new InvalidOperationException("capture token already has recording artifacts");
        }
        Directory.CreateDirectory(candidate);
        RejectReparsePath(candidate);
        return candidate;
    }

    public static void EnsureContained(string root, string candidate) {
        string fullRoot = Path.TrimEndingDirectorySeparator(Path.GetFullPath(root));
        string fullCandidate = Path.GetFullPath(candidate);
        StringComparison comparison = OperatingSystem.IsWindows()
            ? StringComparison.OrdinalIgnoreCase
            : StringComparison.Ordinal;
        string prefix = fullRoot + Path.DirectorySeparatorChar;
        if (!fullCandidate.StartsWith(prefix, comparison)) {
            throw new UnauthorizedAccessException("recording path escapes the fixed per-run root");
        }
    }

    private static void RejectReparsePath(string path) {
        DirectoryInfo? current = new(Path.GetFullPath(path));
        while (current is not null && current.Exists) {
            if ((current.Attributes & FileAttributes.ReparsePoint) != 0) {
                throw new UnauthorizedAccessException("recording root must not contain reparse points");
            }
            current = current.Parent;
        }
    }

    [GeneratedRegex("^[A-Za-z0-9_-]+$", RegexOptions.CultureInvariant)]
    private static partial Regex CaptureTokenPattern();

    [GeneratedRegex("^[A-Za-z0-9][A-Za-z0-9._-]*$", RegexOptions.CultureInvariant)]
    private static partial Regex ScenarioIdPattern();
}
