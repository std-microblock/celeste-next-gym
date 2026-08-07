//! Pure, snapshot-driven Celeste physics research core.
//!
//! This crate intentionally reports its current fidelity level. It is deterministic and
//! source-informed, but is not yet a bit-identical replacement for every `Player.cs` state.

mod binary_packer;
mod entity_decode;
mod ffi;
mod map;
mod map_fixture;
mod playground;
mod sim;
mod types;

pub use ffi::{
    CelesteInputPod, CelesteMapHandle, CelestePlayerPod, CelesteRuntimeEntityPod,
    CelesteSimulatorHandle, celeste_input_pod_size, celeste_map_create, celeste_map_create_room,
    celeste_map_destroy, celeste_player_pod_from_msgpack, celeste_player_pod_size,
    celeste_player_pod_to_msgpack, celeste_runtime_entity_pod_size, celeste_simulator_clone,
    celeste_simulator_create_at_spawn, celeste_simulator_create_msgpack,
    celeste_simulator_create_pod, celeste_simulator_destroy, celeste_simulator_run_pod,
    celeste_simulator_runtime_entities, celeste_simulator_snapshot_msgpack,
};

pub use binary_packer::{
    BinaryElement, BinaryPackerWriteError, BinaryValue, encode_celeste_bin, parse_celeste_bin,
};
pub use map::{
    CelesteRoomAudit, Entity, EntityKind, Map, MapEncodeError, Rect, RoomRuntime,
    audit_celeste_map, celeste_map_rooms, decode_map, decode_map_room, decode_map_room_local,
    encode_celeste_map, encode_map,
};
pub use map_fixture::{
    CelesteMapFixture, FixtureEntity, FixtureEntityKind, FixtureRect, FixtureRoom, FixtureVec2,
    MAP_FIXTURE_FORMAT_VERSION, MapFixtureError, MapPartFixture, RoomContribution,
    canonical_map_fixture_json, encode_map_fixture, merge_map_parts, parse_map_fixture,
};
pub use playground::{PLAYGROUND_PACKAGE, PLAYGROUND_ROOM, PLAYGROUND_SID, mechanics_playground};
pub use sim::{
    DT, Fidelity, INTENTIONALLY_UNSUPPORTED_STATES, SimulationError, SimulationResult, Simulator,
    fidelity, simulate, simulate_trace,
};
pub use types::{
    BounceBlockSnapshot, BumperSnapshot, CassetteBlockSnapshot, CassetteManagerSnapshot,
    CloudSnapshot, CoreMode, ExitBlockSnapshot, FallingBlockSnapshot, GliderSnapshot,
    HeartGemSnapshot, InputState, LookoutSnapshot, MoveBlockSnapshot, PlayerSnapshot, PlayerState,
    RefillSnapshot, RisingLavaSnapshot, SandwichLavaSnapshot, SeekerSnapshot, SpinnerSnapshot,
    TempleGateSnapshot, TheoCrystalSnapshot, Vec2, ZipMoverSnapshot,
};
