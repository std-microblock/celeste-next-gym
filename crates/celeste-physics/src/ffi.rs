use crate::{
    InputState, Map, PlayerSnapshot, PlayerState, Simulator, Vec2, decode_map, decode_map_room,
    simulate,
};
use std::{mem::size_of, panic::catch_unwind, ptr, slice};

/// Compact, C-compatible action. `flags` uses bit 0=jump pressed, bit 1=jump
/// held, bit 2=dash pressed, bit 3=crouch dash pressed, bit 4=grab held, and
/// bit 5=talk pressed.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CelesteInputPod {
    pub move_x: i8,
    pub move_y: i8,
    pub flags: u8,
}

/// Fixed-layout observation returned by the native hot path. Variable-length
/// entity state remains owned by the simulator handle; use
/// `celeste_simulator_snapshot_msgpack` when a complete portable snapshot is
/// required for persistence or transport.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CelestePlayerPod {
    pub pos: Vec2,
    pub speed: Vec2,
    pub dash_dir: Vec2,
    pub boost_target: Vec2,
    pub last_aim: Vec2,
    pub stamina: f32,
    pub state_timer: f32,
    pub dashes: u8,
    pub state: u8,
    pub facing: u8,
    pub flags: u8,
    pub respawn_frames: u16,
}

const FLAG_ON_GROUND: u8 = 1 << 0;
const FLAG_PLAYER_ON_GROUND: u8 = 1 << 1;
const FLAG_DUCKING: u8 = 1 << 2;
const FLAG_CAN_DREAM_DASH: u8 = 1 << 3;
const FLAG_DEAD: u8 = 1 << 4;
const FLAG_DEATH_FREEZE_PENDING: u8 = 1 << 5;
const FLAG_BOOST_RED: u8 = 1 << 6;

impl CelesteInputPod {
    fn input(self) -> InputState {
        InputState {
            move_x: self.move_x,
            move_y: self.move_y,
            jump_pressed: self.flags & (1 << 0) != 0,
            jump_held: self.flags & (1 << 1) != 0,
            dash_pressed: self.flags & (1 << 2) != 0,
            crouch_dash_pressed: self.flags & (1 << 3) != 0,
            grab_held: self.flags & (1 << 4) != 0,
            talk_pressed: self.flags & (1 << 5) != 0,
            frame_delta_time_bits: None,
        }
    }
}

impl From<&PlayerSnapshot> for CelestePlayerPod {
    fn from(snapshot: &PlayerSnapshot) -> Self {
        let mut flags = 0;
        if snapshot.on_ground {
            flags |= FLAG_ON_GROUND;
        }
        if snapshot.player_on_ground {
            flags |= FLAG_PLAYER_ON_GROUND;
        }
        if snapshot.ducking {
            flags |= FLAG_DUCKING;
        }
        if snapshot.can_dream_dash {
            flags |= FLAG_CAN_DREAM_DASH;
        }
        if snapshot.dead {
            flags |= FLAG_DEAD;
        }
        if snapshot.death_freeze_pending {
            flags |= FLAG_DEATH_FREEZE_PENDING;
        }
        if snapshot.boost_red {
            flags |= FLAG_BOOST_RED;
        }
        Self {
            pos: snapshot.pos,
            speed: snapshot.speed,
            dash_dir: snapshot.dash_dir,
            boost_target: snapshot.boost_target,
            last_aim: snapshot.last_aim,
            stamina: snapshot.stamina,
            state_timer: snapshot.state_timer,
            dashes: snapshot.dashes,
            state: snapshot.state as u8,
            facing: snapshot.facing as u8,
            flags,
            respawn_frames: snapshot.respawn_frames,
        }
    }
}

