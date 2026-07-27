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
const JUMP_GRACE: f32 = 0.1;
const JUMP_SPEED: f32 = -105.0;
const JUMP_H_BOOST: f32 = 40.0;
const VAR_JUMP_TIME: f32 = 0.2;
const WALL_JUMP_H: f32 = 130.0;
const WALL_SLIDE_START_MAX: f32 = 20.0;
const WALL_SLIDE_TIME: f32 = 1.2;
const DASH_SPEED: f32 = 240.0;
const END_DASH_SPEED: f32 = 160.0;
const DASH_TIME: f32 = 0.15;
const DASH_COOLDOWN: f32 = 0.2;
const DASH_ATTACK_TIME: f32 = 0.3;
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
const CLIMB_ACCEL: f32 = 900.0;
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
const LAUNCH_CANCEL_THRESHOLD: f32 = 220.0;
const TRANSITION_TIME: f32 = 0.65;
const TRANSITION_MOVE_SPEED: f32 = 60.0;

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

pub fn simulate(
    snapshot: PlayerSnapshot,
    inputs: &[InputState],
    map: &Map,
    frames: u32,
) -> Result<PlayerSnapshot, SimulationError> {
    let mut trace = simulate_trace(snapshot, inputs, map, frames)?;
    Ok(trace
        .states
        .pop()
        .expect("trace always contains initial state"))
}

pub fn simulate_trace(
    mut snapshot: PlayerSnapshot,
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
    validate_snapshot(&snapshot)?;
    let mut states = Vec::with_capacity(frames + 1);
    states.push(snapshot.clone());
    for input in &inputs[..frames] {
        step(&mut snapshot, input.normalized(), map)?;
        states.push(snapshot.clone());
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
        s.dash_buffer_timer,
        s.crouch_dash_buffer_timer,
        s.movement_remainder.x,
        s.movement_remainder.y,
    ];
    if values.iter().all(|x| x.is_finite()) {
        Ok(())
    } else {
        Err(SimulationError::NonFinite)
    }
}

