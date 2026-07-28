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

    [JsonPropertyName("run_nonce")]
    public string? RunNonce { get; set; }

    [JsonPropertyName("process_id")]
    public int? ProcessId { get; set; }

    [JsonPropertyName("capture_token")]
    public string? CaptureToken { get; set; }

    [JsonPropertyName("scenario_id")]
    public string? ScenarioId { get; set; }

    [JsonPropertyName("start_state_index")]
    public int StartStateIndex { get; set; }

    [JsonPropertyName("end_state_index")]
    public int EndStateIndex { get; set; }

    [JsonPropertyName("timeout_ms")]
    public int TimeoutMilliseconds { get; set; } = 120_000;

    [JsonPropertyName("reason")]
    public string? Reason { get; set; }

    [JsonPropertyName("recording_id")]
    public string? RecordingId { get; set; }

    [JsonPropertyName("max_frames")]
    public int MaxFrames { get; set; } = 36_000;
}

public sealed class CaptureStartRequest {
    public string CaptureToken { get; init; } = "";
    public string ScenarioId { get; init; } = "";
    public int StartStateIndex { get; init; }
    public int EndStateIndex { get; init; }
    public int TimeoutMilliseconds { get; init; }
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

    [JsonPropertyName("recording")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public CaptureStatus? Recording { get; set; }

    [JsonPropertyName("interactive_recording")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public InteractiveRecordingStatus? InteractiveRecording { get; set; }
}

public sealed class InteractiveRecordingStatus {
    [JsonPropertyName("state")]
    public string State { get; set; } = "";

    [JsonPropertyName("recording_id")]
    public string RecordingId { get; set; } = "";

    [JsonPropertyName("area_sid")]
    public string AreaSid { get; set; } = "";

    [JsonPropertyName("room")]
    public string Room { get; set; } = "";

    [JsonPropertyName("frame_count")]
    public int FrameCount { get; set; }

    [JsonPropertyName("state_count")]
    public int StateCount { get; set; }

    [JsonPropertyName("trace_path")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? TracePath { get; set; }

    [JsonPropertyName("reason")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? Reason { get; set; }
}

public sealed class CaptureStatus {
    [JsonPropertyName("state")]
    public string State { get; set; } = "";

    [JsonPropertyName("scenario_id")]
    public string ScenarioId { get; set; } = "";

    [JsonPropertyName("start_state_index")]
    public int StartStateIndex { get; set; }

    [JsonPropertyName("end_state_index")]
    public int EndStateIndex { get; set; }

    [JsonPropertyName("latest_state_index")]
    public int LatestStateIndex { get; set; }

    [JsonPropertyName("render_frame_count")]
    public int RenderFrameCount { get; set; }

    [JsonPropertyName("final_state_presented")]
    public bool FinalStatePresented { get; set; }

    [JsonPropertyName("repeated_presentation_count")]
    public int RepeatedPresentationCount { get; set; }

    [JsonPropertyName("unpresented_update_ranges")]
    public List<StateIndexRange> UnpresentedUpdateRanges { get; set; } = [];

    [JsonPropertyName("manifest_path")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? ManifestPath { get; set; }

    [JsonPropertyName("reason")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? Reason { get; set; }
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

    [JsonPropertyName("holding_theo")]
    public bool HoldingTheo { get; set; }

    [JsonPropertyName("dead")]
    public bool Dead { get; set; }

    [JsonPropertyName("freeze_timer")]
    public float FreezeTimer { get; set; }

    [JsonPropertyName("fields")]
    public Dictionary<string, object?> Fields { get; set; } = [];
}
