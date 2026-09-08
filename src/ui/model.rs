//! Archive/session records and the per-capture result the Capture screen shows.

use super::*;

// ── model ────────────────────────────────────────────────────────────────────

// Struct-level serde(default): a session written by a build with fewer fields
// (or a future build's file read by this one) loads with defaults instead of
// failing deserialization and discarding the whole archive.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub(crate) struct Session {
    pub(super) id: String,
    pub(super) subj: String,
    pub(super) date: String,
    /// Identity coverage of this capture (see `CaptureResult::coverage_pct`);
    /// `None` on sessions saved before the statistic existed.
    pub(super) coverage_pct: Option<f32>,
    pub(super) f0: f32,
    /// Similarity vs the enrolled reference; `None` = unscorable (the capture
    /// and reference shared no comparable feature). Older archives stored a
    /// plain number, which loads as `Some`.
    pub(super) match_pct: Option<f32>,
    /// Measured over the capture (means / stop-time snapshot); `None` = not
    /// measured (e.g. too little voiced signal).
    pub(super) hnr_db: Option<f32>,
    pub(super) h1_h2_db: Option<f32>,
    pub(super) vibrato: Option<Vibrato>,
    pub(super) steadiness_cents: Option<f32>,
    pub(super) jitter_pct: Option<f32>,
    pub(super) shimmer_db: Option<f32>,
    pub(super) cpp_db: Option<f32>,
    pub(super) centroid_hz: Option<f32>,
    /// Mean relative harmonic profile (H1..H16, normalized to the strongest
    /// partial) over the capture; drives the Detail screen's Timbre caption.
    pub(super) profile: [f32; 16],
    /// Real measured formants; always `Some` now that the archive holds only
    /// real captures. (The session's voiceprint is derivable from these +
    /// `profile` + `centroid_hz` via `math::build_voiceprint` when needed.)
    pub(super) formants: Option<[Formant; 3]>,
    /// File name of the raw-audio export of this capture (in the capture
    /// directory), so a saved session can be joined to its WAV by the study
    /// harness. `None` when export was off or unavailable (web).
    pub(super) capture_file: Option<String>,
}

pub(super) struct CaptureResult {
    pub(super) match_pct: Option<f32>,
    /// Mean f0 over the capture's voiced frames; `None` = no voiced signal
    /// (nothing is fabricated for the readout, and the capture can't be saved).
    pub(super) f0: Option<f32>,
    pub(super) hnr_db: Option<f32>,
    pub(super) h1_h2_db: Option<f32>,
    pub(super) vibrato: Option<Vibrato>,
    pub(super) steadiness_cents: Option<f32>,
    pub(super) jitter_pct: Option<f32>,
    pub(super) shimmer_db: Option<f32>,
    pub(super) cpp_db: Option<f32>,
    pub(super) centroid_hz: Option<f32>,
    pub(super) profile: [f32; 16],
    pub(super) formants: Option<[Formant; 3]>,
    pub(super) voiceprint: Voiceprint,
    /// Identity coverage: % of the capture's voiced frames that survived
    /// every identity gate (f0 ≤ 200 Hz, off harmonic suspect bands,
    /// SNR ≥ 30 dB). The research-protocol "coverage" statistic: low error
    /// on 10% of frames is a different claim from low error on 90%.
    pub(super) coverage_pct: Option<f32>,
    /// True when no reference was enrolled yet AND this capture passed the
    /// enrollment gate: saving it enrolls it as the reference (shown as
    /// "Reference" rather than a similarity score).
    pub(super) is_reference: bool,
    /// `Some(reason)` = this capture may not be saved (no voiced signal, or a
    /// would-be reference that failed the enrollment gate). The Save button is
    /// withheld and the reason shown instead; `save_session` double-checks.
    pub(super) blocked: Option<String>,
}

/// The real local date a session is saved ("2026-07-02"), stored on the
/// session so reloaded archives show when each capture actually happened.
/// Web builds have no clock access via chrono (and no persistence), so they
/// keep the ephemeral "Today".
pub(super) fn today_string() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    }
    #[cfg(target_arch = "wasm32")]
    {
        "Today".to_string()
    }
}
