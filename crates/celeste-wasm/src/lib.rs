use celeste_fuzz::{OutputMode, SearchOptions, compile, evaluate_current_checks, parse_spec};
use celeste_physics::{
    InputState, Map, PlayerSnapshot, celeste_map_rooms, decode_map, decode_map_room, simulate_trace,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

thread_local! {
    static CACHED_MAP: RefCell<Option<Map>> = const { RefCell::new(None) };
}

#[derive(Serialize)]
struct WasmError<'a> {
    success: bool,
    error: &'a str,
}

#[derive(Deserialize, Serialize)]
struct WasmMapResponse {
    success: bool,
    map: Map,
}

#[derive(Deserialize, Serialize)]
struct WasmRoomListResponse {
    success: bool,
    rooms: Vec<String>,
}

/// Return all room IDs stored in an original Celeste BinaryPacker `.bin` map.
#[wasm_bindgen]
pub fn list_celeste_map_rooms_msgpack(map_bytes: &[u8]) -> Vec<u8> {
    match celeste_map_rooms(map_bytes) {
        Ok(rooms) => rmp_serde::to_vec_named(&WasmRoomListResponse {
            success: true,
            rooms,
        })
        .unwrap_or_default(),
        Err(error) => {
            let message = error.to_string();
            rmp_serde::to_vec_named(&WasmError {
                success: false,
                error: &message,
            })
            .unwrap_or_default()
        }
    }
}

/// Decode an original Celeste BinaryPacker `.bin` map inside WASM and return
/// the selected room as the same MessagePack map used by the simulator.
#[wasm_bindgen]
pub fn decode_celeste_map_msgpack(map_bytes: &[u8], room: &str) -> Vec<u8> {
    let selected_room = (!room.is_empty()).then_some(room);
    match decode_map_room(map_bytes, selected_room) {
        Ok(map) => {
            rmp_serde::to_vec_named(&WasmMapResponse { success: true, map }).unwrap_or_default()
        }
        Err(error) => {
            let message = error.to_string();
            rmp_serde::to_vec_named(&WasmError {
                success: false,
                error: &message,
            })
            .unwrap_or_default()
        }
    }
}

/// MessagePack-in / MessagePack-out bridge designed to run inside a Web Worker.
/// Successful output is `SimulationResult`; failures are `{success:false,error:string}`.
#[wasm_bindgen]
pub fn simulate_msgpack(
    snapshot_bytes: &[u8],
    input_bytes: &[u8],
    map_bytes: &[u8],
    frames: u32,
) -> Vec<u8> {
    match run(snapshot_bytes, input_bytes, map_bytes, frames) {
        Ok(bytes) => bytes,
        Err(message) => rmp_serde::to_vec_named(&WasmError {
            success: false,
            error: &message,
        })
        .unwrap_or_default(),
    }
}

/// Decode and retain a simulation map inside the Worker-owned WASM instance.
/// Subsequent single-frame requests only need to transfer snapshot and input bytes.
#[wasm_bindgen]
pub fn cache_simulation_map_msgpack(map_bytes: &[u8]) -> Result<(), JsValue> {
    let map = decode_map(map_bytes).map_err(|error| JsValue::from_str(&error.to_string()))?;
    CACHED_MAP.with(|cached| *cached.borrow_mut() = Some(map));
    Ok(())
}

/// Simulate against the map installed by `cache_simulation_map_msgpack`.
#[wasm_bindgen]
pub fn simulate_cached_map_msgpack(
    snapshot_bytes: &[u8],
    input_bytes: &[u8],
    frames: u32,
) -> Vec<u8> {
    let result = CACHED_MAP.with(|cached| {
        let cached = cached.borrow();
        let map = cached
            .as_ref()
            .ok_or_else(|| "simulation map is not cached".to_owned())?;
        run_with_map(snapshot_bytes, input_bytes, map, frames)
    });
    match result {
        Ok(bytes) => bytes,
        Err(message) => rmp_serde::to_vec_named(&WasmError {
            success: false,
            error: &message,
        })
        .unwrap_or_default(),
    }
}

