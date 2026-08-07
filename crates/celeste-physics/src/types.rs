use serde::{Deserialize, Serialize};

use crate::Rect;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PlayerState {
    #[default]
    Normal = 0,
    Climb = 1,
    Dash = 2,
    Swim = 3,
    Boost = 4,
    RedDash = 5,
    HitSquash = 6,
    Launch = 7,
    Pickup = 8,
    DreamDash = 9,
    SummitLaunch = 10,
    Dummy = 11,
    IntroWalk = 12,
    IntroJump = 13,
    IntroRespawn = 14,
    IntroWakeUp = 15,
    BirdDashTutorial = 16,
    Frozen = 17,
    ReflectionFall = 18,
    StarFly = 19,
    TempleFall = 20,
    CassetteFly = 21,
    Attract = 22,
    IntroMoonJump = 23,
    FlingBird = 24,
    IntroThinkForABit = 25,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct InputState {
    pub move_x: i8,
    pub move_y: i8,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub dash_pressed: bool,
    #[serde(default)]
    pub crouch_dash_pressed: bool,
    pub grab_held: bool,
    #[serde(default)]
    pub talk_pressed: bool,
    /// Exact `Engine.DeltaTime` bits captured from a real Everest frame. This
    /// is diagnostic replay data only, so portable scenario inputs omit it.
    #[serde(skip)]
    pub frame_delta_time_bits: Option<u32>,
}

impl InputState {
    pub fn normalized(mut self) -> Self {
        self.move_x = self.move_x.clamp(-1, 1);
        self.move_y = self.move_y.clamp(-1, 1);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ZipMoverSnapshot {
    /// Source coroutine phase: waiting, start delay, outbound, target delay,
    /// return, or start delay.
    pub phase: u8,
    /// Coroutine float-yield timer. Like Monocle.Coroutine, a frame that
    /// crosses zero only resumes the iterator on the following update.
    pub wait_timer: f32,
    /// Current outbound/return interpolation cursor before Ease.SineIn.
    pub at: f32,
    /// Integer Platform.Position used for collision and carrying.
    pub position: Vec2,
    /// Platform movementCounter retained by MoveTo across frames.
    pub remainder: Vec2,
    /// Platform.LiftSpeed components last written by MoveToX/MoveToY.
    pub lift_speed: Vec2,
    /// Original entity position captured before the runtime map is moved.
    pub start: Vec2,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BounceBlockSnapshot {
    /// Waiting, winding up, bouncing, bounce end, or broken.
    pub phase: u8,
    pub move_speed: f32,
    pub bounce_dir: Vec2,
    pub bounce_lift: Vec2,
    pub bounce_end_timer: f32,
    pub respawn_timer: f32,
    pub position: Vec2,
    pub remainder: Vec2,
    pub lift_speed: Vec2,
    pub start: Vec2,
    /// Alarm delay between the body becoming collidable again and its
    /// attached StaticMovers being enabled.
    pub reform_timer: f32,
    pub static_movers_enabled: bool,
    pub attached_spike_index: Option<u16>,
    pub attached_spike_position: Vec2,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MoveBlockSnapshot {
    /// Idling, activation delay, moving, break delay, hidden, or reforming.
    pub phase: u8,
    pub wait_timer: f32,
    pub speed: f32,
    pub angle: f32,
    pub crash_timer: f32,
    pub crash_reset_timer: f32,
    pub no_steer_timer: f32,
    pub position: Vec2,
    pub remainder: Vec2,
    pub lift_speed: Vec2,
    pub start: Vec2,
    pub visible: bool,
    pub static_movers_enabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TheoCrystalSnapshot {
    /// Actor.Position: bottom-center of the 8x10 body collider.
    pub position: Vec2,
    pub speed: Vec2,
    pub remainder: Vec2,
    pub held: bool,
    pub cannot_hold_timer: f32,
    pub gravity_timer: f32,
    /// TheoCrystal.Die disables pushing and kills the player after failed squish escape.
    pub dead: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HeartGemSnapshot {
    /// Idle, one-frame pre-freeze yield, frozen yield, or time-rate cutscene.
    pub phase: u8,
    pub wait_frames: u8,
    pub collected: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreMode {
    #[default]
    None,
    Hot,
    Cold,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RisingLavaSnapshot {
    /// Entity.Position, which is also the top-left of the 340x120 lethal hitbox.
    pub position: Vec2,
    pub waiting: bool,
    pub ice_mode: bool,
    pub intro: bool,
    pub delay: f32,
    pub initialized: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SandwichLavaSnapshot {
    /// Entity.Position, shared by the bottom hitbox and the visual components.
    pub position: Vec2,
    pub start_x: f32,
    pub waiting: bool,
    pub ice_mode: bool,
    pub leaving: bool,
    pub persistent: bool,
    pub removed: bool,
    pub delay: f32,
    pub leave_timer: f32,
    /// Source-local LavaRect offsets; kept to preserve Waiting/leaving lifecycle.
    pub top_rect_y: f32,
    pub bottom_rect_y: f32,
    pub initialized: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GliderSnapshot {
    /// Actor.Position: bottom-center of the 8x10 body collider.
    pub position: Vec2,
    pub speed: Vec2,
    pub remainder: Vec2,
    pub held: bool,
    pub cannot_hold_timer: f32,
    pub gravity_timer: f32,
    pub no_gravity_timer: f32,
    pub high_friction_timer: f32,
    /// Glider.OnSquish removes the actor when both wiggle searches fail.
    pub removed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CloudSnapshot {
    /// Vanilla Cloud.Update phase: waiting, rebounding, or returning.
    pub phase: u8,
    /// Vertical speed integrated by the cloud's source-ordered state machine.
    pub speed: f32,
    /// Integer JumpThru position used for collision and carrying.
    pub position: Vec2,
    /// Platform movementCounter retained by MoveV across frames.
    pub remainder_y: f32,
    /// Original entity position captured before the runtime map is moved.
    pub start: Vec2,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SeekerSnapshot {
    /// Actor.Position / physics-hitbox center.
    pub position: Vec2,
    pub speed: Vec2,
    pub remainder: Vec2,
    /// Vanilla Seeker state index (Attack=3, Stunned=4, Skidding=5).
    pub state: u8,
    /// Coroutine time remaining for the supported Stunned lifecycle.
    pub state_timer: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CassetteManagerSnapshot {
    /// Whether Level.LoadLevel has run CassetteBlockManager.OnLevelStart.
    pub initialized: bool,
    /// The non-level-music manager creates its cassette song on its first
    /// Update and does not call AdvanceMusic until the following frame.
    pub startup_music_pending: bool,
    /// Accumulated sixteenth-note time, advanced in single precision.
    pub beat_timer: f32,
    pub beat_index: u8,
    pub current_index: u8,
    pub max_beat: u8,
    pub tempo_mult: f32,
}

impl Default for CassetteManagerSnapshot {
    fn default() -> Self {
        Self {
            initialized: false,
            startup_music_pending: false,
            beat_timer: 0.0,
            beat_index: 0,
            current_index: 0,
            max_beat: 0,
            tempo_mult: 1.0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TempleGateSnapshot {
    /// Original top-left entity position restored after SetHeight.
    pub position: Vec2,
    pub current_height: f32,
    pub closed_height: f32,
    pub open: bool,
    pub triggered: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CassetteBlockSnapshot {
    /// Current Solid position, including the two one-pixel ShiftSize phases.
    pub position: Vec2,
    pub start: Vec2,
    pub width: f32,
    pub height: f32,
    pub index: u8,
    pub activated: bool,
    pub collidable: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpinnerSnapshot {
    /// CrystalStaticSpinner world-space center.
    pub position: Vec2,
    /// Per-instance Calc.Random.NextFloat offset used by Scene.OnInterval.
    pub offset: f32,
    pub visible: bool,
    pub collidable: bool,
}

/// Per-entity Bumper state. `position` is its live Entity.Position (the
/// centre of the Circle(12) collider), while `sine_counter` is the randomized
/// SineWave phase which advances at 0.44 radians per second.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BumperSnapshot {
    /// Immutable room position passed to Bumper's constructor.
    pub anchor: Vec2,
    pub position: Vec2,
    pub sine_counter: f32,
    /// Bumper's own respawn timer. This belongs to the entity rather than
    /// Player, because its moving position cannot safely identify it.
    pub respawn_timer: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LookoutSnapshot {
    pub interacting: bool,
    /// `Lookout.Removed` restores the player state but deliberately does not
    /// clear the entity-owned interaction flag.
    pub removed: bool,
    /// 0 idle, 1 Dummy alignment, 2 pre-HUD wait, 3 HUD ease-in,
    /// 4 camera control, 5 HUD ease-out, 6 long-distance exit wipe.
    pub phase: u8,
    pub timer: f32,
    pub position: Vec2,
    pub cam_start: Vec2,
    pub cam: Vec2,
    pub cam_speed: Vec2,
    /// Camera position captured before the long-distance FadeWipe begins.
    pub wipe_start: Vec2,
    pub node: u16,
    pub node_percent: f32,
    pub hud_easer: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RefillSnapshot {
    /// Regular diamond (one dash) or pink Farewell diamond (two dashes).
    pub two_dashes: bool,
    /// `oneUse` removes the entity instead of respawning after 2.5 seconds.
    pub one_use: bool,
    /// Remaining seconds before `Respawn` restores collidability. Touching a
    /// refill while `UseRefill` succeeds sets this to 2.5.
    pub respawn_timer: f32,
    /// Mirrors the entity `Collidable` flag. The 16x16 PlayerCollider only
    /// fires while this is true.
    pub collidable: bool,
    /// One-use refills remove themselves from the room after the shatter
    /// coroutine; the snapshot stays indexed for split-simulation stability.
    pub removed: bool,
}

/// Vanilla `FallingBlock` Solid coroutine state. `position` is the block's
/// top-left corner, matching both `Entity.Position` and the Solid collider.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FallingBlockSnapshot {
    /// 0 waiting for a trigger, 1 shaking, 2 player-wait before the drop,
    /// 3 falling, 4 impact landing shake, 5 landed permanently (Safe),
    /// 6 resting on a platform below while waiting to fall again.
    pub phase: u8,
    pub position: Vec2,
    /// Original room position captured before the runtime map is moved.
    pub start: Vec2,
    /// Platform movementCounter retained by MoveVCollideSolids across frames.
    pub remainder_y: f32,
    /// Current vertical fall speed, approached toward 160 px/s at 500 px/s^2.
    pub fall_speed: f32,
    /// Remaining seconds of the pre-drop shake or the impact landing shake.
    pub shake_timer: f32,
    /// Remaining seconds of the player-wait window before the drop, or the
    /// 0.1-second platform-rest tick.
    pub wait_timer: f32,
    /// Public `FallDelay` seconds the sequence waits after triggering.
    pub fall_delay: f32,
    /// `Triggered` set by OnStaticMoverTrigger; also forces PlayerWaitCheck.
    pub triggered: bool,
    pub collidable: bool,
    pub removed: bool,
    pub safe: bool,
}

/// Vanilla `ExitBlock` state. The Solid starts collidable unless `Awake`
/// finds the player inside it, then closes permanently after the player has
/// cleared its original rectangle.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExitBlockSnapshot {
    pub position: Vec2,
    pub collidable: bool,
}

/// Vanilla `InvisibleBarrier` state. The constructor starts non-collidable;
/// its first Update either enables the Solid permanently or, when the player
/// overlaps it, leaves it non-collidable and deactivates the entity forever.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InvisibleBarrierSnapshot {
    pub position: Vec2,
    pub initialized: bool,
    pub collidable: bool,
}

fn default_stamina() -> f32 {
    110.0
}
fn default_dashes() -> u8 {
    1
}
fn default_facing() -> bool {
    true
}
fn default_frame_delta_time() -> f32 {
    0.016_666_7
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[repr(C)]
pub struct PlayerSnapshot {
    pub pos: Vec2,
    pub speed: Vec2,
    pub state: PlayerState,
    pub facing: bool,
    pub dashes: u8,
    pub stamina: f32,
    /// Geometric `Player.OnGround()` value exposed by the portable snapshot
    /// after every entity in the Scene has updated.
    pub on_ground: bool,
    /// Source-private `Player.onGround` captured during Player.Update, before
    /// later room entities such as ZipMover can carry or push the player.
    pub player_on_ground: bool,
    /// Distinguishes legacy/input snapshots from a persisted internal ground
    /// value so segmented simulation can resume the one-frame separation.
    pub player_on_ground_initialized: bool,
    pub ducking: bool,
    pub can_dream_dash: bool,
    pub dead: bool,
    pub death_freeze_pending: bool,
    pub respawn_frames: u16,
    pub current_room_bounds: Option<Rect>,
    pub transition_room_bounds: Option<Rect>,
    pub transition_direction: Vec2,
    pub transition_target: Vec2,
    pub transition_timer: f32,
    /// Set by transition completion so its next NormalUpdate can preserve the
    /// source transition's immediate player-state ordering.
    pub post_transition_normal_updates: u8,
    /// Level.Camera.Position used by camera-driven hazards.
    pub camera: Vec2,
    pub camera_initialized: bool,
    /// Session.CoreMode observed by CoreModeListener entities.
    pub core_mode: CoreMode,
    /// Player.JustRespawned gates RisingLava and SandwichLava waiting behavior.
    pub just_respawned: bool,
    pub dash_dir: Vec2,
    pub last_aim: Vec2,
    pub before_dash_speed: Vec2,
    pub demo_dashed: bool,
    pub dash_started_on_ground: bool,
    pub dash_end_pending: bool,
    pub dash_attack_timer: f32,
    pub dash_cooldown_timer: f32,
    pub dash_refill_cooldown_timer: f32,
    /// Global Monocle freeze remaining. While positive, Engine frames advance
    /// but the scene and player state machine do not update.
    pub freeze_timer: f32,
    /// Engine.TimeRate written by HeartGem. Raw engine-frame time remains DT.
    pub time_rate: f32,
    /// Engine.DeltaTime captured once at the beginning of the current raw
    /// engine frame. It is derived from `time_rate`, is not part of the
    /// portable wire snapshot, and prevents an entity that writes TimeRate
    /// mid-frame from changing later entities' delta until the next frame.
    #[serde(skip, default = "default_frame_delta_time")]
    #[doc(hidden)]
    pub frame_delta_time: f32,
    pub state_timer: f32,
    pub boost_target: Vec2,
    pub boost_red: bool,
    pub last_booster_target: Vec2,
    pub booster_reuse_timer: f32,
    /// Mirrors the active Booster's `BoostingPlayer` flag after
    /// `Player.CallDashEvents` consumes `CurrentBooster`.
    pub booster_boosting: bool,
    /// Current `Level.Wind`, advanced by the source WindController rules.
    pub wind: Vec2,
    /// Persistent WindController target selected by the last entered wind trigger.
    pub wind_target: Vec2,
    pub no_wind_timer: f32,
    pub wall_slide_timer: f32,
    pub wall_slide_dir: i8,
    pub jump_grace_timer: f32,
    pub jump_buffer_timer: f32,
    /// Remaining 0.08 second VirtualButton buffer for a normal dash press.
    pub dash_buffer_timer: f32,
    /// Remaining 0.08 second VirtualButton buffer for a crouch-dash press.
    pub crouch_dash_buffer_timer: f32,
    pub auto_jump: bool,
    pub auto_jump_timer: f32,
    pub var_jump_timer: f32,
    pub var_jump_speed: f32,
    pub max_fall: f32,
    /// Player.cs cached moveX. Wallboost reads the previous frame's value
    /// before Update refreshes it from input or forceMoveX.
    pub move_x: i8,
    pub force_move_x: i8,
    pub force_move_x_timer: f32,
    /// A neutral wall jump performed on the first NormalUpdate after a room
    /// transition retains its launch speed through the following update.
    #[serde(default)]
    pub neutral_wall_jump_friction_delay: u8,
    pub wall_speed_retention_timer: f32,
    pub wall_speed_retained: f32,
    pub wall_boost_timer: f32,
    pub wall_boost_dir: i8,
    /// Deferred horizontal launch used by Player.ClimbHop while the player's
    /// body is still overlapping the ledge wall.
    pub hop_wait_x: i8,
    pub hop_wait_x_speed: f32,
    /// `Actor.LiftSpeed` written by a moving platform before Player.Update.
    /// Actor.Update clears this after the state callback every frame.
    pub current_lift_speed: Vec2,
    /// Last non-zero lift speed retained for `LiftSpeedGraceTime` (0.16 s).
    pub last_lift_speed: Vec2,
    pub lift_speed_timer: f32,
    /// Shared deterministic clock for simulator-native constant-velocity
    /// moving solids. Keeping it in the snapshot makes split simulations
    /// resume from the same platform positions.
    pub moving_solid_time: f32,
    /// Monocle Scene.TimeActive. This intentionally remains f32 so the
    /// long-running spinner interval freeze is represented faithfully.
    pub scene_time_active: f32,
    pub cassette_manager: CassetteManagerSnapshot,
    /// Per-entity CassetteBlock state, in map entity order.
    pub cassette_blocks: Vec<CassetteBlockSnapshot>,
    /// Per-entity CrystalStaticSpinner state, in map entity order.
    pub spinners: Vec<SpinnerSnapshot>,
    /// Per-entity Bumper position and randomized SineWave phase, in map
    /// entity order. This is captured from Everest because `Randomize()` is
    /// deliberately not reproducible from a portable map alone.
    pub bumpers: Vec<BumperSnapshot>,
    /// Per-entity Lookout coroutine state, in map entity order.
    pub lookouts: Vec<LookoutSnapshot>,
    /// Per-entity vanilla ZipMover coroutine and Platform movement state, in
    /// map entity order. This keeps segmented simulation composable.
    pub zip_movers: Vec<ZipMoverSnapshot>,
    /// Per-entity hot BounceBlock state, in map entity order.
    pub bounce_blocks: Vec<BounceBlockSnapshot>,
    /// Per-entity vanilla MoveBlock controller and reform state.
    pub move_blocks: Vec<MoveBlockSnapshot>,
    /// Per-entity vanilla TheoCrystal actor and Holdable state.
    pub theo_crystals: Vec<TheoCrystalSnapshot>,
    /// Per-entity vanilla HeartGem collection coroutine state.
    pub heart_gems: Vec<HeartGemSnapshot>,
    /// Per-entity Core RisingLava camera-following hazard state.
    pub rising_lavas: Vec<RisingLavaSnapshot>,
    /// Per-entity persistent Core SandwichLava hazard state.
    pub sandwich_lavas: Vec<SandwichLavaSnapshot>,
    /// Per-entity vanilla Glider actor and Holdable state.
    pub gliders: Vec<GliderSnapshot>,
    /// Per-entity vanilla non-fragile Cloud movement state.
    pub clouds: Vec<CloudSnapshot>,
    /// Per-entity Seeker Actor and StateMachine state, in map entity order.
    pub seekers: Vec<SeekerSnapshot>,
    /// Per-entity CloseBehindPlayerAlways TempleGate state.
    pub temple_gates: Vec<TempleGateSnapshot>,
    /// Per-entity vanilla Refill state, in map entity order.
    pub refills: Vec<RefillSnapshot>,
    /// Per-entity vanilla FallingBlock Solid coroutine state, in map entity order.
    pub falling_blocks: Vec<FallingBlockSnapshot>,
    /// Per-entity vanilla ExitBlock collidability, in map entity order.
    pub exit_blocks: Vec<ExitBlockSnapshot>,
    /// Per-entity vanilla InvisibleBarrier first-update state.
    pub invisible_barriers: Vec<InvisibleBarrierSnapshot>,
    /// Map-order TheoCrystal index currently held by Player.
    pub holding_theo: Option<u16>,
    /// Map-order Glider index currently held by Player.
    pub holding_glider: Option<u16>,
    pub min_hold_timer: f32,
    /// PickupCoroutine state needed to resume the 0.16 second lift tween.
    pub pickup_old_speed: Vec2,
    pub pickup_old_var_jump_timer: f32,
    pub pickup_timer: f32,
    pub climb_no_move_timer: f32,
    /// The signed vertical target selected by the most recent ClimbUpdate.
    /// Player.Update uses this to limit JumpThru Assist to upward climbing.
    pub last_climb_move: i8,
    pub dream_dash_can_end_timer: f32,
    pub launch_approach_x: Option<f32>,
    pub summit_launch_target_x: f32,
    pub summit_launch_particle_timer: f32,
    pub star_fly_timer: f32,
    pub star_fly_transforming: bool,
    /// Remaining source update frames before the startStarFly animation and
    /// its trailing 0.1 second coroutine wait release the player.
    pub star_fly_transform_frames: u8,
    pub star_fly_speed_lerp: f32,
    pub star_fly_last_dir: Vec2,
    pub last_feather_target: Vec2,
    pub feather_reuse_timer: f32,
    pub last_bumper_target: Vec2,
    pub bumper_reuse_timer: f32,
    /// Bitset of ordinary strawberries already attached to the player's
    /// follower train. The bit index is the map entity index.
    pub strawberry_picked_mask: u64,
    pub carried_strawberries: u8,
    /// Follower.DelayTimer for the first ordinary strawberry in the train.
    pub strawberry_follow_delay_timer: f32,
    /// Strawberry.collectTimer for the first ordinary strawberry in the train.
    pub strawberry_collect_timer: f32,
    pub strawberry_collect_index: u16,
    pub strawberry_collect_reset_timer: f32,
    /// `Player.Bounce` can restore the cached StarFly hurtbox as Collider after
    /// `StarFlyEnd` has already restored the normal hurtbox.
    pub star_fly_hitbox_preserved: bool,
    pub last_bounce_target: Vec2,
    pub bounce_reuse_timer: f32,
    /// Legacy portable snapshots may carry a deferred top-bounce. New
    /// FireBall callbacks resolve in the source's same Player.Update frame.
    pub pending_bounce_from_y: Option<f32>,
    pub explode_launch_boost_timer: f32,
    pub explode_launch_boost_speed: f32,
    pub badeline_boost_active: bool,
    pub badeline_boost_final: bool,
    pub badeline_boost_phase: u8,
    pub badeline_boost_frame: u8,
    pub badeline_boost_start: Vec2,
    pub badeline_boost_target: Vec2,
    pub last_badeline_boost_target: Vec2,
    pub badeline_boost_entity_origin: Vec2,
    pub badeline_boost_current_position: Vec2,
    pub badeline_boost_relocation_from: Vec2,
    pub badeline_boost_relocation_to: Vec2,
    pub badeline_boost_relocation_elapsed: f32,
    pub badeline_boost_relocation_duration: f32,
    pub badeline_boost_stage: u16,
    pub badeline_boost_relocating: bool,
    pub badeline_boost_collidable: bool,
    pub dummy_moving: bool,
    pub dummy_gravity: bool,
    pub dummy_friction: bool,
    pub dummy_maxspeed: bool,
    pub temple_fall_landed: bool,
    pub temple_fall_wait_frames: u8,
    pub reflection_fall_phase: u8,
    pub reflection_fall_frames: u16,
    pub reflection_fall_wait_timer: f32,
    pub ignore_jump_thrus: bool,
    pub launched: bool,
    /// Monocle-style sub-pixel remainder required for deterministic axis movement.
    pub movement_remainder: Vec2,
}

impl Default for PlayerSnapshot {
    fn default() -> Self {
        Self {
            pos: Vec2::default(),
            speed: Vec2::default(),
            state: PlayerState::Normal,
            facing: default_facing(),
            dashes: default_dashes(),
            stamina: default_stamina(),
            on_ground: false,
            player_on_ground: false,
            player_on_ground_initialized: false,
            ducking: false,
            can_dream_dash: false,
            dead: false,
            death_freeze_pending: false,
            respawn_frames: 0,
            current_room_bounds: None,
            transition_room_bounds: None,
            transition_direction: Vec2::default(),
            transition_target: Vec2::default(),
            transition_timer: 0.0,
            post_transition_normal_updates: 0,
            camera: Vec2::default(),
            camera_initialized: false,
            core_mode: CoreMode::None,
            just_respawned: false,
            dash_dir: Vec2::default(),
            last_aim: Vec2::new(1.0, 0.0),
            before_dash_speed: Vec2::default(),
            demo_dashed: false,
            dash_started_on_ground: false,
            dash_end_pending: false,
            dash_attack_timer: 0.0,
            dash_cooldown_timer: 0.0,
            dash_refill_cooldown_timer: 0.0,
            freeze_timer: 0.0,
            time_rate: 1.0,
            frame_delta_time: default_frame_delta_time(),
            state_timer: 0.0,
            boost_target: Vec2::default(),
            boost_red: false,
            last_booster_target: Vec2::default(),
            booster_reuse_timer: 0.0,
            booster_boosting: false,
            wind: Vec2::default(),
            wind_target: Vec2::default(),
            no_wind_timer: 0.0,
            wall_slide_timer: 1.2,
            wall_slide_dir: 0,
            jump_grace_timer: 0.0,
            jump_buffer_timer: 0.0,
            dash_buffer_timer: 0.0,
            crouch_dash_buffer_timer: 0.0,
            auto_jump: false,
            auto_jump_timer: 0.0,
            var_jump_timer: 0.0,
            var_jump_speed: 0.0,
            max_fall: 160.0,
            move_x: 0,
            force_move_x: 0,
            force_move_x_timer: 0.0,
            neutral_wall_jump_friction_delay: 0,
            wall_speed_retention_timer: 0.0,
            wall_speed_retained: 0.0,
            wall_boost_timer: 0.0,
            wall_boost_dir: 0,
            hop_wait_x: 0,
            hop_wait_x_speed: 0.0,
            current_lift_speed: Vec2::default(),
            last_lift_speed: Vec2::default(),
            lift_speed_timer: 0.0,
            moving_solid_time: 0.0,
            scene_time_active: 0.0,
            cassette_manager: CassetteManagerSnapshot::default(),
            cassette_blocks: vec![],
            spinners: vec![],
            bumpers: vec![],
            lookouts: vec![],
            zip_movers: vec![],
            bounce_blocks: vec![],
            move_blocks: vec![],
            theo_crystals: vec![],
            heart_gems: vec![],
            rising_lavas: vec![],
            sandwich_lavas: vec![],
            gliders: vec![],
            clouds: vec![],
            seekers: vec![],
            temple_gates: vec![],
            refills: vec![],
            falling_blocks: vec![],
            exit_blocks: vec![],
            invisible_barriers: vec![],
            holding_theo: None,
            holding_glider: None,
            min_hold_timer: 0.0,
            pickup_old_speed: Vec2::default(),
            pickup_old_var_jump_timer: 0.0,
            pickup_timer: 0.0,
            climb_no_move_timer: 0.0,
            last_climb_move: 0,
            dream_dash_can_end_timer: 0.0,
            launch_approach_x: None,
            summit_launch_target_x: 0.0,
            summit_launch_particle_timer: 0.0,
            star_fly_timer: 0.0,
            star_fly_transforming: false,
            star_fly_transform_frames: 0,
            star_fly_speed_lerp: 0.0,
            star_fly_last_dir: Vec2::default(),
            last_feather_target: Vec2::default(),
            feather_reuse_timer: 0.0,
            last_bumper_target: Vec2::default(),
            bumper_reuse_timer: 0.0,
            strawberry_picked_mask: 0,
            carried_strawberries: 0,
            strawberry_follow_delay_timer: 0.0,
            strawberry_collect_timer: 0.0,
            strawberry_collect_index: 0,
            strawberry_collect_reset_timer: 0.0,
            star_fly_hitbox_preserved: false,
            last_bounce_target: Vec2::default(),
            bounce_reuse_timer: 0.0,
            pending_bounce_from_y: None,
            explode_launch_boost_timer: 0.0,
            explode_launch_boost_speed: 0.0,
            badeline_boost_active: false,
            badeline_boost_final: false,
            badeline_boost_phase: 0,
            badeline_boost_frame: 0,
            badeline_boost_start: Vec2::default(),
            badeline_boost_target: Vec2::default(),
            last_badeline_boost_target: Vec2::default(),
            badeline_boost_entity_origin: Vec2::default(),
            badeline_boost_current_position: Vec2::default(),
            badeline_boost_relocation_from: Vec2::default(),
            badeline_boost_relocation_to: Vec2::default(),
            badeline_boost_relocation_elapsed: 0.0,
            badeline_boost_relocation_duration: 0.0,
            badeline_boost_stage: 0,
            badeline_boost_relocating: false,
            badeline_boost_collidable: false,
            dummy_moving: false,
            dummy_gravity: true,
            dummy_friction: true,
            dummy_maxspeed: true,
            temple_fall_landed: false,
            temple_fall_wait_frames: 0,
            reflection_fall_phase: 0,
            reflection_fall_frames: 0,
            reflection_fall_wait_timer: 0.0,
            ignore_jump_thrus: false,
            launched: false,
            movement_remainder: Vec2::default(),
        }
    }
}
