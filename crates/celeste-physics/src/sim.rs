use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{EntityKind, InputState, Map, PlayerSnapshot, PlayerState, Rect, Vec2};

// FNA's fixed GameTime uses a 100 ns TimeSpan tick. A 60 Hz target is thus
// 166_667 ticks, exposed to Monocle as this f32 rather than exact 1 / 60.
pub const DT: f32 = 0.016_666_7;
const MAX_FALL: f32 = 160.0;
const FAST_MAX_FALL: f32 = 240.0;
const GRAVITY: f32 = 900.0;
const HALF_GRAV_THRESHOLD: f32 = 40.0;
const FAST_MAX_ACCEL: f32 = 300.0;
const MAX_RUN: f32 = 90.0;
const RUN_ACCEL: f32 = 1000.0;
const RUN_REDUCE: f32 = 400.0;
const DUCK_FRICTION: f32 = 500.0;
const AIR_MULT: f32 = 0.65;
/// `Player.NormalUpdate`'s Core ice-mode factor (`Player.cs:3681-3684`):
/// `num2 *= 0.3f` while `onGround` and `level.CoreMode == Cold`.
const ICE_GROUND_MULT: f32 = 0.3;
const JUMP_GRACE: f32 = 0.1;
const JUMP_BUFFER_TIME: f32 = 0.08;
const JUMP_SPEED: f32 = -105.0;
const JUMP_H_BOOST: f32 = 40.0;
const VAR_JUMP_TIME: f32 = 0.2;
const JUMP_THRU_ASSIST_SPEED: f32 = -40.0;
const WALL_JUMP_H: f32 = 130.0;
const WALL_JUMP_CHECK_DIST: f32 = 3.0;
/// `Player.SuperWallJumpCheckDist` (`Player.cs:179`).
const SUPER_WALL_JUMP_CHECK_DIST: f32 = 5.0;
/// `Player.WallJumpForceTime` (`Player.cs:181`).
const WALL_JUMP_FORCE_TIME: f32 = 0.16;
/// `Player.GliderWallJumpForceTime` (`Player.cs:345`).
const GLIDER_WALL_JUMP_FORCE_TIME: f32 = 0.26;
/// `Player.LaunchedBoostCheckSpeedSq` (`Player.cs:659`).
const LAUNCHED_BOOST_CHECK_SPEED_SQ: f32 = 10000.0;
/// The 220^2 speed term of `Player.LaunchedBoostCheck` (`Player.cs:2355`).
const LAUNCHED_SPEED_SQ: f32 = 48400.0;
const WALL_SLIDE_START_MAX: f32 = 20.0;
const WALL_SLIDE_TIME: f32 = 1.2;
const DASH_SPEED: f32 = 240.0;
const END_DASH_SPEED: f32 = 160.0;
const DASH_TIME: f32 = 0.15;
const DASH_COOLDOWN: f32 = 0.2;
const DASH_ATTACK_TIME: f32 = 0.3;
/// `Celeste.Freeze(0.05f)` in `Player.DashBegin` (`Player.cs:4282-4285`).
const DASH_FREEZE_TIME: f32 = 0.05;
/// `Player.SpacePhysicsMult` (`Player.cs:673`), the 0.6 scale the Core's zero-gravity
/// rooms apply to the run target, both fall caps and gravity.
const SPACE_PHYSICS_MULT: f32 = 0.6;
/// `Celeste.Freeze(0.05f)` from `CoreModeToggle.OnPlayer` (`CoreModeToggle.cs:123`),
/// the same 0.05 s hold a dash uses but raised by a different mechanic.
const CORE_TOGGLE_FREEZE_TIME: f32 = 0.05;
const DASH_CORNER_CORRECTION: i32 = 4;
/// `Player.cs:1791` probes `Position + Vector2.UnitY * 3f` for the dash's
/// downward corner close.
const DASH_CORRECT_DISTANCE: f32 = 3.0;
const SUPER_JUMP_H: f32 = 260.0;
const SUPER_BOUNCE_SPEED: f32 = -185.0;
const BOUNCE_SPEED: f32 = -140.0;
const SIDE_BOUNCE_SPEED: f32 = 240.0;
const SIDE_BOUNCE_FORCE_MOVE_X_TIME: f32 = 0.3;
const CLIMB_HOP_Y: f32 = -120.0;
const CLIMB_HOP_X: f32 = 100.0;
const CLIMB_HOP_FORCE_TIME: f32 = 0.2;
const CLIMB_HOP_NO_WIND_TIME: f32 = 0.3;
const CLIMB_UP_SPEED: f32 = -45.0;
const CLIMB_DOWN_SPEED: f32 = 80.0;
const CLIMB_SLIP_SPEED: f32 = 30.0;
const CLIMB_ACCEL: f32 = 900.0;
// `Player.cs:3093-3095`: `WallBoosterSpeed`/`WallBoosterLiftSpeed`/`WallBoosterAccel`,
// the conveyor ramp `ClimbUpdate` drives while a `WallBooster` faces the player.
const WALL_BOOSTER_SPEED: f32 = -160.0;
const WALL_BOOSTER_LIFT_SPEED: f32 = -80.0;
const WALL_BOOSTER_ACCEL: f32 = 600.0;
const CLIMB_CHECK_DIST: f32 = 2.0;
const CLIMB_UP_CHECK_DIST: i32 = 2;
const CLIMB_TIRED_THRESHOLD: f32 = 20.0;
const CLIMB_JUMP_COST: f32 = 27.5;
const CLIMB_UP_COST: f32 = 45.454_544;
const CLIMB_STILL_COST: f32 = 10.0;
const SWIM_Y_SPEED_MULT: f32 = 0.5;
const SWIM_MAX_RISE: f32 = -60.0;
const SWIM_MAX: f32 = 80.0;
const SWIM_UNDERWATER_MAX: f32 = 60.0;
const SWIM_ACCEL: f32 = 600.0;
const SWIM_REDUCE: f32 = 400.0;
const SWIM_DASH_SPEED_MULT: f32 = 0.75;
const WIND_ACCEL: f32 = 1000.0;
const WIND_MOVE_MULT: f32 = 0.1;
const WIND_WALL_DISTANCE: f32 = 3.0;
const STAR_FLY_TRANSFORM_DECEL: f32 = 1000.0;
const STAR_FLY_TIME: f32 = 2.0;
const STAR_FLY_START_SPEED: f32 = 250.0;
const STAR_FLY_TARGET_SPEED: f32 = 140.0;
const STAR_FLY_MAX_SPEED: f32 = 190.0;
const STAR_FLY_SLOW_SPEED: f32 = STAR_FLY_TARGET_SPEED * 0.65;
const STAR_FLY_ACCEL: f32 = 1000.0;
const STAR_FLY_ROTATE_SPEED: f32 = 320.0 * std::f32::consts::PI / 180.0;
const STAR_FLY_END_NO_BOUNCE_TIME: f32 = 0.2;
const STAR_FLY_WALL_BOUNCE: f32 = -0.5;
const STAR_FLY_MAX_EXIT_X: f32 = 140.0;
const STAR_FLY_EXIT_UP: f32 = -100.0;
const STAR_FLY_TRANSFORM_FRAMES: u8 = 27;
const FEATHER_RESPAWN_TIME: f32 = 3.0;
const REFILL_RESPAWN_TIME: f32 = 2.5;
const REFILL_FREEZE_TIME: f32 = 0.05;
const FALLING_BLOCK_MAX_SPEED: f32 = 160.0;
const FALLING_BLOCK_ACCEL: f32 = 500.0;
const FALLING_BLOCK_SHAKE_TIME: f32 = 0.2;
const FALLING_BLOCK_WAIT_TIME: f32 = 0.4;
const FALLING_BLOCK_IMPACT_TIME: f32 = 0.2;
const FALLING_BLOCK_PLATFORM_TICK: f32 = 0.1;
const LAUNCH_CANCEL_THRESHOLD: f32 = 220.0;
const TRANSITION_TIME: f32 = 0.65;
const TRANSITION_MOVE_SPEED: f32 = 60.0;
/// `Level.TransitionRoutine`'s `dirPad = direction * 4f` for a side or upward transition
/// (`Level.cs:1516`) and `direction * 12f` for a downward one (`:1517-1520`).
const TRANSITION_ENTRY_PAD: f32 = 4.0;
const TRANSITION_ENTRY_DOWN_PAD: f32 = 12.0;
/// Safety cap for [`transition_entry_target`]'s copy of `Level.TransitionRoutine`'s unbounded
/// `for (; !IsInBounds(playerTo, dirPad); playerTo += direction)`. Every real search terminates in a
/// few hundred whole-pixel steps (a room is at most a few hundred pixels across).
const TRANSITION_TARGET_MAX_STEPS: usize = 4096;

// Player.cs story-intro callbacks. Every constant below is copied from the
// callback that owns it; the `file:line` citations live on the phase bodies.
//
// `IntroWalkCoroutine` (Player.cs:5969-5993).
const INTRO_WALK_WAIT: f32 = 0.3;
const INTRO_WALK_SPEED: f32 = 64.0;
const INTRO_WALK_ARRIVE: f32 = 2.0;
const INTRO_WALK_REST: f32 = 0.2;
// `IntroJumpCoroutine` (Player.cs:5995-6068).
const INTRO_JUMP_SETTLE: f32 = 0.5;
const INTRO_JUMP_RISE_SPEED: f32 = -120.0;
const INTRO_JUMP_RISE_GAP: f32 = 8.0;
const INTRO_JUMP_BOTTOM_GAP: f32 = 16.0;
const INTRO_JUMP_SUMMIT_BOTTOM_GAP: f32 = 24.0;
const INTRO_JUMP_LAUNCH_SPEED: f32 = -100.0;
const INTRO_JUMP_GRAVITY: f32 = 800.0;
const INTRO_JUMP_REST: f32 = 0.1;
const INTRO_JUMP_SUMMIT_REST: f32 = 0.2;
const INTRO_JUMP_SUMMIT_RECOVER: f32 = 0.1;
/// `IntroJumpCoroutine`'s post-landing rest for a Summit hand-off:
/// `if (wasSummitJump) { ...; yield return 0.35f; }` before `StateMachine.State = 0`
/// (`Player.cs:6055-6067`). The non-Summit path has no such rest.
const INTRO_JUMP_SUMMIT_LAND_REST: f32 = 0.35;
// `IntroWakeUpCoroutine` (Player.cs:6112-6119) plus the `wakeUp` animation it
// awaits: `Content/Graphics/Sprites.xml:72`
// `<Anim id="wakeUp" path="wakeUp/" delay=".1" frames="0-4,5*10,6-14"/>`,
// i.e. 24 frames at 0.1 s each. `Monocle.Calc.ReadCSVIntWithTricks`
// (Calc.cs:1221) expands `5*10` into ten copies of frame 5.
const INTRO_WAKE_ASLEEP: f32 = 0.5;
const INTRO_WAKE_REST: f32 = 0.2;
const INTRO_WAKE_ANIM_DELAY: f32 = 0.1;
const INTRO_WAKE_ANIM_FRAMES: u8 = 24;
// `IntroThinkForABitCoroutine` (Player.cs:6156-6174).
const INTRO_THINK_CAMERA_WAIT: f32 = 0.1;
const INTRO_THINK_WALK_SPEED: f32 = 32.0;
const INTRO_THINK_WALK_DISTANCE: f32 = 8.0;
const INTRO_THINK_IDLE: f32 = 0.3;
const INTRO_THINK_LEFT: f32 = 0.8;
const INTRO_THINK_RIGHT: f32 = 0.1;
// `IntroRespawnBegin` (Player.cs:6131) creates a 0.6 second Oneshot tween.
const INTRO_RESPAWN_TIME: f32 = 0.6;

// Intro coroutine program counters.
const INTRO_PHASE_WALK_WAIT: u8 = 1;
const INTRO_PHASE_WALK_MOVE: u8 = 2;
const INTRO_PHASE_WALK_REST: u8 = 3;
const INTRO_PHASE_JUMP_SETTLE: u8 = 4;
const INTRO_PHASE_JUMP_RISE: u8 = 5;
const INTRO_PHASE_JUMP_DECEL: u8 = 6;
const INTRO_PHASE_JUMP_REST: u8 = 7;
const INTRO_PHASE_JUMP_FALL: u8 = 8;
const INTRO_PHASE_JUMP_SUMMIT_REST: u8 = 9;
const INTRO_PHASE_JUMP_SUMMIT_RECOVER: u8 = 10;
const INTRO_PHASE_JUMP_SUMMIT_LAND_REST: u8 = 21;
const INTRO_PHASE_WAKE_ASLEEP: u8 = 11;
const INTRO_PHASE_WAKE_SPRITE: u8 = 12;
const INTRO_PHASE_WAKE_POP: u8 = 13;
const INTRO_PHASE_WAKE_REST: u8 = 14;
const INTRO_PHASE_THINK_CAMERA: u8 = 15;
const INTRO_PHASE_THINK_WALK: u8 = 16;
const INTRO_PHASE_THINK_IDLE: u8 = 17;
const INTRO_PHASE_THINK_LEFT: u8 = 18;
const INTRO_PHASE_THINK_RIGHT: u8 = 19;
const INTRO_PHASE_RESPAWN: u8 = 20;
/// `wasSummitJump = StateMachine.PreviousState == 10` (Player.cs:5998) is not
/// exported by the trace; the Summit finale hand-off is recorded in this bit
/// of `PlayerSnapshot::intro_phase`.
const INTRO_PHASE_SUMMIT_FLAG: u8 = 0x80;
/// `intro_phase` values are all below this mask.
const INTRO_PHASE_MASK: u8 = 0x7F;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fidelity {
    SourceInformedSubset,
}

pub const fn fidelity() -> Fidelity {
    Fidelity::SourceInformedSubset
}

/// Story-specific states deliberately excluded from the technique-training
/// product scope. They remain parseable so real snapshots fail explicitly.
pub const INTENTIONALLY_UNSUPPORTED_STATES: &[PlayerState] = &[PlayerState::Attract];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationResult {
    pub fidelity: Fidelity,
    pub states: Vec<PlayerSnapshot>,
}

#[derive(Debug, Error, PartialEq)]
pub enum SimulationError {
    #[error("requested {frames} frames but only {inputs} inputs were supplied")]
    InsufficientInputs { frames: usize, inputs: usize },
    #[error("player state {0:?} is parsed but not implemented by the source-informed subset")]
    UnsupportedState(PlayerState),
    #[error("snapshot contains a non-finite float")]
    NonFinite,
}

/// A reusable simulation context. Constructing a context performs the one-time
/// map clone and entity-state initialization; subsequent calls only advance the
/// already initialized runtime map. This is the hot path used by native and
/// WASM callers that evaluate many action sequences against one map.
#[derive(Clone)]
pub struct Simulator {
    snapshot: PlayerSnapshot,
    runtime_map: Map,
    static_mover_attachments: Vec<Option<StaticMoverAttachment>>,
    climb_hop_solid: Option<ClimbHopSolid>,
    /// `oshiro_clutter_cleared_<color>` (`Session.Flags`), see
    /// [`Simulator::clutter_cleared`].
    clutter_cleared: [bool; CLUTTER_COLORS],
    /// Per-CrumblePlatform coroutine position, in map entity order.
    crumble_blocks: Vec<CrumbleBlockState>,
    /// Per-room entity coroutines (`CrumblePlatform`, `SwitchGate`, `TouchSwitch`).
    room: RoomCoroutineState,
    /// `Session.DoNotLoad`: `"<Level>:<ID>"` keys of entities the game never constructed.
    do_not_load: Vec<i32>,
}

/// `Player.climbHopSolid` (`Player.cs:553`) and `climbHopSolidPosition`
/// (`Player.cs:555`). `Player.ClimbHop` (`Player.cs:4124`) stores the
/// `CollideFirst<Solid>` it grabbed; while the climb-hop force-move window is
/// open, `Player.Update` replays that Solid's whole-pixel movement onto the
/// player (`Player.cs:1646-1652`). Only the entity index and the last observed
/// position are needed, and both survive `Simulator::fork` so branch searches
/// stay identical to a continuous run.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ClimbHopSolid {
    entity: usize,
    /// `Solid.Position`. The runtime map stores the collider rectangle, whose
    /// offset from `Position` is constant for every Solid kind, so deltas
    /// between frames are exact.
    position: Vec2,
}

impl Simulator {
    pub fn new(mut snapshot: PlayerSnapshot, map: &Map) -> Result<Self, SimulationError> {
        validate_snapshot(&snapshot)?;
        if !snapshot.player_on_ground_initialized {
            snapshot.player_on_ground = snapshot.on_ground;
            snapshot.player_on_ground_initialized = true;
        }
        restore_session_dream_dash(&mut snapshot);
        let mut runtime_map = map.clone();
        add_room_edge_tile_bleed(&mut runtime_map);
        let clutter_cleared = [false; CLUTTER_COLORS];
        add_clutter_solids(&mut runtime_map, &clutter_cleared);
        let static_mover_attachments = initialize_static_mover_attachments(&runtime_map);
        initialize_zip_movers(&mut snapshot, &mut runtime_map);
        initialize_bounce_blocks(&mut snapshot, &mut runtime_map);
        initialize_move_blocks(&mut snapshot, &mut runtime_map);
        initialize_theo_crystals(&mut snapshot, &mut runtime_map);
        initialize_heart_gems(&mut snapshot, &mut runtime_map);
        initialize_rising_lavas(&mut snapshot, &mut runtime_map);
        initialize_sandwich_lavas(&mut snapshot, &mut runtime_map);
        initialize_gliders(&mut snapshot, &mut runtime_map);
        initialize_clouds(&mut snapshot, &mut runtime_map);
        initialize_camera(&mut snapshot, &runtime_map);
        initialize_seekers(&mut snapshot, &mut runtime_map);
        initialize_puffers(&mut snapshot, &mut runtime_map);
        initialize_temple_gates(&mut snapshot, &mut runtime_map);
        initialize_core_mode_toggles(&mut snapshot, &runtime_map);
        initialize_cassette_blocks(&mut snapshot, &mut runtime_map);
        initialize_spinners(&mut snapshot, &mut runtime_map);
        initialize_bumpers(&mut snapshot, &mut runtime_map);
        initialize_refills(&mut snapshot, &mut runtime_map);
        initialize_falling_blocks(&mut snapshot, &mut runtime_map);
        initialize_exit_blocks(&mut snapshot, &mut runtime_map);
        initialize_invisible_barriers(&mut snapshot, &mut runtime_map);
        initialize_killboxes(&mut snapshot, &mut runtime_map);
        initialize_crush_and_dash_blocks(&mut snapshot, &mut runtime_map);
        let crumble_blocks = initialize_crumble_blocks(&runtime_map);
        let room = initialize_room_coroutines(&mut runtime_map);
        initialize_lookouts(&mut snapshot, &runtime_map);
        position_moving_solids(&mut runtime_map, snapshot.moving_solid_time);
        sync_all_platform_static_movers(&snapshot, &mut runtime_map, &static_mover_attachments);
        Ok(Self {
            snapshot,
            runtime_map,
            static_mover_attachments,
            climb_hop_solid: None,
            clutter_cleared,
            crumble_blocks,
            room,
            do_not_load: Vec::new(),
        })
    }

    /// The three `oshiro_clutter_cleared_<color>` session flags
    /// (`ClutterSwitch.cs:138`, read back by `ClutterBlockGenerator.Init`,
    /// `ClutterBlockGenerator.cs:78-81`), indexed by `ClutterBlock.Colors`
    /// (Red = 0, Green = 1, Yellow = 2; `ClutterBlock.cs:10-15`).
    ///
    /// These are chapter-session state, not player state: a trace row cannot
    /// carry them, so a caller that replays several room segments of one
    /// chapter must hand the previous segment's value to the next one exactly
    /// like the live session keeps it.
    pub fn clutter_cleared(&self) -> [bool; CLUTTER_COLORS] {
        self.clutter_cleared
    }

    /// Hand the anchor row's `Session.DoNotLoad` keys to the room build; the v7 trace carries them.
    pub fn set_do_not_load(&mut self, keys: Vec<i32>) {
        // The game never constructed those entities (`Level.cs:472`, `:1188`), so park them - do NOT
        // filter the vector: every room-coroutine index and `entity_ids` entry is positional.
        let count = self.runtime_map.entity_ids.len().min(self.runtime_map.entities.len());
        for index in 0..count {
            let id = self.runtime_map.entity_ids[index];
            if id >= 0 && keys.contains(&id) {
                park_entity(&mut self.runtime_map.entities[index]);
            }
        }
        self.do_not_load = keys;
    }

    /// The room's `switches_<room>` session flag, as `Switch.SetLevelFlag`/`CheckLevelFlag` read and
    /// write it (`Switch.cs`). Per *room*, so the gate threads it keyed by room.
    pub fn switches_on(&self) -> bool {
        self.room.switches_on
    }

    /// Restore the room's `switches_<room>` flag before the first replayed frame: `SwitchGate.Awake`
    /// (`SwitchGate.cs:68-82`) moves the gate straight onto its target when it is set, which is what
    /// a revisit of the same room needs.
    pub fn set_switches_on(&mut self, on: bool) {
        if on && !self.room.switches_on {
            for index in 0..self.room.switch_gates.len() {
                let gate = self.room.switch_gates[index];
                if let Some(entity) = self.runtime_map.entities.get_mut(gate.entity_index) {
                    entity.bounds.x = gate.target.x;
                    entity.bounds.y = gate.target.y;
                }
                self.room.switch_gates[index].phase = 6;
            }
        }
        self.room.switches_on = on;
    }

    /// Restore the `oshiro_clutter_cleared_<color>` flags before the first
    /// replayed frame, deactivating the `ClutterBlockBase` solids of every
    /// colour that was already cleared (`ClutterBlockBase.Deactivate`,
    /// `ClutterBlockBase.cs:46-56`).
    pub fn set_clutter_cleared(&mut self, cleared: [bool; CLUTTER_COLORS]) {
        for color in 0..CLUTTER_COLORS {
            if cleared[color] && !self.clutter_cleared[color] {
                self.clear_clutter_color(color);
            }
        }
        self.clutter_cleared = cleared;
    }

    /// Restore the `dashSwitch_<room>:<id>` session flags (`DashSwitch.cs:255-258`) the anchor row
    /// carries, for the *persistent* dash switches of the room now being replayed.
    ///
    /// `ids` are the `EntityID.ID`s (`EntityID.cs:18-30`) whose flag is already set. The room half
    /// of the key stays with the caller because a [`Map`] carries no room name - and the flag is
    /// per session, not per entity, so the trace's `flags` array is its only witness.
    ///
    /// `DashSwitch.Awake` (`DashSwitch.cs:124-149`) short-circuits on the flag: it plays the pushed
    /// sprite, puts the switch at `pressedTarget - pressDirection * 2f` and sets `Collidable =
    /// false`. That is the state `OnDashed` itself leaves behind (`:203-205`), so the whole
    /// observable effect is "this Solid is not there", which is what `park_entity` models for
    /// `ExitBlock`, `InvisibleBarrier`, `CassetteBlock` and `FallingBlock` - and what the
    /// in-replay press already does for the same entity kind.
    ///
    /// The same `Awake` then opens gates (`:135-148`): `allGates` runs `StartOpen` over every
    /// `NearestSwitch` gate of the room, otherwise `GetGate()` claims and opens the nearest
    /// unclaimed one. Both read the switch's *already pressed* position, which is why the fan-out
    /// below passes `bounds + pressDirection * 6f` and not the parked rectangle.
    pub fn set_pressed_dash_switches(&mut self, ids: &[i32]) {
        if ids.is_empty() {
            return;
        }
        let pressed: Vec<usize> = self
            .runtime_map
            .entities
            .iter()
            .enumerate()
            .filter(|(index, entity)| {
                // `persistent` (`DashSwitch.cs:106`) is the only switch flag a press can restore,
                // and it is carried in `single_use` like `CoreModeToggle`'s.
                entity.kind == EntityKind::DashSwitch
                    && entity.single_use
                    && self
                        .runtime_map
                        .entity_ids
                        .get(*index)
                        .is_some_and(|id| ids.contains(id))
            })
            .map(|(index, _)| index)
            .collect();
        for index in pressed {
            let entity = &self.runtime_map.entities[index];
            let pressed_position = Vec2::new(
                entity.bounds.x + entity.direction.x * 6.0,
                entity.bounds.y + entity.direction.y * 6.0,
            );
            park_entity(&mut self.runtime_map.entities[index]);
            dash_switch_open_gates(
                &mut self.snapshot,
                &mut self.runtime_map,
                index,
                pressed_position,
                false,
            );
        }
    }

    /// `ClutterBlockBase.Deactivate` (`ClutterBlockBase.cs:46-51`) plus
    /// `ClutterSwitch.BePressed`'s ten-pixel drop (`ClutterSwitch.cs:86-87`).
    fn clear_clutter_color(&mut self, color: usize) {
        remove_clutter_solids(&mut self.runtime_map, color);
        for (rect, switch_color) in clutter_switch_rects(&self.runtime_map) {
            if switch_color != Some(color) {
                continue;
            }
            if let Some(index) = self
                .runtime_map
                .solids
                .iter()
                .rposition(|solid| *solid == rect)
            {
                self.runtime_map.solids[index] =
                    Rect::new(rect.x, rect.y + PRESSED_SWITCH_OFFSET, rect.width, rect.height);
            }
        }
    }

    /// `Celeste.Session.Cassette` (`Cassette.CollectRoutine` writes it, `Cassette.cs:176`): this
    /// A-side chapter has already taken its cassette tape.
    ///
    /// Chapter state, not room state. Once it is true the game never constructs a
    /// `CassetteBlockManager` for the rest of the chapter: `Level.ShouldCreateCassetteManager` is
    /// `!Session.Cassette` for `AreaMode.Normal` (`Level.cs:278-288`), which gates both the
    /// construction at `Level.cs:657` and `OnLevelStart` at `Level.cs:1355-1358`. So
    /// `CassetteBlock.SetActivatedSilently` (`CassetteBlock.cs:392-394`, whose only caller is
    /// `CassetteBlockManager.SilentUpdateBlocks`, `CassetteBlockManager.cs:197-206`) is never
    /// reached and every block keeps the `Collidable = false` its constructor set
    /// (`CassetteBlock.cs:70-76`).
    ///
    /// A trace without the key leaves this false, which reproduces the older behaviour exactly:
    /// the manager is created and the blocks at the current index are collidable.
    pub fn set_cassette_taken(&mut self, taken: bool) {
        self.snapshot.cassette_manager.tape_taken = taken;
        if !taken {
            return;
        }
        // Run the same initializer the room loads use, so a flag set after `Simulator::new`
        // leaves the blocks exactly where a room loaded with the tape already taken would be:
        // parked, non-collidable, and with no manager for `advance_cassette_manager` to advance.
        initialize_cassette_blocks(&mut self.snapshot, &mut self.runtime_map);
        // `SetActivatedSilently` is also the only caller of `EnableStaticMovers`
        // (`CassetteBlock.cs:392-398`), so a block that is never activated never enables the
        // entities (spikes and the like) attached to it.
        sync_all_platform_static_movers(
            &self.snapshot,
            &mut self.runtime_map,
            &self.static_mover_attachments,
        );
    }

    pub fn snapshot(&self) -> &PlayerSnapshot {
        &self.snapshot
    }

    /// Make the next [`Simulator::step`] reproduce an engine frame whose
    /// `Scene.Update` did not run.
    ///
    /// `Celeste.Freeze(t)` writes `Engine.FreezeTimer` and `Monocle.Engine.Update`
    /// then skips `Scene.Update` for as long as it is positive; `Level.Update`
    /// likewise runs only `Player`-external transition work while
    /// `Level.Transitioning`. `Engine.FreezeTimer` is an `Engine` field rather
    /// than a `Player` field, so a frame-by-frame trace cannot export it. A row
    /// whose `Player.StrawberryCollectResetTimer` did not move proves the frame
    /// was skipped, and a caller that replays such a row must not run a
    /// `Player.Update` the game never ran. One call covers exactly one engine
    /// frame; repeat it for each skipped row.
    pub fn skip_engine_frame(&mut self) {
        self.snapshot.freeze_timer = self.snapshot.freeze_timer.max(DT);
    }

    /// A replayed row whose `Level.Transitioning` is true (`Level.cs:221`).
    ///
    /// While a room transition runs, `Level.Update` takes the `Transitioning` branch
    /// (`Level.cs:1869-1896`): `Player.Update` does not run at all, and the player is moved by the
    /// transition coroutine instead. `Level.TransitionRoutine`'s movement loop is
    /// `while (!player.TransitionTo(playerTo, direction) || cameraAt < 1f)` (`Level.cs:1569`), and
    /// `Player.TransitionTo` (`Player.cs:1560-1580`) moves the exact position towards `playerTo` by
    /// `60f * Engine.DeltaTime` per axis - one pixel per frame at 60 Hz, with `Speed` untouched and
    /// rounded only on the frame `Position == target`. So a transition row is *not* an engine frame
    /// the game skipped: it is a frame the simulator has to reproduce with its own transition model.
    ///
    /// The trace exports only the `transitioning` boolean, never `playerTo`, the coroutine's
    /// duration or its speed, so the target is derived from the room being entered the way
    /// `Level.TransitionRoutine` converges on it (`Level.cs:1521-1528`): `LoadLevel` has already
    /// installed the destination room (`Level.cs:1509`), the entry direction is the bound the
    /// player's collider still crosses, and `playerTo` is that bound adjusted by `dirPad` - see
    /// [`transition_entry_target`], which also records why a replayed row can derive it without
    /// knowing where the coroutine started.
    ///
    /// `raw_delta_time` is the row's own raw frame delta (the value the caller passes to
    /// [`Simulator::step`]): `transition_timer` stands in for the coroutine's camera clock
    /// (`NextTransitionDuration`, 0.65 s, `Level.cs:57`). Returns whether this call armed a
    /// transition; a simulator that is already transitioning is left untouched, so this may be
    /// called on every `transitioning` row.
    pub fn resume_transition(&mut self, raw_delta_time: f32) -> bool {
        if self.snapshot.transition_timer > 0.0 {
            return false;
        }
        let bounds = self
            .snapshot
            .current_room_bounds
            .unwrap_or(self.runtime_map.bounds);
        let direction = transition_entry_direction(&self.snapshot, bounds);
        let target = transition_entry_target(self.snapshot.pos, direction, bounds);
        self.snapshot.transition_room_bounds = Some(bounds);
        self.snapshot.transition_direction = direction;
        self.snapshot.transition_target = target;
        self.snapshot.transition_timer =
            TRANSITION_TIME + raw_delta_time * self.snapshot.time_rate;
        true
    }

    /// A replayed row whose `Level.Transitioning` is false again: the game's coroutine has left its
    /// movement loop and run `player.OnTransition()` (`Level.cs:1624-1626`).
    ///
    /// The simulator's own clock can outlast the trace's: a transition the harness armed on a row in
    /// the middle of the coroutine starts later than the game's, so its timer is still positive when
    /// the game is already updating the player again. Ending it here makes this the transition's
    /// last frame - the next [`Simulator::step`] still runs `update_transition`, which completes the
    /// transfer (`Player.OnTransition`: `RefillDash`/`RefillStamina`) and returns, and the frame
    /// after that takes the ordinary `Player.Update` path, exactly as the game did.
    pub fn finish_transition(&mut self) {
        if self.snapshot.transition_timer > 0.0 {
            self.snapshot.transition_timer = f32::MIN_POSITIVE;
        }
    }

    /// Current entity rectangles after all runtime movement, visibility, and
    /// StaticMover attachment updates have been applied. The order is stable
    /// and matches the decoded room entity order, which lets hot-path clients
    /// update spatial observations without serializing a full snapshot.
    pub(crate) fn runtime_entities(&self) -> &[crate::Entity] {
        &self.runtime_map.entities
    }

    /// Branch this already-initialized simulation context.
    ///
    /// Searchers use this at divergent input prefixes so entity runtime state
    /// (not only the portable player snapshot) remains identical to a
    /// continuously simulated path.
    pub fn fork(&self) -> Self {
        self.clone()
    }

    pub fn step(&mut self, input: InputState) -> Result<&PlayerSnapshot, SimulationError> {
        step(
            &mut self.snapshot,
            input.normalized(),
            &mut self.runtime_map,
            &mut self.static_mover_attachments,
            &mut self.climb_hop_solid,
            &mut self.crumble_blocks,
            &mut self.room,

        )?;
        Ok(&self.snapshot)
    }

    /// `ClutterSwitch.OnDashed` (`ClutterSwitch.cs:131-156`): a downward dash
    /// that lands on the switch clears its colour for the whole session.
    ///
    /// `resting` is the collider rectangle of the pressing player, one pixel
    /// down, i.e. the shape that touches the switch's own Solid
    /// (`ClutterSwitch.cs:49-50`). A replay that diverges before the press
    /// cannot observe the dash itself, so the gate hands the ground truth's
    /// rectangle in; everything else - which switch, which colour, which
    /// `ClutterBlockBase` rectangles disappear - stays here. Returns the
    /// cleared colour, if the press landed on a switch whose colour the map
    /// decoder retained.
    pub fn press_clutter_switch(&mut self, resting: Rect) -> Option<usize> {
        let colors: Vec<usize> = clutter_switch_rects(&self.runtime_map)
            .into_iter()
            .filter_map(|(rect, color)| {
                let color = color?;
                (color < CLUTTER_COLORS
                    && !self.clutter_cleared[color]
                    && rect.intersects(resting))
                .then_some(color)
            })
            .collect();
        for color in &colors {
            self.clutter_cleared[*color] = true;
            self.clear_clutter_color(*color);
        }
        colors.first().copied()
    }

    pub fn run(&mut self, inputs: &[InputState], frames: u32) -> Result<(), SimulationError> {
        let frames = frames as usize;
        if inputs.len() < frames {
            return Err(SimulationError::InsufficientInputs {
                frames,
                inputs: inputs.len(),
            });
        }
        for input in &inputs[..frames] {
            self.step(*input)?;
        }
        Ok(())
    }

    pub fn into_snapshot(self) -> PlayerSnapshot {
        self.snapshot
    }
}

/// Recover the session-level dream-dash inventory from the player's own timer.
///
/// `Player.dreamDashCanEndTimer` is a `private float` (`Player.cs:551`), so it starts at
/// the C# default `0f` for every fresh `Player`, and the source writes it in exactly two
/// places, both inside state 9:
///
/// * `DreamDashBegin` sets it to `DreamDashMinTime` = `0.1f` (`Player.cs:5144`, constant at
///   `Player.cs:251`);
/// * `DreamDashUpdate` subtracts `Engine.DeltaTime` from it, and only while it is
///   `> 0f` (`Player.cs:5189-5192`), so once it runs out it stays at a tiny non-zero
///   residue instead of returning to zero.
///
/// State 9 is only ever entered by the three `DreamDashCheck` call sites (`Player.cs:3205`,
/// `3313`, `3396`), and each of them is gated on `Inventory.DreamDash` (`Player.cs:3420`).
/// That flag is session state with no equally-named field on the player, so a restored
/// snapshot cannot carry it: the only gameplay writer is the Old Site mirror cutscene
/// (`CS02_Mirror.cs:104`), which turns it on, on top of a chapter inventory that starts with
/// `dreamDash: false` (`PlayerInventory.cs:12`, `AreaData.cs:211`; the console command at
/// `Commands.cs:691` can also toggle it). A non-zero timer — or a snapshot already in state
/// 9 — therefore proves the inventory is on, while a timer still at exactly `0f` is the
/// fresh-player state in which it may be off (Old Site A-side before checkpoint "3",
/// `AreaData.cs:208-211`).
fn restore_session_dream_dash(p: &mut PlayerSnapshot) {
    if p.dream_dash_can_end_timer != 0.0 || p.state == PlayerState::DreamDash {
        p.can_dream_dash = true;
    }
}

pub fn simulate(
    snapshot: PlayerSnapshot,
    inputs: &[InputState],
    map: &Map,
    frames: u32,
) -> Result<PlayerSnapshot, SimulationError> {
    let mut simulator = Simulator::new(snapshot, map)?;
    simulator.run(inputs, frames)?;
    Ok(simulator.into_snapshot())
}

pub fn simulate_trace(
    snapshot: PlayerSnapshot,
    inputs: &[InputState],
    map: &Map,
    frames: u32,
) -> Result<SimulationResult, SimulationError> {
    let frames = frames as usize;
    if inputs.len() < frames {
        return Err(SimulationError::InsufficientInputs {
            frames,
            inputs: inputs.len(),
        });
    }
    let mut simulator = Simulator::new(snapshot, map)?;
    let mut states = Vec::with_capacity(frames + 1);
    states.push(simulator.snapshot().clone());
    for input in &inputs[..frames] {
        simulator.step(*input)?;
        states.push(simulator.snapshot().clone());
    }
    Ok(SimulationResult {
        fidelity: fidelity(),
        states,
    })
}

fn validate_snapshot(s: &PlayerSnapshot) -> Result<(), SimulationError> {
    let values = [
        s.pos.x,
        s.pos.y,
        s.speed.x,
        s.speed.y,
        s.stamina,
        s.dash_attack_timer,
        s.dash_cooldown_timer,
        s.freeze_timer,
        s.time_rate,
        s.current_room_bounds.map_or(0.0, |bounds| bounds.x),
        s.current_room_bounds.map_or(0.0, |bounds| bounds.y),
        s.current_room_bounds.map_or(0.0, |bounds| bounds.width),
        s.current_room_bounds.map_or(0.0, |bounds| bounds.height),
        s.transition_room_bounds.map_or(0.0, |bounds| bounds.x),
        s.transition_room_bounds.map_or(0.0, |bounds| bounds.y),
        s.transition_room_bounds.map_or(0.0, |bounds| bounds.width),
        s.transition_room_bounds.map_or(0.0, |bounds| bounds.height),
        s.transition_direction.x,
        s.transition_direction.y,
        s.transition_target.x,
        s.transition_target.y,
        s.transition_timer,
        s.camera.x,
        s.camera.y,
        s.state_timer,
        s.boost_target.x,
        s.boost_target.y,
        s.wind.x,
        s.wind.y,
        s.wind_target.x,
        s.wind_target.y,
        s.no_wind_timer,
        s.launch_approach_x.unwrap_or(0.0),
        s.summit_launch_target_x,
        s.summit_launch_particle_timer,
        s.star_fly_timer,
        s.star_fly_speed_lerp,
        s.star_fly_last_dir.x,
        s.star_fly_last_dir.y,
        s.last_feather_target.x,
        s.last_feather_target.y,
        s.feather_reuse_timer,
        s.bumper_reuse_timer,
        s.strawberry_follow_delay_timer,
        s.strawberry_collect_timer,
        s.strawberry_collect_reset_timer,
        s.bounce_reuse_timer,
        s.pending_bounce_from_y.unwrap_or(0.0),
        s.explode_launch_boost_timer,
        s.explode_launch_boost_speed,
        s.badeline_boost_start.x,
        s.badeline_boost_start.y,
        s.badeline_boost_target.x,
        s.badeline_boost_target.y,
        s.last_badeline_boost_target.x,
        s.last_badeline_boost_target.y,
        s.badeline_boost_entity_origin.x,
        s.badeline_boost_entity_origin.y,
        s.badeline_boost_current_position.x,
        s.badeline_boost_current_position.y,
        s.badeline_boost_relocation_from.x,
        s.badeline_boost_relocation_from.y,
        s.badeline_boost_relocation_to.x,
        s.badeline_boost_relocation_to.y,
        s.badeline_boost_relocation_elapsed,
        s.badeline_boost_relocation_duration,
        s.reflection_fall_wait_timer,
        s.last_aim.x,
        s.last_aim.y,
        s.wall_speed_retention_timer,
        s.wall_speed_retained,
        s.wall_boost_timer,
        s.hop_wait_x_speed,
        s.current_lift_speed.x,
        s.current_lift_speed.y,
        s.last_lift_speed.x,
        s.last_lift_speed.y,
        s.lift_speed_timer,
        s.moving_solid_time,
        s.scene_time_active,
        s.cassette_manager.beat_timer,
        s.cassette_manager.tempo_mult,
        s.dash_buffer_timer,
        s.crouch_dash_buffer_timer,
        s.movement_remainder.x,
        s.movement_remainder.y,
        s.min_hold_timer,
        s.pickup_old_speed.x,
        s.pickup_old_speed.y,
        s.pickup_old_var_jump_timer,
        s.pickup_timer,
    ];
    let zip_movers_are_finite = s.zip_movers.iter().all(|zip| {
        [
            zip.wait_timer,
            zip.at,
            zip.position.x,
            zip.position.y,
            zip.remainder.x,
            zip.remainder.y,
            zip.lift_speed.x,
            zip.lift_speed.y,
            zip.start.x,
            zip.start.y,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    let bounce_blocks_are_finite = s.bounce_blocks.iter().all(|block| {
        [
            block.move_speed,
            block.bounce_dir.x,
            block.bounce_dir.y,
            block.bounce_lift.x,
            block.bounce_lift.y,
            block.bounce_end_timer,
            block.respawn_timer,
            block.position.x,
            block.position.y,
            block.remainder.x,
            block.remainder.y,
            block.lift_speed.x,
            block.lift_speed.y,
            block.start.x,
            block.start.y,
            block.reform_timer,
            block.attached_spike_position.x,
            block.attached_spike_position.y,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    let move_blocks_are_finite = s.move_blocks.iter().all(|block| {
        [
            block.wait_timer,
            block.speed,
            block.angle,
            block.crash_timer,
            block.crash_reset_timer,
            block.no_steer_timer,
            block.position.x,
            block.position.y,
            block.remainder.x,
            block.remainder.y,
            block.lift_speed.x,
            block.lift_speed.y,
            block.start.x,
            block.start.y,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    let theo_crystals_are_finite = s.theo_crystals.iter().all(|theo| {
        [
            theo.position.x,
            theo.position.y,
            theo.speed.x,
            theo.speed.y,
            theo.remainder.x,
            theo.remainder.y,
            theo.cannot_hold_timer,
            theo.gravity_timer,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    let gliders_are_finite = s.gliders.iter().all(|glider| {
        [
            glider.position.x,
            glider.position.y,
            glider.speed.x,
            glider.speed.y,
            glider.remainder.x,
            glider.remainder.y,
            glider.cannot_hold_timer,
            glider.gravity_timer,
            glider.no_gravity_timer,
            glider.high_friction_timer,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    let rising_lavas_are_finite = s.rising_lavas.iter().all(|lava| {
        [lava.position.x, lava.position.y, lava.delay]
            .iter()
            .all(|value| value.is_finite())
    });
    let sandwich_lavas_are_finite = s.sandwich_lavas.iter().all(|lava| {
        [
            lava.position.x,
            lava.position.y,
            lava.start_x,
            lava.delay,
            lava.leave_timer,
            lava.top_rect_y,
            lava.bottom_rect_y,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    let clouds_are_finite = s.clouds.iter().all(|cloud| {
        [
            cloud.speed,
            cloud.position.x,
            cloud.position.y,
            cloud.remainder_y,
            cloud.start.x,
            cloud.start.y,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    let seekers_are_finite = s.seekers.iter().all(|seeker| {
        [
            seeker.position.x,
            seeker.position.y,
            seeker.speed.x,
            seeker.speed.y,
            seeker.remainder.x,
            seeker.remainder.y,
            seeker.state_timer,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    let cassette_blocks_are_finite = s.cassette_blocks.iter().all(|block| {
        [
            block.position.x,
            block.position.y,
            block.start.x,
            block.start.y,
            block.width,
            block.height,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    let temple_gates_are_finite = s.temple_gates.iter().all(|gate| {
        [
            gate.position.x,
            gate.position.y,
            gate.current_height,
            gate.closed_height,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    let spinners_are_finite = s.spinners.iter().all(|spinner| {
        [spinner.position.x, spinner.position.y, spinner.offset]
            .iter()
            .all(|value| value.is_finite())
    });
    let refills_are_finite = s
        .refills
        .iter()
        .all(|refill| refill.respawn_timer.is_finite());
    let falling_blocks_are_finite = s.falling_blocks.iter().all(|block| {
        [
            block.position.x,
            block.position.y,
            block.start.x,
            block.start.y,
            block.remainder_y,
            block.fall_speed,
            block.fall_delay,
            block.shake_timer,
            block.wait_timer,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    let exit_blocks_are_finite = s
        .exit_blocks
        .iter()
        .all(|block| block.position.x.is_finite() && block.position.y.is_finite());
    let invisible_barriers_are_finite = s
        .invisible_barriers
        .iter()
        .all(|barrier| barrier.position.x.is_finite() && barrier.position.y.is_finite());
    let killboxes_are_finite = s
        .killboxes
        .iter()
        .all(|killbox| killbox.position.x.is_finite() && killbox.position.y.is_finite());
    let lookouts_are_finite = s.lookouts.iter().all(|lookout| {
        [
            lookout.timer,
            lookout.position.x,
            lookout.position.y,
            lookout.cam_start.x,
            lookout.cam_start.y,
            lookout.cam.x,
            lookout.cam.y,
            lookout.cam_speed.x,
            lookout.cam_speed.y,
            lookout.wipe_start.x,
            lookout.wipe_start.y,
            lookout.node_percent,
            lookout.hud_easer,
        ]
        .iter()
        .all(|value| value.is_finite())
    });
    if values.iter().all(|x| x.is_finite())
        && zip_movers_are_finite
        && bounce_blocks_are_finite
        && move_blocks_are_finite
        && theo_crystals_are_finite
        && rising_lavas_are_finite
        && sandwich_lavas_are_finite
        && gliders_are_finite
        && clouds_are_finite
        && seekers_are_finite
        && temple_gates_are_finite
        && cassette_blocks_are_finite
        && spinners_are_finite
        && lookouts_are_finite
        && refills_are_finite
        && falling_blocks_are_finite
        && exit_blocks_are_finite
        && invisible_barriers_are_finite
        && killboxes_are_finite
    {
        Ok(())
    } else {
        Err(SimulationError::NonFinite)
    }
}

fn initialize_theo_crystals(p: &mut PlayerSnapshot, map: &mut Map) {
    let theo_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::TheoCrystal).then_some(index))
        .collect();
    p.theo_crystals.truncate(theo_indices.len());
    for (theo_index, entity_index) in theo_indices.into_iter().enumerate() {
        let entity = &mut map.entities[entity_index];
        if theo_index == p.theo_crystals.len() {
            p.theo_crystals.push(crate::TheoCrystalSnapshot {
                position: Vec2::new(entity.bounds.x + 4.0, entity.bounds.y + 10.0),
                ..crate::TheoCrystalSnapshot::default()
            });
        }
        let state = &mut p.theo_crystals[theo_index];
        state.held = p.holding_theo == Some(theo_index as u16);
        if state.dead {
            entity.bounds.x = -1_000_000.0;
            entity.bounds.y = -1_000_000.0;
        } else {
            entity.bounds.x = state.position.x - 4.0;
            entity.bounds.y = state.position.y - 10.0;
        }
        entity.bounds.width = 8.0;
        entity.bounds.height = 10.0;
    }
    if p.holding_theo
        .is_some_and(|index| index as usize >= p.theo_crystals.len())
    {
        p.holding_theo = None;
    }
}

fn initialize_gliders(p: &mut PlayerSnapshot, map: &mut Map) {
    let glider_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::Glider).then_some(index))
        .collect();
    p.gliders.truncate(glider_indices.len());
    for (glider_index, entity_index) in glider_indices.into_iter().enumerate() {
        let entity = &mut map.entities[entity_index];
        if glider_index == p.gliders.len() {
            p.gliders.push(crate::GliderSnapshot {
                position: Vec2::new(entity.bounds.x + 4.0, entity.bounds.y + 10.0),
                ..crate::GliderSnapshot::default()
            });
        }
        let state = &mut p.gliders[glider_index];
        state.held = p.holding_glider == Some(glider_index as u16);
        if state.removed {
            entity.bounds.x = -1_000_000.0;
            entity.bounds.y = -1_000_000.0;
        } else {
            entity.bounds.x = state.position.x - 4.0;
            entity.bounds.y = state.position.y - 10.0;
        }
        entity.bounds.width = 8.0;
        entity.bounds.height = 10.0;
    }
    if p.holding_glider
        .is_some_and(|index| index as usize >= p.gliders.len())
    {
        p.holding_glider = None;
    }
}

fn initialize_clouds(p: &mut PlayerSnapshot, map: &mut Map) {
    let cloud_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::Cloud).then_some(index))
        .collect();
    p.clouds.truncate(cloud_indices.len());
    for (cloud_index, entity_index) in cloud_indices.into_iter().enumerate() {
        let entity = &mut map.entities[entity_index];
        if cloud_index == p.clouds.len() {
            let start = Vec2::new(entity.bounds.x, entity.bounds.y);
            p.clouds.push(crate::CloudSnapshot {
                position: start,
                start,
                ..crate::CloudSnapshot::default()
            });
        }
        let state = &p.clouds[cloud_index];
        entity.bounds.x = state.position.x;
        entity.bounds.y = state.position.y;
    }
}

const PARKED_ENTITY_POSITION: f32 = -1_000_000.0;
/// `CrumblePlatform.Sequence` (`CrumblePlatform.cs:94-169`) timings: `yield return 0.2f` per shake
/// step (one step with the player on top, three while climbing), then up to `0.4 s` of further
/// standing, then `Collidable = false` for `2 s`, then a wait for the space to clear.
const CRUMBLE_SHAKE_STEP: f32 = 0.2;
const CRUMBLE_STAND: f32 = 0.4;
const CRUMBLE_RESPAWN: f32 = 2.0;

/// One `CrumblePlatform`'s coroutine position. Held on the `Simulator` (which clones for `fork`)
/// rather than in `PlayerSnapshot`, so the gate's `--dump-field-map` audit is untouched.
#[derive(Clone, Copy)]
struct CrumbleBlockState {
    entity_index: usize,
    original: Rect,
    /// 0 waits for a player on top or climbing, 1 shakes, 2 stands, 3 is collapsed, 4 waits for
    /// the space to clear before re-arming.
    phase: u8,
    timer: f32,
    steps: u8,
}

/// Per-room entity coroutine state, held on the `Simulator` (which clones for `fork`) rather than
/// in `PlayerSnapshot`, so the gate's `--dump-field-map` audit stays untouched.
#[derive(Clone)]
struct RoomCoroutineState {
    crumble_blocks: Vec<CrumbleBlockState>,
    floaty_blocks: Vec<FloatyBlockGroup>,
    switch_gates: Vec<SwitchGateState>,
    touch_switches: Vec<TouchSwitchState>,
    /// The room's `switches_<room>` session flag (`Switch.SetLevelFlag`): set when a *persistent*
    /// gate's sequence starts (`SwitchGate.cs:109-112`), and threaded across segments by the gate,
    /// because `SwitchGate.Awake` short-circuits straight to the open position when it is set.
    switches_on: bool,
}

/// `FloatySpaceBlock.sinkTimer = 0.3f` (`FloatySpaceBlock.cs:247`): a rider re-arms the sink each
/// frame, and the timer counts down at `Engine.DeltaTime` per frame once nobody is riding (`:249-252`).
const FLOATY_SINK_REARM: f32 = 0.3;
/// `yLerp = Calc.Approach(yLerp, 1f, 1f * Engine.DeltaTime)` (`FloatySpaceBlock.cs:255`) and the
/// mirrored approach back to zero (`:259`): a full ramp takes one second each way.
const FLOATY_Y_LERP_RATE: f32 = 1.0;
/// `MathHelper.Lerp(value.Y, value.Y + 12f, Ease.SineInOut(yLerp))` (`FloatySpaceBlock.cs:287`).
const FLOATY_SINK_DEPTH: f32 = 12.0;
/// `(float)Math.Sin(sineWave) * 4f` (`FloatySpaceBlock.cs:270`).
const FLOATY_SINE_AMPLITUDE: f32 = 4.0;
/// `Calc.YoYo(Ease.QuadIn(dashEase)) * dashDirection * 8f` (`FloatySpaceBlock.cs:271`).
const FLOATY_DASH_DISTANCE: f32 = 8.0;
/// `dashEase = Calc.Approach(dashEase, 0f, Engine.DeltaTime * 1.5f)` (`FloatySpaceBlock.cs:262`),
/// so an `OnDash` (`:210-218`) offset peaks 0.195 s in and is gone after 2/3 s.
const FLOATY_DASH_EASE_RATE: f32 = 1.5;

/// One `FloatySpaceBlock` in a group. Every member carries its own `Platform.movementCounter`
/// (`Platform.cs:11`) because `MoveToY`/`MoveToX` (`:221-245`) accumulate the sub-pixel remainder
/// per platform before the rounded `MoveHExact`/`MoveVExact` step.
#[derive(Clone, Copy)]
struct FloatyBlockMember {
    entity_index: usize,
    /// The collider at `Awake`, i.e. this platform's `Moves` value (`FloatySpaceBlock.cs:72`,
    /// `:168`); every target is expressed relative to it.
    original: Rect,
    /// `Platform.movementCounter`, the exact-position remainder.
    remainder: Vec2,
}

/// One `FloatySpaceBlock` group and its shared phase. `AddToGroupAndFindChildren`
/// (`FloatySpaceBlock.cs:147-194`) merges touching blocks with the same `tiletype` into one group,
/// whose first-added member is `MasterOfGroup`; only that master runs the `Update` head
/// (`:220-264`) and `MoveToTarget` (`:268-293`), so `sinkTimer`, `yLerp`, `sineWave`, `dashEase` and
/// `dashDirection` exist once per group.
#[derive(Clone)]
struct FloatyBlockGroup {
    members: Vec<FloatyBlockMember>,
    /// `FloatySpaceBlock.sineWave` (`:19`) as its constructor left it: `0` when the map set
    /// `disableSpawnOffset` (`:56`), otherwise the `Calc.Random.NextFloat(2π)` draw (`:52`) taken
    /// at the master's position in the level's construction sequence.
    sine_wave: f32,
    /// `sinkTimer` (`:17`).
    sink_timer: f32,
    /// `yLerp` (`:15`), the 0..1 sink ramp.
    y_lerp: f32,
    /// `dashEase` (`:21`).
    dash_ease: f32,
    /// `dashDirection` (`:23`).
    dash_direction: Vec2,
}

const SWITCH_GATE_OPEN_BEAT: f32 = 0.1;
const SWITCH_GATE_ICON_RAMP: f32 = 0.5;
const SWITCH_GATE_SLIDE: f32 = 2.0;
const SWITCH_GATE_SETTLE: f32 = 1.8;

/// One `SwitchGate`'s `Sequence` position (`SwitchGate.cs:102-145`).
#[derive(Clone, Copy)]
struct SwitchGateState {
    entity_index: usize,
    /// The collider the sequence tweens from.
    start: Rect,
    /// `nodes[0]`: where an open gate slides to.
    target: Rect,
    /// 0 waits for the room's switches, 1 the 0.1 s beat, 2 the 0.5 s icon ramp, 3 the second
    /// 0.1 s beat, 4 the 2 s `Ease.CubeOut` slide, 5 the closing 1.8 s, 6 finished.
    phase: u8,
    timer: f32,
}

/// One `TouchSwitch`'s `Switch` component: `Switch(groundReset: false)`, so it never deactivates.
#[derive(Clone, Copy)]
struct TouchSwitchState {
    entity_index: usize,
    activated: bool,
}

fn initialize_room_coroutines(map: &mut Map) -> RoomCoroutineState {
    let crumble_blocks = initialize_crumble_blocks(map);
    let floaty_blocks = initialize_floaty_blocks(map);
    let switch_gates = map
        .entities
        .iter()
        .enumerate()
        .filter(|(_, entity)| entity.kind == EntityKind::SwitchGate)
        .map(|(entity_index, entity)| {
            // `SwitchGate(data, offset)` (`SwitchGate.cs:63-64`) passes `data.Nodes[0] + offset`
            // as the open target, and `Awake`/the sequence tween the whole collider onto it.
            let target = entity
                .nodes
                .first()
                .map(|node| {
                    Rect::new(
                        node.x,
                        node.y,
                        entity.bounds.width,
                        entity.bounds.height,
                    )
                })
                .unwrap_or(entity.bounds);
            SwitchGateState {
                entity_index,
                start: entity.bounds,
                target,
                phase: 0,
                timer: 0.0,
            }
        })
        .collect();
    let touch_switches = map
        .entities
        .iter()
        .enumerate()
        .filter(|(_, entity)| entity.kind == EntityKind::TouchSwitch)
        .map(|(entity_index, _)| TouchSwitchState {
            entity_index,
            activated: false,
        })
        .collect();
    RoomCoroutineState {
        crumble_blocks,
        floaty_blocks,
        switch_gates,
        touch_switches,
        switches_on: false,
    }
}

/// `SwitchGate.Sequence` (`SwitchGate.cs:102-145`) plus the `TouchSwitch` activations that gate it.
///
/// `TouchSwitch.OnPlayer` (`TouchSwitch.cs:104-110`) calls `Switch.Activate()`, and `Switch.Check`
/// only reports true once **every** `Switch` component in the room has `Finish`ed
/// (`Switch.FinishedCheck`, `Switch.cs:99-110`) - which is what the gate's
/// `while (!Switch.Check(Scene))` loop waits for. The slide is `MoveTo(Lerp(start, node, Eased))`
/// with `Ease.CubeOut`, and `Monocle.Solid.MoveTo` moves by whole pixels.
fn advance_switch_gates(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    state: &mut RoomCoroutineState,

) {
    let dt = p.frame_delta_time;
    // The gate's `while (!Switch.Check(Scene))` coroutine observes the activations as they stand
    // when its own update runs, so `all_on` is sampled *before* this frame's `TouchSwitch`
    // activations are applied.
    let all_on = state.switches_on
        || (!state.touch_switches.is_empty()
            && state.touch_switches.iter().all(|switch| switch.activated));
    // `TouchSwitch.OnPlayer` (`TouchSwitch.cs:104-110`) runs from the `PlayerCollider` pass, so it
    // sees the live hurtbox; the 30x30 activation box is the switch's own decoded rectangle.
    let hurtbox = current_player_hurt_rect(p);
    for index in 0..state.touch_switches.len() {
        let switch = state.touch_switches[index];
        let Some(entity) = map.entities.get(switch.entity_index) else {
            continue;
        };
        if !switch.activated && hurtbox.intersects(entity.bounds) {
            state.touch_switches[index].activated = true;
        }
    }
    for index in 0..state.switch_gates.len() {
        let gate = state.switch_gates[index];
        match gate.phase {
            0 => {
                if !all_on {
                    continue;
                }
                if state.switches_on {
                    // `Awake` (`SwitchGate.cs:68-82`): the room's `switches_<room>` flag is already
                    // set, so the gate is simply at its target.
                    if let Some(entity) = map.entities.get_mut(gate.entity_index) {
                        entity.bounds.x = gate.target.x;
                        entity.bounds.y = gate.target.y;
                    }
                    state.switch_gates[index].phase = 6;
                    continue;
                }
                if map.entities[gate.entity_index].direction.x != 0.0 {
                    state.switches_on = true;
                }
                state.switch_gates[index].phase = 1;
                state.switch_gates[index].timer = SWITCH_GATE_OPEN_BEAT;
            }
            1 => {
                state.switch_gates[index].timer -= dt;
                if state.switch_gates[index].timer > 0.0 {
                    continue;
                }
                state.switch_gates[index].phase = 2;
                state.switch_gates[index].timer = SWITCH_GATE_ICON_RAMP;
            }
            2 => {
                state.switch_gates[index].timer -= dt;
                if state.switch_gates[index].timer > 0.0 {
                    continue;
                }
                state.switch_gates[index].phase = 3;
                state.switch_gates[index].timer = SWITCH_GATE_OPEN_BEAT;
            }
            3 => {
                state.switch_gates[index].timer -= dt;
                if state.switch_gates[index].timer > 0.0 {
                    continue;
                }
                // `Tween.Create(..., start: true)` is added from inside the coroutine, so Monocle
                // updates it on the following frame; priming the timer with one tick starts the
                // slide on the same frame the game does. Starting it unprimed cost one frame on
                // `2-OldSite|0|6`, where the player's head clipped the gate.
                state.switch_gates[index].phase = 4;
                state.switch_gates[index].timer = dt;
            }
            4 => {
                state.switch_gates[index].timer += dt;
                let t = (state.switch_gates[index].timer / SWITCH_GATE_SLIDE).min(1.0);
                let eased = 1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t);
                if let Some(entity) = map.entities.get_mut(gate.entity_index) {
                    // `Solid.MoveTo(position)` is `MoveHExact((int)(position.X - X))` with the same
                    // on Y, so the tween's fractional target is quantized by *truncation toward
                    // zero* against the collider's current position - not by rounding the
                    // interpolated position. Rounding here cost one frame on two segments.
                    let x_target = gate.start.x + (gate.target.x - gate.start.x) * eased;
                    let y_target = gate.start.y + (gate.target.y - gate.start.y) * eased;
                    entity.bounds.x += (x_target - entity.bounds.x) as i32 as f32;
                    entity.bounds.y += (y_target - entity.bounds.y) as i32 as f32;
                }
                if t >= 1.0 {
                    state.switch_gates[index].phase = 5;
                    state.switch_gates[index].timer = SWITCH_GATE_SETTLE;
                }
            }
            5 => {
                state.switch_gates[index].timer -= dt;
                if state.switch_gates[index].timer > 0.0 {
                    continue;
                }
                state.switch_gates[index].phase = 6;
            }
            _ => {}
        }
    }
}

fn initialize_crumble_blocks(map: &Map) -> Vec<CrumbleBlockState> {
    map.entities
        .iter()
        .enumerate()
        .filter(|(_, entity)| entity.kind == EntityKind::CrumbleBlock)
        .map(|(entity_index, entity)| CrumbleBlockState {
            entity_index,
            original: entity.bounds,
            phase: 0,
            timer: 0.0,
            steps: 0,
        })
        .collect()
}

/// `CrumblePlatform.Sequence` (`CrumblePlatform.cs:94-169`), one frame at a time.
///
/// `GetPlayerOnTop()` is `CollideFirst<Player>(Position - Vector2.UnitY)`, i.e. the player's
/// collider overlapping the block's rectangle shifted up by one pixel - what standing on its top
/// edge produces. Collapsing parks the entity's bounds, the simulator's idiom for
/// `Collidable = false`; re-arming restores the original rectangle.
fn advance_crumble_blocks(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    states: &mut [CrumbleBlockState],
) {
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    let dt = p.frame_delta_time;
    for state in states.iter_mut() {
        let Some(entity) = map.entities.get(state.entity_index) else {
            continue;
        };
        let bounds = entity.bounds;
        let on_top =
            player.intersects(Rect::new(bounds.x, bounds.y - 1.0, bounds.width, bounds.height));
        match state.phase {
            0 => {
                if on_top {
                    state.steps = 1;
                    state.timer = CRUMBLE_SHAKE_STEP;
                    state.phase = 1;
                }
            }
            1 => {
                state.timer -= dt;
                if state.timer > 0.0 {
                    continue;
                }
                state.steps = state.steps.saturating_sub(1);
                if state.steps > 0 {
                    state.timer = CRUMBLE_SHAKE_STEP;
                    continue;
                }
                state.phase = 2;
                state.timer = CRUMBLE_STAND;
            }
            2 => {
                // `while (timer > 0f && GetPlayerOnTop() != null)`: leaving the top ends the wait
                // early, so the block collapses as soon as the player steps off.
                if on_top {
                    state.timer -= dt;
                    if state.timer > 0.0 {
                        continue;
                    }
                }
                park_entity(&mut map.entities[state.entity_index]);
                state.phase = 3;
                state.timer = CRUMBLE_RESPAWN;
            }
            3 => {
                state.timer -= dt;
                if state.timer > 0.0 {
                    continue;
                }
                state.phase = 4;
            }
            _ => {
                // `while (CollideCheck<Actor>() || CollideCheck<Solid>()) yield return null;`
                if player.intersects(state.original) || map.non_dream_solid_at(state.original) {
                    continue;
                }
                map.entities[state.entity_index].bounds = state.original;
                state.phase = 0;
            }
        }
    }
}

/// `FloatySpaceBlock.Awake`/`AddToGroupAndFindChildren` (`FloatySpaceBlock.cs:65-194`).
///
/// A group is a maximal set of `floatySpaceBlock` entities that touch each other through
/// `CollideCheck(new Rectangle(X - 1, Y, Width + 2, Height), other)` or
/// `CollideCheck(new Rectangle(X, Y - 1, Width, Height + 2), other)` (`:189`) while sharing the
/// same `tiletype`. The group's **master** is the member whose `Awake` ran first - the lowest map
/// entity index, since `Level.LoadLevel` constructs `levelData.Entities` in order
/// (`Level.cs:468`) - and it alone owns the phase (`:220-264`).
///
/// `Moves` (`:72`, `:168`) records every platform's position at `Awake`; the master's
/// `MoveToTarget` (`:268-293`) recomputes each member's absolute target from that value, so each
/// member's offset from its own original is what the group shares.
///
/// `sineWave` (`:19`) is drawn from `Calc.Random.NextFloat(2π)` in the constructor (`:52`) unless
/// the map set `disableSpawnOffset`; see [`advance_floaty_blocks`] for why the two cases are
/// modelled differently.
fn initialize_floaty_blocks(map: &mut Map) -> Vec<FloatyBlockGroup> {
    let indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| {
            (entity.kind == EntityKind::FloatySpaceBlock).then_some(index)
        })
        .collect();
    // `Calc.Random` is `System.Random` re-seeded to the level's `LoadSeed` for the whole of
    // `Level.LoadLevel` (`Level.cs:386-1386`), and each entity's *constructor* runs inside that
    // window in map entity order (`Level.cs:468`), while every `Awake` - where the autotiler
    // draws (`Autotiler.cs:253`) - runs afterwards (`Level.cs:335` -> `Scene.Begin`, or the next
    // `EntityList` awake pass, `EntityList.cs:112-122`, both after `Calc.PopRandom`). So the
    // phase a block draws is the sequence position reached by its predecessors' constructors.
    let mut random = crate::legacy_random::LegacyRandom::new(map.load_seed);
    let mut drawn: Vec<f32> = Vec::with_capacity(indices.len());
    let mut next = indices.iter().copied().peekable();
    for (index, entity) in map.entities.iter().enumerate() {
        if next.peek() == Some(&index) {
            next.next();
            // `if (!disableSpawnOffset) sineWave = Calc.Random.NextFloat(Math.PI * 2f)`
            // (`FloatySpaceBlock.cs:50-57`): the pinned-zero branch draws nothing.
            drawn.push(if entity.direction.x != 0.0 {
                0.0
            } else {
                random.next_float_max(std::f32::consts::PI * 2.0)
            });
        } else {
            for _ in 0..entity_construction_draws(entity) {
                random.next_double();
            }
        }
    }
    let mut grouped = vec![false; map.entities.len()];
    let mut groups = Vec::new();
    for &root in &indices {
        if grouped[root] {
            continue;
        }
        let tile = map.entities[root].direction.y;
        grouped[root] = true;
        let mut members = vec![root];
        let mut queue = vec![root];
        while let Some(from) = queue.pop() {
            let bounds = map.entities[from].bounds;
            // `Scene.CollideCheck(rect, entity)` against the entity's `Hitbox(width, height)`
            // with collider offset `(0, 0)` (`Solid.cs:25-29`) is plain rectangle overlap.
            let probes = [
                Rect::new(bounds.x - 1.0, bounds.y, bounds.width + 2.0, bounds.height),
                Rect::new(bounds.x, bounds.y - 1.0, bounds.width, bounds.height + 2.0),
            ];
            for &other in &indices {
                if grouped[other] || map.entities[other].direction.y != tile {
                    continue;
                }
                let other_bounds = map.entities[other].bounds;
                if probes.iter().any(|probe| probe.intersects(other_bounds)) {
                    grouped[other] = true;
                    members.push(other);
                    queue.push(other);
                }
            }
        }
        members.sort_unstable();
        let master = members[0];
        groups.push(FloatyBlockGroup {
            members: members
                .into_iter()
                .map(|entity_index| FloatyBlockMember {
                    entity_index,
                    original: map.entities[entity_index].bounds,
                    remainder: Vec2::default(),
                })
                .collect(),
            sine_wave: drawn[indices.binary_search(&master).unwrap_or(0)],
            sink_timer: 0.0,
            y_lerp: 0.0,
            dash_ease: 0.0,
            dash_direction: Vec2::default(),
        });
    }
    for group in &mut groups {
        pre_roll_floaty_group(map, group);
    }
    groups
}

/// `Calc.Random` draws one room entity's **constructor** makes.
///
/// Only constructors are counted: `Awake` runs after `Calc.PopRandom`
/// (`Level.cs:386`/`:1386`), so the sprites and autotiling that draw there - `JumpthruPlatform.Awake`
/// (`JumpthruPlatform.cs:76-77`), `CrystalStaticSpinner.CreateSprites` (`CrystalStaticSpinner.cs:287`,
/// which re-seeds from the value the constructor already stored) - never touch the level's stream.
fn entity_construction_draws(entity: &crate::Entity) -> u32 {
    match entity.name.as_str() {
        // `CrystalStaticSpinner`'s constructor ends with `randomSeed = Calc.Random.Next()`
        // (`CrystalStaticSpinner.cs:168`), one draw. `CreateSprites` only *re-seeds* from that
        // value and runs from `Awake` (`:191-194`), so it consumes nothing here.
        "spinner" | "VivHelper/CustomSpinner" | "FrostHelper/IceSpinner" => 1,
        // `Lightning.toggleOffset = Calc.Random.NextFloat()` is a field initialiser (`:36`).
        "lightning" => 1,
        // `FloatingDebris`'s texture pick and rotation speed are field initialisers
        // (`FloatingDebris.cs:29`, `:33`).
        "floatingDebris" => 2,
        // `DreamBlock`'s two wobble phases are field initialisers (`DreamBlock.cs:54`, `:56`).
        "dreamBlock" => 2,
        // `SwapBlock.timer = Calc.Random.NextFloat()` (`SwapBlock.cs:33`).
        "swapBlock" => 1,
        // `Cloud.timer = Calc.Random.NextFloat() * 4f` (`Cloud.cs:45`).
        "cloud" => 1,
        // `Bumper`'s `Randomize()` draws its `SineWave` phase (`Bumper.cs:135` is the ambient
        // particle, not this one; the phase is the `SineWave` component's own randomisation).
        "bigSpinner" => 1,
        // `TouchSwitch`'s `SineWave` phase (`TouchSwitch.cs:62`) and `SwitchGate`s
        // (`SwitchGate.cs:222`) are field initialisers.
        "touchSwitch" | "switchGate" => 1,
        // `LightBeam.timer = Calc.Random.NextFloat(1000f)` (`LightBeam.cs:25`).
        "lightbeam" => 1,
        // `FallingBlock`'s constructor draws `newSeed` for its own `PushRandom`/`PopRandom`
        // autotiler block (`FallingBlock.cs:38-41`).
        "fallingBlock" => 1,
        // `SeekerBarrier`'s constructor fills `particles` with two draws per entry, one per
        // `Width * Height / 16f` (`SeekerBarrier.cs:28-31`).
        "seekerBarrier" => {
            2 * (entity.bounds.width * entity.bounds.height / 16.0).ceil() as u32
        }
        // `MoonCreature`'s constructor draws its colour (`:65`) and, through
        // `GetRandomTarget` (`:61`, `:121-122`), two more.
        "moonCreature" => 3,
        // Everything else's `Calc.Random` use is inside `Added`/`Awake`/`Update` - e.g.
        // `Spikes.CreateSprites` (`Spikes.cs:117`), `TriggerSpikes.Added` (`:198-202`),
        // `HeartGemDoor.Added` (`:132-134`), `JumpthruPlatform.Awake` (`JumpthruPlatform.cs:76-77`)
        // - which all run after `Calc.PopRandom` (`Level.cs:1386`).
        _ => 0,
    }
}

/// `FloatySpaceBlock.TryToInitPosition` (`:128-145`): the one `MoveToTarget` at `Awake`, run once
/// every group member is awake, before any `Update`.
///
/// The phase a replay starts from is the constructor's own draw; the anchor row a gate segment
/// replays from is the room's first row on which `Player.Update` ran, so no idle frames are
/// pre-rolled. Measured on the 100% trace, pre-rolling the transition rows instead (41 or 80
/// frames, the segment's `leadingSkippedFrames` territory) loses two fully-matching segments,
/// because the room's entities do not step while the transition coroutine plays.
///
/// Nothing rides a block here, so `yLerp`, `dashEase` and the sink are all zero and the spawn
/// target is just `originalY + sin(sineWave) * 4f` (`:270`, `:287`), reached through the same
/// `Platform.MoveV` remainder (`Platform.cs:190-207`) the per-frame move uses.
fn pre_roll_floaty_group(map: &mut Map, group: &mut FloatyBlockGroup) {
    let sine = (group.sine_wave as f64).sin() as f32 * FLOATY_SINE_AMPLITUDE;
    for member in &mut group.members {
        let mut bounds = map.entities[member.entity_index].bounds;
        let target = member.original.y + sine;
        let exact = bounds.y + member.remainder.y;
        member.remainder.y += target - exact;
        let step = member.remainder.y.round_ties_even();
        member.remainder.y -= step;
        bounds.y += step;
        map.entities[member.entity_index].bounds = bounds;
    }
}


/// `FloatySpaceBlock.Update` (`FloatySpaceBlock.cs:220-266`) and `MoveToTarget` (`:268-293`), one
/// frame at a time, for every group.
///
/// Only `MasterOfGroup` runs the head (`:223`), and `Platform.Update`, reached through
/// `base.Update()` (`:222`), clears `LiftSpeed` before the move (`Platform.cs:56`). The rider test
/// is `HasPlayerRider()` over all group members and their attached `JumpThru`s (`:226-244`);
/// `Actor.IsRiding(Solid)` is `CollideCheck(this, Position + UnitY)` (`Actor.cs:129-132`), which is
/// `player_riding_solid`'s default arm. A rider re-arms `sinkTimer` to 0.3 s every frame (`:247`)
/// and otherwise the timer counts down (`:249-252`) while `yLerp` ramps to 1 / back to 0 over one
/// second (`:253-260`).
///
/// `sineWave += Engine.DeltaTime` (`:261`) before `MoveToTarget`, so the phase advances even while
/// the block is still. Two cases:
///
/// * `disableSpawnOffset` blocks start at exactly zero (`:56`) and a gate segment begins on the
///   room's first row, so their phase is the elapsed replay time and is reproduced exactly.
/// * The rest start at a `Calc.Random.NextFloat(2π)` draw (`:52`). `Calc.Random` is a
///   `System.Random` re-seeded to the level's `LoadSeed` for the whole of `Level.LoadLevel`
///   (`Calc.cs:19`, `Level.cs:386-1386`), and the draw index depends on every `Calc.Random` call
///   the room's earlier entities made, so a replay cannot recover the phase. The bob term is
///   therefore left at its mean of zero rather than guessed.
fn advance_floaty_blocks(p: &mut PlayerSnapshot, map: &mut Map, groups: &mut [FloatyBlockGroup]) {
    let dt = p.frame_delta_time;
    // `OnDash` fired earlier in this same frame, inside `Player.Update`. Its
    // `MasterOfGroup && dashEase <= 0.2f` guard (`FloatySpaceBlock.cs:212`) is
    // evaluated here, where the group state is, and before the frame's own
    // `dashEase` approach (`:262`) exactly like the source.
    if let Some((hit, direction)) = p.pending_floaty_dash.take()
        && let Some(group) = groups
            .iter_mut()
            .find(|group| group.members[0].entity_index == hit)
        && group.dash_ease <= 0.2
    {
        group.dash_ease = 1.0;
        group.dash_direction = direction;
    }
    for group in groups.iter_mut() {
        let mut ridden = false;
        for member in &group.members {
            if map
                .entities
                .get(member.entity_index)
                .is_some_and(|entity| player_riding_solid(p, entity.bounds))
            {
                ridden = true;
                break;
            }
        }
        if ridden {
            group.sink_timer = FLOATY_SINK_REARM;
        } else if group.sink_timer > 0.0 {
            group.sink_timer -= dt;
        }
        if group.sink_timer > 0.0 {
            group.y_lerp = approach(group.y_lerp, 1.0, FLOATY_Y_LERP_RATE * dt);
        } else {
            group.y_lerp = approach(group.y_lerp, 0.0, FLOATY_Y_LERP_RATE * dt);
        }
        group.sine_wave += dt;
        group.dash_ease = approach(group.dash_ease, 0.0, dt * FLOATY_DASH_EASE_RATE);
        move_floaty_group_to_target(p, map, group);
    }
}

/// `FloatySpaceBlock.MoveToTarget` (`FloatySpaceBlock.cs:268-293`).
///
/// `MoveToTarget` makes two passes over `Moves`: `i == 0` moves only the platforms that currently
/// have a rider, `i == 1` only those that do not (`:284`). Both passes aim at the same target, so
/// the only observable difference is *when* inside the frame a carried player is moved; the
/// per-platform state is identical either way and every platform moves exactly once per frame.
///
/// The target is `Lerp(originalY, originalY + 12f, Ease.SineInOut(yLerp)) + sin(sineWave) * 4f`
/// vertically (`:287`, `:270`) and `originalX` horizontally (`:289`), both plus
/// `Calc.YoYo(Ease.QuadIn(dashEase)) * dashDirection * 8f` (`:271`). `MoveToY`/`MoveToX`
/// (`:288-289`) are `Platform.MoveV`/`MoveH` (`Platform.cs:190-207`, `:159-176`): the requested
/// displacement is added to `ExactPosition` (collider + `movementCounter`), the accumulated
/// remainder is rounded with `Math.Round` and only the whole-pixel part reaches
/// `MoveVExact`/`MoveHExact`, which is the whole-pixel carry/push `move_runtime_solid_exact`
/// models. The platform's `LiftSpeed` for that axis is the *fractional* displacement over
/// `Engine.DeltaTime`, and each axis overwrites only its own component.
fn move_floaty_group_to_target(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    group: &mut FloatyBlockGroup,
) {
    let dt = p.frame_delta_time;
    let sine = (group.sine_wave as f64).sin() as f32 * FLOATY_SINE_AMPLITUDE;
    // `Calc.YoYo(Ease.QuadIn(dashEase)) * dashDirection * 8f` (`:271`): scalar-times-vector
    // first, then the distance, in that order.
    let dash_amount = yoyo(quad_in(group.dash_ease));
    let dash = Vec2::new(
        dash_amount * group.dash_direction.x * FLOATY_DASH_DISTANCE,
        dash_amount * group.dash_direction.y * FLOATY_DASH_DISTANCE,
    );
    let sink = FLOATY_SINK_DEPTH * ease_sine_in_out(group.y_lerp);
    for index in 0..group.members.len() {
        let entity_index = group.members[index].entity_index;
        let original = group.members[index].original;
        let target_y = original.y + sink + sine + dash.y;
        let target_x = original.x + dash.x;
        let env = solid_collision_env(map, entity_index);
        let mut lift = Vec2::default();
        // `key.MoveToY(...)` then `key.MoveToX(...)` (`FloatySpaceBlock.cs:288-289`).
        for horizontal in [false, true] {
            let bounds = map.entities[entity_index].bounds;
            let exact = if horizontal {
                bounds.x + group.members[index].remainder.x
            } else {
                bounds.y + group.members[index].remainder.y
            };
            let target = if horizontal { target_x } else { target_y };
            // `Platform.MoveH`/`MoveV` (`Platform.cs:159-207`): `LiftSpeed` is the exact
            // fractional displacement over `Engine.DeltaTime`, `movementCounter` accumulates it,
            // and only the rounded whole-pixel part reaches `MoveHExact`/`MoveVExact`. Each axis
            // overwrites its own `LiftSpeed` component, so the Y move reports `(0, dy/dt)` and the
            // X move that follows reports `(dx/dt, dy/dt)`.
            if horizontal {
                lift.x = (target - exact) / dt;
                group.members[index].remainder.x += target - exact;
            } else {
                lift.y = (target - exact) / dt;
                group.members[index].remainder.y += target - exact;
            }
            let step = if horizontal {
                group.members[index].remainder.x
            } else {
                group.members[index].remainder.y
            }
            .round_ties_even();
            if step == 0.0 {
                continue;
            }
            if horizontal {
                group.members[index].remainder.x -= step;
            } else {
                group.members[index].remainder.y -= step;
            }
            move_runtime_solid_exact(
                p,
                &mut map.entities[entity_index].bounds,
                &env,
                horizontal,
                step,
                lift,
            );
        }
    }
}

/// `Ease.SineInOut` (`Ease.cs:15`): `(0f - (float)Math.Cos((float)Math.PI * t)) / 2f + 0.5f`,
/// evaluated in single precision exactly like the source.
fn ease_sine_in_out(t: f32) -> f32 {
    let argument = (std::f32::consts::PI * t) as f64;
    (-(argument.cos() as f32)) / 2.0 + 0.5
}

/// `Ease.QuadIn` (`Ease.cs:17`).
fn quad_in(t: f32) -> f32 {
    t * t
}

/// `Calc.YoYo` (`Calc.cs:721-728`).
fn yoyo(value: f32) -> f32 {
    if value <= 0.5 {
        value * 2.0
    } else {
        1.0 - (value - 0.5) * 2.0
    }
}
const CASSETTE_BEAT_INTERVAL: f32 = 355.0 / (678.0 * std::f32::consts::PI);

fn park_entity(entity: &mut crate::Entity) {
    entity.bounds.x = PARKED_ENTITY_POSITION;
    entity.bounds.y = PARKED_ENTITY_POSITION;
}

fn initialize_cassette_blocks(p: &mut PlayerSnapshot, map: &mut Map) {
    let block_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::CassetteBlock).then_some(index))
        .collect();
    p.cassette_blocks.truncate(block_indices.len());
    if block_indices.is_empty() {
        p.cassette_manager = crate::CassetteManagerSnapshot {
            tape_taken: p.cassette_manager.tape_taken,
            ..crate::CassetteManagerSnapshot::default()
        };
        return;
    }

    // Chapter state, not room state: `Session.Cassette` (`Cassette.cs:176`) is true for the rest of
    // an A-side once the tape is taken, and then the game constructs no manager at all -
    // `Level.ShouldCreateCassetteManager` is `!Session.Cassette` for `AreaMode.Normal`
    // (`Level.cs:278-288`) and gates the construction at `Level.cs:657` and `OnLevelStart` at
    // `Level.cs:1355-1358`. So nothing ever calls `SetActivatedSilently`
    // (`CassetteBlock.cs:392-394`, reached only from `CassetteBlockManager.SilentUpdateBlocks`,
    // `CassetteBlockManager.cs:197-206`) and every block keeps the `Collidable = false` its
    // constructor set (`CassetteBlock.cs:70-76`), here at its untouched map position.
    if p.cassette_manager.tape_taken {
        p.cassette_manager = crate::CassetteManagerSnapshot {
            tape_taken: true,
            ..crate::CassetteManagerSnapshot::default()
        };
        for (block_index, entity_index) in block_indices.into_iter().enumerate() {
            let bounds = map.entities[entity_index].bounds;
            let index = map.entities[entity_index]
                .direction
                .x
                .round()
                .clamp(0.0, 255.0) as u8;
            let state = crate::CassetteBlockSnapshot {
                position: Vec2::new(bounds.x, bounds.y),
                start: Vec2::new(bounds.x, bounds.y),
                width: bounds.width,
                height: bounds.height,
                index,
                activated: false,
                collidable: false,
            };
            if block_index == p.cassette_blocks.len() {
                p.cassette_blocks.push(state);
            } else {
                p.cassette_blocks[block_index] = state;
            }
            park_entity(&mut map.entities[entity_index]);
        }
        return;
    }

    if !p.cassette_manager.initialized {
        p.cassette_manager.max_beat = block_indices
            .iter()
            .map(|&index| map.entities[index].direction.x.round().clamp(0.0, 255.0) as u8 + 1)
            .max()
            .unwrap_or(1);
        p.cassette_manager.tempo_mult = block_indices
            .iter()
            .map(|&index| map.entities[index].direction.y)
            .find(|tempo| *tempo > 0.0)
            .unwrap_or(1.0);
        p.cassette_manager.current_index = if p.cassette_manager.beat_index % 8 >= 5 {
            p.cassette_manager.max_beat.saturating_sub(2)
        } else {
            p.cassette_manager.max_beat.saturating_sub(1)
        };
        // The repository-owned Playground is a custom area with cassette
        // music. CassetteBlockManager.Update creates its sfx on the first
        // frame and takes the branch which skips AdvanceMusic once.
        p.cassette_manager.startup_music_pending = true;
        p.cassette_manager.initialized = true;
    }

    for (block_index, entity_index) in block_indices.into_iter().enumerate() {
        if block_index == p.cassette_blocks.len() {
            let bounds = map.entities[entity_index].bounds;
            let index = map.entities[entity_index]
                .direction
                .x
                .round()
                .clamp(0.0, 255.0) as u8;
            let active = index == p.cassette_manager.current_index;
            let position = Vec2::new(bounds.x, bounds.y + if active { 0.0 } else { 2.0 });
            p.cassette_blocks.push(crate::CassetteBlockSnapshot {
                position,
                start: Vec2::new(bounds.x, bounds.y),
                width: bounds.width,
                height: bounds.height,
                index,
                activated: active,
                collidable: active,
            });
        }
        let state = &p.cassette_blocks[block_index];
        let entity = &mut map.entities[entity_index];
        if state.collidable {
            entity.bounds = Rect::new(
                state.position.x,
                state.position.y,
                state.width,
                state.height,
            );
        } else {
            park_entity(entity);
        }
    }
}

fn spinner_in_view(position: Vec2, camera: Vec2) -> bool {
    position.x > camera.x - 16.0
        && position.y > camera.y - 16.0
        && position.x < camera.x + 336.0
        && position.y < camera.y + 196.0
}

fn initialize_spinners(p: &mut PlayerSnapshot, map: &mut Map) {
    let spinner_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| {
            (entity.kind == EntityKind::CrystalStaticSpinner).then_some(index)
        })
        .collect();
    p.spinners.truncate(spinner_indices.len());
    for (spinner_index, entity_index) in spinner_indices.into_iter().enumerate() {
        if spinner_index == p.spinners.len() {
            let bounds = map.entities[entity_index].bounds;
            p.spinners.push(crate::SpinnerSnapshot {
                position: Vec2::new(
                    bounds.x + bounds.width * 0.5,
                    bounds.y + bounds.height * 0.5,
                ),
                // The real value is a random float in [0, 1). Persisting it in
                // the snapshot makes split simulation deterministic; this
                // map-order seed is used only for a fresh portable snapshot.
                offset: ((spinner_index as f32 + 1.0) * 0.618_034).fract(),
                visible: false,
                collidable: true,
            });
        }
        sync_spinner_entity(&mut map.entities[entity_index], &p.spinners[spinner_index]);
    }
}

fn sync_spinner_entity(entity: &mut crate::Entity, state: &crate::SpinnerSnapshot) {
    if state.visible && state.collidable {
        entity.bounds = Rect::new(state.position.x - 8.0, state.position.y - 6.0, 16.0, 12.0);
    } else {
        park_entity(entity);
    }
}

fn bumper_entity_indices(map: &Map) -> Vec<usize> {
    map.entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::Bumper).then_some(index))
        .collect()
}

fn initialize_bumpers(p: &mut PlayerSnapshot, map: &mut Map) {
    let indices = bumper_entity_indices(map);
    p.bumpers.truncate(indices.len());
    for (bumper_index, entity_index) in indices.into_iter().enumerate() {
        let entity = &mut map.entities[entity_index];
        let anchor = Vec2::new(
            entity.bounds.x + entity.bounds.width * 0.5,
            entity.bounds.y + entity.bounds.height * 0.5,
        );
        if bumper_index == p.bumpers.len() {
            // Fresh portable simulations have no source Randomize() sample.
            // Keep their default phase deterministic; real comparisons replace
            // it with the collector's live SineWave.Counter at state zero.
            p.bumpers.push(crate::BumperSnapshot {
                anchor,
                position: anchor,
                sine_counter: 0.0,
                respawn_timer: 0.0,
            });
        } else {
            // Map data carries Bumper's immutable constructor position;
            // captured state supplies only its live Position and phase.
            p.bumpers[bumper_index].anchor = anchor;
        }
        let state = &p.bumpers[bumper_index];
        entity.bounds = Rect::new(state.position.x - 12.0, state.position.y - 12.0, 24.0, 24.0);
    }
}

fn advance_bumpers(p: &mut PlayerSnapshot, map: &mut Map) {
    // Bumper's SineWave runs before its PlayerCollider callback. Its following
    // UpdatePosition writes the two source components:
    // `anchor + new Vector2(sine.Value * 3f, sine.ValueOverTwo * 2f)`.
    // Therefore a collision this entity frame samples the newly published
    // Circle(12) position, rather than the previous frame's one.
    for (bumper_index, entity_index) in bumper_entity_indices(map).into_iter().enumerate() {
        let state = &mut p.bumpers[bumper_index];
        let entity = &mut map.entities[entity_index];
        // Monocle.SineWave.Update advances in cycles/second, not radians:
        // Counter += 2π * Frequency * DeltaTime, then Counter's setter
        // refreshes Value, ValueOverTwo, and TwoValue.
        state.sine_counter = (state.sine_counter
            + std::f32::consts::TAU * 0.44 * p.frame_delta_time)
            .rem_euclid(std::f32::consts::TAU * 4.0);
        state.position = Vec2::new(
            state.anchor.x + state.sine_counter.sin() * 3.0,
            state.anchor.y + (state.sine_counter * 0.5).sin() * 2.0,
        );
        state.respawn_timer = (state.respawn_timer - p.frame_delta_time).max(0.0);
        entity.bounds = Rect::new(state.position.x - 12.0, state.position.y - 12.0, 24.0, 24.0);
    }
}

fn initialize_refills(p: &mut PlayerSnapshot, map: &mut Map) {
    let refill_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::Refill).then_some(index))
        .collect();
    p.refills.truncate(refill_indices.len());
    for (refill_index, entity_index) in refill_indices.into_iter().enumerate() {
        if refill_index == p.refills.len() {
            let entity = &map.entities[entity_index];
            p.refills.push(crate::RefillSnapshot {
                two_dashes: entity.direction.x != 0.0,
                one_use: entity.single_use,
                respawn_timer: 0.0,
                collidable: true,
                removed: false,
            });
        }
    }
}

fn initialize_falling_blocks(p: &mut PlayerSnapshot, map: &mut Map) {
    let block_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::FallingBlock).then_some(index))
        .collect();
    p.falling_blocks.truncate(block_indices.len());
    for (block_index, entity_index) in block_indices.into_iter().enumerate() {
        if block_index == p.falling_blocks.len() {
            let bounds = map.entities[entity_index].bounds;
            let position = Vec2::new(bounds.x, bounds.y);
            p.falling_blocks.push(crate::FallingBlockSnapshot {
                position,
                start: position,
                collidable: true,
                ..crate::FallingBlockSnapshot::default()
            });
        }
        let state = &p.falling_blocks[block_index];
        let entity = &mut map.entities[entity_index];
        if state.collidable {
            entity.bounds = Rect::new(
                state.position.x,
                state.position.y,
                entity.bounds.width,
                entity.bounds.height,
            );
        } else {
            park_entity(entity);
        }
    }
}

fn initialize_exit_blocks(p: &mut PlayerSnapshot, map: &mut Map) {
    let indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::ExitBlock).then_some(index))
        .collect();
    p.exit_blocks.truncate(indices.len());
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    for (block_index, entity_index) in indices.into_iter().enumerate() {
        let entity = &mut map.entities[entity_index];
        if block_index == p.exit_blocks.len() {
            p.exit_blocks.push(crate::ExitBlockSnapshot {
                position: Vec2::new(entity.bounds.x, entity.bounds.y),
                collidable: !entity.bounds.intersects(player),
            });
        }
        let state = &p.exit_blocks[block_index];
        if state.collidable {
            entity.bounds.x = state.position.x;
            entity.bounds.y = state.position.y;
        } else {
            park_entity(entity);
        }
    }
}

fn advance_exit_blocks(p: &mut PlayerSnapshot, map: &mut Map) {
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    let indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::ExitBlock).then_some(index))
        .collect();
    for (block_index, entity_index) in indices.into_iter().enumerate() {
        let state = &mut p.exit_blocks[block_index];
        if state.collidable {
            continue;
        }
        let entity = &mut map.entities[entity_index];
        let original = Rect::new(
            state.position.x,
            state.position.y,
            entity.bounds.width,
            entity.bounds.height,
        );
        if !original.intersects(player) {
            state.collidable = true;
            entity.bounds.x = state.position.x;
            entity.bounds.y = state.position.y;
        }
    }
}

fn initialize_invisible_barriers(p: &mut PlayerSnapshot, map: &mut Map) {
    let indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| {
            (entity.kind == EntityKind::InvisibleBarrier).then_some(index)
        })
        .collect();
    p.invisible_barriers.truncate(indices.len());
    for (barrier_index, entity_index) in indices.into_iter().enumerate() {
        let entity = &mut map.entities[entity_index];
        if barrier_index == p.invisible_barriers.len() {
            p.invisible_barriers.push(crate::InvisibleBarrierSnapshot {
                position: Vec2::new(entity.bounds.x, entity.bounds.y),
                initialized: false,
                collidable: false,
            });
        }
        let state = &p.invisible_barriers[barrier_index];
        if state.initialized && state.collidable {
            entity.bounds.x = state.position.x;
            entity.bounds.y = state.position.y;
        } else {
            park_entity(entity);
        }
    }
}

fn advance_invisible_barriers(p: &mut PlayerSnapshot, map: &mut Map) {
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    let indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| {
            (entity.kind == EntityKind::InvisibleBarrier).then_some(index)
        })
        .collect();
    for (barrier_index, entity_index) in indices.into_iter().enumerate() {
        let state = &mut p.invisible_barriers[barrier_index];
        if state.initialized {
            continue;
        }
        let entity = &mut map.entities[entity_index];
        let original = Rect::new(
            state.position.x,
            state.position.y,
            entity.bounds.width,
            entity.bounds.height,
        );
        state.initialized = true;
        state.collidable = !original.intersects(player);
        if state.collidable {
            entity.bounds = original;
        }
    }
}

fn initialize_killboxes(p: &mut PlayerSnapshot, map: &mut Map) {
    let indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::Killbox).then_some(index))
        .collect();
    p.killboxes.truncate(indices.len());
    for (killbox_index, entity_index) in indices.into_iter().enumerate() {
        let entity = &mut map.entities[entity_index];
        if killbox_index == p.killboxes.len() {
            p.killboxes.push(crate::KillboxSnapshot {
                position: Vec2::new(entity.bounds.x, entity.bounds.y),
                collidable: false,
            });
        }
        let state = &p.killboxes[killbox_index];
        if state.collidable {
            entity.bounds.x = state.position.x;
            entity.bounds.y = state.position.y;
        } else {
            park_entity(entity);
        }
    }
}

fn advance_killboxes(p: &mut PlayerSnapshot, map: &mut Map) {
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    let indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::Killbox).then_some(index))
        .collect();
    for (killbox_index, entity_index) in indices.into_iter().enumerate() {
        let state = &mut p.killboxes[killbox_index];
        let entity = &mut map.entities[entity_index];
        let original = Rect::new(
            state.position.x,
            state.position.y,
            entity.bounds.width,
            entity.bounds.height,
        );
        if !state.collidable && player.bottom() < original.y - 32.0 {
            state.collidable = true;
            entity.bounds = original;
        } else if state.collidable && player.y > original.bottom() + 32.0 {
            state.collidable = false;
            park_entity(entity);
        }
    }
}

/// `Celeste.DashCollisionResults` as returned by the vanilla Solid entities the
/// runtime models. `CrushBlock.OnDashed` and `DashBlock.OnDashed` never return
/// `Bounce`, so that arm does not exist here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DashCollision {
    /// `DashCollisionResults.NormalCollision`: `Player.OnCollideH`/`OnCollideV`
    /// fall through to their ordinary stop.
    NormalCollision,
    /// `DashCollisionResults.NormalOverride` (`FloatySpaceBlock.OnDash`,
    /// `FloatySpaceBlock.cs:217`): `Player.cs:3158-3161` rewrites it to
    /// `NormalCollision` *before* the state-5 remap at `:3162-3165`, so it takes
    /// the ordinary stop even while state 5 (`StRedDash`) is active - unlike
    /// `NormalCollision`, which that remap turns into `Ignore`.
    NormalOverride,
    /// `Player.cs:3168-3170` / `3266-3268`.
    Rebound,
    /// `Player.cs:3174-3176` / `3272-3274`.
    Ignore,
}

/// `CrushBlock.cs:37`'s `AttackSequence` wind-up
/// (`StartShaking(0.4f); yield return 0.4f;`, `CrushBlock.cs:422-423`).
const CRUSH_BLOCK_WIND_UP: f32 = 0.4;

/// Index of the *first* runtime Solid entity overlapping `rect`, in map entity
/// order. `Monocle.Entity.CollideFirst<Solid>` walks the scene's `Solid` list in
/// insertion order, and `LevelLoader` adds `Level.SolidTiles` before
/// `Level.LoadLevel` adds the room entities (`LevelLoader.cs:271`,
/// `Level.cs:357`), so a tile hit is `SolidTiles` and never has an
/// `OnDashCollide`. Callers exclude static tile solids before using this.
fn first_solid_entity_at(map: &Map, rect: Rect) -> Option<usize> {
    map.entities.iter().position(|entity| {
        is_solid_entity(entity.kind)
            && solid_is_collidable(entity)
            && entity.bounds.intersects(rect)
    })
}

/// Position of `entity_index` inside the per-kind state vector this module
/// keeps in `PlayerSnapshot`, matching the `initialize_*` filter order.
fn kind_state_index(map: &Map, entity_index: usize, kind: EntityKind) -> Option<usize> {
    map.entities[..=entity_index]
        .iter()
        .filter(|entity| entity.kind == kind)
        .count()
        .checked_sub(1)
}

/// `CrushBlock.CanActivate` (`CrushBlock.cs:284-303`). `axes` limits which
/// directions the crusher may travel, `giant` refuses leftward travel for a
/// chilled-out 48x48 crusher, and a block already travelling in `direction`
/// cannot start the same attack twice.
fn crush_block_can_activate(
    entity: &crate::Entity,
    state: &crate::CrushBlockSnapshot,
    direction: Vec2,
) -> bool {
    let axes = entity.direction.x as i32;
    let chill_out = entity.direction.y != 0.0;
    let giant = entity.bounds.width >= 48.0 && entity.bounds.height >= 48.0 && chill_out;
    if giant && direction.x <= 0.0 {
        return false;
    }
    if state.can_activate && state.crush_dir != direction {
        if direction.x != 0.0 && axes == 2 {
            return false;
        }
        if direction.y != 0.0 && axes == 1 {
            return false;
        }
        return true;
    }
    false
}

/// `Solid.OnDashCollide` for the vanilla Solids the runtime models. Returns
/// `None` when the hit Solid has no callback, which leaves
/// `Player.OnCollideH`/`OnCollideV` on their ordinary path.
///
/// Not modelled: `CrushBlock`'s own `AttackSequence` travel (240 px/s toward the
/// player, the `MoveHCheck`/`MoveVCheck` crush, the 60 px/s return leg and the
/// squish kill) and `DashBlock.Break`'s debris. A crusher therefore stays where
/// the room put it, and only its `OnDashCollide` decision and re-arm are
/// reproduced.
///
/// Takes `&mut Map` because `DashSwitch.OnDashed` moves the switch's own
/// collider and clears its `Collidable` flag (`DashSwitch.cs:203-204`) before
/// the rest of the same frame runs.
fn on_dash_collide(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    entity_index: usize,
    direction: Vec2,
) -> Option<DashCollision> {
    let kind = map.entities[entity_index].kind;
    match kind {
        EntityKind::CrushBlock => {
            let entity = &map.entities[entity_index];
            let index = kind_state_index(map, entity_index, EntityKind::CrushBlock)?;
            let state = *p.crush_blocks.get(index)?;
            // `CrushBlock.OnDashed` (`CrushBlock.cs:274-282`) passes
            // `-direction` to both `CanActivate` and `Attack`.
            let towards_player = Vec2::new(-direction.x, -direction.y);
            if crush_block_can_activate(entity, &state, towards_player) {
                let state = &mut p.crush_blocks[index];
                state.crush_dir = towards_player;
                state.can_activate = false;
                state.wind_up_timer = CRUSH_BLOCK_WIND_UP;
                return Some(DashCollision::Rebound);
            }
            Some(DashCollision::NormalCollision)
        }
        EntityKind::DashBlock => {
            // `DashBlock.OnDashed` (`DashBlock.cs:131-139`).
            let can_dash = map.entities[entity_index].direction.x != 0.0;
            if !can_dash && p.state != PlayerState::RedDash && p.state != PlayerState::SummitLaunch
            {
                return Some(DashCollision::NormalCollision);
            }
            let index = kind_state_index(map, entity_index, EntityKind::DashBlock)?;
            let state = p.dash_blocks.get_mut(index)?;
            if !state.broken {
                state.broken = true;
                // `DashBlock.Break` sets `Collidable = false` and `RemoveSelf`/
                // `RemoveAndFlagAsGone`; the `Removed` override then runs
                // `Celeste.Freeze(0.05f)` (`DashBlock.cs:80-84,114-122`).
                p.freeze_timer = 0.05;
            }
            Some(DashCollision::Rebound)
        }
        EntityKind::FloatySpaceBlock => {
            // `FloatySpaceBlock.OnDash` (`FloatySpaceBlock.cs:210-218`) always
            // returns `NormalOverride`, so the dash takes the ordinary stop. The
            // offset itself only starts for the group master and only while its
            // `dashEase` is still at or below 0.2 (`:212`); that test needs the
            // group state, which lives with the entity update, so hand the hit on.
            p.pending_floaty_dash = Some((entity_index, direction));
            Some(DashCollision::NormalOverride)
        }
        EntityKind::DashSwitch => {
            // `DashSwitch.OnDashed` (`DashSwitch.cs:195-229`): the press only
            // happens for `direction == pressDirection`, and it always reports
            // `NormalCollision` - the dash is neither rebound nor cancelled, the
            // player just stops against the button.
            if direction == map.entities[entity_index].direction {
                // `MoveTo(pressedTarget)` (`:203`) is an exact whole-pixel move,
                // `Collidable = false` (`:204`) and `Position -= pressDirection *
                // 2f` (`:205`) leave the collider six pixels along the press
                // direction - and unreachable, because the entity is no longer
                // collidable. Vanilla's own `Awake` restore (`:132-134`) puts a
                // persistent switch back at exactly that position with
                // `Collidable = false` too, so the parked position is the whole
                // observable effect (`park_entity` is how `ExitBlock`,
                // `InvisibleBarrier`, `CassetteBlock` and `FallingBlock` model
                // their own collidable flags).
                //
                // The gate fan-out that follows (`:208-221`) needs that pressed
                // position for `GetGate`'s nearest-gate search, so it is read off
                // the collider before the entity is parked.
                let entity = &map.entities[entity_index];
                let pressed_position = Vec2::new(
                    entity.bounds.x + entity.direction.x * 6.0,
                    entity.bounds.y + entity.direction.y * 6.0,
                );
                park_entity(&mut map.entities[entity_index]);
                dash_switch_open_gates(p, map, entity_index, pressed_position, true);
            }
            Some(DashCollision::NormalCollision)
        }
        _ => None,
    }
}

/// `Player.OnCollideH`'s and `Player.OnCollideV`'s `OnDashCollide` branch
/// (`Player.cs:3155-3177`, `3255-3281`).
///
/// `step_sign` is `data.Direction` on the blocked whole-pixel step and
/// `rect` is the collider position that was blocked. `None` means the branch
/// does not apply and the caller keeps its ordinary stop.
fn try_dash_collide(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    rect: Rect,
    horizontal: bool,
    step_sign: f32,
) -> Option<DashCollision> {
    if p.state == PlayerState::StarFly || p.state == PlayerState::DreamDash {
        return None;
    }
    if !dash_attacking(p) {
        // `Player.cs:3276-3280`: outside the dash the callback still runs for
        // state 10 and the caller returns without an ordinary stop.
        if !horizontal && p.state == PlayerState::SummitLaunch && !map.static_solid_at(rect) {
            if let Some(index) = first_solid_entity_at(map, rect) {
                let direction = Vec2::new(0.0, step_sign);
                on_dash_collide(p, map, index, direction);
                return Some(DashCollision::Ignore);
            }
        }
        return None;
    }
    let dash_sign = if horizontal {
        p.dash_dir.x.signum()
    } else {
        p.dash_dir.y.signum()
    };
    if step_sign != dash_sign {
        return None;
    }
    if map.static_solid_at(rect) {
        return None;
    }
    let index = first_solid_entity_at(map, rect)?;
    let direction = if horizontal {
        Vec2::new(step_sign, 0.0)
    } else {
        Vec2::new(0.0, step_sign)
    };
    let result = on_dash_collide(p, map, index, direction)?;
    // `Player.cs:3158-3165`: `NormalOverride` is remapped to `NormalCollision` first, so only a
    // plain `NormalCollision` becomes `Ignore` while state 5 is active.
    if result == DashCollision::NormalCollision && p.state == PlayerState::RedDash {
        return Some(DashCollision::Ignore);
    }
    Some(result)
}

/// `Player.Rebound(int direction = 0)` (`Player.cs:2784-2800`).
///
/// `Player.lowFrictionStopTimer` and `Player.gliderBoostTimer` do not exist in
/// this simulator's snapshot, so those two writes are the only ones the model
/// cannot reproduce; the remaining fields are exact. The final
/// `StateMachine.State = 0` goes through `Monocle/StateMachine.cs:36-62`, so it
/// only runs `DashEnd`/`RedDashEnd` and `NormalBegin` when the state actually
/// changes.
fn rebound(p: &mut PlayerSnapshot, direction: f32) {
    p.speed = Vec2::new(direction * 120.0, -120.0);
    p.var_jump_speed = p.speed.y;
    p.var_jump_timer = 0.15;
    p.auto_jump = true;
    p.auto_jump_timer = 0.0;
    p.dash_attack_timer = 0.0;
    p.wall_slide_timer = 1.2;
    p.wall_boost_timer = 0.0;
    p.launched = false;
    p.force_move_x_timer = 0.0;
    if p.state == PlayerState::Dash {
        p.demo_dashed = false;
    }
    if p.state != PlayerState::Normal {
        enter_normal(p);
    }
}

/// Per-entity runtime state for the two vanilla dash-collision Solids, plus
/// `DashBlock.Awake`'s remove-if-the-player-starts-inside rule
/// (`DashBlock.cs:74-77`).
///
/// Known limitation: a `permanent` `DashBlock` that was already broken earlier
/// in the session is not observable, because `Break` records it in
/// `Session.DoNotLoad` (`DashBlock.cs:116-118,125-129`) and the trace exports
/// only `Player` fields. Leaving permanent blocks non-collidable is *not* a fix:
/// measured on `trace-202-v3` it loses three `ok` segments
/// (`4-GoldenRidge|0|d-00|56604`, `6-Reflection|0|04|88986`,
/// `6-Reflection|0|04|319789`) for the one it recovers.
fn initialize_crush_and_dash_blocks(p: &mut PlayerSnapshot, map: &mut Map) {
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    let crush_blocks = map
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::CrushBlock)
        .count();
    p.crush_blocks.truncate(crush_blocks);
    while p.crush_blocks.len() < crush_blocks {
        p.crush_blocks.push(crate::CrushBlockSnapshot::default());
    }
    let dash_blocks = map
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::DashBlock)
        .count();
    p.dash_blocks.truncate(dash_blocks);
    while p.dash_blocks.len() < dash_blocks {
        p.dash_blocks.push(crate::DashBlockSnapshot::default());
    }
    let mut dash_index = 0usize;
    for entity in &mut map.entities {
        if entity.kind != EntityKind::DashBlock {
            continue;
        }
        let index = dash_index;
        dash_index += 1;
        if p.dash_blocks[index].broken || entity.bounds.intersects(player) {
            p.dash_blocks[index].broken = true;
            park_entity(entity);
        }
    }
}

/// One-frame-deferred consequences of the dash-collision path: a broken
/// `DashBlock` stops being collidable, and an activated `CrushBlock` re-arms
/// `CRUSH_BLOCK_WIND_UP` seconds later unless it is `chillout`.
fn advance_crush_and_dash_blocks(p: &mut PlayerSnapshot, map: &mut Map) {
    let frame_delta_time = p.frame_delta_time;
    let mut crush_index = 0usize;
    let mut dash_index = 0usize;
    for entity in &mut map.entities {
        match entity.kind {
            EntityKind::CrushBlock => {
                let index = crush_index;
                crush_index += 1;
                let state = &mut p.crush_blocks[index];
                if state.wind_up_timer > 0.0 {
                    state.wind_up_timer = (state.wind_up_timer - frame_delta_time).max(0.0);
                    // `AttackSequence`: `yield return 0.4f; if (!chillOut) {
                    // canActivate = true; }` (`CrushBlock.cs:423-427`).
                    if state.wind_up_timer <= 0.0 && entity.direction.y == 0.0 {
                        state.can_activate = true;
                    }
                }
            }
            EntityKind::DashBlock => {
                let index = dash_index;
                dash_index += 1;
                if p.dash_blocks[index].broken && solid_is_collidable(entity) {
                    park_entity(entity);
                }
            }
            _ => {}
        }
    }
}

fn advance_refills(p: &mut PlayerSnapshot, map: &mut Map) {
    let mut refill_index = 0usize;
    for entity in &mut map.entities {
        if entity.kind != EntityKind::Refill {
            continue;
        }
        let state = &mut p.refills[refill_index];
        refill_index += 1;
        if state.removed {
            continue;
        }
        // Refill.Update decrements respawnTimer, then Respawn restores
        // collidability before the same entity frame runs its PlayerCollider.
        if state.respawn_timer > 0.0 {
            state.respawn_timer = (state.respawn_timer - p.frame_delta_time).max(0.0);
            if state.respawn_timer <= 0.0 {
                state.collidable = true;
            }
        }
    }
}

fn lookout_entity_indices(map: &Map) -> Vec<usize> {
    map.entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::Lookout).then_some(index))
        .collect()
}

fn initialize_lookouts(p: &mut PlayerSnapshot, map: &Map) {
    let indices = lookout_entity_indices(map);
    p.lookouts.truncate(indices.len());
    for (lookout_index, entity_index) in indices.into_iter().enumerate() {
        if lookout_index == p.lookouts.len() {
            let bounds = map.entities[entity_index].bounds;
            let position = Vec2::new(bounds.x + 2.0, bounds.y + 4.0);
            p.lookouts.push(crate::LookoutSnapshot {
                position,
                cam: p.camera,
                cam_start: p.camera,
                ..crate::LookoutSnapshot::default()
            });
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct StaticMoverAttachment {
    platform_index: usize,
    offset: Vec2,
}

fn static_mover_rides_platform(entity: &crate::Entity, platform: &crate::Entity) -> bool {
    let entity_bounds = entity.bounds;
    let platform_bounds = platform.bounds;
    if entity.direction.y < 0.0 {
        entity_bounds.bottom() == platform_bounds.y
            && entity_bounds.x < platform_bounds.right()
            && entity_bounds.right() > platform_bounds.x
    } else if entity.direction.y > 0.0 {
        entity_bounds.y == platform_bounds.bottom()
            && entity_bounds.x < platform_bounds.right()
            && entity_bounds.right() > platform_bounds.x
    } else if entity.direction.x < 0.0 {
        entity_bounds.right() == platform_bounds.x
            && entity_bounds.y < platform_bounds.bottom()
            && entity_bounds.bottom() > platform_bounds.y
    } else if entity.direction.x > 0.0 {
        entity_bounds.x == platform_bounds.right()
            && entity_bounds.y < platform_bounds.bottom()
            && entity_bounds.bottom() > platform_bounds.y
    } else {
        false
    }
}

fn supports_static_mover_platform(kind: EntityKind) -> bool {
    matches!(
        kind,
        EntityKind::CassetteBlock
            | EntityKind::FallingBlock
            | EntityKind::MoveBlock
            | EntityKind::MovingSolid
            | EntityKind::ZipMover
    )
}

// Spikes.cs and Spring.cs install StaticMovers whose SolidChecker tests the
// entity one pixel toward its supporting platform. Solid.Awake claims each
// mover for the first compatible Platform, and Solid.MoveHExact/MoveVExact
// calls MoveStaticMovers with the exact integer displacement before actor
// push/carry resolution. Keep the source offset so segmented Simulator
// construction restores the same attached position.
fn initialize_static_mover_attachments(map: &Map) -> Vec<Option<StaticMoverAttachment>> {
    let mut attachments = vec![None; map.entities.len()];
    for (entity_index, entity) in map.entities.iter().enumerate() {
        if !matches!(entity.kind, EntityKind::Spikes | EntityKind::Spring) {
            continue;
        }
        let Some((platform_index, platform)) =
            map.entities.iter().enumerate().find(|(_, platform)| {
                supports_static_mover_platform(platform.kind)
                    && static_mover_rides_platform(entity, platform)
            })
        else {
            continue;
        };
        attachments[entity_index] = Some(StaticMoverAttachment {
            platform_index,
            offset: Vec2::new(
                entity.bounds.x - platform.bounds.x,
                entity.bounds.y - platform.bounds.y,
            ),
        });
    }
    attachments
}

fn sync_platform_static_movers(
    map: &mut Map,
    attachments: &[Option<StaticMoverAttachment>],
    platform_index: usize,
    enabled: bool,
) {
    let platform = map.entities[platform_index].bounds;
    for (entity_index, attachment) in attachments.iter().enumerate() {
        let Some(attachment) = attachment else {
            continue;
        };
        if attachment.platform_index != platform_index {
            continue;
        }
        let entity = &mut map.entities[entity_index];
        if enabled {
            entity.bounds.x = platform.x + attachment.offset.x;
            entity.bounds.y = platform.y + attachment.offset.y;
        } else {
            entity.bounds.x = -1_000_000.0;
            entity.bounds.y = -1_000_000.0;
        }
    }
}

fn sync_all_platform_static_movers(
    p: &PlayerSnapshot,
    map: &mut Map,
    attachments: &[Option<StaticMoverAttachment>],
) {
    let mut move_block_index = 0usize;
    let mut falling_block_index = 0usize;
    let mut cassette_block_index = 0usize;
    for platform_index in 0..map.entities.len() {
        let kind = map.entities[platform_index].kind;
        let enabled = match kind {
            EntityKind::MoveBlock => {
                let enabled = p
                    .move_blocks
                    .get(move_block_index)
                    .map_or(true, |state| state.static_movers_enabled);
                move_block_index += 1;
                enabled
            }
            EntityKind::FallingBlock => {
                let enabled = p
                    .falling_blocks
                    .get(falling_block_index)
                    .map_or(true, |state| !state.removed);
                falling_block_index += 1;
                enabled
            }
            EntityKind::CassetteBlock => {
                let enabled = p
                    .cassette_blocks
                    .get(cassette_block_index)
                    .map_or(true, |state| state.collidable);
                cassette_block_index += 1;
                enabled
            }
            EntityKind::MovingSolid | EntityKind::ZipMover => true,
            _ => continue,
        };
        sync_platform_static_movers(map, attachments, platform_index, enabled);
    }
}

fn initialize_bounce_blocks(p: &mut PlayerSnapshot, map: &mut Map) {
    let block_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::BounceBlock).then_some(index))
        .collect();
    p.bounce_blocks.truncate(block_indices.len());
    for (block_index, entity_index) in block_indices.into_iter().enumerate() {
        if block_index == p.bounce_blocks.len() {
            let block_bounds = map.entities[entity_index].bounds;
            let start = Vec2::new(block_bounds.x, block_bounds.y);
            let attached_spike_index =
                map.entities.iter().enumerate().find_map(|(index, spike)| {
                    if spike.kind != EntityKind::Spikes {
                        return None;
                    }
                    let attached = if spike.direction.y < 0.0 {
                        spike.bounds.bottom() == block_bounds.y
                            && spike.bounds.x < block_bounds.right()
                            && spike.bounds.right() > block_bounds.x
                    } else if spike.direction.y > 0.0 {
                        spike.bounds.y == block_bounds.bottom()
                            && spike.bounds.x < block_bounds.right()
                            && spike.bounds.right() > block_bounds.x
                    } else if spike.direction.x < 0.0 {
                        spike.bounds.right() == block_bounds.x
                            && spike.bounds.y < block_bounds.bottom()
                            && spike.bounds.bottom() > block_bounds.y
                    } else {
                        spike.bounds.x == block_bounds.right()
                            && spike.bounds.y < block_bounds.bottom()
                            && spike.bounds.bottom() > block_bounds.y
                    };
                    attached.then_some(index as u16)
                });
            let attached_spike_position = attached_spike_index
                .map(|index| {
                    let bounds = map.entities[index as usize].bounds;
                    Vec2::new(bounds.x, bounds.y)
                })
                .unwrap_or_default();
            p.bounce_blocks.push(crate::BounceBlockSnapshot {
                position: start,
                start,
                static_movers_enabled: true,
                attached_spike_index,
                attached_spike_position,
                ..crate::BounceBlockSnapshot::default()
            });
        }
        let state = &p.bounce_blocks[block_index];
        {
            let entity = &mut map.entities[entity_index];
            if state.phase == 4 {
                // Broken BounceBlocks remain in the scene but are non-collidable.
                // Runtime maps do not carry a separate Collidable bit, so park the
                // collision rectangle outside the room until the reform succeeds.
                entity.bounds.x = -1_000_000.0;
                entity.bounds.y = -1_000_000.0;
            } else {
                entity.bounds.x = state.position.x;
                entity.bounds.y = state.position.y;
            }
        }
        if let Some(spike_index) = state.attached_spike_index.map(usize::from) {
            let spike = &mut map.entities[spike_index];
            if state.static_movers_enabled {
                spike.bounds.x = state.attached_spike_position.x;
                spike.bounds.y = state.attached_spike_position.y;
            } else {
                spike.bounds.x = -1_000_000.0;
                spike.bounds.y = -1_000_000.0;
            }
        }
    }
}

fn initialize_move_blocks(p: &mut PlayerSnapshot, map: &mut Map) {
    let block_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::MoveBlock).then_some(index))
        .collect();
    p.move_blocks.truncate(block_indices.len());
    for (block_index, entity_index) in block_indices.into_iter().enumerate() {
        let entity = &mut map.entities[entity_index];
        if block_index == p.move_blocks.len() {
            let start = Vec2::new(entity.bounds.x, entity.bounds.y);
            let angle = entity.direction.y.atan2(entity.direction.x);
            p.move_blocks.push(crate::MoveBlockSnapshot {
                position: start,
                start,
                angle,
                crash_timer: 0.15,
                crash_reset_timer: 0.1,
                no_steer_timer: 0.2,
                visible: true,
                static_movers_enabled: true,
                ..crate::MoveBlockSnapshot::default()
            });
        }
        let state = &p.move_blocks[block_index];
        if state.phase == 4 {
            entity.bounds.x = -1_000_000.0;
            entity.bounds.y = -1_000_000.0;
        } else {
            entity.bounds.x = state.position.x;
            entity.bounds.y = state.position.y;
        }
    }
}

fn initialize_zip_movers(p: &mut PlayerSnapshot, map: &mut Map) {
    let zip_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::ZipMover).then_some(index))
        .collect();
    p.zip_movers.truncate(zip_indices.len());
    for (zip_index, entity_index) in zip_indices.into_iter().enumerate() {
        let entity = &mut map.entities[entity_index];
        if zip_index == p.zip_movers.len() {
            let start = Vec2::new(entity.bounds.x, entity.bounds.y);
            p.zip_movers.push(crate::ZipMoverSnapshot {
                position: start,
                start,
                ..crate::ZipMoverSnapshot::default()
            });
        }
        let state = &p.zip_movers[zip_index];
        entity.bounds.x = state.position.x;
        entity.bounds.y = state.position.y;
    }
}

fn position_moving_solids(map: &mut Map, time: f32) {
    for entity in &mut map.entities {
        if entity.kind == EntityKind::MovingSolid {
            entity.bounds.x += (entity.direction.x * time).round_ties_even();
            entity.bounds.y += (entity.direction.y * time).round_ties_even();
        }
    }
}

fn player_riding_jump_thru(p: &PlayerSnapshot, bounds: Rect) -> bool {
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    player.x < bounds.right()
        && player.right() > bounds.x
        && player.bottom() <= bounds.y + 1.0
        && player.bottom() + 1.0 > bounds.y
}

fn move_cloud_v(
    p: &mut PlayerSnapshot,
    entity: &mut crate::Entity,
    state: &mut crate::CloudSnapshot,
    amount: f32,
    lift_y: f32,
) {
    let riding = player_riding_jump_thru(p, entity.bounds);
    state.remainder_y += amount;
    let move_y = state.remainder_y.round_ties_even();
    state.remainder_y -= move_y;
    if move_y == 0.0 {
        return;
    }

    // JumpThru.MoveVExact checks an upward non-rider against the destination
    // before committing its own Y.  It only pushes an actor that enters the
    // platform on this exact step; actors already overlapping are deliberately
    // left alone.  The ordering is observable when a cloud rises past the
    // corner of a solid during a hyper.
    let previous_bounds = entity.bounds;
    if !riding && move_y < 0.0 {
        let player = current_player_rect(p, p.pos.x, p.pos.y);
        let destination = Rect::new(
            previous_bounds.x,
            previous_bounds.y + move_y,
            previous_bounds.width,
            previous_bounds.height,
        );
        if destination.intersects(player) && !previous_bounds.intersects(player) {
            p.pos.y += previous_bounds.y + move_y - player.bottom();
            set_lift_speed(p, Vec2::new(0.0, lift_y));
        }
    }

    entity.bounds.y += move_y;
    state.position.y += move_y;
    if riding {
        p.pos.y += move_y;
        set_lift_speed(p, Vec2::new(0.0, lift_y));
    }
}

fn advance_clouds(p: &mut PlayerSnapshot, map: &mut Map) {
    let cloud_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::Cloud).then_some(index))
        .collect();
    for (cloud_index, entity_index) in cloud_indices.into_iter().enumerate() {
        let mut state = p.clouds[cloud_index].clone();
        let entity = &mut map.entities[entity_index];
        match state.phase {
            0 => {
                if player_riding_jump_thru(p, entity.bounds) && p.speed.y >= 0.0 {
                    // Cloud.Update starts the depression with 180 px/s but
                    // does not enter the movement branch until the next frame.
                    state.speed = 180.0;
                    state.phase = 1;
                }
            }
            1 => {
                if state.position.y >= state.start.y {
                    state.speed -= 1200.0 * p.frame_delta_time;
                } else {
                    state.speed += 1200.0 * p.frame_delta_time;
                    if state.speed >= -100.0 {
                        if player_riding_jump_thru(p, entity.bounds) && p.speed.y >= 0.0 {
                            p.speed.y = -200.0;
                        }
                        state.phase = 2;
                    }
                }
                let lift_y = if state.speed < 0.0 {
                    -220.0
                } else {
                    state.speed
                };
                let speed = state.speed;
                move_cloud_v(p, entity, &mut state, speed * p.frame_delta_time, lift_y);
            }
            2 => {
                state.speed = approach(state.speed, 180.0, 600.0 * p.frame_delta_time);
                let exact_y = state.position.y + state.remainder_y;
                let desired_y = approach(exact_y, state.start.y, state.speed * p.frame_delta_time);
                let amount = desired_y - exact_y;
                // Platform.MoveTowardsY delegates to MoveV(amount), whose
                // LiftSpeed is the requested displacement divided by DeltaTime,
                // not Cloud.speed.  Its rounded one-pixel step is therefore
                // still reported as the exact fractional return velocity.
                let lift_y = amount / p.frame_delta_time;
                move_cloud_v(p, entity, &mut state, amount, lift_y);
                if state.position.y + state.remainder_y == state.start.y {
                    state.phase = 0;
                    state.speed = 0.0;
                }
            }
            _ => {
                state.phase = 0;
                state.speed = 0.0;
                state.position = state.start;
                state.remainder_y = 0.0;
                entity.bounds.x = state.start.x;
                entity.bounds.y = state.start.y;
            }
        }
        p.clouds[cloud_index] = state;
    }
}

fn seeker_physics_rect(position: Vec2) -> Rect {
    Rect::new(position.x - 3.0, position.y - 3.0, 6.0, 6.0)
}

fn seeker_attack_rect(position: Vec2) -> Rect {
    Rect::new(position.x - 6.0, position.y - 2.0, 12.0, 8.0)
}

fn seeker_bounce_rect(position: Vec2, state: u8, speed: Vec2) -> Rect {
    if state == 3 && speed.x > 0.0 {
        Rect::new(position.x - 10.0, position.y - 8.0, 16.0, 6.0)
    } else if state == 3 && speed.y < 0.0 {
        Rect::new(position.x - 6.0, position.y - 8.0, 16.0, 6.0)
    } else {
        Rect::new(position.x - 6.0, position.y - 8.0, 12.0, 6.0)
    }
}

fn seeker_collides(map: &Map, position: Vec2) -> bool {
    let collider = seeker_physics_rect(position);
    map.solid_at(collider)
        || collider.x < map.bounds.x
        || collider.right() > map.bounds.right()
        || collider.bottom() > map.bounds.bottom()
        || collider.y < map.bounds.y - 8.0
}

fn move_seeker_vertical_exact(seeker: &mut crate::SeekerSnapshot, map: &Map, amount: i32) -> bool {
    let sign = amount.signum();
    for _ in 0..amount.unsigned_abs() {
        let next = Vec2::new(seeker.position.x, seeker.position.y + sign as f32);
        if seeker_collides(map, next) {
            return false;
        }
        seeker.position = next;
    }
    true
}

fn move_seeker_axis(seeker: &mut crate::SeekerSnapshot, map: &Map, horizontal: bool) {
    let amount = if horizontal {
        seeker.speed.x * DT
    } else {
        seeker.speed.y * DT
    };
    let remainder = if horizontal {
        &mut seeker.remainder.x
    } else {
        &mut seeker.remainder.y
    };
    *remainder += amount;
    let pixels = remainder.round_ties_even() as i32;
    *remainder -= pixels as f32;
    let sign = pixels.signum();
    for _ in 0..pixels.unsigned_abs() {
        let next = Vec2::new(
            seeker.position.x + if horizontal { sign as f32 } else { 0.0 },
            seeker.position.y + if horizontal { 0.0 } else { sign as f32 },
        );
        if seeker_collides(map, next) {
            if horizontal {
                if seeker.state == 3 {
                    let original_y = seeker.position.y;
                    let corrected = [4, -4].into_iter().any(|offset| {
                        seeker.position.y = original_y;
                        !seeker_collides(map, Vec2::new(next.x, original_y + offset as f32))
                            && move_seeker_vertical_exact(seeker, map, offset)
                    });
                    if corrected {
                        seeker.position.x = next.x;
                        continue;
                    }
                    seeker.position.y = original_y;
                }
                if matches!(seeker.state, 3 | 5) && seeker.speed.x.abs() >= 100.0 {
                    seeker.speed.x = seeker.speed.x.signum() * -100.0;
                    seeker.speed.y *= 0.4;
                    seeker.state = 4;
                    seeker.state_timer = 0.8;
                } else {
                    seeker.speed.x *= -0.2;
                }
                seeker.remainder.x = 0.0;
            } else {
                seeker.speed.y *= if seeker.state == 3 { -0.6 } else { -0.2 };
                seeker.remainder.y = 0.0;
            }
            break;
        }
        seeker.position = next;
    }
}

fn seeker_player_center(p: &PlayerSnapshot) -> Vec2 {
    let collider = current_player_rect(p, p.pos.x, p.pos.y);
    Vec2::new(
        collider.x + collider.width * 0.5,
        collider.y + collider.height * 0.5,
    )
}

fn advance_seekers(p: &mut PlayerSnapshot, map: &mut Map) {
    let entity_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::Seeker).then_some(index))
        .collect();
    for (seeker_index, entity_index) in entity_indices.into_iter().enumerate() {
        let mut seeker = p.seekers[seeker_index].clone();
        if seeker.state == 4 {
            seeker.speed = approach_vec(seeker.speed, Vec2::default(), 150.0 * DT);
            if seeker.state_timer > 0.0 {
                seeker.state_timer -= DT;
            } else {
                seeker.state = 0;
                seeker.state_timer = 0.0;
            }
        }

        move_seeker_axis(&mut seeker, map, true);
        move_seeker_axis(&mut seeker, map, false);

        if !p.dead {
            let player = current_player_rect(p, p.pos.x, p.pos.y);
            let attack = seeker_attack_rect(seeker.position);
            if attack.intersects(player) {
                if seeker.state == 4 {
                    let player_center = seeker_player_center(p);
                    point_bounce(p, seeker.position);
                    seeker.speed = scale(
                        normalize(Vec2::new(
                            seeker.position.x - player_center.x,
                            seeker.position.y - player_center.y,
                        )),
                        100.0,
                    );
                } else {
                    p.dead = true;
                    p.speed = Vec2::default();
                    p.death_freeze_pending = true;
                    p.respawn_frames = 95;
                }
            } else if seeker_bounce_rect(seeker.position, seeker.state, seeker.speed)
                .intersects(player)
            {
                let player_center = seeker_player_center(p);
                bounce(p, map, seeker.position.y - 3.0);
                seeker.speed = scale(
                    normalize(Vec2::new(
                        seeker.position.x - player_center.x,
                        seeker.position.y - player_center.y,
                    )),
                    200.0,
                );
                seeker.state = 6;
                seeker.state_timer = 0.0;
                p.freeze_timer = 0.15;
            }
        }

        map.entities[entity_index].bounds =
            Rect::new(seeker.position.x - 6.0, seeker.position.y - 6.0, 12.0, 12.0);
        p.seekers[seeker_index] = seeker;
    }
}

fn solid_at_with_gate(map: &Map, rect: Rect, pusher_index: usize, pusher_collidable: bool) -> bool {
    map.static_solid_at(rect)
        || map.entities.iter().enumerate().any(|(index, entity)| {
            let solid = matches!(
                entity.kind,
                EntityKind::DreamBlock
                    | EntityKind::BounceBlock
                    | EntityKind::FallingBlock
                    | EntityKind::MoveBlock
                    | EntityKind::MovingSolid
                    | EntityKind::ZipMover
                    | EntityKind::TempleGate
                    | EntityKind::ExitBlock
                    | EntityKind::InvisibleBarrier
            );
            solid && (index != pusher_index || pusher_collidable) && entity.bounds.intersects(rect)
        })
}

fn jump_thru_blocks_actor(map: &Map, rect: Rect, previous_bottom: f32) -> bool {
    map.entities.iter().any(|entity| {
        matches!(entity.kind, EntityKind::JumpThru | EntityKind::Cloud)
            && previous_bottom <= entity.bounds.y
            && entity.bounds.intersects(rect)
    })
}

fn squish_wiggle_candidate<F>(current: Vec2, target: Vec2, mut collides: F) -> Option<Vec2>
where
    F: FnMut(Vec2) -> bool,
{
    for origin in [current, target] {
        for x in 0..=3 {
            for y in 0..=3 {
                if x == 0 && y == 0 {
                    continue;
                }
                for sign_x in [1.0, -1.0] {
                    for sign_y in [1.0, -1.0] {
                        let candidate =
                            Vec2::new(origin.x + x as f32 * sign_x, origin.y + y as f32 * sign_y);
                        if !collides(candidate) {
                            return Some(candidate);
                        }
                    }
                }
            }
        }
    }
    None
}

fn squish_player(p: &mut PlayerSnapshot, map: &Map, gate_index: usize, target: Vec2) {
    let ducked = !p.ducking;
    if ducked {
        p.ducking = true;
        if !solid_at_with_gate(map, duck_player_rect(p.pos.x, p.pos.y), gate_index, true) {
            return;
        }
        let was = p.pos;
        p.pos = target;
        if !solid_at_with_gate(map, duck_player_rect(p.pos.x, p.pos.y), gate_index, true) {
            return;
        }
        p.pos = was;
    }

    let state = p.state;
    let preserved = p.star_fly_hitbox_preserved;
    let ducking = p.ducking;
    let candidate = squish_wiggle_candidate(p.pos, target, |position| {
        let rect = if state == PlayerState::StarFly {
            star_fly_rect(position.x, position.y)
        } else if preserved {
            star_fly_hurt_rect(position.x, position.y)
        } else if ducking {
            duck_player_rect(position.x, position.y)
        } else {
            player_rect(position.x, position.y)
        };
        solid_at_with_gate(map, rect, gate_index, true)
    });
    if let Some(position) = candidate {
        p.pos = position;
        if ducked && !solid_at_with_gate(map, player_rect(p.pos.x, p.pos.y), gate_index, false) {
            p.ducking = false;
        }
    } else {
        p.dead = true;
        p.speed = Vec2::default();
        p.death_freeze_pending = true;
        p.respawn_frames = 95;
    }
}

fn move_player_v_from_gate(p: &mut PlayerSnapshot, map: &Map, gate_index: usize, amount: i32) {
    let target = Vec2::new(p.pos.x, p.pos.y + amount as f32);
    let sign = amount.signum();
    for _ in 0..amount.unsigned_abs() {
        let next = Vec2::new(p.pos.x, p.pos.y + sign as f32);
        let next_rect = current_player_rect(p, next.x, next.y);
        let blocked = solid_at_with_gate(map, next_rect, gate_index, false)
            || (sign > 0
                && jump_thru_blocks_actor(
                    map,
                    next_rect,
                    current_player_rect(p, p.pos.x, p.pos.y).bottom(),
                ));
        if blocked {
            p.movement_remainder.y = 0.0;
            squish_player(p, map, gate_index, target);
            return;
        }
        p.pos = next;
    }
}

fn move_theo_v_from_gate(
    p: &mut PlayerSnapshot,
    theo_index: usize,
    map: &Map,
    gate_index: usize,
    amount: i32,
) {
    let mut theo = p.theo_crystals[theo_index].clone();
    let target = Vec2::new(theo.position.x, theo.position.y + amount as f32);
    let sign = amount.signum();
    for _ in 0..amount.unsigned_abs() {
        let next = Vec2::new(theo.position.x, theo.position.y + sign as f32);
        let next_rect = theo_body_rect(next);
        let blocked = solid_at_with_gate(map, next_rect, gate_index, false)
            || (sign > 0
                && jump_thru_blocks_actor(map, next_rect, theo_body_rect(theo.position).bottom()));
        if blocked {
            theo.remainder.y = 0.0;
            if let Some(position) = squish_wiggle_candidate(theo.position, target, |position| {
                solid_at_with_gate(map, theo_body_rect(position), gate_index, true)
            }) {
                theo.position = position;
            } else {
                theo.dead = true;
                theo.held = false;
                p.holding_theo = None;
                p.dead = true;
                p.speed = Vec2::default();
                p.death_freeze_pending = true;
                p.respawn_frames = 95;
            }
            p.theo_crystals[theo_index] = theo;
            return;
        }
        theo.position = next;
    }
    p.theo_crystals[theo_index] = theo;
}

fn move_glider_v_from_gate(
    p: &mut PlayerSnapshot,
    glider_index: usize,
    map: &Map,
    gate_index: usize,
    amount: i32,
) {
    let mut glider = p.gliders[glider_index].clone();
    let target = Vec2::new(glider.position.x, glider.position.y + amount as f32);
    let sign = amount.signum();
    for _ in 0..amount.unsigned_abs() {
        let next = Vec2::new(glider.position.x, glider.position.y + sign as f32);
        let next_rect = glider_body_rect(next);
        let blocked = solid_at_with_gate(map, next_rect, gate_index, false)
            || (sign > 0
                && jump_thru_blocks_actor(
                    map,
                    next_rect,
                    glider_body_rect(glider.position).bottom(),
                ));
        if blocked {
            glider.remainder.y = 0.0;
            if let Some(position) = squish_wiggle_candidate(glider.position, target, |position| {
                solid_at_with_gate(map, glider_body_rect(position), gate_index, true)
            }) {
                glider.position = position;
            } else {
                glider.removed = true;
                glider.held = false;
                p.holding_glider = None;
            }
            p.gliders[glider_index] = glider;
            return;
        }
        glider.position = next;
    }
    p.gliders[glider_index] = glider;
}

/// `TempleGate.SwitchOpen` (`TempleGate.cs:124-132`): it only plays the `open` sprite and arms the
/// first 0.2 s `Alarm`; the collider stays up until the second one fires, which
/// `advance_temple_gate_alarms` owns.
fn start_temple_gate_switch_open(p: &mut PlayerSnapshot, snapshot_index: usize) {
    let gate = &mut p.temple_gates[snapshot_index];
    gate.alarm_stage = 1;
    gate.alarm_timer = TEMPLE_GATE_SWITCH_BEAT;
}

/// `TempleGate.StartOpen` (`TempleGate.cs:146-151`): `SetHeight(0); drawHeight = 4f; open = true`.
/// The collider half is `Open`'s, but there is no animation to run - `Awake` ends with
/// `drawHeight = Math.Max(4f, base.Height)` (`:111`), which the collapsed height makes 4, so
/// `drawHeight == max(4, Height)` and `lockState` is false straight away.
fn start_open_temple_gate(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    snapshot_index: usize,
    entity_index: usize,
) {
    open_temple_gate(p, map, snapshot_index, entity_index);
    let gate = &mut p.temple_gates[snapshot_index];
    gate.draw_speed = 0.0;
    gate.draw_height = 4.0;
    gate.lock_state = false;
}

/// The gate half of `DashSwitch`: `OnDashed`'s `SwitchOpen` fan-out (`DashSwitch.cs:208-221`) when
/// `alarm`, or `Awake`'s `StartOpen` one (`:135-148`) when not - the two differ only in which of
/// the gate's two entrances they call.
///
/// `allGates` (decoded into `shielded`) walks every `TempleGate` whose `Type` is `NearestSwitch`
/// and whose `LevelID` is the switch's `EntityID.Level` (`:139`, `:212`). `LevelID` is the level
/// the gate was loaded with (`Level.LoadLevel` passes `Session.Level`, `TempleGate.cs:72-74`), so
/// within one decoded room that comparison is always true. Otherwise `GetGate` (`:231-253`) picks
/// the nearest unclaimed `NearestSwitch` gate by `Vector2.DistanceSquared` from the switch's
/// position, ties going to the earlier entity, and marks it `ClaimedByASwitch` so no later switch
/// can take it.
fn dash_switch_open_gates(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    switch_index: usize,
    switch_position: Vec2,
    alarm: bool,
) {
    let gate_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::TempleGate).then_some(index))
        .collect();
    let mut chosen: Option<(usize, usize)> = None;
    if map.entities[switch_index].shielded {
        for (snapshot_index, entity_index) in gate_indices.into_iter().enumerate() {
            if p.temple_gates[snapshot_index].gate_type != TEMPLE_GATE_NEAREST_SWITCH {
                continue;
            }
            if alarm {
                start_temple_gate_switch_open(p, snapshot_index);
            } else {
                start_open_temple_gate(p, map, snapshot_index, entity_index);
            }
        }
        return;
    }
    let mut best = f32::INFINITY;
    for (snapshot_index, entity_index) in gate_indices.into_iter().enumerate() {
        let gate = &p.temple_gates[snapshot_index];
        if gate.gate_type != TEMPLE_GATE_NEAREST_SWITCH || gate.claimed {
            continue;
        }
        let dx = switch_position.x - gate.position.x;
        let dy = switch_position.y - gate.position.y;
        let distance = dx * dx + dy * dy;
        if chosen.is_none() || distance < best {
            best = distance;
            chosen = Some((snapshot_index, entity_index));
        }
    }
    let Some((snapshot_index, entity_index)) = chosen else {
        return;
    };
    p.temple_gates[snapshot_index].claimed = true;
    if alarm {
        start_temple_gate_switch_open(p, snapshot_index);
    } else {
        start_open_temple_gate(p, map, snapshot_index, entity_index);
    }
}

fn close_temple_gate(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    gate_index: usize,
    gate: &mut crate::TempleGateSnapshot,
) {
    // `TempleGate.Close` (`TempleGate.cs:153-163`): `holdingWaitTimer = 0.2f;
    // drawHeightMoveSpeed = 300f; drawHeight = Math.Max(4f, base.Height)` - that last one read
    // *before* `SetHeight(closedHeight)`, so a gate that was open animates 4 -> closedHeight at
    // 300 px/s and `drawHeight != max(4, Height)` keeps `lockState` true until it arrives (`:265-274`).
    gate.holding_wait = 0.2;
    gate.draw_speed = 300.0;
    gate.draw_height = gate.current_height.max(4.0);
    let old_height = gate.current_height as i32;
    let close_height = gate.closed_height as i32;
    let mut temporary_y = gate.position.y;
    let mut temporary_height = gate.current_height;
    if temporary_height < 64.0 {
        temporary_y -= 64.0 - temporary_height;
        temporary_height = 64.0;
    }
    let old_bottom = temporary_y + temporary_height;
    let move_y = close_height - old_height;
    temporary_y += move_y as f32;
    let moved_gate = Rect::new(
        gate.position.x,
        temporary_y,
        temple_gate_width(gate.gate_type),
        temporary_height,
    );
    map.entities[gate_index].bounds = moved_gate;

    let player_rect = current_player_rect(p, p.pos.x, p.pos.y);
    if moved_gate.intersects(player_rect) {
        let push = move_y - (player_rect.y - old_bottom) as i32;
        move_player_v_from_gate(p, map, gate_index, push);
    }
    for theo_index in 0..p.theo_crystals.len() {
        if p.theo_crystals[theo_index].dead {
            continue;
        }
        let body = theo_body_rect(p.theo_crystals[theo_index].position);
        if moved_gate.intersects(body) {
            let push = move_y - (body.y - old_bottom) as i32;
            move_theo_v_from_gate(p, theo_index, map, gate_index, push);
        }
    }
    for glider_index in 0..p.gliders.len() {
        if p.gliders[glider_index].removed {
            continue;
        }
        let body = glider_body_rect(p.gliders[glider_index].position);
        if moved_gate.intersects(body) {
            let push = move_y - (body.y - old_bottom) as i32;
            move_glider_v_from_gate(p, glider_index, map, gate_index, push);
        }
    }

    gate.current_height = gate.closed_height;
    gate.open = false;
    gate.triggered = true;
    map.entities[gate_index].bounds = Rect::new(
        gate.position.x,
        gate.position.y,
        temple_gate_width(gate.gate_type),
        gate.closed_height,
    );
}

/// The three ways a gate's collider comes back up, plus the `HoldingTheo` proximity toggle - i.e.
/// everything in `TempleGate`'s own `Update` that is not the close-behind coroutine.
///
/// The gate's `Update` runs *before* `Player.Update` (its depth is -9000, `TempleGate.cs:67`),
/// which is why this is called before the state callback while `advance_temple_gates` - whose
/// close check watches the player walk away many frames after the fact - is not.
/// `DashSwitch.OnDashed` calls `SwitchOpen` (`:220`) from inside that same `Player.Update`, so a
/// chain armed on frame N first ticks on frame N+1: `Monocle.Alarm.Update` subtracts
/// `Engine.DeltaTime` once per frame and fires on the tick that reaches zero (`Alarm.cs:57-80`),
/// and the second `Alarm` installed by the first one's `OnComplete` is added while
/// `ComponentList` is locked, so it starts ticking on the frame *after* the first fired
/// (`ComponentList.cs:199-210`). Two 0.2 s beats therefore collapse the collider 0.4 s after the
/// press.
///
/// `CheckTouchSwitches` (`TempleGate.cs:197-212`) waits for `Switch.Check(Scene)` - which is
/// `GetComponent<Switch>()?.Finished ?? false` (`Switch.cs:99-110`), false in a room with no
/// `Switch` at all - and then plays `open` for 0.5 s before the 0.2 s shake beat and `Open()`.
/// `TouchSwitch` is the only vanilla `Switch` carrier (`TouchSwitch.cs:104-110`) and it activates
/// from a `PlayerCollider`, i.e. after `Player.Update`, so the activations sampled here are
/// exactly the ones the gate's own pre-`Player` update observes.
fn advance_temple_gate_alarms(p: &mut PlayerSnapshot, map: &mut Map, room: &RoomCoroutineState) {
    let dt = p.frame_delta_time;
    let all_switches_on = room.switches_on
        || (!room.touch_switches.is_empty()
            && room.touch_switches.iter().all(|switch| switch.activated));
    let entity_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::TempleGate).then_some(index))
        .collect();
    for (snapshot_index, entity_index) in entity_indices.into_iter().enumerate() {
        let mut gate = p.temple_gates[snapshot_index].clone();
        if gate.gate_type == TEMPLE_GATE_TOUCH_SWITCHES && gate.alarm_stage == 0 {
            // `while (!Switch.Check(Scene)) yield return null;` (`TempleGate.cs:199-202`), then
            // `sprite.Play("open")` and the 0.5 s beat before the shake.
            if !all_switches_on {
                continue;
            }
            gate.alarm_stage = 3;
            gate.alarm_timer = TEMPLE_GATE_TOUCH_BEAT;
            p.temple_gates[snapshot_index] = gate;
            continue;
        }
        if gate.alarm_stage != 0 {
            // The two mechanisms count differently. `Monocle.Alarm.Update` (`Alarm.cs:57-80`)
            // subtracts and fires on the very tick the counter reaches zero; `Monocle.Coroutine`'s
            // float `yield` (`Coroutine.cs:35-82`) only resumes on the *next* frame, because the
            // frame that takes `waitTimer` to or below zero still takes the `waitTimer > 0f`
            // branch and returns without advancing. A 0.2 s alarm is therefore 12 frames while a
            // 0.5 s coroutine yield is 31.
            let coroutine_wait = matches!(gate.alarm_stage, 3 | 4);
            let fired = if coroutine_wait {
                // `Coroutine` only resumes once the counter is already at or below zero when the
                // frame starts; the frame that takes it there returns without advancing.
                if gate.alarm_timer > 0.0 {
                    gate.alarm_timer -= dt;
                    false
                } else {
                    true
                }
            } else {
                gate.alarm_timer -= dt;
                gate.alarm_timer <= 0.0
            };
            if !fired {
                p.temple_gates[snapshot_index] = gate;
                continue;
            }
            match gate.alarm_stage {
                // `SwitchOpen`'s first `Alarm.Set`: shake, then arm the second 0.2 s `Alarm`
                // (`TempleGate.cs:127-131`).
                1 => {
                    gate.alarm_stage = 2;
                    gate.alarm_timer = TEMPLE_GATE_SWITCH_BEAT;
                }
                // `CheckTouchSwitches`' shake beat, then `Open()`.
                3 => {
                    gate.alarm_stage = 4;
                    gate.alarm_timer = TEMPLE_GATE_SWITCH_BEAT;
                }
                // `SwitchOpen`'s second `Alarm.Set(..., Open)` (`:130`) and the end of
                // `CheckTouchSwitches` (`:211`).
                _ => {
                    gate.alarm_stage = 0;
                    gate.alarm_timer = 0.0;
                    // `Open` sets `drawHeightMoveSpeed = 200f; drawHeight = base.Height` before
                    // `SetHeight(0)` (`TempleGate.cs:138-141`), and the animation tail below runs
                    // in the same `Update`: `base.Update()` fires the alarm first (`:243-245`).
                    gate.draw_speed = 200.0;
                    gate.draw_height = gate.current_height;
                    p.temple_gates[snapshot_index] = gate;
                    open_temple_gate(p, map, snapshot_index, entity_index);
                    gate = p.temple_gates[snapshot_index].clone();
                }
            }
        }
        // `if (Type == Types.HoldingTheo)` (`TempleGate.cs:246-264`): the gate follows Theo, not
        // the player. A room with no live `TheoCrystal` counts as "nearby" (`:214-222` returns
        // true), which is what keeps the vanilla `HoldingTheo` gate with no Theo in its room open.
        if gate.gate_type == TEMPLE_GATE_HOLDING_THEO {
            if gate.holding_wait > 0.0 {
                gate.holding_wait -= dt;
            } else if !gate.lock_state {
                let nearby = theo_is_nearby(p, map, gate.position, gate.closed_height, gate.open);
                if gate.open && !nearby {
                    // `close_temple_gate` carries `Close`'s own `holdingWaitTimer`/animation fields.
                    close_temple_gate(p, map, entity_index, &mut gate);
                } else if !gate.open && nearby {
                    gate.holding_wait = 0.2;
                    gate.draw_speed = 200.0;
                    gate.draw_height = gate.current_height;
                    p.temple_gates[snapshot_index] = gate;
                    open_temple_gate(p, map, snapshot_index, entity_index);
                    gate = p.temple_gates[snapshot_index].clone();
                }
            }
        }
        // `float num = Math.Max(4f, base.Height); if (drawHeight != num) { lockState = true;
        // drawHeight = Calc.Approach(drawHeight, num, drawHeightMoveSpeed * Engine.DeltaTime); }
        // else lockState = false;` (`TempleGate.cs:265-274`). `Awake` leaves `drawHeight` at
        // `Math.Max(4f, Height)` (`:111`), so a gate starts unlocked.
        let num = gate.current_height.max(4.0);
        if gate.draw_height != num {
            gate.lock_state = true;
            gate.draw_height = approach(gate.draw_height, num, gate.draw_speed * dt);
        } else {
            gate.lock_state = false;
        }
        p.temple_gates[snapshot_index] = gate;
    }
}

/// The close-behind coroutines (`TempleGate.cs:165-195`). Only the three `CloseBehindPlayer*`
/// types have one: a `NearestSwitch` gate a dash switch opened stays open, and a `TouchSwitches`
/// gate a touch switch opened likewise. `HoldingTheo` closes on Theo's proximity instead, which
/// `advance_temple_gate_alarms` owns.
fn advance_temple_gates(p: &mut PlayerSnapshot, map: &mut Map) {
    let entity_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::TempleGate).then_some(index))
        .collect();
    for (snapshot_index, entity_index) in entity_indices.into_iter().enumerate() {
        let mut gate = p.temple_gates[snapshot_index].clone();
        if !gate.open || gate.triggered {
            continue;
        }
        if !matches!(
            gate.gate_type,
            TEMPLE_GATE_CLOSE_BEHIND_PLAYER
                | TEMPLE_GATE_CLOSE_BEHIND_PLAYER_ALWAYS
                | TEMPLE_GATE_CLOSE_BEHIND_PLAYER_AND_THEO
        ) {
            continue;
        }
        let player_left = current_player_rect(p, p.pos.x, p.pos.y).x;
        if player_left <= gate.position.x + 12.0 {
            continue;
        }
        if gate.gate_type == TEMPLE_GATE_CLOSE_BEHIND_PLAYER_AND_THEO
            && !theo_passed_gate(p, map, gate.position.x + 12.0)
        {
            continue;
        }
        close_temple_gate(p, map, entity_index, &mut gate);
        p.temple_gates[snapshot_index] = gate;
    }
}

fn set_lift_speed(p: &mut PlayerSnapshot, speed: Vec2) {
    p.current_lift_speed = speed;
    if speed != Vec2::default() {
        p.last_lift_speed = speed;
        p.lift_speed_timer = 0.16;
    }
}

fn sine_in(value: f32) -> f32 {
    1.0 - (std::f32::consts::FRAC_PI_2 * value).cos()
}

fn player_riding_solid(p: &PlayerSnapshot, bounds: Rect) -> bool {
    match p.state {
        PlayerState::Attract => false,
        PlayerState::DreamDash => bounds.intersects(current_player_rect(p, p.pos.x, p.pos.y)),
        PlayerState::Climb | PlayerState::HitSquash => {
            let facing = if p.facing { 1.0 } else { -1.0 };
            bounds.intersects(current_player_rect(p, p.pos.x + facing, p.pos.y))
        }
        _ => bounds.intersects(current_player_rect(p, p.pos.x, p.pos.y + 1.0)),
    }
}

fn player_on_top_of_solid(p: &PlayerSnapshot, bounds: Rect) -> bool {
    Rect::new(bounds.x, bounds.y - 1.0, bounds.width, bounds.height)
        .intersects(current_player_rect(p, p.pos.x, p.pos.y))
}

fn player_climbing_solid(p: &PlayerSnapshot, bounds: Rect) -> bool {
    if p.state != PlayerState::Climb {
        return false;
    }
    let offset_x = if p.facing { -1.0 } else { 1.0 };
    Rect::new(bounds.x + offset_x, bounds.y, bounds.width, bounds.height)
        .intersects(current_player_rect(p, p.pos.x, p.pos.y))
}

fn initialize_heart_gems(p: &mut PlayerSnapshot, map: &mut Map) {
    let count = map
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::HeartGem)
        .count();
    p.heart_gems.truncate(count);
    while p.heart_gems.len() < count {
        p.heart_gems.push(crate::HeartGemSnapshot::default());
    }
    if !p.time_rate.is_finite() || p.time_rate <= 0.0 {
        p.time_rate = 1.0;
    }
}

fn initialize_camera(p: &mut PlayerSnapshot, map: &Map) {
    if p.camera_initialized {
        return;
    }
    p.camera = camera_target(p, map);
    p.camera_initialized = true;
}

fn camera_target(p: &PlayerSnapshot, map: &Map) -> Vec2 {
    let bounds = p.current_room_bounds.unwrap_or(map.bounds);
    Vec2::new(
        (p.pos.x - 160.0).clamp(bounds.x, (bounds.right() - 320.0).max(bounds.x)),
        (p.pos.y - 90.0).clamp(bounds.y, (bounds.bottom() - 180.0).max(bounds.y)),
    )
}

fn initialize_rising_lavas(p: &mut PlayerSnapshot, map: &mut Map) {
    let indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::RisingLava).then_some(index))
        .collect();
    p.rising_lavas.truncate(indices.len());
    for (lava_index, entity_index) in indices.into_iter().enumerate() {
        if lava_index == p.rising_lavas.len() {
            let intro = map.entities[entity_index].single_use;
            p.rising_lavas.push(crate::RisingLavaSnapshot {
                position: Vec2::new(map.bounds.x - 10.0, map.bounds.bottom() + 16.0),
                waiting: intro || p.just_respawned,
                ice_mode: p.core_mode == crate::CoreMode::Cold,
                intro,
                initialized: true,
                ..crate::RisingLavaSnapshot::default()
            });
        }
        let state = &p.rising_lavas[lava_index];
        let entity = &mut map.entities[entity_index];
        entity.bounds = Rect::new(state.position.x, state.position.y, 340.0, 120.0);
    }
}

fn initialize_sandwich_lavas(p: &mut PlayerSnapshot, map: &mut Map) {
    let indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::SandwichLava).then_some(index))
        .collect();
    p.sandwich_lavas.truncate(indices.len());
    for (lava_index, entity_index) in indices.iter().copied().enumerate() {
        if lava_index == p.sandwich_lavas.len() {
            let start_x = map.entities[entity_index].bounds.x;
            let respawn_intro = p.just_respawned;
            p.sandwich_lavas.push(crate::SandwichLavaSnapshot {
                position: Vec2::new(map.bounds.x - 10.0, map.bounds.bottom() - 10.0),
                start_x,
                waiting: respawn_intro || p.pos.x < start_x,
                ice_mode: p.core_mode == crate::CoreMode::Cold,
                persistent: lava_index == 0,
                top_rect_y: if respawn_intro { -360.0 } else { -420.0 },
                bottom_rect_y: if respawn_intro { 0.0 } else { 60.0 },
                initialized: true,
                ..crate::SandwichLavaSnapshot::default()
            });
        }
    }
    // Awake reuses the first persistent instance when the destination room
    // contains another SandwichLava, updating its activation X and parking
    // the duplicate rather than creating a second hazard.
    if p.sandwich_lavas.len() > 1 {
        for index in 1..p.sandwich_lavas.len() {
            if !p.sandwich_lavas[index].removed && !p.sandwich_lavas[0].leaving {
                p.sandwich_lavas[0].start_x = p.sandwich_lavas[index].start_x;
                p.sandwich_lavas[0].waiting = true;
                p.sandwich_lavas[index].removed = true;
            }
        }
    }
    for (lava_index, entity_index) in indices.into_iter().enumerate() {
        let state = &p.sandwich_lavas[lava_index];
        let entity = &mut map.entities[entity_index];
        entity.bounds = if state.removed {
            Rect::new(-1_000_000.0, -1_000_000.0, 340.0, 120.0)
        } else {
            Rect::new(state.position.x, state.position.y, 340.0, 120.0)
        };
    }
}

#[derive(Clone, Debug, Default)]
struct SolidCollisionEnv {
    solids: Vec<Rect>,
    jump_thrus: Vec<Rect>,
}

fn solid_collision_env(map: &Map, pusher_index: usize) -> SolidCollisionEnv {
    let mut solids = map.solids.clone();
    let mut jump_thrus = Vec::new();
    for (index, entity) in map.entities.iter().enumerate() {
        if index == pusher_index {
            continue;
        }
        if matches!(
            entity.kind,
            EntityKind::BounceBlock
                | EntityKind::CrushBlock
                | EntityKind::DashBlock
                | EntityKind::DreamBlock
                | EntityKind::FallingBlock
                | EntityKind::MoveBlock
                | EntityKind::MovingSolid
                | EntityKind::ZipMover
                | EntityKind::ExitBlock
                | EntityKind::InvisibleBarrier
        ) {
            solids.push(entity.bounds);
        } else if matches!(entity.kind, EntityKind::JumpThru | EntityKind::Cloud) {
            jump_thrus.push(entity.bounds);
        }
    }
    SolidCollisionEnv { solids, jump_thrus }
}

fn env_solid_at(env: &SolidCollisionEnv, rect: Rect, pusher: Option<Rect>) -> bool {
    env.solids.iter().any(|solid| solid.intersects(rect))
        || pusher.is_some_and(|bounds| bounds.intersects(rect))
}

fn env_jump_thru_at(env: &SolidCollisionEnv, rect: Rect, previous_bottom: f32) -> bool {
    env.jump_thrus
        .iter()
        .any(|jump_thru| previous_bottom <= jump_thru.y && jump_thru.intersects(rect))
}

fn player_on_squish(p: &mut PlayerSnapshot, env: &SolidCollisionEnv, pusher: Rect, target: Vec2) {
    let mut ducked = false;
    if !p.ducking {
        ducked = true;
        p.ducking = true;
        if !env_solid_at(env, current_player_rect(p, p.pos.x, p.pos.y), Some(pusher)) {
            return;
        }

        let was = p.pos;
        p.pos = target;
        if !env_solid_at(env, current_player_rect(p, p.pos.x, p.pos.y), Some(pusher)) {
            return;
        }
        p.pos = was;
    }

    for origin in [p.pos, target] {
        for x in 0..=3 {
            for y in 0..=3 {
                if x == 0 && y == 0 {
                    continue;
                }
                for x_sign in [1.0, -1.0] {
                    for y_sign in [1.0, -1.0] {
                        let candidate =
                            Vec2::new(origin.x + x as f32 * x_sign, origin.y + y as f32 * y_sign);
                        // Player.OnSquish re-enables the pusher only for the
                        // two initial Solid checks, then sets
                        // data.Pusher.Collidable = false before calling
                        // TrySquishWiggle. The wiggle itself must therefore
                        // search against the room solids alone.
                        if !env_solid_at(
                            env,
                            current_player_rect(p, candidate.x, candidate.y),
                            None,
                        ) {
                            p.pos = candidate;
                            if ducked && !env_solid_at(env, player_rect(p.pos.x, p.pos.y), None) {
                                p.ducking = false;
                            }
                            return;
                        }
                    }
                }
            }
        }
    }

    p.dead = true;
    p.speed = Vec2::default();
    p.death_freeze_pending = true;
    p.respawn_frames = 95;
}

fn move_player_exact_from_pusher(
    p: &mut PlayerSnapshot,
    env: &SolidCollisionEnv,
    pusher: Rect,
    horizontal: bool,
    amount: f32,
) {
    let move_by = amount as i32;
    if move_by == 0 {
        return;
    }
    let target = if horizontal {
        Vec2::new(p.pos.x + move_by as f32, p.pos.y)
    } else {
        Vec2::new(p.pos.x, p.pos.y + move_by as f32)
    };
    let sign = move_by.signum() as f32;
    for _ in 0..move_by.unsigned_abs() {
        let previous = current_player_rect(p, p.pos.x, p.pos.y);
        let candidate = if horizontal {
            current_player_rect(p, p.pos.x + sign, p.pos.y)
        } else {
            current_player_rect(p, p.pos.x, p.pos.y + sign)
        };
        let blocked = env_solid_at(env, candidate, None)
            || (!horizontal && sign > 0.0 && env_jump_thru_at(env, candidate, previous.bottom()));
        if blocked {
            player_on_squish(p, env, pusher, target);
            return;
        }
        if horizontal {
            p.pos.x += sign;
        } else {
            p.pos.y += sign;
        }
    }
}

fn move_riding_player_exact(
    p: &mut PlayerSnapshot,
    env: &SolidCollisionEnv,
    horizontal: bool,
    amount: f32,
) {
    let move_by = amount as i32;
    if move_by == 0 {
        return;
    }
    let sign = move_by.signum() as f32;
    for _ in 0..move_by.unsigned_abs() {
        let previous = current_player_rect(p, p.pos.x, p.pos.y);
        let candidate = if horizontal {
            current_player_rect(p, p.pos.x + sign, p.pos.y)
        } else {
            current_player_rect(p, p.pos.x, p.pos.y + sign)
        };
        let blocked = env_solid_at(env, candidate, None)
            || (!horizontal && sign > 0.0 && env_jump_thru_at(env, candidate, previous.bottom()));
        if blocked {
            // Solid.MoveHExact/MoveVExact first carries riders through the
            // Actor's exact movement path.  A rider stopped by an unrelated
            // wall stays at the last legal pixel; only actors overlapping
            // the newly moved Solid take the later OnSquish path.
            return;
        }
        if horizontal {
            p.pos.x += sign;
        } else {
            p.pos.y += sign;
        }
    }
}

fn move_runtime_solid_exact(
    p: &mut PlayerSnapshot,
    bounds: &mut Rect,
    env: &SolidCollisionEnv,
    horizontal: bool,
    amount: f32,
    lift_speed: Vec2,
) {
    if amount == 0.0 {
        return;
    }
    let riding = player_riding_solid(p, *bounds);
    let old = *bounds;
    if horizontal {
        bounds.x += amount;
    } else {
        bounds.y += amount;
    }

    let player = current_player_rect(p, p.pos.x, p.pos.y);
    if bounds.intersects(player) {
        let push = if horizontal {
            if amount > 0.0 {
                amount - (player.x - old.right())
            } else {
                amount - (player.right() - old.x)
            }
        } else if amount > 0.0 {
            amount - (player.y - old.bottom())
        } else {
            amount - (player.bottom() - old.y)
        };
        move_player_exact_from_pusher(p, env, *bounds, horizontal, push);
        set_lift_speed(p, lift_speed);
    } else if riding {
        move_riding_player_exact(p, env, horizontal, amount);
        set_lift_speed(p, lift_speed);
    }
}

fn move_zip_mover_to(
    p: &mut PlayerSnapshot,
    entity: &mut crate::Entity,
    state: &mut crate::ZipMoverSnapshot,
    env: &SolidCollisionEnv,
    target: Vec2,
) {
    // Platform.Update clears LiftSpeed before the entity calls MoveTo.
    state.lift_speed = Vec2::default();
    let exact_x = state.position.x + state.remainder.x;
    let move_x = target.x - exact_x;
    state.lift_speed.x = move_x / p.frame_delta_time;
    state.remainder.x += move_x;
    let exact_move_x = state.remainder.x.round_ties_even();
    state.remainder.x -= exact_move_x;
    move_runtime_solid_exact(
        p,
        &mut entity.bounds,
        env,
        true,
        exact_move_x,
        state.lift_speed,
    );
    state.position.x += exact_move_x;

    let exact_y = state.position.y + state.remainder.y;
    let move_y = target.y - exact_y;
    state.lift_speed.y = move_y / p.frame_delta_time;
    state.remainder.y += move_y;
    let exact_move_y = state.remainder.y.round_ties_even();
    state.remainder.y -= exact_move_y;
    move_runtime_solid_exact(
        p,
        &mut entity.bounds,
        env,
        false,
        exact_move_y,
        state.lift_speed,
    );
    state.position.y += exact_move_y;
}

fn vector_length(value: Vec2) -> f32 {
    (value.x * value.x + value.y * value.y).sqrt()
}

fn safe_normalize(value: Vec2, length: f32) -> Vec2 {
    let magnitude = vector_length(value);
    if magnitude == 0.0 {
        Vec2::default()
    } else {
        Vec2::new(value.x / magnitude * length, value.y / magnitude * length)
    }
}

fn approach_vec(value: Vec2, target: Vec2, max_move: f32) -> Vec2 {
    if max_move == 0.0 || value == target {
        return value;
    }
    let delta = Vec2::new(target.x - value.x, target.y - value.y);
    if vector_length(delta) < max_move {
        target
    } else {
        let move_by = safe_normalize(delta, max_move);
        Vec2::new(value.x + move_by.x, value.y + move_by.y)
    }
}

fn bounce_block_player_check(p: &PlayerSnapshot, bounds: Rect) -> bool {
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    let above = Rect::new(bounds.x, bounds.y - 1.0, bounds.width, bounds.height);
    if above.intersects(player) && p.speed.y >= 0.0 {
        return true;
    }
    let right = Rect::new(bounds.x + 1.0, bounds.y, bounds.width, bounds.height);
    if right.intersects(player) && p.state == PlayerState::Climb && !p.facing {
        return true;
    }
    let left = Rect::new(bounds.x - 1.0, bounds.y, bounds.width, bounds.height);
    left.intersects(player) && p.state == PlayerState::Climb && p.facing
}

fn move_bounce_block_to(
    p: &mut PlayerSnapshot,
    entity: &mut crate::Entity,
    state: &mut crate::BounceBlockSnapshot,
    env: &SolidCollisionEnv,
    target: Vec2,
    lift_speed: Vec2,
) {
    // Platform.Update clears LiftSpeed before BounceBlock.Update calls MoveTo.
    state.lift_speed = Vec2::default();
    let exact_x = state.position.x + state.remainder.x;
    state.lift_speed.x = lift_speed.x;
    state.remainder.x += target.x - exact_x;
    let move_x = state.remainder.x.round_ties_even();
    state.remainder.x -= move_x;
    move_runtime_solid_exact(p, &mut entity.bounds, env, true, move_x, state.lift_speed);
    state.position.x += move_x;

    let exact_y = state.position.y + state.remainder.y;
    state.lift_speed.y = lift_speed.y;
    state.remainder.y += target.y - exact_y;
    let move_y = state.remainder.y.round_ties_even();
    state.remainder.y -= move_y;
    move_runtime_solid_exact(p, &mut entity.bounds, env, false, move_y, state.lift_speed);
    state.position.y += move_y;
}

fn bounce_block_exact_position(state: &crate::BounceBlockSnapshot) -> Vec2 {
    Vec2::new(
        state.position.x + state.remainder.x,
        state.position.y + state.remainder.y,
    )
}

fn advance_bounce_blocks(p: &mut PlayerSnapshot, map: &mut Map) {
    let block_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::BounceBlock).then_some(index))
        .collect();
    for (block_index, entity_index) in block_indices.into_iter().enumerate() {
        let mut state = p.bounce_blocks[block_index].clone();
        let old_position = state.position;
        let mut started_reform_alarm = false;
        let reform_blocked = if state.phase == 4 && state.respawn_timer <= 0.0 {
            let source_bounds = map.entities[entity_index].bounds;
            let restored = Rect::new(
                state.start.x,
                state.start.y,
                source_bounds.width,
                source_bounds.height,
            );
            Some(
                restored.intersects(current_player_rect(p, p.pos.x, p.pos.y))
                    || map.static_solid_at(restored)
                    || map.entities.iter().enumerate().any(|(other_index, other)| {
                        other_index != entity_index
                            && matches!(
                                other.kind,
                                EntityKind::BounceBlock
                                    | EntityKind::DreamBlock
                                    | EntityKind::MoveBlock
                                    | EntityKind::MovingSolid
                                    | EntityKind::ZipMover
                                    | EntityKind::ExitBlock
                                    | EntityKind::InvisibleBarrier
                            )
                            && other.bounds.intersects(restored)
                    }),
            )
        } else {
            None
        };
        let env = solid_collision_env(map, entity_index);
        let entity = &mut map.entities[entity_index];
        match state.phase {
            0 => {
                state.move_speed = approach(state.move_speed, 100.0, 400.0 * p.frame_delta_time);
                let exact = bounce_block_exact_position(&state);
                let desired =
                    approach_vec(exact, state.start, state.move_speed * p.frame_delta_time);
                let delta = Vec2::new(desired.x - exact.x, desired.y - exact.y);
                let mut lift = safe_normalize(delta, state.move_speed);
                lift.x *= 0.75;
                move_bounce_block_to(p, entity, &mut state, &env, desired, lift);
                if bounce_block_player_check(p, entity.bounds) {
                    state.move_speed = 80.0;
                    let player_center = Vec2::new(
                        p.pos.x,
                        current_player_rect(p, p.pos.x, p.pos.y).y
                            + current_player_rect(p, p.pos.x, p.pos.y).height * 0.5,
                    );
                    let block_center = Vec2::new(
                        entity.bounds.x + entity.bounds.width * 0.5,
                        entity.bounds.y + entity.bounds.height * 0.5,
                    );
                    state.bounce_dir = safe_normalize(
                        Vec2::new(
                            player_center.x - block_center.x,
                            player_center.y - block_center.y,
                        ),
                        1.0,
                    );
                    state.phase = 1;
                }
            }
            1 => {
                if bounce_block_player_check(p, entity.bounds) {
                    let player_rect = current_player_rect(p, p.pos.x, p.pos.y);
                    let player_center = Vec2::new(
                        player_rect.x + player_rect.width * 0.5,
                        player_rect.y + player_rect.height * 0.5,
                    );
                    let block_center = Vec2::new(
                        entity.bounds.x + entity.bounds.width * 0.5,
                        entity.bounds.y + entity.bounds.height * 0.5,
                    );
                    state.bounce_dir = safe_normalize(
                        Vec2::new(
                            player_center.x - block_center.x,
                            player_center.y - block_center.y,
                        ),
                        1.0,
                    );
                }
                state.move_speed = approach(state.move_speed, 40.0, 600.0 * p.frame_delta_time);
                let target = Vec2::new(
                    state.start.x - state.bounce_dir.x * 10.0,
                    state.start.y - state.bounce_dir.y * 10.0,
                );
                let exact = bounce_block_exact_position(&state);
                let desired = approach_vec(exact, target, state.move_speed * p.frame_delta_time);
                let delta = Vec2::new(desired.x - exact.x, desired.y - exact.y);
                let mut lift = safe_normalize(delta, state.move_speed);
                lift.x *= 0.75;
                move_bounce_block_to(p, entity, &mut state, &env, desired, lift);
                let remaining = Vec2::new(
                    bounce_block_exact_position(&state).x - target.x,
                    bounce_block_exact_position(&state).y - target.y,
                );
                if remaining.x * remaining.x + remaining.y * remaining.y <= 2.0 {
                    state.phase = 2;
                    state.move_speed = 0.0;
                }
            }
            2 => {
                state.move_speed = approach(state.move_speed, 140.0, 800.0 * p.frame_delta_time);
                let target = Vec2::new(
                    state.start.x + state.bounce_dir.x * 24.0,
                    state.start.y + state.bounce_dir.y * 24.0,
                );
                let exact = bounce_block_exact_position(&state);
                let desired = approach_vec(exact, target, state.move_speed * p.frame_delta_time);
                let delta = Vec2::new(desired.x - exact.x, desired.y - exact.y);
                state.bounce_lift = safe_normalize(delta, (state.move_speed * 3.0).min(200.0));
                state.bounce_lift.x *= 0.75;
                let lift = state.bounce_lift;
                move_bounce_block_to(p, entity, &mut state, &env, desired, lift);
                if bounce_block_exact_position(&state) == target
                    || !bounce_block_player_check(p, entity.bounds)
                {
                    state.phase = 3;
                    state.move_speed = 0.0;
                    state.bounce_end_timer = 0.05;
                    if bounce_block_player_check(p, entity.bounds) {
                        p.state = PlayerState::Normal;
                        p.speed = state.bounce_lift;
                        p.jump_grace_timer = JUMP_GRACE;
                    }
                }
            }
            3 => {
                state.bounce_end_timer -= p.frame_delta_time;
                if state.bounce_end_timer <= 0.0 {
                    state.phase = 4;
                    state.respawn_timer = 1.6;
                    state.reform_timer = 0.0;
                    state.static_movers_enabled = false;
                    entity.bounds.x = -1_000_000.0;
                    entity.bounds.y = -1_000_000.0;
                }
            }
            4 => {
                if state.respawn_timer > 0.0 {
                    state.respawn_timer -= p.frame_delta_time;
                } else if reform_blocked == Some(false) {
                    entity.bounds.x = state.start.x;
                    entity.bounds.y = state.start.y;
                    state.position = state.start;
                    state.remainder = Vec2::default();
                    state.lift_speed = Vec2::default();
                    state.phase = 0;
                    state.reform_timer = 0.35;
                    state.static_movers_enabled = false;
                    started_reform_alarm = true;
                }
            }
            _ => {
                state.phase = 0;
                state.move_speed = 0.0;
                state.position = state.start;
                state.remainder = Vec2::default();
                state.reform_timer = 0.0;
                state.static_movers_enabled = true;
                entity.bounds.x = state.start.x;
                entity.bounds.y = state.start.y;
            }
        }
        let block_delta = Vec2::new(
            state.position.x - old_position.x,
            state.position.y - old_position.y,
        );
        if state.attached_spike_index.is_some() {
            state.attached_spike_position.x += block_delta.x;
            state.attached_spike_position.y += block_delta.y;
        }
        if state.reform_timer > 0.0 && !started_reform_alarm {
            state.reform_timer -= p.frame_delta_time;
            if state.reform_timer <= 0.0 {
                state.static_movers_enabled = true;
            }
        }
        if let Some(spike_index) = state.attached_spike_index.map(usize::from) {
            let spike = &mut map.entities[spike_index];
            if state.static_movers_enabled {
                spike.bounds.x = state.attached_spike_position.x;
                spike.bounds.y = state.attached_spike_position.y;
            } else {
                spike.bounds.x = -1_000_000.0;
                spike.bounds.y = -1_000_000.0;
            }
        }
        p.bounce_blocks[block_index] = state;
    }
}

fn runtime_solid_collision(map: &Map, skip_index: usize, rect: Rect) -> bool {
    map.static_solid_at(rect)
        || map.entities.iter().enumerate().any(|(index, entity)| {
            index != skip_index
                && matches!(
                    entity.kind,
                    EntityKind::BounceBlock
                        | EntityKind::CassetteBlock
                        | EntityKind::DreamBlock
                        | EntityKind::FallingBlock
                        | EntityKind::MoveBlock
                        | EntityKind::MovingSolid
                        | EntityKind::ZipMover
                        | EntityKind::ExitBlock
                        | EntityKind::InvisibleBarrier
                )
                && entity.bounds.intersects(rect)
        })
}

fn move_move_block_axis(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    entity_index: usize,
    state: &mut crate::MoveBlockSnapshot,
    horizontal: bool,
    amount: f32,
    primary: bool,
    lift_speed: Vec2,
) -> bool {
    if amount == 0.0 {
        return false;
    }
    let env = solid_collision_env(map, entity_index);
    let original = map.entities[entity_index].bounds;
    let shifted = if horizontal {
        Rect::new(
            original.x + amount,
            original.y,
            original.width,
            original.height,
        )
    } else {
        Rect::new(
            original.x,
            original.y + amount,
            original.width,
            original.height,
        )
    };
    if runtime_solid_collision(map, entity_index, shifted) {
        if !primary {
            return true;
        }
        for correction in 1..=3 {
            for sign in [1.0, -1.0] {
                let offset = correction as f32 * sign;
                let corrected = if horizontal {
                    Rect::new(
                        original.x + amount,
                        original.y + offset,
                        original.width,
                        original.height,
                    )
                } else {
                    Rect::new(
                        original.x + offset,
                        original.y + amount,
                        original.width,
                        original.height,
                    )
                };
                if runtime_solid_collision(map, entity_index, corrected) {
                    continue;
                }
                let entity = &mut map.entities[entity_index];
                move_runtime_solid_exact(
                    p,
                    &mut entity.bounds,
                    &env,
                    !horizontal,
                    offset,
                    lift_speed,
                );
                if horizontal {
                    state.position.y += offset;
                } else {
                    state.position.x += offset;
                }
                move_runtime_solid_exact(
                    p,
                    &mut entity.bounds,
                    &env,
                    horizontal,
                    amount,
                    lift_speed,
                );
                if horizontal {
                    state.position.x += amount;
                } else {
                    state.position.y += amount;
                }
                return false;
            }
        }
        return true;
    }
    let entity = &mut map.entities[entity_index];
    move_runtime_solid_exact(p, &mut entity.bounds, &env, horizontal, amount, lift_speed);
    if horizontal {
        state.position.x += amount;
    } else {
        state.position.y += amount;
    }
    false
}

fn advance_move_blocks(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    input: InputState,
    attachments: &[Option<StaticMoverAttachment>],
) {
    let block_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::MoveBlock).then_some(index))
        .collect();
    for (block_index, entity_index) in block_indices.into_iter().enumerate() {
        let mut state = p.move_blocks[block_index].clone();
        let source_direction = map.entities[entity_index].direction;
        let horizontal_source = source_direction.x != 0.0;
        let home_angle = source_direction.y.atan2(source_direction.x);
        match state.phase {
            0 => {
                state.visible = true;
                state.static_movers_enabled = true;
                if player_riding_solid(p, map.entities[entity_index].bounds) {
                    state.phase = 1;
                    state.wait_timer = 0.2;
                }
            }
            1 => {
                if state.wait_timer > p.frame_delta_time {
                    state.wait_timer -= p.frame_delta_time;
                } else {
                    state.wait_timer = 0.0;
                    state.phase = 2;
                    state.crash_timer = 0.15;
                    state.crash_reset_timer = 0.1;
                    state.no_steer_timer = 0.2;
                }
            }
            2 => {
                // MoveBlock.Controller distinguishes the source direction:
                // a horizontal block follows a player on top, while a
                // vertical block follows only a Player climbing its side.
                // Treating a rider atop a vertical block as a steering player
                // spends noSteerTimer and turns the block a frame early.
                let steering_player = if horizontal_source {
                    player_on_top_of_solid(p, map.entities[entity_index].bounds)
                } else {
                    player_climbing_solid(p, map.entities[entity_index].bounds)
                };
                let mut target_angle = home_angle;
                if steering_player {
                    if state.no_steer_timer > 0.0 {
                        state.no_steer_timer -= p.frame_delta_time;
                    }
                    if state.no_steer_timer <= 0.0 {
                        let steer = if horizontal_source {
                            input.move_y
                        } else {
                            input.move_x
                        };
                        let sign = if source_direction.x < 0.0 || source_direction.y > 0.0 {
                            -1.0
                        } else {
                            1.0
                        };
                        target_angle =
                            home_angle + std::f32::consts::FRAC_PI_4 * sign * steer as f32;
                    }
                } else {
                    state.no_steer_timer = 0.2;
                }
                state.speed = approach(state.speed, 60.0, 300.0 * p.frame_delta_time);
                state.angle = approach(
                    state.angle,
                    target_angle,
                    std::f32::consts::PI * 16.0 * p.frame_delta_time,
                );
                let velocity = Vec2::new(
                    state.angle.cos() * state.speed,
                    state.angle.sin() * state.speed,
                );
                state.lift_speed = velocity;
                state.remainder.x += velocity.x * p.frame_delta_time;
                state.remainder.y += velocity.y * p.frame_delta_time;
                let move_x = state.remainder.x.round_ties_even();
                let move_y = state.remainder.y.round_ties_even();
                state.remainder.x -= move_x;
                state.remainder.y -= move_y;
                let primary_blocked = if horizontal_source {
                    let blocked = move_move_block_axis(
                        p,
                        map,
                        entity_index,
                        &mut state,
                        true,
                        move_x,
                        true,
                        velocity,
                    );
                    let _ = move_move_block_axis(
                        p,
                        map,
                        entity_index,
                        &mut state,
                        false,
                        move_y,
                        false,
                        velocity,
                    );
                    blocked
                } else {
                    let blocked = move_move_block_axis(
                        p,
                        map,
                        entity_index,
                        &mut state,
                        false,
                        move_y,
                        true,
                        velocity,
                    );
                    let _ = move_move_block_axis(
                        p,
                        map,
                        entity_index,
                        &mut state,
                        true,
                        move_x,
                        false,
                        velocity,
                    );
                    blocked
                };
                let bounds = map.entities[entity_index].bounds;
                let outside = bounds.x < map.bounds.x
                    || bounds.y < map.bounds.y
                    || bounds.right() > map.bounds.right()
                    || (source_direction.y > 0.0 && bounds.y > map.bounds.bottom() + 32.0);
                if primary_blocked {
                    state.crash_reset_timer = 0.1;
                    if state.crash_timer > 0.0 {
                        state.crash_timer -= p.frame_delta_time;
                    } else {
                        state.phase = 3;
                        state.wait_timer = 0.2;
                        state.speed = 0.0;
                        state.angle = home_angle;
                    }
                } else if state.crash_reset_timer > 0.0 {
                    state.crash_reset_timer -= p.frame_delta_time;
                } else {
                    state.crash_timer = 0.15;
                }
                if outside {
                    state.phase = 3;
                    state.wait_timer = 0.2;
                    state.speed = 0.0;
                    state.angle = home_angle;
                }
            }
            3 => {
                if state.wait_timer > 0.0 {
                    state.wait_timer -= p.frame_delta_time;
                } else {
                    state.position = state.start;
                    state.remainder = Vec2::default();
                    state.lift_speed = Vec2::default();
                    state.visible = false;
                    state.static_movers_enabled = false;
                    state.phase = 4;
                    state.wait_timer = 2.2;
                    map.entities[entity_index].bounds.x = -1_000_000.0;
                    map.entities[entity_index].bounds.y = -1_000_000.0;
                }
            }
            4 => {
                if state.wait_timer > 0.0 {
                    state.wait_timer -= p.frame_delta_time;
                } else {
                    let source = map.entities[entity_index].bounds;
                    let restored =
                        Rect::new(state.start.x, state.start.y, source.width, source.height);
                    if !restored.intersects(current_player_rect(p, p.pos.x, p.pos.y))
                        && !runtime_solid_collision(map, entity_index, restored)
                    {
                        map.entities[entity_index].bounds = restored;
                        state.position = state.start;
                        state.phase = 5;
                        state.wait_timer = 0.8;
                    }
                }
            }
            5 => {
                if state.wait_timer > 0.0 {
                    state.wait_timer -= p.frame_delta_time;
                } else {
                    state.visible = true;
                    state.static_movers_enabled = true;
                    state.phase = 0;
                }
            }
            _ => {
                state.phase = 0;
                state.wait_timer = 0.0;
                state.speed = 0.0;
                state.angle = home_angle;
                state.position = state.start;
                state.remainder = Vec2::default();
                state.visible = true;
                state.static_movers_enabled = true;
                map.entities[entity_index].bounds.x = state.start.x;
                map.entities[entity_index].bounds.y = state.start.y;
            }
        }
        let static_movers_enabled = state.static_movers_enabled;
        p.move_blocks[block_index] = state;
        sync_platform_static_movers(map, attachments, entity_index, static_movers_enabled);
    }
}

fn theo_body_rect(position: Vec2) -> Rect {
    Rect::new(position.x - 4.0, position.y - 10.0, 8.0, 10.0)
}

fn theo_pickup_rect(position: Vec2) -> Rect {
    Rect::new(position.x - 8.0, position.y - 16.0, 16.0, 22.0)
}

fn holding_holdable(p: &PlayerSnapshot) -> bool {
    p.holding_theo.is_some() || p.holding_glider.is_some()
}

fn holding_slow_fall(p: &PlayerSnapshot) -> bool {
    p.holding_glider.is_some()
}

fn try_pickup_theo(p: &mut PlayerSnapshot) -> bool {
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    let Some(index) = p
        .theo_crystals
        .iter()
        .enumerate()
        .position(|(index, theo)| {
            // LaunchUpdate, unlike NormalUpdate and DashUpdate, does not guard
            // its Holdable loop with `Holding == null`. Holdable.Pickup itself
            // permits the current holder to pick up the same entity again, which
            // restarts PickupCoroutine after a Bumper launch freeze.
            (!theo.held || p.holding_theo == Some(index as u16))
                && theo.cannot_hold_timer <= 0.0
                && theo_pickup_rect(theo.position).intersects(player)
        })
    else {
        return false;
    };
    let theo = &mut p.theo_crystals[index];
    theo.held = true;
    theo.speed = Vec2::default();
    p.holding_theo = Some(index as u16);
    p.min_hold_timer = 0.35;
    p.ducking = false;
    p.pickup_old_speed = p.speed;
    p.pickup_old_var_jump_timer = p.var_jump_timer;
    // PickupCoroutine observes the Tween one player frame before Tween.Update
    // can deactivate it. Keep that trailing frame in the portable timer so
    // speed restoration matches the real state snapshot boundary.
    p.pickup_timer = 0.16 + p.frame_delta_time;
    p.speed = Vec2::default();
    p.demo_dashed = false;
    p.dash_end_pending = false;
    p.state = PlayerState::Pickup;
    true
}

fn release_theo(p: &mut PlayerSnapshot, input: InputState) {
    let Some(index) = p.holding_theo.take().map(usize::from) else {
        return;
    };
    let Some(theo) = p.theo_crystals.get_mut(index) else {
        return;
    };
    theo.held = false;
    theo.gravity_timer = 0.1;
    theo.cannot_hold_timer = 0.1;
    if input.move_y > 0 {
        theo.speed = Vec2::default();
    } else {
        let facing = if p.facing { 1.0 } else { -1.0 };
        theo.speed = Vec2::new(facing * 200.0, -80.0);
        p.speed.x -= facing * 80.0;
    }
}

fn glider_body_rect(position: Vec2) -> Rect {
    Rect::new(position.x - 4.0, position.y - 10.0, 8.0, 10.0)
}

fn glider_pickup_rect(position: Vec2) -> Rect {
    // Glider.cs assigns Hold.PickupCollider a 20x22 Hitbox offset -10,-16.
    // This is deliberately taller than its 8x10 body, so Player.NormalUpdate
    // can begin Pickup before the falling player reaches the jelly itself.
    Rect::new(position.x - 10.0, position.y - 16.0, 20.0, 22.0)
}

fn try_pickup_glider(p: &mut PlayerSnapshot) -> bool {
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    let Some(index) = p.gliders.iter().enumerate().position(|(index, glider)| {
        (!glider.held || p.holding_glider == Some(index as u16))
            && glider.cannot_hold_timer <= 0.0
            && glider_pickup_rect(glider.position).intersects(player)
    }) else {
        return false;
    };
    let glider = &mut p.gliders[index];
    glider.held = true;
    glider.speed = Vec2::default();
    glider.high_friction_timer = 0.5;
    p.holding_glider = Some(index as u16);
    p.min_hold_timer = 0.35;
    p.ducking = false;
    p.pickup_old_speed = p.speed;
    p.pickup_old_var_jump_timer = p.var_jump_timer;
    p.pickup_timer = 0.16 + p.frame_delta_time;
    p.speed = Vec2::default();
    p.demo_dashed = false;
    p.dash_end_pending = false;
    p.state = PlayerState::Pickup;
    true
}

fn try_pickup_holdable(p: &mut PlayerSnapshot) -> bool {
    try_pickup_theo(p) || try_pickup_glider(p)
}

fn release_glider(p: &mut PlayerSnapshot, input: InputState) {
    let Some(index) = p.holding_glider.take().map(usize::from) else {
        return;
    };
    let Some(glider) = p.gliders.get_mut(index) else {
        return;
    };
    glider.held = false;
    glider.gravity_timer = 0.1;
    glider.cannot_hold_timer = 0.3;
    if input.move_y > 0 {
        glider.speed = Vec2::default();
    } else {
        let facing = if p.facing { 1.0 } else { -1.0 };
        glider.speed = Vec2::new(facing * 100.0, -40.0);
        p.speed.x -= facing * 80.0;
    }
}

fn release_holdable(p: &mut PlayerSnapshot, input: InputState) {
    if p.holding_theo.is_some() {
        release_theo(p, input);
    } else {
        release_glider(p, input);
    }
}

fn pickup_update(p: &mut PlayerSnapshot, input: InputState) {
    if p.pickup_timer > 0.0 {
        return;
    }
    p.speed = p.pickup_old_speed;
    p.speed.y = p.speed.y.min(0.0);
    p.var_jump_timer = p.pickup_old_var_jump_timer;
    p.state = PlayerState::Normal;
    // PickupCoroutine assigns StateMachine.State = Normal after restoring
    // oldSpeed.  That transition invokes Player.NormalBegin, which resets
    // maxFall before the next NormalUpdate can apply Glider slow-fall.
    // Leaving the previous reduced cap in place makes an alternating-jelly
    // ladder under-fall after a pickup tween.
    p.max_fall = MAX_FALL;
    // Player.PickupCoroutine applies the slow-fall holdable branch after
    // restoring oldSpeed. A rising Glider pickup is clamped to at least the
    // normal jump speed even when the cached vertical speed was smaller.
    if p.holding_glider.is_some() && p.speed.y < 0.0 {
        p.speed.y = p.speed.y.min(JUMP_SPEED);
    }
    // Player.cs:5116-5131: on the frame the tween completes, a grounded player
    // still holding down with a slow-fall holdable in hand sets `holdCannotDuck`,
    // which blocks the duck re-entry at `Player.cs:3652`.
    if p.on_ground && holding_slow_fall(p) && input.move_y == 1 {
        p.hold_cannot_duck = true;
    }
}

fn theo_collides(map: &Map, position: Vec2) -> bool {
    map.solid_at(theo_body_rect(position))
}

fn move_theo_axis(
    theo: &mut crate::TheoCrystalSnapshot,
    map: &Map,
    horizontal: bool,
    delta_time: f32,
) {
    let amount = if horizontal {
        theo.speed.x * delta_time
    } else {
        theo.speed.y * delta_time
    };
    let remainder = if horizontal {
        &mut theo.remainder.x
    } else {
        &mut theo.remainder.y
    };
    *remainder += amount;
    let pixels = remainder.round_ties_even() as i32;
    *remainder -= pixels as f32;
    let sign = pixels.signum();
    for _ in 0..pixels.unsigned_abs() {
        let next = Vec2::new(
            theo.position.x + if horizontal { sign as f32 } else { 0.0 },
            theo.position.y + if horizontal { 0.0 } else { sign as f32 },
        );
        if theo_collides(map, next) {
            if horizontal {
                theo.speed.x *= -0.4;
                theo.remainder.x = 0.0;
            } else if sign > 0 && theo.speed.y > 140.0 {
                theo.speed.y *= -0.6;
                theo.remainder.y = 0.0;
            } else {
                theo.speed.y = 0.0;
                theo.remainder.y = 0.0;
            }
            break;
        }
        theo.position = next;
    }
}

/// `Puffer.Added` sets `cantExplodeTimer = 0.5f` (`Puffer.cs:167`) and the collide path refuses to
/// explode while it is positive (`:552`), so a freshly loaded Puffer cannot launch for half a second.
/// `Puffer.Update` ticks `cantExplodeTimer` down only while the Puffer is not `Gone` (`Puffer.cs:362-365`).
fn advance_puffers(p: &mut PlayerSnapshot, map: &mut Map) {
    let mut puffer_index = 0usize;
    for entity in map.entities.iter() {
        if entity.kind != EntityKind::Puffer {
            continue;
        }
        if puffer_index >= p.puffers.len() {
            break;
        }
        let puffer = &mut p.puffers[puffer_index];
        if puffer.state != 2 && puffer.cant_explode_timer > 0.0 {
            puffer.cant_explode_timer = (puffer.cant_explode_timer - p.frame_delta_time).max(0.0);
        }
        puffer_index += 1;
    }
}

fn initialize_puffers(p: &mut PlayerSnapshot, map: &mut Map) {
    let puffer_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::Puffer).then_some(index))
        .collect();
    p.puffers.truncate(puffer_indices.len());
    for puffer_index in 0..puffer_indices.len() {
        if puffer_index == p.puffers.len() {
            let bounds = map.entities[puffer_indices[puffer_index]].bounds;
            p.puffers.push(crate::PufferSnapshot {
                state: 0,
                cant_explode_timer: 0.5,
                center: Vec2::new(bounds.x + bounds.width * 0.5, bounds.y + bounds.height * 0.5),
            });
        }
    }
}

fn initialize_seekers(p: &mut PlayerSnapshot, map: &mut Map) {
    let seeker_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::Seeker).then_some(index))
        .collect();
    p.seekers.truncate(seeker_indices.len());
    for (seeker_index, entity_index) in seeker_indices.into_iter().enumerate() {
        let entity = &mut map.entities[entity_index];
        if seeker_index == p.seekers.len() {
            p.seekers.push(crate::SeekerSnapshot {
                position: Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                ),
                ..crate::SeekerSnapshot::default()
            });
        }
        let state = &p.seekers[seeker_index];
        entity.bounds = Rect::new(state.position.x - 6.0, state.position.y - 6.0, 12.0, 12.0);
    }
}

/// `TempleGate.Types` (`TempleGate.cs:11-19`), in the enum's own order, which is what
/// `data.Enum("type", Types.NearestSwitch)` (`:72-74`) parses the map's `type` attribute into.
const TEMPLE_GATE_NEAREST_SWITCH: u8 = 0;
const TEMPLE_GATE_CLOSE_BEHIND_PLAYER: u8 = 1;
const TEMPLE_GATE_CLOSE_BEHIND_PLAYER_ALWAYS: u8 = 2;
const TEMPLE_GATE_HOLDING_THEO: u8 = 3;
const TEMPLE_GATE_TOUCH_SWITCHES: u8 = 4;
const TEMPLE_GATE_CLOSE_BEHIND_PLAYER_AND_THEO: u8 = 5;

/// `TempleGate.SwitchOpen`'s two chained 0.2 s `Alarm`s (`TempleGate.cs:124-132`) and
/// `CheckTouchSwitches`' `yield return 0.5f` + 0.2 s shake beat (`:203-206`).
const TEMPLE_GATE_SWITCH_BEAT: f32 = 0.2;
const TEMPLE_GATE_TOUCH_BEAT: f32 = 0.5;

/// `TempleGate.Types` of one decoded `templeGate` (`TempleGate.cs:72-74`). `data.Enum` falls back
/// to the declared default on an unknown name, so anything unrecognised is `NearestSwitch`.
fn temple_gate_type(map: &Map, index: usize) -> u8 {
    match map
        .entity_visuals
        .get(index)
        .and_then(|visual| visual.variant.as_deref())
    {
        Some("CloseBehindPlayer") => TEMPLE_GATE_CLOSE_BEHIND_PLAYER,
        Some("CloseBehindPlayerAlways") => TEMPLE_GATE_CLOSE_BEHIND_PLAYER_ALWAYS,
        Some("HoldingTheo") => TEMPLE_GATE_HOLDING_THEO,
        Some("TouchSwitches") => TEMPLE_GATE_TOUCH_SWITCHES,
        Some("CloseBehindPlayerAndTheo") => TEMPLE_GATE_CLOSE_BEHIND_PLAYER_AND_THEO,
        _ => TEMPLE_GATE_NEAREST_SWITCH,
    }
}

/// `TempleGate`'s collider width: `Solid(position, 8f, height, safe: true)` (`TempleGate.cs:58`) is
/// eight pixels wide, except that `Awake` widens a `HoldingTheo` gate's hitbox to 16 (`:105`) -
/// which is what `CollideFirst<Player>(Position + new Vector2(8f, 0f))?.Die` (`:257`) probes when
/// such a gate closes on a player standing in the column beside it.
fn temple_gate_width(gate_type: u8) -> f32 {
    if gate_type == TEMPLE_GATE_HOLDING_THEO {
        16.0
    } else {
        8.0
    }
}

/// `TempleGate.SetHeight(0)` (`TempleGate.cs:224-241`), the collider half of `StartOpen`
/// (`:146-151`) and `Open` (`:134-144`): collapsing an 8 px `Solid` to zero height takes the
/// `height < Collider.Height` arm, so the collider shrinks *in place* - no `MoveVExact`, and
/// therefore nothing to push out of the way.
fn open_temple_gate(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    snapshot_index: usize,
    entity_index: usize,
) {
    let gate = &mut p.temple_gates[snapshot_index];
    gate.current_height = 0.0;
    gate.open = true;
    map.entities[entity_index].bounds = Rect::new(
        gate.position.x,
        gate.position.y,
        temple_gate_width(gate.gate_type),
        0.0,
    );
}

/// The room's first still-tracked `TheoCrystal`, as `(Entity.X, Entity.Center)`, or `None` when
/// the room tracks none.
///
/// `TheoCrystal(Vector2 position)` puts `Hitbox(8f, 10f, -4f, -10f)` on that position
/// (`TheoCrystal.cs:47-52`), and the decoded room stores the *hitbox* rectangle, so
/// `TheoCrystalSnapshot::position` is `Entity.Position`: `Entity.X` is four pixels left of it and
/// `Entity.Center` five pixels above it (`Monocle.Entity.CenterX/CenterY`, which add the
/// collider's own offset-and-half-size centre).
///
/// A crystal the player is carrying is not at its decoded position: `TheoCrystal.Update` parks it
/// at `Player.Position + (0, -12)` (`advance_theo_crystals`), and `initialize_theo_crystals` has
/// not moved it there yet when the gates are initialized, so the held case is resolved here.
fn theo_entity_x_and_center(p: &PlayerSnapshot, map: &Map) -> Option<(f32, Vec2)> {
    let slot = map
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::TheoCrystal)
        .enumerate()
        .find(|(slot, _)| {
            p.theo_crystals
                .get(*slot)
                .is_some_and(|theo| !theo.dead)
        })
        .map(|(slot, _)| slot)?;
    if p.holding_theo == Some(slot as u16) {
        let position = Vec2::new(p.pos.x, p.pos.y - 12.0);
        return Some((position.x - 4.0, Vec2::new(position.x, position.y - 5.0)));
    }
    let position = p.theo_crystals[slot].position;
    Some((position.x - 4.0, Vec2::new(position.x, position.y - 5.0)))
}

/// `TempleGate.TheoIsNearby` (`TempleGate.cs:214-222`):
///
/// ```csharp
/// TheoCrystal entity = base.Scene.Tracker.GetEntity<TheoCrystal>();
/// if (entity != null && !(entity.X > base.X + 10f))
///     return Vector2.DistanceSquared(holdingCheckFrom, entity.Center) < (open ? 6400f : 4096f);
/// return true;
/// ```
///
/// `holdingCheckFrom` is `Position + (Width / 2f, height / 2)` from the constructor (`:69`), i.e.
/// four pixels right and *half the decoded height* down from the gate's top-left - the closed
/// height, which `SetHeight` never changes.
fn theo_is_nearby(
    p: &PlayerSnapshot,
    map: &Map,
    gate_position: Vec2,
    closed_height: f32,
    open: bool,
) -> bool {
    let Some((theo_x, theo_center)) = theo_entity_x_and_center(p, map) else {
        return true;
    };
    if theo_x > gate_position.x + 10.0 {
        return false;
    }
    let check_x = gate_position.x + 4.0;
    let check_y = gate_position.y + closed_height * 0.5;
    let dx = check_x - theo_center.x;
    let dy = check_y - theo_center.y;
    let limit = if open { 6400.0 } else { 4096.0 };
    dx * dx + dy * dy < limit
}

/// The `TheoCrystal` half of `TempleGate.CloseBehindPlayerAndTheo`'s break condition
/// (`TempleGate.cs:179-195`): the coroutine only ever `break`s out of its loop when a live
/// crystal is *also* past `base.Right + 4f`, and a removed one leaves the loop spinning forever -
/// so a room without a live Theo never closes that gate.
fn theo_passed_gate(p: &PlayerSnapshot, map: &Map, gate_right_plus_four: f32) -> bool {
    theo_entity_x_and_center(p, map).is_some_and(|(theo_x, _)| theo_x > gate_right_plus_four)
}

fn initialize_temple_gates(p: &mut PlayerSnapshot, map: &mut Map) {
    let gate_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::TempleGate).then_some(index))
        .collect();
    p.temple_gates.truncate(gate_indices.len());
    // `Awake` (`TempleGate.cs:77-112`) is the only place a gate's start state is decided, and the
    // two things it reads - the player's rectangle and Theo's proximity - are both in the anchor
    // snapshot. A `NearestSwitch` or `TouchSwitches` gate just keeps its decoded height: only a
    // `DashSwitch`'s `SwitchOpen` (`:124-132`) or the `CheckTouchSwitches` coroutine (`:197-212`)
    // ever raises one, and both of those are transitions rather than start state.
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    for (gate_index, entity_index) in gate_indices.into_iter().enumerate() {
        if gate_index != p.temple_gates.len() {
            // The snapshot carries a gate this room already ran (`Simulator::fork`, or a room
            // transition into a rebuilt level): `Awake` does not run again, so neither does the
            // start state. Only the Solid is resynced from the carried height.
            let gate = &p.temple_gates[gate_index];
            map.entities[entity_index].bounds = Rect::new(
                gate.position.x,
                gate.position.y,
                temple_gate_width(gate.gate_type),
                gate.current_height,
            );
            continue;
        }
        let bounds = map.entities[entity_index].bounds;
        let gate_type = temple_gate_type(map, entity_index);
        let closed = Rect::new(bounds.x, bounds.y, 8.0, bounds.height);
        let start_open = match gate_type {
            // `Type == CloseBehindPlayerAlways` and `CloseBehindPlayerAndTheo` call `StartOpen()`
            // unconditionally (`TempleGate.cs:89-98`).
            TEMPLE_GATE_CLOSE_BEHIND_PLAYER_ALWAYS
            | TEMPLE_GATE_CLOSE_BEHIND_PLAYER_AND_THEO => true,
            // `entity != null && entity.Left < base.Right && entity.Bottom >= base.Top &&
            // entity.Top <= base.Bottom` (`:83`); a gate the player is not in front of stays shut
            // and no coroutine is ever added, so nothing can open it later either.
            TEMPLE_GATE_CLOSE_BEHIND_PLAYER => {
                player.x < closed.x + closed.width
                    && player.y + player.height >= closed.y
                    && player.y <= closed.y + closed.height
            }
            TEMPLE_GATE_HOLDING_THEO => {
                theo_is_nearby(p, map, Vec2::new(closed.x, closed.y), closed.height, false)
            }
            _ => false,
        };
        p.temple_gates.push(crate::TempleGateSnapshot {
            position: Vec2::new(bounds.x, bounds.y),
            current_height: bounds.height,
            closed_height: bounds.height,
            open: false,
            triggered: false,
            gate_type,
            alarm_stage: 0,
            alarm_timer: 0.0,
            claimed: false,
            // `holdingWaitTimer = 0.2f` from the field initializer (`TempleGate.cs:51`).
            holding_wait: 0.2,
            draw_height: bounds.height.max(4.0),
            draw_speed: 0.0,
            lock_state: false,
        });
        if start_open {
            // `StartOpen` + the `Awake` tail, exactly as above.
            start_open_temple_gate(p, map, gate_index, entity_index);
        } else {
            map.entities[entity_index].bounds = Rect::new(
                bounds.x,
                bounds.y,
                temple_gate_width(gate_type),
                bounds.height,
            );
        }
    }
}

fn initialize_core_mode_toggles(p: &mut PlayerSnapshot, map: &Map) {
    let count = map
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::CoreModeToggle)
        .count();
    p.core_mode_toggle_cooldowns.clear();
    p.core_mode_toggle_cooldowns.resize(count, 0.0);
}

/// `CoreModeToggle.Update` (`CoreModeToggle.cs:128-135`): the cooldown only ever
/// counts down while it is positive, and the entity's depth 2000 (`:50`) puts its
/// `Update` after `Player.Update` in the same frame.
fn advance_core_mode_toggles(p: &mut PlayerSnapshot) {
    for cooldown in &mut p.core_mode_toggle_cooldowns {
        if *cooldown > 0.0 {
            *cooldown -= p.frame_delta_time;
        }
    }
}

fn hit_theo_spring(theo: &mut crate::TheoCrystalSnapshot, map: &Map) {
    if theo.held {
        return;
    }
    let body = theo_body_rect(theo.position);
    for spring in map
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::Spring)
    {
        if !spring.bounds.intersects(body) {
            continue;
        }
        if spring.direction.y < 0.0 && theo.speed.y >= 0.0 {
            theo.speed.x *= 0.5;
            theo.speed.y = -160.0;
            theo.gravity_timer = 0.15;
            return;
        }
        if spring.direction.x > 0.0 && theo.speed.x <= 0.0 {
            theo.position.y = approach(theo.position.y, spring.bounds.y + 13.0, 4.0);
            theo.speed.x = 220.0;
            theo.speed.y = -80.0;
            theo.gravity_timer = 0.1;
            return;
        }
        if spring.direction.x < 0.0 && theo.speed.x >= 0.0 {
            theo.position.y = approach(theo.position.y, spring.bounds.y + 13.0, 4.0);
            theo.speed.x = -220.0;
            theo.speed.y = -80.0;
            theo.gravity_timer = 0.1;
            return;
        }
    }
}

fn advance_theo_crystals(p: &mut PlayerSnapshot, map: &mut Map) {
    let entity_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::TheoCrystal).then_some(index))
        .collect();
    for (theo_index, entity_index) in entity_indices.into_iter().enumerate() {
        let mut theo = p.theo_crystals[theo_index].clone();
        if theo.dead {
            map.entities[entity_index].bounds.x = -1_000_000.0;
            map.entities[entity_index].bounds.y = -1_000_000.0;
            continue;
        }
        theo.cannot_hold_timer = (theo.cannot_hold_timer - p.frame_delta_time).max(0.0);
        theo.gravity_timer = (theo.gravity_timer - p.frame_delta_time).max(0.0);
        if p.holding_theo == Some(theo_index as u16) {
            theo.held = true;
            theo.position = Vec2::new(p.pos.x, p.pos.y - 12.0);
            theo.speed = Vec2::default();
            theo.remainder = Vec2::default();
        } else {
            theo.held = false;
            let on_ground = theo_collides(map, Vec2::new(theo.position.x, theo.position.y + 1.0));
            if on_ground {
                theo.speed.x = approach(theo.speed.x, 0.0, 800.0 * p.frame_delta_time);
            } else if theo.gravity_timer <= 0.0 {
                let gravity = if theo.speed.y.abs() <= 30.0 {
                    400.0
                } else {
                    800.0
                };
                theo.speed.x = approach(
                    theo.speed.x,
                    0.0,
                    // TheoCrystal's release gravity delay postpones vertical
                    // acceleration only. Its airborne horizontal damping is
                    // still the 200 px/s² carry-release curve, including
                    // after the crystal starts falling back toward the floor.
                    200.0 * p.frame_delta_time,
                );
                theo.speed.y = approach(theo.speed.y, 200.0, gravity * p.frame_delta_time);
            }
            move_theo_axis(&mut theo, map, true, p.frame_delta_time);
            move_theo_axis(&mut theo, map, false, p.frame_delta_time);
            hit_theo_spring(&mut theo, map);
        }
        let entity = &mut map.entities[entity_index];
        entity.bounds.x = theo.position.x - 4.0;
        entity.bounds.y = theo.position.y - 10.0;
        p.theo_crystals[theo_index] = theo;
    }
}

fn advance_heart_gems(p: &mut PlayerSnapshot) {
    for heart in &mut p.heart_gems {
        match heart.phase {
            1 => {
                if heart.wait_frames > 0 {
                    heart.wait_frames -= 1;
                } else {
                    // HeartGem.CollectRoutine yields one scene frame, then
                    // calls Celeste.Freeze(.2f) and yields again.
                    p.freeze_timer = 0.2;
                    heart.phase = 2;
                }
            }
            2 => {
                // The coroutine resumes on the first scene update after the
                // raw-time freeze, writes Engine.TimeRate, and immediately
                // executes the first raw-time approach before yielding.
                p.time_rate = approach(0.5, 0.0, DT * 0.25);
                heart.phase = 3;
            }
            3 => {
                p.time_rate = approach(p.time_rate, 0.0, DT * 0.25);
            }
            _ => {}
        }
    }
}

fn update_camera(p: &mut PlayerSnapshot, map: &Map) {
    if p.transition_timer > 0.0 || p.dead {
        return;
    }
    // Lookout.LookRoutine owns Level.Camera directly from the first camera
    // control frame through HUD exit and a possible long-distance FadeWipe.
    if p.lookouts
        .iter()
        .any(|lookout| lookout.interacting && !lookout.removed && lookout.phase >= 4)
    {
        return;
    }
    let target = camera_target(p, map);
    let multiplier = if p.state == PlayerState::TempleFall {
        8.0
    } else {
        1.0
    };
    let amount = 1.0 - (0.01_f32 / multiplier).powf(p.frame_delta_time);
    p.camera.x += (target.x - p.camera.x) * amount;
    p.camera.y += (target.y - p.camera.y) * amount;
}

fn quadratic_curve(begin: Vec2, end: Vec2, control: Vec2, percent: f32) -> Vec2 {
    let inverse = 1.0 - percent;
    Vec2::new(
        inverse * inverse * begin.x
            + 2.0 * inverse * percent * control.x
            + percent * percent * end.x,
        inverse * inverse * begin.y
            + 2.0 * inverse * percent * control.y
            + percent * percent * end.y,
    )
}

fn lerp_vec(begin: Vec2, end: Vec2, percent: f32) -> Vec2 {
    Vec2::new(
        begin.x + (end.x - begin.x) * percent,
        begin.y + (end.y - begin.y) * percent,
    )
}

fn lookout_camera_blocked(map: &Map, camera: Vec2) -> bool {
    let viewport = Rect::new(camera.x, camera.y, 320.0, 180.0);
    map.entities
        .iter()
        .any(|entity| entity.name == "lookoutBlocker" && entity.bounds.intersects(viewport))
}

fn prepare_lookout_player(p: &mut PlayerSnapshot) {
    let Some(lookout) = p
        .lookouts
        .iter()
        .find(|lookout| lookout.interacting && !lookout.removed)
        .cloned()
    else {
        return;
    };
    match lookout.phase {
        // DummyWalkToExact sets StDummy once as its coroutine starts.  It does
        // not reassign that state on every yielded frame: a native Booster can
        // therefore interrupt the walk and leave its StBoost/StNormal recovery
        // running alongside the still-interacting Lookout.
        1 if lookout.timer < 1.0 => {
            p.state = PlayerState::Dummy;
            p.dummy_moving = true;
            let direction = (lookout.position.x - p.pos.x).signum();
            if direction != 0.0 {
                p.facing = direction > 0.0;
            }
        }
        2 | 3 if p.state == PlayerState::Dummy => {
            // Lookout only assigns StDummy once, when DummyWalkToExact starts.
            // If a Booster has replaced it with StBoost, the walk's final
            // speed reset still completes, but the following HUD waits must
            // not overwrite that live state before BoostUpdate can MoveToX.
            p.dummy_moving = false;
            p.speed.x = 0.0;
        }
        _ => {}
    }
}

fn advance_free_lookout_camera(
    state: &mut crate::LookoutSnapshot,
    entity: &crate::Entity,
    p: &mut PlayerSnapshot,
    map: &Map,
    input: InputState,
) {
    let mut aim = input_vector(input);
    if entity.direction.x != 0.0 {
        aim.x = 0.0;
    }
    state.cam_speed.x += 800.0 * aim.x * p.frame_delta_time;
    state.cam_speed.y += 800.0 * aim.y * p.frame_delta_time;
    if aim.x == 0.0 {
        state.cam_speed.x = approach(state.cam_speed.x, 0.0, 1600.0 * p.frame_delta_time);
    }
    if aim.y == 0.0 {
        state.cam_speed.y = approach(state.cam_speed.y, 0.0, 1600.0 * p.frame_delta_time);
    }
    if length(state.cam_speed) > 240.0 {
        state.cam_speed = scale(normalize(state.cam_speed), 240.0);
    }

    let bounds = p.current_room_bounds.unwrap_or(map.bounds);
    let previous = state.cam;
    state.cam.x += state.cam_speed.x * p.frame_delta_time;
    if state.cam.x < bounds.x || state.cam.x + 320.0 > bounds.right() {
        state.cam_speed.x = 0.0;
    }
    state.cam.x = state
        .cam
        .x
        .clamp(bounds.x, (bounds.right() - 320.0).max(bounds.x));
    if lookout_camera_blocked(map, state.cam) {
        state.cam.x = previous.x;
        state.cam_speed.x = 0.0;
    }

    state.cam.y += state.cam_speed.y * p.frame_delta_time;
    if state.cam.y < bounds.y || state.cam.y + 180.0 > bounds.bottom() {
        state.cam_speed.y = 0.0;
    }
    state.cam.y = state
        .cam
        .y
        .clamp(bounds.y, (bounds.bottom() - 180.0).max(bounds.y));
    if lookout_camera_blocked(map, state.cam) {
        state.cam.y = previous.y;
        state.cam_speed.y = 0.0;
    }
    p.camera = state.cam;
}

fn advance_node_lookout_camera(
    state: &mut crate::LookoutSnapshot,
    entity: &crate::Entity,
    p: &mut PlayerSnapshot,
    input: InputState,
) {
    let nodes = &entity.nodes;
    if nodes.is_empty() {
        return;
    }
    let node = (state.node as usize).min(nodes.len() - 1);
    state.node = node as u16;
    let start_center = Vec2::new(state.cam_start.x + 160.0, state.cam_start.y + 90.0);
    let begin = if node == 0 {
        start_center
    } else {
        nodes[node - 1]
    };
    let end = nodes[node];
    let percent = state.node_percent;
    let center = if percent < 0.25 && node > 0 {
        let curve_begin = lerp_vec(
            if node <= 1 {
                start_center
            } else {
                nodes[node - 2]
            },
            begin,
            0.75,
        );
        let curve_end = lerp_vec(begin, end, 0.25);
        quadratic_curve(curve_begin, curve_end, begin, 0.5 + percent / 0.25 * 0.5)
    } else if percent > 0.75 && node + 1 < nodes.len() {
        let curve_begin = lerp_vec(begin, end, 0.75);
        let curve_end = lerp_vec(end, nodes[node + 1], 0.25);
        quadratic_curve(curve_begin, curve_end, end, (percent - 0.75) / 0.25 * 0.5)
    } else {
        lerp_vec(begin, end, percent)
    };
    state.cam = Vec2::new(center.x - 160.0, center.y - 90.0);
    p.camera = state.cam;

    let segment_length = length(Vec2::new(end.x - begin.x, end.y - begin.y));
    if segment_length > 0.0 {
        state.node_percent -= input.move_y as f32 * (240.0 / segment_length) * p.frame_delta_time;
    }
    if state.node_percent < 0.0 {
        if state.node > 0 {
            state.node -= 1;
            state.node_percent = 1.0;
        } else {
            state.node_percent = 0.0;
        }
    } else if state.node_percent > 1.0 {
        if state.node as usize + 1 < nodes.len() {
            state.node += 1;
            state.node_percent = 0.0;
        } else {
            state.node_percent = 1.0;
        }
    }
}

fn finish_lookout(p: &mut PlayerSnapshot, state: &mut crate::LookoutSnapshot) {
    state.interacting = false;
    state.phase = 0;
    state.timer = 0.0;
    state.hud_easer = 0.0;
    p.state = PlayerState::Normal;
    p.dummy_moving = false;
}

fn lookout_wipe_duration(state: &crate::LookoutSnapshot, entity: &crate::Entity) -> f32 {
    let at_summit_top = entity.direction.y != 0.0
        && !entity.nodes.is_empty()
        && state.node as usize >= entity.nodes.len() - 1
        && state.node_percent >= 0.95;
    if at_summit_top { 1.0 } else { 0.5 }
}

/// `LookRoutine` starts `hud.Display.Hide()` in the same coroutine resume
/// that consumes its exit input.  A nearby camera therefore stays in Dummy
/// for the remaining HUD ease, whereas a long-distance camera takes the
/// immediate FadeWipe branch below.
fn advance_lookout_exit(p: &mut PlayerSnapshot, state: &mut crate::LookoutSnapshot) {
    state.hud_easer = approach(state.hud_easer, 0.0, p.frame_delta_time * 3.0);
    if state.hud_easer <= 0.0 {
        finish_lookout(p, state);
    }
}

fn advance_lookouts(
    p: &mut PlayerSnapshot,
    map: &Map,
    input: InputState,
    menu_cancel_pressed: bool,
) {
    let indices = lookout_entity_indices(map);
    let Some((lookout_index, entity_index)) =
        indices.into_iter().enumerate().find(|(lookout_index, _)| {
            p.lookouts
                .get(*lookout_index)
                .is_some_and(|state| state.interacting && !state.removed)
        })
    else {
        return;
    };
    let entity = &map.entities[entity_index];
    let mut state = p.lookouts[lookout_index].clone();

    match state.phase {
        1 => {
            // Lookout updates after Player. Its coroutine first assigns
            // StDummy, then its nested DummyWalkToExact only writes the first
            // 16.667 speed after its two initial coroutine resumes. Advancing it in
            // prepare_lookout_player would move the Rust trace one frame
            // ahead of Everest.
            // The initial `LookRoutine` yield happens even when the player is
            // already exactly at the lookout.  Keep phase 1 through that
            // frame, so the next Player.Update receives DummyWalkToExact's
            // unconditional StDummy assignment before the exact-arrival path
            // advances to the HUD wait.
            if state.timer < 0.0 {
                state.timer += 1.0;
            // DummyWalkToExact yields while `X != x`; being one pixel short
            // must still let its next resume write Speed.X after BoostUpdate.
            } else if p.pos.x == state.position.x {
                p.pos.x = state.position.x;
                // DummyWalkToExact only assigns `X = x` after an overstep.
                // On an ordinary exact arrival it leaves Actor's subpixel
                // counter intact, so the live BoostUpdate continues from the
                // remainder accumulated on the preceding movement frame.
                p.speed.x = 0.0;
                p.dummy_moving = false;
                if p.dead || !grounded_at_offset(p, map, 1.0) {
                    if !p.dead {
                        p.state = PlayerState::Normal;
                    }
                    state.interacting = false;
                    state.phase = 0;
                } else {
                    state.phase = 2;
                    state.timer = 0.2;
                }
            } else if state.timer < 1.0 {
                state.timer += 1.0;
            } else {
                let direction = (state.position.x - p.pos.x).signum();
                p.speed.x = approach(p.speed.x, direction * 64.0, RUN_ACCEL * p.frame_delta_time);
            }
        }
        2 => {
            state.timer -= p.frame_delta_time;
            if state.timer <= 0.0 {
                state.phase = 3;
                state.timer = 0.0;
                state.hud_easer = 0.0;
                state.node = 0;
                state.node_percent = 0.0;
            }
        }
        3 => {
            state.hud_easer = approach(state.hud_easer, 1.0, p.frame_delta_time * 3.0);
            if state.hud_easer >= 1.0 {
                state.phase = 4;
                state.cam_start = p.camera;
                state.cam = p.camera;
                state.cam_speed = Vec2::default();
            }
        }
        4 => {
            // The portable Jump edge also drives Lookout's MenuCancel.  It
            // must remain a raw edge rather than Player's buffered Jump:
            // Player.Update can consume that buffer before LookRoutine reads
            // Input.MenuCancel.Pressed later in the same scene update.
            let exit_pressed =
                menu_cancel_pressed || input.dash_pressed || input.crouch_dash_pressed;
            if entity.nodes.is_empty() {
                advance_free_lookout_camera(&mut state, entity, p, map, input);
            } else {
                advance_node_lookout_camera(&mut state, entity, p, input);
            }
            // Lookout.LookRoutine keeps its node camera alive at the summit
            // endpoint. It exits only through MenuCancel or Dash; reaching a
            // final `summit` node is not a synthetic timeout.
            if exit_pressed {
                let delta = Vec2::new(
                    p.camera.x - state.cam_start.x,
                    p.camera.y - state.cam_start.y,
                );
                if length(delta) > 600.0 {
                    // The source leaves its input loop straight into the
                    // long-distance FadeWipe branch; it does not wait for
                    // the HUD ease-out before starting the summit wipe.
                    state.hud_easer = 0.0;
                    state.phase = 6;
                    // FadeWipe advances once when LookRoutine creates it,
                    // even though its first rendered sample remains at
                    // progress zero. Store the next-frame progress here so
                    // its Wait resumes exactly sixty summit frames later.
                    state.timer =
                        (p.frame_delta_time / lookout_wipe_duration(&state, entity)).min(1.0);
                    state.wipe_start = p.camera;
                } else {
                    state.phase = 5;
                    // The coroutine calls `hud.Display.Hide()` before its
                    // next yield, so consume the first 3/s easing step on
                    // this same MenuCancel/Dash frame.
                    advance_lookout_exit(p, &mut state);
                }
            }
        }
        5 => {
            advance_lookout_exit(p, &mut state);
        }
        6 => {
            let duration = lookout_wipe_duration(&state, entity);
            let direction = normalize(Vec2::new(
                state.wipe_start.x - state.cam_start.x,
                state.wipe_start.y - state.cam_start.y,
            ));
            if state.timer < 1.0 {
                let cube = state.timer * state.timer * state.timer;
                p.camera = Vec2::new(
                    state.wipe_start.x - direction.x * 64.0 * cube,
                    state.wipe_start.y - direction.y * 64.0 * cube,
                );
                state.cam = p.camera;
                // FadeWipe's eased progress is clamped at its duration.  In
                // particular, sixty 1/60 updates must resume LookRoutine on
                // the next frame rather than accumulate below one forever.
                state.timer = (state.timer + p.frame_delta_time / duration).min(1.0);
            } else {
                p.camera = Vec2::new(
                    state.cam_start.x + direction.x * 32.0,
                    state.cam_start.y + direction.y * 32.0,
                );
                state.cam = p.camera;
                finish_lookout(p, &mut state);
            }
        }
        _ => finish_lookout(p, &mut state),
    }
    p.lookouts[lookout_index] = state;
}

fn try_begin_lookout(p: &mut PlayerSnapshot, map: &Map, input: InputState) {
    if !input.talk_pressed || p.dead || p.state != PlayerState::Normal {
        return;
    }
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    for (lookout_index, entity_index) in lookout_entity_indices(map).into_iter().enumerate() {
        let state = &p.lookouts[lookout_index];
        if state.interacting || state.removed {
            continue;
        }
        let position = Vec2::new(
            map.entities[entity_index].bounds.x + 2.0,
            map.entities[entity_index].bounds.y + 4.0,
        );
        let talk = Rect::new(position.x - 24.0, position.y - 8.0, 48.0, 8.0);
        if talk.intersects(player) {
            let state = &mut p.lookouts[lookout_index];
            state.interacting = true;
            state.phase = 1;
            // `Interact` schedules LookRoutine, then its yielded
            // DummyWalkToExact reaches its first movement write after two
            // entity updates. Keep that coroutine latency apart from the
            // phase-2 HUD timer.
            state.timer = -1.0;
            state.position = position;
            state.cam_start = p.camera;
            state.cam = p.camera;
            state.cam_speed = Vec2::default();
            state.node = 0;
            state.node_percent = 0.0;
            state.hud_easer = 0.0;
            // `Lookout.Interact` only starts its coroutine. Its first
            // `LookRoutine` step runs with the following entity update, where
            // it assigns `StDummy`; the Talk-pressed frame remains Normal.
            break;
        }
    }
}

fn clamped_map(value: f32, min: f32, max: f32, out_min: f32, out_max: f32) -> f32 {
    if max == min {
        return out_min;
    }
    let t = ((value - min) / (max - min)).clamp(0.0, 1.0);
    out_min + (out_max - out_min) * t
}

fn advance_rising_lavas(p: &mut PlayerSnapshot, map: &mut Map) {
    let indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::RisingLava).then_some(index))
        .collect();
    for (lava_index, entity_index) in indices.into_iter().enumerate() {
        let state = &mut p.rising_lavas[lava_index];
        state.ice_mode = p.core_mode == crate::CoreMode::Cold;
        state.delay -= p.frame_delta_time;
        state.position.x = p.camera.x;
        if state.waiting {
            if !state.intro && p.just_respawned {
                state.position.y =
                    approach(state.position.y, p.pos.y + 32.0, 32.0 * p.frame_delta_time);
            }
            if (!state.ice_mode || !state.intro) && !p.just_respawned {
                state.waiting = false;
            }
        } else {
            let camera_line = p.camera.y + 168.0;
            if state.position.y > camera_line + 96.0 {
                state.position.y = camera_line + 96.0;
            }
            let speed_multiplier = if state.position.y > camera_line {
                clamped_map(state.position.y - camera_line, 0.0, 96.0, 1.0, 2.0)
            } else {
                clamped_map(camera_line - state.position.y, 0.0, 32.0, 1.0, 0.5)
            };
            if state.delay <= 0.0 {
                state.position.y -= 30.0 * speed_multiplier * p.frame_delta_time;
            }
        }
        map.entities[entity_index].bounds =
            Rect::new(state.position.x, state.position.y, 340.0, 120.0);
    }
}

fn advance_sandwich_lavas(p: &mut PlayerSnapshot, map: &mut Map) {
    let indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::SandwichLava).then_some(index))
        .collect();
    let live_count = p.sandwich_lavas.iter().filter(|lava| !lava.removed).count();
    for (lava_index, entity_index) in indices.into_iter().enumerate() {
        let state = &mut p.sandwich_lavas[lava_index];
        if state.removed {
            map.entities[entity_index].bounds = Rect::new(-1_000_000.0, -1_000_000.0, 340.0, 120.0);
            continue;
        }
        if p.transition_timer > 0.0 && state.persistent && live_count <= 1 && !state.leaving {
            state.leaving = true;
            state.leave_timer = 2.0;
        }
        state.ice_mode = p.core_mode == crate::CoreMode::Cold;
        state.position.x = p.camera.x;
        state.delay -= p.frame_delta_time;
        if state.leaving {
            state.leave_timer = (state.leave_timer - p.frame_delta_time).max(0.0);
            if state.leave_timer <= 0.0 {
                state.removed = true;
            }
        } else if state.waiting {
            state.position.y = approach(
                state.position.y,
                map.bounds.bottom() - 10.0,
                128.0 * p.frame_delta_time,
            );
            if p.pos.x >= state.start_x && !p.just_respawned && p.state != PlayerState::Frozen {
                state.waiting = false;
            }
        } else if state.delay <= 0.0 {
            state.position.y += if state.ice_mode { 20.0 } else { -20.0 } * p.frame_delta_time;
        }
        state.top_rect_y = approach(
            state.top_rect_y,
            -360.0 + if state.leaving { -512.0 } else { 0.0 },
            if state.leaving { 256.0 } else { 64.0 } * p.frame_delta_time,
        );
        state.bottom_rect_y = approach(
            state.bottom_rect_y,
            if state.leaving { 512.0 } else { 0.0 },
            if state.leaving { 256.0 } else { 64.0 } * p.frame_delta_time,
        );
        map.entities[entity_index].bounds = if state.leaving || state.removed {
            Rect::new(-1_000_000.0, -1_000_000.0, 340.0, 120.0)
        } else {
            Rect::new(state.position.x, state.position.y, 340.0, 120.0)
        };
    }
}

fn glider_collides(map: &Map, position: Vec2) -> bool {
    map.solid_at(glider_body_rect(position))
}

fn move_glider_axis(
    glider: &mut crate::GliderSnapshot,
    map: &Map,
    horizontal: bool,
    delta_time: f32,
) {
    let amount = if horizontal {
        glider.speed.x * delta_time
    } else {
        glider.speed.y * delta_time
    };
    let remainder = if horizontal {
        &mut glider.remainder.x
    } else {
        &mut glider.remainder.y
    };
    *remainder += amount;
    let pixels = remainder.round_ties_even() as i32;
    *remainder -= pixels as f32;
    let sign = pixels.signum();
    for _ in 0..pixels.unsigned_abs() {
        let next = Vec2::new(
            glider.position.x + if horizontal { sign as f32 } else { 0.0 },
            glider.position.y + if horizontal { 0.0 } else { sign as f32 },
        );
        if glider_collides(map, next) {
            if horizontal {
                glider.speed.x *= -1.0;
                glider.remainder.x = 0.0;
            } else if glider.speed.y < 0.0 {
                glider.speed.y *= -0.5;
                glider.remainder.y = 0.0;
            } else {
                glider.speed.y = 0.0;
                glider.remainder.y = 0.0;
            }
            break;
        }
        glider.position = next;
    }
}

fn hit_glider_spring(glider: &mut crate::GliderSnapshot, map: &Map) {
    if glider.held {
        return;
    }
    let body = glider_body_rect(glider.position);
    for spring in map
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::Spring)
    {
        if !spring.bounds.intersects(body) {
            continue;
        }
        if spring.direction.y < 0.0 && glider.speed.y >= 0.0 {
            glider.speed.x *= 0.5;
            glider.speed.y = -160.0;
            glider.no_gravity_timer = 0.15;
            return;
        }
        if spring.direction.x > 0.0 && glider.speed.x <= 0.0 {
            glider.speed.x = 160.0;
            glider.speed.y = -80.0;
            glider.no_gravity_timer = 0.1;
            return;
        }
        if spring.direction.x < 0.0 && glider.speed.x >= 0.0 {
            glider.speed.x = -160.0;
            glider.speed.y = -80.0;
            glider.no_gravity_timer = 0.1;
            return;
        }
    }
}

fn advance_gliders(p: &mut PlayerSnapshot, map: &mut Map) {
    let entity_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::Glider).then_some(index))
        .collect();
    for (glider_index, entity_index) in entity_indices.into_iter().enumerate() {
        let mut glider = p.gliders[glider_index].clone();
        if glider.removed {
            map.entities[entity_index].bounds.x = -1_000_000.0;
            map.entities[entity_index].bounds.y = -1_000_000.0;
            continue;
        }
        // Glider.Update checks this timer before its DeltaTime subtraction,
        // so the final positive frame still suppresses gravity.
        let spring_no_gravity_active = glider.no_gravity_timer > 0.0;
        glider.cannot_hold_timer = (glider.cannot_hold_timer - p.frame_delta_time).max(0.0);
        glider.gravity_timer = (glider.gravity_timer - p.frame_delta_time).max(0.0);
        glider.no_gravity_timer = (glider.no_gravity_timer - p.frame_delta_time).max(0.0);
        glider.high_friction_timer = (glider.high_friction_timer - p.frame_delta_time).max(0.0);
        if p.holding_glider == Some(glider_index as u16) {
            glider.held = true;
            glider.position = Vec2::new(p.pos.x, p.pos.y - 12.0);
            glider.speed = Vec2::default();
            glider.remainder = Vec2::default();
        } else {
            glider.held = false;
            let on_ground =
                glider_collides(map, Vec2::new(glider.position.x, glider.position.y + 1.0));
            if on_ground {
                glider.speed.x = approach(glider.speed.x, 0.0, 800.0 * p.frame_delta_time);
            } else if glider.gravity_timer <= 0.0 {
                let gravity = if glider.speed.y >= -30.0 {
                    100.0
                } else {
                    200.0
                };
                let friction = if glider.speed.y < 0.0 || glider.high_friction_timer <= 0.0 {
                    40.0
                } else {
                    10.0
                };
                glider.speed.x = approach(glider.speed.x, 0.0, friction * p.frame_delta_time);
                if !spring_no_gravity_active {
                    glider.speed.y = approach(glider.speed.y, 30.0, gravity * p.frame_delta_time);
                }
            }
            move_glider_axis(&mut glider, map, true, p.frame_delta_time);
            move_glider_axis(&mut glider, map, false, p.frame_delta_time);
            hit_glider_spring(&mut glider, map);
        }
        let entity = &mut map.entities[entity_index];
        entity.bounds.x = glider.position.x - 4.0;
        entity.bounds.y = glider.position.y - 10.0;
        p.gliders[glider_index] = glider;
    }
}

fn advance_zip_movers(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    attachments: &[Option<StaticMoverAttachment>],
) {
    let zip_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::ZipMover).then_some(index))
        .collect();
    for (zip_index, entity_index) in zip_indices.into_iter().enumerate() {
        let mut state = p.zip_movers[zip_index].clone();
        let env = solid_collision_env(map, entity_index);
        let entity = &mut map.entities[entity_index];
        let target = entity.nodes.first().copied().unwrap_or(state.start);
        match state.phase {
            0 => {
                if player_riding_solid(p, entity.bounds) {
                    state.phase = 1;
                    state.wait_timer = 0.1;
                }
            }
            1 => {
                if state.wait_timer > 0.0 {
                    state.wait_timer -= p.frame_delta_time;
                } else {
                    state.phase = 2;
                    state.at = 0.0;
                }
            }
            2 => {
                state.at = approach(state.at, 1.0, 2.0 * p.frame_delta_time);
                let eased = sine_in(state.at);
                let desired = Vec2::new(
                    state.start.x + (target.x - state.start.x) * eased,
                    state.start.y + (target.y - state.start.y) * eased,
                );
                move_zip_mover_to(p, entity, &mut state, &env, desired);
                if state.at >= 1.0 {
                    state.phase = 3;
                    state.wait_timer = 0.5;
                }
            }
            3 => {
                if state.wait_timer > 0.0 {
                    state.wait_timer -= p.frame_delta_time;
                } else {
                    state.phase = 4;
                    state.at = 0.0;
                }
            }
            4 => {
                state.at = approach(state.at, 1.0, 0.5 * p.frame_delta_time);
                let eased = sine_in(state.at);
                let desired = Vec2::new(
                    target.x + (state.start.x - target.x) * eased,
                    target.y + (state.start.y - target.y) * eased,
                );
                move_zip_mover_to(p, entity, &mut state, &env, desired);
                if state.at >= 1.0 {
                    state.phase = 5;
                    state.wait_timer = 0.5;
                }
            }
            5 => {
                if state.wait_timer > 0.0 {
                    state.wait_timer -= p.frame_delta_time;
                } else if player_riding_solid(p, entity.bounds) {
                    state.phase = 1;
                    state.wait_timer = 0.1;
                } else {
                    state.phase = 0;
                }
            }
            _ => {
                state.phase = 0;
                state.wait_timer = 0.0;
                state.at = 0.0;
            }
        }
        p.zip_movers[zip_index] = state;
        sync_platform_static_movers(map, attachments, entity_index, true);
    }
}

fn advance_moving_solids(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    attachments: &[Option<StaticMoverAttachment>],
) {
    let old_time = p.moving_solid_time;
    let new_time = old_time + p.frame_delta_time;
    let moving_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::MovingSolid).then_some(index))
        .collect();
    for entity_index in moving_indices {
        {
            let entity = &mut map.entities[entity_index];
            let old_offset = Vec2::new(
                (entity.direction.x * old_time).round_ties_even(),
                (entity.direction.y * old_time).round_ties_even(),
            );
            let new_offset = Vec2::new(
                (entity.direction.x * new_time).round_ties_even(),
                (entity.direction.y * new_time).round_ties_even(),
            );
            let delta = Vec2::new(new_offset.x - old_offset.x, new_offset.y - old_offset.y);
            let riding = entity
                .bounds
                .intersects(current_player_rect(p, p.pos.x, p.pos.y + 1.0));
            if riding {
                set_lift_speed(p, entity.direction);
            }

            if delta.x != 0.0 {
                entity.bounds.x += delta.x;
                if riding {
                    p.pos.x += delta.x;
                } else {
                    let player = current_player_rect(p, p.pos.x, p.pos.y);
                    if entity.bounds.intersects(player) {
                        if delta.x > 0.0 {
                            p.pos.x += entity.bounds.right() - player.x;
                        } else {
                            p.pos.x -= player.right() - entity.bounds.x;
                        }
                    }
                }
            }
            if delta.y != 0.0 {
                entity.bounds.y += delta.y;
                if riding {
                    p.pos.y += delta.y;
                } else {
                    let player = current_player_rect(p, p.pos.x, p.pos.y);
                    if entity.bounds.intersects(player) {
                        if delta.y > 0.0 {
                            p.pos.y += entity.bounds.bottom() - player.y;
                        } else {
                            p.pos.y -= player.bottom() - entity.bounds.y;
                        }
                    }
                }
            }
        }
        sync_platform_static_movers(map, attachments, entity_index, true);
    }
    p.moving_solid_time = new_time;
}

fn cassette_entity_indices(map: &Map) -> Vec<usize> {
    map.entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::CassetteBlock).then_some(index))
        .collect()
}

fn cassette_bounds(state: &crate::CassetteBlockSnapshot) -> Rect {
    Rect::new(
        state.position.x,
        state.position.y,
        state.width,
        state.height,
    )
}

fn sync_cassette_entity(entity: &mut crate::Entity, state: &crate::CassetteBlockSnapshot) {
    if state.collidable {
        entity.bounds = cassette_bounds(state);
    } else {
        park_entity(entity);
    }
}

fn shift_cassette_block(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    entity_index: usize,
    state: &mut crate::CassetteBlockSnapshot,
    amount: f32,
) {
    if state.collidable {
        let env = solid_collision_env(map, entity_index);
        let entity = &mut map.entities[entity_index];
        move_runtime_solid_exact(
            p,
            &mut entity.bounds,
            &env,
            false,
            amount,
            Vec2::new(0.0, amount / DT),
        );
    }
    state.position.y += amount;
}

fn try_cassette_player_wiggle_up(
    p: &mut PlayerSnapshot,
    map: &Map,
    block_index: usize,
    intended: Rect,
) -> bool {
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    if !intended.intersects(player) {
        return true;
    }
    let index = p.cassette_blocks[block_index].index;
    if p.cassette_blocks
        .iter()
        .enumerate()
        .any(|(other_index, other)| {
            other_index != block_index
                && other.index == index
                && Rect::new(
                    other.position.x,
                    other.position.y + 4.0,
                    other.width,
                    other.height,
                )
                .intersects(player)
        })
    {
        return false;
    }
    for amount in 1..=4 {
        let candidate = current_player_rect(p, p.pos.x, p.pos.y - amount as f32);
        if !intended.intersects(candidate) && !map.solid_at(candidate) {
            p.pos.y -= amount as f32;
            return true;
        }
    }
    false
}

fn advance_cassette_blocks(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    attachments: &[Option<StaticMoverAttachment>],
) {
    let entity_indices = cassette_entity_indices(map);
    for (block_index, entity_index) in entity_indices.iter().copied().enumerate() {
        let mut state = p.cassette_blocks[block_index].clone();
        if state.activated && !state.collidable {
            let intended = cassette_bounds(&state);
            if try_cassette_player_wiggle_up(p, map, block_index, intended) {
                state.collidable = true;
                map.entities[entity_index].bounds = intended;
                shift_cassette_block(p, map, entity_index, &mut state, -1.0);
            }
        } else if !state.activated && state.collidable {
            shift_cassette_block(p, map, entity_index, &mut state, 1.0);
            state.collidable = false;
        }
        sync_cassette_entity(&mut map.entities[entity_index], &state);
        let collidable = state.collidable;
        p.cassette_blocks[block_index] = state;
        sync_platform_static_movers(map, attachments, entity_index, collidable);
    }
}

fn advance_cassette_manager(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    attachments: &[Option<StaticMoverAttachment>],
) {
    // No manager exists once the tape is taken (`Level.cs:278-288`), so the beat clock never runs.
    if p.cassette_manager.tape_taken
        || !p.cassette_manager.initialized
        || p.cassette_manager.max_beat == 0
    {
        return;
    }
    if p.cassette_manager.startup_music_pending {
        p.cassette_manager.startup_music_pending = false;
        return;
    }
    p.cassette_manager.beat_timer += DT * p.cassette_manager.tempo_mult;
    if p.cassette_manager.beat_timer < CASSETTE_BEAT_INTERVAL {
        return;
    }
    p.cassette_manager.beat_timer -= CASSETTE_BEAT_INTERVAL;
    p.cassette_manager.beat_index = p.cassette_manager.beat_index.wrapping_add(1);
    let beat_index = p.cassette_manager.beat_index;
    let entity_indices = cassette_entity_indices(map);
    if beat_index % 8 == 0 {
        p.cassette_manager.current_index =
            (p.cassette_manager.current_index + 1) % p.cassette_manager.max_beat;
        for state in &mut p.cassette_blocks {
            state.activated = state.index == p.cassette_manager.current_index;
        }
    } else if beat_index.wrapping_add(1) % 8 == 0 {
        let next_index = (p.cassette_manager.current_index + 1) % p.cassette_manager.max_beat;
        for (block_index, entity_index) in entity_indices.into_iter().enumerate() {
            let mut state = p.cassette_blocks[block_index].clone();
            if state.index == next_index || state.activated {
                let amount = if state.collidable { 1.0 } else { -1.0 };
                shift_cassette_block(p, map, entity_index, &mut state, amount);
                sync_cassette_entity(&mut map.entities[entity_index], &state);
                let collidable = state.collidable;
                p.cassette_blocks[block_index] = state;
                sync_platform_static_movers(map, attachments, entity_index, collidable);
            }
        }
    }
}

fn scene_on_interval(time_active: f32, interval: f32, offset: f32) -> bool {
    ((time_active - offset - DT) / interval).floor() < ((time_active - offset) / interval).floor()
}

fn advance_spinners(p: &mut PlayerSnapshot, map: &mut Map) {
    let spinner_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| {
            (entity.kind == EntityKind::CrystalStaticSpinner).then_some(index)
        })
        .collect();
    for (spinner_index, entity_index) in spinner_indices.into_iter().enumerate() {
        let mut state = p.spinners[spinner_index].clone();
        let in_view = spinner_in_view(state.position, p.camera);
        if !state.visible {
            state.collidable = false;
            if in_view {
                state.visible = true;
            }
        } else {
            if scene_on_interval(p.scene_time_active, 0.25, state.offset) && !in_view {
                state.visible = false;
            }
            if scene_on_interval(p.scene_time_active, 0.05, state.offset) {
                state.collidable = (p.pos.x - state.position.x).abs() < 128.0
                    && (p.pos.y - state.position.y).abs() < 128.0;
            }
        }
        sync_spinner_entity(&mut map.entities[entity_index], &state);
        p.spinners[spinner_index] = state;
    }
}

fn falling_block_entity_indices(map: &Map) -> Vec<usize> {
    map.entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::FallingBlock).then_some(index))
        .collect()
}

fn falling_block_player_fall_check(
    p: &PlayerSnapshot,
    entity: &crate::Entity,
    climb_fall: bool,
) -> bool {
    // PlayerFallCheck: climbFall blocks trigger on HasPlayerRider; ordinary
    // blocks trigger on HasPlayerOnTop (CollideFirst<Player>(Position - UnitY)).
    if climb_fall {
        player_riding_solid(p, entity.bounds)
    } else {
        player_on_top_of_solid(p, entity.bounds)
    }
}

fn falling_block_player_wait_check(
    p: &PlayerSnapshot,
    entity: &crate::Entity,
    climb_fall: bool,
    triggered: bool,
) -> bool {
    // PlayerWaitCheck keeps the pre-drop timer from counting down while the
    // player is on top. climbFall blocks additionally wait while the player
    // overlaps either side by one pixel (CollideCheck<Player>(Position +/- UnitX)).
    if triggered {
        return true;
    }
    if falling_block_player_fall_check(p, entity, climb_fall) {
        return true;
    }
    if climb_fall {
        let player = current_player_rect(p, p.pos.x, p.pos.y);
        return Rect::new(
            entity.bounds.x - 1.0,
            entity.bounds.y,
            entity.bounds.width,
            entity.bounds.height,
        )
        .intersects(player)
            || Rect::new(
                entity.bounds.x + 1.0,
                entity.bounds.y,
                entity.bounds.width,
                entity.bounds.height,
            )
            .intersects(player);
    }
    false
}

fn falling_block_platform_below(map: &Map, entity_index: usize, below: Rect) -> bool {
    // CollideCheck<Platform>(Position + (0,1)): static tiles plus every
    // Platform entity (JumpThrus, Clouds, and Solid entities) below the block.
    map.static_solid_at(below)
        || map.entities.iter().enumerate().any(|(index, entity)| {
            index != entity_index
                && matches!(
                    entity.kind,
                    EntityKind::JumpThru
                        | EntityKind::Cloud
                        | EntityKind::BounceBlock
                        | EntityKind::CassetteBlock
                        | EntityKind::DreamBlock
                        | EntityKind::FallingBlock
                        | EntityKind::MoveBlock
                        | EntityKind::MovingSolid
                        | EntityKind::ZipMover
                        | EntityKind::TempleGate
                        | EntityKind::ExitBlock
                        | EntityKind::InvisibleBarrier
                )
                && entity.bounds.intersects(below)
        })
}

/// Move a FallingBlock down `amount` pixels, stopping before the first solid
/// or (downward) jumpthru pixel. Riders are carried exactly like
/// Solid.MoveVExact; the returned bool is true when the move was blocked.
fn move_falling_block_down(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    entity_index: usize,
    state: &mut crate::FallingBlockSnapshot,
    amount: i32,
) -> bool {
    if amount <= 0 {
        return false;
    }
    let env = solid_collision_env(map, entity_index);
    let bounds = map.entities[entity_index].bounds;
    let mut moved = 0i32;
    while moved < amount {
        let candidate = Rect::new(
            bounds.x,
            bounds.y + (moved + 1) as f32,
            bounds.width,
            bounds.height,
        );
        if env_solid_at(&env, candidate, None)
            || env_jump_thru_at(&env, candidate, bounds.y + bounds.height + moved as f32)
        {
            break;
        }
        moved += 1;
    }
    if moved > 0 {
        let entity = &mut map.entities[entity_index];
        // MoveVCollideSolids writes LiftSpeed.Y = moveV / DeltaTime for the
        // full requested move before the exact pass truncates it at impact.
        move_runtime_solid_exact(
            p,
            &mut entity.bounds,
            &env,
            false,
            moved as f32,
            Vec2::new(0.0, state.fall_speed),
        );
        state.position.y += moved as f32;
    }
    moved < amount
}

fn advance_falling_blocks(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    attachments: &[Option<StaticMoverAttachment>],
) {
    for (block_index, entity_index) in falling_block_entity_indices(map).into_iter().enumerate() {
        let mut state = p.falling_blocks[block_index].clone();
        if state.removed {
            p.falling_blocks[block_index] = state;
            sync_platform_static_movers(map, attachments, entity_index, false);
            continue;
        }
        let climb_fall = map.entities[entity_index].direction.x != 0.0;
        match state.phase {
            // Wait for Triggered or the player fall check, then the FallDelay.
            0 => {
                if state.triggered
                    || falling_block_player_fall_check(p, &map.entities[entity_index], climb_fall)
                {
                    if state.fall_delay > 0.0 {
                        state.fall_delay = (state.fall_delay - p.frame_delta_time).max(0.0);
                    }
                    if state.fall_delay <= 0.0 {
                        state.fall_delay = 0.0;
                        state.phase = 1;
                        state.shake_timer = FALLING_BLOCK_SHAKE_TIME;
                    }
                }
            }
            // ShakeSfx + StartShaking, then yield return 0.2f.
            1 => {
                state.shake_timer -= p.frame_delta_time;
                if state.shake_timer <= 0.0 {
                    state.phase = 2;
                    state.wait_timer = FALLING_BLOCK_WAIT_TIME;
                    // The loop-head condition runs on the same resume frame as
                    // the float-yield completion, before the timer is ever
                    // decremented. A player who leaves during the shake
                    // therefore ends the wait immediately.
                    if !falling_block_player_wait_check(
                        p,
                        &map.entities[entity_index],
                        climb_fall,
                        state.triggered,
                    ) {
                        state.phase = 3;
                        state.fall_speed = 0.0;
                    }
                }
            }
            // while (timer > 0 && PlayerWaitCheck()) { timer -= dt; }
            2 => {
                // Each later resume decrements first, then re-checks the
                // condition; the drop starts on the frame the timer reaches
                // zero or the player leaves.
                if state.wait_timer > 0.0 {
                    state.wait_timer -= p.frame_delta_time;
                }
                if state.wait_timer <= 0.0
                    || !falling_block_player_wait_check(
                        p,
                        &map.entities[entity_index],
                        climb_fall,
                        state.triggered,
                    )
                {
                    state.phase = 3;
                    state.fall_speed = 0.0;
                }
            }
            // Fall: approach 160 px/s at 500 px/s^2, moving through MoveV.
            3 => {
                state.fall_speed = approach(
                    state.fall_speed,
                    FALLING_BLOCK_MAX_SPEED,
                    FALLING_BLOCK_ACCEL * p.frame_delta_time,
                );
                state.remainder_y += state.fall_speed * p.frame_delta_time;
                let exact = state.remainder_y.round_ties_even() as i32;
                state.remainder_y -= exact as f32;
                if move_falling_block_down(p, map, entity_index, &mut state, exact) {
                    state.phase = 4;
                    state.shake_timer = FALLING_BLOCK_IMPACT_TIME;
                } else {
                    let bounds = map.entities[entity_index].bounds;
                    let below = Rect::new(bounds.x, bounds.y + 1.0, bounds.width, bounds.height);
                    let fell_out = bounds.y > map.bounds.bottom() + 16.0
                        || (bounds.y > map.bounds.bottom() - 1.0
                            && runtime_solid_collision(map, entity_index, below));
                    if fell_out {
                        state.collidable = false;
                        state.removed = true;
                    }
                }
            }
            // Impact: land shake, then rest permanently on tiles or wait on a
            // platform before falling again.
            4 => {
                state.shake_timer -= p.frame_delta_time;
                if state.shake_timer <= 0.0 {
                    let bounds = map.entities[entity_index].bounds;
                    let below = Rect::new(bounds.x, bounds.y + 1.0, bounds.width, bounds.height);
                    if map.static_solid_at(below) {
                        state.phase = 5;
                        state.safe = true;
                    } else if falling_block_platform_below(map, entity_index, below) {
                        state.phase = 6;
                        state.wait_timer = FALLING_BLOCK_PLATFORM_TICK;
                    } else {
                        state.phase = 1;
                        state.shake_timer = FALLING_BLOCK_SHAKE_TIME;
                    }
                }
            }
            // Landed permanently; Safe = true and the coroutine ends.
            5 => {}
            // while (CollideCheck<Platform>(Position + (0,1))) { yield 0.1f; }
            6 => {
                state.wait_timer -= p.frame_delta_time;
                if state.wait_timer <= 0.0 {
                    let bounds = map.entities[entity_index].bounds;
                    let below = Rect::new(bounds.x, bounds.y + 1.0, bounds.width, bounds.height);
                    if falling_block_platform_below(map, entity_index, below) {
                        state.wait_timer = FALLING_BLOCK_PLATFORM_TICK;
                    } else {
                        state.phase = 1;
                        state.shake_timer = FALLING_BLOCK_SHAKE_TIME;
                    }
                }
            }
            _ => {
                state.phase = 0;
            }
        }
        let enabled = !state.removed;
        p.falling_blocks[block_index] = state;
        sync_platform_static_movers(map, attachments, entity_index, enabled);
    }
}
fn advance_post_player_entities(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    input: InputState,
    attachments: &[Option<StaticMoverAttachment>],
) {
    // Booster.BoostRoutine runs after Player.Update. It retains
    // BoostingPlayer throughout Dash/RedDash, then releases the player and
    // starts the one-second respawn timer on the first later state.
    if p.booster_boosting && !matches!(p.state, PlayerState::Dash | PlayerState::RedDash) {
        p.booster_boosting = false;
        p.booster_reuse_timer = p.booster_reuse_timer.max(1.0);
    }
    advance_zip_movers(p, map, attachments);
    // `CoreModeToggle` has depth 2000 (`CoreModeToggle.cs:50`), so its own `Update` - the only
    // place its cooldown moves - runs after `Player.Update` in the same frame.
    advance_core_mode_toggles(p);
    advance_bounce_blocks(p, map);
    advance_move_blocks(p, map, input, attachments);
    advance_falling_blocks(p, map, attachments);
    advance_exit_blocks(p, map);
    advance_theo_crystals(p, map);
    advance_heart_gems(p);
    advance_rising_lavas(p, map);
    advance_sandwich_lavas(p, map);
    advance_gliders(p, map);
    advance_clouds(p, map);
    advance_seekers(p, map);
    advance_puffers(p, map);
    advance_temple_gates(p, map);
    // CassetteBlockManager writes Activated after Player.Update. The block
    // itself runs before the next Player update so its reform MoveV records
    // LiftSpeed for that frame's grounded StarFly Jump.
    advance_cassette_manager(p, map, attachments);
    advance_spinners(p, map);
}

fn step(
    p: &mut PlayerSnapshot,
    mut input: InputState,
    map: &mut Map,
    attachments: &mut Vec<Option<StaticMoverAttachment>>,
    climb_hop_solid: &mut Option<ClimbHopSolid>,
    crumble_blocks: &mut Vec<CrumbleBlockState>,
    room: &mut RoomCoroutineState,

) -> Result<(), SimulationError> {
    // Engine computes DeltaTime once at the beginning of the raw frame. A
    // HeartGem can write TimeRate during Scene.Update, but that write only
    // changes movement and timers from the next engine frame onward.
    let raw_delta_time = input
        .frame_delta_time_bits
        .map(f32::from_bits)
        .filter(|delta| delta.is_finite() && *delta > 0.0)
        .unwrap_or(DT);
    p.frame_delta_time = raw_delta_time * p.time_rate;
    // Collector installs the portable jump edge as Input.MenuCancel.Pressed
    // as well as Input.Jump. Preserve the former before Player state code can
    // consume the latter's VirtualButton buffer.
    let menu_cancel_pressed = input.jump_pressed;
    if input.presses_are_effective {
        // The caller already supplies `VirtualButton.Pressed` itself - the level,
        // not a raw edge - including the zeroing `ConsumeBuffer`/`ConsumePress`
        // perform (`VirtualButton.cs:148-157`). Running it through the buffers
        // below a second time would re-arm them on every frame the game's press is
        // merely still alive, so the press would outlive the game's window. Adopt
        // the level as this frame's press instead; `jump_held` stays the raw
        // `VirtualButton.Check`, which `Player.cs:2952` and `2963` read for
        // half-gravity and variable-jump.
        p.jump_buffer_timer = if input.jump_pressed {
            JUMP_BUFFER_TIME
        } else {
            0.0
        };
        p.dash_buffer_timer = if input.dash_pressed { 0.08 } else { 0.0 };
        p.crouch_dash_buffer_timer = if input.crouch_dash_pressed { 0.08 } else { 0.0 };
    } else {
        // VirtualButton.Update runs in MInput before Celeste.Freeze can skip the
        // Scene. It subtracts DeltaTime first, then a new press restores the full
        // buffer; Jump also clears its buffer as soon as the binding is not held.
        p.jump_buffer_timer -= p.frame_delta_time;
        if input.jump_pressed {
            p.jump_buffer_timer = JUMP_BUFFER_TIME;
        } else if !input.jump_held {
            p.jump_buffer_timer = 0.0;
        }
        // Dash and CrouchDash use a 0.08 second VirtualButton buffer. Their
        // portable input contract only records press edges, so keep the existing
        // press buffer alive across freeze until it is consumed or expires.
        p.dash_buffer_timer = (p.dash_buffer_timer - p.frame_delta_time).max(0.0);
        p.crouch_dash_buffer_timer = (p.crouch_dash_buffer_timer - p.frame_delta_time).max(0.0);
        if input.dash_pressed {
            p.dash_buffer_timer = 0.08;
        }
        if input.crouch_dash_pressed {
            p.crouch_dash_buffer_timer = 0.08;
        }
    }
    // Player state callbacks read VirtualButton.Pressed, not the raw edge.
    input.jump_pressed = p.jump_buffer_timer > 0.0;
    input.dash_pressed = p.dash_buffer_timer > 0.0;
    input.crouch_dash_pressed = p.crouch_dash_buffer_timer > 0.0;
    if p.dead {
        if p.respawn_frames <= 1 {
            p.dead = false;
            p.death_freeze_pending = false;
            p.respawn_frames = 0;
            p.freeze_timer = 0.0;
            p.pos = map.spawn;
            p.speed = Vec2::default();
            p.state = PlayerState::IntroRespawn;
            // Player.IntroRespawnBegin creates a 0.6-second tween whose
            // OnComplete changes StateMachine.State back to StNormal
            // (Player.cs:6131-6144). The tween's `Timer` starts at zero and is
            // advanced by `Tween.Update` on each following frame.
            p.intro_phase = INTRO_PHASE_RESPAWN;
            p.intro_timer = 0.0;
            p.intro_phase_ready = true;
            p.on_ground = grounded(p, map);
            p.player_on_ground = p.on_ground;
            p.player_on_ground_initialized = true;
            // Player.Added: `lastDashes = (Dashes = MaxDashes);` (Player.cs).
            refill_dash(p);
            p.stamina = 110.0;
            p.movement_remainder = Vec2::default();
            p.just_respawned = true;
            return Ok(());
        }
        p.respawn_frames -= 1;
        p.on_ground = false;
        p.player_on_ground = false;
        if p.death_freeze_pending {
            p.death_freeze_pending = false;
            p.freeze_timer = 0.05;
            return Ok(());
        }
        if p.freeze_timer > 0.0 {
            p.freeze_timer = (p.freeze_timer - raw_delta_time).max(0.0);
        }
        return Ok(());
    }
    // Monocle.Engine still counts frames during a Celeste.Freeze, but skips
    // Scene.Update entirely. Freeze uses RawDeltaTime and therefore must be
    // part of the portable snapshot for engine-frame-accurate replay.
    if p.freeze_timer > 0.0 {
        p.freeze_timer = (p.freeze_timer - DT).max(0.0);
        return Ok(());
    }
    if p.just_respawned && p.speed != Vec2::default() {
        p.just_respawned = false;
    }
    p.scene_time_active += raw_delta_time;
    advance_moving_solids(p, map, attachments);
    // InvisibleBarrier has TransitionUpdate and room entities update before
    // Player on an ordinary frame. Its only active Update is therefore run
    // before either Player.Update or the transition coroutine moves Player.
    advance_invisible_barriers(p, map);
    advance_crush_and_dash_blocks(p, map);
    // `TempleGate`'s depth is -9000 (`TempleGate.cs:67`), so its own `Update` - the alarms, the
    // `HoldingTheo` proximity toggle and the `drawHeight` animation - runs before `Player.Update`,
    // and a gate it opens this frame is already passable for this frame's movement.
    //
    // `Level.Update` only runs `base.Update()` - every entity that is not `Tags.TransitionUpdate`,
    // and `TempleGate` is not - in the `else if (!Transitioning)` arm (`Level.cs:1869-1896`), so
    // the alarm must not tick on a transition frame either. The death, `Celeste.Freeze` and
    // `BadelineBoost`/`IntroRespawn` frame shapes bail out above this point and therefore do not
    // advance it; those are frames the trace cannot replay as an ordinary `Player.Update` anyway.
    if p.transition_timer <= 0.0 {
        advance_temple_gate_alarms(p, map, room);
    }
    if p.transition_timer > 0.0 {
        update_transition(p, map);
        advance_sandwich_lavas(p, map);
        advance_cassette_blocks(p, map, attachments);
        advance_cassette_manager(p, map, attachments);
        advance_spinners(p, map);
        // `Actor.OnGround()` is a live collision probe. A screen transition
        // pauses Player.Update, but it does not make a player standing on a
        // floor geometrically airborne for snapshot capture.
        p.on_ground = grounded(p, map);
        return Ok(());
    }
    // Killbox has no TransitionUpdate tag. On ordinary frames its room-entity
    // Update runs before Player.Update and applies the source 32 px hysteresis
    // thresholds before its PlayerCollider can fire later in the frame.
    advance_killboxes(p, map);
    advance_badeline_boost_relocation(p);
    // WindController is updated before Player in the level entity order. It
    // advances and invokes WindMover using the state from the start of the
    // frame; WindTrigger interaction below selects the target for next frame.
    advance_wind_controller(p);
    apply_wind_movement(p, map);
    advance_cassette_blocks(p, map, attachments);
    // Player.Update checks the force-move timer before subtracting DeltaTime,
    // and keeps the forced direction for that whole frame even when the
    // subtraction reaches zero.
    let force_move_x_active = p.force_move_x_timer > 0.0;
    // Dash refill uses an if/else in Player.Update: a timer that was positive
    // at frame start is only decremented, even if it crosses zero.
    let dash_refill_cooldown_active = p.dash_refill_cooldown_timer > 0.0;
    if p.explode_launch_boost_timer > 0.0 {
        // Player.cs:1467 compares `Input.MoveX.Value` against
        // `Math.Sign(explodeLaunchBoostSpeed)`. `Math.Sign(0f)` is 0, so a
        // boosted speed that has been zeroed does not match either direction;
        // `f32::signum(0.0)` is +1 and would match a rightward stick, clearing
        // the timer and overwriting `Speed.X` with zero.
        if input.move_x as f32 == math_sign(p.explode_launch_boost_speed) {
            p.speed.x = p.explode_launch_boost_speed;
            p.explode_launch_boost_timer = 0.0;
        } else {
            p.explode_launch_boost_timer -= p.frame_delta_time;
        }
    }
    tick_timers(p);
    if p.wall_slide_dir != 0 {
        p.wall_slide_timer = (p.wall_slide_timer - p.frame_delta_time).max(0.0);
    }
    p.wall_slide_dir = 0;
    // Player.cs:1560-1570 consumes the wall boost BEFORE the `onGround` block
    // at Player.cs:1571-1576 resets Stamina. Landing on the exact boost frame
    // must therefore end at 110, not 137.5: the previous order let the +27.5
    // refund survive the ground reset.
    update_wall_boost(p);
    if p.strawberry_collect_reset_timer > 0.0 {
        p.strawberry_collect_reset_timer =
            (p.strawberry_collect_reset_timer - p.frame_delta_time).max(0.0);
        if p.strawberry_collect_reset_timer <= 0.0 {
            p.strawberry_collect_index = 0;
        }
    } else {
        p.strawberry_collect_index = 0;
    }
    let was_on_ground = p.player_on_ground;
    // Player.Update only probes the platform below while Speed.Y >= 0.
    // Upward motion is airborne even when the player starts flush with a
    // floor, so NormalUpdate must apply gravity on that same frame.
    p.player_on_ground = p.state != PlayerState::DreamDash && p.speed.y >= 0.0 && grounded(p, map);
    p.on_ground = p.player_on_ground;
    if p.on_ground {
        p.auto_jump = false;
        p.jump_grace_timer = JUMP_GRACE;
        p.wall_slide_timer = WALL_SLIDE_TIME;
        if !dash_refill_cooldown_active && !p.no_refills {
            // Player.Update: `else if (!Inventory.NoRefills) { ... onGround &&
            // CollideCheck<Solid, NegaBlock>(Position + UnitY) ... -> RefillDash(); }`
            // (Player.cs). Excludes the state-3 branch handled below.
            refill_dash(p);
        }
        p.stamina = 110.0;
    }
    if p.state == PlayerState::Swim && !dash_refill_cooldown_active && !p.no_refills {
        // Player.Update: `if (StateMachine.State == 3) RefillDash();` (Player.cs).
        refill_dash(p);
    }
    p.move_x = if force_move_x_active {
        p.force_move_x
    } else {
        input.move_x
    };
    // Player.cs:1642-1652. `Player.ClimbHop` may have stored the Solid it
    // grabbed; while the force-move window is still open the player is carried
    // by that Solid's whole-pixel movement through `MoveHExact`/`MoveVExact`.
    // Those bypass `movementCounter` (they only clear the moving axis when a
    // pixel is blocked, `Actor.cs:220`/`Actor.cs:249`), so this is a raw
    // `Entity.Position` write and the trace shows it as a position delta with a
    // bit-unchanged `movementCounter`. The reference is dropped as soon as the
    // window closes (`Player.cs:1640`) or the Solid stops being collidable
    // (`Player.cs:1642`).
    if force_move_x_active {
        if let Some(carry) = *climb_hop_solid {
            let entity = &map.entities[carry.entity];
            if solid_is_collidable(entity) {
                let position = Vec2::new(entity.bounds.x, entity.bounds.y);
                *climb_hop_solid = Some(ClimbHopSolid {
                    entity: carry.entity,
                    position,
                });
                let delta = Vec2::new(position.x - carry.position.x, position.y - carry.position.y);
                if delta.x != 0.0 {
                    move_h_exact(p, map, delta.x as i32);
                }
                if delta.y != 0.0 {
                    move_v_exact(p, map, delta.y as i32);
                }
            } else {
                *climb_hop_solid = None;
            }
        }
    } else {
        *climb_hop_solid = None;
    }
    if p.move_x != 0
        && player_in_control(p.state)
        && !matches!(
            p.state,
            PlayerState::Climb
                // StDummy's callback does not turn the player. Lookout reads
                // the same horizontal input as camera aim while retaining
                // the facing it set during DummyWalkToExact.
                | PlayerState::Dummy
                | PlayerState::Pickup
                | PlayerState::RedDash
                | PlayerState::HitSquash
        )
    {
        p.facing = p.move_x > 0;
    }
    p.last_aim = input_aim(input, p.facing);

    // Player.Update resolves both dashless-tech timers before the state
    // callback. A neutral climb jump can therefore become a wallboost before
    // NormalUpdate accelerates it, and retained wall speed is restored before
    // the same callback applies air control.
    update_wall_speed_retention(p, map);
    update_climb_hop_wait(p, map);
    prepare_lookout_player(p);
    ascend_manager_takeover(p, map);
    // The Lookout and Booster are room entities which update before Player.
    // During DummyWalkToExact, its Booster PlayerCollider therefore observes
    // the preceding player position, unlike ordinary player-side callbacks.
    let lookout_booster_box = p
        .lookouts
        .iter()
        .any(|lookout| lookout.interacting && !lookout.removed && lookout.phase == 1)
        .then(|| current_player_rect(p, p.pos.x, p.pos.y));

    if p.badeline_boost_active {
        update_badeline_boost(p, map);
        advance_floaty_blocks(p, map, &mut room.floaty_blocks);
        advance_crumble_blocks(p, map, crumble_blocks);
        advance_switch_gates(p, map, room);
    advance_post_player_entities(p, map, input, attachments);
        p.on_ground = grounded(p, map);
        return Ok(());
    }

    // The `Player.Intro*` states carry no update callback: their whole
    // behaviour is the `StateMachine` coroutine, and a trace row does not
    // export the `Monocle.Coroutine` stack. Rebuild the phase once, from the
    // exported snapshot, before the state runs.
    if is_intro_state(p.state) && !p.intro_phase_ready {
        intro_resume(p, map);
    }

    let was_pickup = p.state == PlayerState::Pickup;
    let was_dream_dash = p.state == PlayerState::DreamDash;
    let was_normal = p.state == PlayerState::Normal;
    match p.state {
        PlayerState::Normal => normal_update(p, input, map, was_on_ground),
        PlayerState::Dash => dash_update(p, input, map),
        PlayerState::Climb => climb_update(p, input, map, climb_hop_solid),
        PlayerState::Swim => swim_update(p, input, map),
        PlayerState::Boost => boost_update(p, input, map),
        PlayerState::RedDash => red_dash_update(p, input, map),
        PlayerState::HitSquash => hit_squash_update(p),
        PlayerState::Pickup => pickup_update(p, input),
        PlayerState::Launch => launch_update(p, input, map),
        PlayerState::DreamDash => dream_dash_update(p),
        PlayerState::SummitLaunch => summit_launch_update(p, map),
        PlayerState::StarFly => star_fly_update(p, input, map),
        PlayerState::Dummy => dummy_update(p, input, map),
        PlayerState::Frozen => {}
        PlayerState::TempleFall => temple_fall_update(p, map),
        PlayerState::ReflectionFall => reflection_fall_update(p, map),
        // Player.cs:5969-5993 / 5995-6068 / 6112-6119 / 6156-6174. These
        // callbacks only mutate Position/Speed/state; the shared
        // JumpThru-Assist / MoveH / MoveV tail below still runs for them,
        // exactly like the source's post-`base.Update()` pass.
        PlayerState::IntroWalk => intro_walk_update(p, map),
        PlayerState::IntroJump => intro_jump_update(p, map),
        PlayerState::IntroWakeUp => intro_wake_up_update(p),
        PlayerState::IntroThinkForABit => intro_think_for_a_bit_update(p, map),
        PlayerState::IntroRespawn => {
            // Player.cs:6121-6146. `Player.Update` still runs its ordinary
            // tail for this state; only the tween drives it.
            intro_respawn_update(p);
            advance_floaty_blocks(p, map, &mut room.floaty_blocks);
            advance_crumble_blocks(p, map, crumble_blocks);
        advance_switch_gates(p, map, room);
    advance_post_player_entities(p, map, input, attachments);
            p.on_ground = grounded(p, map);
            return Ok(());
        }
        other => return Err(SimulationError::UnsupportedState(other)),
    }

    // StateMachine runs the previous state's End before the new state's Begin
    // as soon as the callback assigns State. Player.NormalEnd
    // (Player.cs:3536-3541) therefore clears the wall-boost and wall-speed
    // retention windows on the very frame the player leaves StNormal - a
    // retained speed must not survive into a dash and fire several frames later.
    if was_normal && p.state != PlayerState::Normal {
        normal_end(p);
    }

    // StateMachine.Update applies DreamDashUpdate's returned state
    // immediately, including DreamDashEnd, before Player.Update performs its
    // collider restoration and ordinary MoveH/MoveV pass. Keeping this here
    // preserves the horizontal-exit jump grace and therefore the duck
    // collider for the exit frame.
    if was_dream_dash {
        try_end_dream_dash(p, map, input);
    }

    // PickupCoroutine observes the tween before Tween.Update. The pickup
    // frame only creates it; following Pickup frames decrement it here, and
    // the coroutine restores speed on the frame after it becomes inactive.
    if was_pickup && p.state == PlayerState::Pickup {
        p.pickup_timer -= p.frame_delta_time;
    }

    // Actor.Update runs after the StateMachine component callback. Moving
    // platforms have already written currentLiftSpeed before Player.Update;
    // actions above can consume it, then Actor clears current and advances the
    // retained 0.16-second grace window before player movement.
    tick_lift_speed(p);

    // After components/coroutines update but before movement, Player.Update
    // restores the normal collider while falling in open air only after
    // jumpGraceTimer expires and CanUnDuck succeeds. DashBegin's downward
    // crouch therefore survives while the source coyote window is active.
    if (p.ducking || p.star_fly_hitbox_preserved)
        && p.speed.y > 0.0
        && !p.on_ground
        && p.jump_grace_timer <= 0.0
        && can_unduck(p, map)
    {
        p.ducking = false;
        p.star_fly_hitbox_preserved = false;
    }

    // Player.Update applies JumpThru Assist after its state callback and
    // before the ordinary MoveH/MoveV physics pass.  It is a separate Actor
    // MoveV, so its fractional displacement must share movement_remainder.y
    // with the following Speed.Y movement.  This is what lets an upward jump
    // clear a JumpThru by three pixels in one frame.
    if !p.on_ground
        && p.speed.y <= 0.0
        && (p.state != PlayerState::Climb || p.last_climb_move == -1)
        && touching_jump_thru(p, map)
        && !jump_thru_boost_blocked_check(p, map)
    {
        move_axis_amount(p, map, false, JUMP_THRU_ASSIST_SPEED * p.frame_delta_time);
    }

    // Player.cs:1791-1794 sits between JumpThru Assist and the ordinary
    // MoveH/MoveV pass: a DashAttacking player whose DashDir is exactly
    // horizontal snaps down onto a Solid or JumpThru within three pixels,
    // unless the hurtbox would land on a LedgeBlocker (a "DashCorrect"
    // corner). MoveVExact is an exact move, so a blocked pixel clears
    // movementCounter.Y instead of leaving a fraction for this frame.
    if !p.on_ground && p.dash_dir.y == 0.0 && dash_attacking(p) {
        let ahead = current_player_rect(p, p.pos.x, p.pos.y + DASH_CORRECT_DISTANCE);
        if (map.solid_at(ahead) || jump_thru_outside(p, map, DASH_CORRECT_DISTANCE))
            && !dash_correct_check(p, map, DASH_CORRECT_DISTANCE)
        {
            move_v_exact(p, map, DASH_CORRECT_DISTANCE as i32);
        }
    }

    if p.state != PlayerState::DreamDash {
        move_axis(p, map, true);
    }
    if p.state != PlayerState::DreamDash {
        move_axis(p, map, false);
    }
    update_camera(p, map);
    // Bumper.Update advances its SineWave and updates Position before its
    // PlayerCollider invokes OnPlayer. Keep it immediately before the
    // portable collider callbacks, after Player has completed its movement.
    advance_bumpers(p, map);
    advance_refills(p, map);
    interact(p, map, input, lookout_booster_box);
    try_begin_lookout(p, map, input);
    advance_lookouts(p, map, input, menu_cancel_pressed);
    update_strawberry_train(p);
    try_begin_badeline_boost(p, map);
    enforce_level_bounds(p, map, attachments, room);
    // The map loader adds Player before vanilla room entities, so ZipMover's
    // coroutine and Solid carry/push run after Player.Update. A lift speed
    // written by the previous ZipMover update is therefore visible to the
    // player's next action before the platform advances again.
    advance_floaty_blocks(p, map, &mut room.floaty_blocks);
    advance_crumble_blocks(p, map, crumble_blocks);
    advance_switch_gates(p, map, room);
    advance_post_player_entities(p, map, input, attachments);
    p.on_ground = grounded(p, map);
    Ok(())
}

/// `Player.RefillDash()` (`Player.cs`):
///
/// ```csharp
/// public bool RefillDash() {
///     if (Dashes < MaxDashes) { Dashes = MaxDashes; return true; }
///     return false;
/// }
/// ```
///
/// `MaxDashes` is `Inventory.Dashes` for the live session (`Player.cs`
/// `MaxDashes`, `PlayerInventory.cs`): 0 in the Prologue, 2 in The Summit,
/// Core and the Epilogue, 1 elsewhere. Restoring a hardcoded single dash
/// desynchronised every 0-Intro room (the sim granted a dash the Prologue
/// never has) and every The Summit/Core room (the sim stopped at 1 where the
/// game restored 2).
fn refill_dash(p: &mut PlayerSnapshot) {
    if p.dashes < p.max_dashes {
        p.dashes = p.max_dashes;
    }
}

/// `AscendManager.Routine` (`AscendManager.cs:249-273`), the Summit's ascent takeover: the entity
/// waits while `player.Y > base.Y` and then runs `player.Speed = Vector2.Zero;
/// player.StateMachine.State = 11; player.DummyGravity = false;` in its own update. Both are
/// entities at depth 0, and the takeover lands one frame after the player's `Y` first equals the
/// manager's: measured on `7-Summit|0|b-09|110556`, where the player's row `Y` reaches -4059 at row
/// 111325 and the state flips at 111326. Running this before the state dispatch, with the
/// frame-start position, reproduces that exactly; `index == 9`'s 1.6-second delay
/// (`AscendManager.cs:257-260`) is not modelled, and no room in the corpus uses it.
fn ascend_manager_takeover(p: &mut PlayerSnapshot, map: &Map) {
    for entity in map
        .entities
        .iter()
        .filter(|entity| entity.kind == EntityKind::SummitBackgroundManager)
    {
        if p.pos.y <= entity.bounds.y {
            p.speed = Vec2::default();
            p.state = PlayerState::Dummy;
            p.dummy_gravity = false;
        }
    }
}

fn tick_timers(p: &mut PlayerSnapshot) {
    if p.auto_jump_timer > 0.0 {
        if p.auto_jump {
            p.auto_jump_timer = (p.auto_jump_timer - p.frame_delta_time).max(0.0);
            if p.auto_jump_timer <= 0.0 {
                p.auto_jump = false;
            }
        } else {
            p.auto_jump_timer = 0.0;
        }
    }
    for timer in [
        &mut p.dash_attack_timer,
        &mut p.dash_cooldown_timer,
        &mut p.dash_refill_cooldown_timer,
        &mut p.booster_reuse_timer,
        &mut p.feather_reuse_timer,
        &mut p.bumper_reuse_timer,
        &mut p.min_hold_timer,
        &mut p.no_wind_timer,
        &mut p.jump_grace_timer,
        &mut p.bounce_reuse_timer,
        &mut p.var_jump_timer,
        &mut p.force_move_x_timer,
        &mut p.climb_no_move_timer,
        &mut p.dream_dash_can_end_timer,
    ] {
        *timer = (*timer - p.frame_delta_time).max(0.0);
    }
}

fn lift_speed(p: &PlayerSnapshot) -> Vec2 {
    if p.current_lift_speed == Vec2::default() {
        p.last_lift_speed
    } else {
        p.current_lift_speed
    }
}

fn lift_boost(p: &PlayerSnapshot) -> Vec2 {
    let speed = lift_speed(p);
    Vec2::new(speed.x.clamp(-250.0, 250.0), speed.y.clamp(-130.0, 0.0))
}

fn add_lift_boost(p: &mut PlayerSnapshot) {
    let boost = lift_boost(p);
    p.speed.x += boost.x;
    p.speed.y += boost.y;
}

fn tick_lift_speed(p: &mut PlayerSnapshot) {
    p.current_lift_speed = Vec2::default();
    if p.lift_speed_timer > 0.0 {
        p.lift_speed_timer -= p.frame_delta_time;
        if p.lift_speed_timer <= 0.0 {
            p.lift_speed_timer = 0.0;
            p.last_lift_speed = Vec2::default();
        }
    }
}

fn update_wall_boost(p: &mut PlayerSnapshot) {
    if p.wall_boost_timer <= 0.0 {
        return;
    }
    p.wall_boost_timer = (p.wall_boost_timer - p.frame_delta_time).max(0.0);
    if p.move_x == p.wall_boost_dir {
        p.speed.x = WALL_JUMP_H * p.move_x as f32;
        p.stamina += 27.5;
        p.wall_boost_timer = 0.0;
    }
}

fn update_wall_speed_retention(p: &mut PlayerSnapshot, map: &Map) {
    if p.wall_speed_retention_timer <= 0.0 {
        return;
    }
    // Player.cs:1669 compares `Math.Sign(Speed.X)` against
    // `-Math.Sign(wallSpeedRetained)`. `Math.Sign(0f)` is 0, so a Speed.X that
    // the wall collision zeroed on the previous frame (Player.cs:3219) is NOT
    // "moving away from the retained direction": the retained speed is restored
    // instead (Player.cs:1673-1676). Using `f32::signum` here treated the zero
    // speed as +1 and cancelled the retention on every wall-jump chain, leaving
    // the player with pure air-acceleration speed.
    if math_sign(p.speed.x) == -math_sign(p.wall_speed_retained) {
        p.wall_speed_retention_timer = 0.0;
    } else if !map.solid_at(current_player_rect(
        p,
        p.pos.x + math_sign(p.wall_speed_retained),
        p.pos.y,
    )) {
        p.speed.x = p.wall_speed_retained;
        p.wall_speed_retention_timer = 0.0;
    } else {
        p.wall_speed_retention_timer = (p.wall_speed_retention_timer - p.frame_delta_time).max(0.0);
    }
}

fn normal_update(p: &mut PlayerSnapshot, input: InputState, map: &Map, was_on_ground: bool) {
    let boost = lift_boost(p);
    if boost.y < 0.0 && was_on_ground && !p.on_ground && p.speed.y >= 0.0 {
        p.speed.y = boost.y;
    }
    if !holding_holdable(p) {
        // Player.cs:3572-3582: `Input.GrabCheck && !IsTired && !Ducking`.
        // `IsTired` is `CheckStamina < 20f`, and `CheckStamina`
        // (`Player.cs:1048-1060`) adds the pending 27.5 wall-boost refund, so the
        // raw `Stamina` field is the wrong quantity here.
        if input.grab_held
            && check_stamina(p) >= CLIMB_TIRED_THRESHOLD
            && !p.ducking
            && try_pickup_holdable(p)
        {
            return;
        }
    } else if !input.grab_held && p.min_hold_timer <= 0.0 {
        release_holdable(p, input);
    }
    if !holding_holdable(p)
        && (input.dash_pressed || input.crouch_dash_pressed)
        && p.dashes > 0
        && p.dash_cooldown_timer <= 0.0
    {
        add_lift_boost(p);
        begin_dash(p, input, true, false, map);
        return;
    }

    let facing_dir = if p.facing { 1 } else { -1 };
    if !holding_holdable(p)
        && input.grab_held
        && !p.ducking
        && p.speed.y >= 0.0
        // Player.cs uses Math.Sign, where both signed zero values return 0.
        // Rust f32::signum returns +1/-1 for +/-0, which made a stationary
        // left-facing player on a wall's right side look like they were
        // moving away from it and prevented the grab.
        && (p.speed.x == 0.0 || p.speed.x.signum() != -(facing_dir as f32))
        && check_stamina(p) >= CLIMB_TIRED_THRESHOLD
    {
        let climb_y = if climb_check(p, map, facing_dir, 0.0) {
            Some(0.0)
        } else if input.move_y < 1 && p.wind.y <= 0.0 {
            (1..=CLIMB_UP_CHECK_DIST).find_map(|offset| {
                let y_add = -(offset as f32);
                (!map.solid_at(current_player_rect(p, p.pos.x, p.pos.y + y_add))
                    && climb_check(p, map, facing_dir, y_add))
                .then_some(y_add)
            })
        } else {
            None
        };
        if let Some(y_add) = climb_y {
            // Player.NormalUpdate uses MoveVExact for this one/two pixel
            // upward correction. Keep the sub-pixel movement remainder intact.
            p.pos.y += y_add;
            // StateMachine invokes NormalEnd before ClimbBegin. Entering the
            // actual grab state therefore destroys any pending cornerboost speed.
            p.wall_speed_retention_timer = 0.0;
            p.wall_boost_timer = 0.0;
            p.hop_wait_x = 0;
            p.state = PlayerState::Climb;
            p.auto_jump = false;
            p.speed.x = 0.0;
            p.speed.y *= 0.2;
            p.wall_slide_timer = WALL_SLIDE_TIME;
            p.climb_no_move_timer = 0.1;
            p.wall_boost_timer = 0.0;
            // ClimbBegin closes a one-pixel gap when ClimbCheck found the wall at
            // its full two-pixel probe distance. MoveHExact leaves the sub-pixel
            // movement counter unchanged.
            for _ in 0..CLIMB_CHECK_DIST as u8 {
                if map.solid_at(current_player_rect(p, p.pos.x + facing_dir as f32, p.pos.y)) {
                    break;
                }
                p.pos.x += facing_dir as f32;
            }
            return;
        }
    }

    // Player.NormalUpdate changes the active collider before applying run
    // friction. This ordering is what lets a crouched player enter a booster
    // with the six-pixel hitbox (Archie) on the same frame.
    //
    // The duck-entry test differs between the two halves of the source block.
    // With empty hands it is the plain grounded/`MoveY == 1`/`Speed.Y >= 0`
    // test (`Player.cs:3640`); with a holdable in hand the same test is
    // additionally gated on `!holdCannotDuck` (`Player.cs:3652`), and the flag
    // is cleared only once the player is grounded and stops holding down
    // (`Player.cs:3669-3671`). `holdCannotDuck` is restored from
    // `p.holdCannotDuck` and maintained below, so a holdable-carrying player who
    // already ducked once cannot re-enter the duck hitbox by tapping down again.
    if p.ducking {
        if p.on_ground && input.move_y != 1 && can_unduck(p, map) {
            p.ducking = false;
        }
    } else if p.on_ground && input.move_y == 1 && p.speed.y >= 0.0 {
        if !(holding_holdable(p) && p.hold_cannot_duck) {
            p.ducking = true;
        }
    }
    if p.on_ground && input.move_y != 1 && p.hold_cannot_duck {
        // Player.cs:3669-3671.
        p.hold_cannot_duck = false;
    }

    let mult = if p.on_ground {
        // Player.cs:3681-3684: the Core's ice mode scales the whole ground
        // run multiplier by 0.3, which shrinks both `Calc.Approach` steps
        // (400f and 1000f per second) to 30% of their normal size.
        if p.core_mode == crate::CoreMode::Cold {
            ICE_GROUND_MULT
        } else {
            1.0
        }
    } else if holding_slow_fall(p) {
        AIR_MULT * 0.5
    } else {
        AIR_MULT
    };
    let move_x = p.move_x;
    if p.ducking && p.on_ground {
        p.speed.x = approach(p.speed.x, 0.0, DUCK_FRICTION * p.frame_delta_time);
    } else {
        let max_run = if p.holding_theo.is_some() {
            70.0
        } else if holding_slow_fall(p) && !p.on_ground {
            108.000_008
        } else {
            MAX_RUN
        };
        // `Player.cs:2889-2890`: `level.InSpace` scales the run *target*, unlike the Core's
        // ice factor, which scales the acceleration multiplier instead.
        let max_run = if p.in_space {
            max_run * SPACE_PHYSICS_MULT
        } else {
            max_run
        };
        let target = move_x as f32 * max_run;
        let same_direction_over_max = move_x != 0
            && p.speed.x.abs() > max_run
            && p.speed.x.signum() == (move_x as f32).signum();
        p.speed.x = approach(
            p.speed.x,
            target,
            if same_direction_over_max {
                RUN_REDUCE
            } else {
                RUN_ACCEL
            } * mult
                * p.frame_delta_time,
        );
    }
    if move_x != 0 {
        p.facing = move_x > 0;
    }

    // `Player.cs:2904-2908`: the space multiplier scales both fall caps *before* the
    // fast-fall comparison, so it also moves the speed at which the fast cap is selected.
    let max_fall_cap = if p.in_space {
        MAX_FALL * SPACE_PHYSICS_MULT
    } else {
        MAX_FALL
    };
    let fast_max_fall_cap = if p.in_space {
        FAST_MAX_FALL * SPACE_PHYSICS_MULT
    } else {
        FAST_MAX_FALL
    };
    let target_max_fall = if holding_slow_fall(p) && p.force_move_x_timer <= 0.0 {
        if input.move_y > 0 { 120.0 } else { 40.0 }
    } else if input.move_y > 0 && p.speed.y >= max_fall_cap {
        fast_max_fall_cap
    } else {
        max_fall_cap
    };
    p.max_fall = approach(
        p.max_fall,
        target_max_fall,
        FAST_MAX_ACCEL * p.frame_delta_time,
    );
    let mut fall_target = p.max_fall;
    // Player.cs:3742-3748. `NormalUpdate`'s `!onGround` block mirrors the down
    // input into `holdCannotDuck` whenever a slow-fall holdable is in hand, so
    // the flag tracks "the player has been falling with down held" and only the
    // grounded release at `Player.cs:3669-3671` clears it.
    if !p.on_ground && holding_slow_fall(p) {
        p.hold_cannot_duck = input.move_y == 1;
    }
    // Player.cs:3749-3771. The whole wall-slide block is gated on the
    // force-move-adjusted `moveX` pointing into Facing (or a neutral moveX while
    // Grab is held) and on `Input.MoveY != 1` - holding down (a fast-fall)
    // suppresses both the `wallSlideDir` assignment and the
    // `Lerp(160, 20, wallSlideTimer / 1.2)` fall target, so a down-held fall
    // beside a wall keeps the ordinary `maxFall` target instead of decelerating
    // toward the 20 px/s wall-slide cap (W3 `586011d`). Inside that gate the
    // assignment itself additionally needs a live `wallSlideTimer`, the
    // Facing-side solid, a clear ClimbBlocker edge and `CanUnDuck`. The probe
    // follows Facing - not whichever side happens to carry a wall.
    if !p.on_ground
        && !holding_holdable(p)
        && (p.move_x == facing_dir || (p.move_x == 0 && input.grab_held))
        && input.move_y != 1
    {
        if p.speed.y >= 0.0
            && p.wall_slide_timer > 0.0
            && can_unduck(p, map)
            && wall_slide_at(p, map, facing_dir)
        {
            p.ducking = false;
            p.wall_slide_dir = facing_dir;
        }
        if p.wall_slide_dir != 0 {
            // Player.cs:3762-3765: a wall next to a ClimbBlocker only ever
            // reaches the slowest quarter of the slide ramp.
            if p.wall_slide_timer > 0.6
                && climb_blocker_check(p, map, p.wall_slide_dir as f32, 0.0)
            {
                p.wall_slide_timer = 0.6;
            }
            fall_target = WALL_SLIDE_START_MAX
                + (MAX_FALL - WALL_SLIDE_START_MAX) * (1.0 - p.wall_slide_timer / WALL_SLIDE_TIME);
        }
    }
    let mut gravity_mult =
        if (input.jump_held || p.auto_jump) && p.speed.y.abs() < HALF_GRAV_THRESHOLD {
            0.5
        } else {
            1.0
        };
    if holding_slow_fall(p) && p.force_move_x_timer <= 0.0 {
        gravity_mult *= 0.5;
    }
    // `Player.cs:2954-2955`: applied after the slow-fall halving and only ever read inside
    // the `!onGround` block below, exactly as the source nests it.
    if p.in_space {
        gravity_mult *= SPACE_PHYSICS_MULT;
    }
    if !p.on_ground {
        p.speed.y = approach(
            p.speed.y,
            fall_target,
            GRAVITY * gravity_mult * p.frame_delta_time,
        );
    }
    if p.var_jump_timer > 0.0 {
        if input.jump_held || p.auto_jump {
            p.speed.y = p.speed.y.min(p.var_jump_speed);
        } else {
            p.var_jump_timer = 0.0;
        }
    }

    // Player.NormalUpdate performs run acceleration and gravity before it
    // handles the buffered jump press. This order is visible on the first
    // wall-jump frame: WallJump's -130 speed must not be accelerated yet.
    if p.jump_buffer_timer > 0.0 {
        if p.jump_grace_timer > 0.0 {
            p.jump_buffer_timer = 0.0;
            p.jump_grace_timer = 0.0;
            p.speed.y = JUMP_SPEED;
            p.speed.x += p.move_x as f32 * JUMP_H_BOOST;
            add_lift_boost(p);
            p.auto_jump = false;
            p.dash_attack_timer = 0.0;
            p.wall_slide_timer = WALL_SLIDE_TIME;
            p.wall_boost_timer = 0.0;
            p.var_jump_speed = p.speed.y;
            p.var_jump_timer = VAR_JUMP_TIME;
        } else if can_unduck(p, map) {
            // `Player.cs:2969-3004`: the whole wall-jump / water-jump branch of
            // `NormalUpdate` sits inside `else if (CanUnDuck)`. A crouched player
            // whose normal hitbox does not fit where it stands cannot stand up, so
            // `CanUnDuck` is false and the press is *swallowed*: no `WallJump`,
            // no `ClimbJump`, no water `Jump`, and `Ducking` stays true. Without
            // this gate the simulator ate the same press into a `WallJump`
            // (`Ducking = false` plus `WallJumpHSpeed`/`JumpSpeed`), which is a
            // one-frame divergence whose first visible effect is a wrong speed.
            let jump_wall = if wall_jump_check(p, map, 1) {
                1
            } else if wall_jump_check(p, map, -1) {
                -1
            } else {
                0
            };
            if jump_wall == 0 {
                if map.water_at(current_player_rect(p, p.pos.x, p.pos.y + 2.0)) {
                    p.jump_buffer_timer = 0.0;
                    p.jump_grace_timer = 0.0;
                    p.speed.y = JUMP_SPEED;
                    p.speed.x += p.move_x as f32 * JUMP_H_BOOST;
                    add_lift_boost(p);
                    p.auto_jump = false;
                    p.dash_attack_timer = 0.0;
                    p.wall_slide_timer = WALL_SLIDE_TIME;
                    p.wall_boost_timer = 0.0;
                    p.var_jump_speed = p.speed.y;
                    p.var_jump_timer = VAR_JUMP_TIME;
                }
                return;
            }
            if !holding_holdable(p)
                && input.grab_held
                && p.stamina > 0.0
                && facing_dir == jump_wall
                // Player.cs:3807 / 3822: a blocking ClimbBlocker 3 px into the
                // wall downgrades the ClimbJump to an ordinary WallJump.
                && !climb_blocker_check(p, map, 3.0 * jump_wall as f32, 0.0)
            {
                climb_jump(p, jump_wall);
            } else if dash_attacking(p) && super_wall_jump_angle_check(p) {
                // Player.cs:3811 / 3826: `DashAttacking && SuperWallJumpAngleCheck`;
                // DashAttacking lasts 0.3 s, twice the ordinary Dash state's
                // 0.15 s duration, so this also fires after the dash returns
                // to Normal.
                super_wall_jump(p, -jump_wall);
            } else {
                // Player.cs:3817 / 3832: WallJump(-1) launches to the left when
                // WallJumpCheck(1) found the wall on the right.
                wall_jump(p, -jump_wall);
            }
        }
    }
}

fn begin_dash(
    p: &mut PlayerSnapshot,
    input: InputState,
    consume_dash: bool,
    delayed_coroutine: bool,
    map: &Map,
) {
    p.dash_buffer_timer = 0.0;
    p.crouch_dash_buffer_timer = 0.0;
    // DashBegin clears DashDir. DashCoroutine does not sample lastAim until it
    // resumes after its initial yield (and any Celeste.Freeze frames).
    p.dash_dir = Vec2::default();
    p.before_dash_speed = p.speed;
    p.demo_dashed = input.crouch_dash_pressed;
    p.dash_started_on_ground = p.on_ground;
    p.dash_end_pending = false;
    p.speed = Vec2::default();
    p.state = PlayerState::Dash;
    // Player.cs DashCoroutine yields once before applying dash speed.
    p.state_timer = DASH_TIME + p.frame_delta_time * if delayed_coroutine { 2.0 } else { 1.0 };
    p.dash_attack_timer = DASH_ATTACK_TIME;
    p.dash_cooldown_timer = DASH_COOLDOWN;
    p.dash_refill_cooldown_timer = 0.1;
    p.freeze_timer = DASH_FREEZE_TIME;
    if consume_dash {
        p.dashes = p.dashes.saturating_sub(1);
    }
    if !p.on_ground && p.ducking && can_unduck(p, map) {
        p.ducking = false;
    } else if !p.ducking && (p.demo_dashed || input.move_y > 0) {
        p.ducking = true;
    }
}

/// The buffered-jump block that every `Player.DashUpdate` frame runs
/// (`Player.cs:4393-4441`). Returns `true` when the jump was consumed and the
/// state machine must return to `StNormal` (state 0).
///
/// * `|DashDir.Y| < 0.1` with coyote time is a `SuperJump` (`Player.cs:4393-4397`).
/// * `SuperWallJumpAngleCheck` (`|DashDir.X| <= 0.2 && DashDir.Y <= -0.75`,
///   `Player.cs:1092-1102`) turns a wall jump into a `SuperWallJump`
///   (`Player.cs:4399-4414`).
/// * Otherwise `WallJumpCheck(1)`/`WallJumpCheck(-1)` launches a `ClimbJump`
///   when Grab is held while Facing that wall, and a plain `WallJump` in every
///   other case (`Player.cs:4415-4441`).
///
/// The simulator previously only modelled the two super variants, so a climb
/// jump or wall jump buffered during a dash never fired: no 130/170 launch and
/// no 27.5 stamina cost.
fn dash_jump(p: &mut PlayerSnapshot, input: InputState, map: &Map) -> bool {
    if !input.jump_pressed || !can_unduck(p, map) {
        return false;
    }
    if p.dash_dir.y.abs() < 0.1 && p.jump_grace_timer > 0.0 {
        super_jump(p);
        enter_normal(p);
        return true;
    }
    let jump_wall = if wall_jump_check(p, map, 1) {
        1
    } else if wall_jump_check(p, map, -1) {
        -1
    } else {
        0
    };
    if p.dash_dir.x.abs() <= 0.2 && p.dash_dir.y <= -0.75 {
        // Player.cs:4403-4412 requires a WallJumpCheck before SuperWallJump.
        if jump_wall != 0 {
            super_wall_jump(p, -jump_wall);
            enter_normal(p);
            return true;
        }
        return false;
    }
    if jump_wall == 0 {
        return false;
    }
    let facing_dir: i8 = if p.facing { 1 } else { -1 };
    if !holding_holdable(p)
        && input.grab_held
        && p.stamina > 0.0
        && facing_dir == jump_wall
        // Player.cs:4419 / 4431: the 3 px ClimbBlocker probe in front of
        // Facing downgrades the ClimbJump to an ordinary WallJump.
        && !climb_blocker_check(p, map, 3.0 * jump_wall as f32, 0.0)
    {
        // Player.cs:4419-4422 / 4431-4434: Facing into the wall plus Grab is a
        // ClimbJump, which charges 27.5 stamina (`Player.cs:2646-2651`).
        climb_jump(p, jump_wall);
    } else {
        wall_jump(p, -jump_wall);
    }
    // Player.DashUpdate returns state 0 (`StNormal`) from every jump branch
    // (Player.cs:4396/4406/4411/4427/4439), and the setter only runs
    // `begins[0]` when the state actually changes - so every one of these
    // branches also carries NormalBegin's `maxFall = 160f`
    // (Player.cs:3531-3534), which is what `enter_normal` applies.
    enter_normal(p);
    true
}

fn dash_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    // StateMachine starts DashCoroutine beside DashUpdate. Its initial yield
    // occupies the first unfrozen DashUpdate; when that yield resumes, the
    // coroutine publishes DashDir/Speed after this callback. Holdable.Check
    // must therefore wait until the following DashUpdate, where it sees the
    // live dash velocity (rather than cancelling an up-dash at zero speed).
    let dash_coroutine_initial_yield =
        p.dash_dir == Vec2::default() && p.state_timer > DASH_TIME + p.frame_delta_time * 0.5;
    if !dash_coroutine_initial_yield
        && !holding_holdable(p)
        && input.grab_held
        // Player.cs:4374: `Holding == null && DashDir != Zero && Input.GrabCheck
        // && !IsTired && CanUnDuck`. `IsTired` reads `CheckStamina`, which
        // includes the pending wall-boost refund (`Player.cs:1048-1060`).
        && check_stamina(p) >= CLIMB_TIRED_THRESHOLD
        && can_unduck(p, map)
        && try_pickup_holdable(p)
    {
        return;
    }
    // DashUpdate runs while DashCoroutine is still parked at its initial
    // `yield return null`. At this point Player.cs still has DashDir == Zero,
    // so a jump buffered on the frame immediately after DashBegin is a
    // SuperJump before lastAim is sampled. Ducking was already selected by
    // DashBegin from MoveY, which makes the same window an instant Hyper.
    // `Monocle.StateMachine.Update` calls the state callback before it resumes
    // the coroutine (`Monocle/StateMachine.cs:168-183`), so every branch below
    // reads the DashDir the source callback would see - still Zero on the frame
    // the coroutine publishes (W1 `8aa41da`).
    // Player.cs:4384-4392 closes the player onto a JumpThru it already overlaps
    // (any overhang up to six pixels) with an exact move, before the dash jump
    // branches below.
    if p.dash_dir.y.abs() < 0.1 {
        close_dash_onto_jump_thru(p, map);
    }
    // Player.cs:4393-4441 is the whole buffered-jump block of DashUpdate: a
    // SuperJump on a horizontal dash with coyote time (4393-4397), a
    // SuperWallJump while SuperWallJumpAngleCheck holds (4399-4414), and
    // otherwise a ClimbJump (grab held, facing the wall, stamina left, no
    // blocking ClimbBlocker 3 px into it) or a plain WallJump (4415-4441).
    // Every branch returns 0, so `dash_jump` finishes with `enter_normal`,
    // which also applies NormalBegin's `maxFall = 160f` (Player.cs:3531-3534).
    if dash_jump(p, input, map) {
        return;
    }
    p.state_timer = (p.state_timer - p.frame_delta_time).max(0.0);
    if (p.state_timer - DASH_TIME).abs() <= p.frame_delta_time * 0.5 {
        p.dash_dir = p.last_aim;
        // `Player.DashCoroutine` (`Player.cs:4491-4493`): once `DashDir` is published from the aim,
        // `if (DashDir.X != 0f) Facing = (Facings)Math.Sign(DashDir.X);`. The red-dash path already
        // does this; the ordinary dash path did not, so the facing landed a frame late.
        if p.dash_dir.x != 0.0 {
            p.facing = p.dash_dir.x > 0.0;
        }
        p.speed = Vec2::new(p.dash_dir.x * DASH_SPEED, p.dash_dir.y * DASH_SPEED);
        // C# Math.Sign(0f) is 0, unlike Rust f32::signum(), which produces
        // +1 for zero. A vertical dash must therefore not retain pre-dash
        // rightward speed as though its zero horizontal launch were rightward.
        if p.before_dash_speed.x.signum() == p.speed.x.signum()
            && p.speed.x != 0.0
            && p.before_dash_speed.x.abs() > p.speed.x.abs()
        {
            p.speed.x = p.before_dash_speed.x;
        }
        if map.water_at(player_rect(p.pos.x, p.pos.y)) {
            p.speed.x *= SWIM_DASH_SPEED_MULT;
            p.speed.y *= SWIM_DASH_SPEED_MULT;
        }
        // Player.cs keeps a grounded down-diagonal dash diagonal when the
        // current duck collider has an enterable DreamBlock one pixel below.
        // Ordinary ground still converts the dash into the 1.2x horizontal
        // grounded-ultra launch.
        let dream_block_below =
            p.can_dream_dash && map.dream_block_at(current_player_rect(p, p.pos.x, p.pos.y + 1.0));
        if p.on_ground
            && p.dash_dir.x != 0.0
            && p.dash_dir.y > 0.0
            && p.speed.y > 0.0
            && !dream_block_below
        {
            p.dash_dir.x = p.dash_dir.x.signum();
            p.dash_dir.y = 0.0;
            p.speed.y = 0.0;
            p.speed.x *= 1.2;
            p.ducking = true;
        }
        // DashUpdate has already run for this frame (`Monocle/StateMachine.cs:171-183`
        // runs the state callback before the coroutine), so the freshly published
        // DashDir/Speed must not be re-tested against the jump branches here: the
        // next frame's DashUpdate sees them through the block at the top of this
        // function. W1 `8aa41da` duplicated the SuperJump/SuperWallJump checks at
        // this point; those are removed because they fired a frame early.
        return;
    }
    if p.state_timer > 0.0 {
        return;
    }
    if !p.dash_end_pending {
        p.dash_end_pending = true;
        return;
    }
    p.dash_end_pending = false;
    // DashCoroutine ends with `StateMachine.State = 0` (Player.cs:4566). The
    // StateMachine setter runs `begins[0]` = `Player.NormalBegin`, which resets
    // `maxFall = 160f` (Player.cs:3531-3534). Entering Normal with the cap the
    // dash inherited leaves the fast-fall ladder alive: a down-diagonal dash
    // that returns to Normal while `maxFall > 160` then keeps falling past 160
    // instead of clamping to the single `300f * Engine.DeltaTime` step that
    // NormalUpdate's `Input.MoveY == 1 && Speed.Y >= num4` branch adds
    // (Player.cs:3727-3729).
    enter_normal(p);
    p.auto_jump = true;
    if p.dash_dir.y <= 0.0 {
        p.speed.x = p.dash_dir.x * END_DASH_SPEED;
        p.speed.y = p.dash_dir.y * END_DASH_SPEED;
        if p.speed.y < 0.0 {
            p.speed.y *= 0.75;
        }
    }
}

impl PlayerSnapshot {
    /// Rebuild the simulator's dash clock (`state_timer`) from the source
    /// `Player.dashAttackTimer` when a replay anchors in the middle of a dash.
    ///
    /// Celeste has no generic state timer: `Monocle.StateMachine` only carries
    /// the current state id, and the dash is timed entirely by
    /// `Player.DashCoroutine` (`Player.cs:4465-4567`). It starts with
    /// `yield return null`, so the second `Player.Update` after `DashBegin`
    /// publishes `Speed = speed` / `DashDir` (`Player.cs:4479-4489`), and
    /// `yield return 0.15f` (`Player.cs:4551`) plus one more frame later it
    /// writes `Speed = DashDir * 160f` and `StateMachine.State = 0`
    /// (`Player.cs:4556-4566`). `DashBegin` sets `dashAttackTimer = 0.3f`
    /// (`Player.cs:4296`) and `Player.Update` decrements it once per unfrozen
    /// frame (`Player.cs:1577-1580`), so `dashAttackTimer` is an exact clock for
    /// the number of dash frames already played. `state_timer` is the same clock
    /// read as time remaining: `DASH_TIME + frame_delta_time` immediately after
    /// `begin_dash`, one `frame_delta_time` less per following dash frame, which
    /// puts the publish step exactly on `DASH_TIME` and zero exactly nine frames
    /// later - the frame before the coroutine returns the player to StNormal.
    ///
    /// The `DashBegin` frame itself is the only anchor where `dashAttackTimer`
    /// still holds the `0.3f` it was just assigned; `DashBegin`'s
    /// `Celeste.Freeze(0.05f)` (`Player.cs:4282-4285`) is therefore still
    /// pending, because `Monocle.Engine` drains `FreezeTimer` at the start of the
    /// *next* frame. Freeze frames do not run `Player.Update`, so they do not
    /// advance either clock and the relation above is unaffected by them.
    pub fn restore_dash_phase(&mut self, frame_delta_time: f32) {
        if self.state != PlayerState::Dash || !(frame_delta_time > 0.0) {
            return;
        }
        let frames = ((DASH_ATTACK_TIME - self.dash_attack_timer) / frame_delta_time)
            .round()
            .max(0.0);
        self.state_timer = (DASH_TIME + frame_delta_time - frames * frame_delta_time).max(0.0);
        if frames == 0.0 && self.time_rate > 0.25 {
            self.freeze_timer = DASH_FREEZE_TIME;
        }
    }
}

fn super_jump(p: &mut PlayerSnapshot) {
    p.state = PlayerState::Normal;
    p.jump_buffer_timer = 0.0;
    p.jump_grace_timer = 0.0;
    p.var_jump_timer = VAR_JUMP_TIME;
    p.auto_jump = false;
    p.dash_attack_timer = 0.0;
    p.wall_slide_timer = WALL_SLIDE_TIME;
    p.wall_boost_timer = 0.0;
    p.speed = Vec2::new(
        if p.facing {
            SUPER_JUMP_H
        } else {
            -SUPER_JUMP_H
        },
        JUMP_SPEED,
    );
    add_lift_boost(p);
    if p.ducking {
        p.ducking = false;
        p.speed.x *= 1.25;
        p.speed.y *= 0.5;
    }
    p.var_jump_speed = p.speed.y;
    p.launched = true;
}

/// `Player.SuperWallJump(int dir)` (`Player.cs:2607-2622`).
fn super_wall_jump(p: &mut PlayerSnapshot, dir: i8) {
    p.state = PlayerState::Normal;
    p.ducking = false;
    p.jump_buffer_timer = 0.0;
    p.jump_grace_timer = 0.0;
    p.var_jump_timer = 0.25;
    p.auto_jump = false;
    p.dash_attack_timer = 0.0;
    p.wall_slide_timer = WALL_SLIDE_TIME;
    p.wall_boost_timer = 0.0;
    p.speed = Vec2::new(170.0 * dir as f32, -160.0);
    add_lift_boost(p);
    p.var_jump_speed = p.speed.y;
    p.launched = true;
}

/// The state machine's transition into `StNormal`, i.e. `StateMachine.State = 0`
/// while another state is current: the setter runs `begins[0]`, which is
/// `Player.NormalBegin` (`Player.cs:1146`) and resets `maxFall = 160f`
/// (`Player.cs:3531-3534`). Any transition into Normal from a different state
/// must go through here; a `return 0` from `NormalUpdate` itself must not,
/// because the setter early-returns when `state == value`
/// (`Monocle/StateMachine.cs:36-39`).
fn enter_normal(p: &mut PlayerSnapshot) {
    p.state = PlayerState::Normal;
    p.max_fall = MAX_FALL;
}

/// `Player.DashUpdate`'s JumpThru close (`Player.cs:4384-4392`): while the dash
/// is horizontal, every JumpThru the player already overlaps that is no more
/// than six pixels below the player's bottom pulls the player onto its top with
/// `MoveVExact((int)(entity.Top - base.Bottom))`. Like every `MoveVExact` this
/// is an exact move: it steps whole pixels and only clears
/// `movementCounter.Y` when a pixel is blocked. (W1 `8aa41da`.)
fn close_dash_onto_jump_thru(p: &mut PlayerSnapshot, map: &Map) {
    for entity in &map.entities {
        if !matches!(entity.kind, EntityKind::JumpThru | EntityKind::Cloud) {
            continue;
        }
        let bottom = current_player_rect(p, p.pos.x, p.pos.y).bottom();
        if !entity
            .bounds
            .intersects(current_player_rect(p, p.pos.x, p.pos.y))
            || bottom - entity.bounds.y > 6.0
        {
            continue;
        }
        let amount = (entity.bounds.y - bottom) as i32;
        if amount != 0 && !dash_correct_check(p, map, amount as f32) {
            move_v_exact(p, map, amount);
        }
    }
}

/// `Player.WallJump(dir)` (`Player.cs:2548-2582`): the plain wall jump used by
/// `NormalUpdate` (`Player.cs:3817`, `3832`), `ClimbUpdate` (`Player.cs:3937`)
/// and `DashUpdate` (`Player.cs:4425`, `4437`).
///
/// One definition only: W1 `8aa41da` landed the `Ducking = false` and
/// `jumpGraceTimer = 0` resets, and this keeps them plus the two pieces the
/// source has beyond them - `Holding.SlowFall`'s 0.26 s forceMoveX window
/// (`Player.cs:2560-2564`) and `LaunchedBoostCheck` (`Player.cs:2582`).
fn wall_jump(p: &mut PlayerSnapshot, dir: i8) {
    p.ducking = false; // Player.cs:2550
    p.jump_buffer_timer = 0.0; // Player.cs:2551 Input.Jump.ConsumeBuffer()
    p.jump_grace_timer = 0.0; // Player.cs:2552
    p.var_jump_timer = VAR_JUMP_TIME; // Player.cs:2553 (0.2f)
    p.auto_jump = false; // Player.cs:2554
    p.dash_attack_timer = 0.0; // Player.cs:2555
    p.wall_slide_timer = WALL_SLIDE_TIME; // Player.cs:2557
    p.wall_boost_timer = 0.0; // Player.cs:2558
    if holding_slow_fall(p) {
        p.force_move_x = dir;
        p.force_move_x_timer = GLIDER_WALL_JUMP_FORCE_TIME;
    } else if p.move_x != 0 {
        p.force_move_x = dir;
        p.force_move_x_timer = WALL_JUMP_FORCE_TIME;
    }
    p.speed.x = WALL_JUMP_H * dir as f32; // Player.cs:2578
    p.speed.y = JUMP_SPEED; // Player.cs:2579
    add_lift_boost(p); // Player.cs:2580
    p.var_jump_speed = p.speed.y; // Player.cs:2581
    // Player.cs:2582 runs LaunchedBoostCheck (`Player.cs:2353-2362`), which
    // assigns `launched` from the post-lift-boost speed.
    let boost = lift_boost(p);
    p.launched = boost.x * boost.x + boost.y * boost.y >= LAUNCHED_BOOST_CHECK_SPEED_SQ
        && p.speed.x * p.speed.x + p.speed.y * p.speed.y >= LAUNCHED_SPEED_SQ;
}

/// `Player.NormalEnd` (`Player.cs:3536-3541`), run by `StateMachine` whenever
/// the player leaves `StNormal`.
fn normal_end(p: &mut PlayerSnapshot) {
    p.wall_boost_timer = 0.0;
    p.wall_speed_retention_timer = 0.0;
    p.hop_wait_x = 0;
}

fn climb_jump(p: &mut PlayerSnapshot, wall: i8) {
    p.jump_buffer_timer = 0.0;
    p.jump_grace_timer = 0.0;
    p.auto_jump = false;
    p.dash_attack_timer = 0.0;
    p.wall_slide_timer = WALL_SLIDE_TIME;
    p.wall_boost_timer = 0.0;
    p.speed.x += p.move_x as f32 * JUMP_H_BOOST;
    p.speed.y = JUMP_SPEED;
    add_lift_boost(p);
    if !p.on_ground {
        // `Player.ClimbJump` (`Player.cs:2644-2651`) subtracts the cost from a bare
        // `Stamina` field with no floor: the game happily records a negative value, and
        // only the `Stamina <= 0` tests downstream act on it.
        p.stamina -= CLIMB_JUMP_COST;
    }
    if p.move_x == 0 {
        p.wall_boost_dir = -wall;
        p.wall_boost_timer = 0.2;
    }
    p.var_jump_speed = p.speed.y;
    p.var_jump_timer = VAR_JUMP_TIME;
}

fn climb_update(
    p: &mut PlayerSnapshot,
    input: InputState,
    map: &Map,
    climb_hop_solid: &mut Option<ClimbHopSolid>,
) {
    let wall = if p.facing { 1 } else { -1 };
    // Player.ClimbUpdate checks jump and dash before letting go or checking
    // whether the one-pixel wall contact still exists. Ceiling pops rely on
    // that ordering for the first Climb frame just below a wall's bottom edge.
    if input.jump_pressed && (!p.ducking || can_unduck(p, map)) {
        enter_normal(p);
        if p.move_x == -wall {
            // Player.cs:3937 `WallJump(0 - Facing)`: the same WallJump as the
            // NormalUpdate branch, including its 0.16 s forceMoveX window.
            wall_jump(p, -wall);
        } else {
            climb_jump(p, wall);
        }
        return;
    }
    if (input.dash_pressed || input.crouch_dash_pressed)
        && p.dashes > 0
        && p.dash_cooldown_timer <= 0.0
    {
        begin_dash(p, input, true, false, map);
        return;
    }
    if !input.grab_held {
        add_lift_boost(p);
        enter_normal(p);
        return;
    }
    if !touching_wall(p, map, wall) {
        if p.speed.y < 0.0 {
            if p.wall_boosting {
                // `Player.cs:3140-3149`: a released conveyor hands its own
                // `LiftSpeed` over instead of running the ledge `ClimbHop`.
                add_lift_boost(p);
            } else {
                climb_hop(p, map, wall, climb_hop_solid);
            }
        }
        enter_normal(p);
        return;
    }
    // `Player.cs:3154-3167`: a facing-side `WallBooster` that the player overlaps
    // takes over the whole vertical block - the ordinary climb target is never
    // computed - and drives `Speed.Y` toward `WallBoosterSpeed` at
    // `WallBoosterAccel`, publishing the same figure as `LiftSpeed` so a later
    // release inherits it.
    if p.climb_no_move_timer <= 0.0 && wall_booster_check(p, map, wall) {
        p.wall_boosting = true;
        p.speed.y = approach(
            p.speed.y,
            WALL_BOOSTER_SPEED,
            WALL_BOOSTER_ACCEL * p.frame_delta_time,
        );
        set_lift_speed(
            p,
            Vec2::new(0.0, p.speed.y.max(WALL_BOOSTER_LIFT_SPEED)),
        );
    } else {
        p.wall_boosting = false;
        let mut target = 0.0;
        let mut try_slip = false;
        if p.climb_no_move_timer <= 0.0 {
            if climb_blocker_check(p, map, wall as f32, 0.0) {
                try_slip = true;
            } else if input.move_y == -1 {
                target = CLIMB_UP_SPEED;
                if map.solid_at(current_player_rect(p, p.pos.x, p.pos.y - 1.0))
                    || (climb_hop_blocked_check(p, map) && slip_check(p, map, -1.0))
                {
                    if p.speed.y < 0.0 {
                        p.speed.y = 0.0;
                    }
                    target = 0.0;
                    try_slip = true;
                } else if slip_check(p, map, 0.0) {
                    climb_hop(p, map, wall, climb_hop_solid);
                    enter_normal(p);
                    return;
                }
            } else if input.move_y == 1 {
                target = CLIMB_DOWN_SPEED;
                if p.on_ground {
                    if p.speed.y > 0.0 {
                        p.speed.y = 0.0;
                    }
                    target = 0.0;
                }
            } else {
                try_slip = true;
            }
        } else {
            try_slip = true;
        }
        // Player.cs:4045 `lastClimbMove = Math.Sign(num)` uses the BCL sign, so a
        // neutral climb (num == 0, or a `flag` branch that only sets `flag`) stores
        // 0 - not Rust's +1. Player.cs:4405 also reads this field back for the
        // JumpThru assist.
        p.last_climb_move = math_sign(target) as i8;
        if try_slip && slip_check(p, map, 0.0) {
            target = CLIMB_SLIP_SPEED;
        }
        p.speed.y = approach(p.speed.y, target, CLIMB_ACCEL * p.frame_delta_time);
    }
    p.speed.x = 0.0;
    if p.climb_no_move_timer <= 0.0 {
        // Player.cs:4058-4079 drains stamina from `lastClimbMove`, not from the
        // post-SlipCheck `num`: a slip (`num = 30f` at Player.cs:4048) still
        // charges the 10/s "still" cost because lastClimbMove is 0.
        let cost = if p.last_climb_move < 0 {
            CLIMB_UP_COST
        } else if p.last_climb_move == 0 {
            CLIMB_STILL_COST
        } else {
            0.0
        };
        // `Player.cs:4060` and `4078` are bare subtractions: `Stamina` is allowed to go
        // negative, and floored it here made the simulator's value disagree with the
        // game's on every frame a climb outlasted the bar.
        p.stamina -= cost * p.frame_delta_time;
    }
    if p.stamina <= 0.0 {
        enter_normal(p);
    }
    if input.move_y != 1
        && p.speed.y > 0.0
        && !map.solid_at(current_player_rect(p, p.pos.x + wall as f32, p.pos.y + 1.0))
    {
        p.speed.y = 0.0;
    }
}

fn swim_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    if !swim_check(p, map) {
        p.state = PlayerState::Normal;
        return;
    }
    if p.ducking {
        p.ducking = false;
    }
    if (input.dash_pressed || input.crouch_dash_pressed)
        && p.dashes > 0
        && p.dash_cooldown_timer <= 0.0
    {
        begin_dash(p, input, false, false, map);
        return;
    }

    let underwater = swim_underwater_check(p, map);
    let mut x = input.move_x as f32;
    let mut y = input.move_y as f32;
    if x != 0.0 && y != 0.0 {
        const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
        x *= DIAG;
        y *= DIAG;
    }
    let horizontal_max = if underwater {
        SWIM_UNDERWATER_MAX
    } else {
        SWIM_MAX
    };
    // Player.cs:4645 / 4659 compare against `Math.Sign(value.X)` and
    // `Math.Sign(value.Y)` of the safe-normalized Feather vector. `SafeNormalize`
    // keeps a purely vertical or purely horizontal stick at an exact zero
    // component, and `Math.Sign(0f)` is 0, so that axis never takes the reduce
    // branch; `f32::signum(0.0)` would report +1 and wrongly reduce a
    // same-sign speed.
    let horizontal_accel = if p.speed.x.abs() > SWIM_MAX && math_sign(p.speed.x) == math_sign(x) {
        SWIM_REDUCE
    } else {
        SWIM_ACCEL
    };
    p.speed.x = approach(
        p.speed.x,
        horizontal_max * x,
        horizontal_accel * p.frame_delta_time,
    );

    if y == 0.0 && swim_rise_check(p, map) {
        p.speed.y = approach(p.speed.y, SWIM_MAX_RISE, SWIM_ACCEL * p.frame_delta_time);
    } else if y >= 0.0 || underwater {
        let vertical_accel = if p.speed.y.abs() > SWIM_MAX && math_sign(p.speed.y) == math_sign(y) {
            SWIM_REDUCE
        } else {
            SWIM_ACCEL
        };
        p.speed.y = approach(p.speed.y, SWIM_MAX * y, vertical_accel * p.frame_delta_time);
    }

    if p.jump_buffer_timer > 0.0 && swim_jump_check(p, map) {
        p.state = PlayerState::Normal;
        p.jump_buffer_timer = 0.0;
        p.jump_grace_timer = 0.0;
        p.var_jump_timer = VAR_JUMP_TIME;
        p.auto_jump = false;
        p.dash_attack_timer = 0.0;
        p.wall_slide_timer = WALL_SLIDE_TIME;
        p.wall_boost_timer = 0.0;
        p.speed.x += input.move_x as f32 * JUMP_H_BOOST;
        p.speed.y = JUMP_SPEED;
        p.var_jump_speed = p.speed.y;
    }
}

fn dream_dash_update(p: &mut PlayerSnapshot) {
    p.on_ground = false;
    naive_move(
        p,
        Vec2::new(
            p.speed.x * p.frame_delta_time,
            p.speed.y * p.frame_delta_time,
        ),
    );
}

/// `Player.DreamDashedIntoSolid` (`Player.cs:5260-5285`).
///
/// `DreamDashUpdate` calls this only when `CollideFirst<DreamBlock>()` is null,
/// i.e. when the dream dash has left the DreamBlock. If the player's collider
/// then overlaps an ordinary `Solid`, the source walks every offset
/// `(i * j, k * l)` for `i, k` in `1..=5` and `j, l` in `{-1, +1}` in that exact
/// nesting order and takes the first one that is free. The first free offset is
/// applied with `Position += vector` - a raw position write, so `movementCounter`
/// is untouched - and `DreamDashedIntoSolid` returns `false`, which lets
/// `DreamDashUpdate` fall through into the ordinary exit branch on the same
/// frame. Returning `true` (no free offset within five pixels) makes the source
/// call `Die(Vector2.Zero)` instead.
enum DreamDashSolidEscape {
    /// Not overlapping a Solid: the source returns `false` immediately.
    Clear,
    /// Overlapping a Solid, and the first free offset was applied.
    Nudged(Vec2),
    /// Overlapping a Solid with no free offset inside five pixels.
    Trapped,
}

fn dream_dash_solid_escape(p: &PlayerSnapshot, map: &Map) -> DreamDashSolidEscape {
    if !map.solid_at(current_player_rect(p, p.pos.x, p.pos.y)) {
        return DreamDashSolidEscape::Clear;
    }
    for i in 1..=5 {
        for j in [-1.0f32, 1.0] {
            for k in 1..=5 {
                for l in [-1.0f32, 1.0] {
                    let offset = Vec2::new(i as f32 * j, k as f32 * l);
                    if !map.solid_at(current_player_rect(
                        p,
                        p.pos.x + offset.x,
                        p.pos.y + offset.y,
                    )) {
                        return DreamDashSolidEscape::Nudged(offset);
                    }
                }
            }
        }
    }
    DreamDashSolidEscape::Trapped
}

fn try_end_dream_dash(p: &mut PlayerSnapshot, map: &Map, input: InputState) {
    if p.state != PlayerState::DreamDash
        || map.dream_block_at(current_player_rect(p, p.pos.x, p.pos.y))
    {
        return;
    }
    // Player.cs:5196-5208 runs the "dashed into a solid" unstuck check before
    // the `dreamDashCanEndTimer <= 0` gate, so a player who left the DreamBlock
    // while the end timer is still running is still pushed out of a solid.
    match dream_dash_solid_escape(p, map) {
        DreamDashSolidEscape::Clear => {}
        DreamDashSolidEscape::Nudged(offset) => {
            p.pos.x += offset.x;
            p.pos.y += offset.y;
        }
        // The source's `Die(Vector2.Zero)` branch is deliberately not modelled:
        // `Player.Die` removes the Player entity from the scene, so every later
        // engine frame is a stale capture the harness cannot replay. Returning
        // here keeps the DreamDash state (the source returns 9 in that case)
        // without inventing a death.
        DreamDashSolidEscape::Trapped => return,
    }
    if p.dream_dash_can_end_timer > 0.0 {
        return;
    }

    let horizontal_exit = p.dash_dir.x != 0.0;
    let dream_jump = input.jump_pressed && horizontal_exit;
    if !dream_jump && (p.dash_dir.y >= 0.0 || horizontal_exit) {
        // Celeste 1.4 pulls a horizontal exit five pixels back toward the
        // DreamBlock before its two-sided ClimbCheck. DreamDash is still the
        // active state here, so this is the source's naive MoveHExact
        // correction rather than normal solid movement.
        if p.dash_dir.x > 0.0 && map.solid_at(current_player_rect(p, p.pos.x - 5.0, p.pos.y)) {
            dream_dash_exit_move_h_exact(p, map, -5);
        } else if p.dash_dir.x < 0.0 && map.solid_at(current_player_rect(p, p.pos.x + 5.0, p.pos.y))
        {
            dream_dash_exit_move_h_exact(p, map, 5);
        }
    }
    let wall = wall_dir(p, map);
    let dream_grab = input.grab_held
        && (p.dash_dir.y >= 0.0 || horizontal_exit)
        && ((input.move_x == 1 && wall == 1) || (input.move_x == -1 && wall == -1));

    if dream_jump {
        // DreamDashUpdate calls Jump before the state transition, then
        // DreamDashEnd restores horizontal-exit grace. That callback ordering
        // is why a buffered Dream Jump can jump a second time.
        enter_normal(p);
        p.jump_buffer_timer = 0.0;
        p.speed.y = JUMP_SPEED;
        p.speed.x += input.move_x as f32 * JUMP_H_BOOST;
        p.auto_jump = false;
        p.dash_attack_timer = 0.0;
        p.wall_slide_timer = WALL_SLIDE_TIME;
        p.wall_boost_timer = 0.0;
        p.var_jump_speed = p.speed.y;
        p.var_jump_timer = VAR_JUMP_TIME;
    } else if dream_grab {
        p.state = PlayerState::Climb;
        p.facing = wall > 0;
        p.auto_jump = false;
        p.speed.x = 0.0;
        p.speed.y *= 0.2;
        p.wall_slide_timer = WALL_SLIDE_TIME;
        p.climb_no_move_timer = 0.1;
        p.wall_boost_timer = 0.0;
    } else {
        // Player.cs:5240 returns state 0, so `StateMachine` runs
        // `DreamDashEnd` and then `NormalBegin` (`Player.cs:1146`), whose
        // `maxFall = 160f` (`Player.cs:3531-3534`) survives the 0.05 s freeze
        // that follows the exit. Assigning `state` directly left the Core's
        // fast-fall 240 px/s cap in place.
        enter_normal(p);
        p.auto_jump = true;
        p.auto_jump_timer = 0.0;
    }
    p.jump_grace_timer = if horizontal_exit { JUMP_GRACE } else { 0.0 };
    // Player.DreamDashEnd: `if (!Inventory.NoRefills) RefillDash();` (Player.cs).
    if !p.no_refills {
        refill_dash(p);
    }
    p.stamina = 110.0;
    p.dash_attack_timer = 0.0;
    p.freeze_timer = 0.05;
}

fn dream_dash_exit_move_h_exact(p: &mut PlayerSnapshot, map: &Map, amount: i32) {
    let sign = amount.signum();
    for _ in 0..amount.unsigned_abs() {
        let next_x = p.pos.x + sign as f32;
        if map.solid_at(current_player_rect(p, next_x, p.pos.y)) {
            // Actor.MoveHExact clears the shared movement counter even when
            // the very first pixel collides. The DreamBlock itself is still a
            // Solid during DreamDashEnd, so this reset affects the ordinary
            // movement rounding later on the same exit frame.
            p.movement_remainder.x = 0.0;
            break;
        }
        p.pos.x = next_x;
    }
}

fn boost_update(p: &mut PlayerSnapshot, input: InputState, map: &mut Map) {
    // StBoostUpdate moves ExactPosition toward boostTarget but does not reset
    // Speed. A concurrently yielded DummyWalkToExact therefore retains its
    // prior Approach result and can publish the next one after this callback.
    let aim = input_vector(input);
    let target = Vec2::new(
        p.boost_target.x + aim.x * 3.0,
        p.boost_target.y + if p.ducking { 3.0 } else { 5.5 } + aim.y * 3.0,
    );
    approach_exact_position(p, map, target, 80.0 * p.frame_delta_time);
    p.state_timer = (p.state_timer - p.frame_delta_time).max(0.0);
    let manual = input.dash_pressed || input.crouch_dash_pressed;
    if !manual && p.state_timer > 0.0 {
        return;
    }
    snap_to_boost_target(p, map);
    // `Player.CallDashEvents` notifies CurrentBooster as the Boost state
    // hands over to Dash. The Booster coroutine then keeps its
    // `BoostingPlayer` guard until that dash has ended.
    p.booster_boosting = true;
    if p.boost_red {
        begin_red_dash(p, manual.then_some(input), !manual);
    } else {
        begin_dash(p, input, false, !manual, map);
    }
}

fn begin_red_dash(p: &mut PlayerSnapshot, input: Option<InputState>, delayed_coroutine: bool) {
    if let Some(input) = input {
        p.demo_dashed = input.crouch_dash_pressed;
        p.last_aim = input_aim(input, p.facing);
    }
    p.state = PlayerState::RedDash;
    p.speed = Vec2::default();
    p.dash_dir = Vec2::default();
    p.dash_started_on_ground = false;
    p.dash_attack_timer = DASH_ATTACK_TIME;
    p.dash_cooldown_timer = DASH_COOLDOWN;
    p.dash_refill_cooldown_timer = 0.1;
    p.freeze_timer = 0.05;
    p.state_timer = p.frame_delta_time * if delayed_coroutine { 2.0 } else { 1.0 };
    p.ducking = false;
}

fn red_dash_update(p: &mut PlayerSnapshot, input: InputState, map: &mut Map) {
    if (input.dash_pressed || input.crouch_dash_pressed)
        && p.dashes > 0
        && p.dash_cooldown_timer <= 0.0
    {
        begin_dash(p, input, true, false, map);
        return;
    }
    if p.dash_dir == Vec2::default() {
        p.state_timer = (p.state_timer - p.frame_delta_time).max(0.0);
        if p.state_timer <= 0.0 {
            p.dash_dir = p.last_aim;
        // `Player.DashCoroutine` (`Player.cs:4491-4493`): once `DashDir` is published from the aim,
        // `if (DashDir.X != 0f) Facing = (Facings)Math.Sign(DashDir.X);`. The red-dash path already
        // does this; the ordinary dash path did not, so the facing landed a frame late.
        if p.dash_dir.x != 0.0 {
            p.facing = p.dash_dir.x > 0.0;
        }
            p.speed = Vec2::new(p.dash_dir.x * DASH_SPEED, p.dash_dir.y * DASH_SPEED);
            if p.dash_dir.x != 0.0 {
                p.facing = p.dash_dir.x > 0.0;
            }
        }
    }
}

fn hit_squash_update(p: &mut PlayerSnapshot) {
    p.speed.x = approach(p.speed.x, 0.0, 800.0 * p.frame_delta_time);
    p.speed.y = approach(p.speed.y, 0.0, 800.0 * p.frame_delta_time);
    if p.state_timer > 0.0 {
        p.state_timer -= p.frame_delta_time;
    } else {
        p.state = PlayerState::Normal;
    }
}

fn launch_update(p: &mut PlayerSnapshot, input: InputState, map: &mut Map) {
    if let Some(target_x) = p.launch_approach_x {
        move_towards_x(p, map, target_x, 60.0 * p.frame_delta_time);
    }
    if (input.dash_pressed || input.crouch_dash_pressed)
        && p.dashes > 0
        && p.dash_cooldown_timer <= 0.0
    {
        begin_dash(p, input, true, false, map);
        return;
    }
    // Player.LaunchUpdate performs the Holdable scan even while already
    // holding an entity. This permits a Bumper launch to restart the pickup
    // tween after its freeze ends when Grab remains held. Player.cs:5017 gates
    // it on `Input.GrabCheck && !IsTired && !Ducking`; `IsTired` is
    // `CheckStamina < 20f` (`Player.cs:1048-1060`).
    if input.grab_held
        && check_stamina(p) >= CLIMB_TIRED_THRESHOLD
        && !p.ducking
        && try_pickup_holdable(p)
    {
        return;
    }
    p.speed.y = approach(
        p.speed.y,
        MAX_FALL,
        GRAVITY * if p.speed.y < 0.0 { 0.5 } else { 0.25 } * p.frame_delta_time,
    );
    p.speed.x = approach(p.speed.x, 0.0, RUN_ACCEL * 0.2 * p.frame_delta_time);
    if length(p.speed) < LAUNCH_CANCEL_THRESHOLD {
        p.state = PlayerState::Normal;
        p.launch_approach_x = None;
    }
}

fn summit_launch_update(p: &mut PlayerSnapshot, map: &mut Map) {
    p.summit_launch_particle_timer -= p.frame_delta_time;
    p.facing = true;
    move_towards_x(p, map, p.summit_launch_target_x, 20.0 * p.frame_delta_time);
    p.speed = Vec2::new(0.0, -DASH_SPEED);
}

fn dummy_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    if can_unduck(p, map) {
        p.ducking = false;
    }
    if !p.on_ground && p.dummy_gravity {
        let mut gravity_mult =
            if p.speed.y.abs() < HALF_GRAV_THRESHOLD && (input.jump_held || p.auto_jump) {
                0.5
            } else {
                1.0
            };
        // `Player.cs`'s `DummyUpdate` repeats the `!onGround` gravity block, `InSpace`
        // multiplier included.
        if p.in_space {
            gravity_mult *= SPACE_PHYSICS_MULT;
        }
        p.speed.y = approach(
            p.speed.y,
            p.max_fall,
            GRAVITY * gravity_mult * p.frame_delta_time,
        );
    }
    if p.var_jump_timer > 0.0 {
        if p.auto_jump || input.jump_held {
            p.speed.y = p.speed.y.min(p.var_jump_speed);
        } else {
            p.var_jump_timer = 0.0;
        }
    }
    if !p.dummy_moving {
        if p.speed.x.abs() > MAX_RUN && p.dummy_maxspeed {
            p.speed.x = approach(
                p.speed.x,
                MAX_RUN * p.speed.x.signum(),
                RUN_ACCEL * 2.5 * p.frame_delta_time,
            );
        }
        if p.dummy_friction {
            p.speed.x = approach(p.speed.x, 0.0, RUN_ACCEL * p.frame_delta_time);
        }
    }
}

fn temple_fall_update(p: &mut PlayerSnapshot, map: &Map) {
    p.facing = true;
    if !p.on_ground {
        let center_x = map.bounds.x + 160.0;
        let move_x = if (center_x - p.pos.x).abs() > 4.0 {
            (center_x - p.pos.x).signum()
        } else {
            0.0
        };
        p.speed.x = approach(
            p.speed.x,
            MAX_RUN * 0.6 * move_x,
            325.0 * p.frame_delta_time,
        );
        if p.dummy_gravity {
            p.speed.y = approach(
                p.speed.y,
                MAX_FALL * 2.0,
                GRAVITY * 0.25 * p.frame_delta_time,
            );
        }
        return;
    }
    if !p.temple_fall_landed {
        p.temple_fall_landed = true;
        p.temple_fall_wait_frames = 1;
        p.speed.x = 0.0;
    } else if p.temple_fall_wait_frames < 60 {
        p.temple_fall_wait_frames += 1;
    } else {
        p.state = PlayerState::Normal;
        p.max_fall = MAX_FALL;
    }
}

fn reflection_fall_update(p: &mut PlayerSnapshot, map: &Map) {
    p.facing = true;
    p.ignore_jump_thrus = true;
    if map.entities.iter().any(|entity| {
        entity.kind == EntityKind::Water
            && entity
                .bounds
                .intersects(current_player_rect(p, p.pos.x, p.pos.y))
    }) {
        p.speed.y = approach(p.speed.y, -20.0, 400.0 * p.frame_delta_time);
    } else {
        p.speed.y = approach(
            p.speed.y,
            MAX_FALL * 2.0,
            GRAVITY * 0.25 * p.frame_delta_time,
        );
    }
    match p.reflection_fall_phase {
        0 if p.reflection_fall_frames < 120 => {
            p.speed.y = 0.0;
            p.reflection_fall_frames += 1;
        }
        0 => {
            p.speed.y = MAX_FALL * 2.0;
            p.reflection_fall_phase = 1;
        }
        1 if map.entities.iter().any(|entity| {
            entity.kind == EntityKind::Water
                && entity
                    .bounds
                    .intersects(current_player_rect(p, p.pos.x, p.pos.y))
        }) =>
        {
            p.reflection_fall_phase = 2;
            p.reflection_fall_wait_timer = 1.2;
        }
        2 if p.reflection_fall_wait_timer > 0.0 => {
            p.reflection_fall_wait_timer -= p.frame_delta_time;
        }
        2 => {
            p.ignore_jump_thrus = false;
            p.state = PlayerState::Normal;
            p.max_fall = MAX_FALL;
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Player.Intro* story states
//
// These five states have no `StateMachine` update callback at all: their whole
// behaviour lives in the coroutine that `StateMachine.SetCallbacks`
// (Player.cs:1158-1171) installs when the state is entered, plus, for
// IntroRespawn, the 0.6 s `Tween` created by `IntroRespawnBegin`. The
// coroutine runs inside `base.Update()` (`StateMachine.Update`,
// Monocle/StateMachine.cs:168-183), i.e. before `Player.Update`'s Unduck /
// JumpThru Assist / MoveH / MoveV tail, so these states still receive the
// ordinary movement pass.
//
// `Player.InControl` (Player.cs:934-954) is false for states 12-15 and 25, so
// no facing flip, camera follow, or `Level.EnforceBounds` runs.
//
// A trace row is a post-`Player.Update` capture, and the fidelity gate anchors
// on the segment's second live row, which for a level load is the state's
// second `StateMachine.Update`. `intro_resume` therefore reconstructs the
// coroutine one update in: every phase timer is primed with one
// `Engine.DeltaTime` already consumed.
// ---------------------------------------------------------------------------

fn is_intro_state(state: PlayerState) -> bool {
    matches!(
        state,
        PlayerState::IntroWalk
            | PlayerState::IntroJump
            | PlayerState::IntroRespawn
            | PlayerState::IntroWakeUp
            | PlayerState::IntroThinkForABit
    )
}

/// `LevelLoader` builds a single map-wide `SolidTiles` grid and, after copying
/// every level's `solids` layer into it, bleeds each level's boundary tiles up
/// to three cells outward:
///
/// ```text
/// value6 = virtualMap2[num12, Bottom - 1];
/// for (num14 = 1; num14 < 4 && !virtualMap3[...]; num14++)
///     virtualMap2[num12, Bottom - 1 + num14] = value6;
/// ```
///
/// (LevelLoader.cs:233-263; the left/right loop covers
/// `TileBounds.Top - 4 .. TileBounds.Bottom + 4`). `SolidTiles` owns a `Grid`
/// collider over that whole map (SolidTiles.cs:18-26), so a player outside the
/// loaded room still collides with these bled cells. The intro states depend on
/// it: `IntroJumpCoroutine` parks the player 16 px below
/// `level.Bounds.Bottom` (Player.cs:6003), where the game's floor bleed keeps
/// them grounded and blocks `Actor.MoveV`.
///
/// `Map::tile_grid` is only populated for decoded rooms, so synthetic maps keep
/// exactly the solids they declare.
/// `ClutterBlock.Colors` (`ClutterBlock.cs:10-15`): Red, Green, Yellow.
const CLUTTER_COLORS: usize = 3;

/// `ClutterSwitch`'s real collider: `base(position, 32f, 16f, safe: true)`
/// (`ClutterSwitch.cs:49-50`). The map `EntityData` only carries the editor's
/// 8x8 default, exactly like `JumpThru`'s height.
const CLUTTER_SWITCH_WIDTH: f32 = 32.0;
const CLUTTER_SWITCH_HEIGHT: f32 = 16.0;

/// The colour of a `ClutterBlockGenerator` map entity, or `None` for every
/// other entity. `Level.LoadLevel` dispatches the three names
/// (`Level.cs:956-967`), and `ClutterBlockGenerator` indexes its `enabled`
/// array with the same enum value (`ClutterBlockGenerator.cs:78-81`).
fn clutter_color(name: &str) -> Option<usize> {
    match name {
        "redBlocks" => Some(0),
        "greenBlocks" => Some(1),
        "yellowBlocks" => Some(2),
        _ => None,
    }
}

/// `ClutterSwitch.Colors` of a `colorSwitch` entity, from the map's `type`
/// attribute: `ClutterSwitch(EntityData data, Vector2 offset)` forwards
/// `data.Enum("type", ClutterBlock.Colors.Green)` (`ClutterSwitch.cs:65-68`).
/// `ClutterBlock.Colors.Lightning` is reported as `Some(CLUTTER_COLORS)`, a
/// switch whose press does not touch any clutter.
fn clutter_switch_color(map: &Map, index: usize) -> Option<usize> {
    let variant = map.entity_visuals.get(index)?.variant.as_deref()?;
    match variant {
        "Red" => Some(0),
        "Green" => Some(1),
        "Yellow" => Some(2),
        "Lightning" => Some(CLUTTER_COLORS),
        _ => None,
    }
}

/// Every `colorSwitch` rectangle in the *present* room together with its
/// decoded colour. `map.entity_visuals` is index-aligned with `map.entities`.
fn clutter_switch_rects(map: &Map) -> Vec<(Rect, Option<usize>)> {
    map.entities
        .iter()
        .enumerate()
        .filter(|(_, entity)| entity.name == "colorSwitch")
        .map(|(index, entity)| (clutter_switch_rect(entity), clutter_switch_color(map, index)))
        .collect()
}

/// `ClutterSwitch : Solid`'s collider rectangle (`ClutterSwitch.cs:49-50`).
fn clutter_switch_rect(entity: &crate::Entity) -> Rect {
    Rect::new(
        entity.bounds.x,
        entity.bounds.y,
        CLUTTER_SWITCH_WIDTH,
        CLUTTER_SWITCH_HEIGHT,
    )
}

/// The `ClutterBlock.Colors` of the colour switch a rectangle presses, for
/// callers outside the simulator that hold only the ground truth's player
/// rectangle (`ClutterSwitch.cs:65-68,131-156`).
pub fn clutter_switch_color_at(map: &Map, resting: Rect) -> Option<usize> {
    map.entities
        .iter()
        .enumerate()
        .filter(|(_, entity)| entity.name == "colorSwitch")
        .find(|(_, entity)| clutter_switch_rect(entity).intersects(resting))
        .and_then(|(index, _)| clutter_switch_color(map, index))
        .filter(|color| *color < CLUTTER_COLORS)
}

/// `ClutterSwitch.BePressed`: `atY += 10f; base.Y += 10f;`
/// (`ClutterSwitch.cs:86-87`).
const PRESSED_SWITCH_OFFSET: f32 = 10.0;

/// `ClutterBlockGenerator` cannot run at all until the map decoder retains the
/// `type` attribute of every `colorSwitch` in the room: without it the
/// simulator cannot tell which colour a press clears, and adding collision it
/// cannot deactivate would be a regression rather than a fidelity gain.
fn clutter_is_trackable(map: &Map) -> bool {
    map.entities
        .iter()
        .enumerate()
        .filter(|(_, entity)| entity.name == "colorSwitch")
        .all(|(index, _)| clutter_switch_color(map, index).is_some())
}

/// Every `ClutterBlockBase` rectangle a room's entity list contributes for one
/// colour. `ClutterBlockGenerator.Add` is called with
/// `((int)entity.Position.X / 8, (int)entity.Position.Y / 8, entity.Width / 8,
/// entity.Height / 8)` and builds the Solid at `level.Bounds + (x, y) * 8` with
/// size `w * 8` by `h * 8` (`ClutterBlockGenerator.cs:136-138`), which is
/// exactly the decoded entity rectangle, one Solid per entity even when two
/// entities share a rectangle.
fn clutter_rects(entities: &[crate::Entity], color: usize) -> Vec<Rect> {
    entities
        .iter()
        .filter(|entity| clutter_color(&entity.name) == Some(color))
        .map(|entity| entity.bounds)
        .collect()
}

/// `ClutterBlockBase : Solid` (`ClutterBlockBase.cs:9`) covers the whole
/// `yellowBlocks`/`redBlocks`/`greenBlocks` rectangle, and its constructor is
/// reached for every such entity regardless of the session flag - only
/// `Collidable` depends on `oshiro_clutter_cleared_<color>`
/// (`ClutterBlockBase.cs:20-27`). Append the still-enabled rectangles to the
/// present room and to every room reachable by a transition, because
/// `Level.LoadLevel` rebuilds them from the destination room's own entity list
/// (`load_transition_room`).
///
/// `ClutterSwitch : Solid` is appended the same way (`ClutterSwitch.cs:9`), at
/// the rectangle its constructor installs. `BePressed` moves it down ten pixels
/// (`ClutterSwitch.cs:86-87`); that offset is derived from the session flag the
/// press itself sets, so a switch of an already-cleared colour is appended at
/// the pressed position. A transition room carries no `EntityVisual` list, so
/// its switches stay at the unpressed position.
fn add_clutter_solids(map: &mut Map, cleared: &[bool; CLUTTER_COLORS]) {
    if !clutter_is_trackable(map) {
        return;
    }
    for color in 0..CLUTTER_COLORS {
        if cleared[color] {
            continue;
        }
        for rect in clutter_rects(&map.entities, color) {
            map.solids.push(rect);
        }
        for room in &mut map.transition_runtime {
            for rect in clutter_rects(&room.entities, color) {
                room.solids.push(rect);
            }
        }
    }
    for (rect, color) in clutter_switch_rects(map) {
        let pressed = color.is_some_and(|color| color < CLUTTER_COLORS && cleared[color]);
        map.solids.push(if pressed {
            Rect::new(
                rect.x,
                rect.y + PRESSED_SWITCH_OFFSET,
                rect.width,
                rect.height,
            )
        } else {
            rect
        });
    }
    for room in &mut map.transition_runtime {
        for entity in &room.entities {
            if entity.name == "colorSwitch" {
                room.solids.push(clutter_switch_rect(entity));
            }
        }
    }
}

/// `ClutterBlockBase.Deactivate` (`ClutterBlockBase.cs:46-51`) sets
/// `Collidable = false` on every base of the cleared colour. The rectangles
/// were appended last, so remove the last match and leave an identical tile
/// rectangle (a 16x16 clutter patch can coalesce with the tile grid) in place.
fn remove_clutter_solids(map: &mut Map, color: usize) {
    fn remove_one(solids: &mut Vec<Rect>, rect: Rect) {
        if let Some(index) = solids.iter().rposition(|solid| *solid == rect) {
            solids.remove(index);
        }
    }
    for rect in clutter_rects(&map.entities, color) {
        remove_one(&mut map.solids, rect);
    }
    for room in &mut map.transition_runtime {
        for rect in clutter_rects(&room.entities, color) {
            remove_one(&mut room.solids, rect);
        }
    }
}

fn add_room_edge_tile_bleed(map: &mut Map) {
    if map.tile_grid.is_empty() {
        return;
    }
    let occupied = |x: usize, y: usize| {
        map.tile_grid
            .get(y)
            .and_then(|row| row.chars().nth(x))
            .is_some_and(|c| c != '0' && c != ' ')
    };
    let rows = map.tile_grid.len();
    let columns = map
        .tile_grid
        .iter()
        .map(|row| row.chars().count())
        .max()
        .unwrap_or(0);
    if rows == 0 || columns == 0 {
        return;
    }
    let cell = |x: i32, y: i32| {
        Rect::new(
            map.bounds.x + x as f32 * 8.0,
            map.bounds.y + y as f32 * 8.0,
            8.0,
            8.0,
        )
    };
    let mut bled = Vec::new();
    for column in 0..columns {
        // Bottom boundary row, copied downward.
        if occupied(column, rows - 1) {
            for step in 1..4 {
                bled.push(cell(column as i32, rows as i32 - 1 + step));
            }
        }
        // Top boundary row, copied upward.
        if occupied(column, 0) {
            for step in 1..4 {
                bled.push(cell(column as i32, -(step as i32)));
            }
        }
    }
    for row in 0..rows {
        // Left boundary column, copied leftward.
        if occupied(0, row) {
            for step in 1..4 {
                bled.push(cell(-step, row as i32));
            }
        }
        // Right boundary column, copied rightward.
        if occupied(columns - 1, row) {
            for step in 1..4 {
                bled.push(cell(columns as i32 - 1 + step, row as i32));
            }
        }
    }
    if !bled.is_empty() {
        map.solids.extend(bled);
    }
}

/// `Level.DefaultSpawnPoint` (Level.cs:290) is
/// `GetSpawnPoint(new Vector2(Bounds.Left, Bounds.Bottom))`, and
/// `Session.GetSpawnPoint` (Session.cs:256) returns
/// `LevelData.Spawns.ClosestTo(from)`. Every intro state is entered by
/// `Level.LoadLevel` -> `Player.Added` (Level.cs:1297-1317), where the player
/// is created at exactly that point, so this is the coroutine's `start`.
fn intro_default_spawn(map: &Map) -> Vec2 {
    let corner = Vec2::new(map.bounds.x, map.bounds.bottom());
    let candidates: &[Vec2] = if map.room_spawns.is_empty() {
        std::slice::from_ref(&map.spawn)
    } else {
        &map.room_spawns
    };
    let mut best = map.spawn;
    let mut best_distance = f32::INFINITY;
    for spawn in candidates {
        let dx = spawn.x - corner.x;
        let dy = spawn.y - corner.y;
        let distance = dx * dx + dy * dy;
        if distance < best_distance {
            best_distance = distance;
            best = *spawn;
        }
    }
    best
}

/// Rebuild the active intro coroutine's phase from a snapshot.
///
/// Every phase that a post-`Player.Update` anchor can be in is identified from
/// exported state (`PlayerSnapshot::pos`, `speed`, `player_on_ground`); the
/// phase timers are primed with one frame already elapsed.
fn intro_resume(p: &mut PlayerSnapshot, map: &mut Map) {
    p.intro_phase_ready = true;
    p.intro_start = intro_default_spawn(map);
    let dt = p.frame_delta_time;
    match p.state {
        PlayerState::IntroWalk => {
            // The coroutine's first statement snaps X outside the room edge
            // named by `IntroWalkDirection` (Player.cs:5972-5981) and then
            // waits 0.3 s (Player.cs:5982). That snap is what a snapshot taken
            // at the state's entry/next frame shows, so use it to pick the
            // phase; the walk itself is driven by `start.X`.
            let outside = p.pos.x <= map.bounds.x || p.pos.x >= map.bounds.right();
            if outside {
                p.intro_phase = INTRO_PHASE_WALK_WAIT;
                p.intro_timer = INTRO_WALK_WAIT - dt;
            } else if (p.pos.x - p.intro_start.x).abs() > INTRO_WALK_ARRIVE {
                p.intro_phase = INTRO_PHASE_WALK_MOVE;
            } else {
                p.intro_phase = INTRO_PHASE_WALK_REST;
                p.intro_timer = INTRO_WALK_REST - dt;
            }
        }
        PlayerState::IntroJump => {
            // `IntroJumpCoroutine` (Player.cs:5995-6010) either writes
            // `Y = level.Bounds.Bottom + 16` -- placing the player below the
            // room -- and waits 0.5 s, or, when the Summit finale hands off
            // from StSummitLaunch, keeps the launch speed and starts the rise
            // immediately at `start.Y = level.Bounds.Bottom - 24`.
            let settle_y = map.bounds.bottom() + INTRO_JUMP_BOTTOM_GAP;
            // The source decides the whole chain from `wasSummitJump`
            // (`Player.cs:5998`), and the flag has to survive into the later phases -
            // the 0.2 s and 0.1 s rests after the deceleration (`:6028-6034`) and the
            // 0.35 s rest after landing (`:6055-6061`) are Summit-only. So OR it into
            // whichever phase the exported snapshot identifies; do not use it to *pick*
            // a phase, or an anchor that is already falling would re-run the rise.
            let summit_flag = if p.previous_state == PlayerState::SummitLaunch {
                INTRO_PHASE_SUMMIT_FLAG
            } else {
                0
            };
            if p.speed.y == 0.0
                && (p.pos.y > map.bounds.bottom() || (p.pos.y - settle_y).abs() <= 2.0)
            {
                p.intro_phase = INTRO_PHASE_JUMP_SETTLE | summit_flag;
                p.intro_timer = INTRO_JUMP_SETTLE - dt;
            } else if p.speed.y < INTRO_JUMP_LAUNCH_SPEED {
                // Residual speed below the coroutine's own -100 launch speed:
                // the Summit `nextLevelIntro = Jump` hand-off
                // (CS10_FinalLaunch.cs:135) with `PreviousState == 10`, whose
                // branch is `start.Y = Bounds.Bottom - 24;
                // MoveToX((int)Math.Round(X / 8f) * 8f);` (Player.cs:6006-6010).
                p.intro_start.y = map.bounds.bottom() - INTRO_JUMP_SUMMIT_BOTTOM_GAP;
                let aligned_x = (p.pos.x / 8.0).round_ties_even() * 8.0;
                intro_move_to_x(p, map, aligned_x);
                p.intro_phase = INTRO_PHASE_JUMP_RISE | INTRO_PHASE_SUMMIT_FLAG;
            } else if p.speed.y < 0.0 {
                p.intro_phase = INTRO_PHASE_JUMP_DECEL | summit_flag;
            } else {
                p.intro_phase = INTRO_PHASE_JUMP_RISE | summit_flag;
            }
        }
        PlayerState::IntroWakeUp => {
            // Player.cs:6112-6115: `Sprite.Play("asleep"); yield return 0.5f;`
            p.intro_phase = INTRO_PHASE_WAKE_ASLEEP;
            p.intro_timer = INTRO_WAKE_ASLEEP - dt;
        }
        PlayerState::IntroThinkForABit => {
            // Player.cs:6158-6159: `(base.Scene as Level).Camera.X += 8f;`
            // then `yield return 0.1f;`. The camera is not exported by the
            // trace, so the callback's one-shot nudge is applied here, at the
            // state's first update.
            p.camera.x += 8.0;
            p.intro_phase = INTRO_PHASE_THINK_CAMERA;
            p.intro_timer = INTRO_THINK_CAMERA_WAIT - dt;
        }
        PlayerState::IntroRespawn => {
            // `Tween.Update` advances `Timer` by DeltaTime on the frame
            // `IntroRespawnBegin` created the tween, so the second update of
            // the state already carries two frames of tween clock.
            p.intro_phase = INTRO_PHASE_RESPAWN;
            p.intro_timer = 2.0 * dt;
        }
        _ => {}
    }
}

/// `Actor.MoveToX` (Actor.cs:304-307) is `MoveH(toX - ExactPosition.X)`.
fn intro_move_to_x(p: &mut PlayerSnapshot, map: &mut Map, to_x: f32) {
    let exact_x = p.pos.x + p.movement_remainder.x;
    move_axis_amount(p, map, true, to_x - exact_x);
}

/// `IntroWalkCoroutine` (Player.cs:5969-5993).
fn intro_walk_update(p: &mut PlayerSnapshot, map: &mut Map) {
    if p.intro_phase == INTRO_PHASE_WALK_WAIT {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        // `Sprite.Play("runSlow")` has no physics effect.
        p.intro_phase = INTRO_PHASE_WALK_MOVE;
    }
    if p.intro_phase == INTRO_PHASE_WALK_MOVE {
        let facing_dir = if p.facing { 1.0 } else { -1.0 };
        let ahead = current_player_rect(p, p.pos.x + facing_dir, p.pos.y);
        // `while (Math.Abs(X - start.X) > 2f && !CollideCheck<Solid>(Position + new Vector2(Facing, 0)))`
        // (Player.cs:5984) -- `CollideCheck<Solid>` includes DreamBlocks, so
        // the probe is `Map::solid_at`.
        if (p.pos.x - p.intro_start.x).abs() > INTRO_WALK_ARRIVE && !map.solid_at(ahead) {
            // `MoveTowardsX(start.X, 64f * Engine.DeltaTime)` uses
            // `Actor.ExactPosition` (Actor.cs:292-296).
            move_towards_x(
                p,
                map,
                p.intro_start.x,
                INTRO_WALK_SPEED * p.frame_delta_time,
            );
            return;
        }
        // `Position = start` (Player.cs:5989) writes both axes directly; it
        // does not touch `Platform.movementCounter`.
        p.pos = p.intro_start;
        p.intro_phase = INTRO_PHASE_WALK_REST;
        p.intro_timer = INTRO_WALK_REST;
        return;
    }
    if p.intro_phase == INTRO_PHASE_WALK_REST {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        // `StateMachine.State = 0` (Player.cs:5992).
        p.state = PlayerState::Normal;
    }
}

/// `IntroJumpCoroutine` (Player.cs:5995-6068).
///
/// `wasSummitJump = StateMachine.PreviousState == 10` (Player.cs:5998) is not
/// exported by the trace, so the Summit finale hand-off
/// (`CS10_FinalLaunch.cs:135` sets `nextLevelIntro = Jump` while the player is
/// still in StSummitLaunch) is recorded in the phase's
/// `INTRO_PHASE_SUMMIT_FLAG` bit. It changes the 0.1 s post-launch wait into
/// `0.2 s + 0.1 s` (Player.cs:6028-6038) and skips the final
/// `Position = start` (Player.cs:6048-6051).
fn intro_jump_update(p: &mut PlayerSnapshot, _map: &Map) {
    let summit = p.intro_phase & INTRO_PHASE_SUMMIT_FLAG != 0;
    let mut phase = p.intro_phase & INTRO_PHASE_MASK;
    if phase == INTRO_PHASE_JUMP_SETTLE {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        phase = INTRO_PHASE_JUMP_RISE;
        p.intro_phase = phase | (p.intro_phase & INTRO_PHASE_SUMMIT_FLAG);
    }
    if phase == INTRO_PHASE_JUMP_RISE {
        // `while (base.Y > start.Y - 8f) base.Y += -120f * Engine.DeltaTime;`
        // (Player.cs:6015-6019). `base.Y +=` writes `Position.Y` directly, so
        // it must not be routed through `Actor.MoveV`.
        if p.pos.y > p.intro_start.y - INTRO_JUMP_RISE_GAP {
            p.pos.y += INTRO_JUMP_RISE_SPEED * p.frame_delta_time;
            return;
        }
        p.pos.y = p.pos.y.round_ties_even();
        p.speed.y = INTRO_JUMP_LAUNCH_SPEED;
        phase = INTRO_PHASE_JUMP_DECEL;
        p.intro_phase = phase | (p.intro_phase & INTRO_PHASE_SUMMIT_FLAG);
    }
    if phase == INTRO_PHASE_JUMP_DECEL {
        // `while (Speed.Y < 0f) Speed.Y += Engine.DeltaTime * 800f;` then
        // `Speed.Y = 0f;` (Player.cs:6021-6027). The loop's first iteration
        // shares the rise-exit frame.
        if p.speed.y < 0.0 {
            p.speed.y += INTRO_JUMP_GRAVITY * p.frame_delta_time;
            return;
        }
        p.speed.y = 0.0;
        if summit {
            p.intro_phase = INTRO_PHASE_JUMP_SUMMIT_REST | INTRO_PHASE_SUMMIT_FLAG;
            p.intro_timer = INTRO_JUMP_SUMMIT_REST;
        } else {
            p.intro_phase = INTRO_PHASE_JUMP_REST;
            p.intro_timer = INTRO_JUMP_REST;
        }
        return;
    }
    if phase == INTRO_PHASE_JUMP_REST {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        phase = INTRO_PHASE_JUMP_FALL;
        // `wasSummitJump` has to survive into the fall and the landing: the Summit path's
        // post-landing `yield return 0.35f` (`Player.cs:6055-6061`) and the `Position = start`
        // skip (`:6048`) both depend on it. Writing the bare phase here dropped the flag, so the
        // fall ran with `summit == false` and the state ended on the landing frame.
        p.intro_phase = phase | (p.intro_phase & INTRO_PHASE_SUMMIT_FLAG);
    }
    if phase == INTRO_PHASE_JUMP_SUMMIT_REST {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        // `Sprite.Play("launchRecover"); yield return 0.1f;`
        // (Player.cs:6032-6033).
        p.intro_phase = INTRO_PHASE_JUMP_SUMMIT_RECOVER | INTRO_PHASE_SUMMIT_FLAG;
        p.intro_timer = INTRO_JUMP_SUMMIT_RECOVER;
        return;
    }
    if phase == INTRO_PHASE_JUMP_SUMMIT_RECOVER {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        phase = INTRO_PHASE_JUMP_FALL;
        // Same flag preservation as `INTRO_PHASE_JUMP_REST` above: the Summit fall must stay
        // flagged or the landing takes the non-Summit path.
        p.intro_phase = phase | (p.intro_phase & INTRO_PHASE_SUMMIT_FLAG);
    }
    if phase == INTRO_PHASE_JUMP_FALL {
        // `while (!onGround) Speed.Y += Engine.DeltaTime * 800f;`
        // (Player.cs:6043-6047).
        if !p.player_on_ground {
            p.speed.y += INTRO_JUMP_GRAVITY * p.frame_delta_time;
            return;
        }
        if !summit {
            // `if (StateMachine.PreviousState != 10) Position = start;`
            // (Player.cs:6048-6051).
            p.pos = p.intro_start;
        }
        // `Speed.Y` is deliberately *not* zeroed here. The coroutine's loop simply stops
        // while the landing frame still carries the accumulated fall speed, and the frame's
        // own physics then attempts `MoveV(Speed.Y * dt)` into the floor: that blocked step
        // zeroes `movementCounter.Y` and `Player.OnCollideV` zeroes `Speed.Y`. Measured on
        // `7-Summit|1|g-00|209548` offset 45 - the game's counter goes 0.08337 -> 0 and its
        // end-of-frame speed is 0, while zeroing the speed here skipped the move entirely and
        // left a 0.08337 remainder that flipped a pixel 35 frames later.
        if summit {
            // Landing at the end of a Summit hand-off is not the end of the state:
            // `if (wasSummitJump) { ...particles...; yield return 0.35f; }` runs before
            // `StateMachine.State = 0` (Player.cs:6055-6067), so the game stays in
            // `StIntroJump` for 0.35 s after touching the ground. Measured on
            // `7-Summit|1|g-00|209548`, whose divergence is exactly the landing frame:
            // the game's `movementCounter.Y` is zeroed by the landing collision and its
            // state is still `StIntroJump` while the simulator had already returned to
            // `StNormal`.
            p.intro_phase = INTRO_PHASE_JUMP_SUMMIT_LAND_REST | INTRO_PHASE_SUMMIT_FLAG;
            p.intro_timer = INTRO_JUMP_SUMMIT_LAND_REST;
            return;
        }
        p.intro_phase = 0;
        p.state = PlayerState::Normal;
    }
    if phase == INTRO_PHASE_JUMP_SUMMIT_LAND_REST {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        p.intro_phase = 0;
        p.state = PlayerState::Normal;
    }
}

/// `IntroWakeUpCoroutine` (Player.cs:6112-6119).
fn intro_wake_up_update(p: &mut PlayerSnapshot) {
    if p.intro_phase == INTRO_PHASE_WAKE_ASLEEP {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        // `yield return Sprite.PlayRoutine("wakeUp")` starts the animation on
        // this frame; `Monocle.Sprite.Update` (Sprite.cs:104-183) has already
        // run for it, so the first advance is the next frame.
        p.intro_phase = INTRO_PHASE_WAKE_SPRITE;
        p.intro_sprite_timer = 0.0;
        p.intro_sprite_frame = 0;
        return;
    }
    if p.intro_phase == INTRO_PHASE_WAKE_SPRITE {
        // `Sprite.PlayRoutine` -> `PlayUtil` yields while `Sprite.Animating`
        // (Sprite.cs:418-428); `Animating` clears when the non-looping
        // animation walks past its last frame (Sprite.cs:158-177).
        p.intro_sprite_timer += p.frame_delta_time;
        if p.intro_sprite_timer >= INTRO_WAKE_ANIM_DELAY {
            p.intro_sprite_timer -= INTRO_WAKE_ANIM_DELAY;
            p.intro_sprite_frame += 1;
            if p.intro_sprite_frame >= INTRO_WAKE_ANIM_FRAMES {
                // The frame the animation ends: `Coroutine.Update` pops the
                // nested enumerator and does not resume the outer routine
                // until the next frame (Monocle/Coroutine.cs:64-71).
                p.intro_phase = INTRO_PHASE_WAKE_POP;
            }
        }
        return;
    }
    if p.intro_phase == INTRO_PHASE_WAKE_POP {
        p.intro_phase = INTRO_PHASE_WAKE_REST;
        p.intro_timer = INTRO_WAKE_REST;
        return;
    }
    if p.intro_phase == INTRO_PHASE_WAKE_REST {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        p.state = PlayerState::Normal;
    }
}

/// `IntroThinkForABitCoroutine` (Player.cs:6156-6174).
fn intro_think_for_a_bit_update(p: &mut PlayerSnapshot, map: &mut Map) {
    if p.intro_phase == INTRO_PHASE_THINK_CAMERA {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        // `Sprite.Play("walk"); float target = base.X + 8f;`
        // (Player.cs:6160-6161).
        p.intro_start.x = p.pos.x + INTRO_THINK_WALK_DISTANCE;
        p.intro_phase = INTRO_PHASE_THINK_WALK;
    }
    if p.intro_phase == INTRO_PHASE_THINK_WALK {
        // `while (base.X < target) { MoveH(32f * Engine.DeltaTime); ... }`
        // (Player.cs:6162-6166).
        if p.pos.x < p.intro_start.x {
            move_axis_amount(p, map, true, INTRO_THINK_WALK_SPEED * p.frame_delta_time);
            return;
        }
        p.intro_phase = INTRO_PHASE_THINK_IDLE;
        p.intro_timer = INTRO_THINK_IDLE;
        return;
    }
    if p.intro_phase == INTRO_PHASE_THINK_IDLE {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        p.facing = false;
        p.intro_phase = INTRO_PHASE_THINK_LEFT;
        p.intro_timer = INTRO_THINK_LEFT;
        return;
    }
    if p.intro_phase == INTRO_PHASE_THINK_LEFT {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        p.facing = true;
        p.intro_phase = INTRO_PHASE_THINK_RIGHT;
        p.intro_timer = INTRO_THINK_RIGHT;
        return;
    }
    if p.intro_phase == INTRO_PHASE_THINK_RIGHT {
        if p.intro_timer > 0.0 {
            p.intro_timer -= p.frame_delta_time;
            return;
        }
        // `StateMachine.State = 0` (Player.cs:6173).
        p.state = PlayerState::Normal;
    }
}

/// `IntroRespawnBegin`'s tween (Player.cs:6121-6146): `Tween.Update` adds
/// DeltaTime and `OnComplete` restores `StateMachine.State = 0` as soon as
/// `Timer >= Duration` (Monocle/Tween.cs).
fn intro_respawn_update(p: &mut PlayerSnapshot) {
    p.intro_timer += p.frame_delta_time;
    if p.intro_timer >= INTRO_RESPAWN_TIME {
        p.state = PlayerState::Normal;
    }
}

fn begin_star_fly(p: &mut PlayerSnapshot) {
    p.state = PlayerState::StarFly;
    p.star_fly_hitbox_preserved = false;
    p.star_fly_transforming = true;
    p.star_fly_transform_frames = STAR_FLY_TRANSFORM_FRAMES;
    p.star_fly_timer = STAR_FLY_TIME;
    p.star_fly_speed_lerp = 0.0;
    p.star_fly_last_dir = Vec2::default();
    p.jump_grace_timer = 0.0;
}

fn star_fly_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    if p.star_fly_transforming {
        p.speed = approach_vector(
            p.speed,
            Vec2::default(),
            STAR_FLY_TRANSFORM_DECEL * p.frame_delta_time,
        );
        p.star_fly_transform_frames = p.star_fly_transform_frames.saturating_sub(1);
        if p.star_fly_transform_frames == 0 {
            p.star_fly_transforming = false;
            p.star_fly_timer = STAR_FLY_TIME;
            // Player.StarFlyBegin (feather transform): RefillDash(); RefillStamina();
            refill_dash(p);
            p.stamina = 110.0;
            let mut dir = input_vector(input);
            if dir == Vec2::default() {
                dir.x = if p.facing { 1.0 } else { -1.0 };
            }
            p.speed = scale(dir, STAR_FLY_START_SPEED);
            p.star_fly_last_dir = dir;
        }
        return;
    }

    let mut aim = input_vector(input);
    let slow = aim == Vec2::default();
    if slow {
        aim = p.star_fly_last_dir;
    }

    let mut current_dir = normalize(p.speed);
    if current_dir == Vec2::default() {
        current_dir = aim;
    } else {
        current_dir = rotate_towards(
            current_dir,
            angle(aim),
            STAR_FLY_ROTATE_SPEED * p.frame_delta_time,
        );
    }
    p.star_fly_last_dir = current_dir;

    let max_speed = if slow {
        p.star_fly_speed_lerp = 0.0;
        STAR_FLY_SLOW_SPEED
    } else if current_dir != Vec2::default() && dot(current_dir, aim) >= 0.45 {
        p.star_fly_speed_lerp = approach(p.star_fly_speed_lerp, 1.0, p.frame_delta_time);
        STAR_FLY_TARGET_SPEED + (STAR_FLY_MAX_SPEED - STAR_FLY_TARGET_SPEED) * p.star_fly_speed_lerp
    } else {
        p.star_fly_speed_lerp = 0.0;
        STAR_FLY_TARGET_SPEED
    };
    let speed = approach(
        length(p.speed),
        max_speed,
        STAR_FLY_ACCEL * p.frame_delta_time,
    );
    p.speed = scale(current_dir, speed);

    if input.jump_pressed {
        if grounded_at_offset(p, map, 3.0) {
            end_star_fly(p, map);
            p.state = PlayerState::Normal;
            p.jump_buffer_timer = 0.0;
            p.jump_grace_timer = 0.0;
            p.speed.y = JUMP_SPEED;
            p.speed.x += input.move_x as f32 * JUMP_H_BOOST;
            // StarFlyUpdate calls Player.Jump here.  Jump assigns JumpSpeed
            // and then consumes the current or retained LiftBoost; preserve
            // that order for a grounded Feather exit on a moving/reforming
            // CassetteBlock.
            add_lift_boost(p);
            p.auto_jump = false;
            p.dash_attack_timer = 0.0;
            p.wall_slide_timer = WALL_SLIDE_TIME;
            p.wall_boost_timer = 0.0;
            p.var_jump_speed = p.speed.y;
            p.var_jump_timer = VAR_JUMP_TIME;
            return;
        }
        let wall = wall_dir(p, map);
        if wall != 0 {
            end_star_fly(p, map);
            p.state = PlayerState::Normal;
            p.speed = Vec2::new(-(wall as f32) * WALL_JUMP_H, JUMP_SPEED);
            p.auto_jump = false;
            p.dash_attack_timer = 0.0;
            p.wall_slide_timer = WALL_SLIDE_TIME;
            p.wall_boost_timer = 0.0;
            if input.move_x != 0 {
                p.force_move_x = -wall;
                p.force_move_x_timer = 0.16;
            }
            p.var_jump_speed = p.speed.y;
            p.var_jump_timer = VAR_JUMP_TIME;
            return;
        }
    }

    if input.grab_held {
        let right = input.move_x != -1
            && touching_wall(p, map, 1)
            && !climb_blocker_check(p, map, 3.0, 0.0);
        let left = input.move_x != 1
            && touching_wall(p, map, -1)
            && !climb_blocker_check(p, map, -3.0, 0.0);
        if right || left {
            let wall = if right { 1 } else { -1 };
            end_star_fly(p, map);
            p.state = PlayerState::Climb;
            p.facing = wall > 0;
            p.speed.x = 0.0;
            p.speed.y *= 0.2;
            p.wall_slide_timer = WALL_SLIDE_TIME;
            p.climb_no_move_timer = 0.1;
            return;
        }
    }

    if (input.dash_pressed || input.crouch_dash_pressed)
        && p.dashes > 0
        && p.dash_cooldown_timer <= 0.0
    {
        end_star_fly(p, map);
        begin_dash(p, input, true, false, map);
        return;
    }

    p.star_fly_timer -= p.frame_delta_time;
    if p.star_fly_timer <= 0.0 {
        if input.move_y < 0 {
            p.speed.y = STAR_FLY_EXIT_UP;
        }
        if input.move_y < 1 {
            p.var_jump_speed = p.speed.y;
            p.auto_jump = true;
            p.auto_jump_timer = 0.0;
            p.var_jump_timer = VAR_JUMP_TIME;
        }
        p.speed.y = p.speed.y.min(0.0);
        p.speed.x = p.speed.x.clamp(-STAR_FLY_MAX_EXIT_X, STAR_FLY_MAX_EXIT_X);
        end_star_fly(p, map);
        p.state = PlayerState::Normal;
    }
}

fn end_star_fly(p: &mut PlayerSnapshot, map: &Map) {
    p.star_fly_transforming = false;
    p.star_fly_transform_frames = 0;
    p.star_fly_hitbox_preserved = false;
    let normal = player_rect(p.pos.x, p.pos.y);
    if map.solid_at(normal) {
        let start_y = p.pos.y;
        p.pos.y -= 2.0;
        if map.solid_at(player_rect(p.pos.x, p.pos.y)) {
            p.pos.y = start_y - 2.0;
            p.ducking = true;
            if map.solid_at(duck_player_rect(p.pos.x, p.pos.y)) {
                p.pos.y = start_y;
            }
        }
    }
}

fn player_rect(x: f32, y: f32) -> Rect {
    Rect::new(x - 4.0, y - 11.0, 8.0, 11.0)
}

fn duck_player_rect(x: f32, y: f32) -> Rect {
    Rect::new(x - 4.0, y - 6.0, 8.0, 6.0)
}

fn star_fly_rect(x: f32, y: f32) -> Rect {
    Rect::new(x - 4.0, y - 10.0, 8.0, 8.0)
}

fn star_fly_hurt_rect(x: f32, y: f32) -> Rect {
    Rect::new(x - 3.0, y - 9.0, 6.0, 6.0)
}

fn current_player_rect(p: &PlayerSnapshot, x: f32, y: f32) -> Rect {
    if p.state == PlayerState::StarFly {
        star_fly_rect(x, y)
    } else if p.star_fly_hitbox_preserved {
        // Player.Update temporarily assigns the StarFly hurtbox while running
        // PlayerCollider callbacks. Player.Bounce caches that active collider,
        // so an IceBall cancellation restores the 6x6 hurtbox as Collider.
        star_fly_hurt_rect(x, y)
    } else if p.ducking {
        duck_player_rect(x, y)
    } else {
        player_rect(x, y)
    }
}

fn touching_jump_thru(p: &PlayerSnapshot, map: &Map) -> bool {
    let player = current_player_rect(p, p.pos.x, p.pos.y);
    map.entities
        .iter()
        .any(|entity| entity.kind == EntityKind::JumpThru && entity.bounds.intersects(player))
}

/// `Player.DashAttacking` (`Player.cs:1062-1072`): the dash-attack window is the
/// 0.3 second `dashAttackTimer`, or the RedDash state once it has run out.
fn dash_attacking(p: &PlayerSnapshot) -> bool {
    p.dash_attack_timer > 0.0 || p.state == PlayerState::RedDash
}

/// `Monocle.Entity.CollideCheckOutside<JumpThru>(at)` (`Monocle/Entity.cs:648`,
/// `Collide.Check(a, b, at)` = `!Collide.Check(a, b) && Collide.Check(a, b, at)`):
/// a JumpThru (or its `Cloud` subclass) that does not touch the player where it
/// stands but would at `offset`.
fn jump_thru_outside(p: &PlayerSnapshot, map: &Map, offset: f32) -> bool {
    let current = current_player_rect(p, p.pos.x, p.pos.y);
    let probe = current_player_rect(p, p.pos.x, p.pos.y + offset);
    map.entities.iter().any(|entity| {
        matches!(entity.kind, EntityKind::JumpThru | EntityKind::Cloud)
            && entity.bounds.intersects(probe)
            && !entity.bounds.intersects(current)
    })
}

/// `Player.JumpThruBoostBlockedCheck` (`Player.cs:4179-4189`): the JumpThru
/// Assist is skipped when any tracked `LedgeBlocker` reports a hit for the live
/// collider two pixels above the player (`Celeste/LedgeBlocker.cs:33-44`). A
/// spinner that is invisible or out of range is parked far off the map by
/// `sync_spinner_entity`, which mirrors `Collidable = false`
/// (`Celeste/CrystalStaticSpinner.cs:207`).
fn jump_thru_boost_blocked_check(p: &PlayerSnapshot, map: &Map) -> bool {
    let probe = current_player_rect(p, p.pos.x, p.pos.y - 2.0);
    map.entities
        .iter()
        .any(|entity| ledge_blocker_collides(entity, probe))
}

/// Which entities carry a `LedgeBlocker` in vanilla 1.4.0.0.
/// `Celeste/CrystalStaticSpinner.cs:156` and `Celeste/DustStaticSpinner.cs:24`
/// add one with no block checker; `Celeste/Spikes.cs:50/57/61` add one for
/// `Up`/`Left`/`Right` and `Spikes.cs:52-54` adds none for `Down`, which the
/// decoder stores as `direction.y > 0`. `LedgeBlocker.Blocking` is only ever
/// written for `ClimbBlocker` (`Celeste/WallBooster.cs:88`), so these always
/// block. `EntityKind` has no DustStaticSpinner or TriggerSpikes variant.
fn is_ledge_blocker(entity: &crate::Entity) -> bool {
    match entity.kind {
        EntityKind::CrystalStaticSpinner => true,
        EntityKind::Spikes => entity.direction.y <= 0.0,
        _ => false,
    }
}

/// `LedgeBlocker.DashCorrectCheck`/`JumpThruBoostCheck` both end in
/// `player.CollideCheck(base.Entity, ...)`, so the blocker's real collider is
/// what matters. `Celeste/CrystalStaticSpinner.cs:152` installs
/// `ColliderList(Circle(6f), Hitbox(16f, 4f, -8f, -3f))` around the spinner
/// position; the decoded bounds are that union's 16x12 bounding box, so the
/// rounded corners have to be tested separately. `Spikes.cs:49/56/60` install
/// plain `Hitbox` colliders, whose decoded bounds are exact.
fn ledge_blocker_collides(entity: &crate::Entity, rect: Rect) -> bool {
    if !is_ledge_blocker(entity) {
        return false;
    }
    if entity.kind == EntityKind::CrystalStaticSpinner {
        let center = Vec2::new(
            entity.bounds.x + entity.bounds.width * 0.5,
            entity.bounds.y + entity.bounds.height * 0.5,
        );
        return circle_rect_intersects(center, 6.0, rect)
            || Rect::new(center.x - 8.0, center.y - 3.0, 16.0, 4.0).intersects(rect);
    }
    entity.bounds.intersects(rect)
}

/// `Player.DashCorrectCheck(add)` (`Player.cs:4191-4209`): move the player by
/// `add`, force the `hurtbox` collider (`normalHurtbox`, or `duckHurtbox` while
/// `Ducking` - `Player.cs:1020-1025`), and ask every `LedgeBlocker`
/// (`Celeste/LedgeBlocker.cs:46`) whether it touches the player there. Vanilla
/// `Spikes` (`Celeste/Spikes.cs:50`) and `CrystalStaticSpinner`
/// (`Celeste/CrystalStaticSpinner.cs:156`) register a blocker with no custom
/// `BlockChecker`, so a plain overlap blocks the dash close.
fn dash_correct_check(p: &PlayerSnapshot, map: &Map, offset_y: f32) -> bool {
    let hurt = current_player_hurt_rect(p);
    let hurt = Rect::new(hurt.x, hurt.y + offset_y, hurt.width, hurt.height);
    map.entities
        .iter()
        .any(|entity| ledge_blocker_collides(entity, hurt))
}

fn player_hurt_rect(x: f32, y: f32) -> Rect {
    Rect::new(x - 4.0, y - 11.0, 8.0, 9.0)
}

fn current_player_hurt_rect(p: &PlayerSnapshot) -> Rect {
    if p.state == PlayerState::StarFly {
        Rect::new(p.pos.x - 3.0, p.pos.y - 9.0, 6.0, 6.0)
    } else if p.ducking {
        Rect::new(p.pos.x - 4.0, p.pos.y - 6.0, 8.0, 4.0)
    } else {
        player_hurt_rect(p.pos.x, p.pos.y)
    }
}

fn can_unduck(p: &PlayerSnapshot, map: &Map) -> bool {
    !p.ducking || !map.solid_at(player_rect(p.pos.x, p.pos.y))
}

fn input_aim(input: InputState, facing: bool) -> Vec2 {
    let mut value = input_vector(input);
    if value == Vec2::default() {
        value.x = if facing { 1.0 } else { -1.0 };
    }
    value
}

fn input_vector(input: InputState) -> Vec2 {
    let mut x = input.move_x as f32;
    let mut y = input.move_y as f32;
    if x != 0.0 && y != 0.0 {
        const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
        x *= DIAG;
        y *= DIAG;
    }
    Vec2::new(x, y)
}

fn grounded(p: &PlayerSnapshot, map: &Map) -> bool {
    grounded_at_offset(p, map, 1.0)
}

fn grounded_at_offset(p: &PlayerSnapshot, map: &Map, offset: f32) -> bool {
    let at = current_player_rect(p, p.pos.x, p.pos.y + offset);
    map.solid_at(at) || map.jump_thru_at(at, current_player_rect(p, p.pos.x, p.pos.y).bottom())
}

fn grounded_at_position(p: &PlayerSnapshot, map: &Map, position: Vec2) -> bool {
    let player = current_player_rect(p, position.x, position.y);
    let below = current_player_rect(p, position.x, position.y + 1.0);
    map.solid_at(below) || map.jump_thru_at(below, player.bottom())
}

fn water_check(p: &PlayerSnapshot, map: &Map, offset_y: f32) -> bool {
    map.water_at(current_player_rect(p, p.pos.x, p.pos.y + offset_y))
}

fn swim_check(p: &PlayerSnapshot, map: &Map) -> bool {
    water_check(p, map, -8.0) && water_check(p, map, 0.0)
}

fn swim_underwater_check(p: &PlayerSnapshot, map: &Map) -> bool {
    water_check(p, map, -9.0)
}

fn swim_jump_check(p: &PlayerSnapshot, map: &Map) -> bool {
    !water_check(p, map, -14.0)
}

fn swim_rise_check(p: &PlayerSnapshot, map: &Map) -> bool {
    !water_check(p, map, -18.0)
}

fn enter_swim(p: &mut PlayerSnapshot) {
    p.state = PlayerState::Swim;
    if p.speed.y > 0.0 {
        p.speed.y *= SWIM_Y_SPEED_MULT;
    }
    p.stamina = 110.0;
}

fn touching_wall(p: &PlayerSnapshot, map: &Map, dir: i8) -> bool {
    map.solid_at(current_player_rect(p, p.pos.x + dir as f32, p.pos.y))
}

fn check_stamina(p: &PlayerSnapshot) -> f32 {
    p.stamina + if p.wall_boost_timer > 0.0 { 27.5 } else { 0.0 }
}

fn climb_bounds_check(p: &PlayerSnapshot, map: &Map, dir: i8) -> bool {
    let rect = current_player_rect(p, p.pos.x, p.pos.y);
    // Player.ClimbBoundsCheck reads Level.Bounds, which switches to the
    // destination room in OnTransition. Using the map's source bounds here
    // incorrectly disables every wall probe after a horizontal transition.
    let bounds = p.current_room_bounds.unwrap_or(map.bounds);
    rect.x + dir as f32 * CLIMB_CHECK_DIST >= bounds.x
        && rect.right() + dir as f32 * CLIMB_CHECK_DIST < bounds.right()
}

fn climb_check(p: &PlayerSnapshot, map: &Map, dir: i8, y_add: f32) -> bool {
    climb_bounds_check(p, map, dir)
        && !climb_blocker_check(p, map, dir as f32 * CLIMB_CHECK_DIST, y_add)
        && map.solid_at(current_player_rect(
            p,
            p.pos.x + dir as f32 * CLIMB_CHECK_DIST,
            p.pos.y + y_add,
        ))
}

fn climb_hop_blocked_check(p: &PlayerSnapshot, map: &Map) -> bool {
    map.solid_at(current_player_rect(p, p.pos.x, p.pos.y - 6.0))
}

fn wall_jump_check(p: &PlayerSnapshot, map: &Map, dir: i8) -> bool {
    // Player.cs:2523-2540. The probe is 3 px, except while DashAttacking with a
    // straight up DashDir == (0, -1) where it becomes SuperWallJumpCheckDist
    // (5 px) - unless a Spikes entity facing the player would be hit at that
    // distance, in which case the 3 px probe stands.
    let mut dist = WALL_JUMP_CHECK_DIST;
    if dash_attacking(p) && p.dash_dir.x == 0.0 && p.dash_dir.y == -1.0 {
        dist = SUPER_WALL_JUMP_CHECK_DIST;
        let rect = current_player_rect(p, p.pos.x + dir as f32 * dist, p.pos.y);
        let spike_in_the_way = map.entities.iter().any(|entity| {
            entity.kind == EntityKind::Spikes
                && if dir <= 0 {
                    entity.direction.x > 0.0
                } else {
                    entity.direction.x < 0.0
                }
                && entity.bounds.intersects(rect)
        });
        if spike_in_the_way {
            dist = WALL_JUMP_CHECK_DIST;
        }
    }
    climb_bounds_check(p, map, dir)
        && !climb_blocker_edge_check(p, map, dir as f32 * dist)
        && map.solid_at(current_player_rect(
            p,
            p.pos.x + dir as f32 * dist,
            p.pos.y,
        ))
}

/// `Player.SuperWallJumpAngleCheck` (`Player.cs:1092-1102`).
fn super_wall_jump_angle_check(p: &PlayerSnapshot) -> bool {
    p.dash_dir.x.abs() <= 0.2 && p.dash_dir.y <= -0.75
}

/// `ClimbBlocker.Check` (`ClimbBlocker.cs:28-38`) over every component the
/// simulator's entity set can carry. Only `InvisibleBarrier`
/// (`InvisibleBarrier.cs:15`) and `WallBooster` (`WallBooster.cs:42`) add one:
/// the barrier is always blocking, the booster only in `IceMode`
/// (`WallBooster.cs:85-101`, `notCoreMode` or a Cold `Level.CoreMode`), where the
/// strip refuses the grab and drives `ClimbUpdate` into `trySlip`.
///
/// This arm only works because `PlayerSnapshot::core_mode` carries
/// `Level.CoreMode` rather than `Session.CoreMode`: read as the session value it
/// cost 526 replayed frames and regressed two 9H-Core segments, because
/// `9-Core|1|b-03` reports a Cold *session* while the game happily grabs a wall
/// flush against a booster - which cannot happen while that booster blocks.
/// `Level.CoreMode` disagrees with the session value on 3,641 Level rows of the
/// 100% trace, in both directions.
fn climb_blocker_check(p: &PlayerSnapshot, map: &Map, x_add: f32, y_add: f32) -> bool {
    let player = current_player_rect(p, p.pos.x + x_add, p.pos.y + y_add);
    map.entities.iter().any(|entity| match entity.kind {
        EntityKind::InvisibleBarrier => entity.bounds.intersects(player),
        EntityKind::WallBooster => {
            wall_booster_ice_mode(p, entity) && entity.bounds.intersects(player)
        }
        _ => false,
    })
}

/// `WallBooster.IceMode` (`WallBooster.cs:74-101`): a `notCoreMode` booster is
/// always icy, otherwise `Level.CoreMode` decides. `p.core_mode` carries
/// `Level.CoreMode`, not `Session.CoreMode` - the two disagree on 3,641 Level
/// rows of the 100% trace, and the game's own behaviour there (a grab of a wall
/// flush against a booster in a room whose *session* mode is Cold) is what proves
/// which one the entity reads.
fn wall_booster_ice_mode(p: &PlayerSnapshot, entity: &crate::Entity) -> bool {
    entity.direction.y != 0.0 || p.core_mode == crate::CoreMode::Cold
}

/// `Player.WallBoosterCheck` (`Player.cs:3279-3289`): the facing-side booster the
/// player currently overlaps, or nothing when a `ClimbBlocker` occupies that
/// spot - which is the same strip once it is icy.
fn wall_booster_check(p: &PlayerSnapshot, map: &Map, facing: i8) -> bool {
    if climb_blocker_check(p, map, facing as f32, 0.0) {
        return false;
    }
    let rect = current_player_rect(p, p.pos.x, p.pos.y);
    map.entities.iter().any(|entity| {
        entity.kind == EntityKind::WallBooster
            && (entity.direction.x as i8) == facing
            && entity.bounds.intersects(rect)
    })
}

/// `ClimbBlocker.EdgeCheck` (`ClimbBlocker.cs:40-50`) is deliberately narrower
/// than `Check`: it only counts components whose `Edge` flag is set, and among
/// vanilla entities that is `InvisibleBarrier` alone (`InvisibleBarrier.cs:15`).
/// `WallBooster`'s blocker is `edge: false` (`WallBooster.cs:42`), so a conveyor
/// never suppresses a wall jump even while it is icy (`Player.cs:2541-2544`).
fn climb_blocker_edge_check(p: &PlayerSnapshot, map: &Map, x_add: f32) -> bool {
    let player = current_player_rect(p, p.pos.x + x_add, p.pos.y);
    map.entities
        .iter()
        .any(|entity| entity.kind == EntityKind::InvisibleBarrier && entity.bounds.intersects(player))
}

fn slip_check(p: &PlayerSnapshot, map: &Map, add_y: f32) -> bool {
    let rect = current_player_rect(p, p.pos.x, p.pos.y);
    let x = if p.facing { rect.right() } else { rect.x - 1.0 };
    let lower_y = rect.y + 4.0 + add_y;
    !map.solid_at(Rect::new(x, lower_y, 1.0, 1.0))
        && !map.solid_at(Rect::new(x, lower_y - 4.0 + add_y, 1.0, 1.0))
}

fn climb_hop(
    p: &mut PlayerSnapshot,
    map: &Map,
    wall: i8,
    climb_hop_solid: &mut Option<ClimbHopSolid>,
) {
    // `Player.ClimbHop` (`Player.cs:4122-4131`) stores
    // `CollideFirst<Solid>(Position + UnitX * Facing)`. `Level.SolidTiles`
    // never moves, so a tile hit leaves a reference that can never carry the
    // player; the runtime map models tiles as rectangles rather than entities,
    // and `None` reproduces that.
    *climb_hop_solid = first_solid_at(p, map);
    if touching_wall(p, map, wall) {
        p.hop_wait_x = wall;
        p.hop_wait_x_speed = wall as f32 * CLIMB_HOP_X;
    } else {
        p.hop_wait_x = 0;
        p.hop_wait_x_speed = 0.0;
        p.speed.x = wall as f32 * CLIMB_HOP_X;
    }
    p.speed.y = p.speed.y.min(CLIMB_HOP_Y);
    p.force_move_x = 0;
    p.force_move_x_timer = CLIMB_HOP_FORCE_TIME;
    p.no_wind_timer = CLIMB_HOP_NO_WIND_TIME;
}

fn update_climb_hop_wait(p: &mut PlayerSnapshot, map: &Map) {
    if p.hop_wait_x == 0 {
        return;
    }
    // Player.cs:1685 tests `Math.Sign(Speed.X) == -hopWaitX`. `Math.Sign(0f)`
    // is 0, so a stationary player keeps waiting for the ledge instead of
    // dropping `hopWaitXSpeed` (Player.cs:1691-1692).
    if math_sign(p.speed.x) == -(p.hop_wait_x as f32) || p.speed.y > 0.0 {
        p.hop_wait_x = 0;
        p.hop_wait_x_speed = 0.0;
    } else if !touching_wall(p, map, p.hop_wait_x) {
        p.speed.x = p.hop_wait_x_speed;
        p.hop_wait_x = 0;
        p.hop_wait_x_speed = 0.0;
    }
}

fn wall_dir(p: &PlayerSnapshot, map: &Map) -> i8 {
    if touching_wall(p, map, -1) {
        -1
    } else if touching_wall(p, map, 1) {
        1
    } else {
        0
    }
}

/// `Player.NormalUpdate`'s wall-slide probe (`Player.cs:3751`): a Facing-side
/// solid the player is flush against, with no blocking ClimbBlocker edge.
///
/// The probe direction is Facing. The previous helper searched the left wall
/// first and was unrelated to the direction the player actually faces.
fn wall_slide_at(p: &PlayerSnapshot, map: &Map, dir: i8) -> bool {
    touching_wall(p, map, dir) && !climb_blocker_edge_check(p, map, dir as f32)
}

fn move_axis(p: &mut PlayerSnapshot, map: &mut Map, horizontal: bool) {
    let speed = if horizontal { p.speed.x } else { p.speed.y };
    move_axis_amount(p, map, horizontal, speed * p.frame_delta_time);
}

fn naive_move(p: &mut PlayerSnapshot, amount: Vec2) {
    p.movement_remainder.x += amount.x;
    p.movement_remainder.y += amount.y;
    let move_x = p.movement_remainder.x.round_ties_even();
    let move_y = p.movement_remainder.y.round_ties_even();
    p.movement_remainder.x -= move_x;
    p.movement_remainder.y -= move_y;
    p.pos.x += move_x;
    p.pos.y += move_y;
}

fn move_axis_amount(p: &mut PlayerSnapshot, map: &mut Map, horizontal: bool, amount: f32) {
    move_axis_amount_inner(p, map, horizontal, amount, true);
}

/// `Actor.MoveH`/`MoveV` called with a **null** collision callback, which is how
/// `Player.WindMove` moves: `Player.cs:3107` is `MoveH(move.X);` and
/// `Player.cs:3132` is `MoveV(move.Y);`, both with the default `onCollide = null`
/// (`Actor.cs:186-208`). `MoveVExact`/`MoveHExact` still probe
/// `CollideFirst<Solid>` and still clear `movementCounter` on the blocked step
/// (`Actor.cs:220`, `Actor.cs:249`), but `onCollide?.Invoke(...)` is skipped, so
/// `Player.OnCollideH`/`OnCollideV` never run and `Speed` is left untouched.
///
/// This matters for an updraft against a ceiling: the wind's whole-pixel step is
/// blocked, yet the source keeps `Speed.Y` (including a positive, falling
/// `Speed.Y` that would have taken `OnCollideV`'s landing branch and its
/// unconditional `Speed.Y = 0f` at `Player.cs:3408`).
fn move_axis_amount_silent(p: &mut PlayerSnapshot, map: &mut Map, horizontal: bool, amount: f32) {
    move_axis_amount_inner(p, map, horizontal, amount, false);
}

fn move_axis_amount_inner(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    horizontal: bool,
    amount: f32,
    collide_callback: bool,
) {
    let remainder = if horizontal {
        &mut p.movement_remainder.x
    } else {
        &mut p.movement_remainder.y
    };
    *remainder += amount;
    let amount = remainder.round_ties_even() as i32;
    *remainder -= amount as f32;
    let sign = amount.signum();
    for _ in 0..amount.unsigned_abs() {
        let next_x = p.pos.x + if horizontal { sign as f32 } else { 0.0 };
        let next_y = p.pos.y + if horizontal { 0.0 } else { sign as f32 };
        let next = current_player_rect(p, next_x, next_y);
        let dream_block = map.dream_block_at(next);
        let collided = map.non_dream_solid_at(next)
            || (dream_block && p.state != PlayerState::DreamDash)
            || (!horizontal
                && sign > 0
                && !p.ignore_jump_thrus
                && map.jump_thru_at(next, current_player_rect(p, p.pos.x, p.pos.y).bottom()));
        if collided {
            // `Actor.MoveHExact`/`MoveVExact` zero the *moving* axis's
            // `movementCounter` the moment the blocked step is found, and only
            // then invoke the collision callback (`Actor.cs:220` for X,
            // `Actor.cs:249` for Y). `Player.OnCollideH`/`OnCollideV` can still
            // return without touching that axis (dash corner corrections call
            // `MoveVExact`/`MoveHExact`, which never restore it), so the
            // zeroing has to happen here rather than per callback branch.
            if horizontal {
                p.movement_remainder.x = 0.0;
            } else {
                p.movement_remainder.y = 0.0;
            }
            if !collide_callback {
                // The probe and the counter clear above are all `MoveVExact`/
                // `MoveHExact` do when `onCollide` is null; the walk stops and no
                // `Player.OnCollideH`/`OnCollideV` response may run.
                return;
            }
            if dream_block
                && p.can_dream_dash
                && (p.dash_attack_timer > 0.0 || p.state == PlayerState::RedDash)
            {
                p.state = PlayerState::DreamDash;
                p.speed = Vec2::new(p.dash_dir.x * DASH_SPEED, p.dash_dir.y * DASH_SPEED);
                p.dream_dash_can_end_timer = 0.1;
                p.stamina = 110.0;
                p.dash_attack_timer = 0.0;
                p.dash_end_pending = false;
                break;
            }
            if horizontal {
                // `Player.OnCollideH`'s `OnDashCollide` branch runs before the
                // dash corner correction (`Player.cs:3155-3177`). A `Rebound`
                // or `Ignore` returns out of `OnCollideH`, so neither the
                // correction nor the ordinary `Speed.X = 0` stop may run.
                match try_dash_collide(p, map, next, true, sign as f32) {
                    Some(DashCollision::Rebound) => {
                        rebound(p, -p.speed.x.signum());
                        return;
                    }
                    Some(DashCollision::Ignore) => return,
                    _ => {}
                }
                if matches!(p.state, PlayerState::Dash | PlayerState::RedDash)
                    && p.speed.y == 0.0
                    && p.speed.x != 0.0
                {
                    for correction in 1..=DASH_CORNER_CORRECTION {
                        for direction in [1.0, -1.0] {
                            let offset = correction as f32 * direction;
                            let corrected =
                                current_player_rect(p, p.pos.x + sign as f32, p.pos.y + offset);
                            let wedged = current_player_rect(
                                p,
                                p.pos.x + sign as f32,
                                p.pos.y + (correction - 1) as f32 * direction,
                            );
                            if !map.solid_at(corrected) && map.solid_at(wedged) {
                                p.pos.y += offset;
                                p.pos.x += sign as f32;
                                return;
                            }
                        }
                    }
                }
                if p.state == PlayerState::StarFly {
                    if p.star_fly_timer < STAR_FLY_END_NO_BOUNCE_TIME {
                        p.speed.x = 0.0;
                    } else {
                        p.speed.x *= STAR_FLY_WALL_BOUNCE;
                    }
                } else {
                    if p.wall_speed_retention_timer <= 0.0 {
                        p.wall_speed_retained = p.speed.x;
                        p.wall_speed_retention_timer = 0.06;
                    }
                    p.speed.x = 0.0;
                }
                p.movement_remainder.x = 0.0;
                if p.state == PlayerState::RedDash {
                    p.dash_attack_timer = 0.0;
                    p.state = PlayerState::HitSquash;
                    p.state_timer = 0.1;
                }
            } else {
                // `Player.OnCollideV`'s `OnDashCollide` branch runs before the
                // rising-bonk and falling-correction handling
                // (`Player.cs:3255-3281`). `Rebound()` there takes the default
                // `direction = 0` argument, so `Speed.X` is zeroed.
                match try_dash_collide(p, map, next, false, sign as f32) {
                    Some(DashCollision::Rebound) => {
                        rebound(p, 0.0);
                        return;
                    }
                    Some(DashCollision::Ignore) => return,
                    _ => {}
                }
                if sign < 0 && p.state != PlayerState::StarFly && p.speed.y < 0.0 {
                    // Player.cs:3358-3388: a rising ceiling collision slides the
                    // player sideways onto the first free column one pixel up.
                    // The search is four pixels wide, or five while
                    // `DashAttacking && Math.Abs(Speed.X) < 0.01f`, which is what
                    // lets an up-dash climb a five pixel ceiling lip.
                    let correction_limit = if dash_attacking(p) && p.speed.x.abs() < 0.01 {
                        5
                    } else {
                        DASH_CORNER_CORRECTION
                    };
                    if p.speed.x <= 0.0 {
                        for correction in 1..=correction_limit {
                            let corrected =
                                current_player_rect(p, p.pos.x - correction as f32, p.pos.y - 1.0);
                            if !map.solid_at(corrected) {
                                p.pos.x -= correction as f32;
                                p.pos.y -= 1.0;
                                p.movement_remainder.y = 0.0;
                                return;
                            }
                        }
                    }
                    if p.speed.x >= 0.0 {
                        for correction in 1..=correction_limit {
                            let corrected =
                                current_player_rect(p, p.pos.x + correction as f32, p.pos.y - 1.0);
                            if !map.solid_at(corrected) {
                                p.pos.x += correction as f32;
                                p.pos.y -= 1.0;
                                p.movement_remainder.y = 0.0;
                                return;
                            }
                        }
                    }
                    // Player.cs:3389-3392: a ceiling bonk only ends the
                    // variable-jump window once the jump has been rising for
                    // more than five hundredths of a second
                    // (`varJumpTimer < 0.15f`). Before that the window survives,
                    // so the frames after the bonk keep the half-gravity
                    // `num7 = 0.5` fall target instead of falling at 900 px/s^2.
                    // This runs only when neither sideways ceiling slide above
                    // returned, exactly like the source's fall-through.
                    if p.var_jump_timer < 0.15 {
                        p.var_jump_timer = 0.0;
                    }
                }
                if sign > 0
                    && p.speed.y > 0.0
                    && matches!(p.state, PlayerState::Dash | PlayerState::RedDash)
                    && !p.dash_started_on_ground
                {
                    if p.speed.x <= 0.0 {
                        for correction in 1..=DASH_CORNER_CORRECTION {
                            let offset = -(correction as f32);
                            let corrected = Vec2::new(p.pos.x + offset, p.pos.y);
                            if !grounded_at_position(p, map, corrected) {
                                p.pos = Vec2::new(corrected.x, corrected.y + 1.0);
                                p.movement_remainder = Vec2::default();
                                return;
                            }
                        }
                    }
                    if p.speed.x >= 0.0 {
                        for correction in 1..=DASH_CORNER_CORRECTION {
                            let offset = correction as f32;
                            let corrected = Vec2::new(p.pos.x + offset, p.pos.y);
                            if !grounded_at_position(p, map, corrected) {
                                p.pos = Vec2::new(corrected.x, corrected.y + 1.0);
                                p.movement_remainder = Vec2::default();
                                return;
                            }
                        }
                    }
                }
                if sign > 0 && p.dash_dir.x != 0.0 && p.dash_dir.y > 0.0 && p.speed.y > 0.0 {
                    p.dash_dir.x = p.dash_dir.x.signum();
                    p.dash_dir.y = 0.0;
                    p.speed.x *= 1.2;
                    p.ducking = true;
                }
                if p.state == PlayerState::StarFly {
                    if p.star_fly_timer < STAR_FLY_END_NO_BOUNCE_TIME {
                        p.speed.y = 0.0;
                    } else {
                        p.speed.y *= STAR_FLY_WALL_BOUNCE;
                    }
                } else {
                    p.speed.y = 0.0;
                }
                p.movement_remainder.y = 0.0;
                if p.state == PlayerState::RedDash {
                    p.dash_attack_timer = 0.0;
                    p.state = PlayerState::HitSquash;
                    p.state_timer = 0.1;
                }
            }
            break;
        }
        p.pos.x = next_x;
        p.pos.y = next_y;
    }
}

/// `Monocle.Actor.MoveVExact` (`Celeste/Actor.cs:238-290`): whole-pixel steps,
/// each preceded by a `CollideFirst<Solid>` probe and - only while moving down
/// and while `IgnoreJumpThrus` is clear - a `CollideFirstOutside<JumpThru>`
/// probe. A blocked step clears `movementCounter.Y`; the fractional remainder
/// is never consumed, so this is not `MoveV`.
fn move_v_exact(p: &mut PlayerSnapshot, map: &Map, amount: i32) {
    let sign = amount.signum();
    let mut remaining = amount;
    while remaining != 0 {
        let next = current_player_rect(p, p.pos.x, p.pos.y + sign as f32);
        let blocked = map.solid_at(next)
            || (sign > 0
                && !p.ignore_jump_thrus
                && map.jump_thru_at(next, current_player_rect(p, p.pos.x, p.pos.y).bottom()));
        if blocked {
            p.movement_remainder.y = 0.0;
            return;
        }
        remaining -= sign;
        p.pos.y += sign as f32;
    }
}

/// `Monocle.Actor.MoveHExact` (`Celeste/Actor.cs:210-236`): whole-pixel steps,
/// each preceded by a `CollideFirst<Solid>` probe. A blocked step clears
/// `movementCounter.X` and stops the walk; the fractional part of the amount
/// does not exist because the caller already truncated it.
fn move_h_exact(p: &mut PlayerSnapshot, map: &Map, amount: i32) {
    let sign = amount.signum();
    let mut remaining = amount;
    while remaining != 0 {
        let next = current_player_rect(p, p.pos.x + sign as f32, p.pos.y);
        if map.solid_at(next) {
            p.movement_remainder.x = 0.0;
            return;
        }
        remaining -= sign;
        p.pos.x += sign as f32;
    }
}

/// Entity kinds the runtime map keeps as `Monocle.Solid` entities, mirroring
/// `Map::non_dream_solid_at` plus `DreamBlock` (`Celeste/DreamBlock.cs`).
/// `Level.SolidTiles` (`Celeste/SolidTiles.cs:8`) is the level's tile grid and
/// is modelled as `Map::solids` instead, so tile walls never appear here.
fn is_solid_entity(kind: EntityKind) -> bool {
    matches!(
        kind,
        EntityKind::BounceBlock
            | EntityKind::CassetteBlock
            | EntityKind::CrushBlock
            | EntityKind::DashBlock
            | EntityKind::DreamBlock
            | EntityKind::ExitBlock
            | EntityKind::FallingBlock
            | EntityKind::InvisibleBarrier
            | EntityKind::MoveBlock
            | EntityKind::MovingSolid
            | EntityKind::StaticSolid
            | EntityKind::CrumbleBlock
            | EntityKind::FloatySpaceBlock
            | EntityKind::ZipMover
            | EntityKind::TempleGate
            | EntityKind::DashSwitch
    )
}

/// `Entity.CollideFirst<Solid>(at)` (`Monocle/Entity.cs:687`) resolved to the
/// runtime entity index, which is what `Player.ClimbHop` (`Player.cs:4124`)
/// stores in `climbHopSolid`. A tile hit yields `None`: `Level.SolidTiles`
/// (`Celeste/SolidTiles.cs:18`) never moves, so whichever tile Solid wins the
/// probe the carry is inert, and the runtime map models tiles as rectangles
/// rather than entities.
fn first_solid_at(p: &PlayerSnapshot, map: &Map) -> Option<ClimbHopSolid> {
    let facing = if p.facing { 1.0 } else { -1.0 };
    let probe = current_player_rect(p, p.pos.x + facing, p.pos.y);
    if map.static_solid_at(probe) {
        return None;
    }
    map.entities
        .iter()
        .enumerate()
        .find_map(|(index, entity)| {
            (is_solid_entity(entity.kind) && entity.bounds.intersects(probe)).then(|| ClimbHopSolid {
                entity: index,
                position: Vec2::new(entity.bounds.x, entity.bounds.y),
            })
        })
}

/// `Entity.Collidable`. The runtime map parks a disabled Solid far outside the
/// room (`park_entity`), which is how `ExitBlock`, `InvisibleBarrier`,
/// `CassetteBlock` and `FallingBlock` model their collidable flags.
fn solid_is_collidable(entity: &crate::Entity) -> bool {
    entity.bounds.x != PARKED_ENTITY_POSITION || entity.bounds.y != PARKED_ENTITY_POSITION
}

fn interact(
    p: &mut PlayerSnapshot,
    map: &Map,
    input: InputState,
    lookout_booster_box: Option<Rect>,
) {
    if let Some(from_y) = p.pending_bounce_from_y.take() {
        // Backward compatibility for portable snapshots produced before
        // FireBall callbacks were aligned to the source's same-frame order.
        bounce(p, map, from_y);
        return;
    }
    if p.state == PlayerState::DreamDash {
        return;
    }
    if p.state == PlayerState::Swim {
        if p.speed.y < 0.0 && p.speed.y >= SWIM_MAX_RISE {
            while !swim_check(p, map) {
                p.speed.y = 0.0;
                if !move_exact(p, map, false, 1) {
                    break;
                }
            }
        }
    } else if p.state == PlayerState::Normal && swim_check(p, map) {
        enter_swim(p);
    } else if p.state == PlayerState::Climb && swim_check(p, map) {
        // Player.cs treats climbing into the upper half of water specially:
        // move upward until Madeline can remain outside it, and only switch to
        // Swim if that correction cannot get her clear.
        let player_center_y = p.pos.y - 5.5;
        let water_center_y = map
            .entities
            .iter()
            .find(|entity| {
                entity.kind == EntityKind::Water
                    && entity
                        .bounds
                        .intersects(current_player_rect(p, p.pos.x, p.pos.y))
            })
            .map(|entity| entity.bounds.y + entity.bounds.height * 0.5);
        if water_center_y.is_some_and(|center_y| player_center_y < center_y) {
            while swim_check(p, map) {
                if !move_exact(p, map, false, -1) {
                    break;
                }
            }
            if swim_check(p, map) {
                enter_swim(p);
            }
        } else {
            enter_swim(p);
        }
    }
    let hitbox = current_player_rect(p, p.pos.x, p.pos.y);
    let mut heart_index = 0usize;
    let mut rising_lava_index = 0usize;
    let mut sandwich_lava_index = 0usize;
    let mut bumper_index = 0usize;
    let mut refill_index = 0usize;
    let mut core_toggle_index = 0usize;
    for (entity_index, entity) in map.entities.iter().enumerate() {
        let current_bumper = (entity.kind == EntityKind::Bumper).then(|| {
            let index = bumper_index;
            bumper_index += 1;
            index
        });
        let current_heart = if entity.kind == EntityKind::HeartGem {
            let index = heart_index;
            heart_index += 1;
            Some(index)
        } else {
            None
        };
        let current_rising_lava = if entity.kind == EntityKind::RisingLava {
            let index = rising_lava_index;
            rising_lava_index += 1;
            Some(index)
        } else {
            None
        };
        let current_sandwich_lava = if entity.kind == EntityKind::SandwichLava {
            let index = sandwich_lava_index;
            sandwich_lava_index += 1;
            Some(index)
        } else {
            None
        };
        let current_refill = if entity.kind == EntityKind::Refill {
            let index = refill_index;
            refill_index += 1;
            Some(index)
        } else {
            None
        };
        // Seeker PlayerColliders run from the dynamic Seeker update after
        // Player.Update, using its live StateMachine collider selection.
        if entity.kind == EntityKind::Seeker {
            continue;
        }
        let player_box = if matches!(
            entity.kind,
            // Every `Celeste.PlayerCollider` in the room is polled by
            // `Player.Update` itself, inside a block that swaps the player's
            // collider to the live `hurtbox` for the duration of the loop
            // (`Player.cs:1898-1909`: `Collider collider = base.Collider;
            // base.Collider = hurtbox; foreach (PlayerCollider component2 in
            // ...GetComponents<PlayerCollider>())`). `PlayerCollider.Check`
            // (PlayerCollider.cs:24-50) then runs `player.CollideCheck(Entity)`,
            // so those callbacks see `normalHurtbox`/`duckHurtbox`/
            // `starFlyHurtbox` (`Player.cs:609-615`) - never the taller
            // hitbox. A `Booster` registers a bare
            // `new PlayerCollider(OnPlayer)` (Booster.cs:60), so its
            // `OnPlayer` collision test is one of them.
            EntityKind::Spikes
                | EntityKind::FlyFeather
                | EntityKind::Bumper
                | EntityKind::Spring
                | EntityKind::IceBall
                | EntityKind::RisingLava
                | EntityKind::SandwichLava
                | EntityKind::CrystalStaticSpinner
                | EntityKind::Killbox
                | EntityKind::Booster
                | EntityKind::RedBooster
                // `CoreModeToggle` adds a bare `new PlayerCollider(OnPlayer)`
                // (`CoreModeToggle.cs:48`), so its overlap test is one of the callbacks that
                // sees the live hurtbox rather than the taller hitbox.
                | EntityKind::CoreModeToggle
                // Same for these four: `Refill.cs:54`, `HeartGem.cs`,
                // `Puffer.cs` and `Strawberry.cs` all register a `PlayerCollider`. The
                // hurtbox is two pixels shorter than the hitbox (`Hitbox(8, 9, -4, -11)`
                // against `Hitbox(8, 11, -4, -11)`), and that is enough to flip a
                // collection by one frame - measured on `6-Reflection|1|b-04|188460`,
                // where the simulator ate a `refill` at row 188675 while the game's
                // hurtbox first reached it at 188676.
                | EntityKind::Refill
                | EntityKind::HeartGem
                | EntityKind::Puffer
                | EntityKind::Strawberry
        ) {
            current_player_hurt_rect(p)
        } else {
            hitbox
        };
        let intersects = match entity.kind {
            EntityKind::Booster | EntityKind::RedBooster => circle_rect_intersects(
                Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5 + 2.0,
                ),
                10.0,
                lookout_booster_box.unwrap_or(player_box),
            ),
            EntityKind::Bumper => circle_rect_intersects(
                Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                ),
                entity.bounds.width * 0.5,
                player_box,
            ),
            EntityKind::IceBall => {
                let center = Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                );
                Rect::new(center.x - 8.0, center.y - 3.0, 16.0, 6.0).intersects(player_box)
            }
            EntityKind::CrystalStaticSpinner => {
                let center = Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                );
                circle_rect_intersects(center, 6.0, player_box)
                    || Rect::new(center.x - 8.0, center.y - 3.0, 16.0, 4.0).intersects(player_box)
            }
            EntityKind::Puffer
            | EntityKind::AngryOshiro
            | EntityKind::Seeker
            | EntityKind::Snowball => {
                let center = Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                );
                let bounce_height = if entity.kind == EntityKind::Seeker {
                    4.0
                } else {
                    6.0
                };
                let bounce = Rect::new(
                    center.x - 8.0,
                    entity.bounds.y - 2.0,
                    16.0,
                    bounce_height + 2.0,
                );
                bounce.intersects(player_box) || entity.bounds.intersects(player_box)
            }
            EntityKind::RisingLava => current_rising_lava
                .and_then(|index| p.rising_lavas.get(index))
                .is_some_and(|lava| {
                    Rect::new(lava.position.x, lava.position.y, 340.0, 120.0).intersects(player_box)
                }),
            EntityKind::SandwichLava => current_sandwich_lava
                .and_then(|index| p.sandwich_lavas.get(index))
                .is_some_and(|lava| {
                    !lava.waiting
                        && !lava.leaving
                        && !lava.removed
                        && (Rect::new(lava.position.x, lava.position.y, 340.0, 120.0)
                            .intersects(player_box)
                            || Rect::new(lava.position.x, lava.position.y - 280.0, 340.0, 120.0)
                                .intersects(player_box))
                }),
            _ => entity.bounds.intersects(player_box),
        };
        if !intersects {
            continue;
        }
        match entity.kind {
            EntityKind::Spikes
                if (!entity.shielded || !dash_through_spikes_pass(p))
                    && spike_is_lethal(p, entity.direction, entity.bounds) =>
            {
                p.dead = true;
                p.speed = Vec2::default();
                p.death_freeze_pending = true;
                p.respawn_frames = 95;
                return;
            }
            EntityKind::RisingLava
            | EntityKind::SandwichLava
            | EntityKind::CrystalStaticSpinner
            | EntityKind::Killbox => {
                p.dead = true;
                p.speed = Vec2::default();
                p.death_freeze_pending = true;
                p.respawn_frames = 95;
                return;
            }
            EntityKind::Water => {}
            EntityKind::CoreModeToggle => {
                // `CoreModeToggle.OnPlayer` (`CoreModeToggle.cs:103-126`): `Usable` is
                // `(!onlyFire || iceMode) && (!onlyIce || !iceMode)` (`:24-38`) with the toggle's own
                // `iceMode` mirroring `Level.CoreMode` through its `CoreModeListener` (`:65-69`), and
                // a flip is refused for one second after the previous one (`:105`, `:124`).
                let slot = core_toggle_index;
                core_toggle_index += 1;
                let ice = p.core_mode == crate::CoreMode::Cold;
                let only_fire = entity.direction.x != 0.0;
                let only_ice = entity.direction.y != 0.0;
                let usable = (!only_fire || ice) && (!only_ice || !ice);
                if usable
                    && p.core_mode_toggle_cooldowns
                        .get(slot)
                        .is_some_and(|cooldown| *cooldown <= 0.0)
                {
                    p.core_mode_toggle_cooldowns[slot] = 1.0;
                    p.core_mode = if ice {
                        crate::CoreMode::Hot
                    } else {
                        crate::CoreMode::Cold
                    };
                    // `Celeste.Freeze(0.05f)` (`CoreModeToggle.cs:123`) raises
                    // `Engine.FreezeTimer` only when it is currently smaller.
                    if p.freeze_timer < CORE_TOGGLE_FREEZE_TIME {
                        p.freeze_timer = CORE_TOGGLE_FREEZE_TIME;
                    }
                }
            }
            EntityKind::Booster | EntityKind::RedBooster
                if !matches!(
                    p.state,
                    PlayerState::Boost | PlayerState::RedDash | PlayerState::HitSquash
                ) && {
                    let target = Vec2::new(
                        entity.bounds.x + entity.bounds.width * 0.5,
                        entity.bounds.y + entity.bounds.height * 0.5 + 2.0,
                    );
                    // A regular Dash can enter a Booster, but not the same
                    // Booster whose BoostRoutine is still awaiting the end
                    // of this dash.
                    !(p.booster_boosting && p.last_booster_target == target)
                        && (p.booster_reuse_timer <= 0.0 || p.last_booster_target != target)
                } =>
            {
                p.state = PlayerState::Boost;
                p.speed = Vec2::default();
                p.boost_target = Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5 + 2.0,
                );
                p.boost_red = entity.kind == EntityKind::RedBooster;
                p.last_booster_target = p.boost_target;
                p.booster_reuse_timer = 0.45;
                p.state_timer = 0.25 + p.frame_delta_time * 2.0;
                // Player.BoostBegin: RefillDash(); RefillStamina(); (Player.cs).
                refill_dash(p);
                p.stamina = 110.0;
            }
            EntityKind::FlyFeather => {
                let target = Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                );
                if p.feather_reuse_timer > 0.0 && p.last_feather_target == target {
                    continue;
                }
                let dash_attacking = p.dash_attack_timer > 0.0 || p.state == PlayerState::RedDash;
                if entity.shielded && !dash_attacking {
                    point_bounce(p, target);
                    continue;
                }
                p.stamina = 110.0;
                if p.state == PlayerState::StarFly {
                    p.star_fly_timer = STAR_FLY_TIME;
                } else if p.state != PlayerState::ReflectionFall {
                    begin_star_fly(p);
                } else {
                    continue;
                }
                p.last_feather_target = target;
                p.feather_reuse_timer = if entity.single_use {
                    f32::MAX
                } else {
                    FEATHER_RESPAWN_TIME
                };
            }
            EntityKind::Bumper => {
                let target = Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                );
                let index = current_bumper.expect("Bumper has runtime state");
                if p.bumpers[index].respawn_timer <= 0.0 {
                    explode_launch(p, input, target, false, false);
                    p.last_bumper_target = target;
                    p.bumper_reuse_timer = 0.6;
                    p.bumpers[index].respawn_timer = 0.6;
                }
            }
            EntityKind::Spring if p.state != PlayerState::DreamDash => {
                if entity.direction.y < 0.0 {
                    if p.speed.y >= 0.0 {
                        super_bounce(p, map, entity.bounds.y);
                    }
                } else if entity.direction.x != 0.0 {
                    side_bounce(p, map, entity.direction.x.signum() as i8, entity.bounds);
                }
            }
            EntityKind::Strawberry if entity_index < u64::BITS as usize => {
                let mask = 1_u64 << entity_index;
                if p.strawberry_picked_mask & mask == 0 {
                    p.strawberry_picked_mask |= mask;
                    if p.carried_strawberries == 0 {
                        p.strawberry_follow_delay_timer = 0.3;
                        p.strawberry_collect_timer = 0.0;
                    }
                    p.carried_strawberries = p.carried_strawberries.saturating_add(1);
                }
            }
            EntityKind::Refill => {
                let Some(index) = current_refill else {
                    continue;
                };
                let state = &mut p.refills[index];
                if state.removed || !state.collidable {
                    continue;
                }
                // Player.UseRefill(twoDashes): refill only when Dashes < num
                // (MaxDashes, or 2 for the pink diamond) or Stamina < 20
                // (`Player.cs` UseRefill). MaxDashes is Inventory.Dashes.
                let target_dashes = if state.two_dashes { 2 } else { p.max_dashes };
                if p.dashes < target_dashes || p.stamina < CLIMB_TIRED_THRESHOLD {
                    p.dashes = target_dashes;
                    p.stamina = 110.0;
                    state.collidable = false;
                    state.respawn_timer = REFILL_RESPAWN_TIME;
                    // RefillRoutine begins with Celeste.Freeze(0.05f).
                    p.freeze_timer = REFILL_FREEZE_TIME;
                    if state.one_use {
                        state.removed = true;
                    }
                }
            }
            EntityKind::HeartGem => {
                let Some(index) = current_heart else {
                    continue;
                };
                if p.heart_gems.get(index).is_some_and(|heart| heart.collected) {
                    continue;
                }
                let target = Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                );
                if p.dash_attack_timer > 0.0 || p.state == PlayerState::RedDash {
                    if let Some(heart) = p.heart_gems.get_mut(index) {
                        heart.collected = true;
                        heart.phase = 1;
                        // HeartGem.Update has already advanced components when
                        // OnPlayer creates the coroutine. It first runs and
                        // yields on the next entity frame, then freezes on the
                        // following frame.
                        heart.wait_frames = 2;
                    }
                } else {
                    point_bounce(p, target);
                }
            }
            EntityKind::IceBall => {
                let target = Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                );
                if (p.bounce_reuse_timer <= 0.0 || p.last_bounce_target != target)
                    && p.speed.y >= 0.0
                    && current_player_hurt_rect(p).bottom() <= target.y + 4.0
                {
                    p.last_bounce_target = target;
                    // A cold FireBall becomes non-collidable after the bounce.
                    p.bounce_reuse_timer = f32::MAX;
                    // Player.Update runs PlayerCollider checks after its
                    // movement pass. FireBall.OnBounce therefore corrects the
                    // just-moved position and changes state on this same frame.
                    bounce(p, map, target.y - 2.0);
                }
            }
            EntityKind::Puffer
            | EntityKind::AngryOshiro
            | EntityKind::Seeker
            | EntityKind::Snowball => {
                let target = Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                );
                let player_bottom = current_player_rect(p, p.pos.x, p.pos.y).bottom();
                let top_limit = match entity.kind {
                    EntityKind::Puffer => target.y + 3.0,
                    EntityKind::AngryOshiro => entity.bounds.y + 6.0,
                    EntityKind::Seeker => entity.bounds.y + 4.0,
                    EntityKind::Snowball => entity.bounds.y + 6.0,
                    _ => unreachable!(),
                };
                if player_bottom <= top_limit && p.speed.y >= 0.0 {
                    let from_y = match entity.kind {
                        EntityKind::AngryOshiro => entity.bounds.y + 2.0,
                        EntityKind::Snowball => entity.bounds.y - 2.0,
                        _ => entity.bounds.y,
                    };
                    bounce(p, map, from_y);
                    p.last_bounce_target = target;
                    p.bounce_reuse_timer = 0.1;
                    p.freeze_timer = match entity.kind {
                        EntityKind::AngryOshiro => 0.2,
                        EntityKind::Seeker => 0.15,
                        EntityKind::Snowball => 0.1,
                        _ => 0.0,
                    };
                } else if entity.kind == EntityKind::Puffer {
                    // `Puffer.cs:552`: refuse to explode while the Puffer is `Gone` or within the 0.5 s
                    // spawn cooldown (`:167`, ticked down at `:362-365`).
                    let may_explode = p
                        .puffers
                        .iter()
                        .find(|q| (q.center.x - target.x).abs() < 0.5 && (q.center.y - target.y).abs() < 0.5)
                        .map(|q| q.state != 2 && q.cant_explode_timer <= 0.0)
                        .unwrap_or(true);
                    if may_explode {
                    explode_launch(p, input, target, false, true);
                    }
                    p.last_bounce_target = target;
                    p.bounce_reuse_timer = 2.5;
                } else {
                    p.dead = true;
                    p.speed = Vec2::default();
                    p.death_freeze_pending = true;
                    p.respawn_frames = 95;
                    return;
                }
            }
            EntityKind::Wind => {
                // WindTrigger changes the global WindController pattern. It
                // does not apply a local force and leaving the trigger does
                // not clear the selected pattern.
                p.wind_target = entity.direction;
            }
            _ => {}
        }
    }
}

fn update_strawberry_train(p: &mut PlayerSnapshot) {
    if p.carried_strawberries == 0 {
        return;
    }
    if p.strawberry_follow_delay_timer > 0.0 {
        p.strawberry_follow_delay_timer =
            (p.strawberry_follow_delay_timer - p.frame_delta_time).max(0.0);
        return;
    }

    // Strawberry.Update uses Player.OnSafeGround. Normal solids and
    // jumpthroughs are safe in the supported map subset, and Swim is always
    // treated as safe ground by Player.Update.
    if p.on_ground || p.state == PlayerState::Swim {
        p.strawberry_collect_timer += p.frame_delta_time;
        if p.strawberry_collect_timer > 0.15 {
            p.carried_strawberries -= 1;
            p.strawberry_collect_index = p.strawberry_collect_index.saturating_add(1);
            p.strawberry_collect_reset_timer = 2.5;
            p.strawberry_collect_timer = if p.carried_strawberries > 0 {
                // Followers update in train order. Once the first berry calls
                // OnCollect, the next berry becomes FollowIndex 0 and runs its
                // own Update later in the same frame, advancing -0.15 by p.frame_delta_time.
                -0.15 + p.frame_delta_time
            } else {
                0.0
            };
        }
    } else {
        p.strawberry_collect_timer = p.strawberry_collect_timer.min(0.0);
    }
}

fn reset_for_spring_bounce(p: &mut PlayerSnapshot, map: &Map) {
    // `Player.SuperBounce` (`Player.cs:2708-2722`) and `Player.SideBounce`
    // (`:2741-2762`) both wrap the dash refill in `if (!Inventory.NoRefills)`, unlike
    // `Player.Bounce` (`:2677-2691`, which guards it too) and `Player.PointBounce`
    // (`:3061`, which does not). Refilling unconditionally handed the simulator a dash
    // the game never restores in the `NoRefills` Core.
    if !p.no_refills {
        refill_dash(p);
    }
    p.stamina = 110.0;
    if p.state == PlayerState::StarFly {
        // Same as `bounce`: assigning state Normal runs StarFlyEnd first (the cached collider is
        // restored only after that callback returns).
        end_star_fly(p, map);
    }
    p.state = PlayerState::Normal;
    p.jump_grace_timer = 0.0;
    p.var_jump_timer = VAR_JUMP_TIME;
    p.auto_jump = true;
    p.auto_jump_timer = 0.0;
    p.dash_attack_timer = 0.0;
    p.wall_slide_timer = WALL_SLIDE_TIME;
    p.wall_boost_timer = 0.0;
    p.launched = false;
}

/// `Actor.MoveV`/`MoveH(amount)` accumulate into `movementCounter` and move
/// `round(counter + amount)` whole pixels, clearing the counter only on a blocked step, with the
/// default null collide callback. `Player.SuperBounce`/`SideBounce` move exactly this way after
/// swapping in `normalHitbox` (`Player.cs:2717`, `:2749`), so probe with `player_rect`.
fn spring_move(p: &mut PlayerSnapshot, map: &Map, horizontal: bool, amount: f32) {
    let remainder = if horizontal {
        &mut p.movement_remainder.x
    } else {
        &mut p.movement_remainder.y
    };
    *remainder += amount;
    let mut steps = remainder.round_ties_even() as i32;
    *remainder -= steps as f32;
    let sign = steps.signum();
    while steps != 0 {
        let next = if horizontal {
            player_rect(p.pos.x + sign as f32, p.pos.y)
        } else {
            player_rect(p.pos.x, p.pos.y + sign as f32)
        };
        if map.non_dream_solid_at(next) || map.dream_block_at(next) {
            *remainder = 0.0;
            break;
        }
        if horizontal {
            p.pos.x += sign as f32;
        } else {
            p.pos.y += sign as f32;
        }
        steps -= sign;
    }
}

fn super_bounce(p: &mut PlayerSnapshot, map: &Map, from_y: f32) {
    // Player.SuperBounce temporarily uses the normal collider and moves the
    // player's bottom onto the spring before applying the launch.
    spring_move(p, map, false, from_y - p.pos.y);
    reset_for_spring_bounce(p, map);
    p.speed.x = 0.0;
    p.speed.y = SUPER_BOUNCE_SPEED;
    p.var_jump_speed = p.speed.y;
}

fn side_bounce(p: &mut PlayerSnapshot, map: &Map, dir: i8, spring: Rect) {
    // `Player.SideBounce` (`Player.cs:2743-2746`): `if (Math.Abs(Speed.X) > 240f && Math.Sign(Speed.X) == dir)
    // return false;` - the spring refuses to bounce a player already moving that way quickly.
    if p.speed.x.abs() > 240.0 && p.speed.x.signum() as i8 == dir {
        return;
    }
    // SideBounce aligns the normal collider to the spring face and only
    // corrects vertically by at most four pixels.
    let from_y = spring.y + spring.height * 0.5;
    spring_move(p, map, false, (from_y - p.pos.y).clamp(-4.0, 4.0));
    let target_x = if dir > 0 {
        spring.right() + 4.0
    } else {
        spring.x - 4.0
    };
    spring_move(p, map, true, target_x - p.pos.x);
    reset_for_spring_bounce(p, map);
    p.force_move_x = dir;
    p.force_move_x_timer = SIDE_BOUNCE_FORCE_MOVE_X_TIME;
    p.speed.x = SIDE_BOUNCE_SPEED * dir as f32;
    p.speed.y = BOUNCE_SPEED;
    p.var_jump_speed = p.speed.y;
}

fn explode_launch(
    p: &mut PlayerSnapshot,
    input: InputState,
    from: Vec2,
    snap_up: bool,
    sides_only: bool,
) -> Vec2 {
    p.freeze_timer = 0.1;
    p.launch_approach_x = None;
    let collider = current_player_hurt_rect(p);
    let center = Vec2::new(
        collider.x + collider.width * 0.5,
        collider.y + collider.height * 0.5,
    );
    let delta = Vec2::new(center.x - from.x, center.y - from.y);
    let mut direction = if delta == Vec2::default() {
        Vec2::new(0.0, -1.0)
    } else {
        normalize(delta)
    };
    let vertical_dot = direction.y;
    if snap_up && vertical_dot <= -0.7 {
        direction = Vec2::new(0.0, -1.0);
    } else if (-0.55..=0.65).contains(&vertical_dot) {
        direction = Vec2::new(direction.x.signum(), 0.0);
    }
    if sides_only && direction.x != 0.0 {
        direction = Vec2::new(direction.x.signum(), 0.0);
    }
    p.speed = scale(direction, 280.0);
    if p.speed.y <= 50.0 {
        p.speed.y = p.speed.y.min(-150.0);
        p.auto_jump = true;
    }
    if p.speed.x != 0.0 {
        if input.move_x as f32 == p.speed.x.signum() {
            p.explode_launch_boost_timer = 0.0;
            p.speed.x *= 1.2;
        } else {
            p.explode_launch_boost_timer = 0.01;
            p.explode_launch_boost_speed = p.speed.x * 1.2;
        }
    }
    // Player.ExplodeLaunch: `if (!Inventory.NoRefills) RefillDash();` (Player.cs).
    if !p.no_refills {
        refill_dash(p);
    }
    p.stamina = 110.0;
    p.dash_cooldown_timer = DASH_COOLDOWN;
    p.state = PlayerState::Launch;
    p.launched = true;
    direction
}

fn bounce(p: &mut PlayerSnapshot, map: &Map, from_y: f32) {
    let restore_star_fly_hitbox = p.state == PlayerState::StarFly || p.star_fly_hitbox_preserved;
    let restore_duck_hitbox = !restore_star_fly_hitbox && p.ducking;

    // Player.Bounce temporarily assigns normalHitbox before MoveVExact, so
    // the correction uses Madeline's ordinary 8x11 body even when a feather
    // or crouched collider entered the callback.
    // `Actor.MoveV(amount)` accumulates the amount into `movementCounter` and moves
    // `round(counter + amount)` whole pixels, so a fractional amount must be carried rather than
    // truncated: `Player.SuperBounce`/`SideBounce` move exactly this way (`Player.cs:2717`, `:2749`),
    // with the default null collide callback.
    p.movement_remainder.y += from_y - p.pos.y;
    let move_y = p.movement_remainder.y.round_ties_even() as i32;
    p.movement_remainder.y -= move_y as f32;
    let sign = move_y.signum();
    for _ in 0..move_y.unsigned_abs() {
        let next_y = p.pos.y + sign as f32;
        if map.non_dream_solid_at(player_rect(p.pos.x, next_y))
            || map.dream_block_at(player_rect(p.pos.x, next_y))
        {
            p.movement_remainder.y = 0.0;
            break;
        }
        p.pos.y = next_y;
    }

    // Player.DreamDashEnd (from inside a DreamBlock): the source performs
    // `if (!Inventory.NoRefills) RefillDash(); RefillStamina();` (Player.cs).
    if !p.no_refills {
        refill_dash(p);
    }
    p.stamina = 110.0;
    if p.state == PlayerState::StarFly {
        // Setting StateMachine.State to Normal invokes StarFlyEnd first. The
        // cached collider is restored only after that callback returns.
        end_star_fly(p, map);
    }
    p.state = PlayerState::Normal;
    p.star_fly_hitbox_preserved = restore_star_fly_hitbox;
    p.ducking = restore_duck_hitbox;
    p.jump_grace_timer = 0.0;
    p.var_jump_timer = VAR_JUMP_TIME;
    p.auto_jump = true;
    p.auto_jump_timer = 0.1;
    p.dash_attack_timer = 0.0;
    p.wall_slide_timer = WALL_SLIDE_TIME;
    p.wall_boost_timer = 0.0;
    p.var_jump_speed = -140.0;
    p.speed.y = -140.0;
    p.launched = false;
}

fn try_begin_badeline_boost(p: &mut PlayerSnapshot, map: &mut Map) -> bool {
    if !player_in_control(p.state) {
        return false;
    }
    let player_box = current_player_rect(p, p.pos.x, p.pos.y);
    let Some((entity_origin, current_position, node_count)) =
        map.entities.iter().find_map(|entity| {
            if entity.kind != EntityKind::BadelineBoost {
                return None;
            }
            let origin = Vec2::new(
                entity.bounds.x + entity.bounds.width * 0.5,
                entity.bounds.y + entity.bounds.height * 0.5,
            );
            let current = if p.badeline_boost_stage > 0 && p.badeline_boost_entity_origin == origin
            {
                if !p.badeline_boost_collidable {
                    return None;
                }
                p.badeline_boost_current_position
            } else {
                origin
            };
            circle_rect_intersects(current, entity.bounds.width * 0.5, player_box).then_some((
                origin,
                current,
                entity.nodes.len(),
            ))
        })
    else {
        return false;
    };
    if p.badeline_boost_stage == 0 || p.badeline_boost_entity_origin != entity_origin {
        p.badeline_boost_entity_origin = entity_origin;
        p.badeline_boost_current_position = current_position;
        p.badeline_boost_stage = 0;
    }
    p.badeline_boost_stage += 1;
    let offset_x = p.pos.x - current_position.x;
    let side = if offset_x == 0.0 {
        -1.0
    } else {
        offset_x.signum()
    };
    p.state = PlayerState::Dummy;
    p.speed = Vec2::default();
    // BadelineBoost.RefillRoutine: a two-dash session is cut back to one dash,
    // otherwise `RefillDash()` runs (BadelineBoost.cs:145-152).
    if p.max_dashes > 1 {
        p.dashes = 1;
    } else {
        refill_dash(p);
    }
    p.stamina = 110.0;
    p.facing = side < 0.0;
    p.badeline_boost_active = true;
    p.badeline_boost_final = p.badeline_boost_stage as usize > node_count;
    p.badeline_boost_phase = 0;
    p.badeline_boost_frame = 0;
    p.badeline_boost_relocating = false;
    p.badeline_boost_collidable = false;
    p.badeline_boost_start = p.pos;
    p.badeline_boost_target = Vec2::new(current_position.x + side * 4.0, current_position.y - 3.0);
    p.last_badeline_boost_target = current_position;
    let start = p.badeline_boost_start;
    move_to_position(p, map, start);
    p.movement_remainder = Vec2::default();
    true
}

fn advance_badeline_boost_relocation(p: &mut PlayerSnapshot) {
    if !p.badeline_boost_relocating {
        return;
    }
    p.badeline_boost_relocation_elapsed += p.frame_delta_time;
    let progress = if p.badeline_boost_relocation_duration <= 0.0 {
        1.0
    } else {
        (p.badeline_boost_relocation_elapsed / p.badeline_boost_relocation_duration).min(1.0)
    };
    let eased = 0.5 - (progress * std::f32::consts::PI).cos() * 0.5;
    p.badeline_boost_current_position = Vec2::new(
        p.badeline_boost_relocation_from.x
            + (p.badeline_boost_relocation_to.x - p.badeline_boost_relocation_from.x) * eased,
        p.badeline_boost_relocation_from.y
            + (p.badeline_boost_relocation_to.y - p.badeline_boost_relocation_from.y) * eased,
    );
    if progress >= 1.0 {
        p.badeline_boost_current_position = p.badeline_boost_relocation_to;
        p.badeline_boost_relocating = false;
        p.badeline_boost_collidable = true;
    }
}

fn badeline_boost_entity<'a>(p: &PlayerSnapshot, map: &'a Map) -> Option<&'a crate::Entity> {
    map.entities.iter().find(|entity| {
        entity.kind == EntityKind::BadelineBoost
            && Vec2::new(
                entity.bounds.x + entity.bounds.width * 0.5,
                entity.bounds.y + entity.bounds.height * 0.5,
            ) == p.badeline_boost_entity_origin
    })
}

fn update_badeline_boost(p: &mut PlayerSnapshot, map: &mut Map) {
    let wait_frames = if p.badeline_boost_final { 12 } else { 6 };
    match p.badeline_boost_phase {
        0 if p.badeline_boost_frame < 11 => {
            let progress = (p.badeline_boost_frame as f32 + 1.0) * p.frame_delta_time / 0.2;
            let target = Vec2::new(
                p.badeline_boost_start.x
                    + (p.badeline_boost_target.x - p.badeline_boost_start.x) * progress,
                p.badeline_boost_start.y
                    + (p.badeline_boost_target.y - p.badeline_boost_start.y) * progress,
            );
            move_to_position(p, map, target);
            p.badeline_boost_frame += 1;
        }
        0 => {
            p.badeline_boost_phase = 1;
            p.badeline_boost_frame = 0;
        }
        1 if p.badeline_boost_frame < wait_frames => {
            p.badeline_boost_frame += 1;
        }
        1 => {
            move_axis_amount(p, map, false, 5.0);
            p.badeline_boost_phase = 2;
            p.badeline_boost_frame = 0;
        }
        2 if p.badeline_boost_frame < wait_frames => {
            p.badeline_boost_frame += 1;
        }
        2 if p.badeline_boost_final => {
            p.freeze_timer = 0.1;
            p.badeline_boost_phase = 3;
            p.badeline_boost_frame = 0;
        }
        2 => begin_badeline_launch(p, map),
        3 => begin_badeline_summit_launch(p),
        _ => {}
    }
}

fn begin_badeline_launch(p: &mut PlayerSnapshot, map: &mut Map) {
    p.badeline_boost_active = false;
    p.launch_approach_x = Some(p.last_badeline_boost_target.x);
    p.speed = Vec2::new(0.0, -330.0);
    p.auto_jump = true;
    // Player.BadelineBoostLaunch: RefillDash(); RefillStamina(); (Player.cs).
    refill_dash(p);
    p.stamina = 110.0;
    p.dash_cooldown_timer = DASH_COOLDOWN;
    p.state = PlayerState::Launch;
    p.launched = true;
    if let Some(entity) = badeline_boost_entity(p, map)
        && let Some(target) = entity
            .nodes
            .get(p.badeline_boost_stage.saturating_sub(1) as usize)
            .copied()
    {
        p.badeline_boost_relocation_from = p.badeline_boost_current_position;
        p.badeline_boost_relocation_to = target;
        p.badeline_boost_relocation_elapsed = 0.0;
        p.badeline_boost_relocation_duration = (length(Vec2::new(
            target.x - p.badeline_boost_current_position.x,
            target.y - p.badeline_boost_current_position.y,
        )) / 320.0)
            .min(3.0);
        p.badeline_boost_relocating = true;
        p.badeline_boost_collidable = false;
    }
}

fn begin_badeline_summit_launch(p: &mut PlayerSnapshot) {
    p.badeline_boost_active = false;
    p.summit_launch_target_x = p.last_badeline_boost_target.x;
    p.wall_boost_timer = 0.0;
    p.speed = Vec2::new(0.0, -DASH_SPEED);
    p.summit_launch_particle_timer = 0.4;
    p.state = PlayerState::SummitLaunch;
}

fn move_to_position(p: &mut PlayerSnapshot, map: &mut Map, target: Vec2) {
    let exact_x = p.pos.x + p.movement_remainder.x;
    move_axis_amount(p, map, true, target.x - exact_x);
    let exact_y = p.pos.y + p.movement_remainder.y;
    move_axis_amount(p, map, false, target.y - exact_y);
}

fn point_bounce(p: &mut PlayerSnapshot, from: Vec2) {
    if p.state == PlayerState::Dash {
        p.state = PlayerState::Normal;
    }
    // Player.PointBounce: RefillDash(); RefillStamina(); (Player.cs).
    refill_dash(p);
    p.stamina = 110.0;
    let collider = current_player_hurt_rect(p);
    let center = Vec2::new(
        collider.x + collider.width * 0.5,
        collider.y + collider.height * 0.5,
    );
    // Player.PointBounce uses the regular actor center without an artificial
    // vertical-angle clamp. The horizontal multiplier and minimum are applied
    // after SafeNormalize, which matters for the nearly level Seeker contact.
    p.speed = scale(
        normalize(Vec2::new(center.x - from.x, center.y - from.y)),
        200.0,
    );
    p.speed.x *= 1.2;
    if p.speed.x.abs() < 120.0 {
        p.speed.x = if p.speed.x == 0.0 {
            if p.facing { -120.0 } else { 120.0 }
        } else {
            p.speed.x.signum() * 120.0
        };
    }
}

fn enforce_level_bounds(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    attachments: &mut Vec<Option<StaticMoverAttachment>>,
    room: &mut RoomCoroutineState,
) {
    if p.dead || p.state == PlayerState::DreamDash || !player_in_control(p.state) {
        return;
    }
    let bounds = p.current_room_bounds.unwrap_or(map.bounds);
    let mut collider = current_player_rect(p, p.pos.x, p.pos.y);
    if collider.x < bounds.x {
        let center = Vec2::new(p.pos.x, collider.y + collider.height * 0.5);
        if let Some(next) = transition_room_at(map, p, Vec2::new(center.x - 8.0, center.y)) {
            begin_transition(p, map, next, Vec2::new(-1.0, 0.0), attachments, room);
            return;
        }
        p.pos.x += bounds.x - collider.x;
        p.speed.x = 0.0;
        collider = current_player_rect(p, p.pos.x, p.pos.y);
    }
    let right = bounds.right();
    if collider.x + collider.width > right {
        let center = Vec2::new(p.pos.x, collider.y + collider.height * 0.5);
        if let Some(next) = transition_room_at(map, p, Vec2::new(center.x + 8.0, center.y)) {
            begin_transition(p, map, next, Vec2::new(1.0, 0.0), attachments, room);
            return;
        }
        p.pos.x -= collider.x + collider.width - right;
        p.speed.x = 0.0;
        collider = current_player_rect(p, p.pos.x, p.pos.y);
    }

    let top = bounds.y;
    let center_y = collider.y + collider.height * 0.5;
    if center_y < top {
        let center = Vec2::new(p.pos.x, center_y);
        if let Some(next) = transition_room_at(map, p, Vec2::new(center.x, center.y - 12.0)) {
            begin_transition(p, map, next, Vec2::new(0.0, -1.0), attachments, room);
            return;
        }
    }
    if center_y < top && collider.y < top - 24.0 {
        p.pos.y += top - 24.0 - collider.y;
        p.speed.y = 0.0;
        collider = current_player_rect(p, p.pos.x, p.pos.y);
    }

    let bottom = bounds.bottom();
    if collider.bottom() > bottom {
        let center = Vec2::new(p.pos.x, collider.y + collider.height * 0.5);
        if let Some(next) = transition_room_at(map, p, Vec2::new(center.x, center.y + 12.0)) {
            begin_transition(p, map, next, Vec2::new(0.0, 1.0), attachments, room);
            return;
        }
    }
    if collider.y > bottom + 4.0 {
        p.dead = true;
        p.speed = Vec2::default();
        p.death_freeze_pending = true;
        p.respawn_frames = 95;
    }
}

fn transition_room_at(map: &Map, p: &PlayerSnapshot, point: Vec2) -> Option<Rect> {
    let current = p.current_room_bounds.unwrap_or(map.bounds);
    std::iter::once(map.bounds)
        .chain(map.transition_rooms.iter().copied())
        .filter(|room| *room != current)
        .find(|room| {
            point.x >= room.x
                && point.x < room.right()
                && point.y >= room.y
                && point.y < room.bottom()
        })
}

fn load_transition_room(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    next: Rect,
    attachments: &mut Vec<Option<StaticMoverAttachment>>,
    room_state: &mut RoomCoroutineState,
) {
    let Some(room) = map
        .transition_runtime
        .iter()
        .find(|room| room.bounds == next)
        .cloned()
    else {
        return;
    };

    // Level.TransitionRoutine changes Session.Level and calls LoadLevel before
    // yielding the camera transition. CassetteBlockManager is Global, so the
    // destination blocks must be silently initialized from its existing index
    // in this same scene frame, before its subsequent Update can WillToggle.
    map.solids = room.solids;
    map.entities = room.entities;
    map.room_spawns = room.spawns;
    // The destination room's own `LoadSeed`: `Level.LoadLevel` re-pushes `Calc.Random` from it
    // (`Level.cs:386`), so `FloatySpaceBlock`'s constructor draws differ from the source room's.
    map.load_seed = room.load_seed;
    *attachments = initialize_static_mover_attachments(map);

    // Level.LoadLevel constructs every destination room entity before the
    // transition coroutine yields (Level.cs:468+). Their Added/Awake state is
    // therefore live on the next Scene update. All vectors below are indexed
    // by the current room's entity order, so retaining the source room vectors
    // can either associate state with the wrong entity or panic when the
    // destination contains more entities of a kind.
    p.zip_movers.clear();
    p.bounce_blocks.clear();
    p.move_blocks.clear();
    p.theo_crystals.clear();
    p.heart_gems.clear();
    p.rising_lavas.clear();
    p.sandwich_lavas.clear();
    p.gliders.clear();
    p.clouds.clear();
    p.seekers.clear();
    p.temple_gates.clear();
    p.cassette_blocks.clear();
    p.spinners.clear();
    p.bumpers.clear();
    p.refills.clear();
    p.falling_blocks.clear();
    p.exit_blocks.clear();
    p.invisible_barriers.clear();
    p.killboxes.clear();
    // A held actor is a follower that vanilla LoadLevel omits from the new
    // room's EntityData loop. The portable map-order representation cannot yet
    // carry that persistent actor across rooms, so clear the stale index rather
    // than aliasing it to an unrelated destination entity.
    p.holding_theo = None;
    p.holding_glider = None;

    initialize_zip_movers(p, map);
    initialize_bounce_blocks(p, map);
    initialize_move_blocks(p, map);
    initialize_theo_crystals(p, map);
    initialize_heart_gems(p, map);
    initialize_rising_lavas(p, map);
    initialize_sandwich_lavas(p, map);
    initialize_gliders(p, map);
    initialize_clouds(p, map);
    initialize_seekers(p, map);
    initialize_puffers(p, map);
    initialize_temple_gates(p, map);
    initialize_core_mode_toggles(p, map);
    initialize_cassette_blocks(p, map);
    initialize_spinners(p, map);
    initialize_bumpers(p, map);
    initialize_refills(p, map);
    initialize_falling_blocks(p, map);
    initialize_exit_blocks(p, map);
    initialize_invisible_barriers(p, map);
    initialize_killboxes(p, map);
    // `FloatySpaceBlock`s are per-room state too: their group membership is built from the
    // destination room's own entity list and their phase restarts with the new room's `Awake`
    // (`FloatySpaceBlock.cs:65-105`, `:50-57`). Keeping the source room's groups would index the
    // destination room's entities at the wrong offsets.
    room_state.floaty_blocks = initialize_floaty_blocks(map);
    position_moving_solids(map, p.moving_solid_time);
    sync_all_platform_static_movers(p, map, attachments);
}

fn begin_transition(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    next: Rect,
    direction: Vec2,
    attachments: &mut Vec<Option<StaticMoverAttachment>>,
    room: &mut RoomCoroutineState,
) {
    if direction.y > 0.0
        && !matches!(
            p.state,
            PlayerState::RedDash | PlayerState::ReflectionFall | PlayerState::StarFly
        )
    {
        p.state = PlayerState::Normal;
        p.speed.y = p.speed.y.max(0.0);
        p.auto_jump = false;
        p.var_jump_timer = 0.0;
    } else if direction.y < 0.0 {
        p.speed.x = 0.0;
        if !matches!(
            p.state,
            PlayerState::RedDash | PlayerState::ReflectionFall | PlayerState::StarFly
        ) {
            p.state = PlayerState::Normal;
            p.speed.y = JUMP_SPEED;
            p.var_jump_speed = p.speed.y;
            p.auto_jump = true;
            p.auto_jump_timer = 0.0;
            p.var_jump_timer = VAR_JUMP_TIME;
        }
        p.dash_cooldown_timer = 0.2;
    }

    let mut target = p.pos;
    if direction.x > 0.0 {
        target.x = next.x + 4.0;
    } else if direction.x < 0.0 {
        target.x = next.right() - 5.0;
    } else if direction.y > 0.0 {
        target.y = next.y + 12.0;
    } else {
        target.y = next.bottom() - 5.0;
    }
    p.transition_room_bounds = Some(next);
    p.transition_direction = direction;
    p.transition_target = target;
    load_transition_room(p, map, next, attachments, room);
    // TransitionRoutine updates cameraAt after yielding, then resumes once
    // more to observe cameraAt == 1 and run OnTransition. Preserve that final
    // coroutine-resume frame in addition to the 0.65-second camera duration.
    p.transition_timer = TRANSITION_TIME + p.frame_delta_time;
}

fn update_transition(p: &mut PlayerSnapshot, map: &mut Map) {
    let max_move = TRANSITION_MOVE_SPEED * p.frame_delta_time;
    p.pos.x = approach(p.pos.x, p.transition_target.x, max_move);
    p.pos.y = approach(p.pos.y, p.transition_target.y, max_move);
    p.transition_timer = (p.transition_timer - p.frame_delta_time).max(0.0);
    // Player.TransitionTo rounds speed and clears Actor remainders as soon as
    // the player reaches the transfer target; the camera coroutine can keep
    // the room transition open for many more frames after that return value.
    if p.pos == p.transition_target {
        p.movement_remainder = Vec2::default();
        p.speed.x = p.speed.x.round();
        p.speed.y = p.speed.y.round();
    }
    if p.transition_timer <= 0.0 && p.pos == p.transition_target {
        p.wall_slide_timer = WALL_SLIDE_TIME;
        p.jump_grace_timer = 0.0;
        p.force_move_x_timer = 0.0;
        // Player.OnTransition: RefillDash(); RefillStamina(); (Player.cs).
        refill_dash(p);
        p.stamina = 110.0;
        let previous_room = Some(p.current_room_bounds.unwrap_or(map.bounds));
        let next_room = p.transition_room_bounds.take();
        if let (Some(previous), Some(next)) = (previous_room, next_room) {
            for lookout in &mut p.lookouts {
                let was_in_room = lookout.position.x >= previous.x
                    && lookout.position.x < previous.right()
                    && lookout.position.y >= previous.y
                    && lookout.position.y < previous.bottom();
                let remains_in_room = lookout.position.x >= next.x
                    && lookout.position.x < next.right()
                    && lookout.position.y >= next.y
                    && lookout.position.y < next.bottom();
                if lookout.interacting && was_in_room && !remains_in_room {
                    // Lookout.Removed restores StNormal but does not call
                    // StopInteracting, preserving the storage flag.
                    lookout.removed = true;
                    p.state = PlayerState::Normal;
                    p.dummy_moving = false;
                }
            }
            // LoadLevel already installed the destination entities when the
            // transition started. Completion only selects its respawn point
            // and clears source-room entity state that was retained for
            // transition-removal callbacks.
            if let Some(room) = map
                .transition_runtime
                .iter()
                .find(|room| room.bounds == next)
            {
                if let Some(spawn) = room.spawns.iter().copied().min_by(|left, right| {
                    let left_dx = left.x - p.pos.x;
                    let left_dy = left.y - p.pos.y;
                    let right_dx = right.x - p.pos.x;
                    let right_dy = right.y - p.pos.y;
                    (left_dx * left_dx + left_dy * left_dy)
                        .partial_cmp(&(right_dx * right_dx + right_dy * right_dy))
                        .unwrap_or(std::cmp::Ordering::Equal)
                }) {
                    // Level.LoadLevel assigns Session.RespawnPoint from this
                    // room's closest player spawn after the transfer.
                    map.spawn = spawn;
                }
                p.lookouts.clear();
                initialize_lookouts(p, map);
            }
        }
        p.current_room_bounds = next_room;
        p.transition_direction = Vec2::default();
    }
}

/// `Level.EnforceBounds`' transition clauses (`Level.cs:2738-2806`), read from the destination
/// room's side: the player's collider still sticks out of the room it has just entered, and the
/// bound it crosses is the direction the transition travels in. `Vector2.Zero` means the collider is
/// already inside, i.e. the coroutine is in the parked phase of
/// `while (!player.TransitionTo(...) || cameraAt < 1f)` - `Player.Update` is still suspended, but
/// there is nothing left to move.
fn transition_entry_direction(p: &PlayerSnapshot, bounds: Rect) -> Vec2 {
    let collider = current_player_rect(p, p.pos.x, p.pos.y);
    let center_y = collider.y + collider.height * 0.5;
    if collider.x < bounds.x {
        // `player.Left < bounds.Left` (`Level.cs:2738`): entered from the left, travelling right.
        Vec2::new(1.0, 0.0)
    } else if collider.x + collider.width > bounds.right() {
        // `player.Right > bounds.Right` (`Level.cs:2759`).
        Vec2::new(-1.0, 0.0)
    } else if center_y < bounds.y {
        // `player.CenterY < bounds.Top` (`Level.cs:2775`): the destination room is below, so the
        // player starts above its top bound. `NextLevel(..., Vector2.UnitY)`.
        Vec2::new(0.0, 1.0)
    } else if center_y >= bounds.bottom() {
        // `player.Bottom > bounds.Bottom` (`Level.cs:2801`): the destination room is above.
        Vec2::new(0.0, -1.0)
    } else {
        Vec2::default()
    }
}

/// `Level.TransitionRoutine`'s `playerTo` target (`Level.cs:1521-1528`), derived from the room the
/// player is entering rather than from the player's own position.
///
/// The source walks `playerTo` from `player.Position` as it was when the coroutine first ran:
///
/// ```csharp
/// Vector2 playerTo = player.Position;
/// while (direction.X != 0f && playerTo.Y >= (float)Bounds.Bottom) playerTo.Y -= 1f;
/// for (; !IsInBounds(playerTo, dirPad); playerTo += direction) { }
/// ```
///
/// so its target is "the bound being entered, adjusted by `dirPad`" plus the fractional part of that
/// start position. A replayed window cannot see the coroutine's first frame - it belongs to the
/// segment for the room being left, and the row itself is leading-skipped - but the two agree
/// wherever the target is observable: the crossing coordinate is exactly the adjusted bound
/// (`IsInBounds(Vector2, Vector2)`, `Level.cs:2850-2862`), and a fractional part can only differ when
/// `Player.Position` is itself fractional. Measured over `trace-202-v7.jsonl`: 2874 of 438001 level
/// rows carry a fractional `Position` (`StCassetteFly` and friends, which tween `Position`
/// directly), and **none** of its 54521 `transitioning` rows does - the creep only ever moves whole
/// pixels, so the adjusted bound *is* the engine's number on every replayed transition row.
///
/// `dirPad` is `direction * 4f` for a side or upward transition and `direction * 12f` for a
/// downward one (`Level.cs:1516-1520`); `playerTo` is the player's bottom-centre position, so
/// `Bounds.Left + 4f` is the same statement as "the collider's left edge on the bound".
fn transition_entry_target(from: Vec2, direction: Vec2, bounds: Rect) -> Vec2 {
    if direction == Vec2::default() {
        // The collider is already inside the room, so the source's search returns `playerTo`
        // unchanged: the coroutine is in its parked phase, waiting only for its camera clock, and
        // `Player.Update` stays suspended where the player already is.
        return from;
    }
    // Seed the walk on the bound being entered instead of on the player's own position. Only the
    // axis the transition travels along is constrained by `IsInBounds`; the other one is the
    // player's and is kept as it was.
    let mut target = if direction.x > 0.0 {
        Vec2::new(bounds.x, from.y)
    } else if direction.x < 0.0 {
        Vec2::new(bounds.right(), from.y)
    } else if direction.y > 0.0 {
        Vec2::new(from.x, bounds.y)
    } else {
        Vec2::new(from.x, bounds.bottom())
    };
    // `Level.TransitionRoutine`'s pre-loop (`Level.cs:1522-1525`): before the walk, `playerTo.Y` is
    // lifted back inside the room while it sits at or below `Bounds.Bottom`. One pixel per
    // iteration, so it always terminates.
    while direction.x != 0.0 && target.y >= bounds.bottom() {
        target.y -= 1.0;
    }
    let dir_pad = if direction == Vec2::new(0.0, 1.0) {
        Vec2::new(0.0, TRANSITION_ENTRY_DOWN_PAD)
    } else {
        Vec2::new(
            direction.x * TRANSITION_ENTRY_PAD,
            direction.y * TRANSITION_ENTRY_PAD,
        )
    };
    let mut steps = 0usize;
    while !transition_position_in_bounds(target, dir_pad, bounds) {
        // The source's `for` has no bound; every real room terminates in a few hundred whole-pixel
        // steps, so the cap only turns a malformed room into a parked transition instead of a hang.
        if steps >= TRANSITION_TARGET_MAX_STEPS {
            return from;
        }
        target.x += direction.x;
        target.y += direction.y;
        steps += 1;
    }
    target
}

/// `Level.IsInBounds(Vector2 position, Vector2 dirPad)` (`Level.cs:2850-2862`): the pad only widens
/// the room on the side the transition travels away from, which is why a leftward entry stops at
/// `Bounds.Right - 5f` while a rightward one stops at `Bounds.Left + 4f`.
fn transition_position_in_bounds(position: Vec2, dir_pad: Vec2, bounds: Rect) -> bool {
    let left = dir_pad.x.max(0.0);
    let right = (-dir_pad.x).max(0.0);
    let top = dir_pad.y.max(0.0);
    let bottom = (-dir_pad.y).max(0.0);
    position.x >= bounds.x + left
        && position.y >= bounds.y + top
        && position.x < bounds.right() - right
        && position.y < bounds.bottom() - bottom
}

fn move_towards_x(p: &mut PlayerSnapshot, map: &mut Map, target_x: f32, max_move: f32) {
    let exact_x = p.pos.x + p.movement_remainder.x;
    let next_x = approach(exact_x, target_x, max_move);
    move_axis_amount(p, map, true, next_x - exact_x);
}

fn approach_exact_position(p: &mut PlayerSnapshot, map: &mut Map, target: Vec2, max_move: f32) {
    let exact = Vec2::new(
        p.pos.x + p.movement_remainder.x,
        p.pos.y + p.movement_remainder.y,
    );
    let dx = target.x - exact.x;
    let dy = target.y - exact.y;
    let distance = (dx * dx + dy * dy).sqrt();
    let next = if distance <= max_move || distance == 0.0 {
        target
    } else {
        Vec2::new(
            exact.x + dx / distance * max_move,
            exact.y + dy / distance * max_move,
        )
    };
    move_axis_amount(p, map, true, next.x - exact.x);
    move_axis_amount(p, map, false, next.y - exact.y);
}

fn snap_to_boost_target(p: &mut PlayerSnapshot, map: &mut Map) {
    let center_offset_y = if p.ducking { 3.0 } else { 5.5 };
    let target = Vec2::new(
        p.boost_target.x.floor(),
        (p.boost_target.y + center_offset_y).floor(),
    );
    let exact_x = p.pos.x + p.movement_remainder.x;
    move_axis_amount(p, map, true, target.x - exact_x);
    let exact_y = p.pos.y + p.movement_remainder.y;
    move_axis_amount(p, map, false, target.y - exact_y);
}

fn move_exact(p: &mut PlayerSnapshot, map: &Map, horizontal: bool, sign: i32) -> bool {
    let next_x = p.pos.x + if horizontal { sign as f32 } else { 0.0 };
    let next_y = p.pos.y + if horizontal { 0.0 } else { sign as f32 };
    let next = current_player_rect(p, next_x, next_y);
    let collided = map.non_dream_solid_at(next)
        || (map.dream_block_at(next) && p.state != PlayerState::DreamDash)
        || (!horizontal
            && sign > 0
            && !p.ignore_jump_thrus
            && map.jump_thru_at(next, current_player_rect(p, p.pos.x, p.pos.y).bottom()));
    if collided {
        false
    } else {
        p.pos.x = next_x;
        p.pos.y = next_y;
        true
    }
}

fn advance_wind_controller(p: &mut PlayerSnapshot) {
    p.wind.x = approach(p.wind.x, p.wind_target.x, WIND_ACCEL * p.frame_delta_time);
    p.wind.y = approach(p.wind.y, p.wind_target.y, WIND_ACCEL * p.frame_delta_time);
}

fn apply_wind_movement(p: &mut PlayerSnapshot, map: &mut Map) {
    if !player_in_control(p.state)
        || p.no_wind_timer > 0.0
        || matches!(
            p.state,
            PlayerState::Boost | PlayerState::Dash | PlayerState::SummitLaunch
        )
    {
        return;
    }

    let mut move_x = p.wind.x * WIND_MOVE_MULT * p.frame_delta_time;
    if move_x != 0.0 && p.state != PlayerState::Climb {
        let shield_x = p.pos.x - move_x.signum() * WIND_WALL_DISTANCE;
        if !map.solid_at(current_player_rect(p, shield_x, p.pos.y)) {
            // Player.WindMove gates the horizontal push on `Ducking && onGround`
            // (Player.cs:3095), and that `onGround` is the source-private field
            // Player.Update writes behind its `Speed.Y >= 0f` probe (Player.cs:1499-1526),
            // not `Actor.OnGround()`. WindController.Update runs before Player.Update in
            // the frame, so the value it reads is the previous frame's probe: exactly
            // PlayerSnapshot::player_on_ground at this point in step() (it is refreshed
            // later, in the Player.Update mirror). The geometric `p.on_ground` is a
            // different quantity and made the simulator drop a wind push the game applied
            // while the player was rising off a ledge with the duck collider active.
            if p.ducking && p.player_on_ground {
                move_x = 0.0;
            }
            move_axis_amount_silent(p, map, true, move_x);
        }
    }

    let mut move_y = p.wind.y * WIND_MOVE_MULT * p.frame_delta_time;
    if move_y != 0.0 && (p.speed.y < 0.0 || !grounded(p, map)) {
        if p.state == PlayerState::Climb {
            if move_y > 0.0 && p.climb_no_move_timer <= 0.0 {
                move_y *= 0.4;
            } else {
                return;
            }
        }
        move_axis_amount_silent(p, map, false, move_y);
    }
}

fn player_in_control(state: PlayerState) -> bool {
    !matches!(
        state,
        PlayerState::Dummy
            | PlayerState::IntroWalk
            | PlayerState::IntroJump
            | PlayerState::IntroRespawn
            | PlayerState::IntroWakeUp
            | PlayerState::BirdDashTutorial
            | PlayerState::Frozen
            | PlayerState::IntroMoonJump
            | PlayerState::IntroThinkForABit
    )
}

fn spike_is_lethal(p: &PlayerSnapshot, direction: Vec2, bounds: Rect) -> bool {
    if direction.y < 0.0 {
        // Spikes.OnCollide additionally checks Player.Bottom against the
        // three-pixel upward spike collider, so touching it from below is not
        // lethal even while falling.
        p.speed.y >= 0.0 && current_player_hurt_rect(p).bottom() <= bounds.bottom()
    } else if direction.y > 0.0 {
        p.speed.y <= 0.0
    } else if direction.x < 0.0 {
        p.speed.x >= 0.0
    } else if direction.x > 0.0 {
        p.speed.x <= 0.0
    } else {
        false
    }
}

fn dash_through_spikes_pass(p: &PlayerSnapshot) -> bool {
    p.dash_dir != Vec2::default()
        && (matches!(
            p.state,
            PlayerState::Dash | PlayerState::DreamDash | PlayerState::RedDash
        ) || (p.dash_attack_timer > 0.0 && p.state != PlayerState::RedDash))
}

fn approach(value: f32, target: f32, max_move: f32) -> f32 {
    if value < target {
        (value + max_move).min(target)
    } else {
        (value - max_move).max(target)
    }
}

/// `Math.Sign(float)` from the BCL: -1, 0 or +1.
///
/// Rust's `f32::signum` returns +1 for +0.0 and -1 for -0.0 instead, so every
/// `Player.cs` branch that compares `Math.Sign(x)` against another sign value
/// must go through this helper.
fn math_sign(value: f32) -> f32 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        0.0
    }
}

fn dot(a: Vec2, b: Vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

fn circle_rect_intersects(center: Vec2, radius: f32, rect: Rect) -> bool {
    let nearest_x = center.x.clamp(rect.x, rect.x + rect.width);
    let nearest_y = center.y.clamp(rect.y, rect.y + rect.height);
    let dx = center.x - nearest_x;
    let dy = center.y - nearest_y;
    dx * dx + dy * dy < radius * radius
}

fn length(value: Vec2) -> f32 {
    (value.x * value.x + value.y * value.y).sqrt()
}

fn normalize(value: Vec2) -> Vec2 {
    let len = length(value);
    if len == 0.0 {
        Vec2::default()
    } else {
        Vec2::new(value.x / len, value.y / len)
    }
}

fn scale(value: Vec2, amount: f32) -> Vec2 {
    Vec2::new(value.x * amount, value.y * amount)
}

fn approach_vector(value: Vec2, target: Vec2, max_move: f32) -> Vec2 {
    let delta = Vec2::new(target.x - value.x, target.y - value.y);
    let distance = length(delta);
    if distance <= max_move || distance == 0.0 {
        target
    } else {
        Vec2::new(
            value.x + delta.x / distance * max_move,
            value.y + delta.y / distance * max_move,
        )
    }
}

fn angle(value: Vec2) -> f32 {
    value.y.atan2(value.x)
}

fn rotate_towards(value: Vec2, target_angle: f32, max_move: f32) -> Vec2 {
    let current_angle = angle(value);
    let mut difference = target_angle - current_angle;
    while difference > std::f32::consts::PI {
        difference -= std::f32::consts::TAU;
    }
    while difference <= -std::f32::consts::PI {
        difference += std::f32::consts::TAU;
    }
    let next_angle = if difference.abs() < max_move {
        target_angle
    } else {
        current_angle + difference.clamp(-max_move, max_move)
    };
    let len = length(value);
    Vec2::new(next_angle.cos() * len, next_angle.sin() * len)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn floor_map() -> Map {
        Map {
            solids: vec![Rect::new(0.0, 100.0, 320.0, 80.0)],
            ..Map::default()
        }
    }
    fn grounded_player() -> PlayerSnapshot {
        PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        }
    }
    fn moving_solid_map(direction: Vec2) -> Map {
        Map {
            entities: vec![crate::Entity {
                kind: EntityKind::MovingSolid,
                bounds: Rect::new(16.0, 100.0, 64.0, 8.0),
                direction,
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "celesteGymMovingSolid".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn lava_map(kind: EntityKind, start_x: f32) -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 640.0, 360.0),
            entities: vec![crate::Entity {
                kind,
                bounds: Rect::new(start_x, 0.0, 8.0, 8.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: match kind {
                    EntityKind::RisingLava => "risingLava",
                    EntityKind::SandwichLava => "sandwichLava",
                    _ => unreachable!(),
                }
                .to_owned(),
            }],
            ..Map::default()
        }
    }
    fn zip_mover_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 544.0),
            entities: vec![crate::Entity {
                kind: EntityKind::ZipMover,
                bounds: Rect::new(32.0, 440.0, 64.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![Vec2::new(32.0, 320.0)],
                name: "zipMover".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn bounce_block_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 240.0),
            entities: vec![crate::Entity {
                kind: EntityKind::BounceBlock,
                bounds: Rect::new(32.0, 160.0, 64.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "bounceBlock".to_owned(),
            }],
            ..Map::default()
        }
    }

    fn cassette_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![
                crate::Entity {
                    kind: EntityKind::CassetteBlock,
                    bounds: Rect::new(64.0, 101.0, 64.0, 16.0),
                    direction: Vec2::new(0.0, 1.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "cassetteBlock".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::CassetteBlock,
                    bounds: Rect::new(192.0, 101.0, 64.0, 16.0),
                    direction: Vec2::new(1.0, 1.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "cassetteBlock".to_owned(),
                },
            ],
            ..Map::default()
        }
    }

    fn spinner_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![crate::Entity {
                kind: EntityKind::CrystalStaticSpinner,
                bounds: Rect::new(92.0, 94.0, 16.0, 12.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "spinner".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn lookout_map(nodes: Vec<Vec2>, summit: bool, spinner: bool) -> Map {
        let mut entities = vec![crate::Entity {
            kind: EntityKind::Lookout,
            bounds: Rect::new(158.0, 156.0, 4.0, 4.0),
            direction: Vec2::new(0.0, if summit { 1.0 } else { 0.0 }),
            shielded: false,
            single_use: false,
            nodes,
            name: "lookout".to_owned(),
        }];
        if spinner {
            entities.push(crate::Entity {
                kind: EntityKind::CrystalStaticSpinner,
                bounds: Rect::new(232.0, 94.0, 16.0, 12.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "spinner".to_owned(),
            });
        }
        Map {
            bounds: Rect::new(0.0, 0.0, 1280.0, 180.0),
            solids: vec![Rect::new(0.0, 160.0, 1280.0, 20.0)],
            entities,
            ..Map::default()
        }
    }
    fn bounce_block_spikes_map() -> Map {
        let mut map = bounce_block_map();
        map.entities.push(crate::Entity {
            kind: EntityKind::Spikes,
            bounds: Rect::new(32.0, 157.0, 64.0, 3.0),
            direction: Vec2::new(0.0, -1.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "spikesUp".to_owned(),
        });
        map
    }
    fn move_block_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 240.0),
            entities: vec![crate::Entity {
                kind: EntityKind::MoveBlock,
                bounds: Rect::new(64.0, 160.0, 32.0, 16.0),
                direction: Vec2::new(1.0, 0.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "moveBlock".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn theo_crystal_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 240.0),
            solids: vec![Rect::new(0.0, 160.0, 320.0, 80.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::TheoCrystal,
                bounds: Rect::new(64.0, 150.0, 8.0, 10.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "theoCrystal".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn glider_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 240.0),
            solids: vec![Rect::new(0.0, 160.0, 320.0, 80.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::Glider,
                bounds: Rect::new(64.0, 150.0, 8.0, 10.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "glider".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn water_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            solids: vec![Rect::new(0.0, 496.0, 960.0, 48.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::Water,
                bounds: Rect::new(448.0, 416.0, 112.0, 80.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "water".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn wind_map() -> Map {
        Map {
            solids: vec![Rect::new(0.0, 100.0, 320.0, 80.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::Wind,
                bounds: Rect::new(0.0, 0.0, 320.0, 120.0),
                direction: Vec2::new(400.0, 0.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "windTrigger".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn feather_map(shielded: bool) -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            entities: vec![crate::Entity {
                kind: EntityKind::FlyFeather,
                bounds: Rect::new(110.0, 190.0, 20.0, 20.0),
                direction: Vec2::default(),
                shielded,
                single_use: false,
                nodes: vec![],
                name: "infiniteStar".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn bumper_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            entities: vec![crate::Entity {
                kind: EntityKind::Bumper,
                bounds: Rect::new(588.0, 188.0, 24.0, 24.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "bigSpinner".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn bumper_clip_map() -> Map {
        let mut map = bumper_map();
        map.solids.push(Rect::new(560.0, 176.0, 16.0, 48.0));
        map
    }
    fn bumper_theo_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 544.0),
            entities: vec![
                crate::Entity {
                    kind: EntityKind::Bumper,
                    bounds: Rect::new(88.0, 88.0, 24.0, 24.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "bigSpinner".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::TheoCrystal,
                    bounds: Rect::new(96.0, 78.0, 8.0, 10.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "theoCrystal".to_owned(),
                },
            ],
            ..Map::default()
        }
    }
    fn ice_ball_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            entities: vec![crate::Entity {
                kind: EntityKind::IceBall,
                bounds: Rect::new(94.0, 94.0, 12.0, 12.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: true,
                nodes: vec![Vec2::new(116.0, 100.0)],
                name: "fireBall".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn bounce_actor_map(kind: EntityKind) -> Map {
        let bounds = match kind {
            EntityKind::Puffer => Rect::new(94.0, 94.0, 12.0, 10.0),
            EntityKind::AngryOshiro => Rect::new(86.0, 94.0, 28.0, 28.0),
            EntityKind::Seeker => Rect::new(94.0, 96.0, 12.0, 12.0),
            EntityKind::Snowball => Rect::new(94.0, 94.0, 12.0, 9.0),
            _ => panic!("not a top-bounce actor"),
        };
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            entities: vec![crate::Entity {
                kind,
                bounds,
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: String::new(),
            }],
            ..Map::default()
        }
    }
    fn cloud_map(with_spikes: bool) -> Map {
        let mut entities = vec![crate::Entity {
            kind: EntityKind::Cloud,
            bounds: Rect::new(84.0, 100.0, 32.0, 5.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "cloud".to_owned(),
        }];
        if with_spikes {
            entities.push(crate::Entity {
                kind: EntityKind::Spikes,
                bounds: Rect::new(84.0, 153.0, 32.0, 3.0),
                direction: Vec2::new(0.0, -1.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "spikesUp".to_owned(),
            });
        }
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            entities,
            ..Map::default()
        }
    }
    fn booster_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            solids: vec![Rect::new(0.0, 400.0, 320.0, 144.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::Booster,
                bounds: Rect::new(152.0, 384.0, 16.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "booster".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn spring_map(direction: Vec2) -> Map {
        let bounds = if direction.y < 0.0 {
            Rect::new(72.0, 94.0, 16.0, 6.0)
        } else if direction.x > 0.0 {
            Rect::new(100.0, 72.0, 6.0, 16.0)
        } else {
            Rect::new(94.0, 72.0, 6.0, 16.0)
        };
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            solids: vec![Rect::new(0.0, 100.0, 320.0, 80.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::Spring,
                bounds,
                direction,
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "spring".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn refill_map(two_dashes: bool, one_use: bool) -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            solids: vec![Rect::new(0.0, 100.0, 320.0, 80.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::Refill,
                bounds: Rect::new(80.0, 88.0, 16.0, 16.0),
                direction: Vec2::new(if two_dashes { 1.0 } else { 0.0 }, 0.0),
                shielded: false,
                single_use: one_use,
                nodes: vec![],
                name: "refill".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn falling_block_map(climb_fall: bool, top: f32) -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            // The block hangs at (112, top); the floor below starts at 160.
            solids: vec![Rect::new(0.0, 160.0, 320.0, 20.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::FallingBlock,
                bounds: Rect::new(112.0, top, 32.0, 16.0),
                direction: Vec2::new(if climb_fall { 1.0 } else { 0.0 }, 0.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "fallingBlock".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn berry_map(count: usize) -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            solids: vec![Rect::new(0.0, 100.0, 320.0, 80.0)],
            entities: (0..count)
                .map(|_| crate::Entity {
                    kind: EntityKind::Strawberry,
                    bounds: Rect::new(73.0, 86.0, 14.0, 14.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "strawberry".to_owned(),
                })
                .collect(),
            ..Map::default()
        }
    }
    fn dream_exit_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            entities: vec![crate::Entity {
                kind: EntityKind::DreamBlock,
                bounds: Rect::new(40.0, 40.0, 32.0, 40.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "dreamBlock".to_owned(),
            }],
            ..Map::default()
        }
    }
    fn dream_smuggle_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            solids: vec![Rect::new(0.0, 100.0, 320.0, 80.0)],
            entities: vec![
                crate::Entity {
                    kind: EntityKind::DreamBlock,
                    bounds: Rect::new(80.0, 40.0, 96.0, 60.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "dreamBlock".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::TheoCrystal,
                    bounds: Rect::new(68.0, 90.0, 8.0, 10.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "theoCrystal".to_owned(),
                },
            ],
            ..Map::default()
        }
    }
    fn badeline_boost_map(final_boost: bool) -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            entities: vec![crate::Entity {
                kind: EntityKind::BadelineBoost,
                bounds: Rect::new(304.0, 384.0, 32.0, 32.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: if final_boost {
                    vec![]
                } else {
                    vec![Vec2::new(320.0, 288.0)]
                },
                name: "badelineBoost".to_owned(),
            }],
            ..Map::default()
        }
    }

    #[test]
    fn simulation_is_pure_and_deterministic() {
        let p = grounded_player();
        let inputs = vec![
            InputState {
                move_x: 1,
                ..InputState::default()
            };
            30
        ];
        let a = simulate(p.clone(), &inputs, &floor_map(), 30).unwrap();
        let b = simulate(p.clone(), &inputs, &floor_map(), 30).unwrap();
        assert_eq!(a, b);
        assert_eq!(p.pos, Vec2::new(32.0, 100.0));
        assert!(a.speed.x > 80.0);
    }
    #[test]
    fn jump_uses_source_constants() {
        let input = InputState {
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };
        let p = simulate(grounded_player(), &[input], &floor_map(), 1).unwrap();
        assert_eq!(p.state, PlayerState::Normal);
        assert!(p.speed.y <= JUMP_SPEED);
        assert!(p.pos.y < 100.0);
    }
    #[test]
    fn lift_boost_prefers_current_speed_and_uses_player_clamps() {
        let mut p = PlayerSnapshot {
            current_lift_speed: Vec2::new(500.0, 80.0),
            last_lift_speed: Vec2::new(-400.0, -200.0),
            lift_speed_timer: 0.16,
            ..PlayerSnapshot::default()
        };

        assert_eq!(lift_boost(&p), Vec2::new(250.0, 0.0));
        tick_lift_speed(&mut p);
        assert_eq!(p.current_lift_speed, Vec2::default());
        assert_eq!(lift_boost(&p), Vec2::new(-250.0, -130.0));
    }
    #[test]
    fn lift_speed_grace_clears_after_the_source_point_sixteen_seconds() {
        let mut p = PlayerSnapshot {
            last_lift_speed: Vec2::new(90.0, -60.0),
            lift_speed_timer: 0.16,
            ..PlayerSnapshot::default()
        };

        for _ in 0..9 {
            tick_lift_speed(&mut p);
        }
        assert_eq!(p.last_lift_speed, Vec2::new(90.0, -60.0));
        assert!(p.lift_speed_timer > 0.0);

        tick_lift_speed(&mut p);
        assert_eq!(p.last_lift_speed, Vec2::default());
        assert_eq!(p.lift_speed_timer, 0.0);
    }
    #[test]
    fn moving_solid_carries_its_rider_and_records_lift_speed() {
        let mut map = moving_solid_map(Vec2::new(60.0, -120.0));
        let p = simulate(grounded_player(), &[InputState::default()], &map, 1).unwrap();

        assert_eq!(p.pos, Vec2::new(33.0, 98.0));
        assert!(p.on_ground);
        assert_eq!(p.current_lift_speed, Vec2::default());
        assert_eq!(p.last_lift_speed, Vec2::new(60.0, -120.0));
        assert!((p.lift_speed_timer - (0.16 - DT)).abs() < 0.000_001);
    }
    #[test]
    fn moving_solid_jump_combines_carrying_with_same_frame_lift_boost() {
        let mut map = moving_solid_map(Vec2::new(60.0, -120.0));
        let input = InputState {
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };
        let p = simulate(grounded_player(), &[input], &map, 1).unwrap();

        assert_eq!(p.speed, Vec2::new(60.0, -225.0));
        assert_eq!(p.var_jump_speed, -225.0);
        assert_eq!(p.pos, Vec2::new(34.0, 94.0));
    }
    #[test]
    fn moving_solid_clock_keeps_split_simulation_composable() {
        let mut map = moving_solid_map(Vec2::new(60.0, 0.0));
        let inputs = [InputState::default(); 2];
        let direct = simulate(grounded_player(), &inputs, &map, 2).unwrap();
        let first = simulate(grounded_player(), &inputs[..1], &map, 1).unwrap();
        let split = simulate(first, &inputs[1..], &map, 1).unwrap();

        assert_eq!(split, direct);
        assert_eq!(direct.pos.x, 34.0);
    }
    #[test]
    fn moving_solid_moves_attached_spikes_and_restores_them_from_snapshot() {
        let mut map = Map {
            entities: vec![
                crate::Entity {
                    kind: EntityKind::MovingSolid,
                    bounds: Rect::new(16.0, 100.0, 64.0, 8.0),
                    direction: Vec2::new(60.0, 0.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "celesteGymMovingSolid".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::Spikes,
                    bounds: Rect::new(80.0, 100.0, 3.0, 8.0),
                    direction: Vec2::new(1.0, 0.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "spikesRight".to_owned(),
                },
            ],
            ..Map::default()
        };
        let mut simulator = Simulator::new(
            PlayerSnapshot {
                pos: Vec2::new(200.0, 80.0),
                state: PlayerState::Frozen,
                ..PlayerSnapshot::default()
            },
            &map,
        )
        .unwrap();

        simulator.step(InputState::default()).unwrap();
        assert_eq!(simulator.runtime_map.entities[0].bounds.x, 17.0);
        assert_eq!(simulator.runtime_map.entities[1].bounds.x, 81.0);

        let resumed = Simulator::new(simulator.snapshot().clone(), &map).unwrap();
        assert_eq!(resumed.runtime_map.entities[0].bounds.x, 17.0);
        assert_eq!(resumed.runtime_map.entities[1].bounds.x, 81.0);
    }
    #[test]
    fn move_block_moves_its_attached_top_spikes() {
        let mut map = Map {
            entities: vec![
                crate::Entity {
                    kind: EntityKind::MoveBlock,
                    bounds: Rect::new(64.0, 160.0, 32.0, 16.0),
                    direction: Vec2::new(1.0, 0.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "moveBlock".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::Spikes,
                    bounds: Rect::new(64.0, 157.0, 32.0, 3.0),
                    direction: Vec2::new(0.0, -1.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "spikesUp".to_owned(),
                },
            ],
            ..Map::default()
        };
        let mut simulator = Simulator::new(
            PlayerSnapshot {
                pos: Vec2::new(200.0, 100.0),
                state: PlayerState::Frozen,
                move_blocks: vec![crate::MoveBlockSnapshot {
                    phase: 2,
                    speed: 60.0,
                    position: Vec2::new(64.0, 160.0),
                    start: Vec2::new(64.0, 160.0),
                    visible: true,
                    static_movers_enabled: true,
                    ..crate::MoveBlockSnapshot::default()
                }],
                ..PlayerSnapshot::default()
            },
            &map,
        )
        .unwrap();

        simulator.step(InputState::default()).unwrap();
        assert_eq!(simulator.runtime_map.entities[0].bounds.x, 65.0);
        assert_eq!(simulator.runtime_map.entities[1].bounds.x, 65.0);
        assert_eq!(simulator.runtime_map.entities[1].bounds.y, 157.0);
    }
    #[test]
    fn move_block_moves_its_attached_spring() {
        let mut map = Map {
            entities: vec![
                crate::Entity {
                    kind: EntityKind::MoveBlock,
                    bounds: Rect::new(64.0, 160.0, 32.0, 16.0),
                    direction: Vec2::new(1.0, 0.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "moveBlock".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::Spring,
                    bounds: Rect::new(72.0, 154.0, 16.0, 6.0),
                    direction: Vec2::new(0.0, -1.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "spring".to_owned(),
                },
            ],
            ..Map::default()
        };
        let mut simulator = Simulator::new(
            PlayerSnapshot {
                pos: Vec2::new(200.0, 100.0),
                state: PlayerState::Frozen,
                move_blocks: vec![crate::MoveBlockSnapshot {
                    phase: 2,
                    speed: 60.0,
                    position: Vec2::new(64.0, 160.0),
                    start: Vec2::new(64.0, 160.0),
                    visible: true,
                    static_movers_enabled: true,
                    ..crate::MoveBlockSnapshot::default()
                }],
                ..PlayerSnapshot::default()
            },
            &map,
        )
        .unwrap();

        simulator.step(InputState::default()).unwrap();
        assert_eq!(simulator.runtime_map.entities[0].bounds.x, 65.0);
        assert_eq!(simulator.runtime_map.entities[1].bounds.x, 73.0);
        assert_eq!(simulator.runtime_map.entities[1].bounds.y, 154.0);
    }
    #[test]
    fn moving_solid_pushes_an_actor_without_granting_rider_lift_speed() {
        let mut map = Map {
            entities: vec![crate::Entity {
                kind: EntityKind::MovingSolid,
                bounds: Rect::new(20.0, 70.0, 8.0, 40.0),
                direction: Vec2::new(60.0, 0.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "celesteGymMovingSolid".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 90.0),
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &map, 1).unwrap();

        assert_eq!(p.pos.x, 33.0);
        assert_eq!(p.last_lift_speed, Vec2::default());
    }
    #[test]
    fn downward_solid_push_uses_player_squish_target_to_clip_through_jump_thru() {
        let mut map = Map {
            entities: vec![
                crate::Entity {
                    kind: EntityKind::BounceBlock,
                    bounds: Rect::new(40.0, 20.0, 32.0, 16.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "bounceBlock".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::JumpThru,
                    bounds: Rect::new(40.0, 48.0, 32.0, 8.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "jumpThru".to_owned(),
                },
            ],
            ..Map::default()
        };
        let env = solid_collision_env(&map, 0);
        let mut p = PlayerSnapshot {
            pos: Vec2::new(56.0, 48.0),
            ..PlayerSnapshot::default()
        };
        let mut pusher = map.entities[0].bounds;

        move_runtime_solid_exact(&mut p, &mut pusher, &env, false, 8.0, Vec2::new(0.0, 120.0));

        assert_eq!(p.pos, Vec2::new(56.0, 55.0));
        assert!(p.ducking);
        assert!(!p.dead);
        assert_eq!(p.last_lift_speed, Vec2::new(0.0, 120.0));
    }

    #[test]
    fn squish_wiggle_disables_the_pusher_after_target_position_checks() {
        let env = SolidCollisionEnv::default();
        let pusher = Rect::new(52.0, 43.0, 8.0, 11.0);
        let mut p = PlayerSnapshot {
            pos: Vec2::new(56.0, 48.0),
            ..PlayerSnapshot::default()
        };

        let target = p.pos;
        player_on_squish(&mut p, &env, pusher, target);

        // The original position and TargetPosition are both inside the
        // pusher. Player.OnSquish disables it before TrySquishWiggle, which
        // lets the first one-pixel wiggle succeed instead of killing Player.
        assert_eq!(p.pos, Vec2::new(56.0, 49.0));
        assert!(!p.ducking);
        assert!(!p.dead);
    }

    #[test]
    fn zip_mover_runtime_invokes_target_position_jump_thru_clip() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            entities: vec![
                crate::Entity {
                    kind: EntityKind::ZipMover,
                    bounds: Rect::new(40.0, 20.0, 32.0, 16.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![Vec2::new(40.0, 60.0)],
                    name: "zipMover".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::JumpThru,
                    bounds: Rect::new(40.0, 48.0, 32.0, 8.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "jumpThru".to_owned(),
                },
            ],
            ..Map::default()
        };
        let initial = PlayerSnapshot {
            pos: Vec2::new(56.0, 48.0),
            zip_movers: vec![crate::ZipMoverSnapshot {
                phase: 2,
                at: 0.6,
                position: Vec2::new(40.0, 20.0),
                start: Vec2::new(40.0, 20.0),
                ..crate::ZipMoverSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };

        let state = simulate(initial, &[InputState::default()], &map, 1).unwrap();

        assert_eq!(state.pos, Vec2::new(56.0, 65.0));
        assert!(state.pos.y > map.entities[1].bounds.bottom());
        assert!(state.ducking);
        assert!(!state.dead);
        assert!(state.zip_movers[0].position.y > 20.0);
        assert!(state.last_lift_speed.y > 0.0);
    }

    #[test]
    fn zip_mover_departure_and_return_pushes_player_through_jump_thru() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 800.0, 600.0),
            entities: vec![
                crate::Entity {
                    kind: EntityKind::ZipMover,
                    bounds: Rect::new(592.0, 400.0, 64.0, 16.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![Vec2::new(592.0, 300.0)],
                    name: "zipMover".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::JumpThru,
                    bounds: Rect::new(568.0, 416.0, 112.0, 8.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "jumpThru".to_owned(),
                },
            ],
            ..Map::default()
        };
        let inputs: Vec<_> = (0..260)
            .map(|frame| InputState {
                move_x: if frame < 10 {
                    1
                } else if (20..30).contains(&frame) {
                    -1
                } else {
                    0
                },
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(
            PlayerSnapshot {
                pos: Vec2::new(652.0, 400.0),
                on_ground: true,
                ..PlayerSnapshot::default()
            },
            &inputs,
            &map,
            inputs.len() as u32,
        )
        .unwrap();
        let landed = trace
            .states
            .iter()
            .position(|state| state.on_ground && state.pos.y == 416.0)
            .expect("player should land on the JumpThru after leaving the ZipMover");
        let clipped = trace
            .states
            .iter()
            .enumerate()
            .skip(landed + 1)
            .find(|(_, state)| state.pos.y > 416.0 && !state.dead)
            .expect("the returning ZipMover should push the player through the JumpThru");
        assert_eq!(clipped.1.zip_movers[0].phase, 4);
        assert!(trace.states[..=clipped.0].iter().all(|state| !state.dead));
    }
    #[test]
    fn ordinary_downward_solid_push_moves_the_actor_without_squish() {
        let mut map = Map {
            entities: vec![crate::Entity {
                kind: EntityKind::BounceBlock,
                bounds: Rect::new(40.0, 20.0, 32.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "bounceBlock".to_owned(),
            }],
            ..Map::default()
        };
        let env = solid_collision_env(&map, 0);
        let mut p = PlayerSnapshot {
            pos: Vec2::new(56.0, 48.0),
            ..PlayerSnapshot::default()
        };
        let mut pusher = map.entities[0].bounds;

        move_runtime_solid_exact(&mut p, &mut pusher, &env, false, 4.0, Vec2::new(0.0, 60.0));

        assert_eq!(p.pos, Vec2::new(56.0, 51.0));
        assert!(!p.ducking);
        assert!(!p.dead);
    }
    #[test]
    fn zip_mover_uses_source_wait_yield_and_sine_outbound_phases() {
        let p = PlayerSnapshot {
            pos: Vec2::new(64.0, 440.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs = [InputState::default(); 12];
        let trace = simulate_trace(p, &inputs, &zip_mover_map(), 12).unwrap();

        assert_eq!(trace.states[1].zip_movers[0].phase, 1);
        assert_eq!(trace.states[1].zip_movers[0].wait_timer, 0.1);
        let first_outbound = trace
            .states
            .iter()
            .position(|state| state.zip_movers[0].phase == 2)
            .unwrap();
        assert_eq!(trace.states[first_outbound].zip_movers[0].at, 0.0);
        assert_eq!(trace.states[first_outbound].zip_movers[0].position.y, 440.0);
        assert!(trace.states[12].zip_movers[0].at > 0.0);
        assert!(trace.states[12].zip_movers[0].position.y < 440.0);
        assert_eq!(
            trace.states[12].pos.y,
            trace.states[12].zip_movers[0].position.y
        );
        assert!(trace.states[12].last_lift_speed.y < 0.0);
    }

    #[test]
    fn zip_mover_runtime_keeps_split_simulation_composable() {
        let p = PlayerSnapshot {
            pos: Vec2::new(64.0, 440.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs = [InputState::default(); 40];
        let direct = simulate(p.clone(), &inputs, &zip_mover_map(), 40).unwrap();
        let first = simulate(p, &inputs[..17], &zip_mover_map(), 17).unwrap();
        let split = simulate(first, &inputs[17..], &zip_mover_map(), 23).unwrap();

        assert_eq!(split, direct);
        assert_eq!(direct.zip_movers.len(), 1);
        assert_eq!(direct.zip_movers[0].phase, 3);
    }

    #[test]
    fn zip_mover_previous_frame_carry_writes_lift_speed_for_next_player_update() {
        let p = PlayerSnapshot {
            pos: Vec2::new(64.0, 440.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let idle = [InputState::default(); 20];
        let idle_trace = simulate_trace(p.clone(), &idle, &zip_mover_map(), 20).unwrap();
        let lift_state = idle_trace
            .states
            .iter()
            .position(|state| state.last_lift_speed.y < 0.0)
            .unwrap();
        let mut inputs = idle;
        inputs[lift_state] = InputState {
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };
        let trace = simulate_trace(p, &inputs, &zip_mover_map(), 20).unwrap();
        let jumped = &trace.states[lift_state + 1];

        assert!(jumped.speed.y < JUMP_SPEED);
        assert_eq!(jumped.var_jump_speed, jumped.speed.y);
        assert!(jumped.last_lift_speed.y < 0.0);
    }

    #[test]
    fn zip_mover_runs_after_player_and_matches_real_vertical_push_order() {
        let p = PlayerSnapshot {
            pos: Vec2::new(64.0, 440.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..24)
            .map(|frame| InputState {
                jump_pressed: frame == 10,
                jump_held: (10..=15).contains(&frame),
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &zip_mover_map(), 24).unwrap();

        // Real Everest ordering: the first pixel carry is written at the end
        // of frame 10, then Player consumes that retained lift on frame 11.
        assert!((trace.states[10].last_lift_speed.y + 29.575_138).abs() < 0.000_01);
        assert!((trace.states[11].speed.y + 134.575_13).abs() < 0.000_01);

        // At frame 19 Player moves from y=422 to y=420 before ZipMover moves
        // from y=424 to y=421, so there is no early one-pixel platform push.
        assert_eq!(trace.states[18].pos.y, 422.0);
        assert_eq!(trace.states[19].pos.y, 420.0);
        assert_eq!(trace.states[20].pos.y, 417.0);
        assert_eq!(trace.states[18].zip_movers[0].position.y, 424.0);
        assert_eq!(trace.states[19].zip_movers[0].position.y, 421.0);
        assert_eq!(trace.states[20].zip_movers[0].position.y, 417.0);
    }

    #[test]
    fn delayed_blockboost_uses_zip_lift_on_a_later_static_wall_jump() {
        let p = PlayerSnapshot {
            pos: Vec2::new(92.0, 440.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..25)
            .map(|frame| InputState {
                move_x: if frame >= 8 { 1 } else { 0 },
                jump_pressed: frame == 24,
                jump_held: frame == 24,
                ..InputState::default()
            })
            .collect();
        let mut map = zip_mover_map();
        map.solids.push(Rect::new(112.0, 416.0, 8.0, 80.0));
        let trace = simulate_trace(p.clone(), &inputs, &map, 25).unwrap();
        let before = &trace.states[24];
        let jumped = &trace.states[25];

        assert!(trace.states[16].player_on_ground);
        assert!(!trace.states[16].on_ground);
        assert!(!trace.states[17].player_on_ground);
        assert_eq!(trace.states[17].pos.y, 430.0);
        assert!((trace.states[17].speed.y + 110.827_97).abs() < 0.000_01);
        assert_eq!(before.pos.x, 108.0);
        assert!(!before.on_ground);
        assert!(before.last_lift_speed.y < -120.0);
        assert!(before.lift_speed_timer > 0.0);
        assert_eq!(jumped.speed.x, -130.0);
        assert!((jumped.speed.y + 230.828).abs() < 0.000_1);
        assert_eq!(jumped.state, PlayerState::Normal);

        let first = simulate(p, &inputs[..16], &map, 16).unwrap();
        let split = simulate(first, &inputs[16..], &map, 9).unwrap();
        assert_eq!(&split, jumped);
    }

    #[test]
    fn hot_bounce_block_shakes_off_player_with_source_lift_and_jump_grace() {
        let p = PlayerSnapshot {
            pos: Vec2::new(64.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs = vec![InputState::default(); 48];
        let trace = simulate_trace(p, &inputs, &bounce_block_map(), 48).unwrap();
        let launch_index = trace
            .states
            .iter()
            .position(|state| state.bounce_blocks[0].phase == 3)
            .unwrap();
        let launched = &trace.states[launch_index];

        assert_eq!(launched.state, PlayerState::Normal);
        assert_eq!(launched.speed, launched.bounce_blocks[0].bounce_lift);
        assert_eq!(launched.jump_grace_timer, JUMP_GRACE);
        assert!(launched.speed.y < -100.0);
        assert!(launched.on_ground);
        assert!(!trace.states[launch_index + 1].on_ground);
    }

    #[test]
    fn stationary_left_facing_player_can_grab_with_either_signed_zero() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 240.0),
            solids: vec![Rect::new(64.0, 160.0, 32.0, 16.0)],
            ..Map::default()
        };
        for speed_x in [0.0_f32, -0.0_f32] {
            let grabbed = simulate(
                PlayerSnapshot {
                    pos: Vec2::new(100.0, 168.0),
                    speed: Vec2::new(speed_x, 0.0),
                    state: PlayerState::Normal,
                    facing: false,
                    stamina: 110.0,
                    ..PlayerSnapshot::default()
                },
                &[InputState {
                    grab_held: true,
                    ..InputState::default()
                }],
                &map,
                1,
            )
            .unwrap();

            assert_eq!(grabbed.state, PlayerState::Climb, "speed_x={speed_x:?}");
        }
    }

    #[test]
    fn side_grab_activates_and_rides_move_block_away() {
        let mut map = move_block_map();
        map.entities[0].direction = Vec2::new(-1.0, 0.0);
        let inputs = vec![
            InputState {
                grab_held: true,
                ..InputState::default()
            };
            32
        ];
        let trace = simulate_trace(
            PlayerSnapshot {
                pos: Vec2::new(100.0, 168.0),
                state: PlayerState::Normal,
                facing: false,
                stamina: 110.0,
                ..PlayerSnapshot::default()
            },
            &inputs,
            &map,
            inputs.len() as u32,
        )
        .unwrap();

        assert_eq!(trace.states[1].state, PlayerState::Climb);
        assert_eq!(trace.states[1].move_blocks[0].phase, 1);
        let moving = trace
            .states
            .iter()
            .find(|state| state.move_blocks[0].phase == 2 && state.move_blocks[0].position.x < 64.0)
            .expect("side-grabbed MoveBlock should begin moving left");
        assert_eq!(moving.state, PlayerState::Climb);
        assert_eq!(moving.pos.x - moving.move_blocks[0].position.x, 36.0);
        assert_eq!(moving.pos.y - moving.move_blocks[0].position.y, 8.0);
    }

    #[test]
    fn side_grab_rides_bounce_block_until_source_shakeoff() {
        let inputs = vec![
            InputState {
                grab_held: true,
                ..InputState::default()
            };
            48
        ];
        let trace = simulate_trace(
            PlayerSnapshot {
                pos: Vec2::new(100.0, 168.0),
                state: PlayerState::Normal,
                facing: false,
                stamina: 110.0,
                ..PlayerSnapshot::default()
            },
            &inputs,
            &bounce_block_map(),
            inputs.len() as u32,
        )
        .unwrap();

        assert_eq!(trace.states[1].state, PlayerState::Climb);
        assert_eq!(trace.states[1].bounce_blocks[0].phase, 1);
        let carried = trace
            .states
            .iter()
            .find(|state| {
                state.bounce_blocks[0].phase == 1 && state.bounce_blocks[0].position.x < 32.0
            })
            .expect("side-grabbed BounceBlock should wind away from the player");
        assert_eq!(carried.state, PlayerState::Climb);
        assert_eq!(carried.pos.x - carried.bounce_blocks[0].position.x, 68.0);
        assert_eq!(carried.pos.y - carried.bounce_blocks[0].position.y, 8.0);

        let launch_index = trace
            .states
            .iter()
            .position(|state| state.bounce_blocks[0].phase == 3)
            .expect("BounceBlock should reach BounceEnd");
        assert!(
            trace.states[1..launch_index]
                .iter()
                .all(|state| state.state == PlayerState::Climb)
        );
        let launched = &trace.states[launch_index];
        assert_eq!(launched.state, PlayerState::Normal);
        assert_eq!(launched.speed, launched.bounce_blocks[0].bounce_lift);
        assert_eq!(launched.jump_grace_timer, JUMP_GRACE);
    }

    #[test]
    fn bounce_block_runtime_keeps_split_simulation_composable() {
        let p = PlayerSnapshot {
            pos: Vec2::new(64.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs = vec![InputState::default(); 48];
        let direct = simulate(p.clone(), &inputs, &bounce_block_map(), 48).unwrap();
        let first = simulate(p, &inputs[..18], &bounce_block_map(), 18).unwrap();
        let split = simulate(first, &inputs[18..], &bounce_block_map(), 30).unwrap();

        assert_eq!(split, direct);
        assert_eq!(direct.bounce_blocks.len(), 1);
    }

    #[test]
    fn broken_bounce_block_reforms_after_source_respawn_timer() {
        let mut map = bounce_block_map();
        map.solids.push(Rect::new(160.0, 160.0, 160.0, 80.0));
        let initialized = simulate(
            PlayerSnapshot {
                pos: Vec2::new(200.0, 160.0),
                on_ground: true,
                ..PlayerSnapshot::default()
            },
            &[InputState::default()],
            &map,
            1,
        )
        .unwrap();
        let mut broken = initialized;
        broken.bounce_blocks[0].phase = 3;
        broken.bounce_blocks[0].bounce_end_timer = 0.0;
        broken.bounce_blocks[0].position = Vec2::new(32.0, 136.0);
        let inputs = vec![InputState::default(); 110];
        let trace = simulate_trace(broken, &inputs, &map, 110).unwrap();
        let reform_index = trace
            .states
            .iter()
            .position(|state| state.bounce_blocks[0].phase == 0)
            .unwrap();

        assert_eq!(trace.states[1].bounce_blocks[0].phase, 4);
        assert!(trace.states[96].bounce_blocks[0].respawn_timer > 0.0);
        assert!(trace.states[97].bounce_blocks[0].respawn_timer <= 0.0);
        assert_eq!(reform_index, 98);
        assert_eq!(
            trace.states[reform_index].bounce_blocks[0].position,
            Vec2::new(32.0, 160.0)
        );
    }

    #[test]
    fn core_block_moves_disabled_spikes_before_the_reform_alarm_reenables_them() {
        let mut map = bounce_block_spikes_map();
        let mut initial = simulate(
            PlayerSnapshot {
                pos: Vec2::new(240.0, 120.0),
                state: PlayerState::Frozen,
                ..PlayerSnapshot::default()
            },
            &[InputState::default()],
            &map,
            1,
        )
        .unwrap();
        initial.state = PlayerState::Normal;
        initial.pos = Vec2::new(64.0, 160.0);
        initial.on_ground = true;
        initial.player_on_ground = true;
        initial.bounce_blocks[0].phase = 4;
        initial.bounce_blocks[0].respawn_timer = 0.0;
        initial.bounce_blocks[0].position = Vec2::new(32.0, 136.0);
        initial.bounce_blocks[0].attached_spike_position = Vec2::new(32.0, 133.0);
        initial.bounce_blocks[0].static_movers_enabled = false;
        let inputs = vec![InputState::default(); 44];
        let trace = simulate_trace(initial, &inputs, &map, inputs.len() as u32).unwrap();
        let body_reform = trace
            .states
            .iter()
            .position(|state| {
                state.bounce_blocks[0].phase == 0 && state.bounce_blocks[0].reform_timer > 0.0
            })
            .unwrap();
        let spikes_reenabled = trace
            .states
            .iter()
            .position(|state| state.bounce_blocks[0].static_movers_enabled)
            .unwrap();
        assert!(spikes_reenabled > body_reform);
        assert_eq!(spikes_reenabled - body_reform, 21);
        let reenabled = &trace.states[spikes_reenabled].bounce_blocks[0];
        // The player is still standing on the newly collidable body, so the
        // block begins another bounce during Alarm's 0.35-second window.
        // MoveStaticMovers must retain the original top-spike offset while it
        // is disabled; enabling it at the source coordinate would erase CED.
        assert_eq!(
            reenabled.attached_spike_position,
            Vec2::new(reenabled.position.x, reenabled.position.y - 3.0)
        );
        assert_ne!(reenabled.attached_spike_position, Vec2::new(32.0, 157.0));
        assert_ne!(reenabled.position, Vec2::new(32.0, 160.0));
    }

    #[test]
    fn core_block_candidate_clears_source_body_before_reform_blocked_check() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            solids: vec![
                Rect::new(0.0, 496.0, 960.0, 48.0),
                // The physical Playground has this left wall beside the
                // source body. It stops the rider's horizontal carry at
                // x=716 on frame 24 without blocking reset x=712.
                Rect::new(688.0, 448.0, 24.0, 48.0),
            ],
            entities: vec![
                crate::Entity {
                    kind: EntityKind::BounceBlock,
                    bounds: Rect::new(712.0, 480.0, 64.0, 16.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "bounceBlock".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::Spikes,
                    bounds: Rect::new(776.0, 480.0, 3.0, 16.0),
                    direction: Vec2::new(1.0, 0.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "spikesRight".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::JumpThru,
                    bounds: Rect::new(777.0, 480.0, 64.0, 8.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "jumpThru".to_owned(),
                },
            ],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(720.0, 480.0),
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..220)
            .map(|frame| InputState {
                move_x: if (36..108).contains(&frame) {
                    1
                } else if (135..170).contains(&frame) {
                    -1
                } else {
                    0
                },
                jump_pressed: frame == 80,
                jump_held: (80..90).contains(&frame),
                // Everest reports 0.0166667 seconds in this capture. The
                // exact bits put the 24th frame on the same platform carry
                // boundary as the physical CED trace.
                frame_delta_time_bits: Some(0.0166667f32.to_bits()),
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &map, inputs.len() as u32).unwrap();
        assert_eq!(trace.states[24].pos, Vec2::new(716.0, 477.0));
        assert_eq!(trace.states[24].speed, Vec2::default());
        assert_eq!(trace.states[24].state, PlayerState::Normal);
        assert!(trace.states[24].facing);
        assert_eq!(trace.states[24].dashes, 1);
        assert_eq!(trace.states[24].stamina, 110.0);
        assert!(trace.states[24].on_ground);
        assert!(!trace.states[24].ducking);
        assert!(!trace.states[24].dead);
        // Captured Everest frame 83 is inside the CED fixture's JumpThru.
        // Player.Update first applies its -40 px/s JumpThru Assist and then
        // regular -105 px/s jump movement, sharing the same subpixel counter.
        // Lock all nine E2E fields here so a one-pass-only MoveV cannot regress
        // back to the old `(783, 491)` result.
        let jump_thru_assisted = &trace.states[83];
        assert_eq!(jump_thru_assisted.pos, Vec2::new(783.0, 489.0));
        assert!((jump_thru_assisted.speed.x - 121.333_31).abs() <= 0.01);
        assert!((jump_thru_assisted.speed.y + 105.0).abs() <= 0.01);
        assert_eq!(jump_thru_assisted.state, PlayerState::Normal);
        assert!(jump_thru_assisted.facing);
        assert_eq!(jump_thru_assisted.dashes, 1);
        assert!((jump_thru_assisted.stamina - 110.0).abs() <= 0.01);
        assert!(!jump_thru_assisted.on_ground);
        assert!(!jump_thru_assisted.ducking);
        assert!(!jump_thru_assisted.dead);
        let source = Rect::new(712.0, 480.0, 64.0, 16.0);
        assert!(
            !map.static_solid_at(source),
            "the measured +8px reset target must be clear of playground tiles"
        );
        let broken = trace
            .states
            .iter()
            .position(|state| state.bounce_blocks[0].phase == 4)
            .unwrap();
        let body = trace
            .states
            .iter()
            .enumerate()
            .skip(broken + 1)
            .find(|(_, state)| {
                state.bounce_blocks[0].phase == 0 && !state.bounce_blocks[0].static_movers_enabled
            })
            .map(|(frame, _)| frame)
            .unwrap();
        let spike = trace
            .states
            .iter()
            .enumerate()
            .skip(body + 1)
            .find(|(_, state)| state.bounce_blocks[0].static_movers_enabled)
            .map(|(frame, _)| frame)
            .unwrap();
        let broken_block = &trace.states[broken].bounce_blocks[0];
        let body_block = &trace.states[body].bounce_blocks[0];
        assert_eq!(body_block.position, Vec2::new(712.0, 480.0));
        assert_eq!(body_block.phase, 0);
        assert!(!body_block.static_movers_enabled);
        assert!(
            !source.intersects(current_player_rect(
                &trace.states[body],
                trace.states[body].pos.x,
                trace.states[body].pos.y,
            )),
            "player must clear the source volume before reform: frame={body}, state={:?}",
            trace.states[body]
        );
        assert!(
            spike > body,
            "the 0.35-second StaticMover alarm must follow body reform"
        );
        assert!(
            (21..=24).contains(&(spike - body)),
            "the 0.35-second alarm may include Celeste's transition freeze: body={body}, spike={spike}"
        );
        let reenabled = &trace.states[spike].bounce_blocks[0];
        let expected_spike = Vec2::new(
            broken_block.attached_spike_position.x + body_block.position.x
                - broken_block.position.x,
            broken_block.attached_spike_position.y + body_block.position.y
                - broken_block.position.y,
        );
        assert!(
            reenabled.static_movers_enabled && reenabled.attached_spike_position == expected_spike,
            "the spike must re-enable at the broken-body -> restored-body displacement: broken={broken_block:?}, body={body_block:?}, reenabled={reenabled:?}"
        );
        assert_ne!(expected_spike, broken_block.attached_spike_position);
        assert_eq!(reenabled.position, body_block.position);
    }

    #[test]
    fn moon_block_steering_writes_diagonal_lift_for_a_jump() {
        let p = PlayerSnapshot {
            pos: Vec2::new(80.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<InputState> = (0..52)
            .map(|frame| InputState {
                move_x: 0,
                move_y: -1,
                jump_pressed: frame == 40,
                jump_held: frame == 40,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &move_block_map(), inputs.len() as u32).unwrap();
        assert_eq!(trace.states[16].pos.x, 81.0);
        assert!(
            trace.states.iter().any(|state| {
                state.move_blocks[0].lift_speed.y < -20.0 && state.last_lift_speed.y < -20.0
            }),
            "trace={:?}",
            trace
                .states
                .iter()
                .enumerate()
                .map(|(frame, state)| (frame, state.pos, state.speed, state.move_blocks[0].clone()))
                .collect::<Vec<_>>()
        );
        assert!(trace.states.iter().any(|state| state.speed.y < JUMP_SPEED));
    }

    #[test]
    fn move_block_runtime_keeps_split_simulation_composable() {
        let p = PlayerSnapshot {
            pos: Vec2::new(80.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs = vec![
            InputState {
                move_y: -1,
                ..InputState::default()
            };
            52
        ];
        let direct = simulate(p.clone(), &inputs, &move_block_map(), 52).unwrap();
        let first = simulate(p, &inputs[..27], &move_block_map(), 27).unwrap();
        let split = simulate(first, &inputs[27..], &move_block_map(), 25).unwrap();
        assert_eq!(split, direct);
    }

    #[test]
    fn move_block_reform_body_precedes_visibility_and_static_movers_by_point_eight() {
        let mut initial = PlayerSnapshot {
            pos: Vec2::new(240.0, 120.0),
            state: PlayerState::Frozen,
            ..PlayerSnapshot::default()
        };
        initial.move_blocks = vec![crate::MoveBlockSnapshot {
            phase: 3,
            position: Vec2::new(120.0, 160.0),
            start: Vec2::new(64.0, 160.0),
            visible: true,
            static_movers_enabled: true,
            ..crate::MoveBlockSnapshot::default()
        }];
        let inputs = vec![InputState::default(); 220];
        let trace =
            simulate_trace(initial, &inputs, &move_block_map(), inputs.len() as u32).unwrap();
        let collidable = trace
            .states
            .iter()
            .position(|state| state.move_blocks[0].phase == 5)
            .unwrap();
        let visible = trace
            .states
            .iter()
            .position(|state| {
                state.move_blocks[0].phase == 0
                    && state.move_blocks[0].visible
                    && state.move_blocks[0].static_movers_enabled
            })
            .unwrap();
        assert!(!trace.states[collidable].move_blocks[0].visible);
        assert!(!trace.states[collidable].move_blocks[0].static_movers_enabled);
        assert_eq!(visible - collidable, 49);
    }

    #[test]
    fn reform_kick_wall_jumps_from_the_newly_collidable_invisible_body() {
        let mut initial = PlayerSnapshot {
            pos: Vec2::new(60.0, 174.0),
            ..PlayerSnapshot::default()
        };
        initial.move_blocks = vec![crate::MoveBlockSnapshot {
            phase: 4,
            position: Vec2::new(64.0, 160.0),
            start: Vec2::new(64.0, 160.0),
            visible: false,
            static_movers_enabled: false,
            ..crate::MoveBlockSnapshot::default()
        }];
        let reformed = simulate(initial, &[InputState::default()], &move_block_map(), 1).unwrap();
        assert_eq!(reformed.move_blocks[0].phase, 5);
        assert!(!reformed.move_blocks[0].visible);
        let kicked = simulate(
            reformed,
            &[InputState {
                move_x: -1,
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            }],
            &move_block_map(),
            1,
        )
        .unwrap();
        assert_eq!(kicked.speed.x, -WALL_JUMP_H);
        assert_eq!(kicked.speed.y, JUMP_SPEED);
        assert_eq!(kicked.move_blocks[0].phase, 5);
    }

    #[test]
    fn dash_pickup_runs_before_the_dash_coroutine_samples_direction() {
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 160.0),
            speed: Vec2::new(360.0, 0.0),
            state: PlayerState::Dash,
            // DashCoroutine sets DashDir only after its initial yield, but
            // DashUpdate's Holdable loop executes before that coroutine.
            dash_dir: Vec2::default(),
            state_timer: 0.1,
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let held = InputState {
            move_x: 1,
            grab_held: true,
            ..InputState::default()
        };
        let trace = simulate_trace(p, &[held; 13], &theo_crystal_map(), 13).unwrap();

        assert_eq!(trace.states[1].state, PlayerState::Pickup);
        assert_eq!(trace.states[1].speed, Vec2::default());
        assert_eq!(trace.states[1].pickup_old_speed, Vec2::new(360.0, 0.0));
        assert_eq!(trace.states[1].holding_theo, Some(0));
        assert!(trace.states[1].theo_crystals[0].held);
        assert!(!trace.states[1].dash_end_pending);
        assert_eq!(trace.states[11].state, PlayerState::Pickup);
        assert_eq!(trace.states[12].state, PlayerState::Pickup);
        assert_eq!(trace.states[13].state, PlayerState::Normal);
        assert_eq!(trace.states[13].speed, Vec2::new(360.0, 0.0));
    }

    #[test]
    fn dash_pickup_after_the_initial_yield_caches_live_updash_speed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 160.0),
            state: PlayerState::Dash,
            // This is the state at the first DashUpdate following the
            // coroutine's initial `yield return null`: it must publish the
            // up-dash speed before a later holdable check can cache it.
            state_timer: DASH_TIME + DT,
            last_aim: Vec2::new(0.0, -1.0),
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(
            p,
            &[
                InputState {
                    move_y: -1,
                    ..InputState::default()
                },
                InputState {
                    move_y: -1,
                    grab_held: true,
                    ..InputState::default()
                },
            ],
            &theo_crystal_map(),
            2,
        )
        .unwrap();

        assert_eq!(trace.states[1].state, PlayerState::Dash);
        assert_eq!(trace.states[1].dash_dir, Vec2::new(0.0, -1.0));
        assert_eq!(trace.states[1].speed, Vec2::new(0.0, -DASH_SPEED));
        assert_eq!(trace.states[2].state, PlayerState::Pickup);
        assert_eq!(
            trace.states[2].pickup_old_speed,
            Vec2::new(0.0, -DASH_SPEED)
        );
    }

    #[test]
    fn theo_pickup_release_and_runtime_are_split_composable() {
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![
            InputState {
                grab_held: true,
                ..InputState::default()
            };
            25
        ];
        inputs.push(InputState {
            move_x: 1,
            ..InputState::default()
        });
        let direct = simulate(p.clone(), &inputs, &theo_crystal_map(), 26).unwrap();
        let first = simulate(p, &inputs[..8], &theo_crystal_map(), 8).unwrap();
        let split = simulate(first, &inputs[8..], &theo_crystal_map(), 18).unwrap();

        assert_eq!(split, direct);
        assert_eq!(direct.state, PlayerState::Normal);
        assert_eq!(direct.holding_theo, None);
        assert!(!direct.theo_crystals[0].held);
        assert!(direct.theo_crystals[0].cannot_hold_timer > 0.0);
        assert_eq!(direct.theo_crystals[0].speed, Vec2::new(200.0, -80.0));
        assert!((direct.speed.x + 63.333_3).abs() < 0.000_1);
    }

    #[test]
    fn glider_pickup_release_and_runtime_are_split_composable() {
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![
            InputState {
                grab_held: true,
                ..InputState::default()
            };
            25
        ];
        inputs.push(InputState {
            move_x: 1,
            ..InputState::default()
        });
        let direct = simulate(p.clone(), &inputs, &glider_map(), 26).unwrap();
        let first = simulate(p, &inputs[..8], &glider_map(), 8).unwrap();
        let split = simulate(first, &inputs[8..], &glider_map(), 18).unwrap();

        assert_eq!(split, direct);
        assert_eq!(direct.state, PlayerState::Normal);
        assert_eq!(direct.holding_glider, None);
        assert!(!direct.gliders[0].held);
        assert!(direct.gliders[0].cannot_hold_timer > 0.28);
        assert_eq!(direct.gliders[0].speed, Vec2::new(100.0, -40.0));
        assert!((direct.speed.x + 63.333_3).abs() < 0.000_1);
    }

    #[test]
    fn held_glider_uses_slow_fall_air_control_without_slow_run() {
        let p = PlayerSnapshot {
            pos: Vec2::new(80.0, 100.0),
            speed: Vec2::new(200.0, 0.0),
            holding_glider: Some(0),
            gliders: vec![crate::GliderSnapshot {
                position: Vec2::new(80.0, 88.0),
                held: true,
                ..crate::GliderSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let next = simulate(
            p,
            &[InputState {
                move_x: 1,
                grab_held: true,
                ..InputState::default()
            }],
            &glider_map(),
            1,
        )
        .unwrap();

        assert!((next.speed.x - 197.833_33).abs() < 0.000_1);
        assert!((next.speed.y - 7.5).abs() < 0.000_1);
        assert!((next.max_fall - 155.0).abs() < 0.000_1);
        assert_eq!(next.holding_glider, Some(0));
    }

    #[test]
    fn held_glider_turns_grabbed_wall_jump_into_a_normal_neutral() {
        let mut map = glider_map();
        map.bounds = Rect::new(0.0, 0.0, 160.0, 180.0);
        map.solids = vec![Rect::new(64.0, 0.0, 16.0, 180.0)];
        map.entities[0].bounds = Rect::new(56.0, 90.0, 8.0, 10.0);
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 100.0),
            facing: true,
            holding_glider: Some(0),
            gliders: vec![crate::GliderSnapshot {
                position: Vec2::new(60.0, 88.0),
                held: true,
                ..crate::GliderSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let jumped = simulate(
            p,
            &[InputState {
                jump_pressed: true,
                jump_held: true,
                grab_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();

        assert_eq!(jumped.state, PlayerState::Normal);
        assert_eq!(jumped.speed, Vec2::new(-WALL_JUMP_H, JUMP_SPEED));
        assert_eq!(jumped.holding_glider, Some(0));
        assert_eq!(jumped.wall_boost_timer, 0.0);
    }

    #[test]
    fn released_glider_obeys_long_lockout_then_can_be_regrabbed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 160.0),
            on_ground: true,
            holding_glider: Some(0),
            gliders: vec![crate::GliderSnapshot {
                position: Vec2::new(60.0, 148.0),
                held: true,
                ..crate::GliderSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..30)
            .map(|frame| InputState {
                move_y: if frame == 0 { 1 } else { 0 },
                grab_held: frame > 0,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &glider_map(), inputs.len() as u32).unwrap();
        let regrab = trace
            .states
            .iter()
            .enumerate()
            .skip(2)
            .find(|(_, state)| state.holding_glider == Some(0))
            .map(|(frame, _)| frame)
            .unwrap();

        assert!(trace.states[1].gliders[0].cannot_hold_timer > 0.28);
        assert!(regrab >= 19, "regrabbed too early at frame {regrab}");
        assert_eq!(trace.states[regrab].state, PlayerState::Pickup);
    }

    #[test]
    fn glider_pickup_tween_stalls_then_clamps_upward_speed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 160.0),
            speed: Vec2::new(30.0, -20.0),
            ..PlayerSnapshot::default()
        };
        let held = InputState {
            grab_held: true,
            ..InputState::default()
        };
        let trace = simulate_trace(p, &[held; 13], &glider_map(), 13).unwrap();

        assert_eq!(trace.states[1].state, PlayerState::Pickup);
        assert_eq!(trace.states[1].speed, Vec2::default());
        assert_eq!(trace.states[1].pickup_old_speed, Vec2::new(30.0, -20.0));
        assert_eq!(trace.states[13].state, PlayerState::Normal);
        assert_eq!(trace.states[13].speed, Vec2::new(30.0, JUMP_SPEED));
    }

    #[test]
    fn jelly_neutral_drop_wall_jump_regrabs_after_long_lockout() {
        let mut map = glider_map();
        map.bounds = Rect::new(0.0, 0.0, 320.0, 544.0);
        map.solids = vec![Rect::new(144.0, 240.0, 16.0, 256.0)];
        map.entities[0].bounds = Rect::new(136.0, 410.0, 8.0, 10.0);
        let p = PlayerSnapshot {
            pos: Vec2::new(140.0, 420.0),
            facing: true,
            holding_glider: Some(0),
            gliders: vec![crate::GliderSnapshot {
                position: Vec2::new(140.0, 408.0),
                held: true,
                ..crate::GliderSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..24)
            .map(|frame| InputState {
                move_y: if frame == 0 { 1 } else { 0 },
                jump_pressed: frame == 0,
                jump_held: frame == 0,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &map, inputs.len() as u32).unwrap();
        let neutral = trace
            .states
            .iter()
            .position(|state| state.speed == Vec2::new(-WALL_JUMP_H, JUMP_SPEED))
            .unwrap_or_else(|| {
                panic!(
                    "missing neutral: {:?}",
                    trace
                        .states
                        .iter()
                        .take(12)
                        .map(|state| (state.state, state.speed, state.holding_glider))
                        .collect::<Vec<_>>()
                )
            });
        assert_eq!(neutral, 1);
        assert!(trace.states[10].gliders[0].cannot_hold_timer > 0.0);
        let mut after_lockout = trace.states[24].clone();
        assert_eq!(after_lockout.gliders[0].cannot_hold_timer, 0.0);
        after_lockout.gliders[0].position = after_lockout.pos;
        let regrabbed = simulate(
            after_lockout,
            &[InputState {
                grab_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert_eq!(regrabbed.holding_glider, Some(0));
        assert_eq!(regrabbed.state, PlayerState::Pickup);
    }

    #[test]
    fn two_gliders_keep_independent_laddering_lockouts() {
        let mut p = PlayerSnapshot {
            pos: Vec2::new(80.0, 100.0),
            gliders: vec![
                crate::GliderSnapshot {
                    position: Vec2::new(80.0, 98.0),
                    ..crate::GliderSnapshot::default()
                },
                crate::GliderSnapshot {
                    position: Vec2::new(80.0, 98.0),
                    ..crate::GliderSnapshot::default()
                },
            ],
            ..PlayerSnapshot::default()
        };
        assert!(try_pickup_glider(&mut p));
        assert_eq!(p.holding_glider, Some(0));
        release_glider(
            &mut p,
            InputState {
                move_y: 1,
                ..InputState::default()
            },
        );
        assert!(try_pickup_glider(&mut p));

        assert_eq!(p.holding_glider, Some(1));
        assert_eq!(p.gliders[0].cannot_hold_timer, 0.3);
        assert_eq!(p.gliders[1].cannot_hold_timer, 0.0);
    }

    #[test]
    fn holdable_laddering_regrabs_with_glider_source_pickup_collider() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 544.0),
            solids: vec![Rect::new(0.0, 496.0, 320.0, 48.0)],
            entities: vec![
                crate::Entity {
                    kind: EntityKind::Glider,
                    bounds: Rect::new(92.0, 390.0, 8.0, 10.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "glider".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::Glider,
                    bounds: Rect::new(92.0, 380.0, 8.0, 10.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "glider".to_owned(),
                },
            ],
            ..Map::default()
        };
        let inputs: Vec<_> = (0..150)
            .map(|frame| InputState {
                move_y: if matches!(frame, 23 | 65 | 101) {
                    1
                } else {
                    -1
                },
                grab_held: !matches!(frame, 23 | 65 | 101),
                ..InputState::default()
            })
            .collect();
        let initial = PlayerSnapshot {
            pos: Vec2::new(96.0, 400.0),
            speed: Vec2::new(0.0, -30.0),
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(initial, &inputs, &map, inputs.len() as u32).unwrap();
        let pickup_starts: Vec<_> = trace
            .states
            .iter()
            .enumerate()
            .filter_map(|(frame, state)| {
                (state.state == PlayerState::Pickup
                    && trace
                        .states
                        .get(frame.wrapping_sub(1))
                        .is_none_or(|previous| previous.state != PlayerState::Pickup))
                .then_some((frame, state.holding_glider))
            })
            .collect();

        // The second jelly's 20x22 source PickupCollider starts the f25
        // tween while the first jelly remains under its own 0.3 s lockout.
        assert_eq!(
            pickup_starts,
            vec![(1, Some(0)), (25, Some(1)), (67, Some(0))]
        );
        // PickupCoroutine's StateMachine transition invokes NormalBegin.
        // Resetting maxFall there leaves the first post-tween slow-fall
        // sequence at the source f55 cap and speed, rather than retaining
        // the previous Glider cap (375 / 25).
        assert_eq!(trace.states[55].pos, Vec2::new(96.0, 376.0));
        assert!((trace.states[55].speed.y - 30.0).abs() <= 0.01);
        // After maxFall reaches the held-Glider neutral cap, the next source
        // frame preserves that cap (rather than applying another 5 px/s
        // approach) before the second pickup starts.
        assert_eq!(trace.states[62].pos, Vec2::new(96.0, 381.0));
        assert!((trace.states[62].speed.y - 40.0).abs() <= 0.01);
    }

    #[test]
    fn grounded_ultra_glider_pickup_cancel_preserves_multiplied_speed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 160.0),
            speed: Vec2::new(300.0, 0.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..24)
            .map(|frame| InputState {
                move_x: 1,
                move_y: if frame < 10 { 1 } else { 0 },
                dash_pressed: frame == 0,
                grab_held: frame >= 5,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &glider_map(), 24).unwrap();
        let pickup = trace
            .states
            .iter()
            .position(|state| state.state == PlayerState::Pickup)
            .unwrap();
        let restored = trace
            .states
            .iter()
            .enumerate()
            .skip(pickup + 1)
            .find(|(_, state)| state.state == PlayerState::Normal)
            .map(|(frame, _)| frame)
            .unwrap();

        assert_eq!(trace.states[pickup - 1].speed.x, 360.0);
        assert_eq!(trace.states[pickup].holding_glider, Some(0));
        assert_eq!(trace.states[pickup].pickup_old_speed.x, 360.0);
        assert!(!trace.states[pickup].ducking);
        assert_eq!(trace.states[restored].speed.x, 360.0);
        assert!(!trace.states[restored].ducking);
        assert!((trace.states[restored + 1].speed.x - (360.0 - RUN_REDUCE * DT)).abs() < 0.0001);
        assert!(!trace.states[restored + 1].ducking);
    }

    #[test]
    fn jellyvator_regrabs_updash_and_restores_vertical_speed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..80)
            .map(|frame| InputState {
                move_y: if frame == 23 {
                    1
                } else if frame >= 42 {
                    -1
                } else {
                    0
                },
                dash_pressed: frame == 42,
                // Keep Grab held through the DashCoroutine launch frame. The
                // coroutine owns that frame, so the regrab must wait until
                // the next DashUpdate and cache the live -240 speed.
                grab_held: frame <= 22 || frame >= 45,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &glider_map(), inputs.len() as u32).unwrap();
        let pickup = trace
            .states
            .iter()
            .enumerate()
            .skip(42)
            .find(|(_, state)| {
                state.state == PlayerState::Pickup && state.holding_glider == Some(0)
            })
            .map(|(frame, _)| frame)
            .unwrap();
        let restored = trace
            .states
            .iter()
            .enumerate()
            .skip(pickup + 1)
            .find(|(_, state)| state.state == PlayerState::Normal)
            .map(|(frame, _)| frame)
            .unwrap();

        assert_eq!(trace.states[47].state, PlayerState::Dash);
        assert_eq!(trace.states[47].speed, Vec2::new(0.0, -DASH_SPEED));
        assert_eq!(pickup, 48);
        assert_eq!(trace.states[pickup].pickup_old_speed.y, -DASH_SPEED);
        assert_eq!(trace.states[restored].speed.y, -DASH_SPEED);
    }

    #[test]
    fn floor_spring_launches_unheld_glider_after_actor_movement() {
        let mut map = glider_map();
        map.solids.clear();
        map.entities[0].bounds = Rect::new(76.0, 88.0, 8.0, 10.0);
        map.entities.push(crate::Entity {
            kind: EntityKind::Spring,
            bounds: Rect::new(72.0, 94.0, 16.0, 6.0),
            direction: Vec2::new(0.0, -1.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "spring".to_owned(),
        });
        let p = PlayerSnapshot {
            pos: Vec2::new(200.0, 100.0),
            gliders: vec![crate::GliderSnapshot {
                position: Vec2::new(80.0, 98.0),
                speed: Vec2::new(80.0, 20.0),
                ..crate::GliderSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let next = simulate(p, &[InputState::default()], &map, 1).unwrap();

        assert_eq!(next.gliders[0].speed.y, -160.0);
        assert!((next.gliders[0].speed.x - 39.666_668).abs() < 0.000_1);
        assert_eq!(next.gliders[0].no_gravity_timer, 0.15);
    }

    #[test]
    fn glider_spring_no_gravity_keeps_its_final_source_frame() {
        let mut map = glider_map();
        let p = PlayerSnapshot {
            pos: Vec2::new(200.0, 100.0),
            gliders: vec![crate::GliderSnapshot {
                position: Vec2::new(80.0, 100.0),
                speed: Vec2::new(0.0, -160.0),
                no_gravity_timer: DT,
                ..crate::GliderSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };

        let next = simulate(p, &[InputState::default()], &map, 1).unwrap();

        assert_eq!(next.gliders[0].speed.y, -160.0);
        assert_eq!(next.gliders[0].no_gravity_timer, 0.0);
    }

    #[test]
    fn springboost_cancel_reverses_into_the_rising_glider_for_regrab() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 544.0),
            solids: vec![Rect::new(0.0, 496.0, 320.0, 48.0)],
            entities: vec![
                crate::Entity {
                    kind: EntityKind::Glider,
                    bounds: Rect::new(96.0, 486.0, 8.0, 10.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "glider".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::Spring,
                    bounds: Rect::new(128.0, 490.0, 16.0, 6.0),
                    direction: Vec2::new(0.0, -1.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "spring".to_owned(),
                },
            ],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(100.0, 496.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..130)
            .map(|frame| InputState {
                move_x: if (14..45).contains(&frame) {
                    1
                } else if (45..75).contains(&frame) {
                    -1
                } else {
                    0
                },
                move_y: if frame == 35 { 1 } else { 0 },
                jump_pressed: frame == 25,
                jump_held: (25..34).contains(&frame),
                grab_held: frame <= 34 || frame >= 100,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &map, inputs.len() as u32).unwrap();
        let released = trace
            .states
            .windows(2)
            .position(|pair| pair[0].holding_glider == Some(0) && pair[1].holding_glider.is_none())
            .map(|frame| frame + 1)
            .unwrap();
        let spring = trace
            .states
            .iter()
            .enumerate()
            .skip(released + 1)
            .find(|(_, state)| state.gliders[0].speed.y == -160.0)
            .map(|(frame, _)| frame)
            .unwrap_or_else(|| {
                panic!(
                    "missing spring release={released}: {:?}",
                    trace
                        .states
                        .iter()
                        .enumerate()
                        .skip(released)
                        .step_by(5)
                        .map(|(frame, state)| (
                            frame,
                            state.pos,
                            state.gliders[0].position,
                            state.gliders[0].speed,
                            state.holding_glider,
                        ))
                        .collect::<Vec<_>>()
                )
            });
        let regrab = trace
            .states
            .iter()
            .enumerate()
            .skip(spring + 1)
            .find(|(_, state)| {
                state.state == PlayerState::Pickup && state.holding_glider == Some(0)
            })
            .map(|(frame, _)| frame)
            .unwrap_or_else(|| {
                panic!(
                    "missing regrab release={released} spring={spring}: {:?}",
                    trace
                        .states
                        .iter()
                        .enumerate()
                        .skip(spring)
                        .step_by(5)
                        .map(|(frame, state)| (
                            frame,
                            state.pos,
                            state.speed,
                            state.gliders[0].position,
                            state.gliders[0].speed,
                            state.holding_glider,
                        ))
                        .collect::<Vec<_>>()
                )
            });
        // Glider.cs gives Holdable a 20x22 PickupCollider offset -10,-16.
        // Its high upper edge catches the falling player while the jelly is
        // still rising, so the Pickup tween begins before body overlap.
        assert_eq!(trace.states[102].state, PlayerState::Normal);
        assert_eq!(trace.states[102].pos, Vec2::new(126.0, 460.0));
        assert_eq!(trace.states[102].speed, Vec2::new(0.0, MAX_FALL));
        assert_eq!(regrab, 103);
        assert!(regrab > spring);
    }

    #[test]
    fn holdable_springs_apply_theo_floor_and_wall_source_speeds() {
        let mut floor_map = theo_crystal_map();
        floor_map.solids.clear();
        floor_map.entities[0].bounds = Rect::new(76.0, 88.0, 8.0, 10.0);
        floor_map.entities.push(crate::Entity {
            kind: EntityKind::Spring,
            bounds: Rect::new(72.0, 94.0, 16.0, 6.0),
            direction: Vec2::new(0.0, -1.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "spring".to_owned(),
        });
        let floor = simulate(
            PlayerSnapshot {
                pos: Vec2::new(200.0, 100.0),
                theo_crystals: vec![crate::TheoCrystalSnapshot {
                    position: Vec2::new(80.0, 98.0),
                    speed: Vec2::new(80.0, 20.0),
                    ..crate::TheoCrystalSnapshot::default()
                }],
                ..PlayerSnapshot::default()
            },
            &[InputState::default()],
            &floor_map,
            1,
        )
        .unwrap();
        assert_eq!(floor.theo_crystals[0].speed.y, -160.0);
        // Airborne Theo keeps the 200 px/s² release curve before the spring
        // halves its horizontal speed on contact.
        assert!((floor.theo_crystals[0].speed.x - 38.333_332).abs() < 0.000_1);
        assert_eq!(floor.theo_crystals[0].gravity_timer, 0.15);

        let mut wall_map = theo_crystal_map();
        wall_map.solids.clear();
        wall_map.entities[0].bounds = Rect::new(76.0, 88.0, 8.0, 10.0);
        wall_map.entities.push(crate::Entity {
            kind: EntityKind::Spring,
            bounds: Rect::new(72.0, 84.0, 6.0, 16.0),
            direction: Vec2::new(1.0, 0.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "wallSpringLeft".to_owned(),
        });
        let wall = simulate(
            PlayerSnapshot {
                pos: Vec2::new(200.0, 100.0),
                theo_crystals: vec![crate::TheoCrystalSnapshot {
                    position: Vec2::new(80.0, 98.0),
                    speed: Vec2::new(-20.0, 0.0),
                    ..crate::TheoCrystalSnapshot::default()
                }],
                ..PlayerSnapshot::default()
            },
            &[InputState::default()],
            &wall_map,
            1,
        )
        .unwrap();
        assert_eq!(wall.theo_crystals[0].speed, Vec2::new(220.0, -80.0));
        assert_eq!(wall.theo_crystals[0].gravity_timer, 0.1);
    }

    #[test]
    fn neutral_drop_releases_theo_without_throw_speed_or_player_recoil() {
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 160.0),
            on_ground: true,
            holding_theo: Some(0),
            theo_crystals: vec![crate::TheoCrystalSnapshot {
                position: Vec2::new(60.0, 148.0),
                held: true,
                ..crate::TheoCrystalSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let dropped = simulate(
            p,
            &[InputState {
                move_y: 1,
                ..InputState::default()
            }],
            &theo_crystal_map(),
            1,
        )
        .unwrap();

        assert_eq!(dropped.holding_theo, None);
        assert_eq!(dropped.speed.x, 0.0);
        assert_eq!(dropped.theo_crystals[0].speed, Vec2::default());
        assert!(dropped.theo_crystals[0].cannot_hold_timer > 0.0);
        assert!(dropped.theo_crystals[0].gravity_timer > 0.0);
    }

    #[test]
    fn neutral_drop_can_start_a_dash_on_the_same_normal_update() {
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 120.0),
            holding_theo: Some(0),
            theo_crystals: vec![crate::TheoCrystalSnapshot {
                position: Vec2::new(60.0, 108.0),
                held: true,
                ..crate::TheoCrystalSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let dropped = simulate(
            p,
            &[InputState {
                move_x: 1,
                move_y: 1,
                dash_pressed: true,
                ..InputState::default()
            }],
            &theo_crystal_map(),
            1,
        )
        .unwrap();

        assert_eq!(dropped.state, PlayerState::Dash);
        assert_eq!(dropped.holding_theo, None);
        assert_eq!(dropped.theo_crystals[0].speed, Vec2::default());
        assert!(dropped.theo_crystals[0].cannot_hold_timer > 0.0);
    }

    #[test]
    fn held_theo_turns_grabbed_wall_jump_into_a_normal_neutral() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 160.0, 180.0),
            solids: vec![Rect::new(64.0, 0.0, 16.0, 180.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::TheoCrystal,
                bounds: Rect::new(56.0, 90.0, 8.0, 10.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "theoCrystal".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 100.0),
            facing: true,
            holding_theo: Some(0),
            theo_crystals: vec![crate::TheoCrystalSnapshot {
                position: Vec2::new(60.0, 88.0),
                held: true,
                ..crate::TheoCrystalSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let jumped = simulate(
            p,
            &[InputState {
                jump_pressed: true,
                jump_held: true,
                grab_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();

        assert_eq!(jumped.state, PlayerState::Normal);
        assert_eq!(jumped.speed, Vec2::new(-WALL_JUMP_H, JUMP_SPEED));
        assert_eq!(jumped.holding_theo, Some(0));
        assert_eq!(jumped.wall_boost_timer, 0.0);
    }

    #[test]
    fn theo_neutral_drop_dash_regrab_waits_out_cannot_hold() {
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..60)
            .map(|frame| InputState {
                move_x: if (14..28).contains(&frame) {
                    -1
                } else if frame >= 28 {
                    1
                } else {
                    0
                },
                move_y: if frame == 23 { 1 } else { 0 },
                dash_pressed: frame == 28,
                grab_held: frame <= 22 || frame >= 35,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &theo_crystal_map(), inputs.len() as u32).unwrap();
        let released = trace
            .states
            .iter()
            .position(|state| {
                state.holding_theo.is_none() && state.theo_crystals[0].cannot_hold_timer > 0.0
            })
            .unwrap();
        let regrabbed = trace
            .states
            .iter()
            .enumerate()
            .skip(released + 1)
            .find(|(_, state)| state.state == PlayerState::Pickup && state.holding_theo == Some(0))
            .map(|(frame, _)| frame)
            .unwrap();

        assert_eq!(released, 24);
        assert!(
            trace.states[released + 1..regrabbed]
                .iter()
                .all(|state| state.holding_theo.is_none())
        );
        assert_eq!(trace.states[regrabbed].pickup_old_speed.x, DASH_SPEED);
        assert_eq!(trace.states[regrabbed].speed, Vec2::default());
    }

    #[test]
    fn holdable_slash_regrabs_theo_in_horizontal_dash_with_airborne_vertical_speed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..70)
            .map(|frame| InputState {
                move_x: if (14..28).contains(&frame) {
                    -1
                } else if frame >= 28 {
                    1
                } else {
                    0
                },
                move_y: if frame == 23 { 1 } else { 0 },
                jump_pressed: frame == 14,
                jump_held: (14..23).contains(&frame),
                dash_pressed: frame == 28,
                grab_held: frame <= 22 || frame >= 35,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &theo_crystal_map(), inputs.len() as u32).unwrap();
        let released = trace
            .states
            .iter()
            .position(|state| {
                state.holding_theo.is_none() && state.theo_crystals[0].cannot_hold_timer > 0.0
            })
            .unwrap();
        let dash = trace
            .states
            .iter()
            .enumerate()
            .skip(released + 1)
            .find(|(_, state)| state.state == PlayerState::Dash)
            .map(|(frame, _)| frame)
            .unwrap();
        let regrabbed = trace
            .states
            .iter()
            .enumerate()
            .skip(dash + 1)
            .find(|(_, state)| state.state == PlayerState::Pickup && state.holding_theo == Some(0))
            .map(|(frame, _)| frame)
            .unwrap();
        let restored = trace
            .states
            .iter()
            .enumerate()
            .skip(regrabbed + 1)
            .find(|(_, state)| state.state == PlayerState::Normal)
            .map(|(frame, _)| frame)
            .unwrap();

        assert!(!trace.states[released].on_ground);
        assert_ne!(trace.states[released].speed.y, 0.0);
        assert!(trace.states[dash - 1].theo_crystals[0].cannot_hold_timer > 0.0);
        assert_eq!(
            trace.states[regrabbed].pickup_old_speed,
            Vec2::new(DASH_SPEED, 0.0)
        );
        assert_eq!(trace.states[regrabbed].speed, Vec2::default());
        assert_eq!(trace.states[restored].speed, Vec2::new(DASH_SPEED, 0.0));
    }

    #[test]
    fn theovator_regrabs_after_updash_speed_is_live_and_restores_it_after_pickup() {
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..60)
            .map(|frame| InputState {
                move_y: if frame == 23 {
                    1
                } else if frame >= 30 {
                    -1
                } else {
                    0
                },
                dash_pressed: frame == 30,
                grab_held: frame <= 22 || frame >= 36,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &theo_crystal_map(), inputs.len() as u32).unwrap();
        let pickup = trace
            .states
            .iter()
            .enumerate()
            .skip(30)
            .find(|(_, state)| state.state == PlayerState::Pickup && state.holding_theo == Some(0))
            .map(|(frame, _)| frame)
            .unwrap();
        let restored = trace
            .states
            .iter()
            .enumerate()
            .skip(pickup + 1)
            .find(|(_, state)| state.state == PlayerState::Normal)
            .map(|(frame, _)| frame)
            .unwrap();

        assert_eq!(
            trace.states[pickup].pickup_old_speed,
            Vec2::new(0.0, -DASH_SPEED)
        );
        assert_eq!(trace.states[pickup].speed, Vec2::default());
        assert_eq!(trace.states[restored].speed.y, -DASH_SPEED);
        assert!(trace.states[restored].pos.y < 160.0);
    }

    #[test]
    fn neutral_drop_climb_jump_regrabs_theo_after_the_lockout() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 160.0, 180.0),
            solids: vec![
                Rect::new(0.0, 176.0, 160.0, 4.0),
                Rect::new(64.0, 0.0, 16.0, 176.0),
            ],
            entities: vec![crate::Entity {
                kind: EntityKind::TheoCrystal,
                bounds: Rect::new(56.0, 90.0, 8.0, 10.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "theoCrystal".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 100.0),
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..50)
            .map(|frame| InputState {
                move_y: if frame == 23 { 1 } else { 0 },
                jump_pressed: frame == 25,
                jump_held: frame == 25,
                grab_held: frame <= 22 || frame >= 24,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &map, inputs.len() as u32).unwrap();
        let dropped = trace
            .states
            .iter()
            .position(|state| {
                state.holding_theo.is_none() && state.theo_crystals[0].cannot_hold_timer > 0.0
            })
            .unwrap();
        let jumped = dropped + 2;
        let regrabbed = trace
            .states
            .iter()
            .enumerate()
            .skip(jumped + 1)
            .find(|(_, state)| state.state == PlayerState::Pickup && state.holding_theo == Some(0))
            .map(|(frame, _)| frame)
            .unwrap();

        assert_eq!(trace.states[jumped].speed.y, JUMP_SPEED);
        assert!(trace.states[jumped].wall_boost_timer > 0.0);
        assert!(trace.states[regrabbed].pos.y < 160.0);
    }

    #[test]
    fn playground_theo_cancels_a_grounded_ultra_before_the_right_wall() {
        let p = PlayerSnapshot {
            pos: Vec2::new(820.0, 496.0),
            speed: Vec2::new(300.0, 0.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..20)
            .map(|frame| InputState {
                move_x: 1,
                move_y: 1,
                dash_pressed: frame == 0,
                grab_held: (5..=12).contains(&frame),
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &crate::mechanics_playground(), 20).unwrap();
        let pickup_index = trace
            .states
            .iter()
            .position(|state| state.state == PlayerState::Pickup)
            .unwrap();

        assert!(pickup_index <= 13);
        assert_eq!(trace.states[pickup_index].pickup_old_speed.x, 360.0);
        assert!(trace.states[pickup_index].pos.x < 864.0);
        assert_eq!(trace.states[pickup_index].holding_theo, Some(0));
    }

    #[test]
    fn grounded_ultra_pickup_cancel_skips_dash_end_speed_normalization() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 160.0),
            speed: Vec2::new(300.0, 0.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..24)
            .map(|frame| InputState {
                move_x: 1,
                move_y: if frame < 10 { 1 } else { 0 },
                dash_pressed: frame == 0,
                grab_held: (5..=20).contains(&frame),
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p.clone(), &inputs, &theo_crystal_map(), 24).unwrap();
        let pickup = trace
            .states
            .iter()
            .position(|state| state.state == PlayerState::Pickup)
            .unwrap();
        let restored = trace
            .states
            .iter()
            .enumerate()
            .skip(pickup + 1)
            .find(|(_, state)| state.state == PlayerState::Normal)
            .map(|(frame, _)| frame)
            .unwrap();

        assert_eq!(trace.states[pickup - 1].state, PlayerState::Dash);
        assert_eq!(trace.states[pickup - 1].speed, Vec2::new(360.0, 0.0));
        assert!(trace.states[pickup - 1].ducking);
        assert_eq!(trace.states[pickup].pickup_old_speed, Vec2::new(360.0, 0.0));
        assert_eq!(trace.states[pickup].speed, Vec2::default());
        assert!(!trace.states[pickup].ducking);
        assert!(!trace.states[pickup].dash_end_pending);
        assert_eq!(trace.states[restored].speed, Vec2::new(360.0, 0.0));
        assert!((trace.states[restored + 1].speed.x - (360.0 - RUN_REDUCE * DT)).abs() < 0.0001);
        assert!(!trace.states[restored + 1].ducking);

        let without_cancel: Vec<_> = inputs
            .iter()
            .map(|input| InputState {
                grab_held: false,
                ..*input
            })
            .collect();
        let natural = simulate_trace(p, &without_cancel, &theo_crystal_map(), 24).unwrap();
        assert!(
            natural
                .states
                .iter()
                .any(|state| state.state == PlayerState::Normal && state.speed.x <= END_DASH_SPEED)
        );
    }

    #[test]
    fn bumper_freeze_smuggle_releases_dashes_and_regrabs_theo() {
        let p = PlayerSnapshot {
            pos: Vec2::new(100.0, 100.0),
            holding_theo: Some(0),
            theo_crystals: vec![crate::TheoCrystalSnapshot {
                position: Vec2::new(100.0, 88.0),
                held: true,
                ..crate::TheoCrystalSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..80)
            .map(|frame| InputState {
                move_y: if frame == 0 || frame >= 18 { 1 } else { 0 },
                dash_pressed: frame == 18,
                grab_held: frame >= 18,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &bumper_theo_map(), inputs.len() as u32).unwrap();
        let dash = trace
            .states
            .iter()
            .position(|state| state.state == PlayerState::Dash)
            .unwrap();
        let regrab = trace
            .states
            .iter()
            .enumerate()
            .skip(dash + 1)
            .find(|(_, state)| state.state == PlayerState::Pickup && state.holding_theo == Some(0))
            .map(|(frame, _)| frame)
            .unwrap();
        assert_eq!(trace.states[1].state, PlayerState::Launch);
        assert_eq!(trace.states[1].freeze_timer, 0.1);
        assert_eq!(trace.states[1].holding_theo, None);
        assert!(dash > 12);
        assert_eq!(trace.states[dash].holding_theo, None);
        assert_eq!(trace.states[dash].dashes, 0);
        assert_eq!(trace.states[regrab].holding_theo, Some(0));
        assert!(trace.states[regrab].pickup_old_speed.y > 0.0);
    }

    #[test]
    fn bumper_smuggle_releases_down_after_buffered_diagonal_dash_to_regrab_theo() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 544.0),
            solids: vec![Rect::new(0.0, 496.0, 320.0, 48.0)],
            entities: vec![
                crate::Entity {
                    kind: EntityKind::TheoCrystal,
                    bounds: Rect::new(96.0, 486.0, 8.0, 10.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "theoCrystal".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::Bumper,
                    bounds: Rect::new(120.0, 480.0, 24.0, 24.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "bigSpinner".to_owned(),
                },
            ],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(100.0, 496.0),
            on_ground: true,
            // Physical collector state 0 for this map. The Bumper starts at
            // its randomly sampled SineWave phase rather than map centre.
            bumpers: vec![crate::BumperSnapshot {
                anchor: Vec2::new(132.0, 492.0),
                position: Vec2::new(132.483_75, 490.006_56),
                sine_counter: 9.262_819,
                respawn_timer: 0.0,
            }],
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..120)
            .map(|frame| InputState {
                move_x: if frame >= 13 { 1 } else { 0 },
                move_y: if frame == 26 || (45..=47).contains(&frame) {
                    1
                } else {
                    0
                },
                dash_pressed: frame == 45,
                grab_held: frame <= 25 || frame >= 45,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &map, inputs.len() as u32).unwrap();
        // Physical 4.24 collector states 26–28. Bumper's PlayerCollider is
        // updated by Bumper.base.Update before SineWave advances, so f27's
        // launch uses f26's Circle(12), then captures the f27 sine position.
        let f26 = &trace.states[26];
        assert_eq!(f26.pos, Vec2::new(113.0, 496.0));
        assert_eq!(f26.speed, Vec2::new(70.0, 0.0));
        assert_eq!(f26.state, PlayerState::Normal);
        assert_eq!(f26.holding_theo, Some(0));
        assert!((f26.bumpers[0].position.x - 129.418_82).abs() < 0.000_1);
        assert!((f26.bumpers[0].position.y - 490.262_4).abs() < 0.000_1);

        let f27 = &trace.states[27];
        assert_eq!(f27.pos, Vec2::new(114.0, 496.0));
        assert_eq!(f27.speed, Vec2::new(-280.0, -150.0));
        assert_eq!(f27.state, PlayerState::Launch);
        assert_eq!(f27.holding_theo, None);
        assert!((f27.bumpers[0].position.x - 129.351_15).abs() < 0.000_1);
        assert!((f27.bumpers[0].position.y - 490.285_7).abs() < 0.000_1);
        assert!((f27.bumpers[0].sine_counter - 10.506_892).abs() < 0.000_1);

        // The 0.1-second ExplodeLaunch freeze skips the next Scene.Update,
        // preserving the f27 player and Bumper state at f28.
        let f28 = &trace.states[28];
        assert_eq!(f28.pos, f27.pos);
        assert_eq!(f28.speed, f27.speed);
        assert_eq!(f28.state, PlayerState::Launch);
        assert_eq!(f28.bumpers[0].position, f27.bumpers[0].position);
        // Physical collector states 85–86. The second Bumper collision uses
        // the position published by that entity update, before PlayerCollider
        // calls OnPlayer. This is deliberately non-horizontal, so it catches
        // an old-position callback that the f27 horizontal launch cannot.
        let f85 = &trace.states[85];
        assert_eq!(f85.pos, Vec2::new(134.0, 482.0));
        assert_eq!(f85.speed, Vec2::new(107.999_88, 30.000_06));
        assert!((f85.bumpers[0].position.x - 132.590_96).abs() < 0.000_1);
        assert!((f85.bumpers[0].position.y - 492.197_97).abs() < 0.000_1);

        let f86 = &trace.states[86];
        assert_eq!(f86.pos, Vec2::new(135.0, 483.0));
        assert_eq!(f86.state, PlayerState::Launch);
        assert!((f86.speed.x - 48.036_976).abs() < 0.000_1);
        assert!((f86.speed.y + 277.123_7).abs() < 0.000_1);
        assert!((f86.last_bumper_target.x - 132.725_8).abs() < 0.000_1);
        assert!((f86.last_bumper_target.y - 492.243_74).abs() < 0.000_1);

        // LaunchUpdate does not require Holding == null before it scans
        // Holdables. After the Bumper's 0.1 s freeze, the still-held Theo is
        // inside its own PickupCollider, so f93 restarts PickupCoroutine.
        // These are physical collector states 91-95.
        for frame in 91..=92 {
            let state = &trace.states[frame];
            assert_eq!(state.state, PlayerState::Launch);
            assert_eq!(state.pos, Vec2::new(135.0, 483.0));
            assert_eq!(state.speed, f86.speed);
            assert_eq!(state.holding_theo, Some(0));
        }
        for frame in 93..=95 {
            let state = &trace.states[frame];
            assert_eq!(state.state, PlayerState::Pickup);
            assert_eq!(state.pos, Vec2::new(135.0, 483.0));
            assert_eq!(state.speed, Vec2::default());
            assert_eq!(state.holding_theo, Some(0));
        }
        assert_eq!(trace.states[93].min_hold_timer, 0.35);
        let pickup = trace
            .states
            .iter()
            .position(|state| state.state == PlayerState::Pickup && state.holding_theo == Some(0))
            .unwrap();
        let launch = trace
            .states
            .iter()
            .enumerate()
            .skip(pickup + 1)
            .find(|(_, state)| state.state == PlayerState::Launch && state.holding_theo.is_none())
            .map(|(frame, _)| frame)
            .unwrap();
        let dash = trace
            .states
            .iter()
            .enumerate()
            .skip(launch + 1)
            .find(|(_, state)| state.state == PlayerState::Dash && state.holding_theo.is_none())
            .map(|(frame, _)| frame)
            .unwrap();
        let regrab = trace
            .states
            .iter()
            .enumerate()
            .skip(dash + 1)
            .find(|(_, state)| state.state == PlayerState::Pickup && state.holding_theo == Some(0))
            .map(|(frame, _)| frame)
            .unwrap_or_else(|| {
                panic!(
                    "missing regrab pickup={pickup} launch={launch} dash={dash}: {:?}",
                    trace
                        .states
                        .iter()
                        .enumerate()
                        .skip(dash)
                        .step_by(5)
                        .map(|(frame, state)| (
                            frame,
                            state.pos,
                            state.speed,
                            state.theo_crystals[0].position,
                            state.holding_theo,
                            state.state,
                        ))
                        .collect::<Vec<_>>()
                )
            });
        assert!(regrab > dash);
        assert!(trace.states[regrab].pickup_old_speed.x > MAX_RUN);
    }

    #[test]
    fn throwable_backboost_adds_eighty_opposite_the_throw_facing() {
        let mut p = PlayerSnapshot {
            pos: Vec2::new(100.0, 100.0),
            speed: Vec2::new(120.0, 0.0),
            facing: false,
            holding_theo: Some(0),
            theo_crystals: vec![crate::TheoCrystalSnapshot {
                position: Vec2::new(100.0, 88.0),
                held: true,
                ..crate::TheoCrystalSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        release_theo(&mut p, InputState::default());

        assert_eq!(p.speed.x, 200.0);
        assert_eq!(p.holding_theo, None);
        assert_eq!(p.theo_crystals[0].speed, Vec2::new(-200.0, -80.0));
        assert_eq!(p.theo_crystals[0].cannot_hold_timer, 0.1);
    }

    #[test]
    fn water_surface_jumps_can_stack_multiple_forty_speed_boosts() {
        let p = PlayerSnapshot {
            pos: Vec2::new(504.0, 428.0),
            state: PlayerState::Swim,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..100)
            .map(|frame| InputState {
                move_x: 1,
                jump_pressed: matches!(frame, 0 | 1 | 2),
                jump_held: false,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &water_map(), inputs.len() as u32).unwrap();
        assert!((trace.states[1].speed.x - 50.0).abs() < 0.000_1);
        assert!((trace.states[2].speed.x - 100.0).abs() < 0.000_1);
        assert!((trace.states[3].speed.x - 135.666_66).abs() < 0.000_1);
        assert_eq!(trace.states[3].speed.y, JUMP_SPEED);
    }

    #[test]
    fn playground_hot_bounce_block_grace_adds_core_super_lift() {
        let p = PlayerSnapshot {
            pos: Vec2::new(384.0, 360.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..38)
            .map(|frame| InputState {
                move_x: if frame >= 32 { 1 } else { 0 },
                dash_pressed: frame == 32,
                jump_pressed: frame == 36,
                jump_held: frame == 36,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &crate::mechanics_playground(), 38).unwrap();

        assert_eq!(trace.states[32].speed, Vec2::new(0.0, -200.0));
        assert_eq!(trace.states[33].state, PlayerState::Dash);
        assert_eq!(trace.states[37].state, PlayerState::Normal);
        assert_eq!(trace.states[37].speed, Vec2::new(260.0, -235.0));
    }

    #[test]
    fn playground_hot_bounce_block_grace_adds_core_hyper_lift() {
        let p = PlayerSnapshot {
            pos: Vec2::new(384.0, 360.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..38)
            .map(|frame| InputState {
                move_x: if frame >= 32 { 1 } else { 0 },
                crouch_dash_pressed: frame == 32,
                jump_pressed: frame == 36,
                jump_held: frame == 36,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &crate::mechanics_playground(), 38).unwrap();

        assert_eq!(trace.states[32].speed, Vec2::new(0.0, -200.0));
        assert!(trace.states[33].ducking);
        assert_eq!(trace.states[37].state, PlayerState::Normal);
        assert_eq!(trace.states[37].speed, Vec2::new(325.0, -117.5));
    }

    #[test]
    fn holdable_core_hyper_releases_during_grace_then_regrabs_after_cannot_hold() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            entities: vec![crate::Entity {
                kind: EntityKind::TheoCrystal,
                bounds: Rect::new(96.0, 78.0, 8.0, 10.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "theoCrystal".to_owned(),
            }],
            ..Map::default()
        };
        let initial = PlayerSnapshot {
            pos: Vec2::new(100.0, 90.0),
            speed: Vec2::new(0.0, -200.0),
            facing: true,
            jump_grace_timer: JUMP_GRACE,
            last_lift_speed: Vec2::new(0.0, -200.0),
            lift_speed_timer: 0.16,
            holding_theo: Some(0),
            theo_crystals: vec![crate::TheoCrystalSnapshot {
                position: Vec2::new(100.0, 78.0),
                held: true,
                ..crate::TheoCrystalSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![InputState {
            move_x: 1,
            crouch_dash_pressed: true,
            ..InputState::default()
        }];
        inputs.extend(
            [InputState {
                move_x: 1,
                ..InputState::default()
            }; 3],
        );
        inputs.push(InputState {
            move_x: 1,
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        });
        inputs.extend(
            [InputState {
                move_x: -1,
                grab_held: true,
                ..InputState::default()
            }; 36],
        );

        let trace = simulate_trace(initial, &inputs, &map, inputs.len() as u32).unwrap();
        let released = trace
            .states
            .iter()
            .position(|state| state.holding_theo.is_none())
            .unwrap();
        let hyper = trace
            .states
            .iter()
            .position(|state| {
                state.state == PlayerState::Normal && (state.speed.x - 325.0).abs() < 0.001
            })
            .unwrap();
        let regrabbed = trace
            .states
            .iter()
            .enumerate()
            .skip(hyper + 1)
            .find(|(_, state)| state.state == PlayerState::Pickup && state.holding_theo == Some(0))
            .map(|(frame, _)| frame)
            .unwrap();

        assert_eq!(released, 1);
        assert!(trace.states[released].theo_crystals[0].cannot_hold_timer > 0.0);
        assert!(
            trace.states[released..regrabbed]
                .iter()
                .all(|state| state.holding_theo.is_none())
        );
        assert!(hyper < regrabbed);
    }

    #[test]
    fn heart_gem_collect_yields_then_freezes_before_setting_half_time_rate() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            entities: vec![crate::Entity {
                kind: EntityKind::HeartGem,
                bounds: Rect::new(92.0, 82.0, 16.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "blackGem".to_owned(),
            }],
            ..Map::default()
        };
        let initial = PlayerSnapshot {
            pos: Vec2::new(100.0, 93.0),
            dash_attack_timer: 0.1,
            ..PlayerSnapshot::default()
        };
        let inputs = [InputState::default(); 20];
        let trace = simulate_trace(initial, &inputs, &map, inputs.len() as u32).unwrap();
        let collected = trace
            .states
            .iter()
            .position(|state| state.heart_gems[0].collected)
            .unwrap();
        let frozen = trace
            .states
            .iter()
            .position(|state| state.freeze_timer >= 0.19)
            .unwrap();
        let half_time = trace
            .states
            .iter()
            .position(|state| (state.time_rate - (0.5 - DT * 0.25)).abs() < 0.001)
            .unwrap();

        assert_eq!(frozen, collected + 2);
        assert!((trace.states[half_time].time_rate - (0.5 - DT * 0.25)).abs() < 0.001);
        assert!(half_time > frozen + 10);
        assert!(
            trace.states[frozen..half_time]
                .windows(2)
                .all(|states| states[0].pos == states[1].pos)
        );
    }

    #[test]
    fn engine_time_rate_scales_the_entire_next_player_update() {
        let initial = PlayerSnapshot {
            pos: Vec2::new(100.0, 100.0),
            speed: Vec2::new(360.0, 0.0),
            time_rate: 0.5,
            ..PlayerSnapshot::default()
        };

        let state = simulate(initial, &[InputState::default()], &Map::default(), 1).unwrap();
        let scaled_dt = DT * 0.5;
        let expected_x = approach(360.0, 0.0, RUN_ACCEL * AIR_MULT * scaled_dt);

        assert!((state.frame_delta_time - scaled_dt).abs() < 0.000_001);
        assert!((state.speed.x - expected_x).abs() < 0.000_001);
        assert!((state.speed.y - GRAVITY * scaled_dt).abs() < 0.000_001);
        assert_eq!(state.pos, Vec2::new(103.0, 100.0));
    }

    #[test]
    fn heart_gem_point_bounces_a_non_dash_attacking_player() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            entities: vec![crate::Entity {
                kind: EntityKind::HeartGem,
                bounds: Rect::new(92.0, 82.0, 16.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "blackGem".to_owned(),
            }],
            ..Map::default()
        };
        let initial = PlayerSnapshot {
            pos: Vec2::new(100.0, 93.0),
            speed: Vec2::new(0.0, 20.0),
            ..PlayerSnapshot::default()
        };
        let state = simulate(initial, &[InputState::default()], &map, 1).unwrap();

        assert_eq!(state.state, PlayerState::Normal);
        assert!(state.speed.y < 0.0);
        assert!(!state.heart_gems[0].collected);
    }

    #[test]
    fn rising_lava_uses_camera_x_and_source_adaptive_rise_speed() {
        let initial = PlayerSnapshot {
            pos: Vec2::new(320.0, 180.0),
            state: PlayerState::Frozen,
            ..PlayerSnapshot::default()
        };

        let state = simulate(
            initial,
            &[InputState::default()],
            &lava_map(EntityKind::RisingLava, 0.0),
            1,
        )
        .unwrap();
        let lava = &state.rising_lavas[0];

        assert_eq!(state.camera, Vec2::new(160.0, 90.0));
        assert_eq!(lava.position.x, state.camera.x);
        assert!((lava.position.y - 353.0).abs() < 0.000_1);
        assert!(!lava.waiting);
        assert!(!lava.ice_mode);
    }

    #[test]
    fn sandwich_lava_waiting_core_mode_and_transition_lifecycle_match_source() {
        let mut map = lava_map(EntityKind::SandwichLava, 100.0);
        let cold = PlayerSnapshot {
            pos: Vec2::new(200.0, 250.0),
            state: PlayerState::Frozen,
            core_mode: crate::CoreMode::Cold,
            ..PlayerSnapshot::default()
        };
        let cold = simulate(cold, &[InputState::default()], &map, 1).unwrap();
        let lava = &cold.sandwich_lavas[0];
        assert!(lava.ice_mode);
        assert!(!lava.waiting);
        assert!((lava.position.y - (350.0 + 20.0 * DT)).abs() < 0.000_1);
        assert_eq!(lava.position.x, cold.camera.x);
        assert!(lava.persistent);

        let waiting = PlayerSnapshot {
            pos: Vec2::new(80.0, 350.0),
            state: PlayerState::Frozen,
            just_respawned: true,
            ..PlayerSnapshot::default()
        };
        let waiting = simulate(waiting, &[InputState::default()], &map, 1).unwrap();
        assert!(waiting.sandwich_lavas[0].waiting);
        assert!(!waiting.dead);

        let leaving = PlayerSnapshot {
            pos: Vec2::new(200.0, 250.0),
            state: PlayerState::Frozen,
            transition_timer: 0.3,
            transition_direction: Vec2::new(1.0, 0.0),
            transition_target: Vec2::new(320.0, 250.0),
            ..PlayerSnapshot::default()
        };
        let leaving = simulate(leaving, &[InputState::default()], &map, 1).unwrap();
        assert!(leaving.sandwich_lavas[0].leaving);
        assert!(leaving.sandwich_lavas[0].leave_timer < 2.0);
    }

    #[test]
    fn lava_player_collider_preserves_the_one_pixel_safe_lip() {
        let mut map = lava_map(EntityKind::RisingLava, 0.0);
        let lava = crate::RisingLavaSnapshot {
            position: Vec2::new(0.0, 100.0),
            initialized: true,
            ..crate::RisingLavaSnapshot::default()
        };
        let safe = PlayerSnapshot {
            pos: Vec2::new(32.0, 102.0),
            state: PlayerState::Frozen,
            camera: Vec2::new(0.0, 0.0),
            camera_initialized: true,
            rising_lavas: vec![lava.clone()],
            ..PlayerSnapshot::default()
        };
        assert!(
            current_player_rect(&safe, safe.pos.x, safe.pos.y)
                .intersects(Rect::new(0.0, 100.0, 340.0, 120.0))
        );
        assert!(!current_player_hurt_rect(&safe).intersects(Rect::new(0.0, 100.0, 340.0, 120.0)));
        let safe = simulate(safe, &[InputState::default()], &map, 1).unwrap();
        assert!(!safe.dead);

        let lethal = PlayerSnapshot {
            pos: Vec2::new(32.0, 103.0),
            state: PlayerState::Frozen,
            camera: Vec2::new(0.0, 0.0),
            camera_initialized: true,
            rising_lavas: vec![lava],
            ..PlayerSnapshot::default()
        };
        let lethal = simulate(lethal, &[InputState::default()], &map, 1).unwrap();
        assert!(lethal.dead);
    }

    #[test]
    fn rising_lava_safe_lip_accepts_a_buffered_neutral_climb_jump() {
        let mut map = lava_map(EntityKind::RisingLava, 760.0);
        map.solids = vec![
            Rect::new(0.0, 496.0, 960.0, 48.0),
            Rect::new(688.0, 360.0, 24.0, 136.0),
        ];
        map.bounds = Rect::new(0.0, 0.0, 960.0, 544.0);
        let initial = PlayerSnapshot {
            pos: Vec2::new(716.0, 494.0),
            state: PlayerState::Climb,
            facing: false,
            stamina: 110.0,
            ..PlayerSnapshot::default()
        };

        let idle_inputs = vec![
            InputState {
                grab_held: true,
                ..InputState::default()
            };
            220
        ];
        let idle = simulate_trace(initial.clone(), &idle_inputs, &map, 220).unwrap();
        let safe_state = idle
            .states
            .iter()
            .enumerate()
            .filter(|(_, state)| {
                let lava = &state.rising_lavas[0];
                let hazard = Rect::new(lava.position.x, lava.position.y, 340.0, 120.0);
                !state.dead
                    && current_player_rect(state, state.pos.x, state.pos.y).intersects(hazard)
                    && !current_player_hurt_rect(state).intersects(hazard)
            })
            .map(|(frame, _)| frame)
            .last()
            .unwrap();
        let death_state = idle.states.iter().position(|state| state.dead).unwrap();
        assert_eq!(safe_state, 169);
        assert!(death_state > safe_state);

        let inputs: Vec<InputState> = (0..220)
            .map(|frame| InputState {
                jump_pressed: frame == safe_state,
                jump_held: frame >= safe_state && frame < safe_state + 8,
                grab_held: frame <= safe_state,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(initial, &inputs, &map, inputs.len() as u32).unwrap();
        let neutral = &trace.states[safe_state + 1];

        assert_eq!(neutral.state, PlayerState::Normal);
        assert!(neutral.wall_boost_timer > 0.0);
        assert!((neutral.speed.y - JUMP_SPEED).abs() < 0.001);
        assert!(!neutral.dead);
    }

    #[test]
    fn cloud_super_and_hyper_stack_the_cloud_lift_with_source_dash_jump_speeds() {
        for (hyper, expected_x, maximum_y) in [(false, 260.0, -105.0), (true, 325.0, -52.5)] {
            let p = PlayerSnapshot {
                pos: Vec2::new(100.0, 100.0),
                on_ground: true,
                ..PlayerSnapshot::default()
            };
            let inputs: Vec<_> = (0..60)
                .map(|frame| InputState {
                    move_x: if frame >= 23 { 1 } else { 0 },
                    dash_pressed: !hyper && frame == 23,
                    crouch_dash_pressed: hyper && frame == 23,
                    jump_pressed: frame == 27,
                    jump_held: frame == 27,
                    ..InputState::default()
                })
                .collect();
            let trace = simulate_trace(p, &inputs, &cloud_map(false), inputs.len() as u32).unwrap();
            let launch = trace
                .states
                .iter()
                .find(|state| {
                    state.state == PlayerState::Normal && (state.speed.x - expected_x).abs() < 0.001
                })
                .expect("cloud dash jump should return to Normal at source horizontal speed");
            assert!(
                launch.speed.y < maximum_y,
                "hyper={hyper}, launch={:?}",
                launch.speed
            );
            assert!(launch.last_lift_speed.y <= -220.0);
        }
    }

    #[test]
    fn cloud_hyper_completes_an_apex_bunnyhop_in_one_runtime_trace() {
        let p = PlayerSnapshot {
            pos: Vec2::new(88.0, 100.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..45)
            .map(|frame| InputState {
                move_x: if (23..=27).contains(&frame) {
                    -1
                } else if frame >= 28 {
                    1
                } else {
                    0
                },
                crouch_dash_pressed: frame == 23,
                jump_pressed: frame == 28 || frame == 37,
                jump_held: frame == 28 || frame == 37,
                ..InputState::default()
            })
            .collect();
        let mut runtime_map = cloud_map(false);
        runtime_map.solids.push(Rect::new(116.0, 82.0, 160.0, 8.0));
        let trace = simulate_trace(p.clone(), &inputs, &runtime_map, inputs.len() as u32).unwrap();
        assert_eq!(trace.states[28].speed, Vec2::new(-DASH_SPEED, 0.0));
        assert_eq!(trace.states[29].state, PlayerState::Normal);
        assert_eq!(trace.states[29].speed.x, SUPER_JUMP_H * 1.25);

        let apex_y = trace
            .states
            .iter()
            .map(|state| state.clouds[0].position.y)
            .min_by(f32::total_cmp)
            .unwrap();
        let landed = &trace.states[37];
        assert!(landed.on_ground);
        assert_eq!(landed.pos.y, apex_y);
        assert!((landed.clouds[0].position.y - apex_y).abs() <= 1.0);
        let bunnyhop = &trace.states[38];
        assert_eq!(bunnyhop.state, PlayerState::Normal);
        assert!(bunnyhop.speed.x > 250.0);
        assert!((bunnyhop.speed.y - -175.000_47).abs() < 0.001);
        assert!(!bunnyhop.on_ground);

        let whole = trace.states.last().unwrap().clone();
        let first = simulate(p, &inputs[..32], &runtime_map, 32).unwrap();
        let split = simulate(
            first,
            &inputs[32..],
            &runtime_map,
            (inputs.len() - 32) as u32,
        )
        .unwrap();
        assert_eq!(split, whole);
    }

    #[test]
    fn jump_adds_retained_lift_boost_before_caching_variable_jump_speed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            on_ground: true,
            last_lift_speed: Vec2::new(40.0, -80.0),
            lift_speed_timer: 0.16,
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };

        let p = simulate(p, &[input], &floor_map(), 1).unwrap();
        assert_eq!(p.speed, Vec2::new(40.0, -185.0));
        assert_eq!(p.var_jump_speed, -185.0);
    }
    #[test]
    fn dash_caches_lift_boost_in_before_dash_speed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            speed: Vec2::new(20.0, 0.0),
            on_ground: true,
            last_lift_speed: Vec2::new(300.0, -80.0),
            lift_speed_timer: 0.16,
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            move_x: 1,
            dash_pressed: true,
            ..InputState::default()
        };

        let p = simulate(p, &[input], &floor_map(), 1).unwrap();
        assert_eq!(p.state, PlayerState::Dash);
        assert_eq!(p.before_dash_speed, Vec2::new(270.0, -80.0));
    }
    #[test]
    fn delayed_climb_wall_jump_uses_retained_lift_speed() {
        let mut map = Map {
            solids: vec![Rect::new(36.0, 0.0, 8.0, 180.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 80.0),
            state: PlayerState::Climb,
            facing: true,
            stamina: 110.0,
            last_lift_speed: Vec2::new(20.0, -30.0),
            lift_speed_timer: 0.16,
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            move_x: -1,
            jump_pressed: true,
            jump_held: true,
            grab_held: true,
            ..InputState::default()
        };

        let p = simulate(p, &[input], &map, 1).unwrap();
        assert_eq!(p.state, PlayerState::Normal);
        assert_eq!(p.speed, Vec2::new(-110.0, -135.0));
        assert_eq!(p.var_jump_speed, -135.0);
    }
    #[test]
    fn coyote_jump_consumes_source_grace_window_after_leaving_a_ledge() {
        let mut map = Map {
            solids: vec![Rect::new(0.0, 100.0, 36.0, 80.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            speed: Vec2::new(90.0, 0.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState {
                move_x: 1,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            },
        ];
        let p = simulate(p, &inputs, &map, inputs.len() as u32).unwrap();
        assert!(!p.on_ground);
        assert_eq!(p.speed.y, JUMP_SPEED);
        assert_eq!(p.jump_grace_timer, 0.0);
    }
    #[test]
    fn buffered_jump_fires_on_the_first_grounded_update() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 91.0),
            speed: Vec2::new(0.0, 100.0),
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState {
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                jump_held: true,
                ..InputState::default()
            },
        ];
        let trace = simulate_trace(p, &inputs, &floor_map(), inputs.len() as u32).unwrap();
        assert!(trace.states.iter().any(|state| state.on_ground));
        assert_eq!(trace.states.last().unwrap().speed.y, JUMP_SPEED);
        assert!(trace.states.last().unwrap().jump_buffer_timer <= 0.0);
    }
    #[test]
    fn bunnyhop_buffers_the_landing_and_reapplies_horizontal_jump_boost() {
        let player = PlayerSnapshot {
            pos: Vec2::new(32.0, 91.0),
            speed: Vec2::new(160.0, 100.0),
            facing: true,
            ..PlayerSnapshot::default()
        };
        let bunnyhop = std::array::from_fn::<_, 8, _>(|frame| InputState {
            move_x: 1,
            jump_pressed: frame == 0,
            jump_held: true,
            ..InputState::default()
        });
        let control = [InputState {
            move_x: 1,
            ..InputState::default()
        }; 8];
        let bunnyhop = simulate_trace(player.clone(), &bunnyhop, &floor_map(), 8).unwrap();
        let control = simulate_trace(player, &control, &floor_map(), 8).unwrap();
        let jump_state = (1..bunnyhop.states.len())
            .find(|&frame| bunnyhop.states[frame].speed.y == JUMP_SPEED)
            .expect("buffered jump fires after the landing state");
        assert!(bunnyhop.states[jump_state - 1].on_ground);
        assert!(!bunnyhop.states[jump_state].on_ground);
        let expected_speed =
            bunnyhop.states[jump_state - 1].speed.x - RUN_REDUCE * DT + JUMP_H_BOOST;
        assert!((bunnyhop.states[jump_state].speed.x - expected_speed).abs() < 0.001);
        assert!(
            (bunnyhop.states[jump_state].speed.x
                - control.states[jump_state].speed.x
                - JUMP_H_BOOST)
                .abs()
                < 0.001
        );
        assert_eq!(bunnyhop.states[jump_state].state, PlayerState::Normal);
        assert!(bunnyhop.states[jump_state].facing);
        assert_eq!(bunnyhop.states[jump_state].dashes, 1);
        assert_eq!(bunnyhop.states[jump_state].stamina, 110.0);
        assert!(!bunnyhop.states[jump_state].ducking);
        assert!(!bunnyhop.states[jump_state].dead);
    }
    #[test]
    fn crouch_jump_keeps_the_short_hitbox_until_falling_in_open_air() {
        let inputs = std::array::from_fn::<_, 40, _>(|frame| InputState {
            move_y: if frame <= 1 { 1 } else { 0 },
            jump_pressed: frame == 1,
            jump_held: (1..10).contains(&frame),
            ..InputState::default()
        });
        let trace = simulate_trace(grounded_player(), &inputs, &floor_map(), 40).unwrap();
        assert!(trace.states[1].on_ground);
        assert!(trace.states[1].ducking);
        assert_eq!(trace.states[2].speed.y, JUMP_SPEED);
        assert!(trace.states[2].ducking);
        let falling_state = (3..trace.states.len())
            .find(|&frame| trace.states[frame].speed.y > 0.0)
            .expect("crouch jump reaches its falling phase");
        assert!(
            trace.states[2..falling_state]
                .iter()
                .all(|state| state.ducking)
        );
        assert!(!trace.states[falling_state].ducking);

        let low_ceiling = Map {
            solids: vec![Rect::new(0.0, 90.0, 64.0, 4.0)],
            ..Map::default()
        };
        let falling_under_ceiling = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            speed: Vec2::new(0.0, 30.0),
            ducking: true,
            ..PlayerSnapshot::default()
        };
        let falling_under_ceiling = simulate(
            falling_under_ceiling,
            &[InputState::default()],
            &low_ceiling,
            1,
        )
        .unwrap();
        assert!(falling_under_ceiling.ducking);
        assert!(!can_unduck(&falling_under_ceiling, &low_ceiling));
    }
    /// `Player.cs:2969-3004`: the whole wall-jump / water-jump branch of
    /// `NormalUpdate` sits inside `else if (CanUnDuck)`, so a crouched player who
    /// cannot stand up where it is swallows the press completely - no `WallJump`,
    /// no `ClimbJump`, no water `Jump`, and `Ducking` stays true. The same press
    /// beside the same wall does launch when the player can stand.
    #[test]
    fn crouched_jump_press_is_swallowed_while_can_unduck_is_false() {
        // x 20..28 is reachable by `WallJumpCheck(-1)`'s three-pixel probe
        // (`Player.cs`'s `WallJumpCheckDist`) from a player whose hitbox starts at
        // x 28, and y 94..124 overlaps the six-pixel crouch hitbox but not the
        // normal eleven-pixel one.
        let wall = Rect::new(20.0, 94.0, 8.0, 30.0);
        let open_wall = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 360.0),
            solids: vec![wall],
            ..Map::default()
        };
        // Adding a ceiling at y 80..94 keeps the crouch hitbox (y 94..100) clear
        // while the normal hitbox (y 89..100) would collide, so `CanUnDuck` is
        // false exactly as it is inside a one-tile crawlspace.
        let crawlspace = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 360.0),
            solids: vec![Rect::new(0.0, 80.0, 320.0, 14.0), wall],
            ..Map::default()
        };
        let input = InputState {
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };

        let upright = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            speed: Vec2::new(77.0, -20.0),
            ..PlayerSnapshot::default()
        };
        assert!(can_unduck(&upright, &open_wall));
        assert!(wall_jump_check(&upright, &open_wall, -1));
        let launched = simulate(upright, &[input], &open_wall, 1).unwrap();
        assert_eq!(launched.speed.x, WALL_JUMP_H);
        assert_eq!(launched.speed.y, JUMP_SPEED);
        assert_eq!(launched.var_jump_timer, VAR_JUMP_TIME);
        assert!(!launched.ducking);

        let crouched = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            speed: Vec2::new(77.0, -20.0),
            ducking: true,
            ..PlayerSnapshot::default()
        };
        assert!(!can_unduck(&crouched, &crawlspace));
        assert!(wall_jump_check(&crouched, &crawlspace, -1));
        let swallowed = simulate(crouched, &[input], &crawlspace, 1).unwrap();
        assert!(swallowed.ducking);
        assert_eq!(swallowed.var_jump_timer, 0.0);
        assert_eq!(swallowed.var_jump_speed, 0.0);
        assert_ne!(swallowed.speed.x, WALL_JUMP_H);
        assert_ne!(swallowed.speed.y, JUMP_SPEED);
    }
    /// `Player.cs:3154-3167`: a facing-side `WallBooster` the player overlaps takes
    /// over the whole vertical block once `climbNoMoveTimer` has run out, ramping
    /// `Speed.Y` toward `WallBoosterSpeed` at `WallBoosterAccel` and publishing
    /// `UnitY * Math.Max(Speed.Y, WallBoosterLiftSpeed)` as `LiftSpeed`. Before
    /// that timer expires the ordinary climb target still owns the frame.
    #[test]
    fn wall_booster_ramps_climb_speed_and_publishes_lift_speed() {
        // Player at pos.x 36 has its 8 px hitbox on x 32..40, flush against the
        // wall at 40..48; the conveyor serving that face is a `left: false` one,
        // whose 2 px strip sits 6 px right of its entity position, so at 38..40 it
        // overlaps the player exactly as `WallBooster.cs:38` places it.
        let conveyor = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![Rect::new(40.0, 0.0, 8.0, 184.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::WallBooster,
                bounds: Rect::new(38.0, 40.0, 2.0, 40.0),
                direction: Vec2::new(1.0, 0.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "wallBooster".to_owned(),
            }],
            ..Map::default()
        };
        let climbing = PlayerSnapshot {
            pos: Vec2::new(36.0, 64.0),
            state: PlayerState::Climb,
            facing: true,
            stamina: 80.0,
            // What `ClimbBegin` (`Player.cs:3882-3891`) installs on a fresh grab.
            climb_no_move_timer: 0.1,
            ..PlayerSnapshot::default()
        };
        assert!(wall_booster_check(&climbing, &conveyor, 1));
        assert!(!wall_booster_check(&climbing, &conveyor, -1));

        let inputs = [InputState {
            grab_held: true,
            ..InputState::default()
        }; 14];
        let trace = simulate_trace(climbing, &inputs, &conveyor, 14).unwrap();
        assert!(
            !trace.states[3].wall_boosting,
            "climbNoMoveTimer still gates the conveyor"
        );
        let ramp = trace
            .states
            .iter()
            .filter(|s| s.wall_boosting)
            .map(|s| s.speed.y)
            .collect::<Vec<_>>();
        assert!(ramp.len() >= 8, "the conveyor takes over for the rest: {ramp:?}");
        for pair in ramp.windows(2) {
            assert!((pair[0] - pair[1] - WALL_BOOSTER_ACCEL * DT).abs() < 0.001);
        }
        for state in trace.states.iter().filter(|s| s.wall_boosting) {
            assert_eq!(state.speed.x, 0.0);
            assert!(state.speed.y >= WALL_BOOSTER_SPEED);
            // `Actor.LiftSpeed` falls back to `lastLiftSpeed` once the frame's
            // `currentLiftSpeed` is cleared, so read it the way the source does.
            assert_eq!(lift_speed(state).x, 0.0);
            assert_eq!(
                lift_speed(state).y,
                state.speed.y.max(WALL_BOOSTER_LIFT_SPEED)
            );
        }
        // Past -80 the published lift is the cap, not the live speed.
        let last = trace.states.last().unwrap();
        assert!(last.speed.y < WALL_BOOSTER_LIFT_SPEED);
        assert_eq!(lift_speed(last).y, WALL_BOOSTER_LIFT_SPEED);
    }
    /// `Player.SuperBounce` (`Player.cs:2708-2722`) and `Player.SideBounce`
    /// (`:2741-2762`) wrap their dash refill in `if (!Inventory.NoRefills)`, so a spring
    /// bounce in the Core hands back no dash - unlike `Player.PointBounce` (`:3061`),
    /// which is unguarded, and `Player.Bounce` (`:2677-2691`), which guards only the
    /// dash and always refills stamina.
    #[test]
    fn spring_bounces_refill_dashes_only_without_no_refills() {
        let bounced = |no_refills| {
            let mut p = PlayerSnapshot {
                dashes: 0,
                max_dashes: 2,
                no_refills,
                ..PlayerSnapshot::default()
            };
            super_bounce(&mut p, &Map::default(), 100.0);
            p
        };
        assert_eq!(bounced(false).dashes, 2);
        assert_eq!(bounced(false).stamina, 110.0);
        assert_eq!(bounced(true).dashes, 0);
        assert_eq!(bounced(true).stamina, 110.0);

        let side = |no_refills| {
            let mut p = PlayerSnapshot {
                dashes: 0,
                max_dashes: 2,
                no_refills,
                ..PlayerSnapshot::default()
            };
            side_bounce(&mut p, &Map::default(), 1, Rect::new(40.0, 60.0, 6.0, 16.0));
            p
        };
        assert_eq!(side(false).dashes, 2);
        assert_eq!(side(true).dashes, 0);
    }
    /// `Stamina` has no floor in the source: `Player.cs:4060`/`4078` are bare
    /// subtractions, so a climb that outlasts the bar leaves a negative value behind.
    /// Flooring it made every such frame disagree with the game while changing nothing
    /// the `Stamina <= 0` tests could see.
    #[test]
    fn climb_drain_lets_stamina_go_negative_like_the_source() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![Rect::new(40.0, 0.0, 8.0, 184.0)],
            ..Map::default()
        };
        let climbing = PlayerSnapshot {
            pos: Vec2::new(36.0, 64.0),
            state: PlayerState::Climb,
            facing: true,
            stamina: 1.0,
            ..PlayerSnapshot::default()
        };
        let inputs = [InputState {
            grab_held: true,
            move_y: -1,
            ..InputState::default()
        }; 3];
        let trace = simulate_trace(climbing, &inputs, &map, 3).unwrap();
        assert_eq!(trace.states[1].stamina, 1.0 - CLIMB_UP_COST * DT);
        assert_eq!(trace.states[1].state, PlayerState::Climb);
        assert!(
            trace.states[2].stamina < 0.0,
            "the drain is not floored: {:?}",
            trace.states[2].stamina
        );
        assert_eq!(trace.states[2].state, PlayerState::Normal);
    }
    /// `Level.InSpace` (`Level.cs:449`) scales the run target
    /// (`Player.cs:2889-2890`), both fall caps (`2904-2908`) and gravity
    /// (`2954-2955`) by `SpacePhysicsMult = 0.6f`.
    #[test]
    fn space_rooms_scale_run_target_fall_caps_and_gravity() {
        let mut map = floor_map();
        let grounded = PlayerSnapshot {
            in_space: true,
            ..grounded_player()
        };
        let running = [InputState {
            move_x: 1,
            ..InputState::default()
        }; 12];
        let space = simulate_trace(grounded.clone(), &running, &map, 12).unwrap();
        assert_eq!(space.states[12].speed.x, MAX_RUN * SPACE_PHYSICS_MULT);
        let plain = simulate_trace(
            PlayerSnapshot {
                in_space: false,
                ..grounded
            },
            &running,
            &map,
            12,
        )
        .unwrap();
        assert_eq!(plain.states[12].speed.x, MAX_RUN);

        let falling = |in_space| PlayerSnapshot {
            pos: Vec2::new(32.0, 40.0),
            on_ground: false,
            in_space,
            max_fall: MAX_FALL,
            ..PlayerSnapshot::default()
        };
        let one = [InputState::default(); 1];
        assert_eq!(
            simulate(falling(true), &one, &map, 1).unwrap().speed.y,
            GRAVITY * SPACE_PHYSICS_MULT * DT
        );
        assert_eq!(
            simulate(falling(false), &one, &map, 1).unwrap().speed.y,
            GRAVITY * DT
        );
        let long = [InputState::default(); 60];
        let capped = simulate_trace(falling(true), &long, &map, 60).unwrap();
        assert_eq!(capped.states[60].max_fall, MAX_FALL * SPACE_PHYSICS_MULT);
    }
    /// `CoreModeToggle.OnPlayer` (`CoreModeToggle.cs:103-126`): when the player's
    /// hurtbox touches the switch and it is `Usable` (`:24-38`) it flips
    /// `Level.CoreMode`, freezes the engine for 0.05 s and refuses to fire again for
    /// one second.
    #[test]
    fn core_mode_toggle_flips_level_mode_then_cools_down() {
        let map_with = |direction: Vec2| Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![Rect::new(0.0, 100.0, 320.0, 84.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::CoreModeToggle,
                bounds: Rect::new(28.0, 52.0, 16.0, 24.0),
                direction,
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "coreModeToggle".to_owned(),
            }],
            ..Map::default()
        };
        // The vanilla `9-Core|0|d-03` switch is `onlyFire`, so only an icy room may use it.
        let only_fire = map_with(Vec2::new(1.0, 0.0));
        let always = map_with(Vec2::default());
        let player = |core_mode| PlayerSnapshot {
            pos: Vec2::new(36.0, 68.0),
            core_mode,
            ..PlayerSnapshot::default()
        };
        let inputs = [InputState::default(); 10];

        let from_cold = simulate_trace(player(crate::CoreMode::Cold), &inputs, &only_fire, 10).unwrap();
        assert_eq!(from_cold.states[1].core_mode, crate::CoreMode::Hot);
        assert!(from_cold.states[1].freeze_timer > 0.0);
        assert!(
            from_cold.states[1].core_mode_toggle_cooldowns[0] > 0.9,
            "the flip re-arms the one-second cooldown: {:?}",
            from_cold.states[1].core_mode_toggle_cooldowns
        );
        assert_eq!(from_cold.states[10].core_mode, crate::CoreMode::Hot);

        // `onlyFire` while already Hot: not usable, so nothing happens at all.
        let from_hot = simulate_trace(player(crate::CoreMode::Hot), &inputs, &only_fire, 10).unwrap();
        assert_eq!(from_hot.states[10].core_mode, crate::CoreMode::Hot);
        assert_eq!(from_hot.states[10].core_mode_toggle_cooldowns[0], 0.0);

        // A switch that is always usable still may not flip back while its cooldown runs:
        // the flip lands on frame 1, the 0.05 s freeze skips the next frames, and the
        // cooldown (one second) then refuses the re-flip that would otherwise turn it Cold
        // again on the first frame Player.Update resumes.
        let always_usable = simulate_trace(player(crate::CoreMode::Cold), &inputs, &always, 10).unwrap();
        assert_eq!(always_usable.states[1].core_mode, crate::CoreMode::Hot);
        assert_eq!(always_usable.states[10].core_mode, crate::CoreMode::Hot);
        assert!(
            always_usable.states[10].core_mode_toggle_cooldowns[0]
                < always_usable.states[1].core_mode_toggle_cooldowns[0],
            "the cooldown counts down once Player.Update resumes: {:?} -> {:?}",
            always_usable.states[1].core_mode_toggle_cooldowns,
            always_usable.states[10].core_mode_toggle_cooldowns
        );
    }
    #[test]
    fn downward_air_dash_keeps_ducking_until_coyote_grace_expires() {
        let p = PlayerSnapshot {
            pos: Vec2::new(160.0, 100.0),
            jump_grace_timer: JUMP_GRACE,
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState {
            move_x: 1,
            move_y: 1,
            ..InputState::default()
        }; 10];
        inputs[0].dash_pressed = true;
        let trace = simulate_trace(p, &inputs, &Map::default(), inputs.len() as u32).unwrap();

        assert_eq!(trace.states[5].state, PlayerState::Dash);
        assert!(trace.states[5].speed.y > 0.0);
        assert!(trace.states[5].jump_grace_timer > 0.0);
        assert!(trace.states[5].ducking);
        assert_eq!(trace.states[9].jump_grace_timer, 0.0);
        assert!(!trace.states[9].ducking);
    }
    #[test]
    fn superdash_sets_source_launch_speed_and_spends_dash() {
        let mut inputs = [InputState::default(); 5];
        inputs[0] = InputState {
            move_x: 1,
            dash_pressed: true,
            ..InputState::default()
        };
        inputs[4] = InputState {
            move_x: 1,
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };
        let p = simulate(
            grounded_player(),
            &inputs,
            &floor_map(),
            inputs.len() as u32,
        )
        .unwrap();
        assert_eq!(p.state, PlayerState::Normal);
        assert_eq!(p.speed, Vec2::new(SUPER_JUMP_H, JUMP_SPEED));
        assert_eq!(p.dashes, 0);
    }
    #[test]
    fn hyperdash_applies_duck_super_multipliers() {
        let mut inputs = [InputState::default(); 5];
        inputs[0] = InputState {
            move_x: 1,
            move_y: 1,
            dash_pressed: true,
            ..InputState::default()
        };
        for input in &mut inputs[1..4] {
            input.move_x = 1;
            input.move_y = 1;
        }
        inputs[4] = InputState {
            move_x: 1,
            move_y: 1,
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };
        let p = simulate(
            grounded_player(),
            &inputs,
            &floor_map(),
            inputs.len() as u32,
        )
        .unwrap();
        assert_eq!(p.speed, Vec2::new(325.0, -52.5));
        assert!(!p.ducking);
    }
    #[test]
    fn initial_dash_frame_jump_uses_zero_dash_dir_for_instant_super_or_hyper() {
        for (hold_down, expected_speed) in [
            (false, Vec2::new(260.0, -105.0)),
            (true, Vec2::new(325.0, -52.5)),
        ] {
            let mut inputs = [InputState::default(); 8];
            inputs[0] = InputState {
                move_y: i8::from(hold_down),
                dash_pressed: true,
                ..InputState::default()
            };
            for (frame, input) in inputs.iter_mut().enumerate().skip(1) {
                input.move_y = i8::from(hold_down);
                input.jump_pressed = frame == 1;
                input.jump_held = true;
            }
            let trace = simulate_trace(
                grounded_player(),
                &inputs,
                &floor_map(),
                inputs.len() as u32,
            )
            .unwrap();
            let launched = trace
                .states
                .iter()
                .find(|state| state.speed == expected_speed);
            assert!(
                launched.is_some(),
                "hold_down={hold_down}: {:#?}",
                trace.states
            );
            assert_eq!(launched.unwrap().state, PlayerState::Normal);
        }
    }
    #[test]
    fn wavedash_landing_converts_down_diagonal_dash_to_hyper() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 84.0),
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![InputState::default(); 12];
        inputs[0] = InputState {
            move_x: 1,
            move_y: 1,
            dash_pressed: true,
            ..InputState::default()
        };
        for input in &mut inputs[1..=10] {
            input.move_x = 1;
            input.move_y = 1;
        }
        inputs[10].jump_pressed = true;
        inputs[10].jump_held = true;
        inputs[11] = InputState {
            move_x: 1,
            jump_held: true,
            ..InputState::default()
        };
        let trace = simulate_trace(p, &inputs, &floor_map(), inputs.len() as u32).unwrap();
        assert!(
            trace
                .states
                .iter()
                .any(|state| state.on_ground && state.ducking)
        );
        assert_eq!(trace.states.last().unwrap().state, PlayerState::Normal);
        assert_eq!(trace.states.last().unwrap().speed.y, -52.5);
        assert!(trace.states.last().unwrap().speed.x >= 320.0);
    }
    #[test]
    fn wavedash_buffers_jump_at_the_fourteen_pixel_minimum_height() {
        let p = PlayerSnapshot {
            // At fourteen pixels the fifth dash step ends exactly flush with
            // the floor. The following frame checks Jump.Pressed while the
            // dash is still diagonal, then the vertical collision converts
            // DashDir. The buffered press must survive one more Dash frame.
            pos: Vec2::new(32.0, 86.0),
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![InputState::default(); 12];
        for input in &mut inputs {
            input.move_x = 1;
            input.move_y = 1;
        }
        inputs[0].dash_pressed = true;
        inputs[9].jump_pressed = true;
        inputs[9].jump_held = true;
        inputs[10].jump_held = true;

        let trace = simulate_trace(p, &inputs, &floor_map(), inputs.len() as u32).unwrap();
        let landing = &trace.states[10];
        assert_eq!(landing.state, PlayerState::Dash);
        assert!(landing.on_ground);
        assert!(landing.ducking);
        assert_eq!(landing.dash_dir, Vec2::new(1.0, 0.0));
        assert!(landing.jump_buffer_timer > 0.0);
        assert_eq!(landing.dashes, 0);
        assert_eq!(landing.dash_refill_cooldown_timer, 0.0);

        let wavedash = &trace.states[11];
        assert_eq!(wavedash.state, PlayerState::Normal);
        assert_eq!(wavedash.speed, Vec2::new(325.0, -52.5));
        assert_eq!(wavedash.dashes, 1);
        assert!(!wavedash.ducking);
        assert_eq!(wavedash.jump_buffer_timer, 0.0);
    }
    #[test]
    fn thirteen_pixel_wavedash_control_jumps_before_dash_refill() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 87.0),
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![InputState::default(); 12];
        for input in &mut inputs {
            input.move_x = 1;
            input.move_y = 1;
        }
        inputs[0].dash_pressed = true;
        inputs[9].jump_pressed = true;
        inputs[9].jump_held = true;

        let trace = simulate_trace(p, &inputs, &floor_map(), inputs.len() as u32).unwrap();
        let too_low = &trace.states[10];
        assert_eq!(too_low.state, PlayerState::Normal);
        assert_eq!(too_low.speed, Vec2::new(325.0, -52.5));
        assert_eq!(too_low.dashes, 0);
    }
    #[test]
    fn reverse_super_uses_jump_frame_facing_not_dash_direction() {
        let mut inputs = [InputState::default(); 6];
        inputs[0] = InputState {
            move_x: 1,
            dash_pressed: true,
            ..InputState::default()
        };
        for input in &mut inputs[1..=4] {
            input.move_x = 1;
        }
        inputs[5] = InputState {
            move_x: -1,
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };
        let p = simulate(
            grounded_player(),
            &inputs,
            &floor_map(),
            inputs.len() as u32,
        )
        .unwrap();
        assert_eq!(p.dash_dir, Vec2::new(1.0, 0.0));
        assert_eq!(p.speed, Vec2::new(-SUPER_JUMP_H, JUMP_SPEED));
        assert!(!p.facing);
    }
    #[test]
    fn extended_super_refills_dash_before_late_dash_jump() {
        let mut inputs = vec![InputState::default(); 11];
        inputs[0] = InputState {
            move_x: 1,
            dash_pressed: true,
            ..InputState::default()
        };
        for input in &mut inputs[1..] {
            input.move_x = 1;
        }
        inputs[10].jump_pressed = true;
        inputs[10].jump_held = true;
        let p = simulate(
            grounded_player(),
            &inputs,
            &floor_map(),
            inputs.len() as u32,
        )
        .unwrap();
        assert_eq!(p.state, PlayerState::Normal);
        assert_eq!(p.dashes, 1);
        assert_eq!(p.speed.y, JUMP_SPEED);
    }
    #[test]
    fn superwave_chains_extended_super_into_a_reverse_wavedash() {
        let mut inputs = vec![InputState::default(); 30];
        for input in &mut inputs[..=10] {
            input.move_x = 1;
        }
        inputs[0].dash_pressed = true;
        inputs[10].jump_pressed = true;
        inputs[10].jump_held = true;
        for input in &mut inputs[11..] {
            input.move_x = -1;
            input.move_y = 1;
        }
        inputs[11].dash_pressed = true;
        inputs[26].jump_pressed = true;
        inputs[26].jump_held = true;
        let trace = simulate_trace(
            grounded_player(),
            &inputs,
            &floor_map(),
            inputs.len() as u32,
        )
        .unwrap();
        assert_eq!(trace.states[11].speed, Vec2::new(SUPER_JUMP_H, JUMP_SPEED));
        assert_eq!(trace.states[11].dashes, 1);
        assert!(trace.states[22].on_ground);
        assert!(trace.states[22].ducking);
        assert!(trace.states[22].speed.x < -200.0);
        assert_eq!(trace.states[27].speed, Vec2::new(-325.0, -52.5));
        assert_eq!(trace.states[27].dashes, 1);
    }
    #[test]
    fn upward_diagonal_demo_keeps_crouched_dash_hitbox() {
        let mut inputs = [InputState::default(); 5];
        inputs[0] = InputState {
            move_x: 1,
            move_y: -1,
            crouch_dash_pressed: true,
            ..InputState::default()
        };
        for input in &mut inputs[1..] {
            input.move_x = 1;
            input.move_y = -1;
        }
        let p = simulate(
            grounded_player(),
            &inputs,
            &floor_map(),
            inputs.len() as u32,
        )
        .unwrap();
        assert!(p.demo_dashed);
        assert!(p.ducking);
        assert!((p.speed.x - DASH_SPEED * std::f32::consts::FRAC_1_SQRT_2).abs() < 0.01);
        assert!((p.speed.y + DASH_SPEED * std::f32::consts::FRAC_1_SQRT_2).abs() < 0.01);
    }
    #[test]
    fn demohyper_uses_crouched_super_launch_from_horizontal_demo() {
        let mut inputs = [InputState::default(); 5];
        inputs[0] = InputState {
            move_x: 1,
            crouch_dash_pressed: true,
            ..InputState::default()
        };
        inputs[4] = InputState {
            move_x: 1,
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };
        let trace = simulate_trace(
            grounded_player(),
            &inputs,
            &floor_map(),
            inputs.len() as u32,
        )
        .unwrap();
        assert!(trace.states[1].demo_dashed);
        assert!(trace.states[1].ducking);
        assert_eq!(trace.states.last().unwrap().speed, Vec2::new(325.0, -52.5));
    }
    #[test]
    fn wallbounce_sets_super_wall_jump_speed_and_var_window() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 0.0, 8.0, 180.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(36.0, 100.0),
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState::default(); 6];
        inputs[0] = InputState {
            move_y: -1,
            dash_pressed: true,
            ..InputState::default()
        };
        for input in &mut inputs[1..=4] {
            input.move_y = -1;
        }
        inputs[5] = InputState {
            move_y: -1,
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };
        let p = simulate(p, &inputs, &map, inputs.len() as u32).unwrap();
        assert_eq!(p.state, PlayerState::Normal);
        assert_eq!(p.speed, Vec2::new(-170.0, -160.0));
        assert_eq!(p.var_jump_timer, 0.25);
    }
    #[test]
    fn spiked_wallbounce_is_safe_on_the_entry_frame_but_dies_one_frame_late() {
        let mut map = Map {
            solids: vec![Rect::new(100.0, 0.0, 8.0, 180.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::Spikes,
                bounds: Rect::new(97.0, 60.0, 3.0, 20.0),
                direction: Vec2::new(-1.0, 0.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "spikesLeft".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(96.0, 95.0),
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut on_time = [InputState::default(); 7];
        on_time[0] = InputState {
            move_y: -1,
            dash_pressed: true,
            ..InputState::default()
        };
        for input in &mut on_time[1..=4] {
            input.move_y = -1;
        }
        on_time[5] = InputState {
            move_y: -1,
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };
        let on_time = simulate_trace(p.clone(), &on_time, &map, on_time.len() as u32).unwrap();
        assert!(!on_time.states.last().unwrap().dead);
        assert_eq!(on_time.states[6].state, PlayerState::Normal);
        assert_eq!(on_time.states[6].speed, Vec2::new(-170.0, -160.0));

        let mut late = [InputState::default(); 7];
        late[0] = InputState {
            move_y: -1,
            dash_pressed: true,
            ..InputState::default()
        };
        for input in &mut late[1..=4] {
            input.move_y = -1;
        }
        late[6] = InputState {
            move_y: -1,
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };
        let late = simulate(p, &late, &map, late.len() as u32).unwrap();
        assert!(late.dead);
    }
    #[test]
    fn fastfall_approaches_source_240_terminal_speed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 32.0),
            speed: Vec2::new(0.0, 160.0),
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            move_y: 1,
            ..InputState::default()
        };
        let p = simulate(p, &[input; 16], &Map::default(), 16).unwrap();
        assert_eq!(p.max_fall, FAST_MAX_FALL);
        assert_eq!(p.speed.y, FAST_MAX_FALL);
    }
    #[test]
    fn holding_down_suppresses_the_wall_slide_fall_target() {
        // Player.cs:3749 wraps the whole wall-slide block, including the
        // `target2 = MathHelper.Lerp(160f, 20f, wallSlideTimer / 1.2f)` fall
        // target (Player.cs:3766), in `Input.MoveY.Value != 1`: a held-down
        // fast-fall beside a wall keeps the ordinary `maxFall` target.
        let mut map = Map {
            solids: vec![Rect::new(36.0, 0.0, 8.0, 200.0)],
            ..Map::default()
        };
        let base = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            speed: Vec2::new(0.0, 98.000_183),
            max_fall: MAX_FALL,
            ..PlayerSnapshot::default()
        };
        let down = simulate(
            base.clone(),
            &[InputState {
                move_x: 1,
                move_y: 1,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert_eq!(down.wall_slide_dir, 0);
        // min(98.000183 + 900 * DT, maxFall) = 113.00021362304688.
        assert_eq!(down.speed.y, 113.000_214);

        // The same frame with MoveY neutral does wall-slide: the 20 px/s
        // lerp target clamps the fall to 98.000183 - 900 * DT.
        let neutral = simulate(
            base,
            &[InputState {
                move_x: 1,
                move_y: 0,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert_eq!(neutral.wall_slide_dir, 1);
        assert_eq!(neutral.speed.y, 83.000_153);
    }
    #[test]
    fn leaving_the_dash_state_resets_max_fall() {
        // `Player.DashCoroutine` ends with `StateMachine.State = 0`
        // (Player.cs:4566). The StateMachine setter runs `begins[0]` =
        // `Player.NormalBegin`, which resets `maxFall = 160f`
        // (Player.cs:3531-3534), and `Player.NormalUpdate` does not run on that
        // same frame. A down-diagonal dash that reached Normal with the cap the
        // dash inherited therefore loses it before the next gravity step.
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 32.0),
            speed: Vec2::new(0.0, 169.705_63),
            state: PlayerState::Dash,
            dash_dir: Vec2::new(0.0, 1.0),
            state_timer: 0.0,
            dash_attack_timer: DASH_ATTACK_TIME,
            max_fall: FAST_MAX_FALL,
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            move_y: 1,
            ..InputState::default()
        };
        // Frame 1 arms `dash_end_pending`; frame 2 runs the DashCoroutine tail.
        let trace = simulate_trace(p, &[input; 2], &Map::default(), 2).unwrap();
        let ended = trace.states.last().unwrap();
        assert_eq!(ended.state, PlayerState::Normal);
        assert_eq!(ended.max_fall, MAX_FALL);
        assert_eq!(ended.speed.y, 169.705_63);
    }
    #[test]
    fn upward_corner_correction_moves_around_a_one_pixel_ceiling_overlap() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 40.0, 32.0, 8.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(37.0, 59.0),
            speed: Vec2::new(0.0, JUMP_SPEED),
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &map, 1).unwrap();
        assert_eq!(p.pos, Vec2::new(36.0, 58.0));
        assert!(p.speed.y < 0.0);
    }
    #[test]
    fn horizontal_dash_corner_correction_moves_over_a_two_pixel_ledge_overlap() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 80.0, 40.0, 80.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(36.0, 82.0),
            speed: Vec2::new(DASH_SPEED, 0.0),
            state: PlayerState::Dash,
            dash_dir: Vec2::new(1.0, 0.0),
            state_timer: DASH_TIME,
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &map, 1).unwrap();
        assert_eq!(p.pos, Vec2::new(37.0, 80.0));
        assert_eq!(p.speed, Vec2::new(DASH_SPEED, 0.0));
    }
    #[test]
    fn downward_dash_corner_correction_moves_left_around_a_one_pixel_floor_overlap() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 80.0, 40.0, 80.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(37.0, 80.0),
            speed: Vec2::new(0.0, DASH_SPEED),
            state: PlayerState::Dash,
            dash_dir: Vec2::new(0.0, 1.0),
            state_timer: DASH_TIME,
            ducking: true,
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &map, 1).unwrap();
        assert_eq!(p.pos, Vec2::new(36.0, 81.0));
        assert_eq!(p.speed, Vec2::new(0.0, DASH_SPEED));
    }
    #[test]
    fn downward_dash_corner_correction_follows_horizontal_speed_direction() {
        let mut map = Map {
            solids: vec![Rect::new(0.0, 80.0, 40.0, 80.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(43.0, 80.0),
            speed: Vec2::new(0.1, DASH_SPEED),
            state: PlayerState::Dash,
            dash_dir: Vec2::new(0.0, 1.0),
            state_timer: DASH_TIME,
            ducking: true,
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &map, 1).unwrap();
        assert_eq!(p.pos, Vec2::new(44.0, 81.0));
        assert_eq!(p.speed, Vec2::new(0.1, DASH_SPEED));
    }
    #[test]
    fn downward_dash_started_on_ground_does_not_corner_correct() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 80.0, 40.0, 80.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(37.0, 80.0),
            speed: Vec2::new(0.0, DASH_SPEED),
            state: PlayerState::Dash,
            dash_dir: Vec2::new(0.0, 1.0),
            state_timer: DASH_TIME,
            dash_started_on_ground: true,
            ducking: true,
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &map, 1).unwrap();
        assert_eq!(p.pos, Vec2::new(37.0, 80.0));
        assert_eq!(p.speed.y, 0.0);
    }
    #[test]
    fn dash_attack_survives_dash_end_and_breaks_a_late_feather_shield() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            entities: vec![crate::Entity {
                kind: EntityKind::FlyFeather,
                bounds: Rect::new(110.0, 110.0, 20.0, 20.0),
                direction: Vec2::default(),
                shielded: true,
                single_use: false,
                nodes: vec![],
                name: "infiniteStar".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(55.0, 120.0),
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![
            InputState {
                move_x: 1,
                ..InputState::default()
            };
            32
        ];
        inputs[0].dash_pressed = true;
        let trace = simulate_trace(p, &inputs, &map, inputs.len() as u32).unwrap();
        let dash_end = trace
            .states
            .iter()
            .enumerate()
            .skip(2)
            .find(|(_, state)| state.state == PlayerState::Normal)
            .map(|(frame, _)| frame)
            .unwrap();
        let shield_break = trace
            .states
            .iter()
            .enumerate()
            .find(|(_, state)| state.state == PlayerState::StarFly)
            .map(|(frame, _)| frame)
            .unwrap();
        assert!(shield_break > dash_end);
        assert!(trace.states[dash_end].dash_attack_timer > 0.0);
    }
    #[test]
    fn directional_spikes_only_kill_motion_into_their_points() {
        let mut map = Map {
            entities: vec![crate::Entity {
                kind: EntityKind::Spikes,
                bounds: Rect::new(40.0, 80.0, 3.0, 16.0),
                direction: Vec2::new(-1.0, 0.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "spikesLeft".to_owned(),
            }],
            ..Map::default()
        };
        let away = PlayerSnapshot {
            pos: Vec2::new(44.0, 92.0),
            speed: Vec2::new(-60.0, 0.0),
            ..PlayerSnapshot::default()
        };
        let into = PlayerSnapshot {
            pos: Vec2::new(44.0, 92.0),
            speed: Vec2::new(60.0, 0.0),
            ..PlayerSnapshot::default()
        };
        assert!(
            !simulate(away, &[InputState::default()], &map, 1)
                .unwrap()
                .dead
        );
        assert!(
            simulate(into, &[InputState::default()], &map, 1)
                .unwrap()
                .dead
        );
    }

    #[test]
    fn exit_block_awake_allows_overlap_then_closes_permanently_after_clear() {
        let source = Map {
            entities: vec![crate::Entity {
                kind: EntityKind::ExitBlock,
                bounds: Rect::new(40.0, 80.0, 16.0, 32.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "exitBlock".to_owned(),
            }],
            ..Map::default()
        };
        let mut map = source.clone();
        let mut player = PlayerSnapshot {
            pos: Vec2::new(44.0, 92.0),
            ..PlayerSnapshot::default()
        };

        initialize_exit_blocks(&mut player, &mut map);
        assert!(!player.exit_blocks[0].collidable);
        assert!(!map.non_dream_solid_at(Rect::new(44.0, 88.0, 1.0, 1.0)));

        player.pos.x = 80.0;
        advance_exit_blocks(&mut player, &mut map);
        assert!(player.exit_blocks[0].collidable);
        assert_eq!(map.entities[0].bounds, source.entities[0].bounds);
        assert!(map.non_dream_solid_at(Rect::new(44.0, 88.0, 1.0, 1.0)));

        player.pos.x = 44.0;
        advance_exit_blocks(&mut player, &mut map);
        assert!(player.exit_blocks[0].collidable);
    }

    #[test]
    fn exit_block_runtime_is_split_simulation_composable() {
        let mut map = Map {
            entities: vec![crate::Entity {
                kind: EntityKind::ExitBlock,
                bounds: Rect::new(40.0, 80.0, 16.0, 32.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "exitBlock".to_owned(),
            }],
            ..Map::default()
        };
        let inside = PlayerSnapshot {
            pos: Vec2::new(44.0, 92.0),
            ..PlayerSnapshot::default()
        };
        let mut first = Simulator::new(inside, &mut map).unwrap();
        first.step(InputState::default()).unwrap();
        let mut saved = first.into_snapshot();
        assert!(!saved.exit_blocks[0].collidable);

        saved.pos.x = 80.0;
        let closed = simulate(saved, &[InputState::default()], &map, 1).unwrap();
        assert!(closed.exit_blocks[0].collidable);

        let resumed = Simulator::new(closed, &mut map).unwrap();
        assert_eq!(resumed.runtime_entities()[0].bounds, map.entities[0].bounds);
    }

    fn invisible_barrier_map() -> Map {
        Map {
            entities: vec![crate::Entity {
                kind: EntityKind::InvisibleBarrier,
                bounds: Rect::new(40.0, 80.0, 16.0, 32.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "invisibleBarrier".to_owned(),
            }],
            ..Map::default()
        }
    }

    #[test]
    fn invisible_barrier_first_update_disables_forever_when_player_overlaps() {
        let source = invisible_barrier_map();
        let mut map = source.clone();
        let mut player = PlayerSnapshot {
            pos: Vec2::new(44.0, 92.0),
            ..PlayerSnapshot::default()
        };

        initialize_invisible_barriers(&mut player, &mut map);
        assert!(!player.invisible_barriers[0].initialized);
        assert!(!player.invisible_barriers[0].collidable);
        assert!(!map.non_dream_solid_at(Rect::new(44.0, 88.0, 1.0, 1.0)));

        advance_invisible_barriers(&mut player, &mut map);
        assert!(player.invisible_barriers[0].initialized);
        assert!(!player.invisible_barriers[0].collidable);

        player.pos.x = 80.0;
        advance_invisible_barriers(&mut player, &mut map);
        assert!(!player.invisible_barriers[0].collidable);
        assert_ne!(map.entities[0].bounds, source.entities[0].bounds);
    }

    #[test]
    fn invisible_barrier_enables_permanently_and_blocks_climbing_probes() {
        let source = invisible_barrier_map();
        let mut map = source.clone();
        let mut player = PlayerSnapshot {
            pos: Vec2::new(80.0, 92.0),
            ..PlayerSnapshot::default()
        };

        initialize_invisible_barriers(&mut player, &mut map);
        advance_invisible_barriers(&mut player, &mut map);
        assert!(player.invisible_barriers[0].initialized);
        assert!(player.invisible_barriers[0].collidable);
        assert_eq!(map.entities[0].bounds, source.entities[0].bounds);

        player.pos.x = 36.0;
        assert!(touching_wall(&player, &map, 1));
        assert!(!climb_check(&player, &map, 1, 0.0));
        assert!(!wall_jump_check(&player, &map, 1));
        // Player.cs:3751 probes the Facing side, so the same barrier that
        // blocks the climb probes also blocks the wall slide to the right.
        assert!(!wall_slide_at(&player, &map, 1));
    }

    #[test]
    fn invisible_barrier_runtime_is_split_simulation_composable() {
        let mut map = invisible_barrier_map();
        let inside = PlayerSnapshot {
            pos: Vec2::new(44.0, 92.0),
            ..PlayerSnapshot::default()
        };
        let mut first = Simulator::new(inside, &mut map).unwrap();
        first.step(InputState::default()).unwrap();
        let mut saved = first.into_snapshot();
        assert!(saved.invisible_barriers[0].initialized);
        assert!(!saved.invisible_barriers[0].collidable);

        saved.pos.x = 80.0;
        let resumed = Simulator::new(saved, &mut map).unwrap();
        assert_ne!(resumed.runtime_entities()[0].bounds, map.entities[0].bounds);
        assert!(!resumed.snapshot().invisible_barriers[0].collidable);
    }

    #[test]
    fn transition_updates_new_invisible_barrier_before_moving_player() {
        let lower = Rect::new(0.0, 0.0, 320.0, 184.0);
        let upper = Rect::new(0.0, -184.0, 320.0, 184.0);
        let barrier = crate::Entity {
            kind: EntityKind::InvisibleBarrier,
            bounds: Rect::new(152.0, -8.0, 16.0, 16.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "invisibleBarrier".to_owned(),
        };
        let mut map = Map {
            bounds: lower,
            transition_rooms: vec![upper],
            transition_runtime: vec![crate::RoomRuntime {
                bounds: upper,
                spawns: vec![Vec2::new(160.0, -24.0)],
                solids: vec![],
                entities: vec![barrier],
                load_seed: 0,
            }],
            ..Map::default()
        };
        let mut player = PlayerSnapshot {
            pos: Vec2::new(160.0, 4.0),
            speed: Vec2::new(0.0, -160.0),
            current_room_bounds: Some(lower),
            ..PlayerSnapshot::default()
        };
        let mut attachments = initialize_static_mover_attachments(&map);
        begin_transition(
            &mut player,
            &mut map,
            upper,
            Vec2::new(0.0, -1.0),
            &mut attachments,
            &mut RoomCoroutineState {
                crumble_blocks: Vec::new(),
                floaty_blocks: Vec::new(),
                switch_gates: Vec::new(),
                touch_switches: Vec::new(),
                switches_on: false,
            },
        );
        assert!(!player.invisible_barriers[0].initialized);

        step(
            &mut player,
            InputState::default(),
            &mut map,
            &mut attachments,
            &mut None,
            &mut Vec::new(),
            &mut RoomCoroutineState {
                crumble_blocks: Vec::new(),
                floaty_blocks: Vec::new(),
                switch_gates: Vec::new(),
                touch_switches: Vec::new(),
                switches_on: false,
            },
        )
        .unwrap();
        assert!(player.invisible_barriers[0].initialized);
        assert!(!player.invisible_barriers[0].collidable);
    }

    fn killbox_map() -> Map {
        Map {
            entities: vec![crate::Entity {
                kind: EntityKind::Killbox,
                bounds: Rect::new(16.0, 160.0, 288.0, 32.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "killbox".to_owned(),
            }],
            ..Map::default()
        }
    }

    #[test]
    fn killbox_uses_source_vertical_hysteresis_and_kills_on_contact() {
        let source = killbox_map();
        let mut map = source.clone();
        let mut player = PlayerSnapshot {
            pos: Vec2::new(80.0, 128.0),
            ..PlayerSnapshot::default()
        };
        initialize_killboxes(&mut player, &mut map);
        assert!(!player.killboxes[0].collidable);

        // Player.Bottom must be strictly above Top - 32.
        advance_killboxes(&mut player, &mut map);
        assert!(!player.killboxes[0].collidable);
        player.pos.y = 127.0;
        advance_killboxes(&mut player, &mut map);
        assert!(player.killboxes[0].collidable);
        assert_eq!(map.entities[0].bounds, source.entities[0].bounds);

        player.pos.y = 170.0;
        interact(&mut player, &map, InputState::default(), None);
        assert!(player.dead);
    }

    #[test]
    fn killbox_disables_after_player_falls_thirty_two_pixels_below() {
        let source = killbox_map();
        let mut map = source.clone();
        let mut player = PlayerSnapshot {
            pos: Vec2::new(80.0, 100.0),
            ..PlayerSnapshot::default()
        };
        initialize_killboxes(&mut player, &mut map);
        advance_killboxes(&mut player, &mut map);
        assert!(player.killboxes[0].collidable);

        // Player.Top must be strictly below Bottom + 32.
        player.pos.y = 235.0;
        advance_killboxes(&mut player, &mut map);
        assert!(player.killboxes[0].collidable);
        player.pos.y = 236.0;
        advance_killboxes(&mut player, &mut map);
        assert!(!player.killboxes[0].collidable);
        assert_ne!(map.entities[0].bounds, source.entities[0].bounds);
    }

    #[test]
    fn killbox_runtime_is_split_simulation_composable() {
        let mut map = killbox_map();
        let above = PlayerSnapshot {
            pos: Vec2::new(80.0, 100.0),
            ..PlayerSnapshot::default()
        };
        let mut first = Simulator::new(above, &mut map).unwrap();
        first.step(InputState::default()).unwrap();
        let saved = first.into_snapshot();
        assert!(saved.killboxes[0].collidable);

        let resumed = Simulator::new(saved, &mut map).unwrap();
        assert_eq!(resumed.runtime_entities()[0].bounds, map.entities[0].bounds);
    }

    #[test]
    fn default_dash_through_spikes_ignore_live_and_lingering_dash_attacks() {
        let mut map = Map {
            entities: vec![crate::Entity {
                kind: EntityKind::Spikes,
                bounds: Rect::new(40.0, 80.0, 3.0, 16.0),
                direction: Vec2::new(-1.0, 0.0),
                shielded: true,
                single_use: false,
                nodes: vec![],
                name: "NerdHelper/DashThroughSpikesLeft".to_owned(),
            }],
            ..Map::default()
        };
        let into = PlayerSnapshot {
            pos: Vec2::new(44.0, 92.0),
            speed: Vec2::new(60.0, 0.0),
            ..PlayerSnapshot::default()
        };

        let mut ordinary = into.clone();
        interact(&mut ordinary, &map, InputState::default(), None);
        assert!(ordinary.dead);

        let mut dash = into.clone();
        dash.state = PlayerState::Dash;
        dash.dash_dir = Vec2::new(1.0, 0.0);
        interact(&mut dash, &map, InputState::default(), None);
        assert!(!dash.dead);

        let mut lingering = into.clone();
        lingering.dash_dir = Vec2::new(1.0, 0.0);
        lingering.dash_attack_timer = 0.02;
        interact(&mut lingering, &map, InputState::default(), None);
        assert!(!lingering.dead);

        let mut zero_direction = into;
        zero_direction.state = PlayerState::Dash;
        interact(&mut zero_direction, &map, InputState::default(), None);
        assert!(zero_direction.dead);
    }
    #[test]
    fn upward_motion_flush_with_directional_spikes_applies_gravity_on_frame_one() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 512.0),
            solids: vec![Rect::new(0.0, 496.0, 960.0, 24.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::Spikes,
                bounds: Rect::new(328.0, 493.0, 96.0, 3.0),
                direction: Vec2::new(0.0, -1.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "spikesUp".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(360.0, 496.0),
            speed: Vec2::new(0.0, -60.0),
            ..PlayerSnapshot::default()
        };

        let p = simulate(p, &[InputState::default()], &map, 1).unwrap();
        assert_eq!(p.pos, Vec2::new(360.0, 495.0));
        assert_eq!(p.speed.x, 0.0);
        assert!((p.speed.y - -45.0).abs() < 0.001);
        assert!(!p.on_ground);
        assert!(!p.dead);
    }
    #[test]
    fn fastbubble_manual_dash_releases_immediately_without_spending_dash() {
        let p = PlayerSnapshot {
            pos: Vec2::new(160.0, 394.0),
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let entered = simulate(p, &[InputState::default()], &booster_map(), 1).unwrap();
        assert_eq!(entered.state, PlayerState::Boost);
        let released = simulate(
            entered,
            &[InputState {
                move_x: 1,
                dash_pressed: true,
                ..InputState::default()
            }],
            &booster_map(),
            1,
        )
        .unwrap();
        assert_eq!(released.state, PlayerState::Dash);
        assert_eq!(released.dashes, 1);
        assert!(released.state_timer > DASH_TIME);
    }
    #[test]
    fn ultradash_landing_applies_the_source_one_point_two_multiplier() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 84.0),
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![
            InputState {
                move_x: 1,
                move_y: 1,
                ..InputState::default()
            };
            12
        ];
        inputs[0].dash_pressed = true;
        let trace = simulate_trace(p, &inputs, &floor_map(), inputs.len() as u32).unwrap();
        let landed = trace
            .states
            .iter()
            .find(|state| state.on_ground && state.ducking)
            .unwrap();
        assert!((landed.speed.x - DASH_SPEED * std::f32::consts::FRAC_1_SQRT_2 * 1.2).abs() < 0.01);
        assert_eq!(landed.dash_dir, Vec2::new(1.0, 0.0));
    }
    #[test]
    fn grounded_ultra_preserves_faster_entry_speed_before_multiplier() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            speed: Vec2::new(300.0, 0.0),
            dashes: 1,
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState {
            move_x: 1,
            move_y: 1,
            ..InputState::default()
        }; 5];
        inputs[0].dash_pressed = true;
        let p = simulate(p, &inputs, &floor_map(), inputs.len() as u32).unwrap();
        assert_eq!(p.state, PlayerState::Dash);
        assert_eq!(p.speed, Vec2::new(360.0, 0.0));
        assert!(p.ducking);
    }
    #[test]
    fn delayed_ultra_lands_after_dash_state_and_still_multiplies_speed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 24.0),
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![
            InputState {
                move_x: 1,
                move_y: 1,
                ..InputState::default()
            };
            32
        ];
        inputs[0].dash_pressed = true;
        let trace = simulate_trace(p, &inputs, &floor_map(), inputs.len() as u32).unwrap();
        let dash_end = trace
            .states
            .iter()
            .enumerate()
            .skip(2)
            .find(|(_, state)| state.state == PlayerState::Normal)
            .map(|(frame, _)| frame)
            .unwrap();
        let landing = trace
            .states
            .iter()
            .enumerate()
            .skip(dash_end + 1)
            .find(|(_, state)| state.on_ground)
            .map(|(frame, state)| (frame, state))
            .unwrap();
        assert!(landing.0 > dash_end);
        let pre_landing = &trace.states[landing.0 - 1];
        let expected = approach(pre_landing.speed.x, MAX_RUN, RUN_REDUCE * AIR_MULT * DT) * 1.2;
        assert!((landing.1.speed.x - expected).abs() < 0.01);
        assert!(landing.1.speed.x > pre_landing.speed.x);
        assert_eq!(landing.1.dash_dir, Vec2::new(1.0, 0.0));
    }
    #[test]
    fn chained_ultras_compound_two_landing_multipliers() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 65.0),
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![
            InputState {
                move_x: 1,
                move_y: 1,
                ..InputState::default()
            };
            36
        ];
        inputs[0].dash_pressed = true;
        inputs[16].dash_pressed = true;
        let trace = simulate_trace(p, &inputs, &floor_map(), inputs.len() as u32).unwrap();
        let landings: Vec<_> = trace
            .states
            .windows(2)
            .enumerate()
            .filter(|(_, pair)| {
                pair[1].on_ground && pair[1].ducking && pair[1].speed.x > pair[0].speed.x
            })
            .map(|(frame, pair)| (frame + 1, &pair[1]))
            .collect();
        assert_eq!(landings.len(), 2);
        assert_eq!(landings[0].0, 17);
        assert_eq!(landings[1].0, 22);
        assert!(landings[1].1.speed.x > landings[0].1.speed.x);
        assert!(landings[1].1.speed.x > 230.0);
    }
    #[test]
    fn demodash_passes_a_six_pixel_gap_that_blocks_a_normal_dash() {
        let mut map = Map {
            solids: vec![
                Rect::new(0.0, 100.0, 160.0, 80.0),
                Rect::new(40.0, 0.0, 80.0, 94.0),
            ],
            ..Map::default()
        };
        let start = PlayerSnapshot {
            pos: Vec2::new(24.0, 100.0),
            dashes: 1,
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let mut demo_inputs = vec![
            InputState {
                move_x: 1,
                ..InputState::default()
            };
            20
        ];
        demo_inputs[0].crouch_dash_pressed = true;
        let mut normal_inputs = demo_inputs.clone();
        normal_inputs[0].crouch_dash_pressed = false;
        normal_inputs[0].dash_pressed = true;
        let demo = simulate(start.clone(), &demo_inputs, &map, 20).unwrap();
        let normal = simulate(start, &normal_inputs, &map, 20).unwrap();
        assert!(demo.pos.x > 70.0);
        assert_eq!(normal.pos.x, 36.0);
        assert!(demo.demo_dashed);
    }

    #[test]
    fn fastfall_limit_stays_normal_until_downward_speed_reaches_160() {
        let player = PlayerSnapshot {
            pos: Vec2::new(32.0, 32.0),
            speed: Vec2::new(0.0, 150.0),
            max_fall: MAX_FALL,
            ..PlayerSnapshot::default()
        };
        let player = simulate(
            player,
            &[InputState {
                move_y: 1,
                ..InputState::default()
            }],
            &Map::default(),
            1,
        )
        .unwrap();
        assert_eq!(player.max_fall, MAX_FALL);
        assert_eq!(player.speed.y, MAX_FALL);
    }

    #[test]
    fn neutral_climb_jump_converts_to_wallboost_and_refunds_stamina() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 0.0, 8.0, 100.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(36.0, 64.0),
            state: PlayerState::Climb,
            facing: true,
            stamina: 80.0,
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState {
                jump_pressed: true,
                jump_held: true,
                grab_held: true,
                ..InputState::default()
            },
            InputState {
                move_x: -1,
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                jump_held: true,
                ..InputState::default()
            },
        ];
        let trace = simulate_trace(p, &inputs, &map, 3).unwrap();
        assert_eq!(trace.states[1].stamina, 52.5);
        assert_eq!(trace.states[1].wall_boost_dir, -1);
        assert!(trace.states[1].wall_boost_timer > 0.19);
        assert_eq!(trace.states[2].stamina, 52.5);
        assert_eq!(trace.states[3].stamina, 80.0);
        assert_eq!(trace.states[3].wall_boost_timer, 0.0);
        assert!(trace.states[3].speed.x < -110.0);
    }

    #[test]
    fn half_stamina_climbing_chains_wallboost_into_close_wall_climb_jump() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![Rect::new(40.0, 0.0, 8.0, 184.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 120.0),
            state: PlayerState::Climb,
            facing: true,
            stamina: 80.0,
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState {
                jump_pressed: true,
                jump_held: true,
                grab_held: true,
                ..InputState::default()
            },
            InputState {
                move_x: -1,
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                jump_pressed: true,
                jump_held: true,
                grab_held: true,
                ..InputState::default()
            },
        ];
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();

        assert_eq!(trace.states[1].stamina, 52.5);
        assert!(trace.states[1].wall_boost_timer > 0.19);
        assert_eq!(trace.states[2].stamina, 52.5);
        assert_eq!(trace.states[3].stamina, 52.5);
        assert_eq!(trace.states[3].wall_boost_timer, 0.0);
        assert!((trace.states[3].speed.x + 79.166_64).abs() < 0.000_1);
        assert_eq!(trace.states[3].speed.y, JUMP_SPEED);
        assert_eq!(trace.states[3].state, PlayerState::Normal);
        assert!(trace.states[3].facing);
        assert_eq!(trace.states[3].dashes, 1);
        assert!(!trace.states[3].on_ground);
        assert!(!trace.states[3].ducking);
        assert!(!trace.states[3].dead);
    }

    #[test]
    fn neutral_wall_jumps_return_for_a_second_stamina_free_jump() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 300.0),
            solids: vec![Rect::new(40.0, 0.0, 8.0, 300.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(34.0, 240.0),
            speed: Vec2::new(0.0, 30.0),
            facing: true,
            stamina: 80.0,
            ..PlayerSnapshot::default()
        };
        let first_cycle = std::array::from_fn::<_, 60, _>(|frame| InputState {
            move_x: if frame >= 1 { 1 } else { 0 },
            jump_pressed: frame == 0,
            jump_held: frame < 10,
            ..InputState::default()
        });
        let first_trace = simulate_trace(player.clone(), &first_cycle, &map, 60).unwrap();
        let second_jump_frame = (10..60)
            .find(|&frame| {
                first_trace.states[frame].speed.x >= 0.0
                    && wall_jump_check(&first_trace.states[frame], &map, 1)
            })
            .expect("neutral air control returns within wall-jump range");
        assert_eq!(second_jump_frame, 26);
        let inputs = std::array::from_fn::<_, 60, _>(|frame| InputState {
            move_x: if frame == 0 || frame == second_jump_frame {
                0
            } else {
                1
            },
            jump_pressed: frame == 0 || frame == second_jump_frame,
            jump_held: frame < 10 || (second_jump_frame..second_jump_frame + 10).contains(&frame),
            ..InputState::default()
        });
        let trace = simulate_trace(player, &inputs, &map, 60).unwrap();
        for jump_state in [1, second_jump_frame + 1] {
            assert_eq!(trace.states[jump_state].speed.x, -WALL_JUMP_H);
            assert_eq!(trace.states[jump_state].speed.y, JUMP_SPEED);
            assert_eq!(trace.states[jump_state].state, PlayerState::Normal);
            assert!(trace.states[jump_state].facing);
            assert_eq!(trace.states[jump_state].dashes, 1);
            assert_eq!(trace.states[jump_state].stamina, 80.0);
            assert!(!trace.states[jump_state].on_ground);
            assert!(!trace.states[jump_state].ducking);
            assert!(!trace.states[jump_state].dead);
            assert_eq!(trace.states[jump_state].force_move_x_timer, 0.0);
        }
        assert!(trace.states[second_jump_frame + 1].pos.y < trace.states[1].pos.y);
    }

    #[test]
    fn cornerkick_uses_the_three_pixel_probe_on_the_last_corner_pixel() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            solids: vec![Rect::new(40.0, 0.0, 8.0, 40.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(34.0, 50.0),
            speed: Vec2::new(0.0, -30.0),
            facing: true,
            stamina: 80.0,
            ..PlayerSnapshot::default()
        };
        assert!(!touching_wall(&player, &map, 1));
        assert!(!climb_check(&player, &map, 1, 0.0));
        assert!(wall_jump_check(&player, &map, 1));

        let directional = simulate(
            player.clone(),
            &[InputState {
                move_x: 1,
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert_eq!(directional.speed, Vec2::new(-WALL_JUMP_H, JUMP_SPEED));
        assert_eq!(directional.force_move_x, -1);
        assert_eq!(directional.force_move_x_timer, 0.16);
        assert_eq!(directional.stamina, 80.0);

        let neutral = simulate(
            player.clone(),
            &[InputState {
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert_eq!(neutral.speed, Vec2::new(-WALL_JUMP_H, JUMP_SPEED));
        assert_eq!(neutral.force_move_x_timer, 0.0);

        let too_low = PlayerSnapshot {
            pos: Vec2::new(34.0, 51.0),
            ..player
        };
        assert!(!wall_jump_check(&too_low, &map, 1));
        let too_low = simulate(
            too_low,
            &[InputState {
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert_ne!(too_low.speed.y, JUMP_SPEED);
        assert!(too_low.jump_buffer_timer > 0.0);
    }

    #[test]
    fn ceiling_pop_climb_jumps_before_the_lost_wall_check() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            solids: vec![Rect::new(40.0, 0.0, 40.0, 40.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 38.0),
            speed: Vec2::new(0.0, 30.0),
            facing: true,
            stamina: 80.0,
            max_fall: FAST_MAX_FALL,
            ..PlayerSnapshot::default()
        };
        let descend = [InputState {
            move_y: 1,
            grab_held: true,
            ..InputState::default()
        }; 30];
        let descend_trace = simulate_trace(player.clone(), &descend, &map, 30).unwrap();
        let pop_frame = (1..descend_trace.states.len())
            .find(|&frame| {
                descend_trace.states[frame].state == PlayerState::Climb
                    && !touching_wall(&descend_trace.states[frame], &map, 1)
            })
            .expect("downward climb reaches the one-frame lost-wall window");
        assert_eq!(pop_frame, 18);
        assert_eq!(
            descend_trace.states[pop_frame + 1].state,
            PlayerState::Normal
        );

        let inputs = std::array::from_fn::<_, 30, _>(|frame| InputState {
            move_x: if frame == pop_frame { 1 } else { 0 },
            move_y: 1,
            grab_held: true,
            jump_pressed: frame == pop_frame,
            jump_held: frame == pop_frame,
            ..InputState::default()
        });
        let trace = simulate_trace(player, &inputs, &map, 30).unwrap();
        let popped = &trace.states[pop_frame + 1];
        assert_eq!(popped.state, PlayerState::Normal);
        assert_eq!(popped.max_fall, MAX_FALL);
        assert_eq!(popped.stamina, 52.5);
        assert!(popped.pos.x > descend_trace.states[pop_frame + 1].pos.x);
        assert_eq!(popped.speed.x, JUMP_H_BOOST);
        assert_eq!(popped.speed.y, 0.0);
        assert!(!popped.on_ground);
        assert!(!popped.ducking);
        assert!(!popped.dead);
    }

    #[test]
    fn wallboost_neutral_returns_to_the_wall_for_a_second_stamina_free_cycle() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 300.0),
            solids: vec![Rect::new(40.0, 0.0, 8.0, 300.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 240.0),
            state: PlayerState::Climb,
            facing: true,
            stamina: 80.0,
            ..PlayerSnapshot::default()
        };
        let inputs = std::array::from_fn::<_, 60, _>(|frame| InputState {
            move_x: match frame {
                1 | 28 => -1,
                2..=26 | 29.. => 1,
                _ => 0,
            },
            jump_pressed: frame == 0 || frame == 27,
            jump_held: frame < 10 || (27..37).contains(&frame),
            grab_held: frame == 0 || frame >= 20,
            ..InputState::default()
        });
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();
        assert_eq!(trace.states[1].stamina, 52.5);
        assert_eq!(trace.states[3].stamina, 80.0);
        assert_eq!(trace.states[27].state, PlayerState::Climb);
        assert_eq!(trace.states[27].stamina, 80.0);
        assert_eq!(trace.states[28].state, PlayerState::Normal);
        assert_eq!(trace.states[28].stamina, 52.5);
        assert_eq!(trace.states[30].wall_boost_timer, 0.0);
        assert_eq!(trace.states[30].stamina, 80.0);
        assert!(trace.states[30].speed.x < -110.0);
    }

    #[test]
    fn climb_begin_at_a_ledge_uses_slip_speed_during_the_no_move_window() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 300.0),
            solids: vec![Rect::new(40.0, 100.0, 8.0, 200.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 106.0),
            speed: Vec2::new(0.0, 24.0),
            state: PlayerState::Climb,
            facing: true,
            climb_no_move_timer: 0.1,
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            grab_held: true,
            ..InputState::default()
        };
        let player = simulate(player, &[input], &map, 1).unwrap();
        assert_eq!(player.state, PlayerState::Climb);
        assert_eq!(player.speed.y, CLIMB_SLIP_SPEED);
    }

    #[test]
    fn climbhop_waits_for_the_body_to_clear_the_ledge_before_horizontal_launch() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 80.0, 8.0, 100.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 84.0),
            speed: Vec2::new(0.0, -45.0),
            state: PlayerState::Climb,
            facing: true,
            stamina: 100.0,
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            move_x: 1,
            move_y: -1,
            grab_held: true,
            ..InputState::default()
        };
        let trace = simulate_trace(player, &[input; 4], &map, 4).unwrap();
        assert_eq!(trace.states[1].state, PlayerState::Normal);
        assert_eq!(trace.states[1].speed.y, CLIMB_HOP_Y);
        assert_eq!(trace.states[1].speed.x, 0.0);
        assert_eq!(trace.states[1].hop_wait_x, 1);
        assert_eq!(trace.states[1].force_move_x_timer, CLIMB_HOP_FORCE_TIME);
        assert_eq!(trace.states[1].no_wind_timer, CLIMB_HOP_NO_WIND_TIME);
        assert!((trace.states[3].speed.x - 89.166_64).abs() < 0.001);
        assert_eq!(trace.states[3].hop_wait_x, 0);
    }

    #[test]
    fn climb_hop_stores_the_grabbed_solid_for_the_carry() {
        // `Player.ClimbHop` (`Player.cs:4124`) stores
        // `CollideFirst<Solid>(Position + UnitX * Facing)`. The wall here is a
        // moving Solid placed so that the player is beside its face but the
        // `SlipCheck` probes at `Collider.Right` miss it (`Player.cs:4134`), so
        // ClimbUpdate takes the slip branch at `Player.cs:4155`.
        // `SlipCheck` probes at `Collider.Right`, so the Solid is a 2 px slab
        // that only covers the middle of the player's body. It is held still
        // here: this test pins the capture, and
        // `climb_hop_solid_carry_uses_the_solids_whole_pixel_delta` pins the
        // per-frame carry.
        let mut map = Map {
            entities: vec![crate::Entity {
                kind: EntityKind::MovingSolid,
                bounds: Rect::new(40.0, 114.0, 24.0, 2.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "celesteGymMovingSolid".to_owned(),
            }],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 124.0),
            speed: Vec2::new(0.0, -45.0),
            state: PlayerState::Climb,
            facing: true,
            stamina: 100.0,
            ..PlayerSnapshot::default()
        };
        assert!(touching_wall(&player, &map, 1));
        assert!(slip_check(&player, &map, 0.0));
        let mut simulator = Simulator::new(player, &mut map).unwrap();
        simulator
            .step(InputState {
                move_y: -1,
                grab_held: true,
                ..InputState::default()
            })
            .unwrap();
        assert_eq!(simulator.snapshot().state, PlayerState::Normal);
        assert_eq!(simulator.snapshot().hop_wait_x, 1);
        assert_eq!(
            simulator.climb_hop_solid,
            Some(ClimbHopSolid {
                entity: 0,
                position: Vec2::new(40.0, 114.0),
            })
        );
    }

    #[test]
    fn climb_hop_solid_carry_uses_the_solids_whole_pixel_delta() {
        // `Player.cs:1646-1652`: while the force-move window is open, the
        // grabbed Solid's whole-pixel movement is replayed onto the player with
        // `MoveHExact`/`MoveVExact`, which never touch `movementCounter` unless
        // a pixel is blocked (`Actor.cs:220`/`249`).
        let mut map = Map {
            entities: vec![crate::Entity {
                kind: EntityKind::MovingSolid,
                bounds: Rect::new(60.0, 100.0, 16.0, 16.0),
                direction: Vec2::new(0.0, -60.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "celesteGymMovingSolid".to_owned(),
            }],
            ..Map::default()
        };
        let mut p = PlayerSnapshot {
            // Frozen keeps the state callback inert so the carry is the only
            // position write in the frame.
            pos: Vec2::new(36.0, 116.0),
            state: PlayerState::Frozen,
            force_move_x_timer: CLIMB_HOP_FORCE_TIME,
            movement_remainder: Vec2::new(0.0, 0.25),
            ..PlayerSnapshot::default()
        };
        let mut attachments = Vec::new();
        let mut carry = Some(ClimbHopSolid {
            entity: 0,
            position: Vec2::new(60.0, 100.0),
        });
        step(
            &mut p,
            InputState::default(),
            &mut map,
            &mut attachments,
            &mut carry,
            &mut Vec::new(),
            &mut RoomCoroutineState {
                crumble_blocks: Vec::new(),
                floaty_blocks: Vec::new(),
                switch_gates: Vec::new(),
                touch_switches: Vec::new(),
                switches_on: false,
            },
        )
        .unwrap();
        // -60 px/s over one 1/60 second frame is exactly one whole pixel up.
        assert_eq!(p.pos, Vec2::new(36.0, 115.0));
        assert_eq!(p.movement_remainder, Vec2::new(0.0, 0.25));
        assert_eq!(
            carry,
            Some(ClimbHopSolid {
                entity: 0,
                position: Vec2::new(60.0, 99.0),
            })
        );

        // Once the force-move window closes, `Player.cs:1640` drops the
        // reference and the Solid can no longer carry the player.
        p.force_move_x_timer = 0.0;
        step(
            &mut p,
            InputState::default(),
            &mut map,
            &mut attachments,
            &mut carry,
            &mut Vec::new(),
            &mut RoomCoroutineState {
                crumble_blocks: Vec::new(),
                floaty_blocks: Vec::new(),
                switch_gates: Vec::new(),
                touch_switches: Vec::new(),
                switches_on: false,
            },
        )
        .unwrap();
        assert_eq!(carry, None);
        assert_eq!(p.pos, Vec2::new(36.0, 115.0));
    }

    /// `Player.JumpThruBoostBlockedCheck` (`Player.cs:4179-4189`) and the
    /// `LedgeBlocker` set it iterates (`Celeste/LedgeBlocker.cs:8`).
    fn jump_thru_assist_map(blocker: Option<crate::Entity>) -> Map {
        let mut entities = vec![crate::Entity {
            kind: EntityKind::JumpThru,
            bounds: Rect::new(32.0, 95.0, 32.0, 5.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "jumpThru".to_owned(),
        }];
        entities.extend(blocker);
        Map {
            entities,
            ..Map::default()
        }
    }

    #[test]
    fn jump_thru_assist_runs_without_a_ledge_blocker() {
        let mut map = jump_thru_assist_map(None);
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 100.0),
            state: PlayerState::Frozen,
            ..PlayerSnapshot::default()
        };
        let after = simulate(player, &[InputState::default()], &map, 1).unwrap();
        // MoveV(-40 * 1/60) rounds to a whole pixel up and keeps the fraction.
        assert_eq!(after.pos.y, 99.0);
        assert!((after.movement_remainder.y - 1.0 / 3.0).abs() < 0.0001);
    }

    #[test]
    fn jump_thru_assist_is_skipped_when_a_ledge_blocker_blocks_the_probe() {
        // `Spikes.cs:61` gives a right-facing spike strip a `LedgeBlocker`, and
        // `LedgeBlocker.cs:33-44` probes two pixels above the live collider.
        let mut map = jump_thru_assist_map(Some(crate::Entity {
            kind: EntityKind::Spikes,
            bounds: Rect::new(38.0, 90.0, 3.0, 8.0),
            direction: Vec2::new(1.0, 0.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "spikesRight".to_owned(),
        }));
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 100.0),
            state: PlayerState::Frozen,
            ..PlayerSnapshot::default()
        };
        let after = simulate(player, &[InputState::default()], &map, 1).unwrap();
        assert_eq!(after.pos.y, 100.0);
        assert_eq!(after.movement_remainder.y, 0.0);
    }

    #[test]
    fn ledge_blocker_probe_uses_the_spinner_circle_not_its_bounding_box() {
        // `CrystalStaticSpinner.cs:152` is
        // `ColliderList(Circle(6f), Hitbox(16f, 4f, -8f, -3f))`; the decoded
        // bounds are only the union's bounding box, so a probe that touches a
        // rounded corner must not register.
        let spinner = crate::Entity {
            kind: EntityKind::CrystalStaticSpinner,
            bounds: Rect::new(40.0, 100.0, 16.0, 12.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "spinner".to_owned(),
        };
        // Bounding-box corner only: nearest point to the (48, 106) centre is
        // (54, 110), distance sqrt(52) > 6, and the 4 px hitbox is at
        // y 103..107.
        assert!(!ledge_blocker_collides(&spinner, Rect::new(54.0, 110.0, 4.0, 2.0)));
        // Inside the 16x4 hitbox.
        assert!(ledge_blocker_collides(&spinner, Rect::new(44.0, 104.0, 2.0, 2.0)));
        // Inside the circle.
        assert!(ledge_blocker_collides(&spinner, Rect::new(48.0, 101.0, 2.0, 2.0)));
        // Down-facing spikes carry no LedgeBlocker (`Spikes.cs:52-54`).
        let down_spikes = crate::Entity {
            kind: EntityKind::Spikes,
            direction: Vec2::new(0.0, 1.0),
            ..spinner
        };
        assert!(!ledge_blocker_collides(&down_spikes, Rect::new(44.0, 104.0, 2.0, 2.0)));
    }

    #[test]
    fn climb_jump_keeps_priority_over_climbhop_on_the_lost_wall_frame() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 0.0, 8.0, 40.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 51.0),
            speed: Vec2::new(0.0, -45.0),
            state: PlayerState::Climb,
            facing: true,
            stamina: 80.0,
            ..PlayerSnapshot::default()
        };
        assert!(!touching_wall(&player, &map, 1));
        let player = simulate(
            player,
            &[InputState {
                grab_held: true,
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert_eq!(player.state, PlayerState::Normal);
        assert_eq!(player.speed, Vec2::new(0.0, JUMP_SPEED));
        assert_eq!(player.stamina, 52.5);
        assert_eq!(player.wall_boost_timer, 0.2);
        assert_eq!(player.hop_wait_x, 0);
        assert_eq!(player.hop_wait_x_speed, 0.0);
    }

    #[test]
    fn grounded_wall_grab_can_start_climbing_at_wall_root() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 220.0),
            solids: vec![
                Rect::new(40.0, 80.0, 8.0, 140.0),
                Rect::new(0.0, 160.0, 320.0, 60.0),
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 160.0),
            speed: Vec2::default(),
            facing: true,
            stamina: 110.0,
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            move_y: -1,
            grab_held: true,
            ..InputState::default()
        };
        let trace = simulate_trace(player, &[input; 20], &map, 20).unwrap();

        assert_eq!(trace.states[1].state, PlayerState::Climb);
        assert!(trace.states[19].state == PlayerState::Climb);
        assert!(trace.states[19].pos.y < 160.0);
    }

    #[test]
    fn blocked_climbhop_keeps_grabbing_the_wall_at_the_top() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 220.0),
            solids: vec![
                Rect::new(40.0, 80.0, 8.0, 140.0),
                Rect::new(32.0, 64.0, 48.0, 9.0),
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 84.0),
            speed: Vec2::new(0.0, -45.0),
            state: PlayerState::Climb,
            facing: true,
            stamina: 110.0,
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            move_y: -1,
            grab_held: true,
            ..InputState::default()
        };
        let player = simulate(player, &[input], &map, 1).unwrap();

        assert_eq!(player.state, PlayerState::Climb);
        assert!(player.speed.y >= 0.0);
    }

    #[test]
    fn stamina_cancel_regrabs_to_reset_the_no_move_cost_window() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 0.0, 8.0, 100.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 64.0),
            speed: Vec2::new(0.0, 30.0),
            facing: true,
            ..PlayerSnapshot::default()
        };
        let held = [InputState {
            move_y: -1,
            grab_held: true,
            ..InputState::default()
        }; 30];
        let cancelled = std::array::from_fn::<_, 30, _>(|frame| InputState {
            move_y: -1,
            grab_held: frame < 8 || frame >= 11,
            ..InputState::default()
        });
        let held_trace = simulate_trace(player.clone(), &held, &map, 30).unwrap();
        let cancelled_trace = simulate_trace(player, &cancelled, &map, 30).unwrap();
        assert_eq!(cancelled_trace.states[9].state, PlayerState::Normal);
        assert_eq!(cancelled_trace.states[12].state, PlayerState::Climb);
        assert!(cancelled_trace.states[12].climb_no_move_timer > 0.09);
        assert_eq!(
            cancelled_trace.states[12].stamina,
            cancelled_trace.states[17].stamina
        );
        assert!(held_trace.states[17].stamina < cancelled_trace.states[17].stamina);
    }

    #[test]
    fn climbing_down_does_not_pay_the_stationary_stamina_cost() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 0.0, 8.0, 100.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(36.0, 64.0),
            state: PlayerState::Climb,
            facing: true,
            stamina: 80.0,
            ..PlayerSnapshot::default()
        };
        let descending = simulate(
            player.clone(),
            &[InputState {
                move_y: 1,
                grab_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert_eq!(descending.state, PlayerState::Climb);
        assert!(descending.speed.y > 0.0);
        assert_eq!(descending.stamina, 80.0);

        let stationary = simulate(
            player,
            &[InputState {
                grab_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert_eq!(stationary.state, PlayerState::Climb);
        assert!(stationary.stamina < 80.0);
    }

    #[test]
    fn cornerboost_restores_retained_speed_after_clearing_wall_top() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 40.0, 8.0, 60.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(36.0, 44.0),
            speed: Vec2::new(120.0, -120.0),
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(
            p,
            &[InputState {
                move_x: 1,
                jump_held: true,
                ..InputState::default()
            }; 4],
            &map,
            4,
        )
        .unwrap();
        assert_eq!(trace.states[1].speed.x, 0.0);
        assert!((trace.states[1].wall_speed_retained - 115.666_66).abs() < 0.001);
        assert!(trace.states[1].wall_speed_retention_timer > 0.05);
        assert_eq!(trace.states[4].wall_speed_retention_timer, 0.0);
        assert!(trace.states[4].speed.x > 105.0);
    }

    #[test]
    fn cornerboost_climb_jump_stores_jump_boost_before_clearing_wall_top() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 40.0, 8.0, 60.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(35.0, 46.0),
            speed: Vec2::new(90.0, -30.0),
            facing: true,
            stamina: 110.0,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(
            player,
            &[
                InputState {
                    move_x: 1,
                    jump_pressed: true,
                    jump_held: true,
                    grab_held: true,
                    ..InputState::default()
                },
                InputState {
                    move_x: 1,
                    jump_held: true,
                    ..InputState::default()
                },
                InputState {
                    move_x: 1,
                    jump_held: true,
                    ..InputState::default()
                },
                InputState {
                    move_x: 1,
                    jump_held: true,
                    ..InputState::default()
                },
                InputState {
                    move_x: 1,
                    jump_held: true,
                    ..InputState::default()
                },
            ],
            &map,
            5,
        )
        .unwrap();
        assert_eq!(trace.states[1].state, PlayerState::Normal);
        assert_eq!(trace.states[1].speed.x, 0.0);
        assert!((trace.states[1].wall_speed_retained - 130.0).abs() < 0.001);
        assert_eq!(trace.states[1].stamina, 82.5);
        assert_eq!(trace.states[5].wall_speed_retention_timer, 0.0);
        assert!((trace.states[5].speed.x - 125.666_64).abs() < 0.001);
    }

    #[test]
    fn downward_cornerboost_uses_wall_jump_probe_without_entering_climb() {
        let mut map = Map {
            solids: vec![Rect::new(40.0, 40.0, 8.0, 60.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(34.0, 46.0),
            speed: Vec2::new(160.0, 30.0),
            facing: true,
            stamina: 110.0,
            ..PlayerSnapshot::default()
        };
        assert!(!climb_check(&player, &map, 1, 0.0));
        assert!(wall_jump_check(&player, &map, 1));
        let trace = simulate_trace(
            player,
            &[
                InputState {
                    move_x: 1,
                    jump_pressed: true,
                    jump_held: true,
                    grab_held: true,
                    ..InputState::default()
                },
                InputState {
                    move_x: 1,
                    jump_held: true,
                    ..InputState::default()
                },
                InputState {
                    move_x: 1,
                    jump_held: true,
                    ..InputState::default()
                },
                InputState {
                    move_x: 1,
                    jump_held: true,
                    ..InputState::default()
                },
                InputState {
                    move_x: 1,
                    jump_held: true,
                    ..InputState::default()
                },
            ],
            &map,
            5,
        )
        .unwrap();
        assert_eq!(trace.states[1].state, PlayerState::Normal);
        assert_eq!(trace.states[1].speed.x, 0.0);
        assert!((trace.states[1].wall_speed_retained - 195.666_66).abs() < 0.001);
        assert_eq!(trace.states[1].stamina, 82.5);
        assert!(trace.states[5].speed.x > 190.0);
    }

    #[test]
    fn five_jump_chains_neutral_and_lip_climb_jumps_across_five_tiles() {
        let mut map = Map {
            solids: vec![
                Rect::new(0.0, 40.0, 40.0, 80.0),
                Rect::new(80.0, 40.0, 40.0, 8.0),
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(44.0, 52.0),
            state: PlayerState::Climb,
            facing: false,
            stamina: 110.0,
            ..PlayerSnapshot::default()
        };
        let inputs = (0..48)
            .map(|frame| InputState {
                move_x: if frame >= 6 { 1 } else { 0 },
                jump_pressed: frame == 0 || frame == 5,
                jump_held: frame <= 17,
                grab_held: frame == 0 || frame == 5,
                ..InputState::default()
            })
            .collect::<Vec<_>>();
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();
        assert_eq!(trace.states[1].speed.y, JUMP_SPEED);
        assert_eq!(trace.states[6].stamina, 55.0);
        assert!(trace.states.iter().any(|state| state.pos.x >= 76.0));
        assert!(
            trace
                .states
                .iter()
                .any(|state| state.on_ground && state.pos.x >= 76.0)
        );
    }

    #[test]
    fn six_jump_uses_a_full_speed_cornerboost_to_reach_six_tile_landing() {
        let mut map = Map {
            solids: vec![
                Rect::new(40.0, 40.0, 8.0, 80.0),
                Rect::new(88.0, 48.0, 40.0, 8.0),
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(35.0, 46.0),
            speed: Vec2::new(90.0, -30.0),
            facing: true,
            stamina: 110.0,
            ..PlayerSnapshot::default()
        };
        let inputs = (0..60)
            .map(|frame| InputState {
                move_x: 1,
                jump_pressed: frame == 0,
                jump_held: frame < 13,
                grab_held: frame == 0,
                ..InputState::default()
            })
            .collect::<Vec<_>>();
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();
        assert!((trace.states[1].wall_speed_retained - 130.0).abs() < 0.001);
        assert!(trace.states.iter().any(|state| state.pos.x >= 84.0));
        assert!(
            trace
                .states
                .iter()
                .any(|state| state.on_ground && state.pos.x >= 84.0)
        );
    }

    #[test]
    fn double_cornerboost_uses_two_consecutive_climb_jumps_from_a_grounded_setup() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![
                Rect::new(80.0, 152.0, 128.0, 32.0),
                Rect::new(144.0, 80.0, 8.0, 72.0),
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(120.0, 152.0),
            state: PlayerState::Normal,
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs = (0..90)
            .map(|frame| InputState {
                move_x: if frame <= 20 || frame >= 78 {
                    1
                } else if (75..=77).contains(&frame) {
                    -1
                } else {
                    0
                },
                move_y: if (21..=74).contains(&frame) { -1 } else { 0 },
                jump_pressed: frame == 0 || frame == 79 || frame == 80,
                jump_held: frame < 12 || frame == 79 || frame == 80,
                grab_held: frame <= 74 || frame == 79 || frame == 80,
                ..InputState::default()
            })
            .collect::<Vec<_>>();
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();

        assert_eq!(trace.states[20].state, PlayerState::Climb);
        assert_eq!(trace.states[79].pos, Vec2::new(139.0, 87.0));
        assert!((trace.states[79].stamina - 72.1212).abs() < 0.001);
        assert!((trace.states[79].stamina - trace.states[80].stamina - 27.5).abs() < 0.001);
        assert!((trace.states[80].stamina - trace.states[81].stamina - 27.5).abs() < 0.001);
        assert_eq!(trace.states[80].speed.x, JUMP_H_BOOST);
        assert!((trace.states[81].wall_speed_retained - 90.83336).abs() < 0.001);
        assert_eq!(trace.states[81].wall_speed_retention_timer, 0.06);
        assert_eq!(trace.states[85].wall_speed_retention_timer, 0.0);
        assert!((trace.states[85].speed.x - 90.0).abs() < 0.001);
        assert!(trace.states[85].pos.x > 140.0);
    }

    #[test]
    fn seven_jump_lands_on_a_target_seven_tiles_from_the_double_cornerboost_wall() {
        let wall_x = 80.0;
        let target_x = wall_x + 7.0 * 8.0;
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![
                Rect::new(0.0, 120.0, wall_x, 64.0),
                Rect::new(wall_x, 112.0, 8.0, 8.0),
                Rect::new(target_x, 120.0, 80.0, 8.0),
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(8.0, 120.0),
            state: PlayerState::Normal,
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs = (0..120)
            .map(|frame| InputState {
                move_x: 1,
                jump_pressed: frame == 11 || frame == 44 || frame == 45,
                jump_held: (11..23).contains(&frame) || (44..58).contains(&frame),
                grab_held: frame == 44 || frame == 45,
                ..InputState::default()
            })
            .collect::<Vec<_>>();
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();

        assert_eq!(trace.states[44].pos, Vec2::new(74.0, 118.0));
        assert_eq!(trace.states[44].speed.x, MAX_RUN);
        assert_eq!(trace.states[45].stamina, 82.5);
        assert_eq!(trace.states[46].stamina, 55.0);
        assert!((trace.states[46].wall_speed_retained - 165.66666).abs() < 0.001);
        assert!(trace.states[49].speed.x > 160.0);
        assert!(
            trace
                .states
                .iter()
                .any(|state| { state.on_ground && state.pos.x >= target_x - 4.0 })
        );
        assert_eq!(trace.states[80].pos, Vec2::new(134.0, 120.0));
        assert!(trace.states[80].on_ground);
    }

    #[test]
    fn eight_jump_lands_on_a_target_eight_tiles_from_the_cornerboost_wall() {
        let wall_x = 80.0;
        let target_x = wall_x + 8.0 * 8.0;
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![
                Rect::new(0.0, 120.0, wall_x, 64.0),
                Rect::new(wall_x, 104.0, 8.0, 16.0),
                Rect::new(target_x, 112.0, 80.0, 8.0),
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(58.0, 120.0),
            state: PlayerState::Normal,
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs = (0..120)
            .map(|frame| InputState {
                move_x: 1,
                jump_pressed: frame == 5 || frame == 11 || frame == 12 || frame == 13,
                jump_held: frame <= 26,
                grab_held: frame == 11 || frame == 12 || frame == 13,
                ..InputState::default()
            })
            .collect::<Vec<_>>();
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();

        assert_eq!(trace.states[11].pos, Vec2::new(74.0, 109.0));
        assert_eq!(trace.states[12].stamina, 82.5);
        assert_eq!(trace.states[13].stamina, 55.0);
        assert_eq!(trace.states[14].stamina, 27.5);
        assert!((trace.states[14].wall_speed_retained - 179.6666).abs() < 0.001);
        assert_eq!(trace.states[49].pos, Vec2::new(143.0, 112.0));
        assert!(trace.states[49].on_ground);
        assert_eq!(target_x - wall_x, 64.0);
        assert!(trace.states[49].pos.x >= target_x - 4.0);
    }

    #[test]
    fn nine_jump_lands_nine_tiles_away_only_with_the_favorable_timing() {
        let wall_x = 80.0;
        let target_x = wall_x + 9.0 * 8.0;
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![
                Rect::new(0.0, 120.0, wall_x, 64.0),
                Rect::new(wall_x, 112.0, 8.0, 8.0),
                Rect::new(target_x, 120.0, 80.0, 8.0),
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(67.0, 120.0),
            state: PlayerState::Normal,
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs = (0..120)
            .map(|frame| InputState {
                move_x: 1,
                jump_pressed: frame == 4 || frame == 6 || frame == 7 || frame == 8,
                jump_held: frame <= 21,
                grab_held: frame == 6 || frame == 7 || frame == 8,
                ..InputState::default()
            })
            .collect::<Vec<_>>();
        let trace = simulate_trace(player.clone(), &inputs, &map, inputs.len() as u32).unwrap();

        assert_eq!(trace.states[7].stamina, 82.5);
        assert_eq!(trace.states[8].stamina, 55.0);
        assert_eq!(trace.states[9].stamina, 27.5);
        assert!((trace.states[8].wall_speed_retained - 190.33347).abs() < 0.001);
        assert_eq!(trace.states[45].pos, Vec2::new(149.0, 120.0));
        assert!(trace.states[45].on_ground);
        assert_eq!(target_x - wall_x, 72.0);
        assert!(trace.states[45].pos.x >= target_x - 4.0);

        let late_inputs = (0..120)
            .map(|frame| InputState {
                move_x: 1,
                jump_pressed: frame == 5 || frame == 7 || frame == 8 || frame == 9,
                jump_held: frame <= 22,
                grab_held: frame == 7 || frame == 8 || frame == 9,
                ..InputState::default()
            })
            .collect::<Vec<_>>();
        let late = simulate_trace(player, &late_inputs, &map, late_inputs.len() as u32).unwrap();
        assert!(
            !late
                .states
                .iter()
                .any(|state| state.on_ground && state.pos.x >= target_x - 4.0)
        );
    }

    #[test]
    fn eleven_jump_buffers_three_climb_jumps_across_a_room_transition() {
        let next_room = Rect::new(320.0, 0.0, 320.0, 184.0);
        let wall_x = 328.0;
        let target_x = wall_x + 11.0 * 8.0;
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            transition_rooms: vec![next_room],
            solids: vec![
                Rect::new(wall_x, 80.0, 8.0, 16.0),
                Rect::new(target_x, 128.0, 80.0, 8.0),
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(316.0, 86.0),
            speed: Vec2::new(160.0, -30.0),
            stamina: 20.0,
            ..PlayerSnapshot::default()
        };
        let inputs = (0..120)
            .map(|frame| InputState {
                move_x: 1,
                jump_pressed: frame == 37 || frame == 42 || frame == 43,
                jump_held: (37..60).contains(&frame),
                grab_held: (37..=43).contains(&frame),
                ..InputState::default()
            })
            .collect::<Vec<_>>();
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();

        assert_eq!(trace.states[41].current_room_bounds, Some(next_room));
        assert_eq!(trace.states[6].speed.x, 156.0);
        assert_eq!(trace.states[42].stamina, 82.5);
        assert_eq!(trace.states[43].stamina, 55.0);
        assert_eq!(trace.states[44].stamina, 27.5);
        assert!(trace.states[44].wall_speed_retained > 190.0);
        assert!(
            trace
                .states
                .iter()
                .any(|state| state.on_ground && state.pos.x >= target_x - 4.0),
            "last={:?}, max_x={}",
            trace.states.last().unwrap(),
            trace
                .states
                .iter()
                .map(|state| state.pos.x)
                .fold(f32::NEG_INFINITY, f32::max),
        );
    }

    #[test]
    fn reverse_cornerboost_preserves_forward_momentum_minus_backward_jump_boost() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![Rect::new(104.0, 120.0, 8.0, 64.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(116.0, 122.0),
            speed: Vec2::new(160.0, -30.0),
            facing: false,
            stamina: 110.0,
            dash_attack_timer: 0.3,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(
            player,
            &[InputState {
                move_x: -1,
                jump_pressed: true,
                jump_held: true,
                grab_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();

        assert_eq!(trace.states[1].state, PlayerState::Normal);
        assert_eq!(trace.states[1].stamina, 82.5);
        assert_eq!(trace.states[1].dash_attack_timer, 0.0);
        assert!((trace.states[1].speed.x - 109.166_664).abs() < 0.001);
        assert_eq!(trace.states[1].facing, false);
    }

    #[test]
    fn neutral_reverse_cornerboost_keeps_speed_then_converts_within_wallboost_window() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![Rect::new(104.0, 120.0, 8.0, 64.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(116.0, 122.0),
            speed: Vec2::new(160.0, -30.0),
            facing: false,
            stamina: 110.0,
            dash_attack_timer: 0.3,
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState {
                jump_pressed: true,
                jump_held: true,
                grab_held: true,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                jump_held: true,
                ..InputState::default()
            },
        ];
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();

        assert!((trace.states[1].speed.x - 149.166_64).abs() < 0.001);
        assert_eq!(trace.states[1].wall_boost_dir, 1);
        assert!(trace.states[1].wall_boost_timer > 0.19);
        assert_eq!(trace.states[1].stamina, 82.5);
        assert!((trace.states[3].speed.x - 125.666_66).abs() < 0.001);
        assert_eq!(trace.states[3].stamina, 110.0);
        assert_eq!(trace.states[3].wall_boost_timer, 0.0);
    }

    #[test]
    fn spiked_cornerboost_survives_only_while_rising_away_from_top_spikes() {
        let spikes = crate::Entity {
            kind: EntityKind::Spikes,
            bounds: Rect::new(36.0, 37.0, 12.0, 3.0),
            direction: Vec2::new(0.0, -1.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "spikesUp".to_owned(),
        };
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![Rect::new(40.0, 40.0, 8.0, 64.0)],
            entities: vec![spikes],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(35.0, 46.0),
            speed: Vec2::new(90.0, -30.0),
            facing: true,
            stamina: 110.0,
            ..PlayerSnapshot::default()
        };
        let inputs = std::array::from_fn::<_, 5, _>(|frame| InputState {
            move_x: 1,
            jump_pressed: frame == 0,
            jump_held: true,
            grab_held: frame == 0,
            ..InputState::default()
        });
        let cornerboost =
            simulate_trace(player.clone(), &inputs, &map, inputs.len() as u32).unwrap();
        assert!(cornerboost.states.iter().all(|state| !state.dead));
        assert!(cornerboost.states[1].wall_speed_retained > 120.0);

        let falling = simulate(
            PlayerSnapshot {
                pos: Vec2::new(35.0, 41.0),
                speed: Vec2::new(0.0, 30.0),
                ..player
            },
            &[InputState::default()],
            &map,
            1,
        )
        .unwrap();
        assert!(falling.dead);
    }

    #[test]
    fn spike_climb_wall_jump_sets_away_speed_before_the_spike_check() {
        let spikes = crate::Entity {
            kind: EntityKind::Spikes,
            bounds: Rect::new(61.0, 40.0, 3.0, 120.0),
            direction: Vec2::new(-1.0, 0.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "spikesLeft".to_owned(),
        };
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![Rect::new(64.0, 40.0, 8.0, 144.0)],
            entities: vec![spikes],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(59.0, 140.0),
            facing: true,
            ..PlayerSnapshot::default()
        };
        let climbed = simulate(
            player.clone(),
            &[InputState {
                move_x: -1,
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert!(!climbed.dead);
        assert_eq!(climbed.speed, Vec2::new(-WALL_JUMP_H, JUMP_SPEED));
        assert!(climbed.pos.y < player.pos.y);

        let stalled = simulate(player, &[InputState::default()], &map, 1).unwrap();
        assert!(stalled.dead);
    }

    #[test]
    fn narrow_spiked_climb_alternates_away_facing_wall_jumps() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![
                Rect::new(40.0, 24.0, 8.0, 160.0),
                Rect::new(64.0, 24.0, 8.0, 160.0),
            ],
            entities: vec![
                crate::Entity {
                    kind: EntityKind::Spikes,
                    bounds: Rect::new(48.0, 146.0, 3.0, 22.0),
                    direction: Vec2::new(1.0, 0.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "spikesRight".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::Spikes,
                    bounds: Rect::new(61.0, 140.0, 3.0, 28.0),
                    direction: Vec2::new(-1.0, 0.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "spikesLeft".to_owned(),
                },
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(59.0, 152.0),
            facing: true,
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState {
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                jump_held: true,
                ..InputState::default()
            },
        ];
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();

        assert!(
            trace.states.iter().all(|state| !state.dead),
            "timeline={:?}",
            trace
                .states
                .iter()
                .map(|state| (state.pos, state.speed, state.dead))
                .collect::<Vec<_>>(),
        );
        assert_eq!(trace.states[1].speed.x, -WALL_JUMP_H);
        assert!(trace.states[4].speed.x > 0.0);
        assert!(trace.states[5].pos.y < trace.states[1].pos.y - 6.0);
    }

    #[test]
    fn spike_clip_requires_the_hurtbox_bottom_to_skip_past_unsupported_spikes() {
        let spikes = crate::Entity {
            kind: EntityKind::Spikes,
            bounds: Rect::new(80.0, 100.0, 24.0, 3.0),
            direction: Vec2::new(0.0, -1.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "spikesUp".to_owned(),
        };
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![spikes],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(92.0, 103.0),
            ..PlayerSnapshot::default()
        };
        let slow = simulate(
            PlayerSnapshot {
                speed: Vec2::new(0.0, 30.0),
                ..player.clone()
            },
            &[InputState::default()],
            &map,
            1,
        )
        .unwrap();
        assert!(slow.dead);

        let clipped = simulate(
            PlayerSnapshot {
                speed: Vec2::new(0.0, 240.0),
                ..player
            },
            &[InputState {
                move_y: 1,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert!(!clipped.dead);
        assert_eq!(clipped.pos.y, 107.0);
        assert!(current_player_hurt_rect(&clipped).bottom() > 103.0);
    }

    #[test]
    fn spike_jump_uses_the_frame_after_zip_carry_bypasses_player_colliders() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![
                crate::Entity {
                    kind: EntityKind::ZipMover,
                    bounds: Rect::new(32.0, 120.0, 32.0, 16.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![Vec2::new(64.0, 120.0)],
                    name: "zipMover".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::Spikes,
                    bounds: Rect::new(65.0, 117.0, 16.0, 3.0),
                    direction: Vec2::new(0.0, -1.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "spikesUp".to_owned(),
                },
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(48.0, 120.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let idle_inputs = [InputState::default(); 45];
        let idle = simulate_trace(player.clone(), &idle_inputs, &map, 45).unwrap();
        let lethal_state = idle
            .states
            .iter()
            .position(|state| state.dead)
            .expect("ZipMover should carry the idle player into the fixed spikes");
        assert!(lethal_state > 1);
        assert!(!idle.states[lethal_state - 1].dead);

        let mut jump_inputs = idle_inputs;
        jump_inputs[lethal_state - 1] = InputState {
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };
        let jumped = simulate_trace(player, &jump_inputs, &map, 45).unwrap();
        let proof_end = (lethal_state + 8).min(jumped.states.len());
        assert!(
            jumped.states[..proof_end].iter().all(|state| !state.dead),
            "lethal_state={lethal_state}, first jumped death={:?}",
            jumped.states.iter().position(|state| state.dead),
        );
        assert_eq!(jumped.states[lethal_state].speed.y, JUMP_SPEED);
        assert!(!jumped.states[lethal_state].on_ground);
    }

    #[test]
    fn cornerboost_wallboost_overwrites_retained_speed_with_wallkick_speed() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            solids: vec![Rect::new(40.0, 40.0, 8.0, 64.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(35.0, 46.0),
            speed: Vec2::new(160.0, -30.0),
            facing: true,
            stamina: 110.0,
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState {
                jump_pressed: true,
                jump_held: true,
                grab_held: true,
                ..InputState::default()
            },
            InputState {
                move_x: -1,
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                move_x: -1,
                jump_held: true,
                ..InputState::default()
            },
        ];
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();

        assert!(trace.states[1].wall_speed_retained > 140.0);
        assert_eq!(trace.states[1].wall_boost_dir, -1);
        assert_eq!(trace.states[1].stamina, 82.5);
        assert!(trace.states[3].speed.x < -120.0);
        assert!(trace.states[3].speed.x > -130.0);
        assert_eq!(trace.states[3].stamina, 110.0);
        assert_eq!(trace.states[3].wall_boost_timer, 0.0);
        assert!(trace.states[3].speed.x.abs() < trace.states[1].wall_speed_retained);
    }

    #[test]
    fn cornerslip_over_disabled_dream_block_refills_without_vertical_collision() {
        let dream_block = crate::Entity {
            kind: EntityKind::DreamBlock,
            bounds: Rect::new(40.0, 40.0, 32.0, 32.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "dreamBlock".to_owned(),
        };
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            entities: vec![dream_block],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(37.0, 40.0),
            speed: Vec2::new(-90.0, 60.0),
            dashes: 0,
            can_dream_dash: false,
            ..PlayerSnapshot::default()
        };
        let slipped = simulate(
            player.clone(),
            &[InputState {
                move_x: -1,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();

        assert_eq!(slipped.pos, Vec2::new(35.0, 41.0));
        assert_eq!(slipped.speed, Vec2::new(-90.0, 60.0));
        assert_eq!(slipped.dashes, 1);
        assert!(!slipped.on_ground);
        assert_eq!(slipped.jump_grace_timer, JUMP_GRACE);

        let collided = simulate(
            PlayerSnapshot {
                speed: Vec2::new(0.0, 60.0),
                ..player
            },
            &[InputState::default()],
            &map,
            1,
        )
        .unwrap();
        assert_eq!(collided.pos.y, 40.0);
        assert_eq!(collided.speed.y, 0.0);
    }

    #[test]
    fn dash_spends_dash_and_is_diagonal_normalized() {
        let input = InputState {
            move_x: 1,
            move_y: -1,
            dash_pressed: true,
            ..InputState::default()
        };
        let first = simulate(
            PlayerSnapshot {
                facing: false,
                ..grounded_player()
            },
            &[input],
            &floor_map(),
            1,
        )
        .unwrap();
        assert_eq!(first.speed, Vec2::default());
        assert_eq!(first.freeze_timer, 0.05);
        assert!(first.facing);
        let held_aim = InputState {
            move_x: 1,
            move_y: -1,
            ..InputState::default()
        };
        let frozen = simulate(first, &[held_aim; 3], &floor_map(), 3).unwrap();
        assert_eq!(frozen.speed, Vec2::default());
        assert_eq!(frozen.freeze_timer, 0.0);
        assert!(frozen.facing);
        let p = simulate(frozen, &[held_aim], &floor_map(), 1).unwrap();
        assert_eq!(p.state, PlayerState::Dash);
        assert_eq!(p.dashes, 0);
        assert!((p.speed.x.abs() - 169.70563).abs() < 0.01);
        assert!(p.facing);
    }

    #[test]
    fn vertical_dash_entry_clears_velocity_before_the_coroutine_launches() {
        // Player.DashBegin saves beforeDashSpeed, then clears both axes before
        // DashCoroutine's initial `yield return null`. This needs to hold for
        // both pure vertical directions, not just horizontal dashes.
        for facing in [false, true] {
            for (move_y, before_dash_speed) in
                [(-1, Vec2::new(123.0, -80.0)), (1, Vec2::new(-123.0, 80.0))]
            {
                let inputs = std::array::from_fn::<_, 5, _>(|frame| InputState {
                    move_y,
                    dash_pressed: frame == 0,
                    ..InputState::default()
                });
                let trace = simulate_trace(
                    PlayerSnapshot {
                        pos: Vec2::new(64.0, 64.0),
                        speed: before_dash_speed,
                        facing,
                        dashes: 1,
                        ..PlayerSnapshot::default()
                    },
                    &inputs,
                    &Map::default(),
                    inputs.len() as u32,
                )
                .unwrap();
                let entry = &trace.states[1];
                let launched = &trace.states[5];

                assert_eq!(entry.state, PlayerState::Dash);
                assert_eq!(entry.before_dash_speed, before_dash_speed);
                assert_eq!(entry.speed, Vec2::default());
                assert_eq!(entry.dash_dir, Vec2::default());
                assert_eq!(entry.pos, Vec2::new(64.0, 64.0));
                assert_eq!(entry.facing, facing);
                assert_eq!(launched.state, PlayerState::Dash);
                assert_eq!(launched.dash_dir, Vec2::new(0.0, move_y as f32));
                assert_eq!(launched.speed, Vec2::new(0.0, move_y as f32 * DASH_SPEED));
            }
        }
    }

    #[test]
    fn dash_direction_is_sampled_when_coroutine_resumes_after_freeze() {
        let inputs = [
            InputState::default(),
            InputState {
                dash_pressed: true,
                ..InputState::default()
            },
            InputState {
                move_x: -1,
                ..InputState::default()
            },
            InputState {
                move_x: -1,
                ..InputState::default()
            },
            InputState {
                move_x: -1,
                ..InputState::default()
            },
            InputState {
                move_x: -1,
                ..InputState::default()
            },
        ];
        let trace = simulate_trace(
            grounded_player(),
            &inputs,
            &floor_map(),
            inputs.len() as u32,
        )
        .unwrap();

        let dash_begin = &trace.states[2];
        assert_eq!(dash_begin.state, PlayerState::Dash);
        assert_eq!(dash_begin.speed, Vec2::default());
        assert_eq!(dash_begin.dash_dir, Vec2::default());
        assert!(dash_begin.facing);

        let launched = &trace.states[6];
        assert_eq!(launched.state, PlayerState::Dash);
        assert_eq!(launched.dash_dir, Vec2::new(-1.0, 0.0));
        assert_eq!(launched.speed, Vec2::new(-DASH_SPEED, 0.0));
        assert!(!launched.facing);
    }

    #[test]
    fn subpixel_manipulation_accumulates_air_control_until_a_pixel_crossing() {
        let player = PlayerSnapshot {
            pos: Vec2::new(160.0, 80.0),
            ..PlayerSnapshot::default()
        };
        let inputs = std::array::from_fn::<_, 5, _>(|frame| InputState {
            move_x: if frame % 2 == 0 { 1 } else { -1 },
            ..InputState::default()
        });
        let trace = simulate_trace(player, &inputs, &Map::default(), inputs.len() as u32).unwrap();

        assert_eq!(trace.states[1].pos.x, 160.0);
        assert!((trace.states[1].movement_remainder.x - 0.180_556_27).abs() < 0.000_001);
        assert_eq!(trace.states[3].pos.x, 160.0);
        assert!((trace.states[3].movement_remainder.x - 0.361_112_53).abs() < 0.000_001);
        assert_eq!(trace.states[5].pos.x, 161.0);
        assert!((trace.states[5].movement_remainder.x + 0.458_331_23).abs() < 0.000_001);
        assert_eq!(trace.states[5].state, PlayerState::Normal);
        assert!(trace.states[5].facing);
        assert_eq!(trace.states[5].dashes, 1);
        assert_eq!(trace.states[5].stamina, 110.0);
        assert!(!trace.states[5].on_ground);
        assert!(!trace.states[5].ducking);
        assert!(!trace.states[5].dead);
    }
    #[test]
    fn crouch_dash_starts_a_demo_dash() {
        let input = InputState {
            move_x: 1,
            crouch_dash_pressed: true,
            ..InputState::default()
        };
        let p = simulate(grounded_player(), &[input], &floor_map(), 1).unwrap();
        assert_eq!(p.state, PlayerState::Dash);
        assert!(p.demo_dashed);
        assert!(p.ducking);
        assert_eq!(p.dashes, 0);
    }
    #[test]
    fn undemo_redirects_after_dash_begin_without_changing_the_standing_collider() {
        let inputs = [
            InputState {
                move_x: 1,
                dash_pressed: true,
                ..InputState::default()
            },
            InputState {
                move_y: 1,
                ..InputState::default()
            },
            InputState {
                move_y: 1,
                ..InputState::default()
            },
            InputState {
                move_y: 1,
                ..InputState::default()
            },
            InputState {
                move_y: 1,
                ..InputState::default()
            },
        ];
        let trace = simulate_trace(
            PlayerSnapshot {
                pos: Vec2::new(160.0, 80.0),
                ..PlayerSnapshot::default()
            },
            &inputs,
            &Map::default(),
            inputs.len() as u32,
        )
        .unwrap();

        assert_eq!(trace.states[1].state, PlayerState::Dash);
        assert_eq!(trace.states[1].dash_dir, Vec2::default());
        assert!(!trace.states[1].demo_dashed);
        assert!(!trace.states[1].ducking);
        assert_eq!(trace.states[5].dash_dir, Vec2::new(0.0, 1.0));
        assert_eq!(trace.states[5].speed, Vec2::new(0.0, DASH_SPEED));
        assert!(!trace.states[5].ducking);
    }
    #[test]
    fn crouching_uses_source_duck_friction_and_six_pixel_collider() {
        let p = simulate(
            grounded_player(),
            &[InputState {
                move_x: 1,
                move_y: 1,
                ..InputState::default()
            }],
            &floor_map(),
            1,
        )
        .unwrap();
        assert!(p.ducking);
        assert_eq!(p.speed.x, 0.0);
        assert_eq!(current_player_rect(&p, p.pos.x, p.pos.y).height, 6.0);
    }

    #[test]
    fn archie_preserves_the_source_two_and_a_half_pixel_center_offset() {
        let mut map = booster_map();
        let standing = PlayerSnapshot {
            pos: Vec2::new(160.0, 400.0),
            ..PlayerSnapshot::default()
        };
        let ducking = PlayerSnapshot {
            ducking: true,
            ..standing.clone()
        };
        let standing_inputs = [InputState::default(); 20];
        let mut ducking_inputs = [InputState::default(); 20];
        ducking_inputs[0].move_y = 1;
        let standing = simulate_trace(standing, &standing_inputs, &map, 20).unwrap();
        let ducking = simulate_trace(ducking, &ducking_inputs, &map, 20).unwrap();
        let max_height_gain = standing
            .states
            .iter()
            .zip(&ducking.states)
            .map(|(normal, archie)| normal.pos.y - archie.pos.y)
            .fold(0.0_f32, f32::max);
        // The normal collider center is -5.5 while the duck collider center is
        // -3.0. Their exact 2.5-pixel separation appears as a three-pixel peak
        // after Monocle's ties-to-even integer movement.
        assert_eq!(max_height_gain, 3.0);
    }

    #[test]
    fn automatic_booster_dash_unducks_an_airborne_archie_in_open_space() {
        let p = PlayerSnapshot {
            pos: Vec2::new(160.0, 400.0),
            ducking: true,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 36], &booster_map(), 36).unwrap();
        let auto_dash = trace
            .states
            .iter()
            .find(|state| state.state == PlayerState::Dash)
            .expect("booster coroutine should enter Dash");
        assert!(!auto_dash.on_ground);
        assert!(!auto_dash.ducking);
    }

    #[test]
    fn bubble_super_uses_coyote_grace_and_keeps_the_refilled_dash() {
        let p = PlayerSnapshot {
            pos: Vec2::new(220.0, 400.0),
            speed: Vec2::new(90.0, 0.0),
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState::default(); 16];
        for (frame, input) in inputs.iter_mut().enumerate() {
            input.move_x = 1;
            input.dash_pressed = frame == 5;
            input.jump_pressed = frame == 9;
            input.jump_held = (9..15).contains(&frame);
        }
        let trace = simulate_trace(p, &inputs, &crate::mechanics_playground(), 16).unwrap();
        let jumped = trace
            .states
            .iter()
            .find(|state| state.state == PlayerState::Normal && state.speed.y < 0.0)
            .expect("bubble super should jump during coyote grace");
        assert_eq!(jumped.speed.x, SUPER_JUMP_H);
        assert_eq!(jumped.speed.y, JUMP_SPEED);
        assert_eq!(jumped.dashes, 1);
    }

    #[test]
    fn bubble_demohyper_uses_coyote_grace_and_keeps_the_refilled_dash() {
        let p = PlayerSnapshot {
            pos: Vec2::new(220.0, 400.0),
            speed: Vec2::new(90.0, 0.0),
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState::default(); 16];
        for (frame, input) in inputs.iter_mut().enumerate() {
            input.move_x = 1;
            input.crouch_dash_pressed = frame == 5;
            input.jump_pressed = frame == 9;
            input.jump_held = (9..15).contains(&frame);
        }
        let trace = simulate_trace(p, &inputs, &crate::mechanics_playground(), 16).unwrap();
        let jumped = trace
            .states
            .iter()
            .find(|state| state.state == PlayerState::Normal && state.speed.y < 0.0)
            .expect("bubble demohyper should jump during coyote grace");
        assert_eq!(jumped.speed.x, SUPER_JUMP_H * 1.25);
        assert_eq!(jumped.speed.y, JUMP_SPEED * 0.5);
        assert_eq!(jumped.dashes, 1);
    }

    #[test]
    fn dash_virtual_button_buffer_survives_global_freeze() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 64.0),
            freeze_timer: 0.05,
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState {
                move_x: 1,
                dash_pressed: true,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
        ];
        let p = simulate(p, &inputs, &Map::default(), 4).unwrap();
        assert_eq!(p.state, PlayerState::Dash);
        assert_eq!(p.dashes, 0);
        assert_eq!(p.dash_buffer_timer, 0.0);
    }

    #[test]
    fn start_dash_consumes_both_buffers_even_when_a_booster_wins_the_final_state() {
        let p = PlayerSnapshot {
            pos: Vec2::new(160.0, 400.0),
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(
            p,
            &[
                InputState {
                    move_x: 1,
                    dash_pressed: true,
                    ..InputState::default()
                },
                InputState::default(),
            ],
            &booster_map(),
            2,
        )
        .unwrap();
        assert_eq!(trace.states[1].state, PlayerState::Boost);
        assert_eq!(trace.states[1].dash_buffer_timer, 0.0);
        assert_eq!(trace.states[1].crouch_dash_buffer_timer, 0.0);
        assert_eq!(trace.states[2].state, PlayerState::Boost);
    }

    #[test]
    fn boost_approach_uses_the_normal_collider_half_pixel_center() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            entities: vec![crate::Entity {
                kind: EntityKind::Booster,
                bounds: Rect::new(712.0, 312.0, 16.0, 16.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "booster".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(720.0, 330.0),
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState {
                move_x: 1,
                dash_pressed: true,
                ..InputState::default()
            },
            InputState::default(),
            InputState::default(),
            InputState::default(),
            InputState {
                move_x: 1,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
        ];
        let trace = simulate_trace(p, &inputs, &map, inputs.len() as u32).unwrap();

        assert_eq!(trace.states[5].pos, Vec2::new(721.0, 329.0));
        assert!((trace.states[5].movement_remainder.x - 0.024_291_992).abs() < 0.000_001);
        assert!((trace.states[5].movement_remainder.y - 0.146_423_34).abs() < 0.000_001);
        assert_eq!(trace.states[6].pos, Vec2::new(722.0, 328.0));
        assert!((trace.states[6].movement_remainder.x - 0.048_583_984).abs() < 0.000_001);
        assert!((trace.states[6].movement_remainder.y - 0.292_846_68).abs() < 0.000_001);
        assert_eq!(trace.states[7].pos, Vec2::new(723.0, 328.0));
        assert_eq!(trace.states[7].movement_remainder, Vec2::new(0.0, -0.5));
        assert_eq!(trace.states[8].pos, Vec2::new(723.0, 328.0));
        assert_eq!(trace.states[8].movement_remainder, Vec2::new(0.0, -0.5));
    }

    #[test]
    fn dream_dash_enters_a_dream_block() {
        let mut map = Map {
            entities: vec![crate::Entity {
                kind: EntityKind::DreamBlock,
                bounds: Rect::new(40.0, 40.0, 32.0, 40.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "dreamBlock".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 64.0),
            dashes: 1,
            can_dream_dash: true,
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState {
                move_x: 1,
                dash_pressed: true,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
        ];
        let p = simulate(p, &inputs, &map, inputs.len() as u32).unwrap();
        assert_eq!(p.state, PlayerState::DreamDash);
        assert_eq!(p.speed, Vec2::new(240.0, 0.0));
    }

    #[test]
    fn dream_dash_inventory_is_recovered_from_a_spent_dream_dash_timer() {
        // A trace/anchor snapshot cannot carry `Inventory.DreamDash` (it is session state,
        // `Player.cs:3420`), but a `dreamDashCanEndTimer` that has already been spent down
        // to its negative residue (`Player.cs:5189-5192`) can only exist after a
        // `DreamDashBegin` (`Player.cs:5144`), i.e. after the inventory was granted.
        let mut map = Map {
            entities: vec![crate::Entity {
                kind: EntityKind::DreamBlock,
                bounds: Rect::new(40.0, 40.0, 32.0, 40.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "dreamBlock".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 64.0),
            dashes: 1,
            // The exact residue the vanilla 202-berry trace carries on the frames that
            // expose this cluster.
            dream_dash_can_end_timer: -1.937_150_955_200_195_3e-7,
            ..PlayerSnapshot::default()
        };
        let inputs = [InputState {
            move_x: 1,
            dash_pressed: true,
            ..InputState::default()
        }; 6];
        let p = simulate(p, &inputs, &map, inputs.len() as u32).unwrap();
        assert!(p.can_dream_dash);
        assert_eq!(p.state, PlayerState::DreamDash);
        assert_eq!(p.speed, Vec2::new(240.0, 0.0));
    }

    #[test]
    fn fresh_player_timer_leaves_the_dream_dash_inventory_off() {
        // The mirror cutscene has not run yet (`PlayerInventory.OldSite` is
        // `dreamDash: false`, `PlayerInventory.cs:12`): the timer is still at the C#
        // default `0f` (`Player.cs:551`) and the dream block must stay a wall.
        let mut map = Map {
            entities: vec![crate::Entity {
                kind: EntityKind::DreamBlock,
                bounds: Rect::new(40.0, 40.0, 32.0, 40.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "dreamBlock".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 64.0),
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let inputs = [InputState {
            move_x: 1,
            dash_pressed: true,
            ..InputState::default()
        }; 6];
        let p = simulate(p, &inputs, &map, inputs.len() as u32).unwrap();
        assert!(!p.can_dream_dash);
        assert_eq!(p.state, PlayerState::Dash);
        assert_eq!(p.speed, Vec2::new(0.0, 0.0));
    }

    #[test]
    fn grounded_down_diagonal_dash_enters_the_dream_block_below() {
        let mut map = Map {
            entities: vec![crate::Entity {
                kind: EntityKind::DreamBlock,
                bounds: Rect::new(40.0, 40.0, 32.0, 40.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "dreamBlock".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(56.0, 40.0),
            dashes: 1,
            can_dream_dash: true,
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState {
            move_x: 1,
            move_y: 1,
            ..InputState::default()
        }; 6];
        inputs[0].dash_pressed = true;
        let p = simulate(p, &inputs, &map, inputs.len() as u32).unwrap();
        let diagonal_speed = DASH_SPEED * std::f32::consts::FRAC_1_SQRT_2;

        assert_eq!(p.state, PlayerState::DreamDash);
        assert!((p.speed.x - diagonal_speed).abs() < 0.000_1);
        assert!((p.speed.y - diagonal_speed).abs() < 0.000_1);
    }

    #[test]
    fn dream_dash_check_uses_lingering_attack_and_then_moves_naively() {
        let mut map = Map {
            bounds: Rect::new(0.0, -100.0, 960.0, 280.0),
            entities: vec![crate::Entity {
                kind: EntityKind::DreamBlock,
                bounds: Rect::new(880.0, -32.0, 32.0, 40.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "dreamBlock".to_owned(),
            }],
            ..Map::default()
        };
        let diagonal = std::f32::consts::FRAC_1_SQRT_2;
        let p = PlayerSnapshot {
            pos: Vec2::new(876.0, -14.0),
            speed: Vec2::new(231.333_31, 160.0),
            state: PlayerState::Normal,
            facing: true,
            dashes: 0,
            stamina: 110.0,
            on_ground: false,
            dash_dir: Vec2::new(diagonal, diagonal),
            dash_attack_timer: 0.1,
            can_dream_dash: true,
            movement_remainder: Vec2::new(-0.216, 0.446),
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            move_x: 1,
            ..InputState::default()
        };
        let trace = simulate_trace(p, &[input; 2], &map, 2).unwrap();

        let entered = &trace.states[1];
        assert_eq!(entered.state, PlayerState::DreamDash);
        assert_eq!(entered.pos, Vec2::new(876.0, -14.0));
        assert!((entered.speed.x - 169.705_63).abs() < 0.000_1);
        assert!((entered.speed.y - 169.705_63).abs() < 0.000_1);
        assert_eq!(entered.dream_dash_can_end_timer, 0.1);
        assert_eq!(entered.dash_attack_timer, 0.0);

        let travelled = &trace.states[2];
        assert_eq!(travelled.state, PlayerState::DreamDash);
        assert_eq!(travelled.pos, Vec2::new(879.0, -11.0));
        assert!((travelled.speed.x - 169.705_63).abs() < 0.000_1);
        assert!((travelled.speed.y - 169.705_63).abs() < 0.000_1);
    }

    #[test]
    fn dream_jump_runs_on_exit_and_restores_horizontal_exit_grace() {
        let p = PlayerSnapshot {
            pos: Vec2::new(72.0, 64.0),
            speed: Vec2::new(240.0, 0.0),
            state: PlayerState::DreamDash,
            dash_dir: Vec2::new(1.0, 0.0),
            dream_dash_can_end_timer: 0.0,
            ..PlayerSnapshot::default()
        };
        let p = simulate(
            p,
            &[InputState {
                move_x: 1,
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            }],
            &dream_exit_map(),
            1,
        )
        .unwrap();
        assert_eq!(p.state, PlayerState::Normal);
        assert_eq!(p.speed, Vec2::new(280.0, JUMP_SPEED));
        assert_eq!(p.pos, Vec2::new(81.0, 62.0));
        assert_eq!(p.jump_grace_timer, JUMP_GRACE);
        assert_eq!(p.var_jump_timer, VAR_JUMP_TIME);
    }

    #[test]
    fn grounded_diagonal_dream_exit_keeps_duck_collider_for_ordinary_movement() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            solids: vec![Rect::new(0.0, 80.0, 320.0, 100.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::DreamBlock,
                bounds: Rect::new(73.0, 40.0, 32.0, 40.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "dreamBlock".to_owned(),
            }],
            ..Map::default()
        };
        let diagonal_speed = DASH_SPEED * std::f32::consts::FRAC_1_SQRT_2;
        let p = PlayerSnapshot {
            pos: Vec2::new(72.0, 74.0),
            speed: Vec2::new(-diagonal_speed, diagonal_speed),
            state: PlayerState::DreamDash,
            dash_dir: Vec2::new(
                -std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
            ),
            ducking: true,
            dream_dash_can_end_timer: 0.0,
            movement_remainder: Vec2::new(0.2, -0.03),
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &map, 1).unwrap();

        assert_eq!(p.state, PlayerState::Normal);
        assert!(p.ducking);
        assert_eq!(p.jump_grace_timer, JUMP_GRACE);
        assert_eq!(p.pos, Vec2::new(66.0, 80.0));
        assert!(
            (p.movement_remainder.x - 0.171_567_44).abs() < 0.000_001,
            "exit remainder was {:?}",
            p.movement_remainder
        );
    }

    #[test]
    fn jump_buffer_survives_dream_exit_freeze_for_the_second_jump() {
        let p = PlayerSnapshot {
            pos: Vec2::new(825.0, -52.0),
            speed: Vec2::new(280.0, JUMP_SPEED),
            state: PlayerState::Normal,
            facing: true,
            on_ground: false,
            freeze_timer: 0.05,
            jump_grace_timer: JUMP_GRACE,
            var_jump_timer: VAR_JUMP_TIME,
            var_jump_speed: JUMP_SPEED,
            movement_remainder: Vec2::new(-0.333_333, 0.25),
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState {
                move_x: 1,
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                jump_held: true,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                jump_held: true,
                ..InputState::default()
            },
        ];
        let mut map = Map {
            bounds: Rect::new(0.0, -100.0, 960.0, 280.0),
            ..Map::default()
        };
        let trace = simulate_trace(p, &inputs, &map, 4).unwrap();

        assert_eq!(trace.states[1].jump_buffer_timer, JUMP_BUFFER_TIME);
        assert!((trace.states[2].jump_buffer_timer - (JUMP_BUFFER_TIME - DT)).abs() < 0.000_001);
        assert!(
            (trace.states[3].jump_buffer_timer - (JUMP_BUFFER_TIME - DT * 2.0)).abs() < 0.000_001
        );
        let second_jump = &trace.states[4];
        assert_eq!(second_jump.speed.y, JUMP_SPEED);
        assert!(
            (second_jump.speed.x - 315.666_66).abs() < 0.000_1,
            "second jump speed was {:?}",
            second_jump.speed
        );
        assert_eq!(second_jump.jump_buffer_timer, 0.0);
        assert_eq!(second_jump.jump_grace_timer, 0.0);
    }

    #[test]
    fn dream_grab_catches_the_block_wall_on_the_exit_frame() {
        let p = PlayerSnapshot {
            pos: Vec2::new(68.0, 64.0),
            speed: Vec2::new(240.0, 0.0),
            state: PlayerState::DreamDash,
            dash_dir: Vec2::new(1.0, 0.0),
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState::default(),
            InputState {
                move_x: -1,
                grab_held: true,
                ..InputState::default()
            },
        ];
        let p = simulate(p, &inputs, &dream_exit_map(), 2).unwrap();
        assert_eq!(p.state, PlayerState::Climb);
        assert_eq!(p.pos, Vec2::new(76.0, 64.0));
        assert!(!p.facing);
        assert_eq!(p.jump_grace_timer, JUMP_GRACE);
    }

    #[test]
    fn dream_grab_uses_v14_five_pixel_static_solid_correction() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            solids: vec![Rect::new(67.0, 40.0, 1.0, 40.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::DreamBlock,
                bounds: Rect::new(40.0, 40.0, 32.0, 40.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "dreamBlock".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(76.0, 64.0),
            state: PlayerState::DreamDash,
            dash_dir: Vec2::new(1.0, 0.0),
            ..PlayerSnapshot::default()
        };
        let p = simulate(
            p,
            &[InputState {
                move_x: -1,
                grab_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert_eq!(p.state, PlayerState::Climb);
        assert_eq!(p.pos, Vec2::new(76.0, 64.0));
        assert_eq!(p.movement_remainder.x, 0.0);
        assert!(!p.facing);
    }

    #[test]
    fn dream_smuggle_keeps_theo_through_pickup_and_lingering_attack_entry() {
        let mut map = dream_smuggle_map();
        let initial = PlayerSnapshot {
            pos: Vec2::new(60.0, 100.0),
            speed: Vec2::new(240.0, 0.0),
            state: PlayerState::Dash,
            dash_dir: Vec2::new(1.0, 0.0),
            dash_attack_timer: DASH_ATTACK_TIME,
            state_timer: 0.1,
            on_ground: true,
            can_dream_dash: true,
            ..PlayerSnapshot::default()
        };
        let held = InputState {
            move_x: 1,
            grab_held: true,
            ..InputState::default()
        };
        let inputs = [held; 30];
        let trace = simulate_trace(initial.clone(), &inputs, &map, inputs.len() as u32).unwrap();
        let pickup = trace
            .states
            .iter()
            .position(|state| state.state == PlayerState::Pickup)
            .unwrap();
        let dream = trace
            .states
            .iter()
            .position(|state| state.state == PlayerState::DreamDash)
            .unwrap();
        assert!(pickup < dream);
        assert!(
            trace.states[pickup..=dream]
                .iter()
                .all(|state| { state.holding_theo == Some(0) && state.theo_crystals[0].held })
        );

        let whole = trace.states.last().unwrap().clone();
        let first = simulate(initial, &inputs[..13], &map, 13).unwrap();
        let split = simulate(first, &inputs[13..], &map, 17).unwrap();
        assert_eq!(split, whole);
    }

    #[test]
    fn holdable_dream_hyper_throw_cannot_hold_hyper_and_regrab_are_split_composable() {
        let mut map = dream_smuggle_map();
        let initial = PlayerSnapshot {
            pos: Vec2::new(180.0, 88.0),
            speed: Vec2::new(0.0, 0.0),
            state: PlayerState::Climb,
            facing: false,
            dashes: 1,
            jump_grace_timer: JUMP_GRACE,
            holding_theo: Some(0),
            theo_crystals: vec![crate::TheoCrystalSnapshot {
                position: Vec2::new(180.0, 76.0),
                held: true,
                ..crate::TheoCrystalSnapshot::default()
            }],
            min_hold_timer: 0.0,
            ..PlayerSnapshot::default()
        };
        let mut inputs = Vec::new();
        inputs.push(InputState {
            move_x: -1,
            ..InputState::default()
        });
        inputs.push(InputState {
            move_x: 1,
            crouch_dash_pressed: true,
            ..InputState::default()
        });
        inputs.extend(
            [InputState {
                move_x: 1,
                ..InputState::default()
            }; 3],
        );
        inputs.push(InputState {
            move_x: 1,
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        });
        inputs.extend(
            [InputState {
                move_x: -1,
                grab_held: true,
                ..InputState::default()
            }; 30],
        );

        let trace = simulate_trace(initial.clone(), &inputs, &map, inputs.len() as u32).unwrap();
        let released = &trace.states[2];
        assert_eq!(released.holding_theo, None);
        assert_eq!(released.before_dash_speed.x, -80.0);
        assert!(released.theo_crystals[0].cannot_hold_timer > 0.0);
        assert!(
            trace.states[2..7]
                .iter()
                .all(|state| state.holding_theo.is_none())
        );
        assert!(
            trace.states.iter().any(|state| {
                state.state == PlayerState::Normal && (state.speed.x - 325.0).abs() < 0.001
            }),
            "trace={:?}",
            trace
                .states
                .iter()
                .enumerate()
                .map(|(frame, state)| (
                    frame,
                    state.state,
                    state.speed,
                    state.holding_theo,
                    state.freeze_timer,
                    state.jump_grace_timer,
                    state.dash_buffer_timer,
                    state.ducking
                ))
                .collect::<Vec<_>>()
        );
        assert!(
            trace
                .states
                .iter()
                .skip(7)
                .any(|state| state.holding_theo == Some(0))
        );

        let whole = trace.states.last().unwrap().clone();
        let first = simulate(initial, &inputs[..7], &map, 7).unwrap();
        let split = simulate(first, &inputs[7..], &map, (inputs.len() - 7) as u32).unwrap();
        assert_eq!(split, whole);
    }

    #[test]
    fn holdable_dream_hyper_regrabs_on_frame_169_after_theo_release_curve() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            solids: vec![Rect::new(0.0, 496.0, 960.0, 48.0)],
            entities: vec![
                crate::Entity {
                    kind: EntityKind::DreamBlock,
                    bounds: Rect::new(231.0, 432.0, 104.0, 64.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "dreamBlock".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::TheoCrystal,
                    bounds: Rect::new(228.0, 486.0, 8.0, 10.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "theoCrystal".to_owned(),
                },
            ],
            ..Map::default()
        };
        let initial = PlayerSnapshot {
            pos: Vec2::new(208.0, 496.0),
            can_dream_dash: true,
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..240)
            .map(|frame| InputState {
                move_x: if (43..54).contains(&frame) || frame >= 85 {
                    -1
                } else {
                    1
                },
                jump_pressed: frame == 62,
                jump_held: frame == 62,
                dash_pressed: frame == 0,
                crouch_dash_pressed: frame == 54,
                grab_held: frame < 52 || frame >= 65,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(initial, &inputs, &map, inputs.len() as u32).unwrap();
        // Player.Update carries Theo after its own movement.  The frame-168
        // snapshot is therefore still Normal; its next NormalUpdate sees the
        // released crystal's source-sized pickup collider and starts the
        // coroutine on frame 169.
        let before = &trace.states[168];
        assert_eq!(before.state, PlayerState::Normal);
        assert_eq!(before.pos, Vec2::new(371.0, 496.0));
        assert_eq!(before.speed, Vec2::new(-90.0, 0.0));
        assert_eq!(before.theo_crystals[0].position, Vec2::new(360.0, 496.0));
        // TheoCrystal.cs assigns this 16x22 pickup Hitbox.  Its right edge
        // now reaches 368, so the untouched source collider overlaps the
        // player's left edge (367) on the following NormalUpdate.
        assert_eq!(
            theo_pickup_rect(before.theo_crystals[0].position).right(),
            368.0
        );
        assert_eq!(
            current_player_rect(before, before.pos.x, before.pos.y).x,
            367.0
        );

        let pickup = &trace.states[169];
        assert_eq!(pickup.state, PlayerState::Pickup);
        assert_eq!(pickup.pos, Vec2::new(371.0, 496.0));
        assert_eq!(pickup.speed, Vec2::default());
        assert_eq!(pickup.holding_theo, Some(0));
        assert_eq!(pickup.theo_crystals[0].position, Vec2::new(371.0, 484.0));

        let tween = &trace.states[170];
        assert_eq!(tween.state, PlayerState::Pickup);
        assert_eq!(tween.holding_theo, Some(0));
        assert_eq!(tween.theo_crystals[0].position, Vec2::new(371.0, 484.0));
    }

    #[test]
    fn holdable_grabless_dream_hyper_uses_exit_grace_without_a_climb_state() {
        let mut map = dream_smuggle_map();
        let initial = PlayerSnapshot {
            pos: Vec2::new(176.0, 64.0),
            speed: Vec2::new(240.0, 0.0),
            state: PlayerState::DreamDash,
            dash_dir: Vec2::new(1.0, 0.0),
            holding_theo: Some(0),
            theo_crystals: vec![crate::TheoCrystalSnapshot {
                position: Vec2::new(176.0, 52.0),
                held: true,
                ..crate::TheoCrystalSnapshot::default()
            }],
            min_hold_timer: 0.0,
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![
            InputState {
                move_x: 1,
                grab_held: true,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                ..InputState::default()
            },
            InputState {
                move_x: 1,
                crouch_dash_pressed: true,
                ..InputState::default()
            },
        ];
        inputs.extend(
            [InputState {
                move_x: 1,
                ..InputState::default()
            }; 5],
        );
        inputs.push(InputState {
            move_x: 1,
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        });
        inputs.extend((0..60).map(|frame| InputState {
            move_x: if frame < 25 { 1 } else { -1 },
            grab_held: true,
            ..InputState::default()
        }));
        let trace = simulate_trace(initial, &inputs, &map, inputs.len() as u32).unwrap();
        assert_eq!(trace.states[1].state, PlayerState::Normal);
        assert_eq!(trace.states[1].jump_grace_timer, JUMP_GRACE);
        assert!(
            !trace
                .states
                .iter()
                .any(|state| state.state == PlayerState::Climb)
        );
        let released = trace
            .states
            .iter()
            .position(|state| state.holding_theo.is_none())
            .unwrap();
        assert!(released > 1);
        assert_eq!(trace.states[released].before_dash_speed.x, 160.0);
        assert!(trace.states[released].theo_crystals[0].cannot_hold_timer > 0.0);
        assert!(
            trace.states[released + 1..released + 6]
                .iter()
                .all(|state| state.holding_theo.is_none())
        );
        assert!(
            trace.states.iter().any(|state| {
                state.state == PlayerState::Normal && (state.speed.x - 325.0).abs() < 0.001
            }),
            "trace={:?}",
            trace
                .states
                .iter()
                .enumerate()
                .map(|(frame, state)| (
                    frame,
                    state.state,
                    state.speed,
                    state.holding_theo,
                    state.freeze_timer,
                    state.jump_grace_timer,
                    state.dash_buffer_timer,
                    state.ducking
                ))
                .collect::<Vec<_>>()
        );
        assert!(
            trace
                .states
                .iter()
                .skip(released + 6)
                .any(|state| state.state == PlayerState::Pickup && state.holding_theo == Some(0))
        );
    }
    #[test]
    fn entering_water_halves_downward_speed_and_enters_swim() {
        let p = PlayerSnapshot {
            pos: Vec2::new(504.0, 456.0),
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &water_map(), 1).unwrap();
        assert_eq!(p.state, PlayerState::Swim);
        assert_eq!(p.pos, Vec2::new(504.0, 456.0));
        assert!((p.speed.y - 7.500_015).abs() < 0.0001);
    }
    #[test]
    fn underwater_swim_uses_sixty_horizontal_max() {
        let p = PlayerSnapshot {
            pos: Vec2::new(504.0, 456.0),
            state: PlayerState::Swim,
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            move_x: 1,
            ..InputState::default()
        };
        let p = simulate(p, &[input; 6], &water_map(), 6).unwrap();
        assert_eq!(p.state, PlayerState::Swim);
        assert_eq!(p.speed.x, SWIM_UNDERWATER_MAX);
        assert_eq!(p.pos, Vec2::new(508.0, 456.0));
    }
    #[test]
    fn swim_dash_keeps_dash_and_is_water_slowed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(504.0, 456.0),
            state: PlayerState::Swim,
            ..PlayerSnapshot::default()
        };
        let dash = InputState {
            move_x: 1,
            dash_pressed: true,
            ..InputState::default()
        };
        let mut inputs = vec![dash];
        inputs.extend([InputState::default(); 4]);
        let p = simulate(p, &inputs, &water_map(), inputs.len() as u32).unwrap();
        assert_eq!(p.state, PlayerState::Dash);
        assert_eq!(p.dashes, 1);
        assert_eq!(p.speed, Vec2::new(180.0, 0.0));
    }
    #[test]
    fn unsupported_state_is_explicit() {
        let p = PlayerSnapshot {
            state: PlayerState::CassetteFly,
            ..PlayerSnapshot::default()
        };
        assert_eq!(
            simulate(p, &[InputState::default()], &Map::default(), 1),
            Err(SimulationError::UnsupportedState(PlayerState::CassetteFly))
        );
    }

    #[test]
    fn intentionally_unsupported_attract_state_is_explicit() {
        let p = PlayerSnapshot {
            state: PlayerState::Attract,
            ..PlayerSnapshot::default()
        };
        assert_eq!(INTENTIONALLY_UNSUPPORTED_STATES, &[PlayerState::Attract]);
        assert_eq!(
            simulate(p, &[InputState::default()], &Map::default(), 1),
            Err(SimulationError::UnsupportedState(PlayerState::Attract))
        );
    }
    #[test]
    fn resume_transition_creeps_the_player_into_the_room_like_the_coroutine() {
        // `3-CelestialResort|0|roof07|42489`: the first replayable row of the segment is already a
        // few creep frames into the transition, with the collider still outside the destination
        // room's left bound. The trace exports only `Level.Transitioning`, so the simulator derives
        // `playerTo` the way `Level.TransitionRoutine` did (`Level.cs:1521-1528`).
        let map = Map {
            bounds: Rect::new(100.0, 0.0, 320.0, 184.0),
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(98.0, 100.0),
            speed: Vec2::new(323.3328857421875, -59.999908447265625),
            movement_remainder: Vec2::new(0.4169921875, 0.0),
            current_room_bounds: Some(map.bounds),
            ..PlayerSnapshot::default()
        };
        let mut simulator = Simulator::new(player, &map).unwrap();

        assert!(simulator.resume_transition(1.0 / 60.0));
        // The player entered from the left (`Level.EnforceBounds`' `player.Left < bounds.Left`,
        // `Level.cs:2738`), and `playerTo` is the first `IsInBounds(playerTo, Vector2.UnitX * 4f)`
        // position, i.e. `bounds.Left + 4` (`Level.cs:2850-2862`).
        assert_eq!(
            simulator.snapshot().transition_direction,
            Vec2::new(1.0, 0.0)
        );
        assert_eq!(
            simulator.snapshot().transition_target,
            Vec2::new(104.0, 100.0)
        );
        assert!(
            !simulator.resume_transition(1.0 / 60.0),
            "an already running transition is left alone"
        );

        let mut positions = Vec::new();
        for _ in 0..6 {
            simulator.step(InputState::default()).unwrap();
            positions.push(simulator.snapshot().pos.x);
        }
        // `Player.TransitionTo` moves `60f * Engine.DeltaTime` = exactly one pixel per frame at
        // 60 Hz, and the frame that lands on the target zeroes the remainders and rounds `Speed`
        // (`Player.cs:1570-1576`) - the same three quantities the trace shows.
        assert_eq!(positions, vec![99.0, 100.0, 101.0, 102.0, 103.0, 104.0]);
        assert_eq!(simulator.snapshot().speed, Vec2::new(323.0, -60.0));
        assert_eq!(simulator.snapshot().movement_remainder, Vec2::default());
        // `Player.Update` still has not run: the coroutine's camera clock keeps it alive, and the
        // frame that reaches the target is not the frame the transfer completes on.
        assert!(simulator.snapshot().transition_timer > 0.0);
        assert_eq!(simulator.snapshot().current_room_bounds, Some(map.bounds));

        // The trace's `transitioning` bit says when the coroutine is over; `finish_transition`
        // lands the transfer on that frame (`player.OnTransition`, `Level.cs:1624-1626`).
        simulator.finish_transition();
        simulator.step(InputState::default()).unwrap();
        assert_eq!(simulator.snapshot().transition_timer, 0.0);
        assert_eq!(simulator.snapshot().current_room_bounds, Some(map.bounds));
        assert_eq!(simulator.snapshot().wall_slide_timer, WALL_SLIDE_TIME);
        assert_eq!(simulator.snapshot().stamina, 110.0);
    }

    #[test]
    fn resume_transition_derives_the_target_from_the_room_not_the_player_position() {
        // `Level.TransitionRoutine`'s search walks `playerTo` from the position the coroutine first
        // ran on (`Level.cs:1521-1528`), which a replayed window never sees - that row belongs to the
        // segment for the room being left. The target the engine lands on is the bound being
        // entered, adjusted by `dirPad`, and the creep only ever moves whole pixels, so every row of
        // the creep derives the same target whichever starting position it is given.
        let map = Map {
            bounds: Rect::new(100.0, 0.0, 320.0, 184.0),
            ..Map::default()
        };
        let mut targets = Vec::new();
        for offset in 0..4 {
            let player = PlayerSnapshot {
                pos: Vec2::new(98.25 + offset as f32, 100.0),
                current_room_bounds: Some(map.bounds),
                ..PlayerSnapshot::default()
            };
            let mut simulator = Simulator::new(player, &map).unwrap();
            assert!(simulator.resume_transition(1.0 / 60.0));
            targets.push(simulator.snapshot().transition_target.x);
            // ... and the player's current position is not consulted for it.
            assert_eq!(simulator.snapshot().transition_target.y, 100.0);
        }
        // `bounds.Left + 4f`, the first `IsInBounds(playerTo, Vector2.UnitX * 4f)` position
        // (`Level.cs:2850-2862`), for all four starting positions.
        assert_eq!(targets, vec![104.0, 104.0, 104.0, 104.0]);

        let parked = PlayerSnapshot {
            pos: Vec2::new(200.0, 100.0),
            speed: Vec2::new(323.0, -60.0),
            current_room_bounds: Some(map.bounds),
            ..PlayerSnapshot::default()
        };
        let mut simulator = Simulator::new(parked, &map).unwrap();
        assert!(simulator.resume_transition(1.0 / 60.0));
        assert_eq!(
            simulator.snapshot().transition_direction,
            Vec2::default(),
            "a collider already inside the room has no entry direction"
        );
        assert_eq!(simulator.snapshot().transition_target, Vec2::new(200.0, 100.0));
        simulator.step(InputState::default()).unwrap();
        assert_eq!(simulator.snapshot().pos, Vec2::new(200.0, 100.0));
        assert_eq!(simulator.snapshot().speed, Vec2::new(323.0, -60.0));
    }

    #[test]
    fn upward_screen_transition_applies_source_launch_and_completion_refills() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            transition_rooms: vec![Rect::new(0.0, -184.0, 320.0, 184.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(160.0, 4.0),
            speed: Vec2::new(80.0, -160.0),
            dashes: 0,
            stamina: 20.0,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 42], &map, 42).unwrap();
        assert_eq!(trace.states[1].transition_direction, Vec2::new(0.0, -1.0));
        assert_eq!(trace.states[1].speed, Vec2::new(0.0, JUMP_SPEED));
        assert_eq!(trace.states[1].dashes, 0);
        let completed = trace
            .states
            .iter()
            .position(|state| state.current_room_bounds == Some(map.transition_rooms[0]))
            .unwrap();
        assert_eq!(completed, 41);
        assert_eq!(trace.states[completed].pos.y, -5.0);
        assert_eq!(trace.states[completed].dashes, 1);
        assert_eq!(trace.states[completed].stamina, 110.0);
        assert_eq!(trace.states[completed].wall_slide_timer, WALL_SLIDE_TIME);
        assert_eq!(trace.states[completed].jump_grace_timer, 0.0);
    }

    #[test]
    fn transition_initializes_destination_falling_block_runtime_before_next_entity_update() {
        let lower = Rect::new(0.0, 0.0, 320.0, 184.0);
        let upper = Rect::new(0.0, -184.0, 320.0, 184.0);
        let falling_block = crate::Entity {
            kind: crate::EntityKind::FallingBlock,
            bounds: Rect::new(240.0, -80.0, 24.0, 16.0),
            direction: Vec2::new(1.0, 0.0),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "fallingBlock".to_owned(),
        };
        let mut map = Map {
            bounds: lower,
            transition_rooms: vec![upper],
            transition_runtime: vec![
                crate::RoomRuntime {
                    bounds: lower,
                    spawns: vec![Vec2::new(160.0, 160.0)],
                    solids: vec![],
                    entities: vec![],
                    load_seed: 0,
                },
                crate::RoomRuntime {
                    bounds: upper,
                    spawns: vec![Vec2::new(160.0, -24.0)],
                    solids: vec![],
                    entities: vec![falling_block.clone()],
                    load_seed: 0,
                },
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(160.0, 4.0),
            speed: Vec2::new(0.0, -160.0),
            ..PlayerSnapshot::default()
        };

        let trace = simulate_trace(player, &[InputState::default(); 3], &map, 3).unwrap();

        assert_eq!(trace.states[1].falling_blocks.len(), 1);
        assert_eq!(
            trace.states[1].falling_blocks[0].position,
            Vec2::new(falling_block.bounds.x, falling_block.bounds.y)
        );
        assert_eq!(trace.states[3].falling_blocks.len(), 1);
    }

    #[test]
    fn bubsdrop_wall_jump_misses_upper_jumpthru_and_restores_old_room_spawn_set() {
        let lower = Rect::new(0.0, 0.0, 320.0, 184.0);
        let upper = Rect::new(0.0, -184.0, 320.0, 184.0);
        let mut map = Map {
            bounds: lower,
            transition_rooms: vec![upper],
            transition_runtime: vec![
                crate::RoomRuntime {
                    bounds: lower,
                    spawns: vec![Vec2::new(24.0, 32.0), Vec2::new(280.0, 32.0)],
                    solids: vec![],
                    entities: vec![],
                    load_seed: 0,
                },
                crate::RoomRuntime {
                    bounds: upper,
                    spawns: vec![Vec2::new(160.0, -16.0)],
                    // The upward transition ends beside this wall. A normal
                    // auto-jump lands on the JumpThru; the wall jump below
                    // instead sends the player left and back into `lower`.
                    solids: vec![Rect::new(168.0, -80.0, 8.0, 80.0)],
                    entities: vec![crate::Entity {
                        kind: crate::EntityKind::JumpThru,
                        bounds: Rect::new(160.0, -24.0, 40.0, 8.0),
                        direction: Vec2::default(),
                        shielded: false,
                        single_use: false,
                        nodes: vec![],
                        name: "bubsdropJumpThru".to_owned(),
                    }],
                    load_seed: 0,
                },
            ],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(164.0, 4.0),
            speed: Vec2::new(0.0, -160.0),
            dashes: 0,
            stamina: 20.0,
            ..PlayerSnapshot::default()
        };
        let baseline =
            simulate_trace(player.clone(), &[InputState::default(); 120], &map, 120).unwrap();
        assert!(baseline.states.iter().any(|state| {
            state.current_room_bounds == Some(upper) && state.on_ground && state.pos.y == -24.0
        }));
        let inputs = (0..360)
            .map(|frame| InputState {
                // State 41 is the first normal-update frame after the
                // transition coroutine calls Player.OnTransition.
                jump_pressed: frame == 41,
                jump_held: (41..51).contains(&frame),
                ..InputState::default()
            })
            .collect::<Vec<_>>();
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();

        assert_eq!(trace.states[1].speed, Vec2::new(0.0, JUMP_SPEED));
        assert!(trace.states.iter().any(|state| {
            state.current_room_bounds == Some(upper)
                && state.speed == Vec2::new(-WALL_JUMP_H, JUMP_SPEED)
        }));
        assert!(trace.states.iter().any(|state| {
            state.current_room_bounds == Some(lower) && state.transition_room_bounds.is_none()
        }));
        assert!(trace.states.iter().any(|state| {
            state.state == PlayerState::IntroRespawn && state.pos == Vec2::new(24.0, 32.0)
        }));
    }

    #[test]
    fn climb_jump_buffer_uses_the_real_five_frame_transition_boundary() {
        let upper = Rect::new(0.0, -184.0, 320.0, 184.0);
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            transition_rooms: vec![upper],
            solids: vec![Rect::new(168.0, -16.0, 8.0, 16.0)],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(164.0, 4.0),
            speed: Vec2::new(80.0, -160.0),
            stamina: 20.0,
            ..PlayerSnapshot::default()
        };
        let inputs = |press_frame| {
            (0..43)
                .map(|frame| InputState {
                    move_x: 1,
                    jump_pressed: frame == press_frame,
                    jump_held: frame >= press_frame,
                    grab_held: frame >= press_frame,
                    ..InputState::default()
                })
                .collect::<Vec<_>>()
        };
        let on_time_inputs = inputs(37);
        let on_time = simulate_trace(
            player.clone(),
            &on_time_inputs,
            &map,
            on_time_inputs.len() as u32,
        )
        .unwrap();
        let early_inputs = inputs(36);
        let early = simulate_trace(player, &early_inputs, &map, early_inputs.len() as u32).unwrap();

        assert_eq!(on_time.states[41].current_room_bounds, Some(upper));
        assert_eq!(on_time.states[42].stamina, 82.5);
        assert_eq!(on_time.states[42].speed.x, 0.0);
        assert!(on_time.states[42].wall_speed_retained > 0.0);
        assert_eq!(on_time.states[42].jump_buffer_timer, 0.0);
        assert_eq!(early.states[42].stamina, 110.0);
        assert_eq!(early.states[42].speed.x, AIR_MULT * RUN_ACCEL * DT);
    }

    #[test]
    fn kermit_dash_preserves_attack_and_direction_through_vertical_transition() {
        let upper = Rect::new(0.0, -184.0, 320.0, 184.0);
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            transition_rooms: vec![upper],
            entities: vec![crate::Entity {
                kind: EntityKind::FlyFeather,
                bounds: Rect::new(150.0, -40.0, 20.0, 20.0),
                direction: Vec2::default(),
                shielded: true,
                single_use: false,
                nodes: vec![],
                name: "infiniteStar".to_owned(),
            }],
            ..Map::default()
        };
        let player = PlayerSnapshot {
            pos: Vec2::new(160.0, 4.0),
            speed: Vec2::new(0.0, -240.0),
            state: PlayerState::Dash,
            dash_dir: Vec2::new(0.0, -1.0),
            dash_attack_timer: 0.3,
            state_timer: 0.1,
            dashes: 0,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(player, &[InputState::default(); 48], &map, 48).unwrap();

        assert_eq!(trace.states[1].state, PlayerState::Normal);
        assert_eq!(trace.states[1].transition_direction, Vec2::new(0.0, -1.0));
        assert_eq!(trace.states[1].dash_dir, Vec2::new(0.0, -1.0));
        assert!(trace.states[1].dash_attack_timer > 0.28);
        let completed = trace
            .states
            .iter()
            .position(|state| state.current_room_bounds == Some(upper))
            .unwrap();
        assert_eq!(completed, 41);
        assert_eq!(trace.states[completed].dash_dir, Vec2::new(0.0, -1.0));
        assert!(trace.states[completed].dash_attack_timer > 0.28);
        let hit_index = trace
            .states
            .iter()
            .position(|state| state.state == PlayerState::StarFly)
            .unwrap();
        assert!(hit_index > completed);
        let hit = &trace.states[hit_index];
        assert_eq!(hit.state, PlayerState::StarFly);
        assert_eq!(hit.dashes, 1);
        assert_eq!(hit.stamina, 110.0);
        assert!(!hit.on_ground);
        assert!(!hit.ducking);
        assert!(!hit.dead);
    }

    #[test]
    fn downward_screen_transition_clamps_upward_speed_before_transfer() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            transition_rooms: vec![Rect::new(0.0, 184.0, 320.0, 184.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(160.0, 185.0),
            speed: Vec2::new(30.0, -40.0),
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &map, 1).unwrap();
        assert_eq!(p.transition_direction, Vec2::new(0.0, 1.0));
        assert_eq!(p.state, PlayerState::Normal);
        assert_eq!(p.speed.y, 0.0);
        assert_eq!(p.transition_target.y, 196.0);
    }
    #[test]
    fn wind_trigger_selects_a_persistent_next_frame_target() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 64.0),
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 8], &wind_map(), 8).unwrap();
        assert_eq!(trace.states[1].wind, Vec2::default());
        assert_eq!(trace.states[1].wind_target, Vec2::new(400.0, 0.0));
        assert!((trace.states[2].wind.x - WIND_ACCEL * DT).abs() < 0.001);
        assert_eq!(trace.states[8].speed.x, 0.0);
        assert!(trace.states[8].pos.x > 32.0);
    }
    #[test]
    fn grounded_ducking_blocks_horizontal_wind_movement() {
        let mut p = grounded_player();
        p.ducking = true;
        let p = simulate(
            p,
            &[InputState {
                move_y: 1,
                ..InputState::default()
            }; 30],
            &wind_map(),
            30,
        )
        .unwrap();
        assert_eq!(p.wind, Vec2::new(400.0, 0.0));
        assert_eq!(p.pos.x, 32.0);
        assert_eq!(p.speed.x, 0.0);
    }

    #[test]
    fn feather_transform_launches_on_the_source_coroutine_frame() {
        let p = PlayerSnapshot {
            pos: Vec2::new(120.0, 200.0),
            ..PlayerSnapshot::default()
        };
        let inputs = [InputState {
            move_x: 1,
            ..InputState::default()
        }; 28];
        let trace = simulate_trace(p, &inputs, &feather_map(false), 28).unwrap();
        assert_eq!(trace.states[1].state, PlayerState::StarFly);
        assert!(trace.states[27].star_fly_transforming);
        assert!(!trace.states[28].star_fly_transforming);
        assert_eq!(trace.states[28].speed, Vec2::new(250.0, 0.0));
        assert_eq!(trace.states[28].pos, Vec2::new(124.0, 200.0));
    }

    #[test]
    fn featherboost_uses_the_first_live_diagonal_for_the_250_start_speed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(120.0, 200.0),
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState::default(); 28];
        inputs[27].move_x = 1;
        inputs[27].move_y = -1;
        let trace = simulate_trace(p, &inputs, &feather_map(false), 28).unwrap();
        let launched = &trace.states[28];
        assert_eq!(launched.state, PlayerState::StarFly);
        assert!(!launched.star_fly_transforming);
        assert!((length(launched.speed) - STAR_FLY_START_SPEED).abs() < 0.001);
        assert!((launched.speed.x - 176.776_69).abs() < 0.001);
        assert!((launched.speed.y + 176.776_69).abs() < 0.001);
    }

    #[test]
    fn feather_super_jumps_from_grounded_horizontal_starfly_speed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(900.0, 496.0),
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState::default(); 50];
        for (frame, input) in inputs.iter_mut().enumerate() {
            input.move_x = 1;
            input.jump_pressed = frame == 28;
            input.jump_held = (28..40).contains(&frame);
        }
        let trace = simulate_trace(p, &inputs, &crate::mechanics_playground(), 50).unwrap();
        let jumped = &trace.states[29];
        assert_eq!(jumped.state, PlayerState::Normal);
        assert!((jumped.speed.x - 273.333_34).abs() < 0.001);
        assert_eq!(jumped.speed.y, JUMP_SPEED);
        assert_eq!(jumped.var_jump_timer, VAR_JUMP_TIME);
    }

    #[test]
    fn cassette_raise_uses_separate_will_toggle_and_activation_pixels() {
        let p = PlayerSnapshot {
            state: PlayerState::Frozen,
            cassette_manager: crate::CassetteManagerSnapshot {
                initialized: true,
                startup_music_pending: false,
                beat_timer: CASSETTE_BEAT_INTERVAL - DT * 0.5,
                beat_index: 6,
                current_index: 1,
                max_beat: 2,
                tempo_mult: 1.0,
                tape_taken: false,
            },
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 12], &cassette_map(), 12).unwrap();
        let warned = &trace.states[1];
        assert_eq!(warned.cassette_manager.beat_index, 7);
        assert_eq!(warned.cassette_blocks[0].position.y, 102.0);
        assert_eq!(warned.cassette_blocks[1].position.y, 102.0);
        assert!(!warned.cassette_blocks[0].collidable);
        assert!(warned.cassette_blocks[1].collidable);

        let activated = &trace.states[12];
        assert_eq!(activated.cassette_manager.beat_index, 8);
        assert_eq!(activated.cassette_blocks[0].position.y, 101.0);
        assert_eq!(activated.cassette_blocks[1].position.y, 103.0);
        assert!(activated.cassette_blocks[0].collidable);
        assert!(!activated.cassette_blocks[1].collidable);
    }

    #[test]
    fn disappearing_cassette_cornerboost_restores_retained_speed_after_entity_phase() {
        let p = PlayerSnapshot {
            // The player's right edge is four pixels left of cassette index 0.
            // Frame one collides, then the beat-8 activation change is written
            // after Player.Update. CassetteBlock.Update clears collision at
            // the next frame's pre-Player entity phase, so retained speed
            // only returns on the third player update.
            pos: Vec2::new(60.0, 112.0),
            speed: Vec2::new(120.0, 0.0),
            cassette_manager: crate::CassetteManagerSnapshot {
                initialized: true,
                startup_music_pending: false,
                beat_timer: CASSETTE_BEAT_INTERVAL - DT * 0.5,
                beat_index: 7,
                current_index: 0,
                max_beat: 2,
                tempo_mult: 1.0,
                tape_taken: false,
            },
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 3], &cassette_map(), 3).unwrap();
        assert_eq!(trace.states[1].speed.x, 0.0);
        assert!(trace.states[1].wall_speed_retention_timer > 0.05);
        assert!(!trace.states[1].cassette_blocks[0].activated);
        assert!(trace.states[1].cassette_blocks[0].collidable);
        assert!(!trace.states[2].cassette_blocks[0].collidable);
        assert_eq!(trace.states[2].wall_speed_retention_timer, 0.0);
        assert!(trace.states[2].speed.x > 90.0);
    }

    #[test]
    fn disappearing_cassette_cornerboost_fixture_times_hit_clear_and_refund() {
        // This is the generated candidate's timing in a compact map: input
        // 27 hits the initially-active index 1 wall, manager activation then
        // disables it, input 28's pre-Player entity phase clears it, and the
        // following Player.Update restores the retained 90-speed run.
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            solids: vec![Rect::new(0.0, 496.0, 960.0, 48.0)],
            entities: vec![
                crate::Entity {
                    kind: crate::EntityKind::CassetteBlock,
                    bounds: Rect::new(320.0, 400.0, 64.0, 16.0),
                    direction: Vec2::new(0.0, 3.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "cassetteBlock".to_owned(),
                },
                crate::Entity {
                    kind: crate::EntityKind::CassetteBlock,
                    bounds: Rect::new(128.0, 448.0, 32.0, 48.0),
                    direction: Vec2::new(1.0, 3.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "cassetteBlock".to_owned(),
                },
            ],
            ..Map::default()
        };
        let mut inputs = [InputState::default(); 36];
        for input in &mut inputs[22..] {
            input.move_x = 1;
        }
        let trace = simulate_trace(
            PlayerSnapshot {
                pos: Vec2::new(120.0, 496.0),
                on_ground: true,
                ..PlayerSnapshot::default()
            },
            &inputs,
            &map,
            inputs.len() as u32,
        )
        .unwrap();
        assert_eq!(trace.states[28].pos, Vec2::new(124.0, 496.0));
        assert_eq!(trace.states[28].speed.x, 0.0);
        assert!(trace.states[28].cassette_blocks[1].collidable);
        assert!(!trace.states[29].cassette_blocks[1].collidable);
        assert_eq!(trace.states[30].pos, Vec2::new(127.0, 496.0));
        assert_eq!(trace.states[30].speed.x, 90.0);
        assert_eq!(trace.states[30].wall_speed_retention_timer, 0.0);
    }

    #[test]
    fn fresh_custom_cassette_manager_skips_music_advance_on_its_first_update() {
        let p = PlayerSnapshot {
            pos: Vec2::new(96.0, 106.0),
            state: PlayerState::Frozen,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 82], &cassette_map(), 82).unwrap();

        assert!(trace.states[0].cassette_manager.startup_music_pending);
        assert!(!trace.states[1].cassette_manager.startup_music_pending);
        assert_eq!(trace.states[1].cassette_manager.beat_timer, 0.0);
        assert_eq!(trace.states[81].pos.y, 106.0);
        assert_eq!(trace.states[82].pos.y, 101.0);
    }

    #[test]
    fn cassette_reform_wiggles_player_four_pixels_then_carries_one() {
        let p = PlayerSnapshot {
            pos: Vec2::new(96.0, 106.0),
            state: PlayerState::Frozen,
            cassette_manager: crate::CassetteManagerSnapshot {
                initialized: true,
                current_index: 0,
                max_beat: 2,
                tempo_mult: 1.0,
                ..crate::CassetteManagerSnapshot::default()
            },
            cassette_blocks: vec![
                crate::CassetteBlockSnapshot {
                    position: Vec2::new(64.0, 102.0),
                    start: Vec2::new(64.0, 101.0),
                    width: 64.0,
                    height: 16.0,
                    index: 0,
                    activated: true,
                    collidable: false,
                },
                crate::CassetteBlockSnapshot {
                    position: Vec2::new(192.0, 103.0),
                    start: Vec2::new(192.0, 101.0),
                    width: 64.0,
                    height: 16.0,
                    index: 1,
                    activated: false,
                    collidable: false,
                },
            ],
            ..PlayerSnapshot::default()
        };
        let result = simulate(p, &[InputState::default()], &cassette_map(), 1).unwrap();
        assert_eq!(result.pos.y, 101.0);
        assert_eq!(result.cassette_blocks[0].position.y, 101.0);
        assert!(result.cassette_blocks[0].collidable);
        assert!((result.last_lift_speed.y + 60.0).abs() < 0.001);
    }

    #[test]
    fn cassoosted_fuper_combines_grounded_starfly_jump_and_same_frame_reform() {
        let mut map = cassette_map();
        map.solids.push(Rect::new(0.0, 106.0, 320.0, 8.0));
        let p = PlayerSnapshot {
            pos: Vec2::new(96.0, 106.0),
            speed: Vec2::new(250.0, 0.0),
            state: PlayerState::StarFly,
            star_fly_timer: 1.0,
            star_fly_speed_lerp: 1.0,
            star_fly_last_dir: Vec2::new(1.0, 0.0),
            cassette_manager: crate::CassetteManagerSnapshot {
                initialized: true,
                current_index: 0,
                max_beat: 2,
                tempo_mult: 1.0,
                ..crate::CassetteManagerSnapshot::default()
            },
            cassette_blocks: vec![
                crate::CassetteBlockSnapshot {
                    position: Vec2::new(64.0, 102.0),
                    start: Vec2::new(64.0, 101.0),
                    width: 64.0,
                    height: 16.0,
                    index: 0,
                    activated: true,
                    collidable: false,
                },
                crate::CassetteBlockSnapshot {
                    position: Vec2::new(192.0, 103.0),
                    start: Vec2::new(192.0, 101.0),
                    width: 64.0,
                    height: 16.0,
                    index: 1,
                    activated: false,
                    collidable: false,
                },
            ],
            ..PlayerSnapshot::default()
        };
        let result = simulate(
            p,
            &[InputState {
                move_x: 1,
                jump_pressed: true,
                jump_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert_eq!(result.state, PlayerState::Normal);
        assert!((result.speed.x - 273.333_34).abs() < 0.001);
        assert!((result.speed.y - (JUMP_SPEED - 60.0)).abs() < 0.001);
        assert!(result.cassette_blocks[0].collidable);
        assert_eq!(result.cassette_blocks[0].position.y, 101.0);
        assert_eq!(result.pos.y, 98.0);
    }

    #[test]
    fn cassoosted_fuper_fixture_consumes_reform_lift_on_next_player_update() {
        // Mirror the candidate MapPart rather than pre-seeding StarFly. The
        // fresh custom manager skips its first AdvanceMusic call; with tempo
        // three, beat 8 writes Activated on input 27 and CassetteBlock.Update
        // reforms during input 28, after Player.Update. That movement writes
        // LiftSpeed, which the grounded StarFly Jump consumes next frame.
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            solids: vec![Rect::new(0.0, 496.0, 960.0, 48.0)],
            entities: vec![
                crate::Entity {
                    kind: crate::EntityKind::FlyFeather,
                    bounds: Rect::new(340.0, 474.0, 20.0, 20.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "infiniteStar".to_owned(),
                },
                crate::Entity {
                    kind: crate::EntityKind::CassetteBlock,
                    bounds: Rect::new(304.0, 493.0, 384.0, 16.0),
                    direction: Vec2::new(0.0, 3.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "cassetteBlock".to_owned(),
                },
                crate::Entity {
                    kind: crate::EntityKind::CassetteBlock,
                    bounds: Rect::new(720.0, 400.0, 64.0, 16.0),
                    direction: Vec2::new(1.0, 3.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "cassetteBlock".to_owned(),
                },
            ],
            ..Map::default()
        };
        let mut inputs = [InputState::default(); 40];
        for (frame, input) in inputs.iter_mut().enumerate() {
            input.move_x = 1;
            input.jump_pressed = frame == 28;
            input.jump_held = (28..40).contains(&frame);
        }
        let trace = simulate_trace(
            PlayerSnapshot {
                pos: Vec2::new(350.0, 496.0),
                on_ground: true,
                ..PlayerSnapshot::default()
            },
            &inputs,
            &map,
            inputs.len() as u32,
        )
        .unwrap();
        let fuper = trace
            .states
            .iter()
            .position(|state| {
                state.state == PlayerState::Normal
                    && (state.speed.x - 273.333_34).abs() < 0.001
                    && (state.speed.y - (JUMP_SPEED - 60.0)).abs() < 0.001
            })
            .expect("first controllable StarFly frame should produce a Feather Super");
        let reform = trace
            .states
            .iter()
            .position(|state| {
                state.cassette_blocks[0].collidable && state.cassette_blocks[0].position.y == 493.0
            })
            .expect("tempo-three cassette should reform");
        assert_eq!(reform, 29);
        assert_eq!(fuper, reform);
        assert!((trace.states[reform].last_lift_speed.y + 60.0).abs() < 0.001);
        assert!((trace.states[fuper].speed.y - (JUMP_SPEED - 60.0)).abs() < 0.001);
        assert!(trace.states[fuper].pos.y < 496.0);
    }

    #[test]
    fn cassette_manager_keeps_advancing_during_room_transition() {
        let p = PlayerSnapshot {
            state: PlayerState::Frozen,
            transition_timer: 0.5,
            transition_direction: Vec2::new(1.0, 0.0),
            transition_target: Vec2::new(300.0, 100.0),
            cassette_manager: crate::CassetteManagerSnapshot {
                initialized: true,
                startup_music_pending: false,
                beat_timer: CASSETTE_BEAT_INTERVAL - DT * 0.5,
                beat_index: 6,
                current_index: 1,
                max_beat: 2,
                tempo_mult: 1.0,
                tape_taken: false,
            },
            ..PlayerSnapshot::default()
        };
        let result = simulate(p, &[InputState::default()], &cassette_map(), 1).unwrap();
        assert_eq!(result.cassette_manager.beat_index, 7);
        assert_eq!(result.cassette_blocks[0].position.y, 102.0);
        assert_eq!(result.cassette_blocks[1].position.y, 102.0);
    }

    #[test]
    fn transition_loads_destination_cassettes_before_same_frame_will_toggle() {
        let mut map = cassette_map();
        let next = Rect::new(0.0, -184.0, 320.0, 184.0);
        map.transition_rooms = vec![next];
        map.transition_runtime = vec![crate::RoomRuntime {
            bounds: next,
            spawns: vec![Vec2::new(24.0, -16.0), Vec2::new(280.0, -16.0)],
            solids: vec![Rect::new(0.0, -8.0, 320.0, 8.0)],
            entities: vec![
                crate::Entity {
                    kind: crate::EntityKind::CassetteBlock,
                    bounds: Rect::new(64.0, -48.0, 64.0, 16.0),
                    direction: Vec2::new(0.0, 1.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "cassetteBlock".to_owned(),
                },
                crate::Entity {
                    kind: crate::EntityKind::CassetteBlock,
                    bounds: Rect::new(192.0, -48.0, 64.0, 16.0),
                    direction: Vec2::new(1.0, 1.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "cassetteBlock".to_owned(),
                },
            ],
            load_seed: 0,
        }];
        let mut p = PlayerSnapshot {
            pos: Vec2::new(250.0, -172.0),
            state: PlayerState::Frozen,
            current_room_bounds: Some(map.bounds),
            cassette_manager: crate::CassetteManagerSnapshot {
                initialized: true,
                startup_music_pending: false,
                beat_timer: CASSETTE_BEAT_INTERVAL - DT * 0.5,
                beat_index: 6,
                current_index: 1,
                max_beat: 2,
                tempo_mult: 1.0,
                tape_taken: false,
            },
            ..PlayerSnapshot::default()
        };
        let mut attachments = initialize_static_mover_attachments(&map);
        begin_transition(
            &mut p,
            &mut map,
            next,
            Vec2::new(0.0, -1.0),
            &mut attachments,
            &mut RoomCoroutineState {
                crumble_blocks: Vec::new(),
                floaty_blocks: Vec::new(),
                switch_gates: Vec::new(),
                touch_switches: Vec::new(),
                switches_on: false,
            },
        );

        // LoadLevel's OnLevelStart is silent: the new index-0 block begins
        // inactive at +2 px while the destination block matching the retained
        // current index begins at its source position. The manager has not
        // advanced yet.
        assert_eq!(p.cassette_blocks[0].position.y, -46.0);
        assert!(!p.cassette_blocks[0].collidable);
        assert_eq!(p.cassette_blocks[1].position.y, -48.0);
        assert!(p.cassette_blocks[1].collidable);

        step(
            &mut p,
            InputState::default(),
            &mut map,
            &mut attachments,
            &mut None,
            &mut Vec::new(),
            &mut RoomCoroutineState {
                crumble_blocks: Vec::new(),
                floaty_blocks: Vec::new(),
                switch_gates: Vec::new(),
                touch_switches: Vec::new(),
                switches_on: false,
            },
        )
        .unwrap();
        assert_eq!(p.cassette_manager.beat_index, 7);
        // The same scene frame now runs CassetteBlockManager.WillToggle:
        // inactive index 0 moves up one pixel, while active index 1 moves
        // down one pixel. Both land in their opposite one-pixel phase.
        assert_eq!(p.cassette_blocks[0].position.y, -47.0);
        assert_eq!(p.cassette_blocks[1].position.y, -47.0);
    }

    #[test]
    fn spinner_proximity_check_enables_collision_after_player_callback_phase() {
        let p = PlayerSnapshot {
            pos: Vec2::new(100.0, 100.0),
            state: PlayerState::Frozen,
            scene_time_active: 0.04,
            spinners: vec![crate::SpinnerSnapshot {
                position: Vec2::new(100.0, 100.0),
                offset: 0.0,
                visible: true,
                collidable: false,
            }],
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 2], &spinner_map(), 2).unwrap();
        assert!(!trace.states[1].dead);
        assert!(trace.states[1].spinners[0].collidable);
        assert!(trace.states[2].dead);
    }

    #[test]
    fn visible_spinner_collision_survives_one_frame_web_simulation_segments() {
        let player = PlayerSnapshot {
            pos: Vec2::new(100.0, 100.0),
            state: PlayerState::Frozen,
            spinners: vec![crate::SpinnerSnapshot {
                position: Vec2::new(100.0, 100.0),
                offset: 0.0,
                visible: true,
                collidable: true,
            }],
            ..PlayerSnapshot::default()
        };
        let result = simulate(player, &[InputState::default()], &spinner_map(), 1).unwrap();
        assert!(result.dead);
    }

    #[test]
    fn intro_respawn_tween_returns_control_after_source_point_six_seconds() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            spawn: Vec2::new(32.0, 152.0),
            solids: vec![Rect::new(0.0, 152.0, 320.0, 28.0)],
            ..Map::default()
        };
        let dead = PlayerSnapshot {
            dead: true,
            respawn_frames: 1,
            ..PlayerSnapshot::default()
        };
        let respawned = simulate(dead, &[InputState::default()], &map, 1).unwrap();
        assert_eq!(respawned.state, PlayerState::IntroRespawn);
        // `IntroRespawnBegin` starts the tween at `Timer = 0`
        // (Player.cs:6131); the simulator keeps that clock in
        // `PlayerSnapshot::intro_timer` and advances it by DeltaTime per
        // update, matching `Monocle.Tween.Update`.
        assert_eq!(respawned.intro_timer, 0.0);
        assert_eq!(respawned.intro_phase, INTRO_PHASE_RESPAWN);

        let intro = simulate(respawned, &[InputState::default(); 35], &map, 35).unwrap();
        assert_eq!(intro.state, PlayerState::IntroRespawn);
        let ready = simulate(intro, &[InputState::default()], &map, 1).unwrap();
        assert_eq!(ready.state, PlayerState::Normal);
        let moving = simulate(
            ready,
            &[InputState {
                move_x: 1,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert!(moving.speed.x > 0.0);
    }

    /// `Level.DefaultSpawnPoint` picks the spawn closest to the room's
    /// bottom-left corner (`Level.cs:290`, `Session.cs:256`).
    fn intro_walk_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            spawn: Vec2::new(24.0, 152.0),
            room_spawns: vec![Vec2::new(24.0, 152.0)],
            solids: vec![Rect::new(0.0, 152.0, 320.0, 32.0)],
            ..Map::default()
        }
    }

    /// A real decoded room: the bottom tile row is solid and `LevelLoader`
    /// bleeds it three cells downward, so the intro jump's below-the-room
    /// settle stays grounded (`LevelLoader.cs:233-249`).
    fn intro_jump_map() -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 184.0),
            spawn: Vec2::new(24.0, 144.0),
            room_spawns: vec![Vec2::new(24.0, 144.0)],
            solids: vec![Rect::new(0.0, 144.0, 320.0, 40.0)],
            tile_grid: (0..23)
                .map(|row| {
                    if row >= 18 {
                        "1".repeat(40)
                    } else {
                        "0".repeat(40)
                    }
                })
                .collect(),
            ..Map::default()
        }
    }

    #[test]
    fn room_edge_tile_bleed_extends_the_bottom_row_outward() {
        let mut map = intro_jump_map();
        let before = map.solids.len();
        add_room_edge_tile_bleed(&mut map);
        // Row 22 is the room's solid bottom row; rows 23..25 (y 184..208) copy
        // it, and the five solid cells of each side column copy outward too.
        assert!(map.solids.contains(&Rect::new(0.0, 184.0, 8.0, 8.0)));
        assert!(map.solids.contains(&Rect::new(312.0, 200.0, 8.0, 8.0)));
        assert_eq!(map.solids.len(), before + 40 * 3 + 5 * 3 * 2);
    }

    #[test]
    fn intro_jump_settles_below_the_room_then_rises_to_the_source_rise_band() {
        // `IntroJumpCoroutine` (Player.cs:5995-6068): the non-Summit branch
        // writes `Y = level.Bounds.Bottom + 16` and waits 0.5 s, then rises at
        // -120 px/s until `start.Y - 8`.
        let mut map = intro_jump_map();
        let anchor = PlayerSnapshot {
            pos: Vec2::new(24.0, 200.0),
            state: PlayerState::IntroJump,
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(anchor, &[InputState::default(); 120], &map, 120).unwrap();
        // The anchor is the state's second update, so the 0.5 s settle has
        // already consumed one frame: the rise starts on step 30.
        assert_eq!(trace.states[17].state, PlayerState::IntroJump);
        assert_eq!(trace.states[17].pos.y, 200.0);
        assert_eq!(trace.states[29].pos.y, 200.0);
        // The bled bottom row keeps `Player.onGround` true while the coroutine
        // holds the player 16 px below the room (LevelLoader.cs:233-249).
        assert!(trace.states[29].player_on_ground);
        assert_eq!(trace.states[30].pos.y, 198.0);
        // `start.Y` is the spawn (144), so the rise body stops at
        // `start.Y - 8` = 136; the following `Speed.Y = -100` decay keeps
        // moving the player up through `Actor.MoveV`, reaching 131 -- exactly
        // what the real `1-ForsakenCity|0|1` trace shows.
        let lowest = trace
            .states
            .iter()
            .map(|state| state.pos.y)
            .fold(f32::INFINITY, f32::min);
        assert_eq!(lowest, 131.0);
        let landed = trace
            .states
            .iter()
            .position(|state| state.state == PlayerState::Normal)
            .expect("the coroutine releases the player");
        assert_eq!(trace.states[landed].pos, Vec2::new(24.0, 144.0));
        assert_eq!(trace.states[landed].speed, Vec2::default());
    }

    #[test]
    fn intro_walk_snaps_outside_walks_to_the_default_spawn_and_rests() {
        // `IntroWalkCoroutine` (Player.cs:5969-5993) teleports X to
        // `Bounds.Left - 16`, waits 0.3 s, walks toward the captured `start`
        // at 64 px/s, then restores `Position = start` and waits 0.2 s.
        let mut map = intro_walk_map();
        let anchor = PlayerSnapshot {
            pos: Vec2::new(-16.0, 152.0),
            state: PlayerState::IntroWalk,
            facing: true,
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(anchor, &[InputState::default(); 90], &map, 90).unwrap();
        assert_eq!(trace.states[17].pos.x, -16.0);
        // `Actor.MoveTowardsX` (Actor.cs:292-296) steps from ExactPosition, so
        // the first walk frame advances one whole pixel and banks the rest.
        assert_eq!(trace.states[18].pos.x, -15.0);
        assert!((trace.states[18].movement_remainder.x - 0.066_670_7).abs() < 1e-4);
        let restored = trace
            .states
            .iter()
            .position(|state| state.pos == Vec2::new(24.0, 152.0))
            .expect("the walk restores Position = start");
        // `while (Math.Abs(X - start.X) > 2f)` stops within two pixels.
        assert!((trace.states[restored - 1].pos.x - 24.0).abs() <= 2.0);
        // Then `yield return 0.2f` before `StateMachine.State = 0`.
        assert_eq!(trace.states[restored + 12].state, PlayerState::IntroWalk);
        assert_eq!(trace.states[restored + 13].state, PlayerState::Normal);
    }

    #[test]
    fn intro_wake_up_waits_for_the_source_wake_up_animation_frames() {
        // `IntroWakeUpCoroutine` (Player.cs:6112-6119) waits 0.5 s, then awaits
        // `Sprite.PlayRoutine("wakeUp")`. `Sprites.xml:72` defines that
        // animation as 24 frames at 0.1 s, and `Monocle.Sprite.Update`
        // advances one frame per 0.1 s, so the routine clears after
        // 24 * 6 = 144 updates.
        let mut map = intro_walk_map();
        let anchor = PlayerSnapshot {
            pos: Vec2::new(24.0, 152.0),
            state: PlayerState::IntroWakeUp,
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(anchor, &[InputState::default(); 200], &map, 200).unwrap();
        assert_eq!(trace.states[187].state, PlayerState::IntroWakeUp);
        assert_eq!(trace.states[188].state, PlayerState::Normal);
    }

    #[test]
    fn intro_think_for_a_bit_walks_eight_pixels_and_faces_both_ways() {
        // `IntroThinkForABitCoroutine` (Player.cs:6156-6174) nudges the camera,
        // waits 0.1 s, walks `X + 8` at 32 px/s, then alternates facing.
        let mut map = intro_walk_map();
        let anchor = PlayerSnapshot {
            pos: Vec2::new(24.0, 152.0),
            state: PlayerState::IntroThinkForABit,
            facing: false,
            on_ground: true,
            camera: Vec2::new(0.0, 0.0),
            camera_initialized: true,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(anchor, &[InputState::default(); 120], &map, 120).unwrap();
        // `Camera.X += 8f` runs on the coroutine's first update; the camera is
        // not exported by the trace and `update_camera` (which the source gates
        // on `Player.InControl`, Player.cs:1884) then eases it back.
        assert!(trace.states[1].camera.x > 0.0);
        assert_eq!(trace.states[5].pos.x, 24.0);
        assert_eq!(trace.states[6].pos.x, 25.0);
        let walked = trace
            .states
            .iter()
            .map(|state| state.pos.x)
            .fold(f32::NEG_INFINITY, f32::max);
        assert_eq!(walked, 32.0);
        assert!(trace.states.iter().any(|state| !state.facing));
        let released = trace
            .states
            .iter()
            .position(|state| state.state == PlayerState::Normal)
            .expect("the coroutine releases the player");
        assert!(trace.states[released - 1].facing);
    }

    #[test]
    fn float32_scene_clock_freezes_spinner_interval_groups() {
        // At 2^19 seconds the f32 ULP is 1/16 second, so adding 1/60 no
        // longer changes TimeActive. Subtracting each spinner's offset before
        // the interval bucket comparison still leaves distinct stable groups.
        let frozen = 524_288.0_f32;
        assert_eq!(frozen + DT, frozen);
        let hits = (0..=1000)
            .filter(|index| scene_on_interval(frozen, 0.05, *index as f32 / 1000.0))
            .count();
        assert!(hits > 0, "at least one offset group should keep firing");
        assert!(
            hits < 1001,
            "at least one offset group should remain frozen"
        );
    }

    #[test]
    fn feather_clip_exits_below_the_jumpthrough_top() {
        let p = PlayerSnapshot {
            pos: Vec2::new(160.0, 40.0),
            ..PlayerSnapshot::default()
        };
        let inputs = [InputState {
            move_y: 1,
            ..InputState::default()
        }; 180];
        let trace = simulate_trace(p, &inputs, &crate::mechanics_playground(), 180).unwrap();
        let (frame, exit) = trace
            .states
            .windows(2)
            .enumerate()
            .find(|states| {
                states.1[0].state == PlayerState::StarFly
                    && states.1[1].state == PlayerState::Normal
            })
            .expect("StarFly should expire into Normal");
        assert!(
            exit[1].pos.y >= 402.0,
            "exit frame={} pos={:?} speed={:?}",
            frame + 1,
            exit[1].pos,
            exit[1].speed
        );
        assert!(!exit[1].on_ground);
    }

    #[test]
    fn star_fly_wall_collision_uses_half_speed_bounce() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            solids: vec![Rect::new(100.0, 0.0, 8.0, 180.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(96.0, 80.0),
            speed: Vec2::new(190.0, 0.0),
            state: PlayerState::StarFly,
            star_fly_timer: 1.0,
            star_fly_speed_lerp: 1.0,
            star_fly_last_dir: Vec2::new(1.0, 0.0),
            ..PlayerSnapshot::default()
        };
        let p = simulate(
            p,
            &[InputState {
                move_x: 1,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();
        assert_eq!(p.pos.x, 96.0);
        assert_eq!(p.speed.x, -95.0);
    }

    #[test]
    fn shielded_feather_uses_source_point_bounce() {
        let p = PlayerSnapshot {
            pos: Vec2::new(120.0, 200.0),
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &feather_map(true), 1).unwrap();
        assert_eq!(p.state, PlayerState::Normal);
        assert_eq!(p.speed, Vec2::new(-120.0, -200.0));
    }

    #[test]
    fn bumper_enters_launch_and_applies_same_direction_boost_immediately() {
        let p = PlayerSnapshot {
            pos: Vec2::new(589.0, 206.0),
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            move_x: -1,
            ..InputState::default()
        };
        let p = simulate(p, &[input], &bumper_map(), 1).unwrap();
        assert_eq!(p.state, PlayerState::Launch);
        assert_eq!(p.speed, Vec2::new(-336.0, -150.0));
        assert_eq!(p.freeze_timer, 0.1);
        assert_eq!(p.explode_launch_boost_timer, 0.0);
        assert_eq!(p.bumper_reuse_timer, 0.6);
        assert!((p.last_bumper_target.x - 600.138_2).abs() < 0.000_1);
        assert!((p.last_bumper_target.y - 200.046_07).abs() < 0.000_1);
    }

    #[test]
    fn bumper_defers_horizontal_boost_when_input_is_not_held() {
        let p = PlayerSnapshot {
            pos: Vec2::new(589.0, 206.0),
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &bumper_map(), 1).unwrap();
        assert_eq!(p.state, PlayerState::Launch);
        assert_eq!(p.speed, Vec2::new(-280.0, -150.0));
        assert_eq!(p.explode_launch_boost_timer, 0.01);
        assert_eq!(p.explode_launch_boost_speed, -336.0);
    }

    #[test]
    fn bumper_replays_the_collected_sine_phase_and_position_across_split_runs() {
        let phase = std::f32::consts::FRAC_PI_2;
        let initial = PlayerSnapshot {
            bumpers: vec![crate::BumperSnapshot {
                anchor: Vec2::new(600.0, 200.0),
                // The map anchor is (600, 200); this is Bumper.UpdatePosition
                // at Counter=pi/2: (sin(counter)*3, sin(counter/2)*2).
                position: Vec2::new(603.0, 201.414_213_5),
                sine_counter: phase,
                respawn_timer: 0.0,
            }],
            ..PlayerSnapshot::default()
        };
        let inputs = vec![InputState::default(); 20];
        let whole = simulate(initial.clone(), &inputs, &bumper_map(), inputs.len() as u32).unwrap();
        let first = simulate(initial, &inputs[..7], &bumper_map(), 7).unwrap();
        let split = simulate(first, &inputs[7..], &bumper_map(), 13).unwrap();

        assert_eq!(split, whole);
        let expected_counter = phase + std::f32::consts::TAU * 0.44 * DT * 20.0;
        let expected = Vec2::new(
            600.0 + expected_counter.sin() * 3.0,
            200.0 + (expected_counter * 0.5).sin() * 2.0,
        );
        assert!((whole.bumpers[0].position.x - expected.x).abs() < 0.000_1);
        assert!((whole.bumpers[0].position.y - expected.y).abs() < 0.000_1);
    }

    #[test]
    fn bumper_clip_dashes_back_through_during_the_point_six_second_reuse_window() {
        let p = PlayerSnapshot {
            pos: Vec2::new(589.0, 206.0),
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..50)
            .map(|frame| InputState {
                move_x: 1,
                dash_pressed: frame == 20,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &bumper_clip_map(), inputs.len() as u32).unwrap();
        assert_eq!(trace.states[1].state, PlayerState::Launch);
        assert!(
            trace
                .states
                .iter()
                .any(|state| state.state == PlayerState::Dash)
        );
        assert!(trace.states.iter().skip(20).any(|state| {
            state.pos.x > 600.0
                && state.bumper_reuse_timer > 0.0
                && state.state != PlayerState::Launch
        }));
        assert!(trace.states.iter().all(|state| !state.dead));
    }

    #[test]
    fn refill_restores_dash_and_stamina_then_respawns_after_two_point_five_seconds() {
        let p = PlayerSnapshot {
            // The hurtbox is what `Refill`'s `PlayerCollider` sees
            // (`Hitbox(8, 9, -4, -11)`, two pixels shorter than the hitbox), so the
            // player stands inside the crystal's 16x16 box rather than 1 px above it.
            pos: Vec2::new(84.0, 92.0),
            on_ground: true,
            dashes: 0,
            stamina: 5.0,
            ..PlayerSnapshot::default()
        };
        let mut map = refill_map(false, false);
        let trace = simulate_trace(p, &[InputState::default(); 200], &map, 200).unwrap();
        // Frame 1 collects: dashes 0 -> 1, stamina 5 -> 110, 0.05s freeze.
        assert_eq!(trace.states[1].dashes, 1);
        assert_eq!(trace.states[1].stamina, 110.0);
        assert_eq!(trace.states[1].freeze_timer, REFILL_FREEZE_TIME);
        assert!(!trace.states[1].refills[0].collidable);
        assert_eq!(
            trace.states[1].refills[0].respawn_timer,
            REFILL_RESPAWN_TIME
        );
        assert!((trace.states[2].freeze_timer - (REFILL_FREEZE_TIME - DT)).abs() < 1e-6);
        // Respawn restores collidability after ~150 active frames.
        assert!(!trace.states[100].refills[0].collidable);
        assert!(trace.states[160].refills[0].collidable);
    }

    #[test]
    fn pink_refill_sets_two_dashes_while_full_refill_does_not_collect() {
        let mut map = refill_map(true, false);
        let p = PlayerSnapshot {
            // The hurtbox is what `Refill`'s `PlayerCollider` sees
            // (`Hitbox(8, 9, -4, -11)`, two pixels shorter than the hitbox), so the
            // player stands inside the crystal's 16x16 box rather than 1 px above it.
            pos: Vec2::new(84.0, 92.0),
            on_ground: true,
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 3], &map, 3).unwrap();
        assert_eq!(trace.states[1].dashes, 2);
        assert!(!trace.states[1].refills[0].collidable);

        // A full player touching a regular refill gains nothing and the
        // refill stays collidable for a later depleted pass.
        let mut map = refill_map(false, false);
        let p = PlayerSnapshot {
            // The hurtbox is what `Refill`'s `PlayerCollider` sees
            // (`Hitbox(8, 9, -4, -11)`, two pixels shorter than the hitbox), so the
            // player stands inside the crystal's 16x16 box rather than 1 px above it.
            pos: Vec2::new(84.0, 92.0),
            on_ground: true,
            dashes: 1,
            stamina: 110.0,
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default(); 2], &map, 2).unwrap();
        assert_eq!(p.dashes, 1);
        assert!(p.refills[0].collidable);
    }

    #[test]
    fn one_use_refill_removes_itself_after_collection() {
        let p = PlayerSnapshot {
            // The hurtbox is what `Refill`'s `PlayerCollider` sees
            // (`Hitbox(8, 9, -4, -11)`, two pixels shorter than the hitbox), so the
            // player stands inside the crystal's 16x16 box rather than 1 px above it.
            pos: Vec2::new(84.0, 92.0),
            on_ground: true,
            dashes: 0,
            stamina: 5.0,
            ..PlayerSnapshot::default()
        };
        let mut map = refill_map(false, true);
        let trace = simulate_trace(p, &[InputState::default(); 20], &map, 20).unwrap();
        assert!(trace.states[1].refills[0].removed);
        assert!(!trace.states[1].refills[0].collidable);
        assert!(trace.states[20].refills[0].removed);
    }

    #[test]
    fn falling_block_triggers_rides_and_lands_permanently() {
        let p = PlayerSnapshot {
            pos: Vec2::new(120.0, 100.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let mut map = falling_block_map(true, 100.0);
        let trace = simulate_trace(p, &[InputState::default(); 140], &map, 140).unwrap();
        // Phase timeline: 1 shake (0.2s), 2 player-wait (0.4s), 3 falling.
        assert_eq!(trace.states[1].falling_blocks[0].phase, 1);
        assert_eq!(trace.states[13].falling_blocks[0].phase, 2);
        assert_eq!(trace.states[37].falling_blocks[0].phase, 3);
        // The rider is carried: the player bottom (pos.y) stays flush with
        // the block top on the falling frames (top y=100 -> floor at y=160).
        assert!(
            (trace.states[40].pos.y - trace.states[40].falling_blocks[0].position.y).abs() < 0.01
        );
        assert!(trace.states[40].falling_blocks[0].position.y > 100.0);
        // Landing: block rests on the floor, becomes Safe, player on top.
        assert_eq!(trace.states[100].falling_blocks[0].phase, 5);
        assert!(trace.states[100].falling_blocks[0].safe);
        assert_eq!(trace.states[100].falling_blocks[0].position.y, 144.0);
        assert_eq!(trace.states[100].pos.y, 144.0);
        assert!(trace.states[100].on_ground);
        assert!(!trace.states[100].dead);
    }

    #[test]
    fn falling_block_wait_window_ends_early_when_the_player_leaves() {
        let p = PlayerSnapshot {
            pos: Vec2::new(120.0, 100.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let mut map = falling_block_map(true, 100.0);
        // Stand through the 0.2s shake, then jump on the shake-end step.
        // The shake resume's loop-head check sees the airborne player and the
        // block drops that same frame; without the early exit it would wait
        // until frame 37.
        let inputs: Vec<_> = (0..24)
            .map(|frame| InputState {
                jump_pressed: frame == 12,
                jump_held: frame >= 12,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &map, 24).unwrap();
        assert_eq!(trace.states[13].falling_blocks[0].phase, 3);
        assert!(trace.states[20].falling_blocks[0].position.y > 100.0);
    }

    #[test]
    fn ordinary_falling_block_triggers_from_player_on_top() {
        let p = PlayerSnapshot {
            pos: Vec2::new(120.0, 100.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let mut map = falling_block_map(false, 100.0);
        let trace = simulate_trace(p, &[InputState::default(); 50], &map, 50).unwrap();
        assert_eq!(trace.states[1].falling_blocks[0].phase, 1);
        assert_eq!(trace.states[37].falling_blocks[0].phase, 3);
        assert!(trace.states[50].falling_blocks[0].position.y > 100.0);
    }

    #[test]
    fn triggered_falling_block_drops_without_a_player_and_despawns_below_bounds() {
        let mut map = falling_block_map(true, 160.0);
        map.solids.clear();
        let mut p = PlayerSnapshot {
            pos: Vec2::new(20.0, 20.0),
            ..PlayerSnapshot::default()
        };
        p = simulate(p, &[InputState::default(); 1], &map, 1).unwrap();
        p.falling_blocks[0].triggered = true;
        let p = simulate(p, &[InputState::default(); 90], &map, 90).unwrap();
        assert!(p.falling_blocks[0].removed);
        assert!(!p.falling_blocks[0].collidable);
    }
    #[test]
    fn spring_cancel_uses_the_buffered_dash_after_the_spring_refills_it() {
        let p = PlayerSnapshot {
            pos: Vec2::new(80.0, 92.0),
            speed: Vec2::new(0.0, 100.0),
            dashes: 0,
            ..PlayerSnapshot::default()
        };
        let inputs = [
            InputState {
                dash_pressed: true,
                ..InputState::default()
            },
            InputState::default(),
            InputState::default(),
            InputState::default(),
        ];
        let trace = simulate_trace(p, &inputs, &spring_map(Vec2::new(0.0, -1.0)), 4).unwrap();
        assert_eq!(trace.states[2].state, PlayerState::Normal);
        assert_eq!(trace.states[2].pos, Vec2::new(80.0, 96.0));
        assert!((trace.states[2].speed.y - 130.0).abs() < 0.001);
        assert_eq!(trace.states[2].dashes, 0);
        assert_eq!(trace.states[3].state, PlayerState::Normal);
        assert_eq!(trace.states[3].pos, Vec2::new(80.0, 94.0));
        assert_eq!(trace.states[3].dashes, 1);
        assert_eq!(trace.states[3].speed, Vec2::new(0.0, SUPER_BOUNCE_SPEED));
        assert!(trace.states[3].dash_buffer_timer > 0.0);
        assert_eq!(trace.states[4].state, PlayerState::Dash);
        assert_eq!(trace.states[4].dashes, 0);
        assert_eq!(trace.states[4].speed, Vec2::default());
        assert_eq!(trace.states[4].dash_buffer_timer, 0.0);
    }

    #[test]
    fn first_berry_collects_after_nine_consecutive_safe_ground_frames() {
        let p = PlayerSnapshot {
            pos: Vec2::new(80.0, 100.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 30], &berry_map(1), 30).unwrap();
        assert_eq!(trace.states[1].carried_strawberries, 1);
        let ready = trace
            .states
            .iter()
            .position(|state| {
                state.carried_strawberries > 0 && state.strawberry_follow_delay_timer <= 0.0
            })
            .unwrap();
        let collected = trace
            .states
            .iter()
            .position(|state| state.strawberry_collect_index == 1)
            .unwrap();
        assert_eq!(collected - ready, 9);
        assert_eq!(trace.states[collected - 1].carried_strawberries, 1);
        assert_eq!(trace.states[collected].carried_strawberries, 0);
    }

    #[test]
    fn later_berry_in_the_train_waits_through_the_negative_collection_offset() {
        let p = PlayerSnapshot {
            pos: Vec2::new(80.0, 100.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 60], &berry_map(2), 60).unwrap();
        let first = trace
            .states
            .iter()
            .position(|state| state.strawberry_collect_index == 1)
            .unwrap();
        let second = trace
            .states
            .iter()
            .position(|state| state.strawberry_collect_index == 2)
            .unwrap();
        assert_eq!(second - first, 17);
        assert!((trace.states[first].strawberry_collect_timer - (-0.15 + DT)).abs() < 0.000_001);
        assert_eq!(trace.states[second].carried_strawberries, 0);
    }

    #[test]
    fn wall_spring_uses_source_side_bounce_speed_and_force_move() {
        let p = PlayerSnapshot {
            pos: Vec2::new(103.0, 80.0),
            speed: Vec2::new(-30.0, 20.0),
            dashes: 0,
            stamina: 20.0,
            ..PlayerSnapshot::default()
        };
        let p = simulate(
            p,
            &[InputState::default()],
            &spring_map(Vec2::new(1.0, 0.0)),
            1,
        )
        .unwrap();
        assert_eq!(p.state, PlayerState::Normal);
        assert_eq!(p.pos, Vec2::new(110.0, 80.0));
        assert_eq!(p.speed, Vec2::new(SIDE_BOUNCE_SPEED, BOUNCE_SPEED));
        assert_eq!(p.force_move_x, 1);
        assert_eq!(p.force_move_x_timer, SIDE_BOUNCE_FORCE_MOVE_X_TIME);
        assert_eq!(p.dashes, 1);
        assert_eq!(p.stamina, 110.0);
    }

    #[test]
    fn ice_ball_bounce_cancels_dash_and_preserves_horizontal_speed() {
        let mut bounced = PlayerSnapshot {
            pos: Vec2::new(96.0, 100.0),
            speed: Vec2::new(240.0, 0.0),
            state: PlayerState::Dash,
            dashes: 0,
            dash_attack_timer: DASH_ATTACK_TIME,
            ..PlayerSnapshot::default()
        };
        let mut map = ice_ball_map();
        interact(&mut bounced, &map, InputState::default(), None);
        assert_eq!(bounced.state, PlayerState::Normal);
        assert_eq!(bounced.pending_bounce_from_y, None);
        assert_eq!(bounced.pos, Vec2::new(96.0, 98.0));
        assert_eq!(bounced.speed, Vec2::new(240.0, -140.0));
        assert_eq!(bounced.dashes, 1);
        assert_eq!(bounced.stamina, 110.0);
        assert_eq!(bounced.dash_attack_timer, 0.0);
        assert!(bounced.auto_jump);
        assert_eq!(bounced.auto_jump_timer, 0.1);
        assert_eq!(bounced.var_jump_timer, VAR_JUMP_TIME);

        let held = simulate(
            bounced.clone(),
            &[InputState {
                jump_held: true,
                ..InputState::default()
            }; 12],
            &ice_ball_map(),
            12,
        )
        .unwrap();
        let released =
            simulate(bounced, &[InputState::default(); 12], &ice_ball_map(), 12).unwrap();
        assert!(held.speed.y < released.speed.y);
        assert!(held.pos.y < released.pos.y);
    }

    #[test]
    fn ice_ball_same_frame_callback_keeps_split_simulation_composable() {
        let initial = PlayerSnapshot {
            pos: Vec2::new(317.0, 155.0),
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..24)
            .map(|frame| InputState {
                move_x: 1,
                move_y: 1,
                jump_held: true,
                dash_pressed: frame == 0,
                ..InputState::default()
            })
            .collect();
        let mut map = crate::mechanics_playground();
        let trace = simulate_trace(initial.clone(), &inputs[..6], &map, 6).unwrap();
        assert_eq!(trace.states[5].state, PlayerState::Dash);
        assert_eq!(trace.states[5].speed, Vec2::new(169.705_63, 169.705_63));
        assert_eq!(trace.states[6].state, PlayerState::Normal);
        assert_eq!(trace.states[6].speed.y, -140.0);
        assert_eq!(trace.states[6].pending_bounce_from_y, None);
        let whole = simulate(initial.clone(), &inputs, &map, inputs.len() as u32).unwrap();
        let first = simulate(initial, &inputs[..5], &map, 5).unwrap();
        assert_eq!(first.pending_bounce_from_y, None);
        assert_eq!(first.state, PlayerState::Dash);
        let split = simulate(first, &inputs[5..], &map, (inputs.len() - 5) as u32).unwrap();
        assert_eq!(split, whole);
    }

    #[test]
    fn fish_oshiro_and_snowball_top_callbacks_share_player_bounce_semantics() {
        for kind in [
            EntityKind::Puffer,
            EntityKind::AngryOshiro,
            EntityKind::Snowball,
        ] {
            let mut p = PlayerSnapshot {
                pos: Vec2::new(100.0, 100.0),
                speed: Vec2::new(240.0, 0.0),
                state: PlayerState::Dash,
                dashes: 0,
                stamina: 5.0,
                dash_attack_timer: DASH_ATTACK_TIME,
                ..PlayerSnapshot::default()
            };
            let mut map = bounce_actor_map(kind.clone());
            interact(&mut p, &map, InputState::default(), None);
            assert_eq!(p.state, PlayerState::Normal, "kind={kind:?}");
            assert_eq!(p.speed, Vec2::new(240.0, BOUNCE_SPEED), "kind={kind:?}");
            assert_eq!(p.dashes, 1, "kind={kind:?}");
            assert_eq!(p.stamina, 110.0, "kind={kind:?}");
            assert_eq!(p.dash_attack_timer, 0.0, "kind={kind:?}");
            assert_eq!(p.var_jump_timer, VAR_JUMP_TIME, "kind={kind:?}");
        }
    }

    #[test]
    fn seeker_attack_wall_collision_enters_stunned_with_source_speeds_and_timer() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 200.0, 180.0),
            solids: vec![Rect::new(104.0, 0.0, 16.0, 180.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::Seeker,
                bounds: Rect::new(94.0, 94.0, 12.0, 12.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "seeker".to_owned(),
            }],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(20.0, 160.0),
            seekers: vec![crate::SeekerSnapshot {
                position: Vec2::new(100.0, 100.0),
                speed: Vec2::new(120.0, 50.0),
                state: 3,
                ..crate::SeekerSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 2], &map, 2).unwrap();

        assert_eq!(trace.states[1].seekers[0].state, 4);
        assert_eq!(trace.states[1].seekers[0].speed, Vec2::new(-100.0, 20.0));
        assert_eq!(trace.states[1].seekers[0].state_timer, 0.8);
        assert_eq!(
            trace.states[2].seekers[0].speed,
            approach_vec(Vec2::new(-100.0, 20.0), Vec2::default(), 150.0 * DT)
        );
        assert_eq!(trace.states[2].seekers[0].state_timer, 0.8 - DT);
    }

    #[test]
    fn seeker_stunned_coroutine_returns_idle_and_split_simulation_is_composable() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            solids: vec![Rect::new(0.0, 160.0, 320.0, 20.0)],
            entities: vec![crate::Entity {
                kind: EntityKind::Seeker,
                bounds: Rect::new(194.0, 94.0, 12.0, 12.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "seeker".to_owned(),
            }],
            ..Map::default()
        };
        let initial = PlayerSnapshot {
            pos: Vec2::new(20.0, 160.0),
            on_ground: true,
            seekers: vec![crate::SeekerSnapshot {
                position: Vec2::new(200.0, 100.0),
                speed: Vec2::new(-100.0, 20.0),
                state: 4,
                state_timer: 0.8,
                ..crate::SeekerSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let inputs = [InputState::default(); 55];
        let trace = simulate_trace(initial.clone(), &inputs, &map, inputs.len() as u32).unwrap();
        let returned = trace
            .states
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, state)| state.seekers[0].state == 0)
            .map(|(frame, _)| frame)
            .expect("StunnedCoroutine should return to Idle after 0.8 seconds");
        assert_eq!(returned, 49);
        assert_eq!(trace.states[returned].seekers[0].speed, Vec2::default());

        let whole = trace.states.last().unwrap().clone();
        let first = simulate(initial, &inputs[..20], &map, 20).unwrap();
        let split = simulate(first, &inputs[20..], &map, 35).unwrap();
        assert_eq!(split, whole);
    }

    #[test]
    fn stunned_seeker_side_contact_point_bounces_player_and_recoils_at_one_hundred() {
        let mut map = bounce_actor_map(EntityKind::Seeker);
        let mut p = PlayerSnapshot {
            pos: Vec2::new(92.0, 105.5),
            dashes: 0,
            stamina: 5.0,
            seekers: vec![crate::SeekerSnapshot {
                position: Vec2::new(100.0, 100.0),
                state: 4,
                state_timer: 0.8,
                ..crate::SeekerSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        initialize_seekers(&mut p, &mut map);
        advance_seekers(&mut p, &mut map);

        assert!(!p.dead);
        assert_eq!(p.dashes, 1);
        assert_eq!(p.stamina, 110.0);
        // Player.PointBounce uses 200 speed, a 1.2 horizontal multiplier,
        // and only then applies its 120-pixel horizontal minimum.
        assert!(
            (p.speed.x + 238.146_68).abs() < 0.000_01,
            "unexpected point-bounce speed: {:?}",
            p.speed
        );
        assert!((p.speed.y + 24.806_946).abs() < 0.000_01);
        assert_eq!(p.seekers[0].speed, Vec2::new(100.0, 0.0));
        assert_eq!(p.seekers[0].state, 4);
    }

    #[test]
    fn attacking_seeker_side_contact_kills_but_top_contact_bounces_and_regenerates() {
        let mut side_map = bounce_actor_map(EntityKind::Seeker);
        let mut side = PlayerSnapshot {
            pos: Vec2::new(92.0, 105.5),
            seekers: vec![crate::SeekerSnapshot {
                position: Vec2::new(100.0, 100.0),
                state: 3,
                ..crate::SeekerSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        initialize_seekers(&mut side, &mut side_map);
        advance_seekers(&mut side, &mut side_map);
        assert!(side.dead);

        let mut top_map = bounce_actor_map(EntityKind::Seeker);
        let mut top = PlayerSnapshot {
            pos: Vec2::new(100.0, 97.0),
            speed: Vec2::new(50.0, 20.0),
            dashes: 0,
            stamina: 5.0,
            seekers: vec![crate::SeekerSnapshot {
                position: Vec2::new(100.0, 102.0),
                state: 3,
                ..crate::SeekerSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        initialize_seekers(&mut top, &mut top_map);
        advance_seekers(&mut top, &mut top_map);
        assert!(!top.dead);
        assert_eq!(top.state, PlayerState::Normal);
        assert_eq!(top.speed, Vec2::new(50.0, BOUNCE_SPEED));
        assert_eq!(top.dashes, 1);
        assert_eq!(top.stamina, 110.0);
        assert_eq!(top.freeze_timer, 0.15);
        assert_eq!(top.seekers[0].state, 6);
    }

    fn temple_gate_map(obstacle_height: f32) -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            solids: vec![Rect::new(96.0, 148.0, 16.0, obstacle_height)],
            entities: vec![
                crate::Entity {
                    kind: EntityKind::TempleGate,
                    bounds: Rect::new(100.0, 100.0, 8.0, 48.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "templeGate".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::TheoCrystal,
                    bounds: Rect::new(100.0, 135.0, 8.0, 10.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "theoCrystal".to_owned(),
                },
            ],
            // The fixture stands in for the vanilla gate the encoder round-trips, which is the
            // `CloseBehindPlayerAlways` one (`CloseBehindPlayer` is the type that starts shut
            // unless the player is already in front of it).
            entity_visuals: vec![
                crate::map::EntityVisual {
                    variant: Some("CloseBehindPlayerAlways".to_owned()),
                    ..crate::map::EntityVisual::default()
                },
                crate::map::EntityVisual::default(),
            ],
            ..Map::default()
        }
    }

    #[test]
    fn close_behind_player_gate_uses_target_position_fallback_to_clip_theo() {
        let mut map = temple_gate_map(1.0);
        let p = PlayerSnapshot {
            pos: Vec2::new(120.0, 160.0),
            theo_crystals: vec![crate::TheoCrystalSnapshot {
                position: Vec2::new(104.0, 145.0),
                ..crate::TheoCrystalSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let closed = simulate(p.clone(), &[InputState::default()], &map, 1).unwrap();

        assert!(closed.temple_gates[0].triggered);
        assert!(!closed.temple_gates[0].open);
        assert_eq!(closed.temple_gates[0].current_height, 48.0);
        assert_eq!(closed.theo_crystals[0].position, Vec2::new(104.0, 159.0));
        assert!(!closed.theo_crystals[0].dead);
        assert!(!closed.dead);

        let whole = simulate(p, &[InputState::default(); 3], &map, 3).unwrap();
        let split = simulate(closed, &[InputState::default(); 2], &map, 2).unwrap();
        assert_eq!(split, whole);
    }

    #[test]
    fn player_squish_tries_ducked_target_position_before_actor_wiggles() {
        let mut map = temple_gate_map(1.0);
        let mut p = PlayerSnapshot {
            pos: Vec2::new(104.0, 145.0),
            ..PlayerSnapshot::default()
        };
        initialize_temple_gates(&mut p, &mut map);
        let mut gate = p.temple_gates[0].clone();
        close_temple_gate(&mut p, &mut map, 0, &mut gate);

        assert_eq!(p.pos, Vec2::new(104.0, 159.0));
        assert!(p.ducking);
        assert!(!p.dead);
    }

    #[test]
    fn failed_gate_squish_kills_theo_with_player_and_removes_glider() {
        let mut map = temple_gate_map(20.0);
        map.entities.push(crate::Entity {
            kind: EntityKind::Glider,
            bounds: Rect::new(100.0, 135.0, 8.0, 10.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "glider".to_owned(),
        });
        let mut p = PlayerSnapshot {
            pos: Vec2::new(120.0, 160.0),
            theo_crystals: vec![crate::TheoCrystalSnapshot {
                position: Vec2::new(104.0, 145.0),
                ..crate::TheoCrystalSnapshot::default()
            }],
            gliders: vec![crate::GliderSnapshot {
                position: Vec2::new(104.0, 145.0),
                ..crate::GliderSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        initialize_temple_gates(&mut p, &mut map);
        initialize_theo_crystals(&mut p, &mut map);
        initialize_gliders(&mut p, &mut map);
        let mut gate = p.temple_gates[0].clone();
        close_temple_gate(&mut p, &mut map, 0, &mut gate);

        assert!(p.theo_crystals[0].dead);
        assert!(p.dead);
        assert!(p.gliders[0].removed);
    }

    /// One `templeGate` at (100, 100) with the vanilla 8x48 collider, decoded from a map whose
    /// `type` attribute is `gate_type` (`TempleGate.cs:72-74`).
    fn typed_temple_gate_map(gate_type: &str) -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            entities: vec![crate::Entity {
                kind: EntityKind::TempleGate,
                bounds: Rect::new(100.0, 100.0, 8.0, 48.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "templeGate".to_owned(),
            }],
            entity_visuals: vec![crate::map::EntityVisual {
                variant: Some(gate_type.to_owned()),
                ..crate::map::EntityVisual::default()
            }],
            ..Map::default()
        }
    }

    /// `TempleGate.Awake` (`TempleGate.cs:77-112`) decides the start state per type, and only
    /// `NearestSwitch`/`TouchSwitches` gates start at their decoded height.
    #[test]
    fn awake_decides_each_temple_gate_types_start_state() {
        // `NearestSwitch`, `TouchSwitches`, and any unknown name (`data.Enum`'s fallback).
        for gate_type in ["NearestSwitch", "TouchSwitches", "not-a-type"] {
            let mut map = typed_temple_gate_map(gate_type);
            let mut p = PlayerSnapshot {
                pos: Vec2::new(80.0, 140.0),
                ..PlayerSnapshot::default()
            };
            initialize_temple_gates(&mut p, &mut map);
            assert!(!p.temple_gates[0].open, "{gate_type} must start shut");
            assert_eq!(p.temple_gates[0].current_height, 48.0, "{gate_type}");
            assert_eq!(
                map.entities[0].bounds,
                Rect::new(100.0, 100.0, 8.0, 48.0),
                "{gate_type} must stay a Solid"
            );
        }
        // `CloseBehindPlayerAlways` and `CloseBehindPlayerAndTheo` call `StartOpen` (`:89-98`).
        for gate_type in ["CloseBehindPlayerAlways", "CloseBehindPlayerAndTheo"] {
            let mut map = typed_temple_gate_map(gate_type);
            let mut p = PlayerSnapshot {
                pos: Vec2::new(220.0, 140.0),
                ..PlayerSnapshot::default()
            };
            initialize_temple_gates(&mut p, &mut map);
            assert!(p.temple_gates[0].open, "{gate_type} must start open");
            assert_eq!(p.temple_gates[0].draw_height, 4.0, "{gate_type}");
            assert_eq!(map.entities[0].bounds.height, 0.0, "{gate_type}");
        }
        // `CloseBehindPlayer` opens only when the player is already in front of the gate (`:83`):
        // `entity.Left < base.Right && entity.Bottom >= base.Top && entity.Top <= base.Bottom`.
        let mut map = typed_temple_gate_map("CloseBehindPlayer");
        let mut in_front = PlayerSnapshot {
            pos: Vec2::new(80.0, 140.0),
            ..PlayerSnapshot::default()
        };
        initialize_temple_gates(&mut in_front, &mut map);
        assert!(in_front.temple_gates[0].open);

        let mut map = typed_temple_gate_map("CloseBehindPlayer");
        let mut behind = PlayerSnapshot {
            pos: Vec2::new(220.0, 140.0),
            ..PlayerSnapshot::default()
        };
        initialize_temple_gates(&mut behind, &mut map);
        assert!(
            !behind.temple_gates[0].open,
            "no coroutine is added for a gate the player is past, so nothing can open it"
        );

        let mut map = typed_temple_gate_map("CloseBehindPlayer");
        let mut above = PlayerSnapshot {
            pos: Vec2::new(80.0, 60.0),
            ..PlayerSnapshot::default()
        };
        initialize_temple_gates(&mut above, &mut map);
        assert!(
            !above.temple_gates[0].open,
            "the vertical overlap half of the Awake test"
        );
    }

    /// A `Sides.Right` dash switch at (80, 116) and a `NearestSwitch` gate at (100, 100).
    fn dash_switch_and_gate_map(all_gates: bool) -> Map {
        let mut map = typed_temple_gate_map("NearestSwitch");
        map.entities.push(crate::Entity {
            kind: EntityKind::DashSwitch,
            bounds: Rect::new(80.0, 116.0, 8.0, 16.0),
            direction: Vec2::new(1.0, 0.0),
            shielded: all_gates,
            single_use: false,
            nodes: vec![],
            name: "dashSwitchH".to_owned(),
        });
        map.entity_visuals.push(crate::map::EntityVisual::default());
        map
    }

    /// A further-away `NearestSwitch` gate, so `GetGate`'s nearest search has a choice.
    fn push_far_nearest_switch_gate(map: &mut Map) {
        map.entities.push(crate::Entity {
            kind: EntityKind::TempleGate,
            bounds: Rect::new(220.0, 100.0, 8.0, 48.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "templeGate".to_owned(),
        });
        map.entity_visuals.push(crate::map::EntityVisual {
            variant: Some("NearestSwitch".to_owned()),
            ..crate::map::EntityVisual::default()
        });
    }

    /// `TempleGate.SwitchOpen` (`TempleGate.cs:124-132`) is two chained 0.2 s `Alarm`s, so the
    /// collider stays up for 24 frames after the press and comes down on the 24th.
    #[test]
    fn switch_open_collapses_the_gate_on_the_second_alarm() {
        let mut map = dash_switch_and_gate_map(false);
        let mut p = PlayerSnapshot {
            frame_delta_time: DT,
            ..PlayerSnapshot::default()
        };
        initialize_temple_gates(&mut p, &mut map);
        assert!(!p.temple_gates[0].open);
        // `DashSwitch.OnDashed` moves the switch to `bounds + pressDirection * 6f` first
        // (`DashSwitch.cs:203-205`), then runs the gate fan-out.
        dash_switch_open_gates(&mut p, &mut map, 1, Vec2::new(86.0, 124.0), true);
        assert_eq!(p.temple_gates[0].alarm_stage, 1);
        assert_eq!(p.temple_gates[0].alarm_timer, TEMPLE_GATE_SWITCH_BEAT);
        assert!(p.temple_gates[0].claimed, "GetGate marks the gate it took");

        let mut room = initialize_room_coroutines(&mut map);
        for _ in 0..23 {
            advance_temple_gate_alarms(&mut p, &mut map, &mut room);
        }
        assert!(
            !p.temple_gates[0].open,
            "the collider is still up on the 23rd frame after the press"
        );
        assert_eq!(map.entities[0].bounds.height, 48.0);
        advance_temple_gate_alarms(&mut p, &mut map, &mut room);
        assert!(p.temple_gates[0].open);
        assert_eq!(p.temple_gates[0].current_height, 0.0);
        assert_eq!(p.temple_gates[0].alarm_stage, 0);
        assert_eq!(map.entities[0].bounds.height, 0.0);
    }

    /// `allGates` (`DashSwitch.cs:208-217`) opens *every* `NearestSwitch` gate of the room and
    /// claims none of them.
    #[test]
    fn an_all_gates_switch_opens_every_nearest_switch_gate() {
        let mut map = dash_switch_and_gate_map(true);
        push_far_nearest_switch_gate(&mut map);
        map.entities.push(crate::Entity {
            kind: EntityKind::TempleGate,
            bounds: Rect::new(260.0, 100.0, 8.0, 48.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "templeGate".to_owned(),
        });
        map.entity_visuals.push(crate::map::EntityVisual {
            variant: Some("TouchSwitches".to_owned()),
            ..crate::map::EntityVisual::default()
        });

        let mut p = PlayerSnapshot {
            frame_delta_time: DT,
            ..PlayerSnapshot::default()
        };
        initialize_temple_gates(&mut p, &mut map);
        dash_switch_open_gates(&mut p, &mut map, 1, Vec2::new(86.0, 124.0), true);
        assert_eq!(p.temple_gates[0].alarm_stage, 1);
        assert_eq!(p.temple_gates[1].alarm_stage, 1);
        assert_eq!(
            p.temple_gates[2].alarm_stage, 0,
            "a TouchSwitches gate is not a NearestSwitch gate"
        );
        assert!(p.temple_gates.iter().all(|gate| !gate.claimed));
    }

    /// Without `allGates` the press takes the *nearest unclaimed* `NearestSwitch` gate
    /// (`DashSwitch.GetGate`, `DashSwitch.cs:231-253`) and a second press can only take another.
    #[test]
    fn a_press_claims_only_the_nearest_unclaimed_nearest_switch_gate() {
        let mut map = dash_switch_and_gate_map(false);
        push_far_nearest_switch_gate(&mut map);
        let mut p = PlayerSnapshot {
            frame_delta_time: DT,
            ..PlayerSnapshot::default()
        };
        initialize_temple_gates(&mut p, &mut map);
        dash_switch_open_gates(&mut p, &mut map, 1, Vec2::new(86.0, 124.0), true);
        assert!(p.temple_gates[0].claimed);
        assert_eq!(p.temple_gates[0].alarm_stage, 1);
        assert!(!p.temple_gates[1].claimed);
        assert_eq!(p.temple_gates[1].alarm_stage, 0);

        dash_switch_open_gates(&mut p, &mut map, 1, Vec2::new(86.0, 124.0), true);
        assert_eq!(p.temple_gates[1].alarm_stage, 1);
    }

    /// `CheckTouchSwitches` (`TempleGate.cs:197-212`): `Switch.Check` needs *every* switch in the
    /// room, then a 0.5 s `yield` and a 0.2 s one - 31 + 13 frames, since `Monocle.Coroutine`
    /// resumes on the frame *after* the counter reaches zero (`Coroutine.cs:35-42`).
    #[test]
    fn a_touch_switches_gate_waits_for_every_switch_and_the_two_coroutine_beats() {
        let mut map = typed_temple_gate_map("TouchSwitches");
        for x in [160.0, 200.0] {
            map.entities.push(crate::Entity {
                kind: EntityKind::TouchSwitch,
                bounds: Rect::new(x - 15.0, 125.0, 30.0, 30.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "touchSwitch".to_owned(),
            });
            map.entity_visuals.push(crate::map::EntityVisual::default());
        }
        let mut p = PlayerSnapshot {
            pos: Vec2::new(40.0, 140.0),
            frame_delta_time: DT,
            ..PlayerSnapshot::default()
        };
        initialize_temple_gates(&mut p, &mut map);
        let mut room = initialize_room_coroutines(&mut map);
        assert_eq!(room.touch_switches.len(), 2);
        for _ in 0..80 {
            advance_temple_gate_alarms(&mut p, &mut map, &mut room);
        }
        assert!(!p.temple_gates[0].open, "no switch has been hit yet");

        // `Switch.FinishedCheck` (`Switch.cs:104-118`) marks the *last* arrival as the one that
        // finishes every component, so one activation is not enough.
        room.touch_switches[1].activated = true;
        for _ in 0..80 {
            advance_temple_gate_alarms(&mut p, &mut map, &mut room);
        }
        assert!(!p.temple_gates[0].open, "one of the two switches is still off");

        room.touch_switches[0].activated = true;
        for _ in 0..44 {
            advance_temple_gate_alarms(&mut p, &mut map, &mut room);
        }
        assert!(
            !p.temple_gates[0].open,
            "0.5 s + 0.2 s of coroutine waits is 44 frames after the check sees every switch"
        );
        advance_temple_gate_alarms(&mut p, &mut map, &mut room);
        assert!(p.temple_gates[0].open);
        assert_eq!(map.entities[0].bounds.height, 0.0);
    }

    /// Only the `CloseBehindPlayer*` types carry a close coroutine (`TempleGate.cs:77-98`): a gate
    /// a dash switch opened stays open for good.
    #[test]
    fn a_press_opened_nearest_switch_gate_never_closes_behind_the_player() {
        let mut map = dash_switch_and_gate_map(false);
        let mut p = PlayerSnapshot {
            pos: Vec2::new(60.0, 140.0),
            frame_delta_time: DT,
            ..PlayerSnapshot::default()
        };
        initialize_temple_gates(&mut p, &mut map);
        dash_switch_open_gates(&mut p, &mut map, 1, Vec2::new(86.0, 124.0), true);
        let mut room = initialize_room_coroutines(&mut map);
        for _ in 0..24 {
            advance_temple_gate_alarms(&mut p, &mut map, &mut room);
        }
        assert!(p.temple_gates[0].open);
        advance_temple_gates(&mut p, &mut map);
        p.pos = Vec2::new(220.0, 140.0);
        advance_temple_gates(&mut p, &mut map);
        assert!(
            p.temple_gates[0].open && !p.temple_gates[0].triggered,
            "a NearestSwitch gate has no close-behind coroutine"
        );
    }

    /// `CloseBehindPlayerAndTheo` only breaks its loop once a live `TheoCrystal` is past the gate
    /// too, and a room without one never closes it (`TempleGate.cs:179-195`).
    #[test]
    fn close_behind_player_and_theo_waits_for_the_crystal() {
        let mut map = typed_temple_gate_map("CloseBehindPlayerAndTheo");
        map.entities.push(crate::Entity {
            kind: EntityKind::TheoCrystal,
            bounds: Rect::new(96.0, 130.0, 8.0, 10.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "theoCrystal".to_owned(),
        });
        map.entity_visuals.push(crate::map::EntityVisual::default());
        let mut p = PlayerSnapshot {
            pos: Vec2::new(80.0, 140.0),
            frame_delta_time: DT,
            ..PlayerSnapshot::default()
        };
        initialize_theo_crystals(&mut p, &mut map);
        initialize_temple_gates(&mut p, &mut map);
        assert!(p.temple_gates[0].open);

        // The player walks past; Theo is still on the near side.
        p.pos = Vec2::new(220.0, 140.0);
        advance_temple_gates(&mut p, &mut map);
        assert!(
            p.temple_gates[0].open,
            "Theo has not passed base.Right + 4f yet"
        );

        p.theo_crystals[0].position = Vec2::new(200.0, 140.0);
        advance_temple_gates(&mut p, &mut map);
        assert!(!p.temple_gates[0].open);
        assert_eq!(p.temple_gates[0].current_height, 48.0);
    }

    /// `HoldingTheo` gates follow the crystal, and `holdingWaitTimer`/`lockState`
    /// (`TempleGate.cs:246-274`) gate the toggle.
    #[test]
    fn a_holding_theo_gate_follows_the_crystal() {
        let mut map = typed_temple_gate_map("HoldingTheo");
        map.entities.push(crate::Entity {
            kind: EntityKind::TheoCrystal,
            bounds: Rect::new(296.0, 130.0, 8.0, 10.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "theoCrystal".to_owned(),
        });
        map.entity_visuals.push(crate::map::EntityVisual::default());
        let mut p = PlayerSnapshot {
            pos: Vec2::new(40.0, 140.0),
            frame_delta_time: DT,
            ..PlayerSnapshot::default()
        };
        initialize_theo_crystals(&mut p, &mut map);
        initialize_temple_gates(&mut p, &mut map);
        assert!(
            !p.temple_gates[0].open,
            "Theo starts 190 px from the gate's holding point"
        );
        assert_eq!(
            map.entities[0].bounds.width, 16.0,
            "`Awake` widens a HoldingTheo gate's hitbox to 16 (`TempleGate.cs:105`)"
        );

        let mut room = initialize_room_coroutines(&mut map);
        // `holdingCheckFrom` is `Position + (4, height / 2)` = (104, 124) and the closed-state
        // radius is 64 px (`4096`).
        p.theo_crystals[0].position = Vec2::new(104.0, 138.0);
        for _ in 0..12 {
            advance_temple_gate_alarms(&mut p, &mut map, &mut room);
        }
        assert!(
            !p.temple_gates[0].open,
            "holdingWaitTimer holds the toggle off for 0.2 s"
        );
        advance_temple_gate_alarms(&mut p, &mut map, &mut room);
        assert!(p.temple_gates[0].open);

        // Theo leaves: the wait has to expire *and* the 200 px/s draw animation has to release
        // `lockState` before the toggle may close the gate again.
        p.theo_crystals[0].position = Vec2::new(300.0, 140.0);
        for _ in 0..12 {
            advance_temple_gate_alarms(&mut p, &mut map, &mut room);
        }
        assert!(
            p.temple_gates[0].open,
            "the drawHeight animation still locks the toggle"
        );
        for _ in 0..5 {
            advance_temple_gate_alarms(&mut p, &mut map, &mut room);
        }
        assert!(!p.temple_gates[0].open);
        assert_eq!(map.entities[0].bounds.height, 48.0);
    }

    /// `DashSwitch.Awake`'s gate half (`DashSwitch.cs:135-148`): the restored persistent switch
    /// opens the gate `GetGate` claims, and a foreign id leaves both alone.
    #[test]
    fn a_restored_persistent_switch_opens_the_gate_it_claims() {
        let mut map = dash_switch_and_gate_map(false);
        map.entities[1].single_use = true;
        map.entity_ids = vec![-1, 16];
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 140.0),
            ..PlayerSnapshot::default()
        };
        let mut restored = Simulator::new(p.clone(), &map).unwrap();
        assert_eq!(restored.snapshot().temple_gates[0].current_height, 48.0);
        restored.set_pressed_dash_switches(&[16]);
        assert!(restored.snapshot().temple_gates[0].open);
        assert!(restored.snapshot().temple_gates[0].claimed);
        assert_eq!(restored.runtime_entities()[0].bounds.height, 0.0);

        let mut foreign = Simulator::new(p, &mut map).unwrap();
        foreign.set_pressed_dash_switches(&[15]);
        assert!(!foreign.snapshot().temple_gates[0].open);
        assert_eq!(foreign.runtime_entities()[0].bounds.height, 48.0);
    }

    #[test]
    fn cloud_depresses_then_launches_the_rider_at_the_source_threshold() {
        let p = PlayerSnapshot {
            pos: Vec2::new(100.0, 100.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 70], &cloud_map(false), 70).unwrap();
        assert_eq!(trace.states[1].clouds[0].phase, 1);
        assert_eq!(trace.states[1].clouds[0].speed, 180.0);
        let launched = trace
            .states
            .iter()
            .find(|state| state.speed.y == -200.0)
            .expect("cloud should launch its rider when rebound speed reaches -100");
        assert_eq!(launched.state, PlayerState::Normal);
        assert!(launched.clouds[0].position.y < launched.clouds[0].start.y);
        assert!(trace.states.iter().all(|state| !state.dead));
    }

    #[test]
    fn spiked_cloud_jump_keeps_the_rider_clear_of_the_hazard_below() {
        let p = PlayerSnapshot {
            pos: Vec2::new(100.0, 100.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 70], &cloud_map(true), 70).unwrap();
        assert!(trace.states.iter().any(|state| state.speed.y == -200.0));
        assert!(trace.states.iter().all(|state| !state.dead));
    }

    #[test]
    fn cloud_runtime_keeps_split_simulation_composable() {
        let initial = PlayerSnapshot {
            pos: Vec2::new(100.0, 100.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let inputs = [InputState::default(); 70];
        let mut map = cloud_map(false);
        let whole = simulate(initial.clone(), &inputs, &map, 70).unwrap();
        let first = simulate(initial, &inputs[..35], &map, 35).unwrap();
        let split = simulate(first, &inputs[35..], &map, 35).unwrap();
        assert_eq!(split, whole);
    }

    #[test]
    fn ice_ball_feather_cancel_restores_star_fly_collider_after_normal_hurtbox() {
        let mut bounced = PlayerSnapshot {
            pos: Vec2::new(100.0, 101.0),
            state: PlayerState::StarFly,
            star_fly_timer: 1.0,
            ..PlayerSnapshot::default()
        };
        let mut map = ice_ball_map();
        interact(&mut bounced, &map, InputState::default(), None);
        assert_eq!(bounced.state, PlayerState::Normal);
        assert!(bounced.star_fly_hitbox_preserved);
        assert!(!bounced.ducking);
        assert_eq!(
            current_player_rect(&bounced, bounced.pos.x, bounced.pos.y),
            star_fly_hurt_rect(bounced.pos.x, bounced.pos.y)
        );
        assert_eq!(
            current_player_hurt_rect(&bounced),
            player_hurt_rect(bounced.pos.x, bounced.pos.y)
        );
    }

    #[test]
    fn preserved_star_fly_hurtbox_returns_to_normal_when_falling() {
        let p = PlayerSnapshot {
            pos: Vec2::new(100.0, 80.0),
            speed: Vec2::new(0.0, 1.0),
            star_fly_hitbox_preserved: true,
            ..PlayerSnapshot::default()
        };
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 180.0),
            ..Map::default()
        };
        let p = simulate(p, &[InputState::default()], &map, 1).unwrap();
        assert!(!p.star_fly_hitbox_preserved);
        assert_eq!(
            current_player_rect(&p, p.pos.x, p.pos.y),
            player_rect(p.pos.x, p.pos.y)
        );
    }

    #[test]
    fn playground_ice_ball_dash_bounce_scenario_reaches_the_top_collider() {
        let inputs: Vec<_> = (0..24)
            .map(|frame| InputState {
                move_x: 1,
                move_y: 1,
                jump_held: true,
                dash_pressed: frame == 0,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(
            PlayerSnapshot {
                pos: Vec2::new(317.0, 155.0),
                ..PlayerSnapshot::default()
            },
            &inputs,
            &crate::mechanics_playground(),
            inputs.len() as u32,
        )
        .unwrap();
        let bounced = trace
            .states
            .iter()
            .find(|state| state.state == PlayerState::Normal && state.speed.y == -140.0)
            .expect("down-right dash should top-bounce from the stationary ice ball");
        assert_eq!(bounced.state, PlayerState::Normal);
        assert_eq!(bounced.dashes, 1);
        assert_eq!(bounced.speed.y, -140.0);
        assert!(bounced.speed.x > 160.0);
    }

    #[test]
    fn playground_feather_cancel_scenario_preserves_the_star_fly_collider() {
        let inputs = [InputState {
            move_y: 1,
            ..InputState::default()
        }; 60];
        let trace = simulate_trace(
            PlayerSnapshot {
                pos: Vec2::new(320.0, 120.0),
                ..PlayerSnapshot::default()
            },
            &inputs,
            &crate::mechanics_playground(),
            inputs.len() as u32,
        )
        .unwrap();
        let preserved = trace
            .states
            .iter()
            .find(|state| state.star_fly_hitbox_preserved)
            .expect("downward feather flight should bounce on the aligned ice ball");
        assert_eq!(preserved.state, PlayerState::Normal);
        assert_eq!(preserved.speed.y, -140.0);
        assert!(!preserved.ducking);
    }

    #[test]
    fn grounded_fall_speed_reaches_move_v_collision_and_clears_remainder() {
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            speed: Vec2::new(90.0, 160.0),
            on_ground: true,
            movement_remainder: Vec2::new(0.0, -0.145_548),
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &floor_map(), 1).unwrap();
        assert_eq!(p.pos.y, 100.0);
        assert_eq!(p.speed.y, 0.0);
        assert_eq!(p.movement_remainder.y, 0.0);
    }

    #[test]
    fn badeline_boost_coroutine_enters_launch_on_source_frame() {
        let p = PlayerSnapshot {
            pos: Vec2::new(320.0, 400.0),
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(
            p,
            &[InputState::default(); 27],
            &badeline_boost_map(false),
            27,
        )
        .unwrap();
        assert_eq!(trace.states[1].state, PlayerState::Dummy);
        assert_eq!(trace.states[12].pos, Vec2::new(316.0, 397.0));
        assert_eq!(trace.states[20].pos, Vec2::new(316.0, 402.0));
        assert_eq!(trace.states[27].state, PlayerState::Launch);
        assert_eq!(trace.states[27].speed, Vec2::new(0.0, -330.0));
        assert_eq!(trace.states[27].launch_approach_x, Some(320.0));
    }

    #[test]
    fn badeline_boost_relocates_and_completes_the_full_node_chain() {
        let p = PlayerSnapshot {
            pos: Vec2::new(320.0, 400.0),
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(
            p,
            &[InputState::default(); 120],
            &badeline_boost_map(false),
            120,
        )
        .unwrap();
        assert_eq!(trace.states[51].state, PlayerState::Dummy);
        assert_eq!(trace.states[51].pos, Vec2::new(320.0, 313.0));
        assert_eq!(trace.states[51].movement_remainder, Vec2::default());
        assert_eq!(trace.states[96].state, PlayerState::SummitLaunch);
        assert_eq!(trace.states[96].speed, Vec2::new(0.0, -240.0));
        assert_eq!(trace.states[120].pos, Vec2::new(320.0, 196.0));
    }

    #[test]
    fn final_badeline_boost_uses_slow_wait_freeze_and_summit_launch() {
        let p = PlayerSnapshot {
            pos: Vec2::new(320.0, 400.0),
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(
            p,
            &[InputState::default(); 46],
            &badeline_boost_map(true),
            46,
        )
        .unwrap();
        assert_eq!(trace.states[26].pos, Vec2::new(316.0, 402.0));
        assert_eq!(trace.states[39].freeze_timer, 0.1);
        assert_eq!(trace.states[45].freeze_timer, 0.0);
        assert_eq!(trace.states[46].state, PlayerState::SummitLaunch);
        assert_eq!(trace.states[46].speed, Vec2::new(0.0, -240.0));
        assert_eq!(trace.states[46].summit_launch_target_x, 320.0);
    }

    #[test]
    fn ducking_uses_the_source_six_pixel_collider_under_low_ceilings() {
        let mut map = Map {
            solids: vec![Rect::new(0.0, 90.0, 64.0, 4.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            ducking: true,
            ..PlayerSnapshot::default()
        };
        assert!(!map.solid_at(current_player_rect(&p, p.pos.x, p.pos.y)));
        assert!(!can_unduck(&p, &map));
    }

    #[test]
    fn dummy_state_uses_source_gravity_friction_and_preserves_facing() {
        let p = PlayerSnapshot {
            pos: Vec2::new(160.0, 120.0),
            speed: Vec2::new(200.0, -100.0),
            state: PlayerState::Dummy,
            ..PlayerSnapshot::default()
        };
        let p = simulate(
            p,
            &[InputState {
                move_x: -1,
                ..InputState::default()
            }],
            &Map::default(),
            1,
        )
        .unwrap();
        assert!((p.speed.x - 141.666_58).abs() < 0.001);
        assert!((p.speed.y - -84.999_97).abs() < 0.001);
        assert!(p.facing);
    }

    #[test]
    fn frozen_state_preserves_speed_while_actor_movement_continues() {
        let p = PlayerSnapshot {
            pos: Vec2::new(160.0, 120.0),
            speed: Vec2::new(60.0, 30.0),
            state: PlayerState::Frozen,
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &Map::default(), 1).unwrap();
        assert_eq!(p.state, PlayerState::Frozen);
        assert_eq!(p.speed, Vec2::new(60.0, 30.0));
        assert_eq!(p.pos, Vec2::new(161.0, 121.0));
    }

    #[test]
    fn temple_fall_matches_the_landing_and_one_second_wait_frames() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            solids: vec![Rect::new(0.0, 400.0, 960.0, 144.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(200.0, 300.0),
            state: PlayerState::TempleFall,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 117], &map, 117).unwrap();

        assert_eq!(trace.states[56].pos, Vec2::new(160.0, 400.0));
        assert!(trace.states[56].on_ground);
        assert_eq!(trace.states[56].state, PlayerState::TempleFall);
        assert_eq!(trace.states[116].state, PlayerState::TempleFall);
        assert_eq!(trace.states[117].state, PlayerState::Normal);
    }

    #[test]
    fn reflection_fall_matches_hover_drop_water_and_exit_frames() {
        let p = PlayerSnapshot {
            pos: Vec2::new(504.0, 300.0),
            state: PlayerState::ReflectionFall,
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 216], &water_map(), 216).unwrap();

        assert_eq!(trace.states[120].pos, Vec2::new(504.0, 300.0));
        assert_eq!(trace.states[120].speed, Vec2::default());
        assert_eq!(trace.states[121].pos, Vec2::new(504.0, 305.0));
        assert_eq!(trace.states[121].speed, Vec2::new(0.0, 320.0));
        assert_eq!(trace.states[142].pos, Vec2::new(504.0, 417.0));
        assert_eq!(trace.states[216].state, PlayerState::Swim);
        assert_eq!(trace.states[216].speed, Vec2::new(0.0, -20.0));
    }

    #[test]
    fn launch_uses_half_gravity_and_low_horizontal_friction() {
        let p = PlayerSnapshot {
            pos: Vec2::new(160.0, 120.0),
            speed: Vec2::new(280.0, -150.0),
            state: PlayerState::Launch,
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &Map::default(), 1).unwrap();
        assert_eq!(p.state, PlayerState::Launch);
        assert!((p.speed.x - 276.666_66).abs() < 0.001);
        assert!((p.speed.y - -142.499_98).abs() < 0.001);
    }

    #[test]
    fn summit_launch_uses_the_source_upward_corner_correction() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            solids: vec![Rect::new(480.0, 240.0, 96.0, 24.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
            pos: Vec2::new(480.0, 275.0),
            state: PlayerState::SummitLaunch,
            summit_launch_target_x: 0.0,
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &map, 1).unwrap();
        assert_eq!(p.pos, Vec2::new(476.0, 274.0));
        assert_eq!(p.speed, Vec2::new(0.0, -240.0));
    }

    #[test]
    fn lookout_talk_runs_dummy_wait_hud_camera_and_exit_lifecycle() {
        let player = PlayerSnapshot {
            pos: Vec2::new(144.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![InputState::default(); 150];
        inputs[0].talk_pressed = true;
        for input in &mut inputs[60..110] {
            input.move_x = 1;
        }
        inputs[110].jump_pressed = true;
        inputs[110].jump_held = true;
        let trace =
            simulate_trace(player, &inputs, &lookout_map(vec![], false, false), 150).unwrap();

        // `Interact` starts the entity coroutine; `LookRoutine` assigns Dummy
        // on its following update, matching the real Lookout lifecycle.
        assert_eq!(trace.states[1].state, PlayerState::Normal);
        assert!(trace.states[1].lookouts[0].interacting);
        assert_eq!(trace.states[2].state, PlayerState::Dummy);
        assert!(trace.states[70].lookouts[0].phase >= 4);
        assert!(trace.states[110].camera.x > 0.0);
        // Raw MenuCancel enters HUD hide on f110.  At 3/s the twentieth
        // hide step restores Normal on f130, matching the archived Everest
        // candidate trace's former first mismatch.
        assert_eq!(trace.states[129].state, PlayerState::Dummy);
        assert_eq!(trace.states[130].state, PlayerState::Normal);
        assert!(!trace.states[130].lookouts[0].interacting);
        assert!(!trace.states[150].lookouts[0].interacting);
        assert_eq!(trace.states[150].state, PlayerState::Normal);
    }

    #[test]
    fn lookout_talk_exact_alignment_enters_dummy_on_the_native_second_frame() {
        let player = PlayerSnapshot {
            // `lookout_map` centers its Lookout at (160, 160).  This covers
            // the source's exact-alignment path in DummyWalkToExact rather
            // than the ordinary walking path above.
            pos: Vec2::new(160.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![InputState::default(); 3];
        inputs[0].talk_pressed = true;
        let trace = simulate_trace(player, &inputs, &lookout_map(vec![], false, false), 3).unwrap();

        // Collector captures state 0 before input.  Talk starts LookRoutine
        // on f1, and its first entity update sets StDummy on f2 even though
        // DummyWalkToExact has no distance to walk.  The physical 5.1.4
        // trace has this exact f0-f3 sequence.
        assert_eq!(trace.states[0].state, PlayerState::Normal);
        assert_eq!(trace.states[1].state, PlayerState::Normal);
        assert_eq!(trace.states[2].state, PlayerState::Dummy);
        assert_eq!(trace.states[3].state, PlayerState::Dummy);
        assert!(trace.states[1].lookouts[0].interacting);
        assert!(trace.states[2].lookouts[0].interacting);
        assert_eq!(trace.states[3].lookouts[0].phase, 2);
    }

    #[test]
    fn bino_clip_uses_live_camera_and_spinner_interval_state() {
        let player = PlayerSnapshot {
            pos: Vec2::new(160.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![InputState::default(); 260];
        inputs[0].talk_pressed = true;
        for input in &mut inputs[50..260] {
            input.move_x = 1;
        }
        let trace =
            simulate_trace(player, &inputs, &lookout_map(vec![], false, true), 260).unwrap();

        assert!(trace.states.iter().any(|state| state.spinners[0].visible));
        assert!(trace.states[260].camera.x > 500.0);
        assert!(!trace.states[260].spinners[0].visible);
        assert!(!trace.states[260].spinners[0].collidable);
    }

    #[test]
    fn bino_control_storage_keeps_normal_player_and_camera_control_parallel() {
        let mut map = lookout_map(vec![], false, false);
        map.entities.push(crate::Entity {
            kind: EntityKind::Booster,
            bounds: Rect::new(150.0, 152.0, 20.0, 20.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "booster".to_owned(),
        });
        let player = PlayerSnapshot {
            pos: Vec2::new(144.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![InputState::default(); 240];
        inputs[0].talk_pressed = true;
        for input in &mut inputs[100..180] {
            input.move_x = 1;
        }
        inputs[180].jump_pressed = true;
        inputs[180].jump_held = true;
        let trace = simulate_trace(player, &inputs, &map, 240).unwrap();

        assert_eq!(trace.states[2].state, PlayerState::Dummy);
        assert_eq!(trace.states[2].speed.x, 0.0);
        assert!((trace.states[3].speed.x - 16.666_7).abs() < 0.001);
        // Room entities run before Player: at the first geometric contact
        // frame Booster still sees the preceding x position. The following
        // frame is the first StBoostUpdate, after Dummy has reached 64 px/s.
        let first_boost = trace
            .states
            .iter()
            .position(|state| state.state == PlayerState::Boost)
            .expect("native Booster should interrupt the Lookout Dummy walk");
        assert_eq!(first_boost, 8);
        assert_eq!(trace.states[first_boost - 1].state, PlayerState::Dummy);
        assert!((trace.states[first_boost - 1].speed.x - 64.0).abs() < 0.001);
        // DummyWalkToExact only assigns StDummy before its first yield.  The
        // next frame must therefore remain in Boost instead of being forced
        // back to Dummy by the still-running Lookout coroutine.
        assert_eq!(trace.states[first_boost + 1].state, PlayerState::Boost);
        assert!(
            trace
                .states
                .iter()
                .any(|state| state.state == PlayerState::Boost)
        );
        assert!(
            trace.states.iter().any(|state| {
                state.state == PlayerState::Normal && state.lookouts[0].interacting
            })
        );
        assert!(trace.states.windows(2).any(|states| {
            let (before, after) = (&states[0], &states[1]);
            after.state == PlayerState::Normal
                && after.lookouts[0].interacting
                && (after.pos.x - before.pos.x).abs() > 0.01
                && (after.camera.x - before.camera.x).abs() > 0.01
        }));
        assert!(!trace.states[240].lookouts[0].interacting);
    }

    #[test]
    fn bino_dummy_walk_preserves_speed_through_boost_update() {
        let mut map = lookout_map(vec![], false, false);
        map.bounds = Rect::new(0.0, 0.0, 960.0, 544.0);
        map.solids = vec![Rect::new(0.0, 496.0, 960.0, 48.0)];
        map.entities[0].bounds = Rect::new(510.0, 493.0, 4.0, 4.0);
        map.entities.push(crate::Entity {
            kind: EntityKind::Booster,
            bounds: Rect::new(510.0, 489.0, 20.0, 20.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "booster".to_owned(),
        });
        let player = PlayerSnapshot {
            pos: Vec2::new(496.0, 496.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        };
        let mut inputs = vec![InputState::default(); 30];
        inputs[0].talk_pressed = true;
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();

        // In the source order, Booster sets StBoost after the player update;
        // the yielded DummyWalkToExact resumes afterwards. BoostUpdate moves
        // on f18 but preserves f17's 16.667, so that coroutine write reaches
        // 33.333 before f19 reaches the exact lookout x and finishes.
        assert_eq!(trace.states[17].state, PlayerState::Boost);
        assert!((trace.states[17].speed.x - 16.666_7).abs() < 0.001);
        assert_eq!(trace.states[18].state, PlayerState::Boost);
        assert_eq!(trace.states[18].pos, Vec2::new(511.0, 496.0));
        assert!((trace.states[18].speed.x - 33.333_4).abs() < 0.001);
        assert_eq!(trace.states[19].state, PlayerState::Boost);
        assert_eq!(trace.states[19].pos, Vec2::new(512.0, 496.0));
        assert_eq!(trace.states[19].speed.x, 0.0);
        // Completing DummyWalkToExact advances Lookout into its HUD wait,
        // but that coroutine does not assign StDummy again. The next Player
        // update remains Boost and moves one pixel toward Booster.Center.
        assert_eq!(trace.states[20].state, PlayerState::Boost);
        assert_eq!(trace.states[20].pos, Vec2::new(513.0, 496.0));
        assert_eq!(trace.states[21].pos, Vec2::new(514.0, 496.0));
        assert_eq!(trace.states[24].pos, Vec2::new(516.0, 496.0));
    }

    #[test]
    fn bino_interaction_storage_survives_lookout_room_removal() {
        let mut map = lookout_map(vec![], false, false);
        map.bounds = Rect::new(0.0, 0.0, 320.0, 180.0);
        map.transition_rooms = vec![Rect::new(320.0, 0.0, 320.0, 180.0)];
        map.solids = vec![Rect::new(0.0, 160.0, 640.0, 20.0)];
        map.entities[0].bounds = Rect::new(298.0, 156.0, 4.0, 4.0);
        let player = PlayerSnapshot {
            pos: Vec2::new(316.0, 160.0),
            state: PlayerState::Normal,
            on_ground: true,
            current_room_bounds: Some(map.bounds),
            camera_initialized: true,
            lookouts: vec![crate::LookoutSnapshot {
                interacting: true,
                phase: 4,
                position: Vec2::new(300.0, 160.0),
                hud_easer: 1.0,
                ..crate::LookoutSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let inputs = [InputState {
            move_x: 1,
            ..InputState::default()
        }; 90];
        let result = simulate(player, &inputs, &map, 90).unwrap();

        assert_eq!(result.current_room_bounds, Some(map.transition_rooms[0]));
        assert_eq!(result.state, PlayerState::Normal);
        assert!(result.lookouts[0].removed);
        assert!(result.lookouts[0].interacting);
    }

    #[test]
    fn bino_interaction_storage_uses_a_native_booster_after_dummy_walk() {
        let mut map = lookout_map(vec![], false, false);
        map.bounds = Rect::new(0.0, 0.0, 960.0, 544.0);
        map.transition_rooms = vec![Rect::new(960.0, 0.0, 960.0, 544.0)];
        map.solids = vec![Rect::new(0.0, 496.0, 1920.0, 48.0)];
        map.entities[0].bounds = Rect::new(938.0, 493.0, 4.0, 4.0);
        map.entities.push(crate::Entity {
            kind: EntityKind::Booster,
            bounds: Rect::new(924.0, 491.0, 16.0, 16.0),
            direction: Vec2::default(),
            shielded: false,
            single_use: false,
            nodes: vec![],
            name: "interruptingBooster".to_owned(),
        });
        let player = PlayerSnapshot {
            pos: Vec2::new(916.0, 496.0),
            on_ground: true,
            current_room_bounds: Some(map.bounds),
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..300)
            .map(|frame| InputState {
                talk_pressed: frame == 0,
                move_x: (frame >= 120).then_some(1).unwrap_or_default(),
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(player, &inputs, &map, inputs.len() as u32).unwrap();

        assert!(
            trace
                .states
                .iter()
                .any(|state| state.state == PlayerState::Boost)
        );
        assert!(
            trace
                .states
                .iter()
                .any(|state| { state.state == PlayerState::Dash && state.booster_boosting })
        );
        let f39 = &trace.states[39];
        assert_eq!(f39.state, PlayerState::Dash);
        assert!(
            f39.booster_boosting,
            "active Booster suppresses same-target re-entry"
        );
        // The transition camera routine pauses Player.Update, but the source
        // capture's `Actor.OnGround()` remains a live floor collision query.
        assert!(trace.states[125].on_ground);
        assert!(
            trace
                .states
                .iter()
                .any(|state| state.current_room_bounds == Some(map.transition_rooms[0]))
        );
    }

    #[test]
    fn bino_extensions_follow_nodes_and_run_long_distance_exit_wipe() {
        let mut map = lookout_map(vec![Vec2::new(960.0, 90.0)], true, false);
        let player = PlayerSnapshot {
            pos: Vec2::new(160.0, 160.0),
            state: PlayerState::Dummy,
            on_ground: true,
            camera_initialized: true,
            lookouts: vec![crate::LookoutSnapshot {
                interacting: true,
                phase: 4,
                position: Vec2::new(160.0, 160.0),
                cam: Vec2::default(),
                cam_start: Vec2::default(),
                hud_easer: 1.0,
                ..crate::LookoutSnapshot::default()
            }],
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..600)
            .map(|frame| InputState {
                move_y: -1,
                // Summit arrival alone keeps LookRoutine active. The
                // collector maps this press to MenuCancel, which begins the
                // source exit and its one-second summit wipe.
                jump_pressed: frame == 520,
                jump_held: frame == 520,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(player, &inputs, &map, 600).unwrap();

        assert!(trace.states.iter().any(|state| state.camera.x > 600.0));
        assert_eq!(trace.states[519].lookouts[0].phase, 4);
        assert_eq!(trace.states[519].lookouts[0].node_percent, 1.0);
        assert_eq!(trace.states[520].state, PlayerState::Dummy);
        assert!(trace.states[520].lookouts[0].interacting);
        assert!(
            trace
                .states
                .iter()
                .any(|state| state.lookouts[0].phase == 6)
        );
        let exit = trace
            .states
            .iter()
            .find(|state| !state.lookouts[0].interacting)
            .expect("long-distance wipe completes");
        assert!((exit.camera.x - 32.0).abs() < 0.01);
        // A 1-second summit FadeWipe starts after frame 520's MenuCancel.
        // At f581 LookRoutine resumes, clears interacting, and restores
        // StNormal; this mirrors the physical 5.1.4 trace.
        assert!(!trace.states[581].lookouts[0].interacting);
        assert_eq!(trace.states[581].state, PlayerState::Normal);
    }

    #[test]
    fn cloud_hyper_bunnyhop_fixture_leaves_the_platform_side_before_apex_landing() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            solids: vec![
                Rect::new(0.0, 496.0, 960.0, 48.0),
                Rect::new(544.0, 416.0, 160.0, 8.0),
            ],
            entities: vec![crate::Entity {
                kind: EntityKind::Cloud,
                bounds: Rect::new(504.0, 434.0, 32.0, 5.0),
                direction: Vec2::default(),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "cloud".to_owned(),
            }],
            ..Map::default()
        };
        let inputs: Vec<_> = (0..45)
            .map(|frame| InputState {
                move_x: if (24..=28).contains(&frame) {
                    -1
                } else if frame >= 29 {
                    1
                } else {
                    0
                },
                crouch_dash_pressed: frame == 24,
                jump_pressed: frame == 29 || frame == 38,
                jump_held: frame == 29 || frame == 38,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(
            PlayerSnapshot {
                pos: Vec2::new(520.0, 434.0),
                ..PlayerSnapshot::default()
            },
            &inputs,
            &map,
            inputs.len() as u32,
        )
        .unwrap();

        assert_eq!(trace.states[30].speed.x, 325.0);
        assert_eq!(trace.states[30].pos, Vec2::new(521.0, 418.0));
        assert_eq!(trace.states[35].pos, Vec2::new(547.0, 415.0));
        assert!((trace.states[35].movement_remainder.y + 0.5).abs() < 0.000_1);
        assert!(trace.states[36].pos.x > 544.0);
        assert!(trace.states[38].on_ground);
        assert!(trace.states[39].speed.x > 300.0 && trace.states[39].speed.y < -160.0);
    }

    #[test]
    fn real_trace_delta_time_controls_the_matching_player_frame() {
        let state = simulate(
            PlayerSnapshot {
                pos: Vec2::new(160.0, 160.0),
                ..PlayerSnapshot::default()
            },
            &[InputState {
                frame_delta_time_bits: Some(0.02_f32.to_bits()),
                ..InputState::default()
            }],
            &Map::default(),
            1,
        )
        .unwrap();

        assert!((state.frame_delta_time - 0.02).abs() < 0.000_001);
        assert!((state.speed.y - 18.0).abs() < 0.000_001);
    }

    #[test]
    fn roboboost_fixture_restores_climb_jump_speed_before_reversing_input() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            solids: vec![
                Rect::new(0.0, 496.0, 960.0, 48.0),
                Rect::new(448.0, 432.0, 8.0, 8.0),
            ],
            entities: vec![crate::Entity {
                kind: EntityKind::MoveBlock,
                bounds: Rect::new(400.0, 464.0, 64.0, 16.0),
                direction: Vec2::new(0.0, -1.0),
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "moveBlock".to_owned(),
            }],
            ..Map::default()
        };
        let inputs: Vec<_> = (0..90)
            .map(|frame| InputState {
                move_x: if (45..58).contains(&frame) {
                    1
                } else if frame >= 58 {
                    -1
                } else {
                    0
                },
                crouch_dash_pressed: frame == 45,
                jump_pressed: frame == 49 || frame == 51,
                jump_held: frame == 49 || frame == 51,
                grab_held: frame == 51,
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(
            PlayerSnapshot {
                pos: Vec2::new(432.0, 464.0),
                on_ground: true,
                ..PlayerSnapshot::default()
            },
            &inputs,
            &map,
            inputs.len() as u32,
        )
        .unwrap();

        assert!(trace.states[50].speed.x > 300.0);
        assert!(trace.states[52].wall_speed_retention_timer > 0.05);
        assert!(trace.states[52].wall_speed_retained > 300.0);
        assert!(trace.states[55].speed.x > 300.0);
        assert!(trace.states[59].speed.x < trace.states[58].speed.x);
        assert_eq!(trace.states[51].move_blocks[0].position.y, 440.0);
    }

    /// `Player.ClimbUpdate`'s grab-release branch (`Player.cs:3950-3955`) adds
    /// `LiftBoost` before returning to `StNormal`. Releasing a grab on a rising
    /// lift therefore hands the lift's speed to the player instead of leaving
    /// the `StClimb` zero.
    #[test]
    fn climb_release_adds_lift_boost() {
        let mut map = floor_map();
        let mut p = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            state: PlayerState::Climb,
            facing: true,
            current_lift_speed: Vec2::new(121.083_748, 0.0),
            lift_speed_timer: 0.16,
            ..PlayerSnapshot::default()
        };

        // `input.grab_held == false` and no jump/dash press: the first branch
        // `ClimbUpdate` can take is the grab release.
        climb_update(&mut p, InputState::default(), &map, &mut None);

        assert_eq!(p.state, PlayerState::Normal);
        assert_eq!(p.speed.x, 121.083_748);
        assert_eq!(p.max_fall, MAX_FALL);
    }

    /// `Player.OnCollideV`'s rising branch (`Player.cs:3389-3392`) clears
    /// `varJumpTimer` on a ceiling bonk once the jump has been rising for more
    /// than 0.05 s, i.e. once the timer has fallen below 0.15 s. While the timer
    /// is still at or above 0.15 the variable-jump window survives, so the
    /// frames after the bonk keep the half-gravity fall target.
    #[test]
    fn ceiling_bonk_ends_the_variable_jump_window() {
        // `player_rect` puts the hitbox at `y - 11 .. y`; a ceiling ending at
        // y = 89 blocks the first pixel of an upward move from y = 100.
        let mut map = Map {
            solids: vec![Rect::new(0.0, 0.0, 320.0, 89.0)],
            ..Map::default()
        };

        let mut late = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            speed: Vec2::new(0.0, -105.0),
            var_jump_timer: 0.133_333_3,
            var_jump_speed: -105.0,
            ..PlayerSnapshot::default()
        };
        let amount = late.speed.y * DT;
        move_axis_amount(&mut late, &mut map, false, amount);
        assert_eq!(late.speed.y, 0.0);
        assert_eq!(late.var_jump_timer, 0.0);

        // A timer at 0.15 s or above means the jump has been rising for less
        // than 0.05 s, so the window is left alone.
        let mut early = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            speed: Vec2::new(0.0, -105.0),
            var_jump_timer: 0.166_666_6,
            var_jump_speed: -105.0,
            ..PlayerSnapshot::default()
        };
        let amount = early.speed.y * DT;
        move_axis_amount(&mut early, &mut map, false, amount);
        assert_eq!(early.speed.y, 0.0);
        assert_eq!(early.var_jump_timer, 0.166_666_6);
    }

    /// `Player.DreamDashUpdate` returns state 0 (`Player.cs:5240`), so
    /// `StateMachine` runs `DreamDashEnd` and then `NormalBegin`, whose
    /// `maxFall = 160f` (`Player.cs:3531-3534`) must replace the Core's
    /// 240 px/s fast-fall cap.
    #[test]
    fn dream_dash_exit_runs_normal_begin() {
        let mut map = Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 240.0),
            ..Map::default()
        };
        let mut p = PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            state: PlayerState::DreamDash,
            dash_dir: Vec2::new(0.0, 1.0),
            max_fall: FAST_MAX_FALL,
            ..PlayerSnapshot::default()
        };
        p.dream_dash_can_end_timer = 0.0;

        try_end_dream_dash(&mut p, &map, InputState::default());

        assert_eq!(p.state, PlayerState::Normal);
        assert_eq!(p.max_fall, MAX_FALL);
        assert!(p.auto_jump);
    }

    /// `Player.NormalUpdate`'s Core ice factor (`Player.cs:3681-3684`):
    /// `num2 *= 0.3f` while grounded and `level.CoreMode == Cold`, which shrinks
    /// the 400 px/s `Calc.Approach` step to 120 px/s.
    #[test]
    fn core_ice_mode_scales_the_ground_run_approach() {
        let mut map = floor_map();
        let grounded = |core_mode| PlayerSnapshot {
            pos: Vec2::new(32.0, 100.0),
            on_ground: true,
            speed: Vec2::new(299.0, 0.0),
            move_x: 1,
            max_fall: MAX_FALL,
            core_mode,
            ..PlayerSnapshot::default()
        };
        let input = InputState {
            move_x: 1,
            ..InputState::default()
        };

        let mut normal = grounded(crate::CoreMode::None);
        normal_update(&mut normal, input, &map, true);
        let mut cold = grounded(crate::CoreMode::Cold);
        normal_update(&mut cold, input, &map, true);

        let expected_normal = 299.0 - RUN_REDUCE * DT;
        let expected_cold = 299.0 - RUN_REDUCE * ICE_GROUND_MULT * DT;
        assert!((normal.speed.x - expected_normal).abs() < 0.001);
        assert!((cold.speed.x - expected_cold).abs() < 0.001);
        assert!((cold.speed.x - normal.speed.x - RUN_REDUCE * 0.7 * DT).abs() < 0.001);
    }

    fn dash_collide_block_map(kind: EntityKind, bounds: Rect, direction: Vec2) -> Map {
        Map {
            entities: vec![crate::Entity {
                kind,
                bounds,
                direction,
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: match kind {
                    EntityKind::CrushBlock => "crushBlock".to_owned(),
                    _ => "dashBlock".to_owned(),
                },
            }],
            ..Map::default()
        }
    }

    /// `CrushBlock` is a `Solid` (`CrushBlock.cs:9,85-87`) and its
    /// `OnDashCollide` rebounds a dash that `CanActivate` accepts
    /// (`CrushBlock.cs:274-282`). `Player.OnCollideH` then runs
    /// `Rebound(-Math.Sign(Speed.X))` (`Player.cs:3168-3170`), i.e.
    /// `Speed = (-120, -120)` with `AutoJump`, `varJumpTimer = 0.15` and
    /// `StateMachine.State = 0` (`Player.cs:2784-2800`).
    #[test]
    fn crush_block_rebounds_a_horizontal_dash() {
        // `axes = Both` (0), `chillout = false`.
        let mut map = dash_collide_block_map(
            EntityKind::CrushBlock,
            Rect::new(80.0, 96.0, 32.0, 32.0),
            Vec2::new(0.0, 0.0),
        );
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 100.0),
            on_ground: true,
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState {
            move_x: 1,
            ..InputState::default()
        }; 12];
        inputs[0].dash_pressed = true;
        let trace = simulate_trace(p, &inputs, &map, inputs.len() as u32).unwrap();
        let rebound = trace
            .states
            .iter()
            .find(|state| state.auto_jump)
            .expect("the dash must rebound");
        assert_eq!(rebound.speed, Vec2::new(-120.0, -120.0));
        assert_eq!(rebound.state, PlayerState::Normal);
        assert_eq!(rebound.var_jump_timer, 0.15);
        assert_eq!(rebound.var_jump_speed, -120.0);
        assert_eq!(rebound.dash_attack_timer, 0.0);
        assert_eq!(rebound.wall_slide_timer, 1.2);
        // `Attack` clears `canActivate` (`CrushBlock.cs:329`).
        assert!(!rebound.crush_blocks[0].can_activate);
        assert_eq!(rebound.crush_blocks[0].crush_dir, Vec2::new(-1.0, 0.0));
    }

    /// `CrushBlock.CanActivate` refuses a direction the `axes` limit forbids
    /// (`CrushBlock.cs:290-299`), so a horizontal dash into a vertical-only
    /// crusher is an ordinary `OnCollideH` stop: `Speed.X = 0`, no rebound.
    #[test]
    fn vertical_only_crush_block_does_not_rebound_a_horizontal_dash() {
        // `axes = Vertical` (2).
        let mut map = dash_collide_block_map(
            EntityKind::CrushBlock,
            Rect::new(80.0, 96.0, 32.0, 32.0),
            Vec2::new(2.0, 0.0),
        );
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 100.0),
            on_ground: true,
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState {
            move_x: 1,
            ..InputState::default()
        }; 12];
        inputs[0].dash_pressed = true;
        let trace = simulate_trace(p, &inputs, &map, inputs.len() as u32).unwrap();
        assert!(
            trace.states.iter().all(|state| !state.auto_jump),
            "a vertical-only crusher must not rebound a horizontal dash"
        );
        assert!(trace.states.iter().all(|state| state.speed.x <= 240.0));
        // `CrushBlock.cs:290-291`: `crushDir == direction` blocks a repeat.
        assert!(
            trace
                .states
                .iter()
                .all(|state| state.crush_blocks[0].can_activate)
        );
    }

    /// `DashBlock.OnDashed` refuses the break while `canDash` is false and the
    /// player is neither state 5 nor 10 (`DashBlock.cs:133-136`), which leaves
    /// `Player.cs:3178-3219`'s ordinary stop in place.
    #[test]
    fn non_dashable_dash_block_stops_a_normal_dash() {
        let mut map = dash_collide_block_map(
            EntityKind::DashBlock,
            Rect::new(80.0, 88.0, 16.0, 16.0),
            // `canDash = false`, `permanent = true`.
            Vec2::new(0.0, 1.0),
        );
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 100.0),
            on_ground: true,
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState {
            move_x: 1,
            ..InputState::default()
        }; 12];
        inputs[0].dash_pressed = true;
        let trace = simulate_trace(p, &inputs, &map, inputs.len() as u32).unwrap();
        let stopped = trace
            .states
            .iter()
            .find(|state| state.speed.x == 0.0 && !state.auto_jump)
            .expect("the dash must stop against the block");
        assert_eq!(stopped.state, PlayerState::Normal);
        assert!(!stopped.dash_blocks[0].broken);
    }

    /// A `canDash` `DashBlock` breaks and rebounds
    /// (`DashBlock.cs:131-139`), which also removes the Solid for good.
    #[test]
    fn dashable_dash_block_breaks_and_rebounds() {
        let mut map = dash_collide_block_map(
            EntityKind::DashBlock,
            Rect::new(80.0, 88.0, 16.0, 16.0),
            // `canDash = true`, `permanent = true`.
            Vec2::new(1.0, 1.0),
        );
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 100.0),
            on_ground: true,
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState {
            move_x: 1,
            ..InputState::default()
        }; 12];
        inputs[0].dash_pressed = true;
        let trace = simulate_trace(p, &inputs, &map, inputs.len() as u32).unwrap();
        let rebound = trace
            .states
            .iter()
            .find(|state| state.auto_jump)
            .expect("the block must break and rebound the dash");
        assert_eq!(rebound.speed, Vec2::new(-120.0, -120.0));
        assert!(rebound.dash_blocks[0].broken);
        // `DashBlock.Break` -> `Collidable = false` (`DashBlock.cs:114`), modelled
        // by parking the collider out of the room on the next `Simulator::new`.
        let simulator = Simulator::new(rebound.clone(), &map).unwrap();
        assert!(!solid_is_collidable(&simulator.runtime_entities()[0]));
    }

    /// `Sides.Right` dash switch: 8x16 at the entity position with `pressDirection = UnitX`
    /// (`DashSwitch.cs:68-70,87-90`).
    fn dash_switch_map(press_direction: Vec2) -> Map {
        Map {
            bounds: Rect::new(0.0, 0.0, 320.0, 176.0),
            entities: vec![crate::Entity {
                kind: EntityKind::DashSwitch,
                bounds: Rect::new(80.0, 96.0, 8.0, 16.0),
                direction: press_direction,
                shielded: false,
                single_use: false,
                nodes: vec![],
                name: "dashSwitchH".to_owned(),
            }],
            ..Map::default()
        }
    }

    /// `DashSwitch.OnDashed` (`DashSwitch.cs:195-229`): a dash along `pressDirection` presses the
    /// button, which then sets `Collidable = false` (`:204`), so the player keeps going through
    /// the space it occupied. The result is always `NormalCollision` (`:228`) - no rebound.
    #[test]
    fn dash_switch_presses_on_the_matching_dash_direction() {
        let mut map = dash_switch_map(Vec2::new(1.0, 0.0));
        // Feet at y=106 against a button spanning y=96..112: a ten pixel overlap, so the
        // four-pixel dash corner correction cannot lift the player over it
        // (`Player.cs` `OnCollideH`: `for (int i = 1; i <= DashCornerCorrection; i++)`).
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 106.0),
            on_ground: true,
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState {
            move_x: 1,
            ..InputState::default()
        }; 16];
        inputs[0].dash_pressed = true;
        let mut simulator = Simulator::new(p, &mut map).unwrap();
        for input in &inputs {
            simulator.step(*input).unwrap();
        }
        assert!(
            !solid_is_collidable(&simulator.runtime_entities()[0]),
            "a pressed dash switch must stop being collidable"
        );
        // `NormalCollision` (`:228`) leaves the dash alive and the button gone, so the player
        // runs past the right face at x=88 instead of rebounding out of the room.
        assert!(
            simulator.snapshot().pos.x > 88.0,
            "the player must continue through the pressed button, got x={}",
            simulator.snapshot().pos.x
        );
    }

    /// The press is gated on `direction == pressDirection` (`DashSwitch.cs:197`), so the same
    /// collider refuses a dash from the other side and stays a solid wall.
    #[test]
    fn dash_switch_ignores_a_dash_from_the_wrong_side() {
        // `Sides.Left` (`:93-97`): `pressDirection = -UnitX`, pressed only by a leftward dash.
        let mut map = dash_switch_map(Vec2::new(-1.0, 0.0));
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 106.0),
            on_ground: true,
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState {
            move_x: 1,
            ..InputState::default()
        }; 12];
        inputs[0].dash_pressed = true;
        let mut simulator = Simulator::new(p, &mut map).unwrap();
        for input in &inputs {
            simulator.step(*input).unwrap();
        }
        assert!(solid_is_collidable(&simulator.runtime_entities()[0]));
        // Blocked with the 8 px collider's right face flush against x=80.
        assert_eq!(simulator.snapshot().pos.x, 76.0);
    }

    /// A `dashSwitchH` with `persistent` (`DashSwitch.cs:106`) and the map `id` its session flag
    /// would be keyed by (`EntityID.cs:18-30`).
    fn persistent_dash_switch_map() -> Map {
        let mut map = dash_switch_map(Vec2::new(1.0, 0.0));
        map.entities[0].single_use = true;
        map.entity_ids = vec![16];
        map
    }

    /// `DashSwitch.Awake` (`DashSwitch.cs:124-149`): a persistent switch whose
    /// `dashSwitch_<room>:<id>` flag is already set comes back pushed - `Position =
    /// pressedTarget - pressDirection * 2f`, `Collidable = false` - so it is *not* a Solid when the
    /// segment starts, and the same rightward dash that presses a fresh button runs straight
    /// through this one.
    #[test]
    fn persistent_dash_switch_with_the_session_flag_set_starts_pressed() {
        let mut map = persistent_dash_switch_map();
        let p = PlayerSnapshot {
            pos: Vec2::new(60.0, 106.0),
            on_ground: true,
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut inputs = [InputState {
            move_x: 1,
            ..InputState::default()
        }; 16];
        inputs[0].dash_pressed = true;

        let mut restored = Simulator::new(p.clone(), &map).unwrap();
        restored.set_pressed_dash_switches(&[16]);
        assert!(
            !solid_is_collidable(&restored.runtime_entities()[0]),
            "the flag's switch must start non-collidable"
        );
        for input in &inputs {
            restored.step(*input).unwrap();
        }
        assert!(
            restored.snapshot().pos.x > 88.0,
            "the player must run through the already-pressed button, got x={}",
            restored.snapshot().pos.x
        );

        // Without the flag the same setup is the ordinary collidable button.
        let mut fresh = Simulator::new(p, &mut map).unwrap();
        fresh.set_pressed_dash_switches(&[]);
        assert!(solid_is_collidable(&fresh.runtime_entities()[0]));
    }

    /// The restore is keyed by `EntityID.ID`, and only a `persistent` switch can ever be named by a
    /// `dashSwitch_*` flag (`DashSwitch.cs:223-226` writes it only under `persistent`), so neither a
    /// non-persistent switch nor a persistent one with a different id may be pressed by the flag.
    #[test]
    fn dash_switch_flag_only_presses_the_persistent_switch_it_names() {
        let mut non_persistent = dash_switch_map(Vec2::new(1.0, 0.0));
        non_persistent.entity_ids = vec![16];
        let mut simulator = Simulator::new(PlayerSnapshot::default(), &non_persistent).unwrap();
        simulator.set_pressed_dash_switches(&[16]);
        assert!(
            solid_is_collidable(&simulator.runtime_entities()[0]),
            "a non-persistent switch never writes a session flag"
        );

        let persistent = persistent_dash_switch_map();
        let mut simulator = Simulator::new(PlayerSnapshot::default(), &persistent).unwrap();
        simulator.set_pressed_dash_switches(&[15]);
        assert!(
            solid_is_collidable(&simulator.runtime_entities()[0]),
            "a flag naming another id must not press this switch"
        );
    }
    /// `Session.Cassette` (`Cassette.CollectRoutine`, `Cassette.cs:176`) is chapter state: once the
    /// tape is taken in an A-side, `Level.ShouldCreateCassetteManager` (`Level.cs:278-288`) is false,
    /// so the game constructs no `CassetteBlockManager` (`:657`, `:1355-1358`), its
    /// `SilentUpdateBlocks` -> `SetActivatedSilently` (`CassetteBlockManager.cs:197-206`,
    /// `CassetteBlock.cs:392-394`) never runs, and every block keeps the `Collidable = false` from
    /// its constructor (`CassetteBlock.cs:70-76`) however long the room runs.
    #[test]
    fn cassette_tape_taken_leaves_no_manager_and_no_collidable_block() {
        let mut map = cassette_map();
        let snapshot = || PlayerSnapshot {
            pos: Vec2::new(100.0, 60.0),
            ..PlayerSnapshot::default()
        };

        // Without the flag nothing changes: the manager exists and the current index is solid.
        let mut kept = Simulator::new(snapshot(), &map).unwrap();
        assert!(kept.snapshot().cassette_manager.initialized);
        assert!(
            kept.snapshot()
                .cassette_blocks
                .iter()
                .any(|state| state.collidable)
        );
        assert!(solid_is_collidable(&kept.runtime_entities()[1]));

        let mut taken = Simulator::new(snapshot(), &map).unwrap();
        taken.set_cassette_taken(true);
        assert!(!taken.snapshot().cassette_manager.initialized);
        assert_eq!(taken.snapshot().cassette_manager.max_beat, 0);
        assert_eq!(taken.snapshot().cassette_manager.beat_index, 0);

        // Four cassette beats, i.e. past the `(beat_index + 1) % 8 == 0` reform beat the untaken
        // case raises a block on, and no block may become collidable on any of them.
        for _ in 0..40 {
            taken.step(InputState::default()).unwrap();
            assert!(
                taken
                    .snapshot()
                    .cassette_blocks
                    .iter()
                    .all(|state| !state.collidable)
            );
            assert_eq!(taken.snapshot().cassette_manager.beat_index, 0);
        }
        assert!(
            taken
                .snapshot()
                .cassette_blocks
                .iter()
                .all(|state| !state.activated)
        );
        assert!(
            taken
                .runtime_entities()
                .iter()
                .all(|entity| !solid_is_collidable(entity))
        );
    }
}