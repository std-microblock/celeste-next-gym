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
const JUMP_BUFFER_TIME: f32 = 0.08;
const JUMP_SPEED: f32 = -105.0;
const JUMP_H_BOOST: f32 = 40.0;
const VAR_JUMP_TIME: f32 = 0.2;
const WALL_JUMP_H: f32 = 130.0;
const WALL_JUMP_CHECK_DIST: f32 = 3.0;
const WALL_SLIDE_START_MAX: f32 = 20.0;
const WALL_SLIDE_TIME: f32 = 1.2;
const DASH_SPEED: f32 = 240.0;
const END_DASH_SPEED: f32 = 160.0;
const DASH_TIME: f32 = 0.15;
const DASH_COOLDOWN: f32 = 0.2;
const DASH_ATTACK_TIME: f32 = 0.3;
const DASH_CORNER_CORRECTION: i32 = 4;
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
const CLIMB_CHECK_DIST: f32 = 2.0;
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

/// A reusable simulation context. Constructing a context performs the one-time
/// map clone and entity-state initialization; subsequent calls only advance the
/// already initialized runtime map. This is the hot path used by native and
/// WASM callers that evaluate many action sequences against one map.
#[derive(Clone)]
pub struct Simulator {
    snapshot: PlayerSnapshot,
    runtime_map: Map,
}

impl Simulator {
    pub fn new(mut snapshot: PlayerSnapshot, map: &Map) -> Result<Self, SimulationError> {
        validate_snapshot(&snapshot)?;
        if !snapshot.player_on_ground_initialized {
            snapshot.player_on_ground = snapshot.on_ground;
            snapshot.player_on_ground_initialized = true;
        }
        let mut runtime_map = map.clone();
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
        initialize_temple_gates(&mut snapshot, &mut runtime_map);
        initialize_cassette_blocks(&mut snapshot, &mut runtime_map);
        initialize_spinners(&mut snapshot, &mut runtime_map);
        initialize_bumpers(&mut snapshot, &mut runtime_map);
        initialize_lookouts(&mut snapshot, &runtime_map);
        position_moving_solids(&mut runtime_map, snapshot.moving_solid_time);
        Ok(Self {
            snapshot,
            runtime_map,
        })
    }

