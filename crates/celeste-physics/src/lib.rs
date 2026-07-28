//! Pure, snapshot-driven Celeste physics research core.
//!
//! This crate intentionally reports its current fidelity level. It is deterministic and
//! source-informed, but is not yet a bit-identical replacement for every `Player.cs` state.

mod binary_packer;
mod ffi;
mod map;
mod map_fixture;
mod playground;
mod sim;
mod types;

pub use binary_packer::{
    BinaryElement, BinaryPackerWriteError, BinaryValue, encode_celeste_bin, parse_celeste_bin,
};
pub use map::{
    Entity, EntityKind, Map, MapEncodeError, Rect, decode_map, decode_map_room, encode_celeste_map,
    encode_map,
};
pub use map_fixture::{
    CelesteMapFixture, FixtureEntity, FixtureEntityKind, FixtureRect, FixtureRoom, FixtureVec2,
    MAP_FIXTURE_FORMAT_VERSION, MapFixtureError, MapPartFixture, RoomContribution,
    canonical_map_fixture_json, encode_map_fixture, merge_map_parts, parse_map_fixture,
};
pub use playground::{PLAYGROUND_PACKAGE, PLAYGROUND_ROOM, PLAYGROUND_SID, mechanics_playground};
pub use sim::{
    DT, Fidelity, INTENTIONALLY_UNSUPPORTED_STATES, SimulationError, SimulationResult, fidelity,
    simulate, simulate_trace,
};
pub use types::{
    BounceBlockSnapshot, CloudSnapshot, InputState, MoveBlockSnapshot, PlayerSnapshot, PlayerState,
    TheoCrystalSnapshot, Vec2, ZipMoverSnapshot,
};
