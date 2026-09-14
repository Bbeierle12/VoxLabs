//! Coral's choir branch in Rust (Plan v3 D4, Phase 4): the STFT, the
//! multi-F0 note detector with harmonic cancellation, the SATB section
//! labeler, the QIFFT single-F0 estimator, the rehearsal harmony maths
//! and its stateful card analyser, and the synthetic SATB chord generator
//! the contracts measure against. Ported from `apps/coral/src/audio/`
//! with Coral's own tests and librosa oracle fixtures as the contracts;
//! the stages in `pipeline::stages::{coral_stft, qifft, multi_f0, satb}`
//! wrap the kernels.

pub mod cards;
mod family;
pub mod harmony;
pub mod intervals;
pub mod labeler;
pub mod note_detector;
pub mod pipeline;
pub mod qifft;
pub mod stft;
pub mod synth;
