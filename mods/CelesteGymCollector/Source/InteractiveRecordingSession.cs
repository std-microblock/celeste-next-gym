using System.Text.Json;
using System.Text.Json.Serialization;

namespace Celeste.Mod.CelesteGymCollector;

internal sealed class InteractiveRecordingSession {
    private static readonly JsonSerializerOptions Json = new() {
        WriteIndented = true
    };

    private readonly string recordingRoot;
    private readonly string sessionDirectory;
    private readonly int areaId;
    private readonly int maxFrames;
    private readonly List<FrameInput> inputs = [];
    private readonly List<PlayerFrame> states = [];
    private FrameInput? pendingInput;
    private Player? pendingPlayer;
    private PlayerFrame? deathFrame;
    private string state = "loading";
    private string? tracePath;
    private string? reason;

    public InteractiveRecordingSession(
        string recordingRoot,
        string token,
        string recordingId,
        int areaId,
        string areaSid,
        string room,
        int maxFrames
    ) {
        this.recordingRoot = RecordingSecurity.ResolveRecordingRoot(recordingRoot);
        Token = RecordingSecurity.ValidateToken(token);
        RecordingId = RecordingSecurity.ValidateScenarioId(recordingId);
        AreaSid = areaSid;
        Room = RecordingSecurity.ValidateScenarioId(room);
        if (maxFrames is < 1 or > 1_048_576) {
            throw new InvalidOperationException("interactive max_frames must be between 1 and 1048576");
        }
        this.maxFrames = maxFrames;
        this.areaId = areaId;
        sessionDirectory = RecordingSecurity.CreateSessionDirectory(
            this.recordingRoot,
            $"interactive-{RecordingId}",
            Token
        );
    }

    public string Token { get; }
    public string RecordingId { get; }
    public string AreaSid { get; }
    public string Room { get; }
    public bool IsRunning => state is "loading" or "active";

    public void TryBegin(Level level) {
        if (state != "loading" || level.Session.Area.ID != areaId || level.Wipe is not null) return;
        Player? player = level.Tracker.GetEntity<Player>();
        if (player is null) return;
        states.Add(SnapshotCapture.Capture(player, 0));
        state = "active";
    }

    public void BeginPlayerFrame(Player player) {
        if (state != "active" || pendingInput is not null) return;
        if (player.Scene is not Level level || level.Session.Area.ID != areaId) return;
        pendingPlayer = player;
        pendingInput = new FrameInput {
            MoveX = Math.Clamp(Input.MoveX.Value, -1, 1),
            MoveY = Math.Clamp(Input.MoveY.Value, -1, 1),
            JumpPressed = Input.Jump.Pressed,
            JumpHeld = Input.Jump.Check,
            DashPressed = Input.Dash.Pressed,
            CrouchDashPressed = Input.CrouchDash.Pressed,
            GrabHeld = Input.Grab.Check,
            TalkPressed = Input.Talk.Pressed
        };
    }

    public void CaptureDeath(Player player) {
        if (state == "active" && pendingInput is not null) {
            deathFrame = SnapshotCapture.Capture(player, inputs.Count + 1);
        }
    }

    public void FinishPlayerFrame() {
        if (state != "active" || pendingInput is null || pendingPlayer is null) return;
        inputs.Add(pendingInput);
        states.Add(deathFrame ?? SnapshotCapture.Capture(pendingPlayer, inputs.Count));
        pendingInput = null;
        pendingPlayer = null;
        deathFrame = null;
        if (inputs.Count >= maxFrames) Stop("max_frames");
    }

    public InteractiveRecordingStatus Stop(string stopReason) {
        if (state == "stopped") return GetStatus();
        if (state != "active" || states.Count == 0) {
            throw new InvalidOperationException("interactive recording has not reached an active player frame");
        }
        pendingInput = null;
        pendingPlayer = null;
        deathFrame = null;
        reason = string.IsNullOrWhiteSpace(stopReason) ? "explicit_stop" : stopReason;
        string finalPath = Path.Combine(sessionDirectory, "trace.json");
        string temporaryPath = finalPath + ".tmp";
        RecordingSecurity.EnsureContained(recordingRoot, finalPath);
        InteractiveTrace trace = new() {
            RecordedAt = DateTimeOffset.UtcNow.ToString("O"),
            Map = new InteractiveTraceMap {
                Sid = AreaSid,
                Room = Room,
                Binary = "maps/CelesteGymPlayground/Playground.bin"
            },
            Inputs = inputs,
            States = states
        };
        File.WriteAllText(temporaryPath, JsonSerializer.Serialize(trace, Json) + Environment.NewLine);
        File.Move(temporaryPath, finalPath, overwrite: true);
        tracePath = Path.GetRelativePath(recordingRoot, finalPath).Replace('\\', '/');
        state = "stopped";
        return GetStatus();
    }

    public InteractiveRecordingStatus GetStatus() => new() {
        State = state,
        RecordingId = RecordingId,
        AreaSid = AreaSid,
        Room = Room,
        FrameCount = inputs.Count,
        StateCount = states.Count,
        TracePath = tracePath,
        Reason = reason
    };
}

internal sealed class InteractiveTrace {
    [JsonPropertyName("format")]
    public string Format { get; init; } = "celeste-next-gym-trace";

    [JsonPropertyName("version")]
    public int Version { get; init; } = 1;

    [JsonPropertyName("source")]
    public string Source { get; init; } = "game";

    [JsonPropertyName("recorded_at")]
    public string RecordedAt { get; init; } = "";

    [JsonPropertyName("map")]
    public InteractiveTraceMap Map { get; init; } = new();

    [JsonPropertyName("inputs")]
    public List<FrameInput> Inputs { get; init; } = [];

    [JsonPropertyName("states")]
    public List<PlayerFrame> States { get; init; } = [];
}

internal sealed class InteractiveTraceMap {
    [JsonPropertyName("sid")]
    public string Sid { get; init; } = "";

    [JsonPropertyName("room")]
    public string Room { get; init; } = "";

    [JsonPropertyName("binary")]
    public string Binary { get; init; } = "";
}
