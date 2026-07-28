use std::collections::HashMap;

use celeste_fuzz::{OutputMode, SearchOptions, compile, parse_spec};
use celeste_physics::{Map, PlayerSnapshot};
use criterion::{Criterion, criterion_group, criterion_main};

fn benchmark_search(c: &mut Criterion) {
    let spec = parse_spec(
        r#"{
          "version": 1,
          "variables": [{"name":"jump_frame","range":{"from":1,"to":120}}],
          "inputs": [
            {"keys":["right"],"at":0,"held_time":"hold::inf","verify":false},
            {"keys":["jump"],"at":"jump_frame"}
          ],
          "observe_until": 140,
          "success": ["!final.dead"],
          "objectives": [{"type":"maximize","expression":"final.pos.x"}]
        }"#,
    )
    .expect("valid benchmark spec");
    let compiled = compile(spec).expect("compiles");
    c.bench_function("single_variable_cached_search", |b| {
        b.iter(|| {
            compiled
                .search(
                    PlayerSnapshot::default(),
                    &Map::default(),
                    HashMap::new(),
                    vec![OutputMode::Best],
                    SearchOptions::default(),
                )
                .expect("search succeeds")
        })
    });
}

criterion_group!(benches, benchmark_search);
criterion_main!(benches);
