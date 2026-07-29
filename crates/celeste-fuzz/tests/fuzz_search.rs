use std::collections::HashMap;

use celeste_fuzz::{FuzzError, OutputMode, SearchOptions, compile, parse_spec};
use celeste_physics::{InputState, Map, PlayerSnapshot, PlayerState, Simulator};

fn search(json: &str, output: Vec<OutputMode>) -> celeste_fuzz::FuzzResult {
    compile(parse_spec(json).expect("valid spec"))
        .expect("compiles")
        .search(
            PlayerSnapshot::default(),
            &Map::default(),
            HashMap::new(),
            output,
            SearchOptions::default(),
        )
        .expect("search succeeds")
}

#[test]
fn schema_rejects_dependency_order_reserved_names_and_invalid_holds() {
    let later = r#"{
      "version":1,
      "variables":[
        {"name":"b","range":{"from":"a + 1","to":4}},
        {"name":"a","range":{"from":1,"to":2}}
      ], "inputs":[], "observe_until":0
    }"#;
    assert!(parse_spec(later).is_err());
    let reserved = r#"{"version":1,"variables":[{"name":"before","range":{"from":1,"to":2}}],"inputs":[],"observe_until":0}"#;
    assert!(parse_spec(reserved).is_err());
    let direction = r#"{"version":1,"inputs":[{"keys":["right"],"at":0}],"observe_until":1}"#;
    assert!(parse_spec(direction).is_err());
    let dash =
        r#"{"version":1,"inputs":[{"keys":["dash"],"at":0,"held_time":2}],"observe_until":1}"#;
    assert!(parse_spec(dash).is_err());
}

#[test]
fn directional_dash_can_hold_its_direction() {
    let result = search(
        r#"{
          "version":1,
          "inputs":[{"keys":["right","down","dash"],"at":0,"held_time":"hold::inf"}],
          "observe_until":4,
          "success":"!final.dead"
        }"#,
        vec![OutputMode::Candidates],
    );
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(
        result.candidates[0].verified_inputs[0].keys,
        ["right", "down", "dash"]
    );
}

#[test]
fn current_entry_checks_can_read_aim_and_dash_direction() {
    use celeste_fuzz::evaluate_current_checks;
    let snapshot = PlayerSnapshot {
        state: PlayerState::Dash,
        dash_dir: celeste_physics::Vec2 { x: 1.0, y: 1.0 },
        last_aim: celeste_physics::Vec2 { x: 1.0, y: 1.0 },
        ..PlayerSnapshot::default()
    };
    assert!(
        evaluate_current_checks(
            &snapshot,
            &[
                "current.state == state::dash".into(),
                "current.dash_dir.x > 0".into(),
                "current.last_aim.y > 0".into(),
            ]
        )
        .unwrap()
    );
}

#[test]
fn dependent_ranges_estimate_and_bindings_are_exact() {
    let spec = compile(
        parse_spec(
            r#"{
      "version":1,
      "variables":[
        {"name":"a","range":{"from":1,"to":3}},
        {"name":"b","range":{"from":"a + 1","to":"a + 2"}}
      ], "inputs":[], "observe_until":"b", "success":["a < 3"]
    }"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(spec.estimate_candidates(), 6);
    let result = spec
        .search(
            PlayerSnapshot::default(),
            &Map::default(),
            HashMap::from([("a".into(), 2)]),
            vec![OutputMode::Windows],
            SearchOptions::default(),
        )
        .unwrap();
    assert_eq!(result.stats.candidate_count, 2);
    assert_eq!(result.stats.successful_count, 2);
    assert_eq!(result.exact_windows[0].prefix.get("a"), Some(&2));
    assert_eq!(result.exact_windows[0].intervals, vec![(3, 4)]);
}

#[test]
fn enum_constants_and_same_frame_merge_have_native_semantics() {
    let result = search(
        r#"{
      "version":1,
      "inputs":[
        {"keys":["left"],"at":0,"held_time":"hold::inf","verify":false},
        {"keys":["right"],"at":0,"held_time":"hold::inf","verify":false}
      ],
      "observe_until":1,
      "success":["final.state == state::normal", "final.speed.x == 0"]
    }"#,
        vec![OutputMode::Best],
    );
    let best = result.best.expect("merged candidate succeeds");
    assert_eq!(best.final_state.state, PlayerState::Normal);
    assert_eq!(best.final_state.speed.x, 0.0);
    assert!(best.verified_inputs.is_empty());

    let dash = search(
        r#"{
      "version":1,
      "inputs":[{"keys":["dash"],"at":0,"after_input":"after.state == state::dash"}],
      "observe_until":1, "success":"final.state == state::dash"
    }"#,
        vec![OutputMode::Best],
    );
    assert_eq!(dash.best.unwrap().final_state.state, PlayerState::Dash);

    let string_enum = parse_spec(
        r#"{
      "version":1,"inputs":[],"observe_until":0,
      "success":"final.state == \"dash\""
    }"#,
    )
    .unwrap();
    assert!(compile(string_enum).is_err());
}

