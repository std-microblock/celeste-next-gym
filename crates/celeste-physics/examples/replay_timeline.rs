use std::collections::BTreeMap;
use std::error::Error;
use std::io;

use celeste_physics::{InputState, Map, PlayerSnapshot, simulate_trace};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ReplayRequest {
    timeline: Timeline,
    frames: Vec<usize>,
}

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

#[derive(Serialize)]
struct ReplayResponse {
    input_count: usize,
    state_count: usize,
    states: BTreeMap<usize, PlayerSnapshot>,
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

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn main() -> Result<(), Box<dyn Error>> {
    let request: ReplayRequest = serde_json::from_reader(io::stdin().lock())?;
    if request.timeline.version != 2 {
        return Err(invalid_data(format!(
            "unsupported timeline version {} (expected 2)",
            request.timeline.version
        ))
        .into());
    }

    let mut previous = FrameButtons::default();
    let inputs: Vec<_> = request
        .timeline
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
        request.timeline.initial_state,
        &inputs,
        &request.timeline.map,
        inputs.len() as u32,
    )?;

    let mut states = BTreeMap::new();
    for frame in request.frames {
        let state = trace.states.get(frame).ok_or_else(|| {
            invalid_data(format!("requested state frame {frame} is out of range"))
        })?;
        states.insert(frame, state.clone());
    }

    serde_json::to_writer(
        io::stdout().lock(),
        &ReplayResponse {
            input_count: inputs.len(),
            state_count: trace.states.len(),
            states,
        },
    )?;
    Ok(())
}
