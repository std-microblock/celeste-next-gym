//! Stable native bridge used by the Everest C# Mod.
//!
//! The API mirrors the browser-owned `celeste-wasm` bridge: a map is cached
//! once, then the same Rust Fuzz compiler/searcher is called for each lesson.
//! UTF-8 JSON is used at the ABI boundary so the .NET side does not depend on
//! Rust layout, allocator, or MessagePack implementation details.

use std::{
    collections::HashMap,
    panic::{AssertUnwindSafe, catch_unwind},
    slice,
    sync::{LazyLock, Mutex},
};

use celeste_fuzz::{OutputMode, SearchOptions, compile, evaluate_current_checks, parse_spec};
use celeste_physics::{Map, PlayerSnapshot};
use serde::Serialize;

static CACHED_MAP: LazyLock<Mutex<Option<Map>>> = LazyLock::new(|| Mutex::new(None));

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativeBuffer {
    pub data: *mut u8,
    pub len: usize,
    pub capacity: usize,
}

impl NativeBuffer {
    fn from_vec(mut bytes: Vec<u8>) -> Self {
        let result = Self {
            data: bytes.as_mut_ptr(),
            len: bytes.len(),
            capacity: bytes.capacity(),
        };
        std::mem::forget(bytes);
        result
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeResponse<T: Serialize> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Decode and cache a simulator map. Returns a JSON `NativeResponse`.
///
/// # Safety
/// `json` must point to `json_len` readable bytes for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_gym_cache_map_json(
    json: *const u8,
    json_len: usize,
) -> NativeBuffer {
    ffi(|| {
        let map: Map = serde_json::from_slice(unsafe { input(json, json_len)? })
            .map_err(|error| format!("invalid map JSON: {error}"))?;
        *CACHED_MAP
            .lock()
            .map_err(|_| "cached map lock is poisoned".to_owned())? = Some(map);
        Ok(())
    })
}

/// Run the exact `celeste-fuzz` engine used by `celeste-wasm` against the
/// cached map. Every successful candidate and evaluation is returned because
/// the game filters feasible timings as the player performs each action.
///
/// # Safety
/// Both pointers must address their declared readable byte lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_gym_fuzz_search_json(
    snapshot_json: *const u8,
    snapshot_len: usize,
    fuzz_json: *const u8,
    fuzz_len: usize,
) -> NativeBuffer {
    ffi(|| {
        let snapshot: PlayerSnapshot = serde_json::from_slice(unsafe {
            input(snapshot_json, snapshot_len)?
        })
        .map_err(|error| format!("invalid snapshot JSON: {error}"))?;
        let fuzz_text = std::str::from_utf8(unsafe { input(fuzz_json, fuzz_len)? })
            .map_err(|error| format!("fuzz JSON is not UTF-8: {error}"))?;
        let compiled = compile(parse_spec(fuzz_text).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
        let map = CACHED_MAP
            .lock()
            .map_err(|_| "cached map lock is poisoned".to_owned())?;
        let map = map
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
    })
}

/// Evaluate author entry checks in the same restricted Rhai environment.
///
/// # Safety
/// Both pointers must address their declared readable byte lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_gym_training_entry_check_json(
    snapshot_json: *const u8,
    snapshot_len: usize,
    checks_json: *const u8,
    checks_len: usize,
) -> NativeBuffer {
    ffi(|| {
        let snapshot: PlayerSnapshot = serde_json::from_slice(unsafe {
            input(snapshot_json, snapshot_len)?
        })
        .map_err(|error| format!("invalid snapshot JSON: {error}"))?;
        let checks: Vec<String> = serde_json::from_slice(unsafe {
            input(checks_json, checks_len)?
        })
        .map_err(|error| format!("invalid checks JSON: {error}"))?;
        evaluate_current_checks(&snapshot, &checks).map_err(|error| error.to_string())
    })
}

/// Release a buffer returned by this library. A buffer must be released once.
///
/// # Safety
/// `buffer` must be an unchanged value returned by this exact library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_gym_buffer_free(buffer: NativeBuffer) {
    if buffer.data.is_null() {
        return;
    }
    drop(unsafe { Vec::from_raw_parts(buffer.data, buffer.len, buffer.capacity) });
}

unsafe fn input<'a>(pointer: *const u8, len: usize) -> Result<&'a [u8], String> {
    if len == 0 {
        return Ok(&[]);
    }
    if pointer.is_null() {
        return Err("input pointer is null".into());
    }
    Ok(unsafe { slice::from_raw_parts(pointer, len) })
}

fn ffi<T, F>(operation: F) -> NativeBuffer
where
    T: Serialize,
    F: FnOnce() -> Result<T, String>,
{
    let response = match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(result)) => NativeResponse {
            success: true,
            result: Some(result),
            error: None,
        },
        Ok(Err(error)) => NativeResponse::<T> {
            success: false,
            result: None,
            error: Some(error),
        },
        Err(_) => NativeResponse::<T> {
            success: false,
            result: None,
            error: Some("native fuzz bridge panicked".into()),
        },
    };
    NativeBuffer::from_vec(serde_json::to_vec(&response).unwrap_or_else(|error| {
        format!(r#"{{"success":false,"error":"response serialization failed: {error}"}}"#)
            .into_bytes()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncached_fuzz_call_returns_owned_json_error() {
        let snapshot = br#"{}"#;
        let fuzz = br#"{}"#;
        let buffer = unsafe {
            celeste_gym_fuzz_search_json(
                snapshot.as_ptr(),
                snapshot.len(),
                fuzz.as_ptr(),
                fuzz.len(),
            )
        };
        let bytes = unsafe { slice::from_raw_parts(buffer.data, buffer.len) };
        let response: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        assert_eq!(response["success"], false);
        unsafe { celeste_gym_buffer_free(buffer) };
    }
}