#[test]
fn before_and_after_prune_without_simulating_failed_suffixes() {
    let result = search(
        r#"{
      "version":1,
      "variables":[{"name":"a","range":{"from":1,"to":3}}],
      "inputs":[{"keys":["jump"],"at":"a","before_input":"a != 2","after_input":"after.state == state::normal"}],
      "observe_until":5,
      "success":"!final.dead"
    }"#,
        vec![OutputMode::Best],
    );
    assert_eq!(result.stats.candidate_count, 3);
    assert_eq!(result.stats.pruned_before, 1);
    assert_eq!(result.stats.successful_count, 2);
    assert!(result.stats.rhai_evaluations >= 5);
}

#[test]
fn windows_coverage_regions_and_objective_tiebreak_are_deterministic() {
    let result = search(
        r#"{
      "version":1,
      "variables":[
        {"name":"a","range":{"from":1,"to":2}},
        {"name":"b","range":{"from":1,"to":5}}
      ],
      "inputs":[], "observe_until":0,
      "success":"b != 3",
      "objectives":[{"type":"approach","expression":"b","target":2}],
      "search":{"output":["best","windows","coverage","top_3"]}
    }"#,
        vec![],
    );
    assert_eq!(result.best.as_ref().unwrap().bindings.get("a"), Some(&1));
    assert_eq!(result.best.as_ref().unwrap().bindings.get("b"), Some(&2));
    assert_eq!(result.top.len(), 3);
    assert_eq!(result.exact_windows.len(), 2);
    assert_eq!(result.exact_windows[0].intervals, vec![(1, 2), (4, 5)]);
    let coverage = result.coverage_report.unwrap();
    assert_eq!(coverage.entries.len(), 2);
    assert_eq!(coverage.entries[0].successful_width, 4);
    assert_eq!(coverage.entries[0].coverage_percent, 100.0);
    assert_eq!(result.connected_regions.len(), 2);
}

#[test]
fn evaluations_include_objectives_for_failed_candidates() {
    let result = search(
        r#"{
          "version":1,
          "variables":[{"name":"frame","range":{"from":0,"to":2}}],
          "inputs":[], "observe_until":0,
          "success":"frame == 1",
          "objectives":[{"type":"maximize","expression":"frame + 10"}]
        }"#,
        vec![OutputMode::Candidates, OutputMode::Evaluations],
    );
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.evaluations.len(), 3);
    assert_eq!(
        result
            .evaluations
            .iter()
            .map(|evaluation| (
                evaluation.bindings["frame"],
                evaluation.successful,
                evaluation.objective_values[0],
            ))
            .collect::<Vec<_>>(),
        [(0, false, 10.0), (1, true, 11.0), (2, false, 12.0)]
    );
}

#[test]
fn checkpoint_objectives_sample_the_selected_frame_and_sort_candidates() {
    let result = search(
        r#"{
          "version":1,
          "variables":[{"name":"frame","range":{"from":0,"to":1}}],
          "inputs":[], "observe_until":2,
          "success":"!final.dead",
          "checkpoints":[{
            "at":"frame",
            "objectives":[
              {"type":"maximize","expression":"at"},
              {"type":"maximize","expression":"sqrt(after.speed.x * after.speed.x + after.speed.y * after.speed.y)"}
            ]
          }],
          "objectives":[{"type":"maximize","expression":"final.stamina"}]
        }"#,
        vec![OutputMode::Best, OutputMode::Evaluations],
    );
    assert_eq!(result.best.as_ref().unwrap().bindings["frame"], 1);
    assert_eq!(result.best.as_ref().unwrap().objective_values.len(), 3);
    assert_eq!(
        result
            .evaluations
            .iter()
            .map(|candidate| candidate.objective_values[0])
            .collect::<Vec<_>>(),
        [0.0, 1.0]
    );
    assert!(result.evaluations.iter().all(|candidate| {
        candidate
            .objective_values
            .iter()
            .all(|value| value.is_finite())
    }));
}