fn snapshot_from_pod(pod: &CelestePlayerPod) -> Option<PlayerSnapshot> {
    let state = match pod.state {
        0 => PlayerState::Normal,
        1 => PlayerState::Climb,
        2 => PlayerState::Dash,
        3 => PlayerState::Swim,
        4 => PlayerState::Boost,
        5 => PlayerState::RedDash,
        6 => PlayerState::HitSquash,
        7 => PlayerState::Launch,
        8 => PlayerState::Pickup,
        9 => PlayerState::DreamDash,
        10 => PlayerState::SummitLaunch,
        11 => PlayerState::Dummy,
        12 => PlayerState::IntroWalk,
        13 => PlayerState::IntroJump,
        14 => PlayerState::IntroRespawn,
        15 => PlayerState::IntroWakeUp,
        16 => PlayerState::BirdDashTutorial,
        17 => PlayerState::Frozen,
        18 => PlayerState::ReflectionFall,
        19 => PlayerState::StarFly,
        20 => PlayerState::TempleFall,
        21 => PlayerState::CassetteFly,
        22 => PlayerState::Attract,
        23 => PlayerState::IntroMoonJump,
        24 => PlayerState::FlingBird,
        25 => PlayerState::IntroThinkForABit,
        _ => return None,
    };
    let flags = pod.flags;
    Some(PlayerSnapshot {
        pos: pod.pos,
        speed: pod.speed,
        state,
        facing: pod.facing != 0,
        dashes: pod.dashes,
        stamina: pod.stamina,
        on_ground: flags & FLAG_ON_GROUND != 0,
        player_on_ground: flags & FLAG_PLAYER_ON_GROUND != 0,
        player_on_ground_initialized: true,
        ducking: flags & FLAG_DUCKING != 0,
        can_dream_dash: flags & FLAG_CAN_DREAM_DASH != 0,
        dead: flags & FLAG_DEAD != 0,
        death_freeze_pending: flags & FLAG_DEATH_FREEZE_PENDING != 0,
        respawn_frames: pod.respawn_frames,
        dash_dir: pod.dash_dir,
        last_aim: pod.last_aim,
        state_timer: pod.state_timer,
        boost_target: pod.boost_target,
        boost_red: flags & FLAG_BOOST_RED != 0,
        ..PlayerSnapshot::default()
    })
}

pub struct CelesteMapHandle {
    map: Map,
}

pub struct CelesteSimulatorHandle {
    simulator: Simulator,
}

#[unsafe(no_mangle)]
pub extern "C" fn celeste_input_pod_size() -> u32 {
    size_of::<CelesteInputPod>() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn celeste_player_pod_size() -> u32 {
    size_of::<CelestePlayerPod>() as u32
}

/// Decode a MessagePack or Celeste BinaryPacker map once and retain it behind
/// an immutable handle. A map handle can be shared while each thread owns a
/// separate simulator handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_map_create(
    map_ptr: *const u8,
    map_len: u32,
) -> *mut CelesteMapHandle {
    if map_ptr.is_null() || map_len == 0 {
        return ptr::null_mut();
    }
    catch_unwind(|| {
        // SAFETY: caller promises a readable buffer for this call.
        let bytes = unsafe { slice::from_raw_parts(map_ptr, map_len as usize) };
        let map = decode_map(bytes).ok()?;
        Some(Box::into_raw(Box::new(CelesteMapHandle { map })))
    })
    .ok()
    .flatten()
    .unwrap_or(ptr::null_mut())
}