fn step(p: &mut PlayerSnapshot, mut input: InputState, map: &Map) -> Result<(), SimulationError> {
    // Input.Dash and Input.CrouchDash use a 0.08 second VirtualButton buffer.
    // Input keeps advancing during Celeste.Freeze even though Scene.Update and
    // Player.Update are skipped, which is the timing window used by a bumper
    // clip to carry a press out of the hit freeze.
    p.dash_buffer_timer = (p.dash_buffer_timer - DT).max(0.0);
    p.crouch_dash_buffer_timer = (p.crouch_dash_buffer_timer - DT).max(0.0);
    if input.dash_pressed {
        p.dash_buffer_timer = 0.08;
    }
    if input.crouch_dash_pressed {
        p.crouch_dash_buffer_timer = 0.08;
    }
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
            p.on_ground = grounded(p, map);
            p.dashes = p.dashes.max(1);
            p.stamina = 110.0;
            p.movement_remainder = Vec2::default();
            return Ok(());
        }
        p.respawn_frames -= 1;
        p.on_ground = false;
        if p.death_freeze_pending {
            p.death_freeze_pending = false;
            p.freeze_timer = 0.05;
            return Ok(());
        }
        if p.freeze_timer > 0.0 {
            p.freeze_timer = (p.freeze_timer - DT).max(0.0);
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
    if p.transition_timer > 0.0 {
        update_transition(p);
        return Ok(());
    }
    advance_badeline_boost_relocation(p);
    // WindController is updated before Player in the level entity order. It
    // advances and invokes WindMover using the state from the start of the
    // frame; WindTrigger interaction below selects the target for next frame.
    advance_wind_controller(p);
    apply_wind_movement(p, map);
    // Player.Update checks the force-move timer before subtracting DeltaTime,
    // and keeps the forced direction for that whole frame even when the
    // subtraction reaches zero.
    let force_move_x_active = p.force_move_x_timer > 0.0;
    // Dash refill uses an if/else in Player.Update: a timer that was positive
    // at frame start is only decremented, even if it crosses zero.
    let dash_refill_cooldown_active = p.dash_refill_cooldown_timer > 0.0;
    if p.explode_launch_boost_timer > 0.0 {
        if input.move_x as f32 == p.explode_launch_boost_speed.signum() {
            p.speed.x = p.explode_launch_boost_speed;
            p.explode_launch_boost_timer = 0.0;
        } else {
            p.explode_launch_boost_timer -= DT;
        }
    }
    tick_timers(p);
    if p.wall_slide_dir != 0 {
        p.wall_slide_timer = (p.wall_slide_timer - DT).max(0.0);
    }
    p.wall_slide_dir = 0;
    if p.strawberry_collect_reset_timer > 0.0 {
        p.strawberry_collect_reset_timer = (p.strawberry_collect_reset_timer - DT).max(0.0);
        if p.strawberry_collect_reset_timer <= 0.0 {
            p.strawberry_collect_index = 0;
        }
    } else {
        p.strawberry_collect_index = 0;
    }
    p.on_ground = p.state != PlayerState::DreamDash && grounded(p, map);
    if p.on_ground {
        p.auto_jump = false;
        p.jump_grace_timer = JUMP_GRACE;
        p.wall_slide_timer = WALL_SLIDE_TIME;
        if !dash_refill_cooldown_active {
            p.dashes = p.dashes.max(1);
        }
        p.stamina = 110.0;
    }
    if p.state == PlayerState::Swim && !dash_refill_cooldown_active {
        p.dashes = p.dashes.max(1);
    }
    if input.jump_pressed {
        p.jump_buffer_timer = 0.1;
    }
    update_wall_boost(p);
    p.move_x = if force_move_x_active {
        p.force_move_x
    } else {
        input.move_x
    };
    if p.move_x != 0
        && !matches!(
            p.state,
            PlayerState::Climb
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

    if p.badeline_boost_active {
        update_badeline_boost(p, map);
        p.on_ground = grounded(p, map);
        return Ok(());
    }

    match p.state {
        PlayerState::Normal => normal_update(p, input, map),
        PlayerState::Dash => dash_update(p, input, map),
        PlayerState::Climb => climb_update(p, input, map),
        PlayerState::Swim => swim_update(p, input, map),
        PlayerState::Boost => boost_update(p, input, map),
        PlayerState::RedDash => red_dash_update(p, input),
        PlayerState::HitSquash => hit_squash_update(p),
        PlayerState::Launch => launch_update(p, input, map),
        PlayerState::DreamDash => dream_dash_update(p),
        PlayerState::SummitLaunch => summit_launch_update(p, map),
        PlayerState::StarFly => star_fly_update(p, input, map),
        PlayerState::Dummy => dummy_update(p, input, map),
        PlayerState::Frozen => {}
        PlayerState::TempleFall => temple_fall_update(p, map),
        PlayerState::ReflectionFall => reflection_fall_update(p, map),
        PlayerState::IntroRespawn => return Ok(()),
        other => return Err(SimulationError::UnsupportedState(other)),
    }

    // After components/coroutines update but before movement, Player.Update
    // restores the normal collider while rising/falling in open air. A
    // downward air dash therefore only becomes crouched again when it lands.
    if p.ducking && p.speed.y > 0.0 && !p.on_ground && p.jump_grace_timer <= 0.0 {
        p.ducking = false;
    }

    move_axis(p, map, true);
    move_axis(p, map, false);
    interact(p, map, input);
    update_strawberry_train(p);
    try_begin_badeline_boost(p, map);
    enforce_level_bounds(p, map);
    p.on_ground = grounded(p, map);
    Ok(())
}

fn tick_timers(p: &mut PlayerSnapshot) {
    for timer in [
        &mut p.dash_attack_timer,
        &mut p.dash_cooldown_timer,
        &mut p.dash_refill_cooldown_timer,
        &mut p.booster_reuse_timer,
        &mut p.feather_reuse_timer,
        &mut p.bumper_reuse_timer,
        &mut p.no_wind_timer,
        &mut p.jump_grace_timer,
        &mut p.jump_buffer_timer,
        &mut p.auto_jump_timer,
        &mut p.var_jump_timer,
        &mut p.force_move_x_timer,
        &mut p.climb_no_move_timer,
        &mut p.dream_dash_can_end_timer,
    ] {
        *timer = (*timer - DT).max(0.0);
    }
}

fn update_wall_boost(p: &mut PlayerSnapshot) {
    if p.wall_boost_timer <= 0.0 {
        return;
    }
    p.wall_boost_timer = (p.wall_boost_timer - DT).max(0.0);
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
    if p.speed.x.signum() == -p.wall_speed_retained.signum() {
        p.wall_speed_retention_timer = 0.0;
    } else if !map.solid_at(current_player_rect(
        p,
        p.pos.x + p.wall_speed_retained.signum(),
        p.pos.y,
    )) {
        p.speed.x = p.wall_speed_retained;
        p.wall_speed_retention_timer = 0.0;
    } else {
        p.wall_speed_retention_timer = (p.wall_speed_retention_timer - DT).max(0.0);
    }
}

fn normal_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    if (input.dash_pressed || input.crouch_dash_pressed)
        && p.dashes > 0
        && p.dash_cooldown_timer <= 0.0
    {
        begin_dash(p, input, true, false);
        return;
    }

    let wall = wall_dir(p, map);
    if input.grab_held
        && !p.on_ground
        && wall != 0
        && p.stamina > 0.0
        && p.speed.y >= 0.0
        && p.speed.x.signum() != -(wall as f32)
    {
        p.state = PlayerState::Climb;
        p.facing = wall > 0;
        p.auto_jump = false;
        p.speed.x = 0.0;
        p.speed.y *= 0.2;
        p.wall_slide_timer = WALL_SLIDE_TIME;
        p.climb_no_move_timer = 0.1;
        p.wall_boost_timer = 0.0;
        return;
    }

    // Player.NormalUpdate changes the active collider before applying run
    // friction. This ordering is what lets a crouched player enter a booster
    // with the six-pixel hitbox (Archie) on the same frame.
    if p.ducking {
        if p.on_ground && input.move_y != 1 && can_unduck(p, map) {
            p.ducking = false;
        }
    } else if p.on_ground && input.move_y == 1 && p.speed.y >= 0.0 {
        p.ducking = true;
    }

    let mult = if p.on_ground { 1.0 } else { AIR_MULT };
    let move_x = p.move_x;
    if p.ducking && p.on_ground {
        p.speed.x = approach(p.speed.x, 0.0, DUCK_FRICTION * DT);
    } else {
        let target = move_x as f32 * MAX_RUN;
        let same_direction_over_max = move_x != 0
            && p.speed.x.abs() > MAX_RUN
            && p.speed.x.signum() == (move_x as f32).signum();
        p.speed.x = approach(
            p.speed.x,
            target,
            if same_direction_over_max {
                RUN_REDUCE
            } else {
                RUN_ACCEL
            } * mult
                * DT,
        );
    }
    if move_x != 0 {
        p.facing = move_x > 0;
    }

    let target_max_fall = if input.move_y > 0 {
        FAST_MAX_FALL
    } else {
        MAX_FALL
    };
    p.max_fall = approach(p.max_fall, target_max_fall, FAST_MAX_ACCEL * DT);
    let mut fall_target = p.max_fall;
    if wall != 0 && input.move_x == wall && p.speed.y >= 0.0 && !p.on_ground {
        p.wall_slide_dir = wall;
        fall_target = WALL_SLIDE_START_MAX
            + (MAX_FALL - WALL_SLIDE_START_MAX) * (1.0 - p.wall_slide_timer / WALL_SLIDE_TIME);
    }
    let gravity_mult = if (input.jump_held || p.auto_jump) && p.speed.y.abs() < HALF_GRAV_THRESHOLD
    {
        0.5
    } else {
        1.0
    };
    if !p.on_ground {
        p.speed.y = approach(p.speed.y, fall_target, GRAVITY * gravity_mult * DT);
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
            p.auto_jump = false;
            p.dash_attack_timer = 0.0;
            p.wall_slide_timer = WALL_SLIDE_TIME;
            p.wall_boost_timer = 0.0;
            p.var_jump_speed = p.speed.y;
            p.var_jump_timer = VAR_JUMP_TIME;
        } else if wall != 0 {
            p.jump_buffer_timer = 0.0;
            p.speed.x = -(wall as f32) * WALL_JUMP_H;
            p.speed.y = JUMP_SPEED;
            p.auto_jump = false;
            p.dash_attack_timer = 0.0;
            p.wall_slide_timer = WALL_SLIDE_TIME;
            p.wall_boost_timer = 0.0;
            if move_x != 0 {
                p.force_move_x = -wall;
                p.force_move_x_timer = 0.16;
            }
            p.var_jump_speed = p.speed.y;
            p.var_jump_timer = VAR_JUMP_TIME;
        }
    }
}