#[test]
fn prefix_cache_reuses_identical_input_paths() {
    let result = search(
        r#"{
      "version":1,
      "variables":[{"name":"unused","range":{"from":1,"to":4}}],
      "inputs":[{"keys":["right"],"at":0,"held_time":"hold::inf","verify":false}],
      "observe_until":20,
      "success":"!final.dead"
    }"#,
        vec![OutputMode::Best],
    );
    assert_eq!(result.stats.successful_count, 4);
    assert!(result.stats.unique_simulated_frames < result.stats.naive_frame_count);
    assert!(result.stats.saved_frames > 0);
    assert!(result.stats.trie_nodes > 1);
}

#[test]
fn expression_error_identifies_tuple_and_input() {
    let compiled = compile(
        parse_spec(
            r#"{
      "version":1,
      "variables":[{"name":"a","range":{"from":1,"to":1}}],
      "inputs":[{"keys":["jump"],"at":0,"before_input":"before.not_a_field > 0"}],
      "observe_until":2
    }"#,
        )
        .unwrap(),
    )
    .unwrap();
    let error = compiled
        .search(
            PlayerSnapshot::default(),
            &Map::default(),
            HashMap::new(),
            vec![],
            SearchOptions::default(),
        )
        .unwrap_err();
    match error {
        FuzzError::Expression {
            input, bindings, ..
        } => {
            assert_eq!(input, " at input 0");
            assert_eq!(bindings, "(a=1)");
        }
        other => panic!("expected contextual expression error, got {other}"),
    }
}

#[test]
fn input_metadata_exposes_default_and_explicit_hold_policies() {
    let result = search(
        r#"{
          "version":1,
          "inputs":[
            {"keys":["jump"],"at":0,"before_input":"held_time == 12"},
            {"keys":["grab"],"at":0,"before_input":"held_time == hold::inf"},
            {"keys":["jump"],"at":1,"held_time":6,"before_input":"held_time == 6"}
          ],
          "observe_until":3, "success":"!final.dead"
        }"#,
        vec![OutputMode::Best],
    );
    assert_eq!(result.stats.pruned_before, 0);
    assert_eq!(result.stats.successful_count, 1);
}

#[test]
fn simulator_fork_keeps_runtime_state_composable() {
    let mut original = Simulator::new(PlayerSnapshot::default(), &Map::default()).unwrap();
    original
        .step(InputState {
            move_x: 1,
            ..InputState::default()
        })
        .unwrap();
    let mut fork = original.fork();
    let input = InputState {
        jump_pressed: true,
        jump_held: true,
        ..InputState::default()
    };
    original.step(input).unwrap();
    fork.step(input).unwrap();
    assert_eq!(original.snapshot(), fork.snapshot());
}

#[cfg(feature = "parallel")]
#[test]
fn parallel_search_matches_serial_candidates_and_reports() {
    let compiled = compile(
        parse_spec(
            r#"{
              "version":1,
              "variables":[
                {"name":"a","range":{"from":1,"to":4}},
                {"name":"b","range":{"from":1,"to":5}}
              ],
              "inputs":[{"keys":["jump"],"at":"a","held_time":"b"}],
              "observe_until":8,
              "success":["!final.dead", "b != 3"],
              "objectives":[{"type":"minimize","expression":"a + b"}]
            }"#,
        )
        .unwrap(),
    )
    .unwrap();
    let serial = compiled
        .search(
            PlayerSnapshot::default(),
            &Map::default(),
            HashMap::new(),
            vec![
                OutputMode::Best,
                OutputMode::Windows,
                OutputMode::Coverage,
                OutputMode::Evaluations,
                OutputMode::Top(5),
            ],
            SearchOptions::default(),
        )
        .unwrap();
    let parallel = compiled
        .search_parallel(
            PlayerSnapshot::default(),
            &Map::default(),
            HashMap::new(),
            vec![
                OutputMode::Best,
                OutputMode::Windows,
                OutputMode::Coverage,
                OutputMode::Evaluations,
                OutputMode::Top(5),
            ],
            SearchOptions {
                workers: Some(2),
                ..SearchOptions::default()
            },
        )
        .unwrap();
    assert_eq!(parallel.best, serial.best);
    assert_eq!(parallel.top, serial.top);
    assert_eq!(parallel.exact_windows, serial.exact_windows);
    assert_eq!(parallel.connected_regions, serial.connected_regions);
    assert_eq!(parallel.coverage_report, serial.coverage_report);
    assert_eq!(parallel.evaluations, serial.evaluations);
    assert_eq!(parallel.stats.candidate_count, serial.stats.candidate_count);
    assert_eq!(
        parallel.stats.successful_count,
        serial.stats.successful_count
    );
}
