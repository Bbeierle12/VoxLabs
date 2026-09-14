//! The analysis pipeline — Plan v3 §2–§5, Phase 1 (the walking skeleton).
//!
//! * [`types`] — the §2 type universe: every wire type the runner will
//!   connect, and the Phase-1 payloads (`AudioFrame`, `F0Track`,
//!   `FormantTrack`, `TractParams`, `AreaFunction`).
//! * [`stage`] — the `Stage` contract (§3) and its type-erased runner form.
//! * [`definition`] — the per-mode pipeline file (`pipelines/live_model.toml`):
//!   stream format, ordered stage list with backend ids, taps, runner limits,
//!   and parameter overrides on top of `pipeline.toml`.
//! * [`builder`] — validates every wire against the §2 table, names the
//!   adapter chain when one is missing, instantiates stages, preallocates.
//! * [`tap`] — bounded channel from the worker to the shell.
//! * [`runner`] — the worker thread: drains the audio ring, re-frames it to
//!   the hop itself (D6, cpal #902), runs the stages per hop with per-stage
//!   timing and deadline accounting, publishes taps.
//! * [`stages`] — the wrapped kernels: YIN, LPC/Levinson, the grid inverse,
//!   the Story tract. Wrappers, not rewrites (§3).
//!
//! Not on wasm: the runner is a thread and stamps hops with `Instant`, and
//! the web target has no analysis thread yet (Phase 5b).

pub mod builder;
pub mod definition;
pub mod stage;
pub mod stages;
pub mod tap;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod runner;
// The Phase 1 contract checks as functions: host tests and the phone's
// self-test run the same bodies.
#[cfg(not(target_arch = "wasm32"))]
pub mod contract;

#[cfg(test)]
mod tests;

pub use builder::{Pipeline, WiringError, build};
pub use definition::{DefinitionError, PipelineDefinition, RunnerConfig};
pub use stage::{Stage, StageError, StageIdent, StreamFormat};
pub use types::{AreaFunction, AudioFrame, F0Track, FormantTrack, TractParams, Wire, WireType};