fn begin_dash(
    p: &mut PlayerSnapshot,
    input: InputState,
    consume_dash: bool,
    delayed_coroutine: bool,
) {
    p.dash_buffer_timer = 0.0;
    p.crouch_dash_buffer_timer = 0.0;
    let Vec2 { x, y } = input_aim(input, p.facing);
    p.dash_dir = Vec2::new(x, y);
    p.before_dash_speed = p.speed;
    p.demo_dashed = input.crouch_dash_pressed;
    p.dash_started_on_ground = p.on_ground;
    p.dash_end_pending = false;
    p.speed = Vec2::default();
    p.state = PlayerState::Dash;
    // Player.cs DashCoroutine yields once before applying dash speed.
    p.state_timer = DASH_TIME + DT * if delayed_coroutine { 2.0 } else { 1.0 };
    p.dash_attack_timer = DASH_ATTACK_TIME;
    p.dash_cooldown_timer = DASH_COOLDOWN;
    p.dash_refill_cooldown_timer = 0.1;
    p.freeze_timer = 0.05;
    if consume_dash {
        p.dashes = p.dashes.saturating_sub(1);
    }
    if p.demo_dashed || (!p.ducking && input.move_y > 0) {
        p.ducking = true;
    }
    if x != 0.0 {
        p.facing = x > 0.0;
    }
}

fn dash_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    p.state_timer = (p.state_timer - DT).max(0.0);
    if (p.state_timer - DASH_TIME).abs() <= DT * 0.5 {
        p.speed = Vec2::new(p.dash_dir.x * DASH_SPEED, p.dash_dir.y * DASH_SPEED);
        if p.before_dash_speed.x.signum() == p.speed.x.signum()
            && p.before_dash_speed.x.abs() > p.speed.x.abs()
        {
            p.speed.x = p.before_dash_speed.x;
        }
        if map.water_at(player_rect(p.pos.x, p.pos.y)) {
            p.speed.x *= SWIM_DASH_SPEED_MULT;
            p.speed.y *= SWIM_DASH_SPEED_MULT;
        }
        if p.on_ground && p.dash_dir.x != 0.0 && p.dash_dir.y > 0.0 && p.speed.y > 0.0 {
            p.dash_dir.x = p.dash_dir.x.signum();
            p.dash_dir.y = 0.0;
            p.speed.y = 0.0;
            p.speed.x *= 1.2;
            p.ducking = true;
        }
        if p.dash_dir.y.abs() < 0.1 && input.jump_pressed && p.jump_grace_timer > 0.0 {
            super_jump(p);
        } else if p.dash_dir.x.abs() <= 0.2 && p.dash_dir.y <= -0.75 && input.jump_pressed {
            let wall = wall_dir(p, map);
            if wall != 0 {
                super_wall_jump(p, -wall);
            }
        }
        return;
    }
    if p.dash_dir.y.abs() < 0.1 && input.jump_pressed && p.jump_grace_timer > 0.0 {
        super_jump(p);
        return;
    }
    if p.dash_dir.x.abs() <= 0.2 && p.dash_dir.y <= -0.75 && input.jump_pressed {
        let wall = wall_dir(p, map);
        if wall != 0 {
            super_wall_jump(p, -wall);
            return;
        }
    }
    if p.state_timer > 0.0 {
        return;
    }
    if !p.dash_end_pending {
        p.dash_end_pending = true;
        return;
    }
    p.dash_end_pending = false;
    p.state = PlayerState::Normal;
    p.auto_jump = true;
    if p.dash_dir.y <= 0.0 {
        p.speed.x = p.dash_dir.x * END_DASH_SPEED;
        p.speed.y = p.dash_dir.y * END_DASH_SPEED;
        if p.speed.y < 0.0 {
            p.speed.y *= 0.75;
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
    if p.ducking {
        p.ducking = false;
        p.speed.x *= 1.25;
        p.speed.y *= 0.5;
    }
    p.var_jump_speed = p.speed.y;
    p.launched = true;
}

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
    p.var_jump_speed = p.speed.y;
    p.launched = true;
}

fn climb_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    let wall = if p.facing { 1 } else { -1 };
    if !input.grab_held || p.stamina <= 0.0 {
        p.state = PlayerState::Normal;
        return;
    }
    if !touching_wall(p, map, wall) {
        if p.speed.y < 0.0 {
            climb_hop(p, map, wall);
        }
        p.state = PlayerState::Normal;
        return;
    }
    if input.jump_pressed {
        p.state = PlayerState::Normal;
        p.jump_buffer_timer = 0.0;
        p.jump_grace_timer = 0.0;
        p.auto_jump = false;
        p.dash_attack_timer = 0.0;
        p.wall_slide_timer = WALL_SLIDE_TIME;
        p.wall_boost_timer = 0.0;
        if p.move_x == -wall {
            p.speed = Vec2::new(-(wall as f32) * WALL_JUMP_H, JUMP_SPEED);
        } else {
            p.speed.x += p.move_x as f32 * JUMP_H_BOOST;
            p.speed.y = JUMP_SPEED;
            if !p.on_ground {
                p.stamina = (p.stamina - 27.5).max(0.0);
            }
            if p.move_x == 0 {
                p.wall_boost_dir = -wall;
                p.wall_boost_timer = 0.2;
            }
        }
        p.var_jump_speed = p.speed.y;
        p.var_jump_timer = VAR_JUMP_TIME;
        return;
    }
    let target = if p.climb_no_move_timer > 0.0 {
        0.0
    } else {
        match input.move_y {
            -1 if climb_slip_check(p, map, wall) => {
                climb_hop(p, map, wall);
                p.state = PlayerState::Normal;
                return;
            }
            -1 => CLIMB_UP_SPEED,
            1 => CLIMB_DOWN_SPEED,
            // Player.cs only targets 30 when SlipCheck succeeds (the wall no
            // longer supports the lower probe). A full wall holds at zero.
            _ => 0.0,
        }
    };
    p.speed.y = approach(p.speed.y, target, CLIMB_ACCEL * DT);
    p.speed.x = 0.0;
    if p.climb_no_move_timer <= 0.0 {
        let cost = if input.move_y < 0 {
            CLIMB_UP_COST
        } else {
            CLIMB_STILL_COST
        };
        p.stamina = (p.stamina - cost * DT).max(0.0);
    }
}

