use std::{fs, path::PathBuf};

use celeste_physics::{InputState, Map, PlayerSnapshot, simulate_trace};
use serde::Deserialize;

#[derive(Deserialize)]
struct Timeline {
    version: u8,
    map: Map,
    initial_state: PlayerSnapshot,
    inputs: Vec<FrameButtons>,
}

#[derive(Clone, Copy, Default, Deserialize)]
struct FrameButtons {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    jump: bool,
    dash: bool,
    crouch_dash: bool,
    grab: bool,
}

impl FrameButtons {
    fn to_input(self, previous: Self) -> InputState {
        InputState {
            move_x: match (self.left, self.right) {
                (true, false) => -1,
                (false, true) => 1,
                _ => 0,
            },
            move_y: match (self.up, self.down) {
                (true, false) => -1,
                (false, true) => 1,
                _ => 0,
            },
            jump_pressed: self.jump && !previous.jump,
            jump_held: self.jump,
            dash_pressed: self.dash && !previous.dash,
            crouch_dash_pressed: self.crouch_dash && !previous.crouch_dash,
            grab_held: self.grab,
            talk_pressed: false,
            frame_delta_time_bits: None,
        }
    }
}

fn load_timeline(name: &str) -> Timeline {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/timelines")
        .join(name);
    let bytes = fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read timeline fixture {}: {error}",
            path.display()
        )
    });
    serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "failed to decode timeline fixture {}: {error}",
            path.display()
        )
    })
}

#[test]
fn delayed_wallboost_timeline_replays() {
    let timeline = load_timeline("delayed-wallboost.json");
    assert_eq!(timeline.version, 2);

    let mut previous = FrameButtons::default();
    let inputs: Vec<_> = timeline
        .inputs
        .iter()
        .copied()
        .map(|buttons| {
            let input = buttons.to_input(previous);
            previous = buttons;
            input
        })
        .collect();
    let trace = simulate_trace(
        timeline.initial_state,
        &inputs,
        &timeline.map,
        inputs.len() as u32,
    )
    .expect("timeline should replay");

    for (frame, states) in trace.states.windows(2).enumerate() {
        let before = &states[0];
        let after = &states[1];
        if before.wall_boost_timer > 0.0 || after.wall_boost_timer > 0.0 {
            eprintln!(
                "frame={} input=({}, {}, jump={}, grab={}) pos=({:.3},{:.3}) speed=({:.3},{:.3}) state={:?} stamina={:.3} wall_boost=({:.6},{}) move_x={}",
                frame + 1,
                inputs[frame].move_x,
                inputs[frame].move_y,
                inputs[frame].jump_pressed,
                inputs[frame].grab_held,
                after.pos.x,
                after.pos.y,
                after.speed.x,
                after.speed.y,
                after.state,
                after.stamina,
                after.wall_boost_timer,
                after.wall_boost_dir,
                after.move_x,
            );
        }
    }

    assert_eq!(trace.states.len(), timeline.inputs.len() + 1);
}
