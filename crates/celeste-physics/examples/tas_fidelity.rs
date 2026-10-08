//! Replay a real-game Celeste TAS ground-truth trace through `celeste-physics`
//! room segment by room segment, and emit a machine-readable fidelity report.
//!
//! ```text
//! cargo run -q -p celeste-physics --release --example tas_fidelity -- \
//!   --trace <trace.jsonl> --maps <dir> --out <report.json> \
//!   [--min-frames 1] [--limit-segments N] [--max-frames N] [--rooms a,b]
//! ```
//!
//! `--dump-field-map` prints the generated `PlayerSnapshot` field-coverage table
//! (the same table that is embedded in the report) as Markdown and exits.

// The report views are single large `serde_json::json!` literals; the default
// macro recursion limit is too small once the exporter tail fields are included.
#![recursion_limit = "256"]

use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use celeste_physics::{
    CoreMode, EntityKind, InputState, Map, PlayerSnapshot, PlayerState, Rect, SimulationError,
    Simulator, Vec2, celeste_map_rooms, clutter_switch_color_at, decode_map_room,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value, json};

/// One chapter's `oshiro_clutter_cleared_*` session flags plus the session clock
/// of the last segment replayed for it.
///
/// `ClutterSwitch.OnDashed` writes `Session.SetFlag("oshiro_clutter_cleared_" +
/// (int)color)` when a downward dash lands on it (`ClutterSwitch.cs:131-156`),
/// and `ClutterBlockGenerator.Init` reads the three flags back to decide
/// whether each colour's `ClutterBlockBase` Solid starts collidable
/// (`ClutterBlockGenerator.cs:78-81`). They are chapter state, not player
/// state, so no trace row carries them: the gate walks the trace in row order,
/// so keeping one entry per chapter reproduces the live session exactly.
/// `Session.Deaths` and `Session.Time` fall back when the chapter is restarted,
/// which is the only way those flags are cleared again.
#[derive(Clone, Copy, Default)]
struct ClutterCarry {
    deaths: i64,
    level_time: i64,
    seen: bool,
    cleared: [bool; 3],
}

// The chapter whose segment is currently being replayed, and the carried flags.
thread_local! {
    static Y3_CHAPTER: std::cell::RefCell<(String, i64)> =
        const { std::cell::RefCell::new((String::new(), 0)) };
    static Y3_CLUTTER: std::cell::RefCell<HashMap<(String, i64), ClutterCarry>> =
        std::cell::RefCell::new(HashMap::new());
    /// Flags the chapter carried into the segment now being replayed (used by
    /// the `--probe-remainder` diagnostic, which re-anchors that segment).
    static Y3_CARRIED: std::cell::RefCell<[bool; 3]> = const { std::cell::RefCell::new([false; 3]) };
}

// ---------------------------------------------------------------------------
// Trace parsing
// ---------------------------------------------------------------------------

/// The virtual `Celeste.Input` state read back from the game after the frame.
///
/// `talkP` is optional because the earlier exporter revision that produced
/// `trace-1a.jsonl` did not emit the `Pressed` edge for Talk.
///
/// The `*P0` keys are the same press edges sampled **before** anything in the frame could consume
/// them (`TasFrameTrace.CaptureInput`, called right after `InputHelper.FeedInputs`). They exist
/// because the post-frame `*P` keys are lossy: `Player.BoostUpdate` calls
/// `Input.Dash.ConsumePress()` (`Player.cs:4721-4731`), which makes Monocle's `VirtualButton.Pressed`
/// false for the rest of the frame (`Monocle/VirtualButton.cs:55-58`, `:153-157`), so every
/// `StBoost` frame that consumed a dash press reports `dashP: false`. They are absent from the v1
/// and v2 traces, where the fallback reproduces the old behaviour exactly.
#[derive(Clone, Default, Deserialize)]
#[serde(default)]
struct InputRec {
    mx: i64,
    my: i64,
    #[allow(dead_code)]
    gly: i64,
    jump: bool,
    #[serde(rename = "jumpP")]
    jump_pressed: bool,
    #[serde(rename = "jumpP0")]
    jump_pressed_0: Option<bool>,
    dash: bool,
    #[serde(rename = "dashP")]
    dash_pressed: bool,
    #[serde(rename = "dashP0")]
    dash_pressed_0: Option<bool>,
    cdash: bool,
    #[serde(rename = "cdashP")]
    crouch_dash_pressed: bool,
    #[serde(rename = "cdashP0")]
    crouch_dash_pressed_0: Option<bool>,
    grab: bool,
    talk: bool,
    #[serde(rename = "talkP")]
    talk_pressed: Option<bool>,
    #[serde(rename = "talkP0")]
    talk_pressed_0: Option<bool>,
}

impl InputRec {
    fn to_input_state(&self, frame_delta_time_bits: u32) -> InputState {
        InputState {
            move_x: self.mx.clamp(-1, 1) as i8,
            move_y: self.my.clamp(-1, 1) as i8,
            jump_pressed: self.jump_pressed_0.unwrap_or(self.jump_pressed),
            jump_held: self.jump,
            dash_pressed: self.dash_pressed_0.unwrap_or(self.dash_pressed),
            crouch_dash_pressed: self
                .crouch_dash_pressed_0
                .unwrap_or(self.crouch_dash_pressed),
            grab_held: self.grab,
            talk_pressed: self
                .talk_pressed_0
                .or(self.talk_pressed)
                .unwrap_or(self.talk),
            frame_delta_time_bits: Some(frame_delta_time_bits),
            // `*P0` is `Monocle.VirtualButton.Pressed`, i.e. the buffered level the
            // game itself read, already zeroed wherever the game consumed the
            // press. `Simulator::step` must therefore adopt it verbatim instead of
            // running it through its own `VirtualButton` buffer a second time.
            presses_are_effective: true,
        }
    }
}

/// `Session.Inventory` (`Celeste.Session.cs:35`, `PlayerInventory.cs:22-28`), the
/// session-level flag block `Player.Inventory` forwards (`Player.cs:956-966`).
#[derive(Clone, Default, Deserialize)]
#[serde(default)]
struct InventoryRec {
    #[serde(rename = "Dashes")]
    dashes: Option<i64>,
    #[serde(rename = "DreamDash")]
    dream_dash: Option<bool>,
    #[serde(rename = "Backpack")]
    backpack: Option<bool>,
    #[serde(rename = "NoRefills")]
    no_refills: Option<bool>,
}

/// Two- and four-element float arrays. Kept as `Vec<f64>` rather than a fixed-size
/// array so that an exporter that writes `null` for a non-finite component does not
/// turn the whole row into a parse error.
fn pair_field(values: Option<&Vec<f64>>) -> Option<[f64; 2]> {
    let values = values?;
    Some([*values.first()?, *values.get(1)?])
}

fn quad_field(values: Option<&Vec<f64>>) -> Option<[f64; 4]> {
    let values = values?;
    Some([
        *values.first()?,
        *values.get(1)?,
        *values.get(2)?,
        *values.get(3)?,
    ])
}