fn climb_slip_check(p: &PlayerSnapshot, map: &Map, wall: i8) -> bool {
    let x = p.pos.x + if wall > 0 { 4.0 } else { -5.0 };
    let top = p.pos.y - 11.0;
    !map.solid_at(Rect::new(x, top + 4.0, 1.0, 1.0)) && !map.solid_at(Rect::new(x, top, 1.0, 1.0))
}

fn climb_hop(p: &mut PlayerSnapshot, map: &Map, wall: i8) {
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
    if p.speed.x.signum() == -(p.hop_wait_x as f32) || p.speed.y > 0.0 {
        p.hop_wait_x = 0;
        p.hop_wait_x_speed = 0.0;
    } else if !touching_wall(p, map, p.hop_wait_x) {
        p.speed.x = p.hop_wait_x_speed;
        p.hop_wait_x = 0;
        p.hop_wait_x_speed = 0.0;
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
        begin_dash(p, input, false, false);
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
    let horizontal_accel = if p.speed.x.abs() > SWIM_MAX && p.speed.x.signum() == x.signum() {
        SWIM_REDUCE
    } else {
        SWIM_ACCEL
    };
    p.speed.x = approach(p.speed.x, horizontal_max * x, horizontal_accel * DT);

    if y == 0.0 && swim_rise_check(p, map) {
        p.speed.y = approach(p.speed.y, SWIM_MAX_RISE, SWIM_ACCEL * DT);
    } else if y >= 0.0 || underwater {
        let vertical_accel = if p.speed.y.abs() > SWIM_MAX && p.speed.y.signum() == y.signum() {
            SWIM_REDUCE
        } else {
            SWIM_ACCEL
        };
        p.speed.y = approach(p.speed.y, SWIM_MAX * y, vertical_accel * DT);
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
}

fn boost_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    p.speed = Vec2::default();
    let aim = input_vector(input);
    let target = Vec2::new(
        p.boost_target.x + aim.x * 3.0,
        p.boost_target.y + if p.ducking { 3.0 } else { 5.0 } + aim.y * 3.0,
    );
    approach_exact_position(p, map, target, 80.0 * DT);
    p.state_timer = (p.state_timer - DT).max(0.0);
    let manual = input.dash_pressed || input.crouch_dash_pressed;
    if !manual && p.state_timer > 0.0 {
        return;
    }
    snap_to_boost_target(p, map);
    if p.boost_red {
        begin_red_dash(p, manual.then_some(input), !manual);
    } else {
        begin_dash(p, input, false, !manual);
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
    p.state_timer = DT * if delayed_coroutine { 2.0 } else { 1.0 };
    p.ducking = false;
}

fn red_dash_update(p: &mut PlayerSnapshot, input: InputState) {
    if (input.dash_pressed || input.crouch_dash_pressed)
        && p.dashes > 0
        && p.dash_cooldown_timer <= 0.0
    {
        begin_dash(p, input, true, false);
        return;
    }
    if p.dash_dir == Vec2::default() {
        p.state_timer = (p.state_timer - DT).max(0.0);
        if p.state_timer <= 0.0 {
            p.dash_dir = p.last_aim;
            p.speed = Vec2::new(p.dash_dir.x * DASH_SPEED, p.dash_dir.y * DASH_SPEED);
            if p.dash_dir.x != 0.0 {
                p.facing = p.dash_dir.x > 0.0;
            }
        }
    }
}

fn hit_squash_update(p: &mut PlayerSnapshot) {
    p.speed.x = approach(p.speed.x, 0.0, 800.0 * DT);
    p.speed.y = approach(p.speed.y, 0.0, 800.0 * DT);
    if p.state_timer > 0.0 {
        p.state_timer -= DT;
    } else {
        p.state = PlayerState::Normal;
    }
}

fn launch_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    if let Some(target_x) = p.launch_approach_x {
        move_towards_x(p, map, target_x, 60.0 * DT);
    }
    if (input.dash_pressed || input.crouch_dash_pressed)
        && p.dashes > 0
        && p.dash_cooldown_timer <= 0.0
    {
        begin_dash(p, input, true, false);
        return;
    }
    p.speed.y = approach(
        p.speed.y,
        MAX_FALL,
        GRAVITY * if p.speed.y < 0.0 { 0.5 } else { 0.25 } * DT,
    );
    p.speed.x = approach(p.speed.x, 0.0, RUN_ACCEL * 0.2 * DT);
    if length(p.speed) < LAUNCH_CANCEL_THRESHOLD {
        p.state = PlayerState::Normal;
        p.launch_approach_x = None;
    }
}

fn summit_launch_update(p: &mut PlayerSnapshot, map: &Map) {
    p.summit_launch_particle_timer -= DT;
    p.facing = true;
    move_towards_x(p, map, p.summit_launch_target_x, 20.0 * DT);
    p.speed = Vec2::new(0.0, -DASH_SPEED);
}

fn dummy_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    if can_unduck(p, map) {
        p.ducking = false;
    }
    if !p.on_ground && p.dummy_gravity {
        let gravity_mult =
            if p.speed.y.abs() < HALF_GRAV_THRESHOLD && (input.jump_held || p.auto_jump) {
                0.5
            } else {
                1.0
            };
        p.speed.y = approach(p.speed.y, p.max_fall, GRAVITY * gravity_mult * DT);
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
                RUN_ACCEL * 2.5 * DT,
            );
        }
        if p.dummy_friction {
            p.speed.x = approach(p.speed.x, 0.0, RUN_ACCEL * DT);
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
        p.speed.x = approach(p.speed.x, MAX_RUN * 0.6 * move_x, 325.0 * DT);
        if p.dummy_gravity {
            p.speed.y = approach(p.speed.y, MAX_FALL * 2.0, GRAVITY * 0.25 * DT);
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
        p.speed.y = approach(p.speed.y, -20.0, 400.0 * DT);
    } else {
        p.speed.y = approach(p.speed.y, MAX_FALL * 2.0, GRAVITY * 0.25 * DT);
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
            p.reflection_fall_wait_timer -= DT;
        }
        2 => {
            p.ignore_jump_thrus = false;
            p.state = PlayerState::Normal;
            p.max_fall = MAX_FALL;
        }
        _ => {}
    }
}

