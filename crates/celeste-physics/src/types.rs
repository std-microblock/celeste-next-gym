use serde::{Deserialize, Serialize};

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
}

impl InputState {
    pub fn normalized(mut self) -> Self {
        self.move_x = self.move_x.clamp(-1, 1);
        self.move_y = self.move_y.clamp(-1, 1);
        self
    }
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
    pub on_ground: bool,
    pub ducking: bool,
    pub can_dream_dash: bool,
    pub dead: bool,
    pub death_freeze_pending: bool,
    pub respawn_frames: u16,
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
    pub state_timer: f32,
    pub boost_target: Vec2,
    pub boost_red: bool,
    pub last_booster_target: Vec2,
    pub booster_reuse_timer: f32,
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
    pub wall_speed_retention_timer: f32,
    pub wall_speed_retained: f32,
    pub wall_boost_timer: f32,
    pub wall_boost_dir: i8,
    /// `Actor.LiftSpeed` written by a moving platform before Player.Update.
    /// Actor.Update clears this after the state callback every frame.
    pub current_lift_speed: Vec2,
    /// Last non-zero lift speed retained for `LiftSpeedGraceTime` (0.16 s).
    pub last_lift_speed: Vec2,
    pub lift_speed_timer: f32,
    pub climb_no_move_timer: f32,
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
    /// `Player.Bounce` can restore the cached StarFly collider after
    /// `StarFlyEnd` has already restored the normal hurtbox.
    pub star_fly_hitbox_preserved: bool,
    pub last_bounce_target: Vec2,
    pub bounce_reuse_timer: f32,
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
            ducking: false,
            can_dream_dash: false,
            dead: false,
            death_freeze_pending: false,
            respawn_frames: 0,
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
            state_timer: 0.0,
            boost_target: Vec2::default(),
            boost_red: false,
            last_booster_target: Vec2::default(),
            booster_reuse_timer: 0.0,
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
            wall_speed_retention_timer: 0.0,
            wall_speed_retained: 0.0,
            wall_boost_timer: 0.0,
            wall_boost_dir: 0,
            current_lift_speed: Vec2::default(),
            last_lift_speed: Vec2::default(),
            lift_speed_timer: 0.0,
            climb_no_move_timer: 0.0,
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
            star_fly_hitbox_preserved: false,
            last_bounce_target: Vec2::default(),
            bounce_reuse_timer: 0.0,
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