/// Decode one named room from a Celeste BinaryPacker map. `room_ptr` is UTF-8
/// and does not need a trailing NUL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_map_create_room(
    map_ptr: *const u8,
    map_len: u32,
    room_ptr: *const u8,
    room_len: u32,
) -> *mut CelesteMapHandle {
    if map_ptr.is_null() || map_len == 0 || room_ptr.is_null() || room_len == 0 {
        return ptr::null_mut();
    }
    catch_unwind(|| {
        let bytes = unsafe { slice::from_raw_parts(map_ptr, map_len as usize) };
        let room_bytes = unsafe { slice::from_raw_parts(room_ptr, room_len as usize) };
        let room = std::str::from_utf8(room_bytes).ok()?;
        let map = decode_map_room(bytes, Some(room)).ok()?;
        Some(Box::into_raw(Box::new(CelesteMapHandle { map })))
    })
    .ok()
    .flatten()
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_map_destroy(handle: *mut CelesteMapHandle) {
    if !handle.is_null() {
        // SAFETY: handle was returned by celeste_map_create and is released once.
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// Create a persistent simulator directly from a fixed-layout POD state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_simulator_create_pod(
    snapshot: *const CelestePlayerPod,
    map: *const CelesteMapHandle,
) -> *mut CelesteSimulatorHandle {
    if snapshot.is_null() || map.is_null() {
        return ptr::null_mut();
    }
    catch_unwind(|| {
        // SAFETY: both pointers are valid for the duration of this call.
        let snapshot = snapshot_from_pod(unsafe { &*snapshot })?;
        let map = unsafe { &*map };
        let simulator = Simulator::new(snapshot, &map.map).ok()?;
        Some(Box::into_raw(Box::new(CelesteSimulatorHandle {
            simulator,
        })))
    })
    .ok()
    .flatten()
    .unwrap_or(ptr::null_mut())
}

/// Compatibility conversion: decode one complete portable snapshot, then use
/// the POD simulator API for all subsequent steps.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_simulator_create_msgpack(
    snapshot_ptr: *const u8,
    snapshot_len: u32,
    map: *const CelesteMapHandle,
) -> *mut CelesteSimulatorHandle {
    if snapshot_ptr.is_null() || snapshot_len == 0 || map.is_null() {
        return ptr::null_mut();
    }
    catch_unwind(|| {
        // SAFETY: caller promises valid buffers/handles for this call.
        let bytes = unsafe { slice::from_raw_parts(snapshot_ptr, snapshot_len as usize) };
        let snapshot: PlayerSnapshot = rmp_serde::from_slice(bytes).ok()?;
        let map = unsafe { &*map };
        let simulator = Simulator::new(snapshot, &map.map).ok()?;
        Some(Box::into_raw(Box::new(CelesteSimulatorHandle {
            simulator,
        })))
    })
    .ok()
    .flatten()
    .unwrap_or(ptr::null_mut())
}

/// Construct a fresh episode at the selected room's first player spawn.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_simulator_create_at_spawn(
    map: *const CelesteMapHandle,
) -> *mut CelesteSimulatorHandle {
    if map.is_null() {
        return ptr::null_mut();
    }
    catch_unwind(|| {
        let map = unsafe { &*map };
        let snapshot = PlayerSnapshot {
            pos: map.map.spawn,
            current_room_bounds: Some(map.map.bounds),
            ..PlayerSnapshot::default()
        };
        let simulator = Simulator::new(snapshot, &map.map).ok()?;
        Some(Box::into_raw(Box::new(CelesteSimulatorHandle {
            simulator,
        })))
    })
    .ok()
    .flatten()
    .unwrap_or(ptr::null_mut())
}

