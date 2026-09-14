//! `vox-validation` — reads a pipeline run's provenance record, rebuilds the
//! pipeline from it, re-runs the fixture, and reports every tap's agreement
//! against the D5 tolerance bands; emits the Evidence-format summary the
//! Engineering Console's bundle carries (Plan v3 D11, Phase 2).
//!
//! Phase 2 fills this crate in; the workspace split lands it first so the
//! crate layout (D2) is settled before the Coral subtree arrives.

/// The crate is present; its API lands with the provenance record.
pub const CRATE: &str = env!("CARGO_PKG_NAME");
