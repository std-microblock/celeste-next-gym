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
fn delayed_wallbounce_timeline_uses_lingering_dash_attack() {
    let timeline = load_timeline("delayed-wallbounce.json");
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

    assert_eq!(trace.states.len(), timeline.inputs.len() + 1);

    // The recording presses jump three Normal frames after the up-dash state
    // ends. Player.cs NormalUpdate checks DashAttacking and the retained
    // straight-up DashDir before falling back to an ordinary WallJump.
    let before_jump = &trace.states[1565];
    let after_jump = &trace.states[1566];
    assert_eq!(before_jump.state, celeste_physics::PlayerState::Normal);
    assert_eq!(before_jump.dash_dir, celeste_physics::Vec2::new(0.0, -1.0));
    assert!(before_jump.dash_attack_timer > 0.08);
    assert!(inputs[1565].jump_pressed);
    assert_eq!(after_jump.state, celeste_physics::PlayerState::Normal);
    assert_eq!(after_jump.speed, celeste_physics::Vec2::new(170.0, -160.0));
    assert_eq!(after_jump.dash_attack_timer, 0.0);
    assert_eq!(after_jump.force_move_x_timer, 0.0);
    assert!(!after_jump.dead);
}
