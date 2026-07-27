use celeste_physics::{InputState, Map, PlayerSnapshot, PlayerState, Rect, Vec2, simulate_trace};

fn main() {
    let left = Rect::new(0.0, 0.0, 320.0, 184.0);
    let right = Rect::new(320.0, 0.0, 320.0, 184.0);
    let map = Map {
        bounds: left,
        transition_rooms: vec![right],
        solids: vec![
            Rect::new(232.0, 120.0, 80.0, 64.0),
            Rect::new(312.0, 112.0, 8.0, 8.0),
        ],
        ..Map::default()
    };
    let player = PlayerSnapshot {
        pos: Vec2::new(240.0, 120.0),
        state: PlayerState::Normal,
        on_ground: true,
        ..PlayerSnapshot::default()
    };
    let mut found = 0;
    for ground_jump in 0..=20 {
        for cb in ground_jump + 5..=65 {
            let inputs = (0..150)
                .map(|frame| InputState {
                    move_x: 1,
                    jump_pressed: frame == ground_jump || frame == cb || frame == cb + 1,
                    jump_held: (ground_jump..ground_jump + 13).contains(&frame)
                        || (cb..cb + 14).contains(&frame),
                    grab_held: frame == cb || frame == cb + 1,
                    ..InputState::default()
                })
                .collect::<Vec<_>>();
            let trace = simulate_trace(player.clone(), &inputs, &map, inputs.len() as u32).unwrap();
            let Some(completed) = trace
                .states
                .iter()
                .position(|state| state.current_room_bounds == Some(right))
            else {
                continue;
            };
            let state = &trace.states[completed];
            let costs = [cb, cb + 1]
                .into_iter()
                .filter(|&frame| {
                    trace.states[frame].stamina - trace.states[frame + 1].stamina > 27.49
                })
                .count();
            if costs == 2 && state.speed.x >= 130.0 {
                println!(
                    "ground={ground_jump} cb={cb},{} completed={completed} pos={:?} speed={:?} retained={} stamina={}",
                    cb + 1,
                    state.pos,
                    state.speed,
                    state.wall_speed_retained,
                    state.stamina,
                );
                found += 1;
                if found >= 50 {
                    return;
                }
            }
        }
    }
    println!("found={found}");
}