    pub fn snapshot(&self) -> &PlayerSnapshot {
        &self.snapshot
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
        )?;
        Ok(&self.snapshot)
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
        p.cassette_manager = crate::CassetteManagerSnapshot::default();
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
        park_entity(&mut map.entities[entity_index]);
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
    entity.bounds.y += move_y;
    state.position.y += move_y;
    if riding {
        p.pos.y += move_y;
        set_lift_speed(p, Vec2::new(0.0, lift_y));
    } else if move_y < 0.0 {
        let player = current_player_rect(p, p.pos.x, p.pos.y);
        if entity.bounds.intersects(player) && player.bottom() > entity.bounds.y {
            p.pos.y += entity.bounds.y - player.bottom();
            set_lift_speed(p, Vec2::new(0.0, lift_y));
        }
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
                let speed = state.speed;
                move_cloud_v(p, entity, &mut state, amount, speed);
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
                    | EntityKind::MoveBlock
                    | EntityKind::MovingSolid
                    | EntityKind::ZipMover
                    | EntityKind::TempleGate
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

fn close_temple_gate(
    p: &mut PlayerSnapshot,
    map: &mut Map,
    gate_index: usize,
    gate: &mut crate::TempleGateSnapshot,
) {
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
    let moved_gate = Rect::new(gate.position.x, temporary_y, 8.0, temporary_height);
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
    map.entities[gate_index].bounds =
        Rect::new(gate.position.x, gate.position.y, 8.0, gate.closed_height);
}

fn advance_temple_gates(p: &mut PlayerSnapshot, map: &mut Map) {
    let entity_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::TempleGate).then_some(index))
        .collect();
    for (snapshot_index, entity_index) in entity_indices.into_iter().enumerate() {
        let mut gate = p.temple_gates[snapshot_index].clone();
        let player_left = current_player_rect(p, p.pos.x, p.pos.y).x;
        if gate.open && !gate.triggered && player_left > gate.position.x + 12.0 {
            close_temple_gate(p, map, entity_index, &mut gate);
        }
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
    bounds.intersects(current_player_rect(p, p.pos.x, p.pos.y + 1.0))
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
                | EntityKind::DreamBlock
                | EntityKind::MoveBlock
                | EntityKind::MovingSolid
                | EntityKind::ZipMover
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
        if horizontal {
            p.pos.x += amount;
        } else {
            p.pos.y += amount;
        }
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
                        | EntityKind::MoveBlock
                        | EntityKind::MovingSolid
                        | EntityKind::ZipMover
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

fn advance_move_blocks(p: &mut PlayerSnapshot, map: &mut Map, input: InputState) {
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
                let riding = player_riding_solid(p, map.entities[entity_index].bounds);
                let mut target_angle = home_angle;
                if riding {
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
        p.move_blocks[block_index] = state;
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
    let Some(index) = p.theo_crystals.iter().enumerate().position(|(index, theo)| {
        // LaunchUpdate, unlike NormalUpdate and DashUpdate, does not guard
        // its Holdable loop with `Holding == null`. Holdable.Pickup itself
        // permits the current holder to pick up the same entity again, which
        // restarts PickupCoroutine after a Bumper launch freeze.
        (!theo.held || p.holding_theo == Some(index as u16))
            && theo.cannot_hold_timer <= 0.0
            && theo_pickup_rect(theo.position).intersects(player)
    }) else {
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

fn pickup_update(p: &mut PlayerSnapshot) {
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

fn initialize_temple_gates(p: &mut PlayerSnapshot, map: &mut Map) {
    let gate_indices: Vec<usize> = map
        .entities
        .iter()
        .enumerate()
        .filter_map(|(index, entity)| (entity.kind == EntityKind::TempleGate).then_some(index))
        .collect();
    p.temple_gates.truncate(gate_indices.len());
    for (gate_index, entity_index) in gate_indices.into_iter().enumerate() {
        let entity = &mut map.entities[entity_index];
        if gate_index == p.temple_gates.len() {
            p.temple_gates.push(crate::TempleGateSnapshot {
                position: Vec2::new(entity.bounds.x, entity.bounds.y),
                current_height: 0.0,
                closed_height: entity.bounds.height,
                open: true,
                triggered: false,
            });
        }
        let gate = &p.temple_gates[gate_index];
        entity.bounds = Rect::new(gate.position.x, gate.position.y, 8.0, gate.current_height);
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
    map.entities.iter().any(|entity| {
        entity.name == "lookoutBlocker" && entity.bounds.intersects(viewport)
    })
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
        1 => {
            p.state = PlayerState::Dummy;
            p.dummy_moving = true;
            let direction = (lookout.position.x - p.pos.x).signum();
            if direction != 0.0 {
                p.facing = direction > 0.0;
            }
        }
        2 | 3 => {
            p.state = PlayerState::Dummy;
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
) -> bool {
    let nodes = &entity.nodes;
    if nodes.is_empty() {
        return false;
    }
    let node = (state.node as usize).min(nodes.len() - 1);
    state.node = node as u16;
    let start_center = Vec2::new(state.cam_start.x + 160.0, state.cam_start.y + 90.0);
    let begin = if node == 0 { start_center } else { nodes[node - 1] };
    let end = nodes[node];
    let percent = state.node_percent;
    let center = if percent < 0.25 && node > 0 {
        let curve_begin = lerp_vec(
            if node <= 1 { start_center } else { nodes[node - 2] },
            begin,
            0.75,
        );
        let curve_end = lerp_vec(begin, end, 0.25);
        quadratic_curve(
            curve_begin,
            curve_end,
            begin,
            0.5 + percent / 0.25 * 0.5,
        )
    } else if percent > 0.75 && node + 1 < nodes.len() {
        let curve_begin = lerp_vec(begin, end, 0.75);
        let curve_end = lerp_vec(end, nodes[node + 1], 0.25);
        quadratic_curve(
            curve_begin,
            curve_end,
            end,
            (percent - 0.75) / 0.25 * 0.5,
        )
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
            return entity.direction.y != 0.0;
        }
    }
    false
}

fn finish_lookout(p: &mut PlayerSnapshot, state: &mut crate::LookoutSnapshot) {
    state.interacting = false;
    state.phase = 0;
    state.timer = 0.0;
    state.hud_easer = 0.0;
    p.state = PlayerState::Normal;
    p.dummy_moving = false;
}

fn advance_lookouts(p: &mut PlayerSnapshot, map: &Map, input: InputState) {
    let indices = lookout_entity_indices(map);
    let Some((lookout_index, entity_index)) = indices
        .into_iter()
        .enumerate()
        .find(|(lookout_index, _)| {
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
            if (p.pos.x - state.position.x).abs() <= 1.1 {
                p.pos.x = state.position.x;
                p.movement_remainder.x = 0.0;
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
                p.speed.x = approach(
                    p.speed.x,
                    direction * 64.0,
                    RUN_ACCEL * p.frame_delta_time,
                );
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
            let exit_pressed = input.jump_pressed || input.dash_pressed || input.crouch_dash_pressed;
            let summit_end = if entity.nodes.is_empty() {
                advance_free_lookout_camera(&mut state, entity, p, map, input);
                false
            } else {
                advance_node_lookout_camera(&mut state, entity, p, input)
            };
            if exit_pressed || summit_end {
                state.phase = 5;
            }
        }
        5 => {
            state.hud_easer = approach(state.hud_easer, 0.0, p.frame_delta_time * 3.0);
            if state.hud_easer <= 0.0 {
                let delta = Vec2::new(p.camera.x - state.cam_start.x, p.camera.y - state.cam_start.y);
                if length(delta) > 600.0 {
                    state.phase = 6;
                    state.timer = 0.0;
                    state.wipe_start = p.camera;
                } else {
                    finish_lookout(p, &mut state);
                }
            }
        }
        6 => {
            let at_summit_top = entity.direction.y != 0.0
                && !entity.nodes.is_empty()
                && state.node as usize >= entity.nodes.len() - 1
                && state.node_percent >= 0.95;
            let duration = if at_summit_top { 1.0 } else { 0.5 };
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
                state.timer += p.frame_delta_time / duration;
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

fn advance_zip_movers(p: &mut PlayerSnapshot, map: &mut Map) {
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
    }
}

fn advance_moving_solids(p: &mut PlayerSnapshot, map: &mut Map) {
    let old_time = p.moving_solid_time;
    let new_time = old_time + p.frame_delta_time;
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

fn advance_cassette_blocks(p: &mut PlayerSnapshot, map: &mut Map) {
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
        p.cassette_blocks[block_index] = state;
    }
}

fn advance_cassette_manager(p: &mut PlayerSnapshot, map: &mut Map) {
    if !p.cassette_manager.initialized || p.cassette_manager.max_beat == 0 {
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
                p.cassette_blocks[block_index] = state;
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
        let entity = &mut map.entities[entity_index];
        if state.visible && state.collidable {
            entity.bounds = Rect::new(state.position.x - 8.0, state.position.y - 6.0, 16.0, 12.0);
        } else {
            park_entity(entity);
        }
        p.spinners[spinner_index] = state;
    }
}

fn advance_post_player_entities(p: &mut PlayerSnapshot, map: &mut Map, input: InputState) {
    advance_zip_movers(p, map);
    advance_bounce_blocks(p, map);
    advance_move_blocks(p, map, input);
    advance_theo_crystals(p, map);
    advance_heart_gems(p);
    advance_rising_lavas(p, map);
    advance_sandwich_lavas(p, map);
    advance_gliders(p, map);
    advance_clouds(p, map);
    advance_seekers(p, map);
    advance_temple_gates(p, map);
    // Player is loaded before room entities. CassetteBlock.Update runs before
    // the manager inserted after the first block, preserving WillToggle now /
    // activation on the following frame.
    advance_cassette_blocks(p, map);
    advance_cassette_manager(p, map);
    advance_spinners(p, map);
}

fn step(
    p: &mut PlayerSnapshot,
    mut input: InputState,
    map: &mut Map,
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
    // VirtualButton.Update runs in MInput before Celeste.Freeze can skip the
    // Scene. It subtracts DeltaTime first, then a new press restores the full
    // buffer; Jump also clears its buffer as soon as the binding is not held.
    p.jump_buffer_timer -= p.frame_delta_time;
    if input.jump_pressed {
        p.jump_buffer_timer = JUMP_BUFFER_TIME;
    } else if !input.jump_held {
        p.jump_buffer_timer = 0.0;
    }
    // Player state callbacks read VirtualButton.Pressed, not the raw edge.
    input.jump_pressed = p.jump_buffer_timer > 0.0;
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
            p.player_on_ground = p.on_ground;
            p.player_on_ground_initialized = true;
            p.dashes = p.dashes.max(1);
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
    advance_moving_solids(p, map);
    if p.transition_timer > 0.0 {
        update_transition(p, map);
        advance_sandwich_lavas(p, map);
        advance_cassette_blocks(p, map);
        advance_cassette_manager(p, map);
        advance_spinners(p, map);
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
            p.explode_launch_boost_timer -= p.frame_delta_time;
        }
    }
    tick_timers(p);
    if p.wall_slide_dir != 0 {
        p.wall_slide_timer = (p.wall_slide_timer - p.frame_delta_time).max(0.0);
    }
    p.wall_slide_dir = 0;
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
    update_climb_hop_wait(p, map);
    prepare_lookout_player(p);

    if p.badeline_boost_active {
        update_badeline_boost(p, map);
        advance_post_player_entities(p, map, input);
        p.on_ground = grounded(p, map);
        return Ok(());
    }

    let was_pickup = p.state == PlayerState::Pickup;
    match p.state {
        PlayerState::Normal => normal_update(p, input, map, was_on_ground),
        PlayerState::Dash => dash_update(p, input, map),
        PlayerState::Climb => climb_update(p, input, map),
        PlayerState::Swim => swim_update(p, input, map),
        PlayerState::Boost => boost_update(p, input, map),
        PlayerState::RedDash => red_dash_update(p, input, map),
        PlayerState::HitSquash => hit_squash_update(p),
        PlayerState::Pickup => pickup_update(p),
        PlayerState::Launch => launch_update(p, input, map),
        PlayerState::DreamDash => dream_dash_update(p),
        PlayerState::SummitLaunch => summit_launch_update(p, map),
        PlayerState::StarFly => star_fly_update(p, input, map),
        PlayerState::Dummy => dummy_update(p, input, map),
        PlayerState::Frozen => {}
        PlayerState::TempleFall => temple_fall_update(p, map),
        PlayerState::ReflectionFall => reflection_fall_update(p, map),
        PlayerState::IntroRespawn => {
            advance_post_player_entities(p, map, input);
            p.on_ground = grounded(p, map);
            return Ok(());
        }
        other => return Err(SimulationError::UnsupportedState(other)),
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
    interact(p, map, input);
    try_begin_lookout(p, map, input);
    advance_lookouts(p, map, input);
    update_strawberry_train(p);
    try_begin_badeline_boost(p, map);
    enforce_level_bounds(p, map);
    // The map loader adds Player before vanilla room entities, so ZipMover's
    // coroutine and Solid carry/push run after Player.Update. A lift speed
    // written by the previous ZipMover update is therefore visible to the
    // player's next action before the platform advances again.
    advance_post_player_entities(p, map, input);
    p.on_ground = grounded(p, map);
    Ok(())
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
        p.wall_speed_retention_timer = (p.wall_speed_retention_timer - p.frame_delta_time).max(0.0);
    }
}

fn normal_update(p: &mut PlayerSnapshot, input: InputState, map: &Map, was_on_ground: bool) {
    let boost = lift_boost(p);
    if boost.y < 0.0 && was_on_ground && !p.on_ground && p.speed.y >= 0.0 {
        p.speed.y = boost.y;
    }
    if !holding_holdable(p) {
        if input.grab_held && p.stamina >= 20.0 && !p.ducking && try_pickup_holdable(p) {
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

    let wall = wall_dir(p, map);
    let facing_dir = if p.facing { 1 } else { -1 };
    if !holding_holdable(p)
        && input.grab_held
        && !p.on_ground
        && !p.ducking
        && p.speed.y >= 0.0
        && p.speed.x.signum() != -(facing_dir as f32)
        && check_stamina(p) >= CLIMB_TIRED_THRESHOLD
        && climb_check(p, map, facing_dir)
    {
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

    let mult = if p.on_ground {
        1.0
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

    let target_max_fall = if holding_slow_fall(p) && p.force_move_x_timer <= 0.0 {
        if input.move_y > 0 {
            120.0
        } else {
            40.0
        }
    } else if input.move_y > 0 && p.speed.y >= MAX_FALL {
        FAST_MAX_FALL
    } else {
        MAX_FALL
    };
    p.max_fall = approach(
        p.max_fall,
        target_max_fall,
        FAST_MAX_ACCEL * p.frame_delta_time,
    );
    let mut fall_target = p.max_fall;
    if !holding_holdable(p) && wall != 0 && input.move_x == wall && p.speed.y >= 0.0 && !p.on_ground
    {
        p.wall_slide_dir = wall;
        fall_target = WALL_SLIDE_START_MAX
            + (MAX_FALL - WALL_SLIDE_START_MAX) * (1.0 - p.wall_slide_timer / WALL_SLIDE_TIME);
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
        } else {
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
            if !holding_holdable(p) && input.grab_held && p.stamina > 0.0 && facing_dir == jump_wall
            {
                climb_jump(p, jump_wall);
            } else if p.dash_attack_timer > 0.0
                && p.dash_dir.x == 0.0
                && p.dash_dir.y == -1.0
            {
                // Player.NormalUpdate still allows SuperWallJump after the
                // Dash coroutine has returned to Normal. DashAttacking lasts
                // 0.3 s, twice the ordinary Dash state's 0.15 s duration.
                super_wall_jump(p, -jump_wall);
            } else {
                p.jump_buffer_timer = 0.0;
                p.speed.x = -(jump_wall as f32) * WALL_JUMP_H;
                p.speed.y = JUMP_SPEED;
                add_lift_boost(p);
                p.auto_jump = false;
                p.dash_attack_timer = 0.0;
                p.wall_slide_timer = WALL_SLIDE_TIME;
                p.wall_boost_timer = 0.0;
                if move_x != 0 {
                    p.force_move_x = -jump_wall;
                    p.force_move_x_timer = 0.16;
                }
                p.var_jump_speed = p.speed.y;
                p.var_jump_timer = VAR_JUMP_TIME;
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
    p.freeze_timer = 0.05;
    if consume_dash {
        p.dashes = p.dashes.saturating_sub(1);
    }
    if !p.on_ground && p.ducking && can_unduck(p, map) {
        p.ducking = false;
    } else if !p.ducking && (p.demo_dashed || input.move_y > 0) {
        p.ducking = true;
    }
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
        && p.stamina >= 20.0
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
    if p.dash_dir == Vec2::default() && input.jump_pressed && p.jump_grace_timer > 0.0 {
        super_jump(p);
        return;
    }
    p.state_timer = (p.state_timer - p.frame_delta_time).max(0.0);
    if (p.state_timer - DASH_TIME).abs() <= p.frame_delta_time * 0.5 {
        p.dash_dir = p.last_aim;
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

fn enter_normal(p: &mut PlayerSnapshot) {
    p.state = PlayerState::Normal;
    p.max_fall = MAX_FALL;
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
        p.stamina = (p.stamina - CLIMB_JUMP_COST).max(0.0);
    }
    if p.move_x == 0 {
        p.wall_boost_dir = -wall;
        p.wall_boost_timer = 0.2;
    }
    p.var_jump_speed = p.speed.y;
    p.var_jump_timer = VAR_JUMP_TIME;
}

fn climb_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    let wall = if p.facing { 1 } else { -1 };
    // Player.ClimbUpdate checks jump and dash before letting go or checking
    // whether the one-pixel wall contact still exists. Ceiling pops rely on
    // that ordering for the first Climb frame just below a wall's bottom edge.
    if input.jump_pressed && (!p.ducking || can_unduck(p, map)) {
        enter_normal(p);
        if p.move_x == -wall {
            p.jump_buffer_timer = 0.0;
            p.jump_grace_timer = 0.0;
            p.auto_jump = false;
            p.dash_attack_timer = 0.0;
            p.wall_slide_timer = WALL_SLIDE_TIME;
            p.wall_boost_timer = 0.0;
            p.ducking = false;
            p.speed = Vec2::new(-(wall as f32) * WALL_JUMP_H, JUMP_SPEED);
            add_lift_boost(p);
            p.var_jump_speed = p.speed.y;
            p.var_jump_timer = VAR_JUMP_TIME;
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
        enter_normal(p);
        return;
    }
    if !touching_wall(p, map, wall) {
        if p.speed.y < 0.0 {
            climb_hop(p, map, wall);
        }
        enter_normal(p);
        return;
    }
    let slipping = slip_check(p, map, 0.0);
    let target = if p.climb_no_move_timer > 0.0 {
        if slipping { CLIMB_SLIP_SPEED } else { 0.0 }
    } else {
        match input.move_y {
            -1 if slipping => {
                climb_hop(p, map, wall);
                enter_normal(p);
                return;
            }
            -1 => CLIMB_UP_SPEED,
            1 => CLIMB_DOWN_SPEED,
            _ => {
                if slipping {
                    CLIMB_SLIP_SPEED
                } else {
                    0.0
                }
            }
        }
    };
    p.speed.y = approach(p.speed.y, target, CLIMB_ACCEL * p.frame_delta_time);
    p.speed.x = 0.0;
    if p.climb_no_move_timer <= 0.0 {
        let cost = if target < 0.0 {
            CLIMB_UP_COST
        } else if target == 0.0 {
            CLIMB_STILL_COST
        } else {
            0.0
        };
        p.stamina = (p.stamina - cost * p.frame_delta_time).max(0.0);
    }
    if p.stamina <= 0.0 {
        enter_normal(p);
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
    p.speed.x = approach(
        p.speed.x,
        horizontal_max * x,
        horizontal_accel * p.frame_delta_time,
    );

    if y == 0.0 && swim_rise_check(p, map) {
        p.speed.y = approach(p.speed.y, SWIM_MAX_RISE, SWIM_ACCEL * p.frame_delta_time);
    } else if y >= 0.0 || underwater {
        let vertical_accel = if p.speed.y.abs() > SWIM_MAX && p.speed.y.signum() == y.signum() {
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

fn boost_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
    p.speed = Vec2::default();
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

fn red_dash_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
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

fn launch_update(p: &mut PlayerSnapshot, input: InputState, map: &Map) {
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
    // tween after its freeze ends when Grab remains held.
    if input.grab_held && p.stamina >= 20.0 && !p.ducking && try_pickup_holdable(p) {
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

fn summit_launch_update(p: &mut PlayerSnapshot, map: &Map) {
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
        let gravity_mult =
            if p.speed.y.abs() < HALF_GRAV_THRESHOLD && (input.jump_held || p.auto_jump) {
                0.5
            } else {
                1.0
            };
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

fn climb_check(p: &PlayerSnapshot, map: &Map, dir: i8) -> bool {
    climb_bounds_check(p, map, dir)
        && map.solid_at(current_player_rect(
            p,
            p.pos.x + dir as f32 * CLIMB_CHECK_DIST,
            p.pos.y,
        ))
}

fn wall_jump_check(p: &PlayerSnapshot, map: &Map, dir: i8) -> bool {
    climb_bounds_check(p, map, dir)
        && map.solid_at(current_player_rect(
            p,
            p.pos.x + dir as f32 * WALL_JUMP_CHECK_DIST,
            p.pos.y,
        ))
}

fn slip_check(p: &PlayerSnapshot, map: &Map, add_y: f32) -> bool {
    let rect = current_player_rect(p, p.pos.x, p.pos.y);
    let x = if p.facing { rect.right() } else { rect.x - 1.0 };
    let lower_y = rect.y + 4.0 + add_y;
    !map.solid_at(Rect::new(x, lower_y, 1.0, 1.0))
        && !map.solid_at(Rect::new(x, lower_y - 4.0 + add_y, 1.0, 1.0))
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
                if matches!(p.state, PlayerState::Dash | PlayerState::RedDash)
                    && p.speed.y == 0.0
                    && p.speed.x != 0.0
                {
                    for correction in 1..=DASH_CORNER_CORRECTION {
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

fn interact(p: &mut PlayerSnapshot, map: &Map, input: InputState) {
    if let Some(from_y) = p.pending_bounce_from_y.take() {
        // Backward compatibility for portable snapshots produced before
        // FireBall callbacks were aligned to the source's same-frame order.
        bounce(p, map, from_y);
        return;
    }
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
    let mut heart_index = 0usize;
    let mut rising_lava_index = 0usize;
    let mut sandwich_lava_index = 0usize;
    let mut bumper_index = 0usize;
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
        // Seeker PlayerColliders run from the dynamic Seeker update after
        // Player.Update, using its live StateMachine collider selection.
        if entity.kind == EntityKind::Seeker {
            continue;
        }
        let player_box = if matches!(
            entity.kind,
            EntityKind::Spikes
                | EntityKind::FlyFeather
                | EntityKind::Bumper
                | EntityKind::Spring
                | EntityKind::IceBall
                | EntityKind::RisingLava
                | EntityKind::SandwichLava
                | EntityKind::CrystalStaticSpinner
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
            EntityKind::Spikes if spike_is_lethal(p, entity.direction, entity.bounds) => {
                p.dead = true;
                p.speed = Vec2::default();
                p.death_freeze_pending = true;
                p.respawn_frames = 95;
                return;
            }
            EntityKind::RisingLava
            | EntityKind::SandwichLava
            | EntityKind::CrystalStaticSpinner => {
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
                p.state_timer = 0.25 + p.frame_delta_time * 2.0;
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
                    explode_launch(p, input, target, false, true);
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

fn update_badeline_boost(p: &mut PlayerSnapshot, map: &Map) {
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
    // Player.PointBounce uses the regular actor center without an artificial
    // vertical-angle clamp. The horizontal multiplier and minimum are applied
    // after SafeNormalize, which matters for the nearly level Seeker contact.
    p.speed = scale(normalize(Vec2::new(center.x - from.x, center.y - from.y)), 200.0);
    p.speed.x *= 1.2;
    if p.speed.x.abs() < 120.0 {
        p.speed.x = if p.speed.x == 0.0 {
            if p.facing { -120.0 } else { 120.0 }
        } else {
            p.speed.x.signum() * 120.0
        };
    }
}

fn enforce_level_bounds(p: &mut PlayerSnapshot, map: &mut Map) {
    if p.dead || p.state == PlayerState::DreamDash || !player_in_control(p.state) {
        return;
    }
    let bounds = p.current_room_bounds.unwrap_or(map.bounds);
    let mut collider = current_player_rect(p, p.pos.x, p.pos.y);
    if collider.x < bounds.x {
        let center = Vec2::new(p.pos.x, collider.y + collider.height * 0.5);
        if let Some(next) = transition_room_at(map, p, Vec2::new(center.x - 8.0, center.y)) {
            begin_transition(p, map, next, Vec2::new(-1.0, 0.0));
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
            begin_transition(p, map, next, Vec2::new(1.0, 0.0));
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
            begin_transition(p, map, next, Vec2::new(0.0, -1.0));
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
            begin_transition(p, map, next, Vec2::new(0.0, 1.0));
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

fn load_transition_room(p: &mut PlayerSnapshot, map: &mut Map, next: Rect) {
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
    p.cassette_blocks.clear();
    p.spinners.clear();
    initialize_cassette_blocks(p, map);
    initialize_spinners(p, map);
}

fn begin_transition(p: &mut PlayerSnapshot, map: &mut Map, next: Rect, direction: Vec2) {
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
    load_transition_room(p, map, next);
    // TransitionRoutine updates cameraAt after yielding, then resumes once
    // more to observe cameraAt == 1 and run OnTransition. Preserve that final
    // coroutine-resume frame in addition to the 0.65-second camera duration.
    p.transition_timer = TRANSITION_TIME + p.frame_delta_time;
    p.on_ground = false;
}

fn update_transition(p: &mut PlayerSnapshot, map: &mut Map) {
    let max_move = TRANSITION_MOVE_SPEED * p.frame_delta_time;
    p.pos.x = approach(p.pos.x, p.transition_target.x, max_move);
    p.pos.y = approach(p.pos.y, p.transition_target.y, max_move);
    p.transition_timer = (p.transition_timer - p.frame_delta_time).max(0.0);
    p.on_ground = false;
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
        p.dashes = p.dashes.max(1);
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
    p.wind.x = approach(p.wind.x, p.wind_target.x, WIND_ACCEL * p.frame_delta_time);
    p.wind.y = approach(p.wind.y, p.wind_target.y, WIND_ACCEL * p.frame_delta_time);
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

    let mut move_x = p.wind.x * WIND_MOVE_MULT * p.frame_delta_time;
    if move_x != 0.0 && p.state != PlayerState::Climb {
        let shield_x = p.pos.x - move_x.signum() * WIND_WALL_DISTANCE;
        if !map.solid_at(current_player_rect(p, shield_x, p.pos.y)) {
            if p.ducking && p.on_ground {
                move_x = 0.0;
            }
            move_axis_amount(p, map, true, move_x);
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
    fn downward_solid_push_uses_player_squish_target_to_clip_through_jump_thru() {
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let inputs: Vec<_> = (0..260).map(|frame| InputState {
            move_x: if frame < 10 { 1 } else if (20..30).contains(&frame) { -1 } else { 0 },
            ..InputState::default()
        }).collect();
        let trace = simulate_trace(PlayerSnapshot {
            pos: Vec2::new(652.0, 400.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        }, &inputs, &map, inputs.len() as u32).unwrap();
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
        let map = Map {
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
        let map = bounce_block_spikes_map();
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
        assert_ne!(
            reenabled.attached_spike_position,
            Vec2::new(32.0, 157.0)
        );
        assert_ne!(reenabled.position, Vec2::new(32.0, 160.0));
    }

    #[test]
    fn core_block_candidate_clears_source_body_before_reform_blocked_check() {
        let map = Map {
            bounds: Rect::new(0.0, 0.0, 960.0, 544.0),
            solids: vec![Rect::new(0.0, 496.0, 960.0, 48.0)],
            entities: vec![
                crate::Entity {
                    kind: EntityKind::BounceBlock,
                    bounds: Rect::new(704.0, 440.0, 64.0, 16.0),
                    direction: Vec2::default(),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "bounceBlock".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::Spikes,
                    bounds: Rect::new(768.0, 440.0, 3.0, 16.0),
                    direction: Vec2::new(1.0, 0.0),
                    shielded: false,
                    single_use: false,
                    nodes: vec![],
                    name: "spikesRight".to_owned(),
                },
                crate::Entity {
                    kind: EntityKind::JumpThru,
                    bounds: Rect::new(704.0, 456.0, 64.0, 8.0),
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
            pos: Vec2::new(736.0, 440.0),
            ..PlayerSnapshot::default()
        };
        let inputs: Vec<_> = (0..280)
            .map(|frame| InputState {
                move_x: if frame < 80 { -1 } else { 0 },
                ..InputState::default()
            })
            .collect();
        let trace = simulate_trace(p, &inputs, &map, inputs.len() as u32).unwrap();
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
        assert!(trace.states[body].pos.x < 700.0);
        assert!(spike > body);
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
        let map = Map {
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
                move_y: if matches!(frame, 23 | 65 | 101) { 1 } else { -1 },
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
        assert_eq!(pickup_starts, vec![(1, Some(0)), (25, Some(1)), (67, Some(0))]);
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
        let map = glider_map();
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = lava_map(EntityKind::SandwichLava, 100.0);
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
        let map = lava_map(EntityKind::RisingLava, 0.0);
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
        for (hold_down, expected_speed) in [(false, Vec2::new(260.0, -105.0)), (true, Vec2::new(325.0, -52.5))] {
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
            let trace = simulate_trace(grounded_player(), &inputs, &floor_map(), inputs.len() as u32).unwrap();
            let launched = trace.states.iter().find(|state| state.speed == expected_speed);
            assert!(launched.is_some(), "hold_down={hold_down}: {:#?}", trace.states);
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
    fn downward_dash_corner_correction_moves_left_around_a_one_pixel_floor_overlap() {
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
    fn upward_motion_flush_with_directional_spikes_applies_gravity_on_frame_one() {
        let map = Map {
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
    fn half_stamina_climbing_chains_wallboost_into_close_wall_climb_jump() {
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        assert!(!climb_check(&player, &map, 1));
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
    fn climb_jump_keeps_priority_over_climbhop_on_the_lost_wall_frame() {
        let map = Map {
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
    fn climbing_down_does_not_pay_the_stationary_stamina_cost() {
        let map = Map {
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
    fn cornerboost_climb_jump_stores_jump_boost_before_clearing_wall_top() {
        let map = Map {
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
        let map = Map {
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
        assert!(!climb_check(&player, &map, 1));
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
        let map = Map {
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
            for (move_y, before_dash_speed) in [
                (-1, Vec2::new(123.0, -80.0)),
                (1, Vec2::new(-123.0, 80.0)),
            ] {
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
    fn dream_smuggle_keeps_theo_through_pickup_and_lingering_attack_entry() {
        let map = dream_smuggle_map();
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
        let map = dream_smuggle_map();
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
        let map = Map {
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
                move_x: if (43..54).contains(&frame) || frame >= 85 { -1 } else { 1 },
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
        assert_eq!(theo_pickup_rect(before.theo_crystals[0].position).right(), 368.0);
        assert_eq!(current_player_rect(before, before.pos.x, before.pos.y).x, 367.0);

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
        let map = dream_smuggle_map();
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
        assert_eq!(completed, 41);
        assert_eq!(trace.states[completed].pos.y, -5.0);
        assert_eq!(trace.states[completed].dashes, 1);
        assert_eq!(trace.states[completed].stamina, 110.0);
        assert_eq!(trace.states[completed].wall_slide_timer, WALL_SLIDE_TIME);
        assert_eq!(trace.states[completed].jump_grace_timer, 0.0);
    }

    #[test]
    fn bubsdrop_wall_jump_misses_upper_jumpthru_and_restores_old_room_spawn_set() {
        let lower = Rect::new(0.0, 0.0, 320.0, 184.0);
        let upper = Rect::new(0.0, -184.0, 320.0, 184.0);
        let map = Map {
            bounds: lower,
            transition_rooms: vec![upper],
            transition_runtime: vec![
                crate::RoomRuntime {
                    bounds: lower,
                    spawns: vec![Vec2::new(24.0, 32.0), Vec2::new(280.0, 32.0)],
                    solids: vec![],
                    entities: vec![],
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
        let map = Map {
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
        let map = Map {
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
            // after Player.Update. CassetteBlock.Update does not actually
            // clear collision until the following entity phase, so retained
            // speed can only return on the third player update.
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
            },
            ..PlayerSnapshot::default()
        };
        let trace = simulate_trace(p, &[InputState::default(); 3], &cassette_map(), 3).unwrap();
        assert_eq!(trace.states[1].speed.x, 0.0);
        assert!(trace.states[1].wall_speed_retention_timer > 0.05);
        assert!(!trace.states[1].cassette_blocks[0].activated);
        assert!(trace.states[1].cassette_blocks[0].collidable);
        assert!(trace.states[2].wall_speed_retention_timer > 0.0);
        assert!(!trace.states[2].cassette_blocks[0].collidable);
        assert_eq!(trace.states[3].wall_speed_retention_timer, 0.0);
        assert!(trace.states[3].speed.x > 90.0);
    }

    #[test]
    fn disappearing_cassette_cornerboost_fixture_times_hit_clear_and_refund() {
        // This is the generated candidate's timing in a compact map: input
        // 28 hits the initially-active index 1 wall, manager activation then
        // disables it, input 29's entity phase clears it, and the following
        // Player.Update restores the retained 90-speed run.
        let map = Map {
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
        for input in &mut inputs[23..] {
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
        assert_eq!(trace.states[29].pos, Vec2::new(124.0, 496.0));
        assert_eq!(trace.states[29].speed.x, 0.0);
        assert!(!trace.states[29].cassette_blocks[1].collidable);
        assert_eq!(trace.states[30].pos, Vec2::new(126.0, 496.0));
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
        assert!((result.current_lift_speed.y + 60.0).abs() < 0.001);
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
        assert_eq!(result.speed.y, JUMP_SPEED);
        assert!(result.cassette_blocks[0].collidable);
        assert_eq!(result.cassette_blocks[0].position.y, 101.0);
        assert_eq!(result.pos.y, 101.0);
    }

    #[test]
    fn cassoosted_fuper_fixture_aligns_first_starfly_jump_with_tempo_three_reform() {
        // Mirror the candidate MapPart rather than pre-seeding StarFly.  The
        // fresh custom manager skips its first AdvanceMusic call; with tempo
        // three, beat 8 writes Activated on input 27 and CassetteBlock.Update
        // reforms during input 28, after this frame's Player.Jump.
        let map = Map {
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
                    && state.speed.y == JUMP_SPEED
            })
            .expect("first controllable StarFly frame should produce a Feather Super");
        let reform = trace
            .states
            .iter()
            .position(|state| {
                state.cassette_blocks[0].collidable && state.cassette_blocks[0].position.y == 493.0
            })
            .expect("tempo-three cassette should reform");
        assert_eq!(fuper, 29);
        assert_eq!(reform, fuper);
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
            },
            ..PlayerSnapshot::default()
        };
        begin_transition(&mut p, &mut map, next, Vec2::new(0.0, -1.0));

        // LoadLevel's OnLevelStart is silent: the new index-0 block begins
        // inactive at +2 px while the destination block matching the retained
        // current index begins at its source position. The manager has not
        // advanced yet.
        assert_eq!(p.cassette_blocks[0].position.y, -46.0);
        assert!(!p.cassette_blocks[0].collidable);
        assert_eq!(p.cassette_blocks[1].position.y, -48.0);
        assert!(p.cassette_blocks[1].collidable);

        step(&mut p, InputState::default(), &mut map).unwrap();
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
        let map = ice_ball_map();
        interact(&mut bounced, &map, InputState::default());
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
        let map = crate::mechanics_playground();
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
            let map = bounce_actor_map(kind.clone());
            interact(&mut p, &map, InputState::default());
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
        let map = Map {
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
        let map = Map {
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
            ..Map::default()
        }
    }

    #[test]
    fn close_behind_player_gate_uses_target_position_fallback_to_clip_theo() {
        let map = temple_gate_map(1.0);
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
        let map = cloud_map(false);
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
        let map = ice_ball_map();
        interact(&mut bounced, &map, InputState::default());
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
        let map = Map {
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
        let trace = simulate_trace(player, &inputs, &lookout_map(vec![], false, false), 150)
            .unwrap();

        // `Interact` starts the entity coroutine; `LookRoutine` assigns Dummy
        // on its following update, matching the real Lookout lifecycle.
        assert_eq!(trace.states[1].state, PlayerState::Normal);
        assert!(trace.states[1].lookouts[0].interacting);
        assert_eq!(trace.states[2].state, PlayerState::Dummy);
        assert!(trace.states[70].lookouts[0].phase >= 4);
        assert!(trace.states[110].camera.x > 0.0);
        assert!(!trace.states[150].lookouts[0].interacting);
        assert_eq!(trace.states[150].state, PlayerState::Normal);
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
        let trace = simulate_trace(player, &inputs, &lookout_map(vec![], false, true), 260)
            .unwrap();

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
        assert!(trace.states.iter().any(|state| state.state == PlayerState::Boost));
        assert!(trace.states.iter().any(|state| {
            state.state == PlayerState::Normal && state.lookouts[0].interacting
        }));
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
    fn bino_extensions_follow_nodes_and_run_long_distance_exit_wipe() {
        let map = lookout_map(vec![Vec2::new(960.0, 90.0)], true, false);
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
        let inputs = [InputState {
            move_y: -1,
            ..InputState::default()
        }; 330];
        let trace = simulate_trace(player, &inputs, &map, 330).unwrap();

        assert!(trace.states.iter().any(|state| state.camera.x > 600.0));
        assert!(trace.states.iter().any(|state| state.lookouts[0].phase == 6));
        let exit = trace
            .states
            .iter()
            .find(|state| !state.lookouts[0].interacting)
            .expect("long-distance wipe completes");
        assert!((exit.camera.x - 32.0).abs() < 0.01);
        assert!(!trace.states[330].lookouts[0].interacting);
        assert_eq!(trace.states[330].state, PlayerState::Normal);
    }

    #[test]
    fn cloud_hyper_bunnyhop_fixture_leaves_the_platform_side_before_apex_landing() {
        let map = Map {
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
        assert_eq!(trace.states[35].pos.y, 414.0);
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
        let map = Map {
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
        assert!(trace.states[51].wall_speed_retention_timer > 0.05);
        assert!(trace.states[51].wall_speed_retained > 300.0);
        assert!(trace.states[55].speed.x > 300.0);
        assert!(trace.states[59].speed.x < trace.states[58].speed.x);
        assert_eq!(trace.states[51].move_blocks[0].position.y, 440.0);
    }
}
