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

    [JsonPropertyName("episode_id")]
    public string? EpisodeId { get; set; }

    [JsonPropertyName("max_episode_frames")]
    public int MaxEpisodeFrames { get; set; } = 36_000;

    [JsonPropertyName("include_entities")]
    public bool? IncludeEntities { get; set; }

    [JsonPropertyName("include_player_states")]
    public bool? IncludePlayerStates { get; set; }

    [JsonPropertyName("fast_mode")]
    public bool FastMode { get; set; }
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

    [JsonPropertyName("talk_pressed")]
    public bool TalkPressed { get; set; }
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

    [JsonPropertyName("observation")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public GymObservation? Observation { get; set; }

    [JsonPropertyName("player_states")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<PlayerFrame>? PlayerStates { get; set; }

    [JsonPropertyName("frames_executed")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public int? FramesExecuted { get; set; }
}

public sealed class GymObservation {
    [JsonPropertyName("episode_id")]
    public string EpisodeId { get; set; } = "";

    [JsonPropertyName("episode_frame")]
    public int EpisodeFrame { get; set; }

    [JsonPropertyName("area_id")]
    public int AreaId { get; set; }

    [JsonPropertyName("area_sid")]
    public string AreaSid { get; set; } = "";

    [JsonPropertyName("room")]
    public string Room { get; set; } = "";

    [JsonPropertyName("fast_mode")]
    public bool FastMode { get; set; }

    [JsonPropertyName("player")]
    public PlayerFrame Player { get; set; } = new();

    [JsonPropertyName("room_geometry")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public GymRoomGeometry? RoomGeometry { get; set; }

    [JsonPropertyName("entities")]
    public List<GymEntityFrame> Entities { get; set; } = [];

    [JsonPropertyName("terminated")]
    public bool Terminated { get; set; }

    [JsonPropertyName("truncated")]
    public bool Truncated { get; set; }

    [JsonPropertyName("success")]
    public bool Success { get; set; }

    [JsonPropertyName("termination_reason")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? TerminationReason { get; set; }
}

public sealed class GymRoomGeometry {
    [JsonPropertyName("tile_size")]
    public int TileSize { get; set; } = 8;

    [JsonPropertyName("bounds")]
    public int[] Bounds { get; set; } = [0, 0, 0, 0];

    [JsonPropertyName("tile_origin")]
    public int[] TileOrigin { get; set; } = [0, 0];

    [JsonPropertyName("width")]
    public int Width { get; set; }

    [JsonPropertyName("height")]
    public int Height { get; set; }

    [JsonPropertyName("solids")]
    public List<string> Solids { get; set; } = [];
}

public sealed class GymEntityFrame {
    [JsonPropertyName("id")]
    public int Id { get; set; }

    [JsonPropertyName("type")]
    public string Type { get; set; } = "";

    [JsonPropertyName("position")]
    public float[] Position { get; set; } = [0, 0];

    [JsonPropertyName("collider")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public float[]? Collider { get; set; }

    [JsonPropertyName("collider_type")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? ColliderType { get; set; }

    [JsonPropertyName("speed")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public float[]? Speed { get; set; }

    [JsonPropertyName("lift_speed")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public float[]? LiftSpeed { get; set; }

    [JsonPropertyName("active")]
    public bool Active { get; set; }

    [JsonPropertyName("visible")]
    public bool Visible { get; set; }

    [JsonPropertyName("collidable")]
    public bool Collidable { get; set; }

    [JsonPropertyName("depth")]
    public int Depth { get; set; }

    [JsonPropertyName("tag")]
    public int Tag { get; set; }

    [JsonPropertyName("fields")]
    public Dictionary<string, object?> Fields { get; set; } = [];
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

    [JsonPropertyName("holding_glider")]
    public bool HoldingGlider { get; set; }

    [JsonPropertyName("dead")]
    public bool Dead { get; set; }

    [JsonPropertyName("freeze_timer")]
    public float FreezeTimer { get; set; }

    [JsonPropertyName("fields")]
    public Dictionary<string, object?> Fields { get; set; } = [];
}
