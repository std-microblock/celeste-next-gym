use celeste_physics::{
    InputState, Map, PlayerSnapshot, decode_map, decode_map_room, simulate_trace,
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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
}