fn begin_star_fly(p: &mut PlayerSnapshot) {
    p.state = PlayerState::StarFly;
    p.star_fly_transforming = true;
    p.star_fly_transform_frames = STAR_FLY_TRANSFORM_FRAMES;
    p.star_fly_timer = STAR_FLY_TIME;
    p.star_fly_speed_lerp = 0.0;
    p.star_fly_last_dir = Vec2::default();
    p.jump_grace_timer = 0.0;
}

fn star_fly_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    if p.star_fly_transforming {
        p.speed = approach_vector(p.speed, Vec2::default(), STAR_FLY_TRANSFORM_DECEL * DT);
        p.star_fly_transform_frames = p.star_fly_transform_frames.saturating_sub(1);
        if p.star_fly_transform_frames == 0 {
            p.star_fly_transforming = false;
            p.star_fly_timer = STAR_FLY_TIME;
            p.dashes = p.dashes.max(1);
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
        current_dir = rotate_towards(current_dir, angle(aim), STAR_FLY_ROTATE_SPEED * DT);
    }
    p.star_fly_last_dir = current_dir;

    let max_speed = if slow {
        p.star_fly_speed_lerp = 0.0;
        STAR_FLY_SLOW_SPEED
    } else if current_dir != Vec2::default() && dot(current_dir, aim) >= 0.45 {
        p.star_fly_speed_lerp = approach(p.star_fly_speed_lerp, 1.0, DT);
        STAR_FLY_TARGET_SPEED + (STAR_FLY_MAX_SPEED - STAR_FLY_TARGET_SPEED) * p.star_fly_speed_lerp
    } else {
        p.star_fly_speed_lerp = 0.0;
        STAR_FLY_TARGET_SPEED
    };
    let speed = approach(length(p.speed), max_speed, STAR_FLY_ACCEL * DT);
    p.speed = scale(current_dir, speed);

    if input.jump_pressed {
        if grounded_at_offset(p, map, 3.0) {
            end_star_fly(p, map);
            p.state = PlayerState::Normal;
            p.jump_buffer_timer = 0.0;
            p.jump_grace_timer = 0.0;
            p.speed.y = JUMP_SPEED;
            p.speed.x += input.move_x as f32 * JUMP_H_BOOST;
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
        let right = input.move_x != -1 && touching_wall(p, map, 1);
        let left = input.move_x != 1 && touching_wall(p, map, -1);
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
        begin_dash(p, input, true, false);
        return;
    }

    p.star_fly_timer -= DT;
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

fn current_player_rect(p: &PlayerSnapshot, x: f32, y: f32) -> Rect {
    if p.state == PlayerState::StarFly {
        star_fly_rect(x, y)
    } else if p.ducking {
        duck_player_rect(x, y)
    } else {
        player_rect(x, y)
    }
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
fn wall_dir(p: &PlayerSnapshot, map: &Map) -> i8 {
    if touching_wall(p, map, -1) {
        -1
    } else if touching_wall(p, map, 1) {
        1
    } else {
        0
    }
}

fn move_axis(p: &mut PlayerSnapshot, map: &Map, horizontal: bool) {
    let speed = if horizontal { p.speed.x } else { p.speed.y };
    move_axis_amount(p, map, horizontal, speed * DT);
}

fn move_axis_amount(p: &mut PlayerSnapshot, map: &Map, horizontal: bool, amount: f32) {
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
        let collided = map.static_solid_at(next)
            || (dream_block && p.state != PlayerState::DreamDash)
            || (!horizontal
                && sign > 0
                && !p.ignore_jump_thrus
                && map.jump_thru_at(next, current_player_rect(p, p.pos.x, p.pos.y).bottom()));
        if collided {
            if dream_block
                && p.state == PlayerState::Dash
                && p.can_dream_dash
                && p.dash_attack_timer > 0.0
            {
                p.state = PlayerState::DreamDash;
                p.dream_dash_can_end_timer = 0.1;
                p.dash_end_pending = false;
                break;
            }
            if horizontal {
                if matches!(p.state, PlayerState::Dash | PlayerState::RedDash)
                    && p.speed.y == 0.0
                    && p.speed.x != 0.0
                {
                    for correction in 1..=4 {
                        for direction in [1.0, -1.0] {
                            let offset = correction as f32 * direction;
                            let corrected =
                                current_player_rect(p, p.pos.x + sign as f32, p.pos.y + offset);
                            if !map.solid_at(corrected) {
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
                if sign < 0 && p.state != PlayerState::StarFly && p.speed.y < 0.0 {
                    if p.speed.x <= 0.0 {
                        for correction in 1..=4 {
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
                        for correction in 1..=4 {
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

fn interact(p: &mut PlayerSnapshot, map: &Map, input: InputState) {
    if p.state == PlayerState::DreamDash {
        if !map.dream_block_at(current_player_rect(p, p.pos.x, p.pos.y))
            && p.dream_dash_can_end_timer <= 0.0
        {
            let horizontal_exit = p.dash_dir.x != 0.0;
            let wall = wall_dir(p, map);
            let dream_jump = input.jump_pressed && horizontal_exit;
            let dream_grab = input.grab_held
                && (p.dash_dir.y >= 0.0 || horizontal_exit)
                && ((input.move_x == 1 && wall == 1) || (input.move_x == -1 && wall == -1));

            if dream_jump {
                // DreamDashUpdate calls Jump before the state transition, then
                // DreamDashEnd restores horizontal-exit grace. That callback
                // ordering is why a buffered Dream Jump can jump a second time.
                p.state = PlayerState::Normal;
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
                p.state = PlayerState::Normal;
                p.auto_jump = true;
                p.auto_jump_timer = 0.0;
            }
            p.jump_grace_timer = if horizontal_exit { JUMP_GRACE } else { 0.0 };
            p.dashes = p.dashes.max(1);
            p.stamina = 110.0;
            p.dash_attack_timer = 0.0;
            p.freeze_timer = 0.05;
        }
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
    for (entity_index, entity) in map.entities.iter().enumerate() {
        let player_box = if matches!(
            entity.kind,
            EntityKind::Spikes | EntityKind::FlyFeather | EntityKind::Bumper
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
                player_box,
            ),
            EntityKind::Bumper => circle_rect_intersects(
                Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                ),
                entity.bounds.width * 0.5,
                player_box,
            ),
            _ => entity.bounds.intersects(player_box),
        };
        if !intersects {
            continue;
        }
        match entity.kind {
            EntityKind::Spikes if spike_is_lethal(p, entity.direction) => {
                p.dead = true;
                p.speed = Vec2::default();
                p.death_freeze_pending = true;
                p.respawn_frames = 95;
                return;
            }
            EntityKind::Water => {}
            EntityKind::Booster | EntityKind::RedBooster
                if !matches!(
                    p.state,
                    PlayerState::Boost | PlayerState::RedDash | PlayerState::HitSquash
                ) && (p.booster_reuse_timer <= 0.0
                    || p.last_booster_target
                        != Vec2::new(
                            entity.bounds.x + entity.bounds.width * 0.5,
                            entity.bounds.y + entity.bounds.height * 0.5 + 2.0,
                        )) =>
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
                p.state_timer = 0.25 + DT * 2.0;
                p.dashes = p.dashes.max(1);
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
                if p.bumper_reuse_timer <= 0.0 || p.last_bumper_target != target {
                    explode_launch(p, input, target, false, false);
                    p.last_bumper_target = target;
                    p.bumper_reuse_timer = 0.6;
                }
            }
            EntityKind::Spring if p.state != PlayerState::DreamDash => {
                if entity.direction.y < 0.0 {
                    if p.speed.y >= 0.0 {
                        super_bounce(p, entity.bounds.y);
                    }
                } else if entity.direction.x != 0.0 {
                    side_bounce(p, entity.direction.x.signum() as i8, entity.bounds);
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
        p.strawberry_follow_delay_timer = (p.strawberry_follow_delay_timer - DT).max(0.0);
        return;
    }

    // Strawberry.Update uses Player.OnSafeGround. Normal solids and
    // jumpthroughs are safe in the supported map subset, and Swim is always
    // treated as safe ground by Player.Update.
    if p.on_ground || p.state == PlayerState::Swim {
        p.strawberry_collect_timer += DT;
        if p.strawberry_collect_timer > 0.15 {
            p.carried_strawberries -= 1;
            p.strawberry_collect_index = p.strawberry_collect_index.saturating_add(1);
            p.strawberry_collect_reset_timer = 2.5;
            p.strawberry_collect_timer = if p.carried_strawberries > 0 {
                -0.15
            } else {
                0.0
            };
        }
    } else {
        p.strawberry_collect_timer = p.strawberry_collect_timer.min(0.0);
    }
}

fn reset_for_spring_bounce(p: &mut PlayerSnapshot) {
    p.dashes = p.dashes.max(1);
    p.stamina = 110.0;
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

fn super_bounce(p: &mut PlayerSnapshot, from_y: f32) {
    // Player.SuperBounce temporarily uses the normal collider and moves the
    // player's bottom onto the spring before applying the launch.
    p.pos.y = from_y;
    p.movement_remainder.y = 0.0;
    reset_for_spring_bounce(p);
    p.speed.x = 0.0;
    p.speed.y = SUPER_BOUNCE_SPEED;
    p.var_jump_speed = p.speed.y;
}

fn side_bounce(p: &mut PlayerSnapshot, dir: i8, spring: Rect) {
    // SideBounce aligns the normal collider to the spring face and only
    // corrects vertically by at most four pixels.
    let from_y = spring.y + spring.height * 0.5;
    p.pos.y += (from_y - p.pos.y).clamp(-4.0, 4.0);
    p.pos.x = if dir > 0 {
        spring.right() + 4.0
    } else {
        spring.x - 4.0
    };
    p.movement_remainder = Vec2::default();
    reset_for_spring_bounce(p);
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
    p.dashes = p.dashes.max(1);
    p.stamina = 110.0;
    p.dash_cooldown_timer = DASH_COOLDOWN;
    p.state = PlayerState::Launch;
    p.launched = true;
    direction
}

fn try_begin_badeline_boost(p: &mut PlayerSnapshot, map: &Map) -> bool {
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
    p.dashes = p.dashes.max(1);
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
    p.badeline_boost_relocation_elapsed += DT;
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

fn update_badeline_boost(p: &mut PlayerSnapshot, map: &Map) {
    let wait_frames = if p.badeline_boost_final { 12 } else { 6 };
    match p.badeline_boost_phase {
        0 if p.badeline_boost_frame < 11 => {
            let progress = (p.badeline_boost_frame as f32 + 1.0) * DT / 0.2;
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

fn begin_badeline_launch(p: &mut PlayerSnapshot, map: &Map) {
    p.badeline_boost_active = false;
    p.launch_approach_x = Some(p.last_badeline_boost_target.x);
    p.speed = Vec2::new(0.0, -330.0);
    p.auto_jump = true;
    p.dashes = p.dashes.max(1);
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

fn move_to_position(p: &mut PlayerSnapshot, map: &Map, target: Vec2) {
    let exact_x = p.pos.x + p.movement_remainder.x;
    move_axis_amount(p, map, true, target.x - exact_x);
    let exact_y = p.pos.y + p.movement_remainder.y;
    move_axis_amount(p, map, false, target.y - exact_y);
}

fn point_bounce(p: &mut PlayerSnapshot, from: Vec2) {
    if p.state == PlayerState::Dash {
        p.state = PlayerState::Normal;
    }
    p.dashes = p.dashes.max(1);
    p.stamina = 110.0;
    let collider = current_player_hurt_rect(p);
    let center = Vec2::new(
        collider.x + collider.width * 0.5,
        collider.y + collider.height * 0.5,
    );
    let mut direction = normalize(Vec2::new(center.x - from.x, center.y - from.y));
    if direction.y > -0.2 && direction.y <= 0.4 {
        direction.y = -0.2;
    }
    p.speed = scale(direction, 220.0);
    p.speed.x *= 1.5;
    if p.speed.x.abs() < 100.0 {
        p.speed.x = if p.speed.x == 0.0 {
            if p.facing { -100.0 } else { 100.0 }
        } else {
            p.speed.x.signum() * 100.0
        };
    }
}

fn enforce_level_bounds(p: &mut PlayerSnapshot, map: &Map) {
    if p.dead || p.state == PlayerState::DreamDash || !player_in_control(p.state) {
        return;
    }
    let bounds = p.current_room_bounds.unwrap_or(map.bounds);
    let mut collider = current_player_rect(p, p.pos.x, p.pos.y);
    if collider.x < bounds.x {
        let center = Vec2::new(p.pos.x, collider.y + collider.height * 0.5);
        if let Some(next) = transition_room_at(map, p, Vec2::new(center.x - 8.0, center.y)) {
            begin_transition(p, next, Vec2::new(-1.0, 0.0));
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
            begin_transition(p, next, Vec2::new(1.0, 0.0));
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
            begin_transition(p, next, Vec2::new(0.0, -1.0));
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
            begin_transition(p, next, Vec2::new(0.0, 1.0));
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

fn begin_transition(p: &mut PlayerSnapshot, next: Rect, direction: Vec2) {
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
        target.y = next.bottom() - 13.0;
    }
    p.transition_room_bounds = Some(next);
    p.transition_direction = direction;
    p.transition_target = target;
    p.transition_timer = TRANSITION_TIME;
    p.on_ground = false;
}

fn update_transition(p: &mut PlayerSnapshot) {
    let max_move = TRANSITION_MOVE_SPEED * DT;
    p.pos.x = approach(p.pos.x, p.transition_target.x, max_move);
    p.pos.y = approach(p.pos.y, p.transition_target.y, max_move);
    p.transition_timer = (p.transition_timer - DT).max(0.0);
    p.on_ground = false;
    if p.transition_timer <= 0.0 && p.pos == p.transition_target {
        p.movement_remainder = Vec2::default();
        p.speed.x = p.speed.x.round();
        p.speed.y = p.speed.y.round();
        p.wall_slide_timer = WALL_SLIDE_TIME;
        p.jump_grace_timer = 0.0;
        p.force_move_x_timer = 0.0;
        p.dashes = p.dashes.max(1);
        p.stamina = 110.0;
        p.current_room_bounds = p.transition_room_bounds.take();
        p.transition_direction = Vec2::default();
    }
}

fn move_towards_x(p: &mut PlayerSnapshot, map: &Map, target_x: f32, max_move: f32) {
    let exact_x = p.pos.x + p.movement_remainder.x;
    let next_x = approach(exact_x, target_x, max_move);
    move_axis_amount(p, map, true, next_x - exact_x);
}

fn approach_exact_position(p: &mut PlayerSnapshot, map: &Map, target: Vec2, max_move: f32) {
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

fn snap_to_boost_target(p: &mut PlayerSnapshot, map: &Map) {
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
    let collided = map.static_solid_at(next)
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
    p.wind.x = approach(p.wind.x, p.wind_target.x, WIND_ACCEL * DT);
    p.wind.y = approach(p.wind.y, p.wind_target.y, WIND_ACCEL * DT);
}

fn apply_wind_movement(p: &mut PlayerSnapshot, map: &Map) {
    if !player_in_control(p.state)
        || p.no_wind_timer > 0.0
        || matches!(
            p.state,
            PlayerState::Boost | PlayerState::Dash | PlayerState::SummitLaunch
        )
    {
        return;
    }

    let mut move_x = p.wind.x * WIND_MOVE_MULT * DT;
    if move_x != 0.0 && p.state != PlayerState::Climb {
        let shield_x = p.pos.x - move_x.signum() * WIND_WALL_DISTANCE;
        if !map.solid_at(current_player_rect(p, shield_x, p.pos.y)) {
            if p.ducking && p.on_ground {
                move_x = 0.0;
            }
            move_axis_amount(p, map, true, move_x);
        }
    }

    let mut move_y = p.wind.y * WIND_MOVE_MULT * DT;
    if move_y != 0.0 && (p.speed.y < 0.0 || !grounded(p, map)) {
        if p.state == PlayerState::Climb {
            if move_y > 0.0 && p.climb_no_move_timer <= 0.0 {
                move_y *= 0.4;
            } else {
                return;
            }
        }
        move_axis_amount(p, map, false, move_y);
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

fn spike_is_lethal(p: &PlayerSnapshot, direction: Vec2) -> bool {
    if direction.y < 0.0 {
        p.speed.y >= 0.0
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

fn approach(value: f32, target: f32, max_move: f32) -> f32 {
    if value < target {
        (value + max_move).min(target)
    } else {
        (value - max_move).max(target)
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
    fn coyote_jump_consumes_source_grace_window_after_leaving_a_ledge() {
        let map = Map {
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
        assert_eq!(trace.states.last().unwrap().jump_buffer_timer, 0.0);
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
    fn reverse_super_uses_jump_frame_facing_not_dash_direction() {
        let mut inputs = [InputState::default(); 5];
        inputs[0] = InputState {
            move_x: 1,
            dash_pressed: true,
            ..InputState::default()
        };
        inputs[4] = InputState {
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
        let map = Map {
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
        let map = Map {
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
            pos: Vec2::new(96.0, 91.0),
            dashes: 1,
            ..PlayerSnapshot::default()
        };
        let mut on_time = [InputState::default(); 6];
        on_time[0] = InputState {
            move_y: -1,
            dash_pressed: true,
            ..InputState::default()
        };
        on_time[4] = InputState {
            move_y: -1,
            jump_pressed: true,
            jump_held: true,
            ..InputState::default()
        };
        let on_time = simulate_trace(p.clone(), &on_time, &map, on_time.len() as u32).unwrap();
        assert!(!on_time.states.last().unwrap().dead);
        assert_eq!(on_time.states[5].state, PlayerState::Normal);
        assert_eq!(on_time.states[5].speed, Vec2::new(-170.0, -160.0));

        let mut late = [InputState::default(); 6];
        late[0] = InputState {
            move_y: -1,
            dash_pressed: true,
            ..InputState::default()
        };
        late[5] = InputState {
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
    fn climbhop_waits_for_the_body_to_clear_the_ledge_before_horizontal_launch() {
        let map = Map {
            solids: vec![Rect::new(40.0, 80.0, 8.0, 100.0)],
            ..Map::default()
        };
        let p = PlayerSnapshot {
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
        let trace = simulate_trace(p, &[input; 4], &map, 4).unwrap();
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
    fn upward_corner_correction_moves_around_a_one_pixel_ceiling_overlap() {
        let map = Map {
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
        let map = Map {
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
    fn dash_attack_survives_dash_end_and_breaks_a_late_feather_shield() {
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
    fn neutral_climb_jump_converts_to_wallboost_and_refunds_stamina() {
        let map = Map {
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
    fn stamina_cancel_regrabs_to_reset_the_no_move_cost_window() {
        let map = Map {
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
    fn cornerboost_restores_retained_speed_after_clearing_wall_top() {
        let map = Map {
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
    fn dash_spends_dash_and_is_diagonal_normalized() {
        let input = InputState {
            move_x: 1,
            move_y: -1,
            dash_pressed: true,
            ..InputState::default()
        };
        let first = simulate(grounded_player(), &[input], &floor_map(), 1).unwrap();
        assert_eq!(first.speed, Vec2::default());
        assert_eq!(first.freeze_timer, 0.05);
        let frozen = simulate(first, &[InputState::default(); 3], &floor_map(), 3).unwrap();
        assert_eq!(frozen.speed, Vec2::default());
        assert_eq!(frozen.freeze_timer, 0.0);
        let p = simulate(frozen, &[InputState::default()], &floor_map(), 1).unwrap();
        assert_eq!(p.state, PlayerState::Dash);
        assert_eq!(p.dashes, 0);
        assert!((p.speed.x.abs() - 169.70563).abs() < 0.01);
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
    fn archie_centers_the_duck_hitbox_two_pixels_above_a_normal_boost() {
        let map = booster_map();
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
        assert_eq!(max_height_gain, 2.0);
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
    fn dream_dash_enters_a_dream_block() {
        let map = Map {
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
        assert_eq!(p.jump_grace_timer, JUMP_GRACE);
        assert_eq!(p.var_jump_timer, VAR_JUMP_TIME);
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
        assert!(!p.facing);
        assert_eq!(p.jump_grace_timer, JUMP_GRACE);
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
            state: PlayerState::Pickup,
            ..PlayerSnapshot::default()
        };
        assert_eq!(
            simulate(p, &[InputState::default()], &Map::default(), 1),
            Err(SimulationError::UnsupportedState(PlayerState::Pickup))
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
    fn upward_screen_transition_applies_source_launch_and_completion_refills() {
        let map = Map {
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
        assert_eq!(trace.states[completed].pos.y, -13.0);
        assert_eq!(trace.states[completed].dashes, 1);
        assert_eq!(trace.states[completed].stamina, 110.0);
        assert_eq!(trace.states[completed].wall_slide_timer, WALL_SLIDE_TIME);
        assert_eq!(trace.states[completed].jump_grace_timer, 0.0);
    }

    #[test]
    fn downward_screen_transition_clamps_upward_speed_before_transfer() {
        let map = Map {
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
    fn star_fly_wall_collision_uses_half_speed_bounce() {
        let map = Map {
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
    fn shielded_feather_uses_v14_point_bounce() {
        let p = PlayerSnapshot {
            pos: Vec2::new(120.0, 200.0),
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &feather_map(true), 1).unwrap();
        assert_eq!(p.state, PlayerState::Normal);
        assert_eq!(p.speed, Vec2::new(-100.0, -220.0));
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
        assert_eq!(p.last_bumper_target, Vec2::new(600.0, 200.0));
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
        ];
        let trace = simulate_trace(p, &inputs, &spring_map(Vec2::new(0.0, -1.0)), 3).unwrap();
        assert_eq!(trace.states[2].state, PlayerState::Normal);
        assert_eq!(trace.states[2].dashes, 1);
        assert_eq!(trace.states[2].speed, Vec2::new(0.0, SUPER_BOUNCE_SPEED));
        assert!(trace.states[2].dash_buffer_timer > 0.0);
        assert_eq!(trace.states[3].state, PlayerState::Dash);
        assert_eq!(trace.states[3].dashes, 0);
        assert_eq!(trace.states[3].speed, Vec2::default());
        assert_eq!(trace.states[3].dash_buffer_timer, 0.0);
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
        assert_eq!(second - first, 18);
        assert_eq!(trace.states[first].strawberry_collect_timer, -0.15);
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
        let map = Map {
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
    fn dummy_state_uses_source_gravity_and_friction() {
        let p = PlayerSnapshot {
            pos: Vec2::new(160.0, 120.0),
            speed: Vec2::new(200.0, -100.0),
            state: PlayerState::Dummy,
            ..PlayerSnapshot::default()
        };
        let p = simulate(p, &[InputState::default()], &Map::default(), 1).unwrap();
        assert!((p.speed.x - 141.666_58).abs() < 0.001);
        assert!((p.speed.y - -84.999_97).abs() < 0.001);
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
        let map = Map {
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
        let map = Map {
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
}
