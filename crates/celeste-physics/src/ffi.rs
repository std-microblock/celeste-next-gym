use crate::{InputState, PlayerSnapshot, decode_map, simulate};
use std::{panic::catch_unwind, ptr, slice};

/// Conservative upper bound for the current named MessagePack snapshot representation.
#[unsafe(no_mangle)]
pub extern "C" fn celeste_snapshot_size() -> u32 {
    4096
}

/// Validate either the native MessagePack map format or an original Celeste `.bin` map.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_validate_map(map_ptr: *const u8, map_len: u32) -> u32 {
    if map_ptr.is_null() || map_len == 0 {
        return 0;
    }
    catch_unwind(|| {
        // SAFETY: caller promises a readable buffer of map_len bytes for this call.
        let bytes = unsafe { slice::from_raw_parts(map_ptr, map_len as usize) };
        u32::from(decode_map(bytes).is_ok())
    })
    .unwrap_or(0)
}

/// Simulate a sequence encoded with MessagePack.
///
/// `inputs_len` is the byte length of the serialized `InputState[]`. The original draft ABI
/// called this a frame count, but a byte length is required for memory-safe deserialization;
/// the number of frames remains the separate `frames` argument.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_simulate(
    snapshot_ptr: *const u8,
    snapshot_len: u32,
    inputs_ptr: *const u8,
    inputs_len: u32,
    map_ptr: *const u8,
    map_len: u32,
    frames: u32,
    out_buffer: *mut u8,
    out_size: u32,
) -> u32 {
    if snapshot_ptr.is_null() || inputs_ptr.is_null() || map_ptr.is_null() || out_buffer.is_null() {
        return 0;
    }
    catch_unwind(|| {
        // SAFETY: the C caller owns all buffers and promises the declared lengths.
        let snapshot_bytes = unsafe { slice::from_raw_parts(snapshot_ptr, snapshot_len as usize) };
        let input_bytes = unsafe { slice::from_raw_parts(inputs_ptr, inputs_len as usize) };
        let map_bytes = unsafe { slice::from_raw_parts(map_ptr, map_len as usize) };
        let snapshot: PlayerSnapshot = rmp_serde::from_slice(snapshot_bytes).ok()?;
        let inputs: Vec<InputState> = rmp_serde::from_slice(input_bytes).ok()?;
        let map = decode_map(map_bytes).ok()?;
        let result = simulate(snapshot, &inputs, &map, frames).ok()?;
        let encoded = rmp_serde::to_vec_named(&result).ok()?;
        if encoded.len() > out_size as usize {
            return None;
        }
        // SAFETY: size was checked and buffers must not overlap per the ABI contract.
        unsafe {
            ptr::copy_nonoverlapping(encoded.as_ptr(), out_buffer, encoded.len());
        }
        Some(encoded.len() as u32)
    })
    .ok()
    .flatten()
    .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Map, encode_map};
    #[test]
    fn ffi_round_trip() {
        let snapshot = rmp_serde::to_vec_named(&PlayerSnapshot::default()).unwrap();
        let inputs = rmp_serde::to_vec_named(&vec![InputState::default()]).unwrap();
        let map = encode_map(&Map::default()).unwrap();
        let mut out = vec![0u8; celeste_snapshot_size() as usize];
        let written = unsafe {
            celeste_simulate(
                snapshot.as_ptr(),
                snapshot.len() as u32,
                inputs.as_ptr(),
                inputs.len() as u32,
                map.as_ptr(),
                map.len() as u32,
                1,
                out.as_mut_ptr(),
                out.len() as u32,
            )
        };
        assert!(written > 0);
        let decoded: PlayerSnapshot = rmp_serde::from_slice(&out[..written as usize]).unwrap();
        assert_eq!(decoded.state, crate::PlayerState::Normal);
    }
}