/// Run the same restricted-Rhai Fuzz engine used by native tooling against the
/// Worker-owned map.  The training UI asks for all successful candidates so it
/// can filter a live attempt incrementally; author configuration still only
/// controls the documented Fuzz outputs.
#[wasm_bindgen]
pub fn fuzz_search_cached_map_msgpack(snapshot_bytes: &[u8], fuzz_json: &str) -> Vec<u8> {
    let result = (|| -> Result<Vec<u8>, String> {
        let snapshot: PlayerSnapshot = rmp_serde::from_slice(snapshot_bytes)
            .map_err(|error| format!("invalid snapshot: {error}"))?;
        let compiled = compile(parse_spec(fuzz_json).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
        let search = CACHED_MAP.with(|cached| {
            let cached = cached.borrow();
            let map = cached
                .as_ref()
                .ok_or_else(|| "simulation map is not cached".to_owned())?;
            compiled
                .search(
                    snapshot,
                    map,
                    HashMap::new(),
                    vec![
                        OutputMode::Best,
                        OutputMode::Windows,
                        OutputMode::Coverage,
                        OutputMode::Candidates,
                        OutputMode::Evaluations,
                    ],
                    SearchOptions::default(),
                )
                .map_err(|error| error.to_string())
        })?;
        rmp_serde::to_vec_named(&search).map_err(|error| error.to_string())
    })();
    match result {
        Ok(bytes) => bytes,
        Err(message) => rmp_serde::to_vec_named(&WasmError {
            success: false,
            error: &message,
        })
        .unwrap_or_default(),
    }
}

/// Evaluate a training entry check using the exact same restricted Rhai surface
/// as Fuzz.  The caller supplies the post-simulation snapshot.
#[wasm_bindgen]
pub fn training_entry_check_msgpack(snapshot_bytes: &[u8], checks_json: &str) -> Vec<u8> {
    let result = (|| -> Result<Vec<u8>, String> {
        let snapshot: PlayerSnapshot = rmp_serde::from_slice(snapshot_bytes)
            .map_err(|error| format!("invalid snapshot: {error}"))?;
        let checks: Vec<String> = serde_json::from_str(checks_json)
            .map_err(|error| format!("invalid entry checks: {error}"))?;
        rmp_serde::to_vec_named(
            &evaluate_current_checks(&snapshot, &checks).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())
    })();
    match result {
        Ok(bytes) => bytes,
        Err(message) => rmp_serde::to_vec_named(&WasmError {
            success: false,
            error: &message,
        })
        .unwrap_or_default(),
    }
}

#[derive(Serialize)]
struct WasmAuditResponse {
    success: bool,
    rooms: Vec<celeste_physics::CelesteRoomAudit>,
}

/// Enumerate every room of a Celeste BinaryPacker `.bin` with its raw entity
/// and trigger names. Used by tooling to decide whether a map is loadable and
/// which gameplay objects the simulator would ignore.
#[wasm_bindgen]
pub fn audit_celeste_map_msgpack(map_bytes: &[u8]) -> Vec<u8> {
    match celeste_physics::audit_celeste_map(map_bytes) {
        Ok(rooms) => rmp_serde::to_vec_named(&WasmAuditResponse {
            success: true,
            rooms,
        })
        .unwrap_or_default(),
        Err(error) => error_bytes(&error.to_string()),
    }
}

#[derive(Serialize)]
struct WasmFuzzResponse {
    success: bool,
    result: celeste_fuzz::FuzzResult,
    estimated_candidates: u64,
    /// Fully resolved per-frame inputs of `result.best`, when present.
    best_inputs: Option<Vec<InputState>>,
}

/// General-purpose Fuzz entry point for tooling (CLI / scripts). Unlike the
/// training bridge this honours the specification's own `search.output` and
/// `search.bindings`, takes the map explicitly, and also returns the resolved
/// inputs of the best candidate so it can be replayed or rendered.
/// `max_candidates == 0` keeps the specification limit.
#[wasm_bindgen]
pub fn fuzz_search_msgpack(
    snapshot_bytes: &[u8],
    map_bytes: &[u8],
    fuzz_json: &str,
    max_candidates: f64,
) -> Vec<u8> {
    let result = (|| -> Result<Vec<u8>, String> {
        let snapshot: PlayerSnapshot = rmp_serde::from_slice(snapshot_bytes)
            .map_err(|error| format!("invalid snapshot: {error}"))?;
        let map = decode_map(map_bytes).map_err(|error| error.to_string())?;
        let compiled = compile(parse_spec(fuzz_json).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
        let options = SearchOptions {
            max_candidates: (max_candidates >= 1.0).then_some(max_candidates as u64),
            ..SearchOptions::default()
        };
        let result = compiled
            .search(snapshot, &map, HashMap::new(), Vec::new(), options)
            .map_err(|error| error.to_string())?;
        let best_inputs = match &result.best {
            Some(best) => Some(
                compiled
                    .resolve_inputs(&best.bindings)
                    .map_err(|error| error.to_string())?,
            ),
            None => None,
        };
        rmp_serde::to_vec_named(&WasmFuzzResponse {
            success: true,
            estimated_candidates: compiled.estimate_candidates(),
            result,
            best_inputs,
        })
        .map_err(|error| error.to_string())
    })();
    result.unwrap_or_else(|message| error_bytes(&message))
}

/// Resolve the full per-frame input schedule of a Fuzz candidate.
/// `bindings_json` is a JSON object mapping every variable to an integer.
#[wasm_bindgen]
pub fn fuzz_resolve_inputs_msgpack(fuzz_json: &str, bindings_json: &str) -> Vec<u8> {
    let result = (|| -> Result<Vec<u8>, String> {
        let compiled = compile(parse_spec(fuzz_json).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
        let bindings: std::collections::BTreeMap<String, i64> = serde_json::from_str(bindings_json)
            .map_err(|error| format!("invalid bindings: {error}"))?;
        let inputs = compiled
            .resolve_inputs(&bindings)
            .map_err(|error| error.to_string())?;
        rmp_serde::to_vec_named(&inputs).map_err(|error| error.to_string())
    })();
    result.unwrap_or_else(|message| error_bytes(&message))
}

fn error_bytes(message: &str) -> Vec<u8> {
    rmp_serde::to_vec_named(&WasmError {
        success: false,
        error: message,
    })
    .unwrap_or_default()
}

fn run(
    snapshot_bytes: &[u8],
    input_bytes: &[u8],
    map_bytes: &[u8],
    frames: u32,
) -> Result<Vec<u8>, String> {
    let snapshot: PlayerSnapshot =
        rmp_serde::from_slice(snapshot_bytes).map_err(|e| format!("invalid snapshot: {e}"))?;
    let inputs: Vec<InputState> =
        rmp_serde::from_slice(input_bytes).map_err(|e| format!("invalid inputs: {e}"))?;
    let map = decode_map(map_bytes).map_err(|e| e.to_string())?;
    run_decoded(snapshot, inputs, &map, frames)
}

fn run_with_map(
    snapshot_bytes: &[u8],
    input_bytes: &[u8],
    map: &Map,
    frames: u32,
) -> Result<Vec<u8>, String> {
    let snapshot: PlayerSnapshot =
        rmp_serde::from_slice(snapshot_bytes).map_err(|e| format!("invalid snapshot: {e}"))?;
    let inputs: Vec<InputState> =
        rmp_serde::from_slice(input_bytes).map_err(|e| format!("invalid inputs: {e}"))?;
    run_decoded(snapshot, inputs, map, frames)
}

fn run_decoded(
    snapshot: PlayerSnapshot,
    inputs: Vec<InputState>,
    map: &Map,
    frames: u32,
) -> Result<Vec<u8>, String> {
    let result = simulate_trace(snapshot, &inputs, &map, frames).map_err(|e| e.to_string())?;
    rmp_serde::to_vec_named(&result).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use celeste_physics::{Map, encode_map};

    #[test]
    fn bridge_returns_a_trace() {
        let snapshot = rmp_serde::to_vec_named(&PlayerSnapshot::default()).unwrap();
        let inputs = rmp_serde::to_vec_named(&vec![InputState::default()]).unwrap();
        let map = encode_map(&Map::default()).unwrap();
        let result = simulate_msgpack(&snapshot, &inputs, &map, 1);
        let decoded: celeste_physics::SimulationResult = rmp_serde::from_slice(&result).unwrap();
        assert_eq!(decoded.states.len(), 2);
    }

    #[test]
    fn cached_map_bridge_returns_a_trace() {
        let snapshot = rmp_serde::to_vec_named(&PlayerSnapshot::default()).unwrap();
        let inputs = rmp_serde::to_vec_named(&vec![InputState::default()]).unwrap();
        let map = encode_map(&Map::default()).unwrap();
        cache_simulation_map_msgpack(&map).unwrap();
        let result = simulate_cached_map_msgpack(&snapshot, &inputs, 1);
        let decoded: celeste_physics::SimulationResult = rmp_serde::from_slice(&result).unwrap();
        assert_eq!(decoded.states.len(), 2);
    }

    #[test]
    fn bridge_decodes_a_celeste_room() {
        let mut source_map = Map::default();
        source_map.bounds.height = 184.0;
        let map =
            celeste_physics::encode_celeste_map(&source_map, "CelesteGymPlayground", "playground")
                .unwrap();
        let result = decode_celeste_map_msgpack(&map, "playground");
        let decoded: WasmMapResponse = rmp_serde::from_slice(&result).unwrap();
        assert!(decoded.success);
        assert_eq!(decoded.map.bounds.width, 320.0);
    }

    #[test]
    fn tooling_fuzz_returns_best_inputs() {
        let mut source_map = Map::default();
        source_map
            .solids
            .push(celeste_physics::Rect::new(0.0, 160.0, 320.0, 20.0));
        let snapshot = rmp_serde::to_vec_named(&PlayerSnapshot {
            pos: celeste_physics::Vec2::new(40.0, 160.0),
            on_ground: true,
            ..PlayerSnapshot::default()
        })
        .unwrap();
        let map = encode_map(&source_map).unwrap();
        let spec = r#"{"version":1,
            "variables":[{"name":"hold","range":{"from":1,"to":4}}],
            "inputs":[{"keys":["right"],"at":0,"held_time":"hold"}],
            "observe_until":6,
            "objectives":[{"type":"maximize","expression":"final.pos.x"}],
            "search":{"output":["best","top_2"]}}"#;
        let bytes = fuzz_search_msgpack(&snapshot, &map, spec, 0.0);
        let value: serde_json::Value = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(value["success"], true, "{value}");
        assert_eq!(value["result"]["best"]["bindings"]["hold"], 4);
        assert_eq!(value["result"]["top"].as_array().unwrap().len(), 2);
        let inputs = value["best_inputs"].as_array().unwrap();
        assert_eq!(inputs.len(), 6);
        assert_eq!(inputs[3]["move_x"], 1);
        assert_eq!(inputs[4]["move_x"], 0);

        let resolved = fuzz_resolve_inputs_msgpack(spec, r#"{"hold":2}"#);
        let inputs: Vec<InputState> = rmp_serde::from_slice(&resolved).unwrap();
        assert_eq!(inputs.iter().filter(|input| input.move_x == 1).count(), 2);
    }

    #[test]
    fn bridge_audits_celeste_rooms() {
        let mut source_map = Map::default();
        source_map.bounds.height = 184.0;
        let map = celeste_physics::encode_celeste_map(&source_map, "TestMap", "a-00").unwrap();
        let value: serde_json::Value =
            rmp_serde::from_slice(&audit_celeste_map_msgpack(&map)).unwrap();
        assert_eq!(value["success"], true);
        assert_eq!(value["rooms"][0]["name"], "a-00");
    }

    #[test]
    fn bridge_lists_celeste_rooms() {
        let mut source_map = Map::default();
        source_map.bounds.height = 184.0;
        let map = celeste_physics::encode_celeste_map(&source_map, "TestMap", "a-00").unwrap();
        let result = list_celeste_map_rooms_msgpack(&map);
        let decoded: WasmRoomListResponse = rmp_serde::from_slice(&result).unwrap();
        assert!(decoded.success);
        assert_eq!(decoded.rooms[0], "a-00");
    }
}