/// `Celeste.Session.CoreModes` (`Session.cs:22-27`): `None = 0, Hot = 1, Cold = 2`, matching the
/// declaration order of `celeste_physics::CoreMode` (`types.rs:162-169`).
fn core_mode_from_int(value: i64) -> Option<CoreMode> {
    Some(match value {
        0 => CoreMode::None,
        1 => CoreMode::Hot,
        2 => CoreMode::Cold,
        _ => return None,
    })
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Record {
    n: u64,
    f: i64,
    dt: f64,
    #[serde(rename = "rawDt")]
    raw_dt: Option<f64>,
    #[serde(rename = "timeRate")]
    time_rate: Option<f64>,
    #[serde(rename = "in")]
    input: InputRec,
    scene: String,
    sid: Option<String>,
    mode: Option<i64>,
    /// `Session.Area.ID`. Selects the `PlayerInventory` (`AreaData`) the run is
    /// playing with, which sets `Player.MaxDashes` and `NoRefills`.
    area: Option<i64>,
    room: Option<String>,
    /// `Session.Deaths` and `Session.Time` (`Session.cs`). Used only to detect
    /// that a chapter session restarted - both go backwards - so the carried
    /// `oshiro_clutter_cleared_*` flags are reset with it.
    deaths: Option<i64>,
    #[serde(rename = "levelTime")]
    level_time: Option<i64>,
    state: Option<String>,
    p: Option<JsonMap<String, Value>>,
    // ---- Append-only exporter tail (see tools/celestetas-trace/TasFrameTrace.cs).
    /// `Celeste.Level.Wind` (`Level.cs:149`).
    wind: Option<Vec<f64>>,
    /// `WindController.targetSpeed` (`WindController.cs:47`), reached by reflection
    /// through the private `Level.windController` field (`Level.cs:101`).
    #[serde(rename = "windTarget")]
    wind_target: Option<Vec<f64>>,
    #[serde(rename = "windPattern")]
    wind_pattern: Option<i64>,
    /// `Celeste.Level.Transitioning` (`Level.cs:221`).
    transitioning: Option<bool>,
    /// `Monocle.Engine.FreezeTimer` (`Monocle/Engine.cs:28`).
    #[serde(rename = "freezeTimer")]
    freeze_timer: Option<f64>,
    /// `Player.Ducking` (`Player.cs:1005-1028`), a computed `Entity.Collider` property.
    ducking: Option<bool>,
    /// `Celeste.Session.CoreMode` (`Session.cs:22-27,111`), i.e. `Level.Session.CoreMode`
    /// as read by `Player.NormalUpdate` (`Player.cs:3681-3684`). Session state on
    /// `Level.Session`, not a `Player` field, so the reflection dump cannot carry
    /// it. Workstream W8 adds the key; the traces in use do not have it yet.
    #[serde(rename = "coreMode")]
    core_mode: Option<i64>,
    /// The active collider as `[absoluteLeft, absoluteTop, width, height]`
    /// (`Monocle/Collider.cs:229,205,12,14`).
    collider: Option<Vec<f64>>,
    inventory: Option<InventoryRec>,
    /// `Celeste.Level.InSpace` = `levelData.Space` (`Level.cs:449`), the per-room map property
    /// `Player.cs:3703-3706,3718-3722,3778-3781` reads. `map.rs` does not decode the `.bin` level
    /// element's `space` attribute, so this is the only source.
    #[serde(rename = "inSpace")]
    in_space: Option<bool>,
}

#[derive(Clone)]
struct Frame {
    n: u64,
    f: i64,
    dt: f64,
    raw_dt: Option<f64>,
    time_rate: Option<f64>,
    input: InputRec,
    state_name: Option<String>,
    fields: Option<JsonMap<String, Value>>,
    /// `Level.Wind` captured at the end of this engine frame, i.e. exactly the value
    /// `WindController.Update` ramps from on the next frame (`WindController.cs:194`).
    wind: Option<[f64; 2]>,
    /// `WindController.targetSpeed` at the end of this engine frame.
    wind_target: Option<[f64; 2]>,
    wind_pattern: Option<i64>,
    transitioning: Option<bool>,
    /// `Engine.FreezeTimer` at the end of this engine frame (`Engine.cs:266-269`).
    freeze_timer: Option<f64>,
    ducking: Option<bool>,
    /// `Celeste.Session.CoreMode` (`Session.cs:22-27,111`) at the end of this
    /// engine frame, already converted from the trace's integer.
    core_mode: Option<CoreMode>,
    collider: Option<[f64; 4]>,
    inventory: Option<InventoryRec>,
    /// `Level.InSpace` at the end of this engine frame. Parsed and reported, but there is no
    /// `PlayerSnapshot::in_space` and `map.rs` does not decode the room's `Space` property, so it is
    /// not restored (see the harness's open-gap notes).
    in_space: Option<bool>,
    /// `Session.Deaths` / `Session.Time`, carried so the gate can notice that a
    /// chapter session restarted between two segments of the same chapter.
    deaths: Option<i64>,
    level_time: Option<i64>,
}

/// The subset of `Celeste.Player` fields `tas_fidelity` diffs.
#[derive(Clone, Copy, Default)]
struct Truth {
    /// `Monocle.Entity.Position` - the post-frame player position. This is the
    /// exact counterpart of `PlayerSnapshot::pos`.
    position: Option<[f64; 2]>,
    speed: Option<[f64; 2]>,
    /// `Monocle.Platform.movementCounter`, the sub-pixel remainder that
    /// `Actor.MoveH`/`MoveV` round away every call.
    movement_counter: Option<[f64; 2]>,
    stamina: Option<f64>,
    dashes: Option<i64>,
    facing: Option<i64>,
    on_ground: Option<bool>,
    dead: Option<bool>,
    /// `Player.StrawberryCollectResetTimer`. `Player.Update` decrements it
    /// unconditionally, so an unchanged value across two rows proves that the
    /// entity update did not run for that engine frame.
    collect_reset_timer: Option<f64>,
}

impl Truth {
    fn read(fields: Option<&JsonMap<String, Value>>) -> Self {
        let Some(fields) = fields else {
            return Self::default();
        };
        Self {
            position: vector_field(fields, "Position"),
            speed: vector_field(fields, "Speed"),
            movement_counter: vector_field(fields, "movementCounter"),
            stamina: float_field(fields, "Stamina"),
            dashes: fields.get("Dashes").and_then(Value::as_i64),
            facing: fields.get("Facing").and_then(Value::as_i64),
            on_ground: fields.get("onGround").and_then(Value::as_bool),
            dead: fields
                .get("<Dead>k__BackingField")
                .and_then(Value::as_bool),
            collect_reset_timer: float_field(fields, "StrawberryCollectResetTimer"),
        }
    }
}

fn float_field(fields: &JsonMap<String, Value>, name: &str) -> Option<f64> {
    fields.get(name).and_then(Value::as_f64)
}

/// Whether a row on which `Player.Update` did not run still carries a different
/// player snapshot than the row before it.
///
/// `Player.Update` decrements `StrawberryCollectResetTimer` unconditionally
/// (`Player.cs:1477`), so two consecutive rows with the same value prove the
/// entity did not update. When the compared snapshot fields changed anyway, the
/// change came from a path outside `Player.Update`
/// (`Player.TransitionTo`, `Player.StartCassetteFly`, ...), i.e. a row the
/// simulator cannot produce. Compares exactly the fields the gate diffs, so it
/// can never excuse a difference the gate would otherwise have caught.
fn stalled_row_mutates_player(frames: &[Frame], truth: &[Truth], index: usize) -> bool {
    let Some(previous) = index.checked_sub(1) else {
        return false;
    };
    let (before, after) = (&truth[previous], &truth[index]);
    before.position != after.position
        || before.speed != after.speed
        || before.movement_counter != after.movement_counter
        || before.stamina != after.stamina
        || before.dashes != after.dashes
        || before.facing != after.facing
        || before.on_ground != after.on_ground
        || before.dead != after.dead
        || frames[previous].state_name != frames[index].state_name
        || frames[previous].ducking != frames[index].ducking
}

fn vector_field(fields: &JsonMap<String, Value>, name: &str) -> Option<[f64; 2]> {
    let values = fields.get(name)?.as_array()?;
    Some([values.first()?.as_f64()?, values.get(1)?.as_f64()?])
}

// ---------------------------------------------------------------------------
// Snake <-> trace value conversion
// ---------------------------------------------------------------------------

trait FromTraceValue: Sized {
    fn from_trace_value(value: &Value) -> Option<Self>;
}

impl FromTraceValue for f32 {
    fn from_trace_value(value: &Value) -> Option<Self> {
        value.as_f64().map(|value| value as f32)
    }
}

impl FromTraceValue for i8 {
    fn from_trace_value(value: &Value) -> Option<Self> {
        value.as_i64().and_then(|value| i8::try_from(value).ok())
    }
}

impl FromTraceValue for u8 {
    fn from_trace_value(value: &Value) -> Option<Self> {
        value.as_i64().and_then(|value| u8::try_from(value).ok())
    }
}

impl FromTraceValue for u16 {
    fn from_trace_value(value: &Value) -> Option<Self> {
        value.as_i64().and_then(|value| u16::try_from(value).ok())
    }
}

impl FromTraceValue for bool {
    fn from_trace_value(value: &Value) -> Option<Self> {
        value.as_bool()
    }
}

impl FromTraceValue for Vec2 {
    fn from_trace_value(value: &Value) -> Option<Self> {
        let values = value.as_array()?;
        let x = values.first()?.as_f64()? as f32;
        let y = values.get(1)?.as_f64()? as f32;
        Some(Vec2::new(x, y))
    }
}

/// `float?` source fields (`launchApproachX`). `null` explicitly restores
/// `None`; a missing key leaves the field at its default.
impl FromTraceValue for Option<f32> {
    fn from_trace_value(value: &Value) -> Option<Self> {
        if value.is_null() {
            return Some(None);
        }
        value.as_f64().map(|value| Some(value as f32))
    }
}

// ---------------------------------------------------------------------------
// Field coverage table
// ---------------------------------------------------------------------------

struct RestoredField {
    field: &'static str,
    ty: &'static str,
    source: &'static str,
    set: fn(&mut PlayerSnapshot, &Value),
}

macro_rules! restored_fields {
    ($( $field:ident : $ty:ty = $source:literal ),* $(,)?) => {
        const RESTORED_FIELDS: &[RestoredField] = &[
            $(
                RestoredField {
                    field: stringify!($field),
                    ty: stringify!($ty),
                    source: $source,
                    set: |snapshot, value| {
                        if let Some(parsed) = <$ty as FromTraceValue>::from_trace_value(value) {
                            snapshot.$field = parsed;
                        }
                    },
                }
            ),*
        ];
    };
}

restored_fields! {
    pos: Vec2 = "Position",
    speed: Vec2 = "Speed",
    movement_remainder: Vec2 = "movementCounter",
    current_lift_speed: Vec2 = "currentLiftSpeed",
    last_lift_speed: Vec2 = "lastLiftSpeed",
    lift_speed_timer: f32 = "liftSpeedTimer",
    ignore_jump_thrus: bool = "IgnoreJumpThrus",
    dashes: u8 = "Dashes",
    stamina: f32 = "Stamina",
    on_ground: bool = "onGround",
    player_on_ground: bool = "onGround",
    dead: bool = "<Dead>k__BackingField",
    just_respawned: bool = "JustRespawned",
    dash_dir: Vec2 = "DashDir",
    last_aim: Vec2 = "lastAim",
    before_dash_speed: Vec2 = "beforeDashSpeed",
    demo_dashed: bool = "demoDashed",
    dash_started_on_ground: bool = "dashStartedOnGround",
    dash_attack_timer: f32 = "dashAttackTimer",
    dash_cooldown_timer: f32 = "dashCooldownTimer",
    dash_refill_cooldown_timer: f32 = "dashRefillCooldownTimer",
    boost_target: Vec2 = "boostTarget",
    boost_red: bool = "boostRed",
    no_wind_timer: f32 = "noWindTimer",
    wall_slide_timer: f32 = "wallSlideTimer",
    wall_slide_dir: i8 = "wallSlideDir",
    jump_grace_timer: f32 = "jumpGraceTimer",
    auto_jump: bool = "AutoJump",
    auto_jump_timer: f32 = "AutoJumpTimer",
    var_jump_timer: f32 = "varJumpTimer",
    var_jump_speed: f32 = "varJumpSpeed",
    max_fall: f32 = "maxFall",
    move_x: i8 = "moveX",
    force_move_x: i8 = "forceMoveX",
    force_move_x_timer: f32 = "forceMoveXTimer",
    wall_speed_retention_timer: f32 = "wallSpeedRetentionTimer",
    wall_speed_retained: f32 = "wallSpeedRetained",
    wall_boost_timer: f32 = "wallBoostTimer",
    wall_boost_dir: i8 = "wallBoostDir",
    hop_wait_x: i8 = "hopWaitX",
    hop_wait_x_speed: f32 = "hopWaitXSpeed",
    min_hold_timer: f32 = "minHoldTimer",
    climb_no_move_timer: f32 = "climbNoMoveTimer",
    last_climb_move: i8 = "lastClimbMove",
    dream_dash_can_end_timer: f32 = "dreamDashCanEndTimer",
    launch_approach_x: Option<f32> = "launchApproachX",
    summit_launch_target_x: f32 = "summitLaunchTargetX",
    summit_launch_particle_timer: f32 = "summitLaunchParticleTimer",
    star_fly_timer: f32 = "starFlyTimer",
    star_fly_transforming: bool = "starFlyTransforming",
    star_fly_speed_lerp: f32 = "starFlySpeedLerp",
    star_fly_last_dir: Vec2 = "starFlyLastDir",
    strawberry_collect_index: u16 = "StrawberryCollectIndex",
    strawberry_collect_reset_timer: f32 = "StrawberryCollectResetTimer",
    explode_launch_boost_timer: f32 = "explodeLaunchBoostTimer",
    explode_launch_boost_speed: f32 = "explodeLaunchBoostSpeed",
    dummy_moving: bool = "DummyMoving",
    dummy_gravity: bool = "DummyGravity",
    dummy_friction: bool = "DummyFriction",
    dummy_maxspeed: bool = "DummyMaxspeed",
    launched: bool = "launched",
    // `Player.wasDucking` (`Player.cs:461`) and `Player.holdCannotDuck`
    // (`Player.cs:465`) are declared private fields of `Player`, so the reflection
    // dump carries them under their exact C# names.
    was_ducking: bool = "wasDucking",
    hold_cannot_duck: bool = "holdCannotDuck",
}

/// Fields restored by bespoke code rather than the generic `p`-key loop.
const DERIVED_FIELDS: &[(&str, &str)] = &[
    ("state", "top-level `state` name (`PlayerStates.GetCurrentStateName`) mapped exhaustively onto `PlayerState`."),
    ("facing", "`p.Facing` (`Facings` enum: -1 Left, 1 Right) compared against the boolean `facing`."),
    ("time_rate", "top-level `timeRate` (`Engine.TimeRate`)."),
    ("player_on_ground_initialized", "set to true; the anchor row is a post-`Player.Update` capture, so the source-private `onGround` is authoritative."),
    ("max_dashes", "`Player.MaxDashes` -> `PlayerInventory.Dashes` for the segment's `Session.Area.ID` (`AreaData`), tightened by the largest `Dashes` the trace reports in the segment."),
    ("no_refills", "`PlayerInventory.NoRefills` for the segment's `Session.Area.ID` (`AreaData`)."),
    ("frame_delta_time", "`#[serde(skip)]` on the wire type. `Simulator::step` recomputes it every frame as the supplied `rawDt` bits times `time_rate`."),
    ("state_timer", "in the Dash state, `PlayerSnapshot::restore_dash_phase` rebuilds the simulator's dash clock from `p.dashAttackTimer`: Celeste times the dash with `DashCoroutine` (`Player.cs:4465-4567`), not a `StateMachine.Timer`, and `dashAttackTimer` (`Player.cs:4296`, decremented per unfrozen frame at `Player.cs:1577-1580`) counts exactly those frames."),
    ("wind", "top-level `wind` (`Celeste.Level.Wind`, `Level.cs:149`). Restored verbatim from the anchor row so the room segment continues the source's own ramp: `WindController.Update` rewrites it with `Calc.Approach(level.Wind, targetSpeed, 1000f * Engine.DeltaTime)` (`WindController.cs:194`) and displaces the Player's `WindMover` component (`Player.cs:1180`) by `level.Wind * 0.1f * Engine.DeltaTime` (`WindController.cs:199-201`)."),
    ("wind_target", "top-level `windTarget` (`WindController.targetSpeed`, `WindController.cs:47`, read by reflection through the private `Level.windController` field at `Level.cs:101`). The replayed ramp needs the source's own target, which a room segment cannot reconstruct."),
    ("ducking", "top-level `ducking` (`Player.Ducking`, `Player.cs:1005-1028`). A computed property over `Monocle.Entity.Collider` (`Monocle/Entity.cs:73`), so it is not a declared field; the exporter also writes the active collider as `collider` = `[absoluteLeft, absoluteTop, width, height]` (`Monocle/Collider.cs:229,205,12,14`)."),
    ("freeze_timer", "top-level `freezeTimer` (`Monocle.Engine.FreezeTimer`, `Monocle/Engine.cs:28`). While positive `Engine.Update` only decrements it and skips `Scene.Update` entirely (`Engine.cs:266-269`); the exporter writes the post-decrement value, which is exactly the snapshot state `Simulator::step` reads at the top of the next frame."),
    ("can_dream_dash", "top-level `inventory.DreamDash` (`Celeste.Session.Inventory`, `Session.cs:35`, `PlayerInventory.cs:24`). `Player.Inventory` forwards it (`Player.cs:956-966`) and the source reads it at `Player.cs:3420` and `4500`; restoring it removes the need to infer the flag from `dreamDashCanEndTimer`."),
    ("core_mode", "top-level `coreMode` (`Celeste.Session.CoreMode`, `Session.cs:111`; `None = 0, Hot = 1, Cold = 2` per `Session.cs:22-27`). Session state, so the base-chain dump cannot see it; the Core's ice factor (`Player.cs:3681-3684`, `if (onGround && level.CoreMode == Cold) num2 *= 0.3f`) and the `CoreModeListener` entities read it. Without it every Core room replays as `CoreMode::None`."),
    ("wall_boosting", "`Player.wallBoosting` (`Player.cs:3100`) is private, so `Celeste.Player`'s declared-field dump cannot see it. `Simulator::climb_update` derives it from the room's `WallBooster` set exactly as `Player.ClimbUpdate` does (`Player.cs:3154-3167` on, `3168-3170` off), and it is only read by the \"climbed over the ledge\" branch (`Player.cs:3140-3149`)."),
];

/// Fields with no ground-truth source anywhere in the trace.
fn unrestored_fields() -> Vec<(&'static str, &'static str)> {
    const NO_PLAYER_FIELD: &str =
        "no equally-named `Player` field in the trace; left to the simulator (`Default` or `Simulator::new` initialize_*)";
    const NOT_DECLARED_ON_PLAYER: &str =
        "not reachable from the player's base chain (`Celeste.Player` -> `Monocle.Actor` -> `Monocle.Platform` -> `Monocle.Entity`), so the reflection dump cannot see it";
    const VIRTUAL_BUTTON: &str =
        "`VirtualButton` buffers live on the static `Celeste.Input` object, not on `Player`; `Simulator::step` rebuilds them from the press edges";
    const ENTITY_STATE: &str =
        "room-entity runtime state, not `Player` state; initialized by `Simulator::new` (`initialize_*`) from the decoded room";

    let mut fields: Vec<(&'static str, &'static str)> = Vec::new();
    let mut push = |names: &[&'static str], reason: &'static str| {
        for name in names {
            fields.push((*name, reason));
        }
    };

    push(
        &[
            "intro_phase",
            "intro_phase_ready",
            "intro_start",
            "intro_sprite_frame",
            "intro_sprite_timer",
            "intro_timer",
        ],
        "`Player.Intro*` coroutine progress. The intro state callbacks (`Player.cs` IntroWalk/IntroJump/IntroWakeUp/IntroThinkForABit) live in the state machine's enumerators, and the sprite clock lives on the `Monocle.Sprite` component, so no `Player` field carries it. The simulator reconstructs the phase once from the anchor's state, position and facing (`intro_resume`) instead.",
    );
    push(
        &[
            "camera",
            "camera_initialized",
        ],
        NOT_DECLARED_ON_PLAYER,
    );
    push(
        &[
            "jump_buffer_timer",
            "dash_buffer_timer",
            "crouch_dash_buffer_timer",
        ],
        VIRTUAL_BUTTON,
    );
    push(
        &[
            "dash_end_pending",
            "death_freeze_pending",
            "respawn_frames",
            "neutral_wall_jump_friction_delay",
            "moving_solid_time",
            "scene_time_active",
            "booster_boosting",
            "last_booster_target",
            "booster_reuse_timer",
            "star_fly_transform_frames",
            "last_feather_target",
            "feather_reuse_timer",
            "last_bumper_target",
            "bumper_reuse_timer",
            "star_fly_hitbox_preserved",
            "last_bounce_target",
            "bounce_reuse_timer",
            "pending_bounce_from_y",
            "temple_fall_landed",
            "temple_fall_wait_frames",
            "reflection_fall_phase",
            "reflection_fall_frames",
            "reflection_fall_wait_timer",
            "transition_direction",
            "transition_target",
            "transition_timer",
            "post_transition_normal_updates",
            "current_room_bounds",
            "transition_room_bounds",
            "holding_theo",
            "holding_glider",
            "pickup_old_speed",
            "pickup_old_var_jump_timer",
            "pickup_timer",
            "carried_strawberries",
            "strawberry_follow_delay_timer",
            "strawberry_collect_timer",
            "strawberry_picked_mask",
        ],
        NO_PLAYER_FIELD,
    );
    push(
        &[
            "cassette_manager",
            "cassette_blocks",
            "spinners",
            "bumpers",
            "lookouts",
            "zip_movers",
            "bounce_blocks",
            "move_blocks",
            "theo_crystals",
            "heart_gems",
            "rising_lavas",
            "sandwich_lavas",
            "gliders",
            "clouds",
            "seekers",
            "temple_gates",
            "refills",
            "falling_blocks",
            "exit_blocks",
            "invisible_barriers",
            "killboxes",
            "crush_blocks",
            "dash_blocks",
            "badeline_boost_active",
            "badeline_boost_final",
            "badeline_boost_phase",
            "badeline_boost_frame",
            "badeline_boost_start",
            "badeline_boost_target",
            "last_badeline_boost_target",
            "badeline_boost_entity_origin",
            "badeline_boost_current_position",
            "badeline_boost_relocation_from",
            "badeline_boost_relocation_to",
            "badeline_boost_relocation_elapsed",
            "badeline_boost_relocation_duration",
            "badeline_boost_stage",
            "badeline_boost_relocating",
            "badeline_boost_collidable",
        ],
        ENTITY_STATE,
    );
    fields
}

fn snapshot_field_names() -> Vec<String> {
    let value = serde_json::to_value(PlayerSnapshot::default()).expect("PlayerSnapshot serializes");
    let mut names: Vec<String> = value
        .as_object()
        .expect("PlayerSnapshot is a struct")
        .keys()
        .cloned()
        .collect();
    names.sort();
    names
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CoverageAudit {
    declared_fields: usize,
    restored: usize,
    derived: usize,
    unrestored: usize,
    /// Declared `PlayerSnapshot` fields absent from the coverage table.
    missing: Vec<String>,
    /// Table entries that no longer exist on `PlayerSnapshot`.
    stale: Vec<String>,
}

fn coverage_audit() -> CoverageAudit {
    let known: Vec<String> = RESTORED_FIELDS
        .iter()
        .map(|field| field.field.to_owned())
        .chain(DERIVED_FIELDS.iter().map(|(name, _)| (*name).to_owned()))
        .chain(unrestored_fields().iter().map(|(name, _)| (*name).to_owned()))
        .chain(std::iter::once("frame_delta_time".to_owned()))
        .collect();
    let declared = snapshot_field_names();
    let missing: Vec<String> = declared
        .iter()
        .filter(|name| !known.contains(*name))
        .cloned()
        .collect();
    let stale: Vec<String> = known
        .iter()
        .filter(|name| !declared.contains(*name) && name.as_str() != "frame_delta_time")
        .cloned()
        .collect();
    CoverageAudit {
        declared_fields: declared.len(),
        restored: RESTORED_FIELDS.len(),
        derived: DERIVED_FIELDS.len(),
        unrestored: unrestored_fields().len(),
        missing,
        stale,
    }
}

fn snapshot_fields_markdown() -> String {
    let mut out = String::new();
    out.push_str("| PlayerSnapshot field | type | trace source |\n");
    out.push_str("| --- | --- | --- |\n");
    for field in RESTORED_FIELDS {
        out.push_str(&format!(
            "| `{}` | `{}` | `p.{}` |\n",
            field.field, field.ty, field.source
        ));
    }
    for (name, note) in DERIVED_FIELDS {
        out.push_str(&format!("| `{name}` | derived | {note} |\n"));
    }
    let mut unrestored = unrestored_fields();
    unrestored.sort_by(|a, b| a.0.cmp(b.0));
    for (name, reason) in unrestored {
        out.push_str(&format!("| `{name}` | none | {reason} |\n"));
    }
    let audit = coverage_audit();
    out.push_str(&format!(
        "\nDeclared `PlayerSnapshot` fields: {}. Restored from a `p` key: {}. Derived: {}. Unrestored: {}. Missing from table: {:?}. Stale table entries: {:?}.\n",
        audit.declared_fields,
        audit.restored,
        audit.derived,
        audit.unrestored,
        audit.missing,
        audit.stale,
    ));
    out
}

// ---------------------------------------------------------------------------
// Segment bookkeeping
// ---------------------------------------------------------------------------

struct Segment {
    sid: String,
    mode: i64,
    /// `Session.Area.ID` of the segment's first row.
    area: i64,
    room: String,
    start_row: u64,
    start_frame: i64,
    /// `StrawberryCollectResetTimer` of the immediately preceding trace row,
    /// when that row is this segment's row - 1. Used to decide whether the
    /// segment's first row itself is a live `Player.Update` capture.
    previous_timer: Option<f64>,
    frames: Vec<Frame>,
}

impl Segment {
    fn key(&self) -> String {
        format!("{}|{}|{}", self.sid, self.mode, self.room)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FirstMismatch {
    /// Zero-based index of the replayed frame inside the segment.
    offset: u64,
    reasons: Vec<String>,
    rust: Value,
    game: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SegmentReport {
    sid: String,
    mode: i64,
    room: String,
    area_file: Option<String>,
    start_row: u64,
    start_frame: i64,
    /// Number of input frames replayed (stepped and diffed).
    frames: u64,
    /// Number of trace rows grouped into the segment.
    records: u64,
    status: String,
    exact_prefix_frames: u64,
    /// Rows at the head of the segment that were not replayed because
    /// `Player.Update` did not run on them (room transition / `Celeste.Freeze`)
    /// or because they were needed as the anchor row.
    leading_skipped_frames: u64,
    /// Rows inside the replay window where `Player.Update` did not run.
    stalled_frames: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    stall_offsets: Vec<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unsupported_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unsupported_offset: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_mismatch: Option<FirstMismatch>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    unavailable_checks: Vec<String>,
    /// Frames where the simulator's post-entity geometric `on_ground` differs
    /// from its source-private `player_on_ground` (diagnostic only; the
    /// game-side `onGround` is compared against `player_on_ground`).
    geometric_ground_diff_frames: u64,
    /// Frames where `Engine.FreezeTimer`/`Level.Transitioning` state in the
    /// simulator disagrees with the trace's stalled rows.
    freeze_disagreement_frames: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_freeze_disagreement_offset: Option<u64>,
    /// Frames where the simulator's own `WindController` ramp disagrees with the
    /// trace's exported `Level.Wind` (diagnostic; never a mismatch reason).
    #[serde(skip_serializing_if = "is_zero")]
    wind_disagreement_frames: u64,
    /// Frames where the simulator's `Player.Ducking` disagrees with the trace's
    /// exported `Player.Ducking` (diagnostic; never a mismatch reason).
    #[serde(skip_serializing_if = "is_zero")]
    ducking_disagreement_frames: u64,
    /// Replayed frames where `Player.Update` did not run but the player snapshot
    /// still changed, so the row came from a coroutine that drives the player
    /// directly and was not compared. Diagnostic only.
    #[serde(skip_serializing_if = "is_zero")]
    frozen_mutation_frames: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    frozen_mutation_offsets: Vec<u64>,
    /// Mechanism split of the stalled rows inside the replay window, from the
    /// v3 exporter tail (`transitioning`: `Level.cs:221`; `freezeTimer`:
    /// `Engine.cs:28`; neither: the `Level.FrozenOrPaused` branch at
    /// `Level.cs:1837`). Diagnostic only.
    #[serde(skip_serializing_if = "is_zero")]
    stall_transition_frames: u64,
    #[serde(skip_serializing_if = "is_zero")]
    stall_engine_freeze_frames: u64,
    #[serde(skip_serializing_if = "is_zero")]
    stall_frozen_entity_frames: u64,
    /// Present only with `--probe-remainder`; see `probe_remainder`.
    #[serde(skip_serializing_if = "Option::is_none")]
    remainder_probe: Option<Value>,
    /// Present only with `--dump-segment`; one line per replayed frame with the
    /// recovered per-frame `Actor.MoveH`/`MoveV` totals on both sides.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    dump: Vec<String>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Totals {
    records: u64,
    level_records: u64,
    non_level_records: u64,
    parse_errors: u64,
    segments: u64,
    simulated_segments: u64,
    ok: u64,
    mismatch: u64,
    unsupported: u64,
    init_error: u64,
    map_error: u64,
    state_map_error: u64,
    no_live_window: u64,
    sim_error: u64,
    skipped: u64,
    total_frames: u64,
    exact_frames: u64,
    stalled_frames: u64,
    truncated: bool,
}

#[derive(Default, Clone)]
struct RoomAccumulator {
    segments: u64,
    frames: u64,
    ok: u64,
    mismatch: u64,
    unsupported: u64,
    other: u64,
    exact_prefix: Vec<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RoomStat {
    segments: u64,
    frames: u64,
    ok: u64,
    mismatch: u64,
    unsupported: u64,
    other: u64,
    median_exact_prefix: u64,
    max_exact_prefix: u64,
    min_exact_prefix: u64,
}

impl RoomAccumulator {
    fn finish(&self) -> RoomStat {
        let mut sorted = self.exact_prefix.clone();
        sorted.sort_unstable();
        let median = if sorted.is_empty() {
            0
        } else {
            sorted[sorted.len() / 2]
        };
        RoomStat {
            segments: self.segments,
            frames: self.frames,
            ok: self.ok,
            mismatch: self.mismatch,
            unsupported: self.unsupported,
            other: self.other,
            median_exact_prefix: median,
            max_exact_prefix: sorted.last().copied().unwrap_or(0),
            min_exact_prefix: sorted.first().copied().unwrap_or(0),
        }
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct StateStat {
    segments: u64,
    frames: u64,
    rows_in_state: u64,
}

// ---------------------------------------------------------------------------
// State name mapping
// ---------------------------------------------------------------------------

/// Exhaustive map from `PlayerStates.GetCurrentStateName` to `PlayerState`.
///
/// The ids come from the `Player.St*` constants in
/// `1.4.0.0 itch.io FNA 代码/Celeste/Player.cs`, which are also the order of
/// the `StateMachine` callbacks in `Player.ctor` and of `PlayerState`.
fn state_from_name(name: &str) -> Option<PlayerState> {
    Some(match name {
        "StNormal" | "Normal" => PlayerState::Normal,
        "StClimb" | "Climb" => PlayerState::Climb,
        "StDash" | "Dash" => PlayerState::Dash,
        "StSwim" | "Swim" => PlayerState::Swim,
        "StBoost" | "Boost" => PlayerState::Boost,
        "StRedDash" | "RedDash" => PlayerState::RedDash,
        "StHitSquash" | "HitSquash" => PlayerState::HitSquash,
        "StLaunch" | "Launch" => PlayerState::Launch,
        "StPickup" | "Pickup" => PlayerState::Pickup,
        "StDreamDash" | "DreamDash" => PlayerState::DreamDash,
        "StSummitLaunch" | "SummitLaunch" => PlayerState::SummitLaunch,
        "StDummy" | "Dummy" => PlayerState::Dummy,
        "StIntroWalk" | "IntroWalk" => PlayerState::IntroWalk,
        "StIntroJump" | "IntroJump" => PlayerState::IntroJump,
        "StIntroRespawn" | "IntroRespawn" => PlayerState::IntroRespawn,
        "StIntroWakeUp" | "IntroWakeUp" => PlayerState::IntroWakeUp,
        "StBirdDashTutorial" | "BirdDashTutorial" => PlayerState::BirdDashTutorial,
        "StFrozen" | "Frozen" => PlayerState::Frozen,
        "StReflectionFall" | "ReflectionFall" => PlayerState::ReflectionFall,
        "StStarFly" | "StarFly" => PlayerState::StarFly,
        "StTempleFall" | "TempleFall" => PlayerState::TempleFall,
        "StCassetteFly" | "CassetteFly" => PlayerState::CassetteFly,
        "StAttract" | "Attract" => PlayerState::Attract,
        "StIntroMoonJump" | "IntroMoonJump" => PlayerState::IntroMoonJump,
        "StFlingBird" | "FlingBird" => PlayerState::FlingBird,
        "StIntroThinkForABit" | "IntroThinkForABit" => PlayerState::IntroThinkForABit,
        _ => return None,
    })
}

fn state_name(state: PlayerState) -> &'static str {
    match state {
        PlayerState::Normal => "StNormal",
        PlayerState::Climb => "StClimb",
        PlayerState::Dash => "StDash",
        PlayerState::Swim => "StSwim",
        PlayerState::Boost => "StBoost",
        PlayerState::RedDash => "StRedDash",
        PlayerState::HitSquash => "StHitSquash",
        PlayerState::Launch => "StLaunch",
        PlayerState::Pickup => "StPickup",
        PlayerState::DreamDash => "StDreamDash",
        PlayerState::SummitLaunch => "StSummitLaunch",
        PlayerState::Dummy => "StDummy",
        PlayerState::IntroWalk => "StIntroWalk",
        PlayerState::IntroJump => "StIntroJump",
        PlayerState::IntroRespawn => "StIntroRespawn",
        PlayerState::IntroWakeUp => "StIntroWakeUp",
        PlayerState::BirdDashTutorial => "StBirdDashTutorial",
        PlayerState::Frozen => "StFrozen",
        PlayerState::ReflectionFall => "StReflectionFall",
        PlayerState::StarFly => "StStarFly",
        PlayerState::TempleFall => "StTempleFall",
        PlayerState::CassetteFly => "StCassetteFly",
        PlayerState::Attract => "StAttract",
        PlayerState::IntroMoonJump => "StIntroMoonJump",
        PlayerState::FlingBird => "StFlingBird",
        PlayerState::IntroThinkForABit => "StIntroThinkForABit",
    }
}

// ---------------------------------------------------------------------------
// Map resolution
// ---------------------------------------------------------------------------

/// `Celeste/1-ForsakenCity` + mode 1 -> `1H-ForsakenCity.bin`.
fn area_file_name(sid: &str, mode: i64) -> Option<String> {
    let suffix = sid.rsplit('/').next().unwrap_or(sid);
    let mode = if suffix.ends_with("-BSide") {
        1
    } else if suffix.ends_with("-CSide") {
        2
    } else {
        mode
    };
    let suffix = suffix
        .trim_end_matches("-BSide")
        .trim_end_matches("-CSide");
    if mode == 0 {
        return Some(format!("{suffix}.bin"));
    }
    let letter = match mode {
        1 => "H",
        2 => "X",
        _ => return None,
    };
    let (id, name) = suffix.split_once('-')?;
    Some(format!("{id}{letter}-{name}.bin"))
}

#[derive(Default)]
struct MapCache {
    bytes: HashMap<PathBuf, Option<Vec<u8>>>,
    rooms: HashMap<PathBuf, Vec<String>>,
    decoded: HashMap<String, Map>,
}

enum MapLookup {
    Ready(String, Map),
    Failed(String),
}

impl MapCache {
    fn get(&mut self, maps_dir: &Path, sid: &str, mode: i64, room: &str) -> MapLookup {
        let Some(file_name) = area_file_name(sid, mode) else {
            return MapLookup::Failed(format!("no vanilla map file for sid {sid:?} mode {mode}"));
        };
        let path = maps_dir.join(&file_name);
        let bytes = self
            .bytes
            .entry(path.clone())
            .or_insert_with(|| match fs::read(&path) {
                Ok(bytes) => Some(bytes),
                Err(error) => {
                    eprintln!("tas_fidelity: cannot read {path:?}: {error}");
                    None
                }
            });
        let Some(bytes) = bytes.as_ref() else {
            return MapLookup::Failed(format!(
                "cannot read area map {} (resolved from sid {sid:?} mode {mode})",
                path.display()
            ));
        };
        let rooms = self
            .rooms
            .entry(path.clone())
            .or_insert_with(|| match celeste_map_rooms(bytes) {
                Ok(rooms) => rooms,
                Err(error) => {
                    eprintln!("tas_fidelity: cannot list rooms in {path:?}: {error}");
                    Vec::new()
                }
            });
        if !rooms.is_empty() && !rooms.iter().any(|name| name == room) {
            return MapLookup::Failed(format!(
                "room {room:?} is not in {} (rooms: {})",
                path.display(),
                rooms.join(",")
            ));
        }
        let key = format!("{}#{room}", path.display());
        if !self.decoded.contains_key(&key) {
            match decode_map_room(bytes, Some(room)) {
                Ok(map) => {
                    self.decoded.insert(key.clone(), map);
                }
                Err(error) => {
                    return MapLookup::Failed(format!(
                        "decode_map_room({}, {room:?}) failed: {error}",
                        path.display()
                    ));
                }
            }
        }
        MapLookup::Ready(file_name, self.decoded[&key].clone())
    }
}

// ---------------------------------------------------------------------------
// Simulation
// ---------------------------------------------------------------------------

/// Restore every `PlayerSnapshot` field that has a `p` key (see
/// `RESTORED_FIELDS`). Returns the field names actually applied, so a trace
/// that omits optional keys is visible rather than silently defaulted.
fn restore_restored_fields(
    snapshot: &mut PlayerSnapshot,
    fields: &JsonMap<String, Value>,
) -> Vec<&'static str> {
    let mut applied = Vec::new();
    for field in RESTORED_FIELDS {
        if let Some(value) = fields.get(field.source) {
            (field.set)(snapshot, value);
            applied.push(field.field);
        }
    }
    applied
}

const TOLERANCE: f32 = 0.01;

/// `PlayerInventory.Dashes` for the area whose `Session.Area.ID` is `area`
/// (`AreaData` assigns one `PlayerInventory` per area/mode; `PlayerInventory.cs`
/// declares the values):
///
/// * area 0 (Prologue) is `PlayerInventory.Prologue` -> `Dashes = 0`;
/// * areas 7 (The Summit), 8 (Epilogue) are `PlayerInventory.TheSummit` -> 2;
/// * area 9 (Core) is `PlayerInventory.Core` -> 2;
/// * area 10 (`LostLevels`, the Farewell chapter) is `PlayerInventory.Farewell`
///   -> 1. The chapter's intro cutscene raises the *session* value to 2
///   (`CS10_Gravestone.cs` `BadelineRejoin`/`OnEnd`:
///   `Level.Session.Inventory.Dashes = 2;`), and `CS10_FinalRoom.cs:76` returns
///   it to 1 before the ending, so no per-area constant is right for the whole
///   chapter: the per-segment witness below recovers the raised value wherever
///   the trace demonstrates it, and this table value covers the rest;
/// * everything else is `PlayerInventory.Default` -> 1.
///
/// `PlayerInventory.cs` cannot be observed through the trace: `Player.Inventory`
/// is a property over `Level.Session`, so the exporter's `DeclaredOnly` field
/// walk cannot see it. The session value is therefore recovered from the area
/// table above, tightened by `observe_session_dashes` below whenever the trace
/// actually witnesses the session's capacity.
fn area_inventory_dashes(area: i64) -> u8 {
    match area {
        0 => 0,
        7 | 8 | 9 => 2,
        _ => 1,
    }
}

/// `PlayerInventory.NoRefills` (`PlayerInventory.cs`): `PlayerInventory.Core`
/// (area 9) sets it, which suppresses `RefillDash()` at `Player.cs:1602`,
/// `2687`, `2718`, `2758`, `4961` and `5159`.
///
/// `NoRefillTrigger` (`NoRefillTrigger.cs`) can also toggle this per room; the
/// trace carries no session-inventory field, so only the area default is
/// restored here.
fn area_inventory_no_refills(area: i64) -> bool {
    area == 9
}

/// Recover `PlayerInventory.Dashes` for this segment from the trace.
///
/// `Player.RefillDash()` only ever writes `Dashes = MaxDashes` and
/// `Player.MaxDashes` is `Inventory.Dashes`, so a `Dashes` value the game itself
/// reports after a refill is a witness for the session's capacity there. It is a
/// *lower* bound: `Refill.UseRefill(twoDashes)` can also land on the pink
/// diamond's fixed `2` (`Player.cs` UseRefill) regardless of `MaxDashes`, and
/// `BadelineBoost`'s alarm only ever increments up to `Inventory.Dashes`
/// (`BadelineBoost.cs:216-219`). The pink diamond is therefore excluded below by
/// position, which leaves only writes that set exactly `MaxDashes` - or a
/// bounded increment - as witnesses.
///
/// Taking the maximum when the segment shows any dash at all, and falling back
/// to the area's `PlayerInventory` otherwise, keeps the restored capacity
/// monotone: it never exceeds what the game demonstrated in that segment, and
/// never discards a dash the area's inventory grants. Farewell is why the
/// witness has to win over the area table: the chapter's intro
/// (`CS10_Gravestone.cs:133-134/144-145`) raises `Inventory.Dashes` to 2 while
/// `CS10_FinalRoom.cs:76` returns it to 1, so no single per-area constant fits
/// the chapter, and the second `LostLevels` pass (which never replays the intro)
/// only ever refills to one dash.
fn observe_session_dashes(area: i64, witnessed: Option<i64>) -> u8 {
    let area_dashes = area_inventory_dashes(area);
    match witnessed.map(|value| value.clamp(0, u8::MAX as i64) as u8) {
        Some(witnessed) if witnessed > 0 => witnessed.min(area_dashes),
        _ => area_dashes,
    }
}

/// Is the player's post-frame position inside a pink `twoDash` diamond?
///
/// `Refill.RefillRoutine`/`OnPlayer` (`Refill.cs`) is the only refill that can
/// exceed `Player.MaxDashes`, and `twoDashes` comes straight from the map's
/// `twoDash` attribute, so the decoded room is the witness. The probe is the
/// player's own box widened by a few pixels, because a collected diamond stops
/// being collidable on that same frame and a fast dash can travel several pixels
/// while inside its 16x16 hitbox.
fn near_pink_refill(map: &Map, position: [f64; 2]) -> bool {
    let probe = Rect::new(
        position[0] as f32 - 12.0,
        position[1] as f32 - 14.0,
        24.0,
        28.0,
    );
    map.entities.iter().any(|entity| {
        entity.kind == EntityKind::Refill
            && entity.direction.x != 0.0
            && entity.bounds.intersects(probe)
    })
}

/// Largest `Dashes` this segment reaches through a refill that cannot be the
/// pink diamond, i.e. through a write that lands on exactly `Player.MaxDashes`.
fn witnessed_session_dashes(map: &Map, truth: &[Truth]) -> Option<i64> {
    let mut best: Option<i64> = None;
    for index in 1..truth.len() {
        let (Some(previous), Some(current)) = (truth[index - 1].dashes, truth[index].dashes) else {
            continue;
        };
        if current <= previous || current <= 0 {
            continue;
        }
        let Some(position) = truth[index].position else {
            continue;
        };
        if near_pink_refill(map, position) {
            continue;
        }
        best = Some(best.map_or(current, |value| value.max(current)));
    }
    best
}

fn bits_of(delta: f64) -> u32 {
    (delta as f32).to_bits()
}

/// `skip_serializing_if` helper for the optional diagnostic counters: a segment that
/// never disagreed keeps its report entry byte-identical to the pre-tail schema.
fn is_zero(value: &u64) -> bool {
    *value == 0
}

fn rust_view(snapshot: &PlayerSnapshot) -> Value {
    json!({
        "pos": [snapshot.pos.x, snapshot.pos.y],
        "speed": [snapshot.speed.x, snapshot.speed.y],
        "state": state_name(snapshot.state),
        "stateId": snapshot.state as u8,
        "facing": snapshot.facing,
        "dashes": snapshot.dashes,
        "stamina": snapshot.stamina,
        "onGround": snapshot.on_ground,
        "playerOnGround": snapshot.player_on_ground,        "ducking": snapshot.ducking,
        "dead": snapshot.dead,
        "freezeTimer": snapshot.freeze_timer,
        "wind": [snapshot.wind.x, snapshot.wind.y],
        "windTarget": [snapshot.wind_target.x, snapshot.wind_target.y],
        "canDreamDash": snapshot.can_dream_dash,
        "coreMode": match snapshot.core_mode {
            CoreMode::None => 0,
            CoreMode::Hot => 1,
            CoreMode::Cold => 2,
        },
        "stateTimer": snapshot.state_timer,
        "movementRemainder": [snapshot.movement_remainder.x, snapshot.movement_remainder.y],
        "dashAttackTimer": snapshot.dash_attack_timer,
        "dashCooldownTimer": snapshot.dash_cooldown_timer,
        "dashRefillCooldownTimer": snapshot.dash_refill_cooldown_timer,
        "varJumpTimer": snapshot.var_jump_timer,
        "varJumpSpeed": snapshot.var_jump_speed,
        "jumpGraceTimer": snapshot.jump_grace_timer,
        "wallSlideTimer": snapshot.wall_slide_timer,
        "wallBoostTimer": snapshot.wall_boost_timer,
        "forceMoveX": snapshot.force_move_x,
        "forceMoveXTimer": snapshot.force_move_x_timer,
        "moveX": snapshot.move_x,
        "hopWaitX": snapshot.hop_wait_x,
        "hopWaitXSpeed": snapshot.hop_wait_x_speed,
        "climbNoMoveTimer": snapshot.climb_no_move_timer,
        "lastClimbMove": snapshot.last_climb_move,
        "maxFall": snapshot.max_fall,
        "starFlyTimer": snapshot.star_fly_timer,
        "starFlySpeedLerp": snapshot.star_fly_speed_lerp,
        "starFlyTransforming": snapshot.star_fly_transforming,
        "dashDir": [snapshot.dash_dir.x, snapshot.dash_dir.y],
        "beforeDashSpeed": [snapshot.before_dash_speed.x, snapshot.before_dash_speed.y],
        "demoDashed": snapshot.demo_dashed,
        "dashStartedOnGround": snapshot.dash_started_on_ground,
        "launched": snapshot.launched,
        "timeRate": snapshot.time_rate,
        "frameDeltaTime": snapshot.frame_delta_time,
        "explodeLaunchBoostTimer": snapshot.explode_launch_boost_timer,
        "explodeLaunchBoostSpeed": snapshot.explode_launch_boost_speed,
    })
}

fn game_view(frame: &Frame, truth: &Truth, pos: Option<[f64; 2]>) -> Value {
    json!({
        "row": frame.n,
        "tasFrame": frame.f,
        "pos": pos,
        "speed": truth.speed,
        "state": frame.state_name,
        "facing": truth.facing,
        "dashes": truth.dashes,
        "stamina": truth.stamina,
        "onGround": truth.on_ground,
        "dead": truth.dead,
        "ducking": frame.ducking,
        "collider": frame.collider,
        "freezeTimer": frame.freeze_timer,
        "wind": frame.wind,
        "windTarget": frame.wind_target,
        "windPattern": frame.wind_pattern,
        "transitioning": frame.transitioning,
        "coreMode": frame.core_mode.map(|mode| match mode {
            CoreMode::None => 0,
            CoreMode::Hot => 1,
            CoreMode::Cold => 2,
        }),
        "inSpace": frame.in_space,
        // Pre-consumption press edges (v3 traces only; `null` on v1/v2).
        "jumpP0": frame.input.jump_pressed_0,
        "dashP0": frame.input.dash_pressed_0,
        "cdashP0": frame.input.crouch_dash_pressed_0,
        "talkP0": frame.input.talk_pressed_0,
        "inventory": frame.inventory.as_ref().map(|inv| json!({
            "Dashes": inv.dashes,
            "DreamDash": inv.dream_dash,
            "Backpack": inv.backpack,
            "NoRefills": inv.no_refills,
        })),
        "stateTimer": Value::Null,
        "movementRemainder": truth.movement_counter,
        "collectResetTimer": truth.collect_reset_timer,
        "rawDt": frame.raw_dt,
        "dt": frame.dt,
    })
}

#[derive(Default)]
struct ReplayOutcome {
    exact_prefix: u64,
    replayed: u64,
    records: u64,
    leading_skipped: u64,
    stalled_frames: u64,
    stall_offsets: Vec<u64>,
    unavailable: Vec<String>,
    /// Present only with `--dump-segment`; one line per replayed frame with the
    /// per-frame `Actor.MoveH`/`MoveV` total recovered as
    /// `ΔPosition + ΔmovementCounter` on both sides. Diagnostic only.
    dump: Vec<String>,
    /// Replayed frames where the simulator's own post-entity geometric
    /// `on_ground` disagrees with its source-private `player_on_ground`.
    geometric_ground_diff: u64,
    /// Replayed frames where the simulator was not frozen although the trace
    /// shows `Player.Update` did not run (or vice versa).
    freeze_disagreement_frames: u64,
    first_freeze_disagreement: Option<u64>,
    /// Replayed frames where the simulator's `Level.Wind` (advanced by its own
    /// `WindController` ramp) differs from the trace's exported `Level.Wind`.
    /// Diagnostic only: the wind does not fail a segment on its own.
    wind_disagreement_frames: u64,
    /// Replayed frames where the simulator's `Player.Ducking` differs from the
    /// trace's exported `Player.Ducking` / active collider. Diagnostic only.
    ducking_disagreement_frames: u64,
    /// Row offsets where `Player.Update` did not run but the player snapshot
    /// changed anyway, i.e. rows produced by a coroutine that drives the player
    /// directly (`Level.Transitioning` -> `Player.TransitionTo`,
    /// `Level.FrozenOrPaused` -> `Cassette.CollectRoutine`). Such rows are
    /// stepped but not compared; the segment continues.
    frozen_mutation_frames: u64,
    frozen_mutation_offsets: Vec<u64>,
    /// Mechanism split of the stalled rows inside the replay window, from the v3
    /// exporter tail. Diagnostic only.
    stall_transition_frames: u64,
    stall_engine_freeze_frames: u64,
    stall_frozen_entity_frames: u64,
    status: &'static str,
    error: Option<String>,
    unsupported: Option<(String, u64)>,
    mismatch: Option<FirstMismatch>,
}

/// Replay one segment.
///
/// `remainder_override` replaces the anchor's `PlayerSnapshot::movement_remainder`
/// (a diagnostic used by `--probe-remainder`; the trace cannot observe
/// `Monocle.Actor.movementCounter`). `frame_cap` stops the replay early.
fn replay(
    segment: &Segment,
    map: &Map,
    remainder_override: Option<Vec2>,
    frame_cap: Option<usize>,
    dump: bool,
    carried_clutter: [bool; 3],
) -> ReplayOutcome {
    let row_count = segment.frames.len();
    let mut outcome = ReplayOutcome {
        records: row_count as u64,
        status: "skipped",
        ..Default::default()
    };
    if row_count < 2 {
        return outcome;
    }

    let truth: Vec<Truth> = segment
        .frames
        .iter()
        .map(|frame| Truth::read(frame.fields.as_ref()))
        .collect();

    // -----------------------------------------------------------------
    // Frames on which `Player.Update` did not run.
    //
    // `Player.Update` decrements `StrawberryCollectResetTimer` unconditionally
    // (before any state callback), so two consecutive rows carrying the same
    // value prove the entity update was skipped for that engine frame. That is
    // what happens on:
    //   * a room transition: `Level.Update` takes the `Transitioning` branch
    //     and updates only `Tags.TransitionUpdate` entities plus the transition
    //     coroutine, which drives `Player.TransitionTo` directly;
    //   * a `Celeste.Freeze(t)`: `Monocle.Engine.Update` skips `Scene.Update`
    //     entirely while `FreezeTimer > 0`.
    // Those rows cannot be replayed as ordinary input frames: the simulator
    // would run a full `Player.Update`. They are skipped, never stepped and
    // never counted as matched, and reported per segment.
    // -----------------------------------------------------------------
    let mut stalled = vec![false; row_count];
    for index in 1..row_count {
        stalled[index] = match (
            truth[index].collect_reset_timer,
            truth[index - 1].collect_reset_timer,
        ) {
            (Some(current), Some(previous)) => current == previous,
            _ => false,
        };
    }
    if let Some(previous) = segment.previous_timer {
        stalled[0] = truth[0]
            .collect_reset_timer
            .is_some_and(|value| value == previous);
    }

    // A replay window needs an anchor row that is a genuine post-`Player.Update`
    // capture plus a following live row, because the following row's
    // `PreviousPosition` is the only trace-derived position source.
    let mut window_start = None;
    for index in 1..row_count {
        if !stalled[index] && segment.frames[index].state_name.is_some() {
            window_start = Some(index);
            break;
        }
    }
    let Some(window_start) = window_start else {
        outcome.status = "no_live_window";
        outcome.error = Some(format!(
            "no row is a live `Player.Update` capture of a Player snapshot (rows={row_count})"
        ));
        return outcome;
    };
    outcome.leading_skipped = window_start as u64;
    outcome.stalled_frames = stalled[window_start + 1..]
        .iter()
        .filter(|is_stalled| **is_stalled)
        .count() as u64;
    for (offset, is_stalled) in stalled.iter().enumerate().skip(window_start + 1) {
        if *is_stalled {
            outcome.stall_offsets.push(offset as u64);
        }
    }

    // Build the anchor snapshot from the anchor row.
    let anchor = &segment.frames[window_start];
    let Some(anchor_state_name) = anchor.state_name.as_deref() else {
        outcome.status = "init_error";
        outcome.error = Some(format!(
            "anchor row {window_start} has no `state`/`p`: the trace row is a Level frame without a Player entity"
        ));
        return outcome;
    };
    let Some(anchor_state) = state_from_name(anchor_state_name) else {
        outcome.status = "state_map_error";
        outcome.error = Some(format!(
            "unknown state name {anchor_state_name:?} on anchor row {window_start}"
        ));
        return outcome;
    };
    let Some(anchor_fields) = anchor.fields.as_ref() else {
        outcome.status = "init_error";
        outcome.error = Some(format!(
            "anchor row {window_start} has no `p` object (no Player entity in the scene)"
        ));
        return outcome;
    };
    let mut snapshot = PlayerSnapshot::default();
    restore_restored_fields(&mut snapshot, anchor_fields);
    // `Player.MaxDashes` and `PlayerInventory.NoRefills` are session state, not
    // `Player` fields, so the reflection dump cannot carry them. Restore them
    // from the area's `PlayerInventory` (`AreaData`) as tightened by the
    // capacity the trace itself witnesses in this segment.
    snapshot.max_dashes = observe_session_dashes(segment.area, witnessed_session_dashes(map, &truth));
    snapshot.no_refills = area_inventory_no_refills(segment.area);
    snapshot.state = anchor_state;
    snapshot.facing = anchor_fields
        .get("Facing")
        .and_then(Value::as_i64)
        .map(|value| value >= 0)
        .unwrap_or(true);
    snapshot.time_rate = anchor.time_rate.map_or(1.0, |value| value as f32);
    snapshot.player_on_ground_initialized = true;
    // `Engine.DeltaTime` for the anchor frame: the same bits the simulator will
    // recompute on the next `step`, needed to convert the dash's
    // `dashAttackTimer` into the simulator's own `state_timer` (see
    // `PlayerSnapshot::restore_dash_phase`).
    let anchor_delta = anchor.raw_dt.unwrap_or(anchor.dt) as f32;
    snapshot.restore_dash_phase(anchor_delta * snapshot.time_rate);
    if let Some(remainder) = remainder_override {
        snapshot.movement_remainder = remainder;
    }
    match truth[window_start].position {
        Some(pos) => snapshot.pos = Vec2::new(pos[0] as f32, pos[1] as f32),
        None => {
            outcome.status = "init_error";
            outcome.error = Some(format!(
                "anchor row {window_start} has no `Position`, so the start position is unknown"
            ));
            return outcome;
        }
    }
    // -----------------------------------------------------------------
    // Append-only exporter tail (tools/celestetas-trace/TasFrameTrace.cs).
    //
    // Every value below is captured at the end of the anchor engine frame, which is
    // exactly the state the next `Player.Update` starts from, so it is restored
    // verbatim rather than inferred:
    //
    //  * `Level.Wind` (`Level.cs:149`) is the quantity `WindController.Update` ramps
    //    with `Calc.Approach(level.Wind, targetSpeed, 1000f * Engine.DeltaTime)`
    //    (`WindController.cs:194`) and then applies to the Player's own `WindMover`
    //    component (`Player.cs:1180`) as `level.Wind * 0.1f * Engine.DeltaTime`
    //    (`WindController.cs:199-201`). A room segment anchored mid-flight cannot
    //    reconstruct the cross-room ramp, so the anchored value must come from the
    //    trace. `WindController.targetSpeed` (`WindController.cs:47`) is restored
    //    with it so the simulator's ramp continues on the source's own target.
    //  * `Player.Ducking` (`Player.cs:1005-1028`) selects between `normalHitbox`
    //    (8x11) and `duckHitbox` (8x6); without it every collider probe reads the
    //    wrong box.
    //  * `Engine.FreezeTimer` (`Monocle/Engine.cs:28`) is the engine-level gate that
    //    makes `Engine.Update` skip `Scene.Update` (`Engine.cs:266-269`). The
    //    exporter writes the post-decrement value, which is precisely the snapshot
    //    state `Simulator::step` reads at the top of the next frame.
    //  * `Session.Inventory.DreamDash` (`Session.cs:35`, `PlayerInventory.cs:24`) is
    //    what `Player.Inventory.DreamDash` forwards (`Player.cs:956-966`) and what
    //    the simulator models as `PlayerSnapshot::can_dream_dash`
    //    (`Player.cs:3420,4500`). Restoring it removes the need to infer the flag
    //    from `dreamDashCanEndTimer`.
    // -----------------------------------------------------------------
    if let Some(wind) = anchor.wind {
        snapshot.wind = Vec2::new(wind[0] as f32, wind[1] as f32);
    } else {
        outcome
            .unavailable
            .push("wind: the trace row has no `Level.Wind`".to_owned());
    }
    if let Some(target) = anchor.wind_target {
        snapshot.wind_target = Vec2::new(target[0] as f32, target[1] as f32);
    } else {
        outcome.unavailable.push(
            "wind_target: the trace row has no `WindController.targetSpeed`; the simulator's own \
             windTrigger target is used for the ramp"
                .to_owned(),
        );
    }
    // `Player.Ducking` is the computed `Collider` property (`Player.cs:1005-1028`).
    // The exporter writes it directly on every row that has a Player, but
    // `p.wasDucking` (`Player.cs:461`) is an independent witness of the same
    // quantity: `Player.Update` assigns `wasDucking = Ducking` at
    // `Player.cs:1921-1924` on every frame it runs, so the two agree on every
    // post-`Player.Update` capture except the death early-return at
    // `Player.cs:1907`. Prefer the computed property, fall back to `wasDucking`
    // when a trace predates it, and surface a disagreement instead of hiding it.
    let was_ducking = anchor_fields.get("wasDucking").and_then(Value::as_bool);
    match (anchor.ducking, was_ducking) {
        (Some(ducking), Some(was)) => {
            snapshot.ducking = ducking;
            if ducking != was {
                outcome.unavailable.push(format!(
                    "ducking: the anchor row's computed `Player.Ducking` ({ducking}) disagrees \
                     with `p.wasDucking` ({was}); the death early-return at Player.cs:1907 is the \
                     only Player.Update path that leaves them different"
                ));
            }
        }
        (Some(ducking), None) => snapshot.ducking = ducking,
        (None, Some(was)) => {
            snapshot.ducking = was;
            outcome.unavailable.push(
                "ducking: the trace row has no computed `Player.Ducking`; fell back to \
                 `p.wasDucking` (Player.cs:1921-1924)"
                    .to_owned(),
            );
        }
        (None, None) => outcome
            .unavailable
            .push("ducking: the trace row has no `Player.Ducking`".to_owned()),
    }
    if let Some(freeze_timer) = anchor.freeze_timer {
        snapshot.freeze_timer = freeze_timer as f32;
    } else {
        outcome
            .unavailable
            .push("freeze_timer: the trace row has no `Engine.FreezeTimer`".to_owned());
    }
    match anchor.inventory.as_ref().and_then(|inv| inv.dream_dash) {
        Some(dream_dash) => snapshot.can_dream_dash = dream_dash,
        None => outcome
            .unavailable
            .push("can_dream_dash: the trace row has no `Session.Inventory.DreamDash`".to_owned()),
    }
    // `Celeste.Session.CoreMode` (`Session.cs:111`) is the session's chapter-9 mode; the Core's ice
    // factor (`Player.cs:3681-3684`) and the `CoreModeListener` entities read it. It is not a
    // `Player` field, so the base-chain dump cannot see it and the simulator would otherwise replay
    // every Core room with `CoreMode::None`.
    if let Some(core_mode) = anchor.core_mode {
        snapshot.core_mode = core_mode;
    } else {
        outcome
            .unavailable
            .push("core_mode: the trace row has no `Session.CoreMode`".to_owned());
    }
    // `Level.InSpace` is exported (`inSpace`) but deliberately not restored: there is no
    // `PlayerSnapshot::in_space` and `map.rs` does not decode the `.bin` level element's `space`
    // attribute, so the `* 0.6f` branches at `Player.cs:3703-3706,3718-3722,3778-3781` are
    // unimplemented. Reported rather than silently ignored.
    if anchor.in_space == Some(true) {
        outcome.unavailable.push(
            "InSpace: the room sets Level.InSpace (`Level.cs:449`) but PlayerSnapshot has no \
             `in_space`, so the Player.cs:3703-3706/3718-3722/3778-3781 `* 0.6f` branches are not \
             modelled"
                .to_owned(),
        );
    }

    let mut simulator = match Simulator::new(snapshot, map) {
        Ok(simulator) => simulator,
        Err(error) => {
            outcome.status = "init_error";
            outcome.error = Some(error.to_string());
            return outcome;
        }
    };
    // `oshiro_clutter_cleared_*` is chapter state that the trace cannot carry
    // (`ClutterSwitch.cs:138`, `ClutterBlockGenerator.cs:78-81`); restore the
    // caller's carried value before the first replayed frame.
    simulator.set_clutter_cleared(carried_clutter);

    let mut exact_prefix = 0u64;
    let mut replayed = 0u64;
    let mut geometric_ground_diff = 0u64;
    let mut freeze_disagreement = 0u64;
    let mut first_freeze_disagreement = None;
    let mut wind_disagreement = 0u64;
    let mut ducking_disagreement = 0u64;
    for index in window_start + 1..row_count {
        if frame_cap.is_some_and(|cap| replayed as usize >= cap) {
            outcome.status = "capped";
            outcome.exact_prefix = exact_prefix;
            outcome.replayed = replayed;
            outcome.geometric_ground_diff = geometric_ground_diff;
            outcome.freeze_disagreement_frames = freeze_disagreement;
            outcome.first_freeze_disagreement = first_freeze_disagreement;
            outcome.wind_disagreement_frames = wind_disagreement;
            outcome.ducking_disagreement_frames = ducking_disagreement;
            return outcome;
        }
        let frame = &segment.frames[index];
        let expected = &truth[index];
        let offset = (index - window_start - 1) as u64;
        replayed += 1;
        // Which engine mechanism skipped `Player.Update` on this row, read from
        // the v3 exporter tail: `Level.Transitioning` (`Level.cs:221`), a
        // positive `Engine.FreezeTimer` (`Engine.cs:266-269`), or neither - the
        // `Level.FrozenOrPaused` branch (`Level.cs:1837`) that `Cassette`,
        // `HeartGem`, `CSGEN_StrawberrySeeds` and `ForsakenCitySatellite` drive
        // with `level.Frozen = true`. Diagnostic only.
        if stalled[index] {
            match (frame.transitioning, frame.freeze_timer) {
                (Some(true), _) => outcome.stall_transition_frames += 1,
                (_, Some(timer)) if timer > 0.0 => outcome.stall_engine_freeze_frames += 1,
                _ => outcome.stall_frozen_entity_frames += 1,
            }
        }
        let frozen_mutation =
            stalled[index] && stalled_row_mutates_player(&segment.frames, &truth, index);
        // The trace says `Player.Update` did not run on this engine frame
        // (`Level.Transitioning` transition coroutine or `Celeste.Freeze`).
        // The simulator must therefore skip its own player update this frame,
        // which it does while `snapshot.freeze_timer > 0`. Stepping anyway
        // keeps `MInput`-equivalent VirtualButton buffers aligned.
        let simulator_frozen = simulator.snapshot().freeze_timer > 0.0;
        if simulator_frozen != stalled[index] {
            freeze_disagreement += 1;
            if first_freeze_disagreement.is_none() {
                first_freeze_disagreement = Some(offset);
            }
        }
        // `Engine.FreezeTimer` is not a `Player` field, so the trace cannot
        // export the freeze that started inside the anchor row (for example
        // `Player.DashBegin` -> `Celeste.Freeze(0.05f)`, or a collected
        // `Refill`). The stalled row is the only witness, and without this the
        // simulator runs a `Player.Update` the game skipped and then compares
        // its freshly computed fields against the game's frozen ones.
        if stalled[index] && !simulator_frozen {
            simulator.skip_engine_frame();
        }
        // A `ClutterSwitch` press inside a replayed frame: the ground truth
        // freezes the engine (`ClutterSwitch.cs:135`) and deactivates that
        // colour's `ClutterBlockBase` solids (`ClutterSwitch.cs:227-233`).
        if let Some(resting) = clutter_press_rect(frame) {
            simulator.press_clutter_switch(resting);
        }
        let delta = frame.raw_dt.unwrap_or(frame.dt);
        let input = frame.input.to_input_state(bits_of(delta));
        let before = simulator.snapshot().clone();
        match simulator.step(input) {
            Ok(actual) => {
                if dump {
                    // `Actor.MoveH`/`MoveV(amount)` leaves
                    // `position += round(counter + amount)` and
                    // `counter += amount - round(...)`, so the amount the frame
                    // fed to the actor is `Δposition + ΔmovementCounter` on each
                    // axis. Recovering it for both sides shows whether the
                    // simulator performed the same actor moves as the game.
                    let previous = if index > 0 {
                        &truth[index - 1]
                    } else {
                        expected
                    };
                    let game_move = match (
                        previous.position,
                        expected.position,
                        previous.movement_counter,
                        expected.movement_counter,
                    ) {
                        (Some(a), Some(b), Some(c), Some(d)) => {
                            format!(
                                "({:.5},{:.5})",
                                b[0] - a[0] + d[0] - c[0],
                                b[1] - a[1] + d[1] - c[1]
                            )
                        }
                        _ => "?".to_owned(),
                    };
                    let rust_move = format!(
                        "({:.5},{:.5})",
                        actual.pos.x - before.pos.x + actual.movement_remainder.x
                            - before.movement_remainder.x,
                        actual.pos.y - before.pos.y + actual.movement_remainder.y
                            - before.movement_remainder.y
                    );
                    outcome.dump.push(format!(
                        "row={} offset={offset} gamePos=({:.5},{:.5}) gameCounter=({:.5},{:.5}) gameMove={game_move} rustPos=({:.5},{:.5}) rustCounter=({:.5},{:.5}) rustMove={rust_move} gameState={:?} rustState={:?} stalled={} freeze={:.5}",
                        frame.n,
                        expected.position.map_or(f64::NAN, |p| p[0]),
                        expected.position.map_or(f64::NAN, |p| p[1]),
                        expected.movement_counter.map_or(f64::NAN, |p| p[0]),
                        expected.movement_counter.map_or(f64::NAN, |p| p[1]),
                        actual.pos.x,
                        actual.pos.y,
                        actual.movement_remainder.x,
                        actual.movement_remainder.y,
                        frame.state_name.as_deref().unwrap_or("?"),
                        state_name(actual.state),
                        stalled[index],
                        before.freeze_timer,
                    ));
                }
                if actual.on_ground != actual.player_on_ground {
                    geometric_ground_diff += 1;
                }
                // Diagnostic only: the source applies `Level.Wind * 0.1f *
                // Engine.DeltaTime` through the Player's own WindMover
                // (WindController.cs:199-201); this counts how far the simulator's
                // own ramp has drifted from the trace's `Level.Wind` since the
                // anchor.
                if let Some(wind) = frame.wind {
                    if (actual.wind.x - wind[0] as f32).abs() > TOLERANCE
                        || (actual.wind.y - wind[1] as f32).abs() > TOLERANCE
                    {
                        wind_disagreement += 1;
                    }
                }
                // Diagnostic only: `Player.Ducking` selects the active collider
                // (Player.cs:1005-1028).
                if let Some(ducking) = frame.ducking {
                    if actual.ducking != ducking {
                        ducking_disagreement += 1;
                    }
                }
                // The stall detector only proves that `Player.Update` did not run
                // on this row. It does not prove the row is a stale copy of the
                // previous one: both `Level.Update` branches that skip the player
                // still run other entities and coroutines, and some of those
                // mutate the player directly.
                //
                //  * `Level.Transitioning` (`Level.cs:1888-1896`) runs
                //    `transition.Update()`, which drives `Player.TransitionTo`
                //    (`Player.cs:2293-2308`) - that writes `Position`/`Speed`
                //    directly with no `Player.Update`.
                //  * `Level.FrozenOrPaused` (`Level.cs:1837-1868`) runs the
                //    `Tags.FrozenUpdate` entities. `Cassette.CollectRoutine`
                //    (`Cassette.cs:168-243`, tagged at `Cassette.cs:175`) calls
                //    `player.StartCassetteFly` (`Cassette.cs:231` ->
                //    `Player.cs:5590`), which sets `StateMachine.State = 21`
                //    (`StCassetteFly`) and starts the fly coroutine.
                //
                // The player itself is `Tags.Persistent` only (`Player.cs:1122`),
                // so those writes are the whole story: a stalled row whose player
                // fields differ from the previous row's cannot have been produced
                // by `Player.Update` at all. Comparing it would report a
                // divergence the simulator structurally cannot reproduce, so the
                // row is counted, not compared, and the segment stays alive to
                // catch the next genuine `Player.Update` mismatch.
                if frozen_mutation {
                    outcome.frozen_mutation_frames += 1;
                    outcome.frozen_mutation_offsets.push(offset);
                    continue;
                }
                let mut reasons: Vec<String> = Vec::new();
                // Position comes from the row *after* the one being diffed:
                // row k+1's PreviousPosition is the position at the end of
                // row k. A stalled following row makes it stale.
                let pos_source = truth[index].position;
                if let Some(game_pos) = pos_source {
                    let dx = (actual.pos.x - game_pos[0] as f32).abs();
                    let dy = (actual.pos.y - game_pos[1] as f32).abs();
                    if dx > TOLERANCE || dy > TOLERANCE {
                        reasons.push("pos".to_owned());
                    }
                }
                if let Some(speed) = expected.speed {
                    let dx = (actual.speed.x - speed[0] as f32).abs();
                    let dy = (actual.speed.y - speed[1] as f32).abs();
                    if dx > TOLERANCE || dy > TOLERANCE {
                        reasons.push("speed".to_owned());
                    }
                }
                if let Some(stamina) = expected.stamina {
                    if (actual.stamina - stamina as f32).abs() > TOLERANCE {
                        reasons.push("stamina".to_owned());
                    }
                }
                match frame.state_name.as_deref() {
                    Some(name) => match state_from_name(name) {
                        Some(wanted) => {
                            if actual.state != wanted {
                                reasons.push("state".to_owned());
                            }
                        }
                        None => {
                            outcome.status = "state_map_error";
                            outcome.error =
                                Some(format!("unknown state name {name:?} at frame offset {offset}"));
                            outcome.exact_prefix = exact_prefix;
                            outcome.replayed = replayed;
                            outcome.geometric_ground_diff = geometric_ground_diff;
                            outcome.freeze_disagreement_frames = freeze_disagreement;
                            outcome.first_freeze_disagreement = first_freeze_disagreement;
                            outcome.wind_disagreement_frames = wind_disagreement;
                            outcome.ducking_disagreement_frames = ducking_disagreement;
                            return outcome;
                        }
                    },
                    None => {
                        outcome.unavailable.push(format!(
                            "state: row for frame offset {offset} has no `state` name"
                        ));
                    }
                }
                if let Some(facing) = expected.facing {
                    if actual.facing != (facing >= 0) {
                        reasons.push("facing".to_owned());
                    }
                }
                if let Some(dashes) = expected.dashes {
                    if i64::from(actual.dashes) != dashes {
                        reasons.push("dashes".to_owned());
                    }
                }
                if let Some(on_ground) = expected.on_ground {
                    // The trace's `p.onGround` is `Celeste.Player.onGround`, the
                    // source-private field `Player.Update` writes behind a
                    // `Speed.Y >= 0` gate (`Player.cs`). Its exact counterpart is
                    // `PlayerSnapshot::player_on_ground`, not the simulator's
                    // post-entity geometric `PlayerSnapshot::on_ground`.
                    if actual.player_on_ground != on_ground {
                        reasons.push("on_ground".to_owned());
                    }
                }
                if let Some(dead) = expected.dead {
                    if actual.dead != dead {
                        reasons.push("dead".to_owned());
                    }
                }

                if reasons.is_empty() {
                    exact_prefix += 1;
                    continue;
                }

                outcome.status = "mismatch";
                outcome.exact_prefix = exact_prefix;
                outcome.replayed = replayed;
                outcome.geometric_ground_diff = geometric_ground_diff;
                outcome.freeze_disagreement_frames = freeze_disagreement;
                outcome.first_freeze_disagreement = first_freeze_disagreement;
                outcome.wind_disagreement_frames = wind_disagreement;
                outcome.ducking_disagreement_frames = ducking_disagreement;
                outcome.mismatch = Some(FirstMismatch {
                    offset,
                    reasons,
                    rust: rust_view(actual),
                    game: game_view(frame, expected, pos_source),
                });
                return outcome;
            }
            Err(SimulationError::UnsupportedState(state)) => {
                let name = state_name(state);
                outcome.status = "unsupported";
                outcome.exact_prefix = exact_prefix;
                outcome.replayed = replayed;
                outcome.geometric_ground_diff = geometric_ground_diff;
                outcome.freeze_disagreement_frames = freeze_disagreement;
                outcome.first_freeze_disagreement = first_freeze_disagreement;
                outcome.wind_disagreement_frames = wind_disagreement;
                outcome.ducking_disagreement_frames = ducking_disagreement;
                outcome.error = Some(format!(
                    "SimulationError::UnsupportedState({name}) while executing frame offset {offset}"
                ));
                outcome.unsupported = Some((name.to_owned(), offset));
                return outcome;
            }
            Err(error) => {
                outcome.status = "sim_error";
                outcome.exact_prefix = exact_prefix;
                outcome.replayed = replayed;
                outcome.geometric_ground_diff = geometric_ground_diff;
                outcome.freeze_disagreement_frames = freeze_disagreement;
                outcome.first_freeze_disagreement = first_freeze_disagreement;
                outcome.wind_disagreement_frames = wind_disagreement;
                outcome.ducking_disagreement_frames = ducking_disagreement;
                outcome.error = Some(format!("frame offset {offset}: {error}"));
                return outcome;
            }
        }
    }

    outcome.status = "ok";
    outcome.exact_prefix = exact_prefix;
    outcome.replayed = replayed;
    outcome.geometric_ground_diff = geometric_ground_diff;
    outcome.freeze_disagreement_frames = freeze_disagreement;
    outcome.first_freeze_disagreement = first_freeze_disagreement;
    outcome.wind_disagreement_frames = wind_disagreement;
    outcome.ducking_disagreement_frames = ducking_disagreement;
    outcome
}

struct SegmentOutcome {
    report: SegmentReport,
    unsupported_state: Option<String>,
    status: &'static str,
}

/// The collider rectangle of a frame on which the ground truth shows a
/// `ClutterSwitch` being pressed.
///
/// `ClutterSwitch.OnDashed` is the only writer of the session's
/// `oshiro_clutter_cleared_<color>` flags (`ClutterSwitch.cs:131-156`), and it
/// freezes the engine for exactly 0.2 s (`:135`) while `AbsorbRoutine`
/// switches the player to `StDummy` (`:190`) on that same frame, leaving the
/// player resting on the switch's own Solid (`:49-50`). The exported collider
/// is that resting rectangle, moved one pixel down so it touches the switch.
fn clutter_press_rect(frame: &Frame) -> Option<Rect> {
    if frame.state_name.as_deref() != Some("StDummy") {
        return None;
    }
    if !frame
        .freeze_timer
        .is_some_and(|freeze| (freeze - 0.2).abs() < 0.005)
    {
        return None;
    }
    let collider = frame.collider?;
    Some(Rect::new(
        collider[0] as f32,
        collider[1] as f32 + 1.0,
        collider[2] as f32,
        collider[3] as f32,
    ))
}

fn simulate_segment(
    segment: &Segment,
    map: &Map,
    area_file: Option<String>,
    dump: bool,
) -> SegmentOutcome {
    // Carry `oshiro_clutter_cleared_*` across the segments of one chapter, and
    // reset it when the trace shows the chapter session restarted (its clock
    // moved backwards).
    Y3_CHAPTER.with(|chapter| *chapter.borrow_mut() = (segment.sid.clone(), segment.mode));
    let session = segment
        .frames
        .first()
        .map(|frame| (frame.deaths, frame.level_time));
    let carried = Y3_CLUTTER.with(|carry| {
        let mut carry = carry.borrow_mut();
        let entry = carry
            .entry((segment.sid.clone(), segment.mode))
            .or_default();
        if let Some((Some(deaths), Some(level_time))) = session {
            if entry.seen && (deaths < entry.deaths || level_time < entry.level_time) {
                entry.cleared = [false; 3];
            }
            entry.deaths = deaths;
            entry.level_time = level_time;
            entry.seen = true;
        }
        entry.cleared
    });
    Y3_CARRIED.with(|carry| *carry.borrow_mut() = carried);
    // A `ClutterSwitch` press inside this segment clears its colour for the
    // rest of the chapter even when the replay diverges before reaching it, so
    // the presses are folded into the carried flags for the next segment.
    let mut next = carried;
    for frame in &segment.frames {
        if let Some(rect) = clutter_press_rect(frame) {
            if let Some(color) = clutter_switch_color_at(map, rect) {
                next[color] = true;
            }
        }
    }
    Y3_CLUTTER.with(|carry| {
        let chapter = Y3_CHAPTER.with(|c| c.borrow().clone());
        carry.borrow_mut().entry(chapter).or_default().cleared = next;
    });
    let outcome = replay(segment, map, None, None, dump, carried);
    let report = SegmentReport {
        sid: segment.sid.clone(),
        mode: segment.mode,
        room: segment.room.clone(),
        area_file,
        start_row: segment.start_row,
        start_frame: segment.start_frame,
        frames: outcome.replayed,
        records: outcome.records,
        status: outcome.status.to_owned(),
        exact_prefix_frames: outcome.exact_prefix,
        leading_skipped_frames: outcome.leading_skipped,
        stalled_frames: outcome.stalled_frames,
        stall_offsets: outcome.stall_offsets,
        error: outcome.error,
        unsupported_state: outcome.unsupported.as_ref().map(|(name, _)| name.clone()),
        unsupported_offset: outcome.unsupported.as_ref().map(|(_, offset)| *offset),
        first_mismatch: outcome.mismatch,
        unavailable_checks: outcome.unavailable,
        geometric_ground_diff_frames: outcome.geometric_ground_diff,
        freeze_disagreement_frames: outcome.freeze_disagreement_frames,
        first_freeze_disagreement_offset: outcome.first_freeze_disagreement,
        wind_disagreement_frames: outcome.wind_disagreement_frames,
        ducking_disagreement_frames: outcome.ducking_disagreement_frames,
        frozen_mutation_frames: outcome.frozen_mutation_frames,
        frozen_mutation_offsets: outcome.frozen_mutation_offsets,
        stall_transition_frames: outcome.stall_transition_frames,
        stall_engine_freeze_frames: outcome.stall_engine_freeze_frames,
        stall_frozen_entity_frames: outcome.stall_frozen_entity_frames,
        remainder_probe: None,
        dump: outcome.dump,
    };
    SegmentOutcome {
        report,
        unsupported_state: outcome.unsupported.map(|(name, _)| name),
        status: outcome.status,
    }
}

/// Diagnostic: the anchor's `PlayerSnapshot::movement_remainder` cannot be read
/// from the trace (`Monocle.Actor.movementCounter` is private and is not a
/// declared field of `Celeste.Player`), so this sweeps one axis at a time over
/// a 1/64 grid on `[-0.5, 0.5]` and reports how far the segment replays for each
/// candidate. A candidate that unlocks a long prefix proves the divergence was
/// pure `Actor.MoveH`/`MoveV` rounding rather than a physics discrepancy.
/// `--probe-remainder` is a diagnostic that re-anchors the same segment with
/// swept remainders; it reuses the chapter's carried clutter flags.
fn probe_remainder(
    segment: &Segment,
    map: &Map,
    frame_cap: usize,
    carried: [bool; 3],
) -> Value {
    const STEPS: i32 = 32;
    let baseline = replay(segment, map, None, Some(frame_cap), false, carried);
    let mut axes = Vec::new();
    for (axis, name) in [(0usize, "x"), (1usize, "y")] {
        let mut best: Option<(ReplayOutcome, f32)> = None;
        for step in -STEPS..=STEPS {
            let value = step as f32 / (2.0 * STEPS as f32);
            let remainder = if axis == 0 {
                Vec2::new(value, 0.0)
            } else {
                Vec2::new(0.0, value)
            };
            let result = replay(segment, map, Some(remainder), Some(frame_cap), false, carried);
            if best
                .as_ref()
                .is_none_or(|(current, _)| result.exact_prefix > current.exact_prefix)
            {
                best = Some((result, value));
            }
        }
        if let Some((result, value)) = best {
            axes.push(json!({
                "axis": name,
                "gridStep": 1.0 / (2.0 * STEPS as f32),
                "bestValue": value,
                "bestExactPrefixFrames": result.exact_prefix,
                "bestReplayedFrames": result.replayed,
                "bestStatus": result.status,
                "bestError": result.error,
                "bestMismatch": result.mismatch,
            }));
        }
    }
    let mut best2d: Option<(ReplayOutcome, f32, f32)> = None;
    const STEPS2D: i32 = 16;
    for x_step in -STEPS2D..=STEPS2D {
        for y_step in -STEPS2D..=STEPS2D {
            let x = x_step as f32 / (2.0 * STEPS2D as f32);
            let y = y_step as f32 / (2.0 * STEPS2D as f32);
            let result = replay(segment, map, Some(Vec2::new(x, y)), Some(frame_cap), false, carried);
            if best2d
                .as_ref()
                .is_none_or(|(current, _, _)| result.exact_prefix > current.exact_prefix)
            {
                best2d = Some((result, x, y));
            }
        }
    }
    let best2d = best2d.map(|(result, x, y)| {
        json!({
            "x": x,
            "y": y,
            "gridStep": 1.0 / (2.0 * STEPS2D as f32),
            "exactPrefixFrames": result.exact_prefix,
            "replayedFrames": result.replayed,
            "status": result.status,
            "error": result.error,
            "mismatch": result.mismatch,
        })
    });

    json!({
        "sweep": "one axis at a time (1/64 over [-0.5, 0.5], other axis pinned to 0) plus a joint sweep (1/32 over [-0.5, 0.5] on both axes)",
        "frameCap": frame_cap,
        "baselineExactPrefixFrames": baseline.exact_prefix,
        "baselineStatus": baseline.status,
        "axes": axes,
        "best2d": best2d,
    })
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

struct Args {
    trace: PathBuf,
    maps: PathBuf,
    out: PathBuf,
    min_frames: usize,
    limit_segments: Option<usize>,
    max_frames: Option<usize>,
    rooms: Vec<String>,
    dump_field_map: bool,
    /// Probe the first N mismatch segments with `probe_remainder`.
    probe_remainder: usize,
    /// Frame cap for each probe replay.
    probe_frames: usize,
    /// `sid|mode|room|startRow` of one segment to dump frame by frame.
    dump_segment: Option<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut trace: Option<PathBuf> = None;
    let mut maps: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut min_frames = 1usize;
    let mut limit_segments = None;
    let mut max_frames = None;
    let mut rooms = Vec::new();
    let mut dump_field_map = false;
    let mut probe_remainder = 0usize;
    let mut probe_frames = 200usize;
    let mut dump_segment = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| -> Result<String, String> {
            args.next().ok_or_else(|| format!("{name} requires a value"))
        };
        match arg.as_str() {
            "--trace" => trace = Some(PathBuf::from(value("--trace")?)),
            "--maps" => maps = Some(PathBuf::from(value("--maps")?)),
            "--out" => out = Some(PathBuf::from(value("--out")?)),
            "--min-frames" => {
                min_frames = value("--min-frames")?
                    .parse()
                    .map_err(|error| format!("--min-frames: {error}"))?
            }
            "--limit-segments" => {
                limit_segments = Some(
                    value("--limit-segments")?
                        .parse()
                        .map_err(|error| format!("--limit-segments: {error}"))?,
                )
            }
            "--max-frames" => {
                max_frames = Some(
                    value("--max-frames")?
                        .parse()
                        .map_err(|error| format!("--max-frames: {error}"))?,
                )
            }
            "--rooms" => {
                rooms = value("--rooms")?
                    .split(',')
                    .map(str::trim)
                    .filter(|room| !room.is_empty())
                    .map(str::to_owned)
                    .collect()
            }
            "--dump-field-map" => dump_field_map = true,
            "--probe-remainder" => {
                probe_remainder = value("--probe-remainder")?
                    .parse()
                    .map_err(|error| format!("--probe-remainder: {error}"))?
            }
            "--probe-frames" => {
                probe_frames = value("--probe-frames")?
                    .parse()
                    .map_err(|error| format!("--probe-frames: {error}"))?
            }
            "--dump-segment" => dump_segment = Some(value("--dump-segment")?),
            "-h" | "--help" => {
                println!(
                    "usage: tas_fidelity --trace <jsonl> --maps <dir> --out <report.json> \
                     [--min-frames 1] [--limit-segments N] [--max-frames N] [--rooms a,b] \
                     [--probe-remainder N] [--probe-frames N] [--dump-segment sid|mode|room|startRow] \
                     [--dump-field-map]"
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument {other:?}")),
        }
    }

    if dump_field_map {
        return Ok(Args {
            trace: trace.unwrap_or_default(),
            maps: maps.unwrap_or_default(),
            out: out.unwrap_or_default(),
            min_frames,
            limit_segments,
            max_frames,
            rooms,
            dump_field_map,
            probe_remainder,
            probe_frames,
            dump_segment,
        });
    }

    Ok(Args {
        trace: trace.ok_or("--trace is required")?,
        maps: maps.ok_or("--maps is required")?,
        out: out.ok_or("--out is required")?,
        min_frames,
        limit_segments,
        max_frames,
        rooms,
        dump_field_map,
        probe_remainder,
        probe_frames,
        dump_segment,
    })
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FieldMapEntry {
    field: String,
    kind: String,
    source: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FieldMap {
    restored: Vec<FieldMapEntry>,
    derived: Vec<FieldMapEntry>,
    unrestored: Vec<FieldMapEntry>,
    audit: CoverageAudit,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    trace: String,
    maps_dir: String,
    generated_by: String,
    comparison: Value,
    notes: Vec<String>,
    totals: Totals,
    by_unsupported_state: BTreeMap<String, StateStat>,
    rooms: BTreeMap<String, RoomStat>,
    segments: Vec<SegmentReport>,
    field_map: FieldMap,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("tas_fidelity: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    if args.dump_field_map {
        print!("{}", snapshot_fields_markdown());
        return Ok(());
    }

    let file = File::open(&args.trace)
        .map_err(|error| format!("cannot open trace {:?}: {error}", args.trace))?;
    let mut reader = BufReader::with_capacity(1 << 20, file);

    let mut totals = Totals::default();
    let mut rooms: BTreeMap<String, RoomAccumulator> = BTreeMap::new();
    let mut by_unsupported: BTreeMap<String, StateStat> = BTreeMap::new();
    let mut segments: Vec<SegmentReport> = Vec::new();
    let mut cache = MapCache::default();

    let mut current: Option<Segment> = None;
    let mut previous_level_row: Option<(u64, Option<f64>)> = None;
    let mut processed = 0usize;
    let mut probe_budget = args.probe_remainder;
    let mut line = String::new();

    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .map_err(|error| format!("read error: {error}"))?;
        if read == 0 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        totals.records += 1;
        let record: Record = match serde_json::from_str(&line) {
            Ok(record) => record,
            Err(error) => {
                totals.parse_errors += 1;
                if totals.parse_errors <= 5 {
                    eprintln!("tas_fidelity: cannot parse row {}: {error}", totals.records);
                }
                continue;
            }
        };

        let key = if record.scene == "Level" {
            Some((
                record.sid.clone().unwrap_or_default(),
                record.mode.unwrap_or(0),
                record.room.clone().unwrap_or_default(),
            ))
        } else {
            None
        };

        let matches_current = match (&current, &key) {
            (Some(segment), Some((sid, mode, room))) => {
                segment.sid == *sid && segment.mode == *mode && segment.room == *room
            }
            _ => false,
        };

        if !matches_current
            && let Some(segment) = current.take()
        {
            finish_segment(
                segment,
                &args,
                &mut cache,
                &mut totals,
                &mut rooms,
                &mut by_unsupported,
                &mut segments,
                &mut processed,
                &mut probe_budget,
            );
        }

        let Some((sid, mode, room)) = key else {
            totals.non_level_records += 1;
            continue;
        };
        totals.level_records += 1;

        let timer = record
            .p
            .as_ref()
            .and_then(|fields| float_field(fields, "StrawberryCollectResetTimer"));

        if current.is_none() {
            let previous_timer = previous_level_row
                .filter(|(row, _)| *row + 1 == record.n)
                .and_then(|(_, timer)| timer);
            current = Some(Segment {
                sid,
                mode,
                area: record.area.unwrap_or(0),
                room,
                start_row: record.n,
                start_frame: record.f,
                previous_timer,
                frames: Vec::new(),
            });
        }
        previous_level_row = Some((record.n, timer));

        let segment = current.as_mut().expect("segment open");
        if args
            .max_frames
            .is_some_and(|max| segment.frames.len() >= max)
        {
            continue;
        }
        segment.frames.push(Frame {
            n: record.n,
            f: record.f,
            dt: record.dt,
            raw_dt: record.raw_dt,
            time_rate: record.time_rate,
            input: record.input,
            state_name: record.state,
            fields: record.p,
            wind: pair_field(record.wind.as_ref()),
            wind_target: pair_field(record.wind_target.as_ref()),
            wind_pattern: record.wind_pattern,
            transitioning: record.transitioning,
            freeze_timer: record.freeze_timer,
            ducking: record.ducking,
            collider: quad_field(record.collider.as_ref()),
            inventory: record.inventory,
            core_mode: record.core_mode.and_then(core_mode_from_int),
            in_space: record.in_space,
            deaths: record.deaths,
            level_time: record.level_time,
        });

        if args.limit_segments.is_some_and(|limit| processed >= limit) {
            totals.truncated = true;
            break;
        }
    }

    if let Some(segment) = current.take() {
        finish_segment(
            segment,
            &args,
            &mut cache,
            &mut totals,
            &mut rooms,
            &mut by_unsupported,
            &mut segments,
            &mut processed,
            &mut probe_budget,
        );
    }

    let mut room_stats = BTreeMap::new();
    for (key, accumulator) in &rooms {
        room_stats.insert(key.clone(), accumulator.finish());
    }

    let field_map = FieldMap {
        restored: RESTORED_FIELDS
            .iter()
            .map(|field| FieldMapEntry {
                field: field.field.to_owned(),
                kind: field.ty.to_owned(),
                source: format!("p.{}", field.source),
            })
            .collect(),
        derived: DERIVED_FIELDS
            .iter()
            .map(|(field, note)| FieldMapEntry {
                field: (*field).to_owned(),
                kind: "derived".to_owned(),
                source: (*note).to_owned(),
            })
            .collect(),
        unrestored: unrestored_fields()
            .into_iter()
            .map(|(field, note)| FieldMapEntry {
                field: field.to_owned(),
                kind: "none".to_owned(),
                source: note.to_owned(),
            })
            .collect(),
        audit: coverage_audit(),
    };

    let report = Report {
        trace: args.trace.display().to_string(),
        maps_dir: args.maps.display().to_string(),
        generated_by: format!(
            "cargo run -p celeste-physics --release --example tas_fidelity -- --trace {} --maps {} --out {} --min-frames {}{}{}{}",
            args.trace.display(),
            args.maps.display(),
            args.out.display(),
            args.min_frames,
            args.limit_segments
                .map(|value| format!(" --limit-segments {value}"))
                .unwrap_or_default(),
            args.max_frames
                .map(|value| format!(" --max-frames {value}"))
                .unwrap_or_default(),
            if args.rooms.is_empty() {
                String::new()
            } else {
                format!(" --rooms {}", args.rooms.join(","))
            },
        ),
        comparison: json!({
            "positionSource": "the row's own `p.Position` (`Monocle.Entity.Position`), compared directly against `PlayerSnapshot::pos`",
            "tolerance": 0.01_f64,
            "toleranceNote": "compared as 0.01f32 (the f32 nearest 0.01, i.e. 0.009999999776482582)",
            "exact": ["state", "facing", "dashes", "on_ground", "dead"],
            "unavailable": {
                "ducking": "Player.Ducking is a computed Collider property, not an exported field",
                "state_timer": "Player.StateMachine.Timer is not exported (non-primitive field)",
                "camera": "Level.Camera.Position is not exported",
                "freeze_timer": "Engine.FreezeTimer is not a Player field; skipped Scene.Update frames are detected and skipped instead",
            },
            "frameDeltaTime": "input[j] carries frames[j+1].rawDt when present. Monocle computes `Engine.DeltaTime = RawDeltaTime * TimeRate * TimeRateB` and `Simulator::step` multiplies the supplied bits by `snapshot.time_rate`, so the raw delta is the correct input.",
            "stalledFrames": "Rows where `StrawberryCollectResetTimer` is unchanged from the previous row: `Player.Update` did not run (Level.Transitioning transition coroutine, or Celeste.Freeze). Leading stalled rows (the room transition) are skipped because the trace does not export the transition coroutine state; stalled rows inside the replay window ARE stepped so the simulator's own Celeste.Freeze model and VirtualButton buffers stay aligned, and `freezeDisagreementFrames` counts frames where the simulator's freeze state disagreed with the trace.",
        }),
        notes: vec![
            "Segments are maximal runs of consecutive rows with scene == \"Level\" and identical (sid, mode, room).".to_owned(),
            "The segment's first live row is the anchor: its `p` restores the initial PlayerSnapshot and the next row's `p.PreviousPosition` restores the start position. Inputs come from the rows after the anchor, so replaying input i reproduces the row it came from.".to_owned(),
            "exactPrefixFrames counts consecutive replayed (live) frames that matched from the anchor; replay stops at the first mismatch, unsupported state or simulator error.".to_owned(),
            "`frames` is the number of replayed input frames; `records` is the number of trace rows in the segment; `leadingSkippedFrames` + `stalledFrames` account for rows where `Player.Update` did not run.".to_owned(),
        ],
        totals,
        by_unsupported_state: by_unsupported,
        rooms: room_stats,
        segments,
        field_map,
    };

    if let Some(parent) = args.out.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {:?}: {error}", parent))?;
    }
    let mut writer = std::io::BufWriter::new(
        File::create(&args.out).map_err(|error| format!("cannot create {:?}: {error}", args.out))?,
    );
    serde_json::to_writer_pretty(&mut writer, &report)
        .map_err(|error| format!("cannot serialize report: {error}"))?;
    writer
        .write_all(b"\n")
        .map_err(|error| format!("cannot write report: {error}"))?;
    writer
        .flush()
        .map_err(|error| format!("cannot flush report: {error}"))?;

    if !report.field_map.audit.missing.is_empty() || !report.field_map.audit.stale.is_empty() {
        eprintln!(
            "tas_fidelity: WARNING field coverage table is out of date: missing={:?} stale={:?}",
            report.field_map.audit.missing, report.field_map.audit.stale
        );
    }

    println!(
        "segments={} simulated={} ok={} mismatch={} unsupported={} init_error={} map_error={} state_map_error={} no_live_window={} sim_error={} skipped={} frames={} exact={} stalled={}",
        report.totals.segments,
        report.totals.simulated_segments,
        report.totals.ok,
        report.totals.mismatch,
        report.totals.unsupported,
        report.totals.init_error,
        report.totals.map_error,
        report.totals.state_map_error,
        report.totals.no_live_window,
        report.totals.sim_error,
        report.totals.skipped,
        report.totals.total_frames,
        report.totals.exact_frames,
        report.totals.stalled_frames,
    );
    println!("report written to {}", args.out.display());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn finish_segment(
    segment: Segment,
    args: &Args,
    cache: &mut MapCache,
    totals: &mut Totals,
    rooms: &mut BTreeMap<String, RoomAccumulator>,
    by_unsupported: &mut BTreeMap<String, StateStat>,
    segments: &mut Vec<SegmentReport>,
    processed: &mut usize,
    probe_budget: &mut usize,
) {
    totals.segments += 1;
    if segment.frames.len() < args.min_frames.max(2) {
        totals.skipped += 1;
        return;
    }
    if !args.rooms.is_empty() && !args.rooms.iter().any(|room| *room == segment.room) {
        totals.skipped += 1;
        return;
    }

    let room_key = segment.key();
    let lookup = cache.get(&args.maps, &segment.sid, segment.mode, &segment.room);
    let dump = args.dump_segment.as_deref().is_some_and(|wanted| {
        let full = format!(
            "{}|{}|{}|{}",
            segment.sid, segment.mode, segment.room, segment.start_row
        );
        let short = format!(
            "{}|{}|{}|{}",
            segment.sid.trim_start_matches("Celeste/"),
            segment.mode,
            segment.room,
            segment.start_row
        );
        wanted == full || wanted == short
    });
    let outcome = match lookup {
        MapLookup::Ready(area_file, map) => {
            let mut outcome = simulate_segment(&segment, &map, Some(area_file), dump);
            for line in &outcome.report.dump {
                println!("{line}");
            }
            if *probe_budget > 0 && outcome.status == "mismatch" {
                *probe_budget -= 1;
                // The probe re-anchors this same segment, so it starts from the
                // flags the chapter carried *into* it.
                let carried = Y3_CARRIED.with(|carry| *carry.borrow());
                outcome.report.remainder_probe =
                    Some(probe_remainder(&segment, &map, args.probe_frames, carried));
            }
            outcome
        }
        MapLookup::Failed(message) => SegmentOutcome {
            report: SegmentReport {
                sid: segment.sid.clone(),
                mode: segment.mode,
                room: segment.room.clone(),
                area_file: None,
                start_row: segment.start_row,
                start_frame: segment.start_frame,
                frames: 0,
                records: segment.frames.len() as u64,
                status: "map_error".to_owned(),
                exact_prefix_frames: 0,
                leading_skipped_frames: 0,
                stalled_frames: 0,
                stall_offsets: Vec::new(),
                error: Some(message),
                unsupported_state: None,
                unsupported_offset: None,
                first_mismatch: None,
                unavailable_checks: Vec::new(),
                geometric_ground_diff_frames: 0,
                freeze_disagreement_frames: 0,
                first_freeze_disagreement_offset: None,
                wind_disagreement_frames: 0,
                ducking_disagreement_frames: 0,
                frozen_mutation_frames: 0,
                frozen_mutation_offsets: Vec::new(),
                stall_transition_frames: 0,
                stall_engine_freeze_frames: 0,
                stall_frozen_entity_frames: 0,
                remainder_probe: None,
                dump: Vec::new(),
            },
            unsupported_state: None,
            status: "map_error",
        },
    };

    let room = rooms.entry(room_key).or_default();
    room.segments += 1;
    room.frames += outcome.report.frames;

    *processed += 1;
    totals.simulated_segments += 1;
    totals.total_frames += outcome.report.frames;
    totals.exact_frames += outcome.report.exact_prefix_frames;
    totals.stalled_frames += outcome.report.stalled_frames;
    match outcome.status {
        "ok" => {
            totals.ok += 1;
            room.ok += 1;
        }
        "mismatch" => {
            totals.mismatch += 1;
            room.mismatch += 1;
        }
        "unsupported" => {
            totals.unsupported += 1;
            room.unsupported += 1;
        }
        "init_error" => {
            totals.init_error += 1;
            room.other += 1;
        }
        "map_error" => {
            totals.map_error += 1;
            room.other += 1;
        }
        "state_map_error" => {
            totals.state_map_error += 1;
            room.other += 1;
        }
        "no_live_window" => {
            totals.no_live_window += 1;
            room.other += 1;
        }
        _ => {
            totals.sim_error += 1;
            room.other += 1;
        }
    }
    room.exact_prefix.push(outcome.report.exact_prefix_frames);

    if let Some(state) = &outcome.unsupported_state {
        // `rowsInState` counts rows whose state name equals the unsupported
        // state, so the histogram can be cross-checked against the trace.
        let rows = segment
            .frames
            .iter()
            .filter(|frame| frame.state_name.as_deref() == Some(state.as_str()))
            .count() as u64;
        let entry = by_unsupported.entry(state.clone()).or_default();
        entry.segments += 1;
        entry.frames += outcome.report.frames;
        entry.rows_in_state += rows;
    }

    segments.push(outcome.report);
}
