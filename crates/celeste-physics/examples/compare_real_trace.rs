use std::{env, fs, process::ExitCode};

use celeste_physics::{
    decode_map_room, simulate_trace, InputState, PlayerSnapshot, PlayerState, Vec2,
};
use serde::Deserialize;
use serde_json::Value;

#[cfg(test)]
use celeste_physics::{simulate, Map, Rect};

#[derive(Deserialize)]
struct TraceFile {
    inputs: Vec<InputState>,
    states: Vec<PortableSnapshot>,
}

#[derive(Deserialize)]
struct PortableSnapshot {
    pos: [f32; 2],
    speed: [f32; 2],
    state: StateValue,
    facing: FacingValue,
    dashes: u8,
    stamina: f32,
    on_ground: bool,
    ducking: bool,
    #[serde(default)]
    can_dream_dash: bool,
    #[serde(default)]
    dead: bool,
    #[serde(default)]
    freeze_timer: f32,
    #[serde(default, rename = "_everest_fields")]
    fields: serde_json::Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StateValue {
    Id(u8),
    Name(String),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FacingValue {
    Bool(bool),
    Name(String),
}

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let (Some(trace_path), Some(map_path)) = (args.next(), args.next()) else {
        eprintln!("usage: compare_real_trace <trace.json> <map.bin> [room]");
        return ExitCode::from(2);
    };
    let room = args.next();
    let trace: TraceFile = match fs::read(&trace_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    {
        Some(trace) => trace,
        None => {
            eprintln!("failed to read trace {:?}", trace_path);
            return ExitCode::FAILURE;
        }
    };
    let map = match fs::read(&map_path).ok().and_then(|bytes| {
        decode_map_room(&bytes, room.as_deref().and_then(|room| room.to_str())).ok()
    }) {
        Some(map) => map,
        None => {
            eprintln!("failed to read map {:?}", map_path);
            return ExitCode::FAILURE;
        }
    };
    let initial = to_snapshot(&trace.states[0]);
    let mut inputs = trace.inputs.clone();
    // State 0 precedes the first scripted Player.Update. Each later capture
    // contains the Engine.DeltaTime consumed by its matching input frame. New
    // Collector traces preserve its f32 bits; old decimal-only traces remain
    // readable but cannot reproduce a movementCounter rounding boundary.
    for (input, state) in inputs.iter_mut().zip(trace.states.iter().skip(1)) {
        input.frame_delta_time_bits = captured_delta_time_bits(&state.fields);
    }
    let simulated = match simulate_trace(initial, &inputs, &map, inputs.len() as u32) {
        Ok(result) => result.states,
        Err(error) => {
            eprintln!("simulation failed: {error}");
            return ExitCode::FAILURE;
        }
    };

    let mut max_pos = 0.0f32;
    let mut max_speed = 0.0f32;
    let mut first_mismatch = None;
    for (frame, (actual, expected)) in simulated.iter().zip(&trace.states).enumerate() {
        let pos_error = (actual.pos.x - expected.pos[0])
            .abs()
            .max((actual.pos.y - expected.pos[1]).abs());
        let speed_error = (actual.speed.x - expected.speed[0])
            .abs()
            .max((actual.speed.y - expected.speed[1]).abs());
        max_pos = max_pos.max(pos_error);
        max_speed = max_speed.max(speed_error);
        let expected_state = state_from_value(&expected.state);
        let expected_facing = match &expected.facing {
            FacingValue::Bool(value) => *value,
            FacingValue::Name(value) => value == "Right",
        };
        if first_mismatch.is_none()
            && (pos_error > 0.01
                || speed_error > 0.01
                || actual.state != expected_state
                || actual.facing != expected_facing
                || actual.dashes != expected.dashes
                || (actual.stamina - expected.stamina).abs() > 0.01
                || actual.on_ground != expected.on_ground
                || actual.ducking != expected.ducking
                || actual.dead != expected.dead)
        {
            first_mismatch = Some((frame, pos_error, speed_error, actual, expected));
        }
    }

    println!(
        "frames={} max_pos_error={max_pos:.6} max_speed_error={max_speed:.6}",
        simulated.len()
    );
    if let Some((frame, pos, speed, actual, expected)) = first_mismatch {
        let expected_facing = match &expected.facing {
            FacingValue::Bool(value) => *value,
            FacingValue::Name(value) => value == "Right",
        };
        println!(
            "first_mismatch frame={frame} pos_error={pos:.6} speed_error={speed:.6}\n  rust pos=({:.3},{:.3}) speed=({:.3},{:.3}) state={:?} dead={}\n  game pos=({:.3},{:.3}) speed=({:.3},{:.3}) state={:?} dead={}",
            actual.pos.x,
            actual.pos.y,
            actual.speed.x,
            actual.speed.y,
            actual.state,
            actual.dead,
            expected.pos[0],
            expected.pos[1],
            expected.speed[0],
            expected.speed[1],
            state_from_value(&expected.state),
            expected.dead
        );
        println!(
            "  rust facing={} dashes={} stamina={:.3} ground={} duck={} | game facing={} dashes={} stamina={:.3} ground={} duck={}",
            actual.facing,
            actual.dashes,
            actual.stamina,
            actual.on_ground,
            actual.ducking,
            expected_facing,
            expected.dashes,
            expected.stamina,
            expected.on_ground,
            expected.ducking,
        );
        let nearby: Vec<_> = map
            .solids
            .iter()
            .filter(|solid| {
                solid.x <= expected.pos[0] + 8.0
                    && solid.x + solid.width >= expected.pos[0] - 8.0
                    && solid.y <= expected.pos[1] + 24.0
                    && solid.y + solid.height >= expected.pos[1] - 24.0
            })
            .collect();
        println!("nearby_solids={nearby:?}");
        println!("entities={:?}", map.entities);
        let start = frame.saturating_sub(3);
        let end = (frame + 3).min(simulated.len() - 1);
        for (index, rust) in simulated.iter().enumerate().take(end + 1).skip(start) {
            let game = &trace.states[index];
            let zip = rust.zip_movers.first();
            println!(
                "f{index}: rust p=({:.3},{:.3}) v=({:.3},{:.3}) rem=({:.3},{:.3}) zip={:?} | game p=({:.3},{:.3}) v=({:.3},{:.3})",
                rust.pos.x,
                rust.pos.y,
                rust.speed.x,
                rust.speed.y,
                rust.movement_remainder.x,
                rust.movement_remainder.y,
                zip.map(|zip| (
                    zip.phase,
                    zip.at,
                    zip.position,
                    zip.remainder,
                    zip.lift_speed
                )),
                game.pos[0],
                game.pos[1],
                game.speed[0],
                game.speed[1]
            );
        }
        ExitCode::FAILURE
    } else {
        println!("trace matches within 0.01");
        ExitCode::SUCCESS
    }
}

fn to_snapshot(value: &PortableSnapshot) -> PlayerSnapshot {
    let mut snapshot = PlayerSnapshot {
        pos: Vec2::new(value.pos[0], value.pos[1]),
        speed: Vec2::new(value.speed[0], value.speed[1]),
        state: state_from_value(&value.state),
        facing: match &value.facing {
            FacingValue::Bool(value) => *value,
            FacingValue::Name(value) => value == "Right",
        },
        dashes: value.dashes,
        stamina: value.stamina,
        on_ground: value.on_ground,
        ducking: value.ducking,
        can_dream_dash: value.can_dream_dash,
        dead: value.dead,
        freeze_timer: value.freeze_timer,
        ..PlayerSnapshot::default()
    };
    snapshot.dash_dir = vector_field(&value.fields, "DashDir");
    snapshot.time_rate = value
        .fields
        .get("engineTimeRate")
        .and_then(Value::as_f64)
        .map_or(1.0, |value| value as f32);
    snapshot.before_dash_speed = vector_field(&value.fields, "beforeDashSpeed");
    snapshot.demo_dashed = bool_field(&value.fields, "demoDashed");
    snapshot.movement_remainder = vector_field(&value.fields, "movementCounter");
    snapshot.wind = vector_field(&value.fields, "levelWind");
    snapshot.wind_target = snapshot.wind;
    snapshot.no_wind_timer = float_field(&value.fields, "noWindTimer");
    snapshot.dash_attack_timer = float_field(&value.fields, "dashAttackTimer");
    snapshot.dash_cooldown_timer = float_field(&value.fields, "dashCooldownTimer");
    snapshot.dash_refill_cooldown_timer = float_field(&value.fields, "dashRefillCooldownTimer");
    snapshot.jump_grace_timer = float_field(&value.fields, "jumpGraceTimer");
    snapshot.auto_jump = bool_field(&value.fields, "AutoJump");
    snapshot.auto_jump_timer = float_field(&value.fields, "AutoJumpTimer");
    snapshot.var_jump_timer = float_field(&value.fields, "varJumpTimer");
    snapshot.var_jump_speed = float_field(&value.fields, "varJumpSpeed");
    snapshot.move_x = int_field(&value.fields, "moveX") as i8;
    snapshot.force_move_x = int_field(&value.fields, "forceMoveX") as i8;
    snapshot.force_move_x_timer = float_field(&value.fields, "forceMoveXTimer");
    snapshot.wall_speed_retention_timer = float_field(&value.fields, "wallSpeedRetentionTimer");
    snapshot.wall_speed_retained = float_field(&value.fields, "wallSpeedRetained");
    snapshot.wall_boost_timer = float_field(&value.fields, "wallBoostTimer");
    snapshot.wall_boost_dir = int_field(&value.fields, "wallBoostDir") as i8;
    snapshot.wall_slide_timer = float_field(&value.fields, "wallSlideTimer");
    snapshot.wall_slide_dir = int_field(&value.fields, "wallSlideDir") as i8;
    // A trace can begin immediately after StateMachine enters Climb. ClimbBegin
    // sets this to 0.1 before the first ClimbUpdate subtracts DeltaTime; losing
    // it charges the 10/s stationary-climb stamina cost one frame too early.
    snapshot.climb_no_move_timer = float_field(&value.fields, "climbNoMoveTimer");
    snapshot.hop_wait_x = int_field(&value.fields, "hopWaitX") as i8;
    snapshot.hop_wait_x_speed = float_field(&value.fields, "hopWaitXSpeed");
    snapshot.max_fall = float_field(&value.fields, "maxFall");
    snapshot.launch_approach_x = value
        .fields
        .get("launchApproachX")
        .and_then(Value::as_f64)
        .map(|value| value as f32);
    snapshot.summit_launch_target_x = float_field(&value.fields, "summitLaunchTargetX");
    snapshot.summit_launch_particle_timer = float_field(&value.fields, "summitLaunchParticleTimer");
    snapshot.explode_launch_boost_timer = float_field(&value.fields, "explodeLaunchBoostTimer");
    snapshot.explode_launch_boost_speed = float_field(&value.fields, "explodeLaunchBoostSpeed");
    snapshot.star_fly_timer = float_field(&value.fields, "starFlyTimer");
    snapshot.star_fly_transforming = bool_field(&value.fields, "starFlyTransforming");
    snapshot.star_fly_speed_lerp = float_field(&value.fields, "starFlySpeedLerp");
    snapshot.star_fly_last_dir = vector_field(&value.fields, "starFlyLastDir");
    snapshot.strawberry_collect_index = int_field(&value.fields, "StrawberryCollectIndex") as u16;
    snapshot.strawberry_collect_reset_timer =
        float_field(&value.fields, "StrawberryCollectResetTimer");
    snapshot
}

fn float_field(fields: &serde_json::Map<String, Value>, name: &str) -> f32 {
    fields.get(name).and_then(Value::as_f64).unwrap_or(0.0) as f32
}

fn captured_delta_time_bits(fields: &serde_json::Map<String, Value>) -> Option<u32> {
    fields
        .get("engineDeltaTimeBits")
        .and_then(Value::as_i64)
        .and_then(|bits| u32::try_from(bits).ok())
        // Pre-bit captures used a JSON decimal. It is useful for broad
        // comparison but can be one or more ULPs away from Engine.DeltaTime.
        .or_else(|| {
            fields
                .get("engineDeltaTime")
                .and_then(Value::as_f64)
                .map(|delta| (delta as f32).to_bits())
        })
}

fn int_field(fields: &serde_json::Map<String, Value>, name: &str) -> i64 {
    fields.get(name).and_then(Value::as_i64).unwrap_or(0)
}

fn bool_field(fields: &serde_json::Map<String, Value>, name: &str) -> bool {
    fields.get(name).and_then(Value::as_bool).unwrap_or(false)
}

fn vector_field(fields: &serde_json::Map<String, Value>, name: &str) -> Vec2 {
    let Some(values) = fields.get(name).and_then(Value::as_array) else {
        return Vec2::default();
    };
    Vec2::new(
        values.first().and_then(Value::as_f64).unwrap_or(0.0) as f32,
        values.get(1).and_then(Value::as_f64).unwrap_or(0.0) as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captured_climb_entry_timer_keeps_the_first_stationary_frame_stamina_free() {
        let snapshot = to_snapshot(&PortableSnapshot {
            pos: [12.0, 100.0],
            speed: [0.0, 0.0],
            state: StateValue::Id(1),
            facing: FacingValue::Bool(false),
            dashes: 1,
            stamina: 110.0,
            on_ground: false,
            ducking: false,
            can_dream_dash: false,
            dead: false,
            freeze_timer: 0.0,
            fields: serde_json::Map::from_iter([("climbNoMoveTimer".to_owned(), Value::from(0.1))]),
        });
        let map = Map {
            solids: vec![Rect::new(0.0, 80.0, 8.0, 100.0)],
            ..Map::default()
        };
        let after = simulate(
            snapshot,
            &[InputState {
                grab_held: true,
                ..InputState::default()
            }],
            &map,
            1,
        )
        .unwrap();

        assert!((after.climb_no_move_timer - (0.1 - 1.0 / 60.0)).abs() < 0.0001);
        assert!((after.stamina - 110.0).abs() < 0.0001);
    }

    #[test]
    fn captured_delta_time_bits_take_priority_over_decimal_fallback() {
        let exact = 0.016_666_668_f32.to_bits();
        let fields = serde_json::Map::from_iter([
            ("engineDeltaTimeBits".to_owned(), Value::from(exact)),
            ("engineDeltaTime".to_owned(), Value::from(0.02)),
        ]);
        assert_eq!(captured_delta_time_bits(&fields), Some(exact));

        let legacy =
            serde_json::Map::from_iter([("engineDeltaTime".to_owned(), Value::from(0.02))]);
        assert_eq!(captured_delta_time_bits(&legacy), Some(0.02_f32.to_bits()));
    }
}

fn state_from_value(value: &StateValue) -> PlayerState {
    let id = match value {
        StateValue::Id(id) => *id,
        StateValue::Name(name) => match name.as_str() {
            "Normal" => 0,
            "Climb" => 1,
            "Dash" => 2,
            other => panic!("unsupported state name {other}"),
        },
    };
    match id {
        0 => PlayerState::Normal,
        1 => PlayerState::Climb,
        2 => PlayerState::Dash,
        3 => PlayerState::Swim,
        4 => PlayerState::Boost,
        5 => PlayerState::RedDash,
        6 => PlayerState::HitSquash,
        7 => PlayerState::Launch,
        8 => PlayerState::Pickup,
        9 => PlayerState::DreamDash,
        10 => PlayerState::SummitLaunch,
        11 => PlayerState::Dummy,
        12 => PlayerState::IntroWalk,
        13 => PlayerState::IntroJump,
        14 => PlayerState::IntroRespawn,
        15 => PlayerState::IntroWakeUp,
        16 => PlayerState::BirdDashTutorial,
        17 => PlayerState::Frozen,
        18 => PlayerState::ReflectionFall,
        19 => PlayerState::StarFly,
        20 => PlayerState::TempleFall,
        21 => PlayerState::CassetteFly,
        22 => PlayerState::Attract,
        23 => PlayerState::IntroMoonJump,
        24 => PlayerState::FlingBird,
        25 => PlayerState::IntroThinkForABit,
        _ => panic!("invalid state id {id}"),
    }
}
