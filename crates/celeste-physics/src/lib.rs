//! Pure, snapshot-driven Celeste physics research core.
//!
//! This crate intentionally reports its current fidelity level. It is deterministic and
//! source-informed, but is not yet a bit-identical replacement for every `Player.cs` state.

mod binary_packer;
mod ffi;
mod map;
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
pub use playground::{PLAYGROUND_PACKAGE, PLAYGROUND_ROOM, PLAYGROUND_SID, mechanics_playground};
pub use sim::{
    DT, Fidelity, INTENTIONALLY_UNSUPPORTED_STATES, SimulationError, SimulationResult, fidelity,
    simulate, simulate_trace,
};
pub use types::{
    InputState, PlayerSnapshot, PlayerState, TheoCrystalSnapshot, Vec2, ZipMoverSnapshot,
};
