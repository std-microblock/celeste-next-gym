//! Native, deterministic input-space search over [`celeste_physics`].
//!
//! The crate deliberately keeps configuration parsing, expression compilation,
//! input scheduling, and simulation in one native API.  It is suitable for
//! command-line tooling and tests; browser bindings live elsewhere.

mod engine;
mod model;
mod search;

pub use model::{
    CandidateResult, ConnectedRegion, CoverageEntry, CoverageReport, ExactWindow, FrameInterval,
    FuzzError, FuzzResult, FuzzSpec, HoldTime, InputDeclaration, JumpHoldPolicy, Limits, Objective,
    ObjectiveKind, OutputMode, Range, RegionSummary, SearchOptions, SearchSpec, SearchStats,
    Variable, VerifiedInput,
};
pub use search::{CompiledFuzz, compile, parse_spec};
