using System.Text.Json.Serialization;

namespace Celeste.Mod.CelesteGymCollector;

public sealed class CollectorRequest {
    [JsonPropertyName("command")]
    public string Command { get; set; } = "ping";

    [JsonPropertyName("area_id")]
    public int AreaId { get; set; }

    [JsonPropertyName("area_sid")]
    public string? AreaSid { get; set; }

    [JsonPropertyName("room")]
    public string? Room { get; set; }

    [JsonPropertyName("dream_dash")]
    public bool DreamDash { get; set; }

    [JsonPropertyName("inputs")]
    public List<FrameInput> Inputs { get; set; } = [];

    [JsonPropertyName("initial_snapshot")]
    public InitialSnapshot? InitialSnapshot { get; set; }

    [JsonPropertyName("skip_transitions")]
    public bool SkipTransitions { get; set; }
}

public sealed class FrameInput {
    [JsonPropertyName("move_x")]
    public int MoveX { get; set; }

    [JsonPropertyName("move_y")]
    public int MoveY { get; set; }

    [JsonPropertyName("jump_pressed")]
    public bool JumpPressed { get; set; }

    [JsonPropertyName("jump_held")]
    public bool JumpHeld { get; set; }

    [JsonPropertyName("dash_pressed")]
    public bool DashPressed { get; set; }

    [JsonPropertyName("crouch_dash_pressed")]
    public bool CrouchDashPressed { get; set; }

    [JsonPropertyName("grab_held")]
    public bool GrabHeld { get; set; }
}

public sealed class InitialSnapshot {
    [JsonPropertyName("pos")]
    public float[]? Pos { get; set; }

    [JsonPropertyName("speed")]
    public float[]? Speed { get; set; }

    [JsonPropertyName("dashes")]
    public int? Dashes { get; set; }

    [JsonPropertyName("stamina")]
    public float? Stamina { get; set; }

    [JsonPropertyName("state")]
    public int? State { get; set; }

    [JsonPropertyName("facing")]
    public bool? Facing { get; set; }

    [JsonPropertyName("ducking")]
    public bool? Ducking { get; set; }
}

public sealed class CollectorResponse {
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    [JsonPropertyName("error")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? Error { get; set; }

    [JsonPropertyName("states")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<PlayerFrame>? States { get; set; }

    [JsonPropertyName("version")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? Version { get; set; }

    [JsonPropertyName("run_nonce")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? RunNonce { get; set; }

    [JsonPropertyName("process_id")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public int? ProcessId { get; set; }

    [JsonPropertyName("collector_port")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public int? CollectorPort { get; set; }
}

public sealed class PlayerFrame {
    [JsonPropertyName("frame")]
    public int Frame { get; set; }

    [JsonPropertyName("pos")]
    public float[] Pos { get; set; } = [0, 0];

    [JsonPropertyName("speed")]
    public float[] Speed { get; set; } = [0, 0];

    [JsonPropertyName("state")]
    public int State { get; set; }

    [JsonPropertyName("facing")]
    public int Facing { get; set; }

    [JsonPropertyName("dashes")]
    public int Dashes { get; set; }

    [JsonPropertyName("stamina")]
    public float Stamina { get; set; }

    [JsonPropertyName("on_ground")]
    public bool OnGround { get; set; }

    [JsonPropertyName("ducking")]
    public bool Ducking { get; set; }

    [JsonPropertyName("can_dream_dash")]
    public bool CanDreamDash { get; set; }

    [JsonPropertyName("dead")]
    public bool Dead { get; set; }

    [JsonPropertyName("freeze_timer")]
    public float FreezeTimer { get; set; }

    [JsonPropertyName("fields")]
    public Dictionary<string, object?> Fields { get; set; } = [];
}
