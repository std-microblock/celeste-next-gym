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
    let mut runtime_map = map.clone();
    position_moving_solids(&mut runtime_map, snapshot.moving_solid_time);
    let mut states = Vec::with_capacity(frames + 1);
    states.push(snapshot.clone());
    for input in &inputs[..frames] {
        step(&mut snapshot, input.normalized(), &mut runtime_map)?;
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
        s.bounce_reuse_timer,
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
        s.current_lift_speed.x,
        s.current_lift_speed.y,
        s.last_lift_speed.x,
        s.last_lift_speed.y,
        s.lift_speed_timer,
        s.moving_solid_time,
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

fn position_moving_solids(map: &mut Map, time: f32) {
    for entity in &mut map.entities {
        if entity.kind == EntityKind::MovingSolid {
            entity.bounds.x += (entity.direction.x * time).round_ties_even();
            entity.bounds.y += (entity.direction.y * time).round_ties_even();
        }
    }
}

fn set_lift_speed(p: &mut PlayerSnapshot, speed: Vec2) {
    p.current_lift_speed = speed;
    if speed != Vec2::default() {
        p.last_lift_speed = speed;
        p.lift_speed_timer = 0.16;
    }
}

fn advance_moving_solids(p: &mut PlayerSnapshot, map: &mut Map) {
    let old_time = p.moving_solid_time;
    let new_time = old_time + DT;
    for entity in &mut map.entities {
        if entity.kind != EntityKind::MovingSolid {
            continue;
        }
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
    p.moving_solid_time = new_time;
}

fn step(
    p: &mut PlayerSnapshot,
    mut input: InputState,
    map: &mut Map,
) -> Result<(), SimulationError> {
    // VirtualButton.Update runs in MInput before Celeste.Freeze can skip the
    // Scene. It subtracts DeltaTime first, then a new press restores the full
    // buffer; Jump also clears its buffer as soon as the binding is not held.
    p.jump_buffer_timer -= DT;
    if input.jump_pressed {
        p.jump_buffer_timer = 0.1;
    } else if !input.jump_held {
        p.jump_buffer_timer = 0.0;
    }
    // Dash and CrouchDash use a 0.08 second VirtualButton buffer. Their
    // portable input contract only records press edges, so keep the existing
    // press buffer alive across freeze until it is consumed or expires.
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
    advance_moving_solids(p, map);
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
    let was_on_ground = p.on_ground;
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

    if p.badeline_boost_active {
        update_badeline_boost(p, map);
        p.on_ground = grounded(p, map);
        return Ok(());
    }

    match p.state {
        PlayerState::Normal => normal_update(p, input, map, was_on_ground),
        PlayerState::Dash => dash_update(p, input, map),
        PlayerState::Climb => climb_update(p, input, map),
        PlayerState::Swim => swim_update(p, input, map),
        PlayerState::Boost => boost_update(p, input, map),
        PlayerState::RedDash => red_dash_update(p, input, map),
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

    // Actor.Update runs after the StateMachine component callback. Moving
    // platforms have already written currentLiftSpeed before Player.Update;
    // actions above can consume it, then Actor clears current and advances the
    // retained 0.16-second grace window before player movement.
    tick_lift_speed(p);

    // After components/coroutines update but before movement, Player.Update
    // restores the normal collider while rising/falling in open air. A
    // downward air dash therefore only becomes crouched again when it lands.
    if p.ducking && p.speed.y > 0.0 && !p.on_ground && p.jump_grace_timer <= 0.0 {
        p.ducking = false;
    }

    if p.state != PlayerState::DreamDash {
        move_axis(p, map, true);
    }
    if p.state != PlayerState::DreamDash {
        move_axis(p, map, false);
    }
    interact(p, map, input);
    try_begin_badeline_boost(p, map);
    enforce_level_bounds(p, map);
    p.on_ground = grounded(p, map);
    Ok(())
}

fn tick_timers(p: &mut PlayerSnapshot) {
    if p.auto_jump_timer > 0.0 {
        if p.auto_jump {
            p.auto_jump_timer = (p.auto_jump_timer - DT).max(0.0);
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
        &mut p.bounce_reuse_timer,
        &mut p.no_wind_timer,
        &mut p.jump_grace_timer,
        &mut p.var_jump_timer,
        &mut p.force_move_x_timer,
        &mut p.climb_no_move_timer,
        &mut p.dream_dash_can_end_timer,
    ] {
        *timer = (*timer - DT).max(0.0);
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
        p.lift_speed_timer -= DT;
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

fn normal_update(p: &mut PlayerSnapshot, input: InputState, map: &Map, was_on_ground: bool) {
    let boost = lift_boost(p);
    if boost.y < 0.0 && was_on_ground && !p.on_ground && p.speed.y >= 0.0 {
        p.speed.y = boost.y;
    }
    if (input.dash_pressed || input.crouch_dash_pressed)
        && p.dashes > 0
        && p.dash_cooldown_timer <= 0.0
    {
        add_lift_boost(p);
        begin_dash(p, input, true, false, map);
        return;
    }

    let wall = wall_dir(p, map);
    if input.grab_held && !p.on_ground && wall != 0 && p.stamina > 0.0 {
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
            add_lift_boost(p);
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
            add_lift_boost(p);
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
    map: &Map,
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
    if !p.on_ground && p.ducking && can_unduck(p, map) {
        p.ducking = false;
    } else if !p.ducking && (p.demo_dashed || input.move_y > 0) {
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
    add_lift_boost(p);
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
    add_lift_boost(p);
    p.var_jump_speed = p.speed.y;
    p.launched = true;
}

fn climb_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    let wall = if p.facing { 1 } else { -1 };
    if !input.grab_held || p.stamina <= 0.0 || !touching_wall(p, map, wall) {
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
        add_lift_boost(p);
        p.var_jump_speed = p.speed.y;
        p.var_jump_timer = VAR_JUMP_TIME;
        return;
    }
    let target = if p.climb_no_move_timer > 0.0 {
        0.0
    } else {
        match input.move_y {
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
    naive_move(p, Vec2::new(p.speed.x * DT, p.speed.y * DT));
}

fn boost_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    p.speed = Vec2::default();
    let aim = input_vector(input);
    let target = Vec2::new(
        p.boost_target.x + aim.x * 3.0,
        p.boost_target.y + if p.ducking { 3.0 } else { 5.5 } + aim.y * 3.0,
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
    p.state_timer = DT * if delayed_coroutine { 2.0 } else { 1.0 };
    p.ducking = false;
}

fn red_dash_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    if (input.dash_pressed || input.crouch_dash_pressed)
        && p.dashes > 0
        && p.dash_cooldown_timer <= 0.0
    {
        begin_dash(p, input, true, false, map);
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
        begin_dash(p, input, true, false, map);
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
        begin_dash(p, input, true, false, map);
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

fn current_player_rect(p: &PlayerSnapshot, x: f32, y: f32) -> Rect {
    if p.state == PlayerState::StarFly || p.star_fly_hitbox_preserved {
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
        let collided = map.non_dream_solid_at(next)
            || (dream_block && p.state != PlayerState::DreamDash)
            || (!horizontal
                && sign > 0
                && !p.ignore_jump_thrus
                && map.jump_thru_at(next, current_player_rect(p, p.pos.x, p.pos.y).bottom()));
        if collided {
            if dream_block
                && p.can_dream_dash
                && (p.dash_attack_timer > 0.0 || p.state == PlayerState::RedDash)
            {
                if horizontal {
                    p.movement_remainder.x = 0.0;
                } else {
                    p.movement_remainder.y = 0.0;
                }
                p.state = PlayerState::DreamDash;
                p.speed = Vec2::new(p.dash_dir.x * DASH_SPEED, p.dash_dir.y * DASH_SPEED);
                p.dream_dash_can_end_timer = 0.1;
                p.stamina = 110.0;
                p.dash_attack_timer = 0.0;
                p.dash_end_pending = false;
                break;
            }
            if horizontal {
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
            let dream_jump = input.jump_pressed && horizontal_exit;
            if !dream_jump && (p.dash_dir.y >= 0.0 || horizontal_exit) {
                // Celeste 1.4 pulls a horizontal exit five pixels back toward
                // the DreamBlock before its two-sided ClimbCheck. DreamDash is
                // still the active state here, so this is the source's naive
                // MoveHExact correction rather than normal solid movement.
                if p.dash_dir.x > 0.0
                    && map.non_dream_solid_at(current_player_rect(p, p.pos.x - 5.0, p.pos.y))
                {
                    p.pos.x -= 5.0;
                } else if p.dash_dir.x < 0.0
                    && map.non_dream_solid_at(current_player_rect(p, p.pos.x + 5.0, p.pos.y))
                {
                    p.pos.x += 5.0;
                }
            }
            let wall = wall_dir(p, map);
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

            // DreamDashUpdate performs its naive 240 px/s move before
            // returning the next state. Player.Update then performs the
            // ordinary movement pass in that resulting state on the same
            // frame, which is visible as a second displacement in Everest.
            move_axis(p, map, true);
            move_axis(p, map, false);
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
    for entity in &map.entities {
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
            EntityKind::IceBall => {
                let center = Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                );
                Rect::new(center.x - 8.0, center.y - 3.0, 16.0, 6.0).intersects(player_box)
            }
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
            EntityKind::IceBall => {
                let target = Vec2::new(
                    entity.bounds.x + entity.bounds.width * 0.5,
                    entity.bounds.y + entity.bounds.height * 0.5,
                );
                if (p.bounce_reuse_timer <= 0.0 || p.last_bounce_target != target)
                    && p.speed.y >= 0.0
                    && current_player_rect(p, p.pos.x, p.pos.y).bottom() <= target.y + 4.0
                {
                    bounce(p, map, target.y - 2.0);
                    p.last_bounce_target = target;
                    // A cold FireBall becomes non-collidable after the bounce.
                    p.bounce_reuse_timer = f32::MAX;
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

fn bounce(p: &mut PlayerSnapshot, map: &Map, from_y: f32) {
    let restore_star_fly_hitbox = p.state == PlayerState::StarFly || p.star_fly_hitbox_preserved;
    let restore_duck_hitbox = !restore_star_fly_hitbox && p.ducking;

    // Player.Bounce temporarily assigns normalHitbox before MoveVExact, so
    // the correction uses Madeline's ordinary 8x11 body even when a feather
    // or crouched collider entered the callback.
    let move_y = (from_y - p.pos.y) as i32;
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

    p.dashes = p.dashes.max(1);
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
    let mut collider = current_player_rect(p, p.pos.x, p.pos.y);
    if collider.x < map.bounds.x {
        p.pos.x += map.bounds.x - collider.x;
        p.speed.x = 0.0;
        collider = current_player_rect(p, p.pos.x, p.pos.y);
    }
    let right = map.bounds.x + map.bounds.width;
    if collider.x + collider.width > right {
        p.pos.x -= collider.x + collider.width - right;
        p.speed.x = 0.0;
        collider = current_player_rect(p, p.pos.x, p.pos.y);
    }

    let top = map.bounds.y;
    let center_y = collider.y + collider.height * 0.5;
    if center_y < top && collider.y < top - 24.0 {
        p.pos.y += top - 24.0 - collider.y;
        p.speed.y = 0.0;
        collider = current_player_rect(p, p.pos.x, p.pos.y);
    }

    let bottom = map.bounds.y + map.bounds.height;
    if collider.y > bottom + 4.0 {
        p.dead = true;
        p.speed = Vec2::default();
        p.death_freeze_pending = true;
        p.respawn_frames = 95;
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
        let map = moving_solid_map(Vec2::new(60.0, -120.0));
        let p = simulate(grounded_player(), &[InputState::default()], &map, 1).unwrap();

        assert_eq!(p.pos, Vec2::new(33.0, 98.0));
        assert!(p.on_ground);
        assert_eq!(p.current_lift_speed, Vec2::default());
        assert_eq!(p.last_lift_speed, Vec2::new(60.0, -120.0));
        assert!((p.lift_speed_timer - (0.16 - DT)).abs() < 0.000_001);
    }
    #[test]
    fn moving_solid_jump_combines_carrying_with_same_frame_lift_boost() {
        let map = moving_solid_map(Vec2::new(60.0, -120.0));
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
        let map = moving_solid_map(Vec2::new(60.0, 0.0));
        let inputs = [InputState::default(); 2];
        let direct = simulate(grounded_player(), &inputs, &map, 2).unwrap();
        let first = simulate(grounded_player(), &inputs[..1], &map, 1).unwrap();
        let split = simulate(first, &inputs[1..], &map, 1).unwrap();

        assert_eq!(split, direct);
        assert_eq!(direct.pos.x, 34.0);
    }
    #[test]
    fn moving_solid_pushes_an_actor_without_granting_rider_lift_speed() {
        let map = Map {
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
        let map = Map {
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
        assert!(trace.states.last().unwrap().jump_buffer_timer <= 0.0);
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
    fn archie_preserves_the_source_two_and_a_half_pixel_center_offset() {
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
        let map = Map {
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
    fn dream_dash_check_uses_lingering_attack_and_then_moves_naively() {
        let map = Map {
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
        let map = Map {
            bounds: Rect::new(0.0, -100.0, 960.0, 280.0),
            ..Map::default()
        };
        let trace = simulate_trace(p, &inputs, &map, 4).unwrap();

        assert_eq!(trace.states[1].jump_buffer_timer, 0.1);
        assert!((trace.states[2].jump_buffer_timer - (0.1 - DT)).abs() < 0.000_001);
        assert!((trace.states[3].jump_buffer_timer - (0.1 - DT * 2.0)).abs() < 0.000_001);
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
        let map = Map {
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
        assert_eq!(p.pos, Vec2::new(71.0, 64.0));
        assert!(!p.facing);
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
    fn ice_ball_bounce_cancels_dash_and_preserves_horizontal_speed() {
        let p = PlayerSnapshot {
            pos: Vec2::new(96.0, 100.0),
            speed: Vec2::new(240.0, 0.0),
            state: PlayerState::Dash,
            dashes: 0,
            dash_attack_timer: DASH_ATTACK_TIME,
            ..PlayerSnapshot::default()
        };
        let bounced = simulate(
            p,
            &[InputState {
                jump_held: true,
                ..InputState::default()
            }],
            &ice_ball_map(),
            1,
        )
        .unwrap();
        assert_eq!(bounced.state, PlayerState::Normal);
        assert_eq!(bounced.pos, Vec2::new(100.0, 98.0));
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
    fn ice_ball_feather_cancel_restores_star_fly_collider_after_normal_hurtbox() {
        let p = PlayerSnapshot {
            pos: Vec2::new(100.0, 100.0),
            state: PlayerState::StarFly,
            star_fly_timer: 1.0,
            ..PlayerSnapshot::default()
        };
        let bounced = simulate(p, &[InputState::default()], &ice_ball_map(), 1).unwrap();
        assert_eq!(bounced.state, PlayerState::Normal);
        assert!(bounced.star_fly_hitbox_preserved);
        assert!(!bounced.ducking);
        assert_eq!(
            current_player_rect(&bounced, bounced.pos.x, bounced.pos.y),
            star_fly_rect(bounced.pos.x, bounced.pos.y)
        );
        assert_eq!(
            current_player_hurt_rect(&bounced),
            player_hurt_rect(bounced.pos.x, bounced.pos.y)
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
            .find(|state| state.bounce_reuse_timer > 0.0)
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