/// Clone an initialized context for exact state rewind or branching search.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_simulator_clone(
    handle: *const CelesteSimulatorHandle,
) -> *mut CelesteSimulatorHandle {
    if handle.is_null() {
        return ptr::null_mut();
    }
    catch_unwind(|| {
        let simulator = unsafe { &*handle }.simulator.fork();
        Box::into_raw(Box::new(CelesteSimulatorHandle { simulator }))
    })
    .unwrap_or(ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_simulator_destroy(handle: *mut CelesteSimulatorHandle) {
    if !handle.is_null() {
        // SAFETY: handle was returned by a celeste_simulator_create_* function.
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// Advance a persistent context and write one POD state for the initial state
/// plus one after every input. Returns the number of states written, or zero.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_simulator_run_pod(
    handle: *mut CelesteSimulatorHandle,
    inputs_ptr: *const CelesteInputPod,
    inputs_len: u32,
    out_states: *mut CelestePlayerPod,
    out_capacity: u32,
) -> u32 {
    if handle.is_null() || inputs_ptr.is_null() || out_states.is_null() {
        return 0;
    }
    let required = match inputs_len.checked_add(1) {
        Some(required) if required <= out_capacity => required,
        _ => return 0,
    };
    catch_unwind(|| {
        // SAFETY: caller provides non-overlapping input/output arrays of the declared sizes and
        // exclusive access to this simulator handle for the entire call.
        let simulator = unsafe { &mut *handle };
        let inputs = unsafe { slice::from_raw_parts(inputs_ptr, inputs_len as usize) };
        let states = unsafe { slice::from_raw_parts_mut(out_states, required as usize) };
        states[0] = CelestePlayerPod::from(simulator.simulator.snapshot());
        for (index, input) in inputs.iter().enumerate() {
            simulator.simulator.step(input.input()).ok()?;
            states[index + 1] = CelestePlayerPod::from(simulator.simulator.snapshot());
        }
        Some(required)
    })
    .ok()
    .flatten()
    .unwrap_or(0)
}

/// Encode the simulator's complete state for persistence or inter-process
/// transport. This conversion is deliberately outside the stepping hot path.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_simulator_snapshot_msgpack(
    handle: *const CelesteSimulatorHandle,
    out_buffer: *mut u8,
    out_size: u32,
) -> u32 {
    if handle.is_null() || out_buffer.is_null() {
        return 0;
    }
    catch_unwind(|| {
        // SAFETY: caller promises a readable handle and writable output buffer.
        let simulator = unsafe { &*handle };
        let encoded = rmp_serde::to_vec_named(simulator.simulator.snapshot()).ok()?;
        if encoded.len() > out_size as usize {
            return None;
        }
        unsafe {
            ptr::copy_nonoverlapping(encoded.as_ptr(), out_buffer, encoded.len());
        }
        Some(encoded.len() as u32)
    })
    .ok()
    .flatten()
    .unwrap_or(0)
}

/// Convert a MessagePack snapshot to the compact observation POD without
/// constructing a simulator.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_player_pod_from_msgpack(
    snapshot_ptr: *const u8,
    snapshot_len: u32,
    out_snapshot: *mut CelestePlayerPod,
) -> u32 {
    if snapshot_ptr.is_null() || snapshot_len == 0 || out_snapshot.is_null() {
        return 0;
    }
    catch_unwind(|| {
        let bytes = unsafe { slice::from_raw_parts(snapshot_ptr, snapshot_len as usize) };
        let snapshot: PlayerSnapshot = rmp_serde::from_slice(bytes).ok()?;
        unsafe {
            out_snapshot.write(CelestePlayerPod::from(&snapshot));
        }
        Some(1)
    })
    .ok()
    .flatten()
    .unwrap_or(0)
}

/// Convert a compact POD into a portable initial snapshot. Variable-length
/// entity state is initialized empty; create a simulator to populate it from
/// the selected map before exporting a resumable snapshot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn celeste_player_pod_to_msgpack(
    snapshot: *const CelestePlayerPod,
    out_buffer: *mut u8,
    out_size: u32,
) -> u32 {
    if snapshot.is_null() || out_buffer.is_null() {
        return 0;
    }
    catch_unwind(|| {
        let snapshot = snapshot_from_pod(unsafe { &*snapshot })?;
        let encoded = rmp_serde::to_vec_named(&snapshot).ok()?;
        if encoded.len() > out_size as usize {
            return None;
        }
        unsafe {
            ptr::copy_nonoverlapping(encoded.as_ptr(), out_buffer, encoded.len());
        }
        Some(encoded.len() as u32)
    })
    .ok()
    .flatten()
    .unwrap_or(0)
}

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
    use crate::{Map, Rect, Vec2, encode_celeste_map, encode_map};

    #[test]
    fn named_room_spawn_and_clone_are_independent() {
        let map = Map {
            bounds: Rect::new(320.0, -184.0, 320.0, 184.0),
            spawn: Vec2::new(344.0, -24.0),
            room_spawns: vec![Vec2::new(344.0, -24.0)],
            solids: vec![Rect::new(320.0, -8.0, 320.0, 8.0)],
            ..Map::default()
        };
        let bytes = encode_celeste_map(&map, "CelesteGymFfi", "chosen").unwrap();
        let room = b"chosen";
        let map_handle = unsafe {
            celeste_map_create_room(
                bytes.as_ptr(),
                bytes.len() as u32,
                room.as_ptr(),
                room.len() as u32,
            )
        };
        assert!(!map_handle.is_null());
        let original = unsafe { celeste_simulator_create_at_spawn(map_handle) };
        assert!(!original.is_null());
        assert_eq!(unsafe { &*original }.simulator.snapshot().pos, map.spawn);

        let branch = unsafe { celeste_simulator_clone(original) };
        assert!(!branch.is_null());
        let input = CelesteInputPod {
            move_x: 1,
            move_y: 0,
            flags: 0,
        };
        let mut states = [CelestePlayerPod::default(); 2];
        assert_eq!(
            unsafe { celeste_simulator_run_pod(branch, &input, 1, states.as_mut_ptr(), 2) },
            2
        );
        assert_eq!(unsafe { &*original }.simulator.snapshot().pos, map.spawn);

        unsafe {
            celeste_simulator_destroy(branch);
            celeste_simulator_destroy(original);
            celeste_map_destroy(map_handle);
        }
    }

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

    #[test]
    fn pod_context_matches_direct_simulation_and_exports_full_snapshot() {
        let map = Map {
            spawn: Vec2::new(24.0, 160.0),
            solids: vec![crate::Rect::new(0.0, 168.0, 320.0, 12.0)],
            ..Map::default()
        };
        let snapshot = PlayerSnapshot {
            pos: map.spawn,
            can_dream_dash: true,
            ..PlayerSnapshot::default()
        };
        let inputs = [
            CelesteInputPod {
                move_x: 1,
                move_y: 0,
                flags: 0b11,
            },
            CelesteInputPod {
                move_x: 1,
                move_y: 0,
                flags: 0b10,
            },
        ];
        let direct_inputs = inputs.map(CelesteInputPod::input);
        let expected = simulate(snapshot.clone(), &direct_inputs, &map, 2).unwrap();
        let map_bytes = encode_map(&map).unwrap();
        let snapshot_bytes = rmp_serde::to_vec_named(&snapshot).unwrap();
        let map_handle = unsafe { celeste_map_create(map_bytes.as_ptr(), map_bytes.len() as u32) };
        assert!(!map_handle.is_null());
        let simulator = unsafe {
            celeste_simulator_create_msgpack(
                snapshot_bytes.as_ptr(),
                snapshot_bytes.len() as u32,
                map_handle,
            )
        };
        assert!(!simulator.is_null());
        let mut states = [CelestePlayerPod::default(); 3];
        let written = unsafe {
            celeste_simulator_run_pod(
                simulator,
                inputs.as_ptr(),
                inputs.len() as u32,
                states.as_mut_ptr(),
                states.len() as u32,
            )
        };
        assert_eq!(written, 3);
        assert_eq!(states[2], CelestePlayerPod::from(&expected));

        let mut encoded = vec![0; celeste_snapshot_size() as usize];
        let encoded_len = unsafe {
            celeste_simulator_snapshot_msgpack(
                simulator,
                encoded.as_mut_ptr(),
                encoded.len() as u32,
            )
        };
        let exported: PlayerSnapshot =
            rmp_serde::from_slice(&encoded[..encoded_len as usize]).unwrap();
        assert_eq!(exported, expected);
        unsafe {
            celeste_simulator_destroy(simulator);
            celeste_map_destroy(map_handle);
        }
    }

    #[test]
    fn pod_layout_and_msgpack_conversion_are_stable() {
        assert_eq!(celeste_input_pod_size(), 3);
        assert_eq!(celeste_player_pod_size(), 56);
        let snapshot = PlayerSnapshot {
            state: PlayerState::Dash,
            pos: Vec2::new(12.0, 34.0),
            ducking: true,
            ..PlayerSnapshot::default()
        };
        let bytes = rmp_serde::to_vec_named(&snapshot).unwrap();
        let mut pod = CelestePlayerPod::default();
        assert_eq!(
            unsafe {
                celeste_player_pod_from_msgpack(bytes.as_ptr(), bytes.len() as u32, &mut pod)
            },
            1
        );
        assert_eq!(pod.state, PlayerState::Dash as u8);
        assert_eq!(pod.pos, snapshot.pos);
        assert_ne!(pod.flags & FLAG_DUCKING, 0);
        let mut encoded = vec![0; celeste_snapshot_size() as usize];
        let encoded_len = unsafe {
            celeste_player_pod_to_msgpack(&pod, encoded.as_mut_ptr(), encoded.len() as u32)
        };
        let decoded: PlayerSnapshot =
            rmp_serde::from_slice(&encoded[..encoded_len as usize]).unwrap();
        assert_eq!(CelestePlayerPod::from(&decoded), pod);
    }
}
