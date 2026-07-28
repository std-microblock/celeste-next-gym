use celeste_physics::{
    InputState, PlayerSnapshot, Simulator, encode_map, mechanics_playground, simulate,
};
use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const FRAMES: usize = 120;
const JOBS: usize = 2048;
const SINGLE_FRAME_JOBS: usize = 16384;
const REPEATS: usize = 5;

fn inputs() -> Vec<InputState> {
    (0..FRAMES)
        .map(|frame| InputState {
            move_x: if frame % 90 < 45 { 1 } else { -1 },
            jump_pressed: frame % 37 == 0,
            jump_held: frame % 37 < 8,
            dash_pressed: frame % 53 == 11,
            ..InputState::default()
        })
        .collect()
}

fn timed<F: FnMut()>(mut f: F) -> Duration {
    let start = Instant::now();
    for _ in 0..REPEATS {
        f();
    }
    start.elapsed() / REPEATS as u32
}

fn throughput_jobs(label: &str, duration: Duration, jobs: usize) {
    println!(
        "{label}_ms={:.3} sims_per_sec={:.1}",
        duration.as_secs_f64() * 1e3,
        jobs as f64 / duration.as_secs_f64()
    );
}

fn throughput(label: &str, duration: Duration) {
    throughput_jobs(label, duration, JOBS);
}

fn main() {
    let map = Arc::new(mechanics_playground());
    let inputs = Arc::new(inputs());
    let snapshot = PlayerSnapshot {
        pos: map.spawn,
        ..PlayerSnapshot::default()
    };
    let map_bytes = encode_map(&map).unwrap();
    let snapshot_bytes = rmp_serde::to_vec_named(&snapshot).unwrap();
    let input_bytes = rmp_serde::to_vec_named(&*inputs).unwrap();

    println!("jobs={JOBS} frames={FRAMES}");
    let single = timed(|| {
        let mut checksum = 0.0;
        for _ in 0..JOBS {
            checksum += simulate(snapshot.clone(), &inputs, &map, FRAMES as u32)
                .unwrap()
                .pos
                .x;
        }
        black_box(checksum);
    });
    throughput("direct_single", single);

    for workers in [2, 4, 8, 16, 24] {
        let parallel = timed(|| {
            let mut handles = Vec::new();
            for worker in 0..workers {
                let map = Arc::clone(&map);
                let inputs = Arc::clone(&inputs);
                let snapshot = snapshot.clone();
                handles.push(thread::spawn(move || {
                    let start = JOBS * worker / workers;
                    let end = JOBS * (worker + 1) / workers;
                    let mut checksum = 0.0;
                    for _ in start..end {
                        checksum += simulate(snapshot.clone(), &inputs, &map, FRAMES as u32)
                            .unwrap()
                            .pos
                            .x;
                    }
                    checksum
                }));
            }
            let checksum: f32 = handles.into_iter().map(|h| h.join().unwrap()).sum();
            black_box(checksum);
        });
        throughput(&format!("direct_{workers}t"), parallel);
    }

    let msgpack = timed(|| {
        let mut checksum = 0.0;
        for _ in 0..JOBS {
            let decoded_snapshot: PlayerSnapshot = rmp_serde::from_slice(&snapshot_bytes).unwrap();
            let decoded_inputs: Vec<InputState> = rmp_serde::from_slice(&input_bytes).unwrap();
            let decoded_map = celeste_physics::decode_map(&map_bytes).unwrap();
            let result = simulate(
                decoded_snapshot,
                &decoded_inputs,
                &decoded_map,
                FRAMES as u32,
            )
            .unwrap();
            let encoded = rmp_serde::to_vec_named(&result).unwrap();
            checksum += encoded.len() as f32 + result.pos.x;
        }
        black_box(checksum);
    });
    throughput("msgpack_roundtrip", msgpack);

    let persistent = timed(|| {
        let mut checksum = 0.0;
        let mut simulator = Simulator::new(snapshot.clone(), &map).unwrap();
        for _ in 0..JOBS {
            simulator.run(&inputs, FRAMES as u32).unwrap();
            checksum += simulator.snapshot().pos.x;
        }
        black_box(checksum);
    });
    throughput("persistent_context", persistent);

    let one_input = Arc::new(vec![inputs[0]]);
    let one_frame = timed(|| {
        let mut checksum = 0.0;
        for _ in 0..SINGLE_FRAME_JOBS {
            checksum += simulate(snapshot.clone(), &one_input, &map, 1)
                .unwrap()
                .pos
                .x;
        }
        black_box(checksum);
    });
    throughput_jobs("single_frame_direct", one_frame, SINGLE_FRAME_JOBS);
    let one_frame_persistent = timed(|| {
        let mut checksum = 0.0;
        let mut simulator = Simulator::new(snapshot.clone(), &map).unwrap();
        for _ in 0..SINGLE_FRAME_JOBS {
            simulator.run(&one_input, 1).unwrap();
            checksum += simulator.snapshot().pos.x;
        }
        black_box(checksum);
    });
    throughput_jobs(
        "single_frame_persistent",
        one_frame_persistent,
        SINGLE_FRAME_JOBS,
    );
}
