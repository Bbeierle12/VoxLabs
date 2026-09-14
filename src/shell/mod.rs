//! The shell side shared by the phone and the desktop (Plan v3 D9: the
//! pipeline is shell-agnostic; this is the thin layer that owns a runner
//! on the live audio stream). `engine` starts, switches and retires the
//! runner by mode name; `consumers` is the hop observer that feeds the
//! egui screens and the frame-cadence passes.
//!
//! Phase 5a decision (`DECISIONS.md`): the phone's shell is the existing
//! egui NativeActivity build, not Tauri; the desktop runs the same runner
//! (D15: the GPU engine is gone).

pub mod consumers;
pub mod engine;
