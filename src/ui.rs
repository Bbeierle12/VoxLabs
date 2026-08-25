//! VoxLab dashboard UI — native egui implementation of the claude.ai/design
//! prototype `VoxLab Prototype.dc.html` (project 4c72096c).
//!
//! Four screens — Overview, Capture, Sessions, Session detail — plus a
//! floating tab bar, in the light "sterile lab" glass style. egui has no
//! backdrop blur, so the glass is approximated with translucent white fills,
//! hairline strokes and soft shadows.
//!
//! Data policy: everything shown is REAL. Live acquisition readouts come from
//! the analysis engine (f0/formants) and the audio callback's RMS telemetry;
//! HNR/jitter/shimmer/CPP are measured per capture; the session archive holds
//! only real captures; and match scoring is a deterministic classical
//! voiceprint similarity against the enrolled reference (see `math::
//! voiceprint_similarity`) — a voice-consistency measure, not a forensic
//! biometric. The archive persists to disk via the `persist` module.

use crate::concurrency::{AnalysisState, EngineEvent, Telemetry};
use crate::types::{Formant, MAX_PARTIALS, Vibrato, VocalProfile, Voiceprint};
use eframe::egui::{
    self, Align, Align2, Color32, ColorImage, FontId, Layout, Pos2, Rect, RichText, Sense, Shape,
    Stroke, StrokeKind, TextureHandle, TextureOptions, pos2, vec2,
};
use rtrb::Producer;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use triple_buffer::Output;

// ── design tokens ────────────────────────────────────────────────────────────

const INK: Color32 = Color32::from_rgb(11, 43, 49); // #0B2B31
const TEAL: Color32 = Color32::from_rgb(14, 148, 136); // #0E9488
const TEAL_DARK: Color32 = Color32::from_rgb(15, 118, 110); // #0F766E
const CYAN_DEEP: Color32 = Color32::from_rgb(8, 145, 178); // #0891B2
const AMBER_TEXT: Color32 = Color32::from_rgb(180, 83, 9); // #B45309
const AMBER: Color32 = Color32::from_rgb(217, 119, 6); // #D97706
const BG_BASE: Color32 = Color32::from_rgb(234, 243, 245); // between #E3EBEE / #F2F8FA

/// Sessions scoring at or above this are "Verified", below are "Flagged"
/// (the prototype's `matchThreshold` prop, default 85).
const MATCH_THRESHOLD: f32 = 85.0;

/// Minimum wall-clock length and voiced fraction a capture needs before it may
/// enroll as the reference voiceprint — the Capture hint's "8 s min", enforced.
/// A bad reference poisons every later match, so enrollment is gated harder
/// than ordinary session saves.
const MIN_ENROLL_SECS: f64 = 8.0;
const MIN_ENROLL_VOICED_FRACTION: f32 = 0.5;

/// Recording auto-stops here so a capture left running off-screen can't grow
/// its accumulators unbounded.
const MAX_REC_SECS: f64 = 60.0;

/// How long the analysis thread may go without publishing a frame — while the
/// microphone is demonstrably live — before the UI calls it stalled. One
/// analysis frame is ~46 ms, so 2 s is ~40 missed frames: far past scheduling
/// jitter, well short of a length a user would sit through wondering.
const ANALYSIS_STALL_SECS: f64 = 2.0;

/// Input RMS above which the microphone counts as delivering signal, for the
/// staleness check above. A silent room reads well below this; a stalled
/// analysis thread with a live mic reads well above it.
const ANALYSIS_STALL_RMS_FLOOR: f32 = 0.002;

/// Tract-view geometry: section index where the 90° velar bend begins and
/// ends (of 44, glottis to lips), display height, and the smoothing rates
/// for the mode coefficients (fast enough to track a vowel change, slow
/// enough not to jitter) and the VTL estimate (anatomy: accumulate slowly).
const TRACT_BEND_START: usize = 14;
const TRACT_BEND_END: usize = 30;
const TRACT_VIEW_H: f32 = 190.0;
const TRACT_Q_EMA_ALPHA: f32 = 0.2;
const VTL_EMA_ALPHA: f32 = 0.05;

/// Articulatory-log ring: capacity and decimation. 4096 samples at one per
/// 150 ms is ~10 minutes of continuous fully-gated phonation; a fixed cap
/// keeps a day-long session's memory bounded.
const TRACT_LOG_CAP: usize = 4096;
const TRACT_LOG_MIN_DT: f64 = 0.15;
/// Trail dots drawn in the vowel map (newest of the log; older ones fade).
const TRACT_MAP_TRAIL: usize = 240;

/// How long a cpal stream error keeps its banner up after the last occurrence.
/// Stream errors are often transient, so the notice expires on its own rather
/// than pinning a warning for the rest of the session.
const AUDIO_ERROR_NOTICE_SECS: f64 = 10.0;

/// Content column width — the prototype is a 428 px phone layout.
const COL_WIDTH: f32 = 430.0;

/// Content inset of a `glass` card: its 1 px stroke plus 14 px inner margin.
/// The Sessions header row is *not* inside a card, so it pads by this much on
/// each side to sit on the same grid as the rows below it.
const GLASS_INSET: f32 = 15.0;

/// Sessions-table column grid. The header and the data rows are laid out by
/// separate code — the header on the panel, each row inside a `glass` card —
/// so the widths live here instead of as literals in both places, where they
/// had drifted 4 px apart and left the SUBJECT column visibly misaligned with
/// its own header.
const SESSIONS_ID_W: f32 = 78.0;
const SESSIONS_F0_W: f32 = 58.0;
/// Space reserved at the right for the match badge, which sizes itself to its
/// text; this is the room the flexible SUBJECT column must leave it.
const SESSIONS_MATCH_W: f32 = 68.0;

/// Width of the flexible SUBJECT column from the width available at that point
/// in the layout. `inset_right` is the card inset the header must give back and
/// the rows must not — the card already supplies it. Both call sites use this
/// so the grid has exactly one definition.
fn sessions_subject_w(available: f32, inset_right: f32) -> f32 {
    available - SESSIONS_F0_W - SESSIONS_MATCH_W - inset_right
}

/// Safe-area padding for Android's edge-to-edge rendering (status bar /
/// gesture bar). Zero elsewhere: desktop and web windows are not overlaid.
const TOP_INSET: f32 = if cfg!(target_os = "android") {
    40.0
} else {
    0.0
};
const BOTTOM_INSET: f32 = if cfg!(target_os = "android") {
    18.0
} else {
    0.0
};

// ── spectrogram waterfall ─────────────────────────────────────────────────────

/// Which visualization fills the switchable region *below* the always-present
/// harmonic ladder.
#[derive(PartialEq, Clone, Copy)]
enum VizMode {
    Radial,
    Spectrogram,
    Scope,
    /// Story two-mode tract model, inverted live from measured formants.
    Tract,
}

/// Waterfall grid: time on X (newest at right), log-frequency on Y.
const SPEC_TIME_COLS: usize = 320;
const SPEC_FREQ_ROWS: usize = 200;
const SPEC_FMIN_HZ: f32 = 50.0; // matches YIN_F0_MIN
const SPEC_FMAX_HZ: f32 = 5000.0; // matches the formant search band ceiling
const SPEC_DB_FLOOR: f32 = -90.0;
const SPEC_DB_CEIL: f32 = -10.0;

/// How one waterfall display-row samples the FFT magnitude bins. When the row
/// spans ≥ 1 bin, take the max (peaks survive); when narrower than a bin,
/// linearly interpolate. Ported from Resonator's `ColMap`.
#[derive(Clone, Copy)]
struct BinSpan {
    lo: usize,
    hi: usize,
    frac: f32,
}

impl BinSpan {
    fn sample(&self, bins: &[f32]) -> f32 {
        if self.hi > self.lo {
            bins[self.lo..=self.hi]
                .iter()
                .cloned()
                .fold(f32::NEG_INFINITY, f32::max)
        } else {
            let a = bins[self.lo];
            let b = bins[(self.lo + 1).min(bins.len().saturating_sub(1))];
            a + (b - a) * self.frac
        }
    }
}

fn spec_db_to_byte(db: f32) -> u8 {
    let v = ((db - SPEC_DB_FLOOR) / (SPEC_DB_CEIL - SPEC_DB_FLOOR)).clamp(0.0, 1.0);
    (v * 255.0) as u8
}

/// Viridis 6th-degree polynomial colormap fit (Matt Zucker, shadertoy WlfXRN)
/// — a dependency-free perceptual ramp, ported from Resonator. Reads well on
/// the light glass background (dark-purple low end contrasts cleanly).
#[rustfmt::skip]
const VIRIDIS: [[f32; 3]; 7] = [
    [0.27772733, 0.0054073445, 0.334_099_8],
    [0.10509304, 1.4046135, 1.3845902],
    [-0.33086183, 0.21484756, 0.095095163],
    [-4.6342305, -5.799_101, -19.332441],
    [6.228_27, 14.179933, 56.690_55],
    [4.776_385, -13.745145, -65.353033],
    [-5.435_456, 4.6458526, 26.312435],
];

fn build_heat_lut() -> [Color32; 256] {
    let mut lut = [Color32::BLACK; 256];
    for (v, slot) in lut.iter_mut().enumerate() {
        let t = v as f32 / 255.0;
        let mut c = VIRIDIS[6];
        for coef in VIRIDIS.iter().take(6).rev() {
            c = [coef[0] + t * c[0], coef[1] + t * c[1], coef[2] + t * c[2]];
        }
        *slot = Color32::from_rgb(
            (c[0].clamp(0.0, 1.0) * 255.0) as u8,
            (c[1].clamp(0.0, 1.0) * 255.0) as u8,
            (c[2].clamp(0.0, 1.0) * 255.0) as u8,
        );
    }
    lut
}

/// Build the row→bin map for the log-frequency Y axis: row 0 is the top
/// (SPEC_FMAX_HZ), the last row is the bottom (SPEC_FMIN_HZ).
fn build_freq_row_map(sample_rate: f32) -> Vec<BinSpan> {
    let n = crate::spectrogram::N_BINS;
    let nyquist = sample_rate * 0.5;
    let to_bin = |hz: f32| hz * (n - 1) as f32 / nyquist.max(1.0);
    let ratio = SPEC_FMAX_HZ / SPEC_FMIN_HZ;
    let rows = SPEC_FREQ_ROWS;
    let half_step = ratio.powf(0.5 / (rows - 1).max(1) as f32);
    (0..rows)
        .map(|r| {
            let t = r as f32 / (rows - 1).max(1) as f32; // 0 top .. 1 bottom
            let fc = SPEC_FMIN_HZ * ratio.powf(1.0 - t); // high at top, low at bottom
            let bc = to_bin(fc);
            let blo = to_bin(fc / half_step);
            let bhi = to_bin(fc * half_step);
            if bhi - blo >= 1.0 {
                let lo = (blo.floor().max(0.0) as usize).min(n - 1);
                let hi = (bhi.ceil() as usize).min(n - 1);
                BinSpan { lo, hi, frac: 0.0 }
            } else {
                let lo = (bc.floor().max(0.0) as usize).min(n.saturating_sub(2));
                BinSpan {
                    lo,
                    hi: lo,
                    frac: (bc - lo as f32).clamp(0.0, 1.0),
                }
            }
        })
        .collect()
}

fn ink(a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(11, 43, 49, a)
}
fn white(a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, a)
}
fn teal_a(a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(14, 148, 136, a)
}

/// Translucent white "glass" card frame (blur approximated by opacity).
fn glass(corner: f32) -> egui::Frame {
    egui::Frame::default()
        .fill(white(168))
        .stroke(Stroke::new(1.0, white(230)))
        .corner_radius(corner)
        .inner_margin(14.0)
}

// ── model ────────────────────────────────────────────────────────────────────

// Struct-level serde(default): a session written by a build with fewer fields
// (or a future build's file read by this one) loads with defaults instead of
// failing deserialization and discarding the whole archive.
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub(crate) struct Session {
    id: String,
    subj: String,
    date: String,
    f0: f32,
    /// Similarity vs the enrolled reference; `None` = unscorable (the capture
    /// and reference shared no comparable feature). Older archives stored a
    /// plain number, which loads as `Some`.
    match_pct: Option<f32>,
    /// Measured over the capture (means / stop-time snapshot); `None` = not
    /// measured (e.g. too little voiced signal).
    hnr_db: Option<f32>,
    h1_h2_db: Option<f32>,
    vibrato: Option<Vibrato>,
    steadiness_cents: Option<f32>,
    jitter_pct: Option<f32>,
    shimmer_db: Option<f32>,
    cpp_db: Option<f32>,
    centroid_hz: Option<f32>,
    /// Mean relative harmonic profile (H1..H16, normalized to the strongest
    /// partial) over the capture; drives the Detail screen's Timbre caption.
    profile: [f32; 16],
    /// Real measured formants; always `Some` now that the archive holds only
    /// real captures. (The session's voiceprint is derivable from these +
    /// `profile` + `centroid_hz` via `math::build_voiceprint` when needed.)
    formants: Option<[Formant; 3]>,
}

struct CaptureResult {
    match_pct: Option<f32>,
    /// Mean f0 over the capture's voiced frames; `None` = no voiced signal
    /// (nothing is fabricated for the readout, and the capture can't be saved).
    f0: Option<f32>,
    hnr_db: Option<f32>,
    h1_h2_db: Option<f32>,
    vibrato: Option<Vibrato>,
    steadiness_cents: Option<f32>,
    jitter_pct: Option<f32>,
    shimmer_db: Option<f32>,
    cpp_db: Option<f32>,
    centroid_hz: Option<f32>,
    profile: [f32; 16],
    formants: Option<[Formant; 3]>,
    voiceprint: Voiceprint,
    /// True when no reference was enrolled yet AND this capture passed the
    /// enrollment gate: saving it enrolls it as the reference (shown as
    /// "Reference" rather than a similarity score).
    is_reference: bool,
    /// `Some(reason)` = this capture may not be saved (no voiced signal, or a
    /// would-be reference that failed the enrollment gate). The Save button is
    /// withheld and the reason shown instead; `save_session` double-checks.
    blocked: Option<String>,
}

/// The real local date a session is saved ("2026-07-02"), stored on the
/// session so reloaded archives show when each capture actually happened.
/// Web builds have no clock access via chrono (and no persistence), so they
/// keep the ephemeral "Today".
fn today_string() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        chrono::Local::now().format("%Y-%m-%d").to_string()
    }
    #[cfg(target_arch = "wasm32")]
    {
        "Today".to_string()
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Screen {
    Overview,
    Capture,
    Sessions,
    Detail,
}

#[derive(PartialEq, Clone, Copy)]
enum Filter {
    All,
    Verified,
    Flagged,
}

enum RecState {
    Idle,
    Recording { start: f64 },
    Analyzing { start: f64, elapsed: f64 },
    Done { elapsed: f64 },
}

// ── app ──────────────────────────────────────────────────────────────────────

pub struct DashboardApp {
    /// Reserved: the VoxLab prototype has no engine-control surface yet, so no
    /// events are sent; kept so the entry points' wiring stays unchanged.
    #[allow(dead_code)]
    event_tx: Producer<EngineEvent>,
    telemetry: Arc<Telemetry>,
    ui_profile_rx: Output<VocalProfile>,
    current_profile: VocalProfile,

    screen: Screen,
    filter: Filter,
    rec: RecState,
    sessions: Vec<Session>,
    selected: Option<usize>,
    result: Option<CaptureResult>,
    export_queued: bool,

    /// Waveform bar half-heights (px), scrolling left; length 110.
    wave: Vec<f32>,
    /// EMA-smoothed harmonic amplitudes for the ladder (profiles arrive every
    /// ~46 ms; raw bars at repaint rate read as flicker, not music).
    harm_ema: [f32; MAX_PARTIALS],
    /// Real f0 samples collected while recording (mean shown in the result).
    rec_f0_acc: Vec<f32>,
    /// Update ticks while recording (voiced or not); with `rec_f0_acc.len()`
    /// this gives the capture's voiced fraction for the enrollment gate.
    rec_frames_total: u32,
    /// Two-tap confirm state for the Overview card's re-enroll control.
    reenroll_armed: bool,
    /// User-facing persistence problem (unreadable archive at startup, failed
    /// save). Shown as a dismissible banner above every screen.
    persist_notice: Option<String>,
    /// Analysis-thread staleness watch: the `analysis_frames` count last seen
    /// and when (`None` until the first observation, so a slow start isn't
    /// mistaken for a stall). `analysis_stalled` is the derived verdict.
    last_analysis_frames: u32,
    last_analysis_change: Option<f64>,
    analysis_stalled: bool,
    /// Latest cpal stream error as (message, time seen); expires after
    /// [`AUDIO_ERROR_NOTICE_SECS`]. The counters behind it only ever rise, so
    /// the UI watches for *changes* rather than for a nonzero value.
    audio_notice: Option<(&'static str, f64)>,
    last_input_errors: u32,
    last_output_errors: u32,
    /// Real per-frame metrics collected while recording (means stored).
    rec_hnr_acc: Vec<f32>,
    rec_h1h2_acc: Vec<f32>,
    rec_jitter_acc: Vec<f32>,
    rec_shimmer_acc: Vec<f32>,
    rec_cpp_acc: Vec<f32>,
    rec_centroid_acc: Vec<f32>,
    /// Running sum + frame count for the mean relative harmonic profile over
    /// the capture (snapshots of `harm_ema`, normalized to its own max).
    rec_profile_sum: [f32; 16],
    rec_profile_n: u32,
    /// Last display-grade formants seen while recording — measured at
    /// f0 ≤ 350 Hz, off the harmonic suspect bands (`math::formant_grade`).
    /// These reach the session's Detail chips but never the voiceprint.
    rec_formants: Option<[Formant; 3]>,
    /// Identity-grade formant frequencies accumulated over the capture, one
    /// vec per slot (only frames measured at f0 ≤ 200 Hz, off the harmonic
    /// suspect bands, contribute — above that, LPC's harmonic attraction
    /// makes the estimate re-encode pitch, and a similarity score built on
    /// it compares f0, not voices; Chen, Whalen & Shadle 2019). The
    /// voiceprint takes each slot's *median*: a long-term-distribution-style
    /// central tendency, robust to the odd stray fit, instead of whatever
    /// frame happened to be last.
    rec_id_f_acc: [Vec<f32>; 3],
    /// Per-frame vocal-tract-length estimates over the capture, identity-
    /// grade frames only; the voiceprint stores the median.
    rec_vtl_acc: Vec<f32>,
    /// Display-smoothed live readouts (raw values update every ~46 ms and
    /// flicker as digits). `None` = unvoiced/unknown.
    hnr_disp: Option<f32>,
    h1h2_disp: Option<f32>,
    jitter_disp: Option<f32>,
    shimmer_disp: Option<f32>,
    cpp_disp: Option<f32>,
    centroid_disp: Option<f32>,
    /// A2/A1 in dB (see [`crate::math::a2_a1_db`]) — Bozeman's observable
    /// for the acoustic passaggio — display-smoothed. Positive = H2 dominant
    /// (open timbre), negative = H1 dominant (turned over).
    turnover_disp: Option<f32>,

    /// Which visualization the analyzer card's switchable region shows.
    viz_mode: VizMode,
    /// Displayed tract-model mode coefficients (q1, q2), display-smoothed.
    /// `None` until the first successful inversion; the view then draws the
    /// model's neutral shape, greyed, so the card has context without
    /// claiming data.
    tract_q: Option<(f32, f32)>,
    /// Whether `tract_q` was updated from the current frame. False = the
    /// shape on screen is HELD from the last frame whose formants passed the
    /// reliability gate — drawn grey, never animated.
    tract_live: bool,
    /// Running articulatory log: (time, q1, q2) samples appended whenever a
    /// frame clears every gate, across the whole session (not only while
    /// recording). Capped ring; ~10 min of continuous fully-gated phonation
    /// at the decimated rate. In-memory only for now.
    tract_log: std::collections::VecDeque<(f64, f32, f32)>,
    tract_log_last_t: f64,
    /// Slow EMA of the per-frame vocal-tract-length estimate, cm. Fed only
    /// by identity-grade frames (see `tract::vtl_from_formants`); scales the
    /// model and captions the display. Expect ±1+ cm accuracy, never show
    /// more than one decimal.
    vtl_est_cm: Option<f32>,
    /// Spectrogram magnitudes (dB) from the analysis thread; raw waveform for
    /// the oscilloscope; and the mic sample rate for bin→Hz mapping.
    spectrum_rx: Output<Vec<f32>>,
    scope_rx: Output<Vec<f32>>,
    sample_rate: f32,
    /// Waterfall state (reused buffers; see the spectrogram tokens block).
    wf_scratch: Vec<f32>,
    wf_freq_map: Vec<BinSpan>,
    wf_cols: Vec<Vec<u8>>,
    wf_head: usize,
    wf_tex: Option<TextureHandle>,
    wf_pixels: Vec<u8>,
    wf_lut: [Color32; 256],

    /// Enrolled reference voiceprint (set by the first saved capture). Every
    /// later capture's match % is a real similarity against this. `None` until
    /// enrollment. Persisted to `store_path` and reloaded at startup.
    enrolled: Option<Voiceprint>,
    /// Short human ID for the enrolled reference (e.g. "V-3F9A").
    enrolled_id: Option<String>,

    next_session_num: u32,
    /// Where the reference + archive are persisted (JSON). `None` on web (no
    /// disk) — the archive then lives only for the session.
    store_path: Option<PathBuf>,
    rng: u64,
}

impl DashboardApp {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        event_tx: Producer<EngineEvent>,
        telemetry: Arc<Telemetry>,
        ui_profile_rx: Output<VocalProfile>,
        spectrum_rx: Output<Vec<f32>>,
        scope_rx: Output<Vec<f32>>,
        sample_rate: f32,
        store_path: Option<PathBuf>,
    ) -> Self {
        let mut visuals = egui::Visuals::light();
        visuals.override_text_color = Some(INK);
        visuals.panel_fill = BG_BASE;
        visuals.window_fill = BG_BASE;
        cc.egui_ctx.set_visuals(visuals);

        // Real archive: the enrolled reference + saved captures persist to disk
        // and are reloaded here. A never-saved (or web) archive comes back
        // empty, with the first capture enrolling the reference. Every session
        // is a real capture with a real match score — no seeded demo rows.
        // An unreadable archive is backed up by `load` and reported via the
        // notice banner rather than silently discarded.
        let loaded = store_path
            .as_deref()
            .map(crate::persist::load)
            .unwrap_or_default();
        let saved = loaded.state;
        let persist_notice = loaded.notice;
        Self {
            event_tx,
            telemetry,
            ui_profile_rx,
            current_profile: VocalProfile::default(),
            screen: Screen::Overview,
            filter: Filter::All,
            rec: RecState::Idle,
            sessions: saved.sessions,
            selected: None,
            result: None,
            export_queued: false,
            wave: vec![2.0; 110],
            harm_ema: [0.0; MAX_PARTIALS],
            rec_f0_acc: Vec::new(),
            rec_frames_total: 0,
            reenroll_armed: false,
            persist_notice,
            last_analysis_frames: 0,
            last_analysis_change: None,
            analysis_stalled: false,
            audio_notice: None,
            last_input_errors: 0,
            last_output_errors: 0,
            rec_hnr_acc: Vec::new(),
            rec_h1h2_acc: Vec::new(),
            rec_jitter_acc: Vec::new(),
            rec_shimmer_acc: Vec::new(),
            rec_cpp_acc: Vec::new(),
            rec_centroid_acc: Vec::new(),
            rec_profile_sum: [0.0; 16],
            rec_profile_n: 0,
            rec_formants: None,
            rec_id_f_acc: [Vec::new(), Vec::new(), Vec::new()],
            rec_vtl_acc: Vec::new(),
            hnr_disp: None,
            h1h2_disp: None,
            jitter_disp: None,
            shimmer_disp: None,
            cpp_disp: None,
            centroid_disp: None,
            turnover_disp: None,

            viz_mode: VizMode::Spectrogram,
            tract_q: None,
            tract_live: false,
            tract_log: std::collections::VecDeque::new(),
            tract_log_last_t: 0.0,
            // Seed the tract-length estimate from the enrolled reference:
            // anatomy carries across sessions, so the returning singer's
            // model starts at their calibration instead of the default.
            vtl_est_cm: saved
                .enrolled
                .map(|vp| vp.vtl_cm)
                .filter(|&l| l > 0.0 && l.is_finite()),
            spectrum_rx,
            scope_rx,
            sample_rate,
            wf_scratch: vec![crate::spectrogram::DB_FLOOR; crate::spectrogram::N_BINS],
            wf_freq_map: build_freq_row_map(sample_rate),
            wf_cols: vec![vec![0u8; SPEC_FREQ_ROWS]; SPEC_TIME_COLS],
            wf_head: 0,
            wf_tex: None,
            wf_pixels: vec![0u8; SPEC_TIME_COLS * SPEC_FREQ_ROWS * 4],
            wf_lut: build_heat_lut(),

            enrolled: saved.enrolled,
            enrolled_id: saved.enrolled_id,

            next_session_num: saved.next_session_num,
            store_path,
            rng: 0x9E37_79B9_7F4A_7C15,
        }
    }

    /// Write the current reference + archive to `store_path` (no-op on web).
    /// Called whenever the archive changes. A failed write surfaces in the
    /// notice banner — the UI must never claim a save that didn't stick.
    fn persist_state(&mut self) {
        if let Some(path) = &self.store_path {
            let state = crate::persist::ArchiveState {
                version: crate::persist::ARCHIVE_VERSION,
                enrolled: self.enrolled,
                enrolled_id: self.enrolled_id.clone(),
                next_session_num: self.next_session_num,
                sessions: self.sessions.clone(),
            };
            if let Err(msg) = crate::persist::save(path, &state) {
                self.persist_notice = Some(msg);
            }
        }
    }

    fn rand01(&mut self) -> f32 {
        // xorshift64 — no rng dependency, prototype-grade randomness.
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u64 << 24) as f32
    }

    fn verified(match_pct: Option<f32>) -> bool {
        match_pct.is_some_and(|m| m >= MATCH_THRESHOLD)
    }

    fn badge_style(match_pct: Option<f32>) -> (&'static str, Color32, Color32) {
        match match_pct {
            Some(_) if Self::verified(match_pct) => ("Verified", teal_a(31), TEAL_DARK),
            Some(_) => (
                "Flagged",
                Color32::from_rgba_unmultiplied(217, 119, 6, 36),
                AMBER_TEXT,
            ),
            // No comparable features between capture and reference: neither
            // verified nor flagged — there is simply no score.
            None => (
                "Unscored",
                Color32::from_rgba_unmultiplied(107, 114, 128, 36),
                ink(150),
            ),
        }
    }

    /// Match score for display: one decimal, or an em-dash when unscorable.
    fn match_text(match_pct: Option<f32>) -> String {
        match_pct.map_or_else(|| "—".into(), |m| format!("{m:.1}"))
    }

    fn input_rms(&self) -> f32 {
        f32::from_bits(self.telemetry.input_rms.load(Ordering::Relaxed))
    }

    // ── engine health ────────────────────────────────────────────────────────

    /// Polls the engine's telemetry once per frame and updates the two derived
    /// conditions the UI can't read directly from a flag: an analysis thread
    /// that is alive but no longer producing, and a cpal stream error recent
    /// enough to still be worth showing.
    fn watch_engine(&mut self, now: f64) {
        let frames = self.telemetry.analysis_frames.load(Ordering::Relaxed);
        match self.last_analysis_change {
            // First observation: start the clock, never accuse on frame one.
            None => {
                self.last_analysis_frames = frames;
                self.last_analysis_change = Some(now);
                self.analysis_stalled = false;
            }
            Some(_) if frames != self.last_analysis_frames => {
                self.last_analysis_frames = frames;
                self.last_analysis_change = Some(now);
                self.analysis_stalled = false;
            }
            Some(since) => {
                // Only a live microphone makes silence suspicious: with no
                // input there is legitimately nothing to analyze.
                self.analysis_stalled = self.input_rms() > ANALYSIS_STALL_RMS_FLOOR
                    && now - since >= ANALYSIS_STALL_SECS;
            }
        }

        let input_errors = self.telemetry.input_stream_errors.load(Ordering::Relaxed);
        let output_errors = self.telemetry.output_stream_errors.load(Ordering::Relaxed);
        if input_errors != self.last_input_errors {
            self.last_input_errors = input_errors;
            self.audio_notice = Some((
                "Microphone stream error — input may be interrupted. Check the input device.",
                now,
            ));
        } else if output_errors != self.last_output_errors {
            self.last_output_errors = output_errors;
            self.audio_notice = Some((
                "Audio output stream error — monitoring may be interrupted.",
                now,
            ));
        }
        if let Some((_, seen)) = self.audio_notice
            && now - seen >= AUDIO_ERROR_NOTICE_SECS
        {
            self.audio_notice = None;
        }
    }

    /// The engine's worst current problem, as one line for the status banner,
    /// or `None` when everything is running. Ordered by how much it costs the
    /// user: no analysis at all, then stopped, then stalled, then audio I/O.
    fn engine_notice(&self) -> Option<&'static str> {
        // No microphone outranks everything else: with no input stream the
        // analysis path is idle by definition, so any staleness or GPU notice
        // below would only describe a consequence of this.
        if self.telemetry.audio_unavailable() {
            return Some(
                "Microphone unavailable — the audio engine could not open the mic. \
                 Grant the Microphone permission in Settings, then reopen the app.",
            );
        }
        match self.telemetry.analysis_state() {
            AnalysisState::Unavailable => Some(
                "Analysis unavailable — no compatible GPU adapter was found. \
                 Live readouts and capture are off.",
            ),
            AnalysisState::Stopped => Some(
                "Analysis stopped — the analysis thread exited. \
                 Restart the app to resume live readouts.",
            ),
            // A slow start is not a stall: `Starting` means the engine has not
            // claimed to be producing yet, so the absence of frames is expected.
            AnalysisState::Starting => self.audio_notice.map(|(msg, _)| msg),
            AnalysisState::Running => {
                if self.analysis_stalled {
                    Some(
                        "Analysis stalled — the microphone is live but no frames have \
                         arrived recently. Readouts below may be out of date.",
                    )
                } else {
                    self.audio_notice.map(|(msg, _)| msg)
                }
            }
        }
    }

    // ── state machine ────────────────────────────────────────────────────────

    fn advance(&mut self, now: f64) {
        // A capture left running (e.g. off another screen) auto-stops so the
        // accumulators can't grow unbounded; the tab bar shows a live dot
        // meanwhile.
        if let RecState::Recording { start } = self.rec
            && now - start >= MAX_REC_SECS
        {
            self.stop_rec(now);
        }
        if let RecState::Analyzing { start, elapsed } = self.rec
            && now - start >= 2.1
        {
            // Real f0 mean over voiced frames; a silent capture stays honest
            // (None readout) and is blocked from saving below.
            let f0 = (!self.rec_f0_acc.is_empty())
                .then(|| self.rec_f0_acc.iter().sum::<f32>() / self.rec_f0_acc.len() as f32)
                .map(|f| (f * 10.0).round() / 10.0);
            let mean =
                |acc: &[f32]| (!acc.is_empty()).then(|| acc.iter().sum::<f32>() / acc.len() as f32);
            // Median: robust central tendency for the identity features —
            // one harmonic-attracted stray fit shifts a mean, not a median.
            let median = |acc: &[f32]| -> Option<f32> {
                if acc.is_empty() {
                    return None;
                }
                let mut v: Vec<f32> = acc.iter().copied().filter(|x| x.is_finite()).collect();
                if v.is_empty() {
                    return None;
                }
                v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mid = v.len() / 2;
                Some(if v.len().is_multiple_of(2) {
                    0.5 * (v[mid - 1] + v[mid])
                } else {
                    v[mid]
                })
            };
            // Vibrato/steadiness are contour-level: snapshot the stop-time
            // values rather than averaging per-frame reports.
            let m = self.current_profile.metrics;
            let profile = if self.rec_profile_n > 0 {
                let n = self.rec_profile_n as f32;
                std::array::from_fn(|i| self.rec_profile_sum[i] / n)
            } else {
                [0.0; 16]
            };

            // Build this capture's classical voiceprint and score it against
            // the enrolled reference.
            // Identity-grade formants only: a capture sung above ~200 Hz f0
            // gets a formant-less voiceprint and is scored on its remaining
            // features (or reads "Unscored") — phase C's skip-and-renormalize
            // machinery — instead of on harmonic-attraction artifacts. Each
            // slot is the capture's *median* over identity-grade frames;
            // slots with no accepted frames stay at the 0.0 unmeasured
            // encoding and are skipped by the similarity scorer.
            let id_formants = {
                let med: Vec<Option<f32>> = self.rec_id_f_acc.iter().map(|v| median(v)).collect();
                med.iter().any(Option::is_some).then(|| {
                    std::array::from_fn(|i| Formant {
                        frequency: med[i].unwrap_or(0.0),
                        bandwidth: 0.0,
                    })
                })
            };
            let voiceprint = crate::math::build_voiceprint(
                id_formants,
                &profile,
                mean(&self.rec_centroid_acc),
                median(&self.rec_vtl_acc),
            );

            // Enrollment gate: only a long-enough, mostly-voiced capture may
            // become the reference (a bad reference poisons every later match).
            let voiced_frac = if self.rec_frames_total > 0 {
                self.rec_f0_acc.len() as f32 / self.rec_frames_total as f32
            } else {
                0.0
            };
            let enroll_eligible = f0.is_some()
                && elapsed >= MIN_ENROLL_SECS
                && voiced_frac >= MIN_ENROLL_VOICED_FRACTION;
            let is_reference = self.enrolled.is_none() && enroll_eligible;
            let match_pct = match &self.enrolled {
                Some(reference) => crate::math::voiceprint_similarity(&voiceprint, reference)
                    .map(|s| (s * 10.0).round() / 10.0),
                // An eligible first capture *becomes* the reference (self-match
                // 100); an ineligible one has nothing to score against.
                None => is_reference.then_some(100.0),
            };
            let blocked = if f0.is_none() {
                Some("No voiced signal captured — nothing to save".to_string())
            } else if self.enrolled.is_none() && !enroll_eligible {
                Some(format!(
                    "The reference capture needs ≥{MIN_ENROLL_SECS:.0} s of sustained voice — \
                     this one ran {elapsed:.0} s at {:.0}% voiced",
                    voiced_frac * 100.0
                ))
            } else {
                None
            };

            self.result = Some(CaptureResult {
                match_pct,
                f0,
                hnr_db: mean(&self.rec_hnr_acc),
                h1_h2_db: mean(&self.rec_h1h2_acc),
                vibrato: m.vibrato,
                steadiness_cents: m.steadiness_cents,
                jitter_pct: mean(&self.rec_jitter_acc),
                shimmer_db: mean(&self.rec_shimmer_acc),
                cpp_db: mean(&self.rec_cpp_acc),
                centroid_hz: mean(&self.rec_centroid_acc),
                profile,
                formants: self.rec_formants,
                voiceprint,
                is_reference,
                blocked,
            });
            self.rec = RecState::Done { elapsed };
        }
    }

    fn start_rec(&mut self, now: f64) {
        self.rec = RecState::Recording { start: now };
        self.rec_f0_acc.clear();
        self.rec_frames_total = 0;
        self.rec_hnr_acc.clear();
        self.rec_h1h2_acc.clear();
        self.rec_jitter_acc.clear();
        self.rec_shimmer_acc.clear();
        self.rec_cpp_acc.clear();
        self.rec_centroid_acc.clear();
        self.rec_profile_sum = [0.0; 16];
        self.rec_profile_n = 0;
        self.rec_formants = None;
        self.rec_id_f_acc = [Vec::new(), Vec::new(), Vec::new()];
        self.rec_vtl_acc.clear();
        self.result = None;
    }

    fn stop_rec(&mut self, now: f64) {
        if let RecState::Recording { start } = self.rec {
            self.rec = RecState::Analyzing {
                start: now,
                elapsed: now - start,
            };
        }
    }

    fn save_session(&mut self) {
        // Degenerate captures may not be saved (G2): the Save button is
        // withheld for them, and this guard keeps the archive honest even if
        // this is reached some other way.
        if self.result.as_ref().is_none_or(|r| r.blocked.is_some()) {
            return;
        }
        if let Some(res) = self.result.take() {
            // First *eligible* saved capture enrolls the reference voiceprint;
            // its match reads 100% against itself.
            if self.enrolled.is_none() && res.is_reference {
                self.enrolled = Some(res.voiceprint);
                self.enrolled_id = Some(self.derive_voice_id(&res.voiceprint));
            }
            let subj = self
                .enrolled_id
                .clone()
                .unwrap_or_else(|| "V-????".to_string());
            let session = Session {
                id: format!("VS-{:03}", self.next_session_num),
                subj,
                date: today_string(),
                // Gated above: blocked.is_none() implies a measured f0.
                f0: res.f0.unwrap_or(0.0),
                match_pct: res.match_pct,
                hnr_db: res.hnr_db,
                h1_h2_db: res.h1_h2_db,
                vibrato: res.vibrato,
                steadiness_cents: res.steadiness_cents,
                jitter_pct: res.jitter_pct,
                shimmer_db: res.shimmer_db,
                cpp_db: res.cpp_db,
                centroid_hz: res.centroid_hz,
                profile: res.profile,
                formants: res.formants,
            };
            self.next_session_num += 1;
            self.sessions.insert(0, session);
            // Enrollment and the new capture just changed the archive — write it
            // through so it survives a restart.
            self.persist_state();
            self.rec = RecState::Idle;
            self.filter = Filter::All;
            self.screen = Screen::Sessions;
        }
    }

    /// A short, stable ID for a voiceprint (e.g. "V-3F9A"), hashed from its
    /// formants + centroid so the same reference reads consistently.
    fn derive_voice_id(&self, vp: &Voiceprint) -> String {
        let mut h: u32 = 0x811c_9dc5;
        for v in [
            vp.formants[0],
            vp.formants[1],
            vp.formants[2],
            vp.centroid_hz,
        ] {
            h ^= (v as i32) as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
        format!("V-{:04X}", (h >> 16) as u16)
    }

    /// One waveform sample per frame: recording follows the live input RMS,
    /// idle decays flat (same smoothing constants as the prototype).
    fn push_wave_sample(&mut self) {
        const H: f32 = 52.0;
        let recording = matches!(self.rec, RecState::Recording { .. });
        let last = *self.wave.last().unwrap_or(&2.0);
        let next = if recording {
            let jitter = 0.7 + 0.6 * self.rand01();
            let target = (3.0 + self.input_rms() * 260.0 * jitter).min(H * 0.42);
            last + (target - last) * 0.55
        } else {
            last + (2.0 - last) * 0.12
        };
        self.wave.push(next.max(1.5));
        self.wave.remove(0);
    }
}

// ── top level ────────────────────────────────────────────────────────────────

impl eframe::App for DashboardApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Continuous repaint: canvases + live readouts animate every frame.
        ui.ctx().request_repaint();
        let now = ui.input(|i| i.time);

        self.watch_engine(now);
        self.advance(now);

        if self.ui_profile_rx.updated() {
            self.current_profile = *self.ui_profile_rx.read();
        }
        if matches!(self.rec, RecState::Recording { .. }) {
            // Count every recording tick (voiced or not) so the enrollment
            // gate can compute the capture's voiced fraction.
            self.rec_frames_total += 1;
        }
        if matches!(self.rec, RecState::Recording { .. }) && self.current_profile.valid {
            self.rec_f0_acc.push(self.current_profile.f0);
            // Formants are latched by the reliability of their *measurement*
            // frame (`formants_f0`, not the current f0): LPC's harmonic-
            // attraction bias belongs to the frame the fit ran on. Identity
            // grade additionally feeds the voiceprint; rejects feed nothing.
            match crate::math::formant_grade(
                &self.current_profile.formants,
                self.current_profile.formants_f0,
            ) {
                crate::math::FormantGrade::Identity => {
                    let f = self.current_profile.formants;
                    self.rec_formants = Some(f);
                    // Identity data additionally demands measurement-grade
                    // SNR (ASHA's ≥30 dB; math::IDENTITY_MIN_SNR_DB): a
                    // frame can be clean enough to show and still too
                    // noise-contaminated to become part of who the app
                    // thinks this singer is.
                    let id_snr_ok = self
                        .current_profile
                        .metrics
                        .snr_db
                        .is_none_or(|snr| snr >= crate::math::IDENTITY_MIN_SNR_DB);
                    if id_snr_ok {
                        for (acc, fm) in self.rec_id_f_acc.iter_mut().zip(&f) {
                            if fm.frequency > 0.0 && fm.frequency.is_finite() {
                                acc.push(fm.frequency);
                            }
                        }
                        if let Some(l) =
                            crate::tract::vtl_from_formants(f[1].frequency, f[2].frequency)
                        {
                            self.rec_vtl_acc.push(l);
                        }
                    }
                }
                crate::math::FormantGrade::DisplayOnly => {
                    self.rec_formants = Some(self.current_profile.formants);
                }
                crate::math::FormantGrade::Reject => {}
            }
            if let Some(h) = self.current_profile.metrics.hnr_db {
                self.rec_hnr_acc.push(h);
            }
            if let Some(h) = self.current_profile.metrics.h1_h2_db {
                self.rec_h1h2_acc.push(h);
            }
            if let Some(j) = self.current_profile.metrics.jitter_pct {
                self.rec_jitter_acc.push(j);
            }
            if let Some(s) = self.current_profile.metrics.shimmer_db {
                self.rec_shimmer_acc.push(s);
            }
            if let Some(c) = self.current_profile.metrics.cpp_db {
                self.rec_cpp_acc.push(c);
            }
            if let Some(c) = self.current_profile.metrics.centroid_hz {
                self.rec_centroid_acc.push(c);
            }
        }
        for (ema, &a) in self
            .harm_ema
            .iter_mut()
            .zip(&self.current_profile.partial_amplitudes)
        {
            *ema += 0.3 * (a - *ema);
        }
        if matches!(self.rec, RecState::Recording { .. }) && self.current_profile.valid {
            // Snapshot the just-updated harm_ema (normalized to its own max)
            // for the capture-mean profile driving the Timbre classification.
            let max_amp = self
                .harm_ema
                .iter()
                .take(16)
                .cloned()
                .fold(0.0f32, f32::max);
            if max_amp > 1e-6 {
                for (sum, &a) in self.rec_profile_sum.iter_mut().zip(&self.harm_ema[..16]) {
                    *sum += a / max_amp;
                }
                self.rec_profile_n += 1;
            }
        }
        // Display smoothing for the digit readouts; reset when the value goes
        // away so stale numbers never linger.
        let smooth = |disp: &mut Option<f32>, v: Option<f32>| match (disp.as_mut(), v) {
            (Some(d), Some(v)) => *d += 0.2 * (v - *d),
            _ => *disp = v,
        };
        smooth(&mut self.hnr_disp, self.current_profile.metrics.hnr_db);
        smooth(&mut self.h1h2_disp, self.current_profile.metrics.h1_h2_db);
        smooth(
            &mut self.jitter_disp,
            self.current_profile.metrics.jitter_pct,
        );
        smooth(
            &mut self.shimmer_disp,
            self.current_profile.metrics.shimmer_db,
        );
        smooth(&mut self.cpp_disp, self.current_profile.metrics.cpp_db);
        smooth(
            &mut self.centroid_disp,
            self.current_profile.metrics.centroid_hz,
        );
        // Bozeman's turnover, read from his own observable: the A2/A1
        // harmonic ratio (Journal of Singing 66:291, 2010) — "when H2 passes
        // through F1 and begins to weaken, H1 will increase in power". Unlike
        // the previous F1-based semitone distance, this needs no formant
        // estimate, so it works exactly where the gauge matters most: in the
        // passaggio, where LPC formants are unreliable (f0 > 350 Hz).
        let turnover_raw = if self.current_profile.valid {
            crate::math::a2_a1_db(&self.current_profile.partial_amplitudes)
        } else {
            None
        };
        smooth(&mut self.turnover_disp, turnover_raw);
        self.update_tract_model(now);
        self.push_wave_sample();

        let full = ui.max_rect();
        paint_background(ui.painter(), full);

        // Centered phone-width content column.
        let col_w = full.width().min(COL_WIDTH);
        let col_rect = Rect::from_min_max(
            pos2(full.center().x - col_w / 2.0, full.top()),
            pos2(full.center().x + col_w / 2.0, full.bottom()),
        );
        let mut col_ui = ui.new_child(egui::UiBuilder::new().max_rect(col_rect));
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(&mut col_ui, |ui| {
                egui::Frame::default().inner_margin(18.0).show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    // Android renders edge-to-edge under the system bars and
                    // eframe exposes no safe-area insets, so pad past them.
                    ui.add_space(TOP_INSET);
                    // Engine health (analysis unavailable/stopped/stalled, cpal
                    // stream errors) is live state, so this banner is not
                    // dismissible — it clears itself when the condition does.
                    if let Some(notice) = self.engine_notice() {
                        notice_banner(ui, notice, false);
                        ui.add_space(10.0);
                    }
                    // Persistence problems (unreadable archive, failed save)
                    // stay visible on every screen until dismissed.
                    if let Some(notice) = self.persist_notice.clone() {
                        if notice_banner(ui, &notice, true) {
                            self.persist_notice = None;
                        }
                        ui.add_space(10.0);
                    }
                    match self.screen {
                        Screen::Overview => self.screen_overview(ui),
                        Screen::Capture => self.screen_capture(ui, now),
                        Screen::Sessions => self.screen_sessions(ui),
                        Screen::Detail => self.screen_detail(ui),
                    }
                    // Clearance for the floating tab bar.
                    ui.add_space(104.0 + BOTTOM_INSET);
                });
            });

        self.tab_bar(ui.ctx().clone(), now);
    }
}

impl DashboardApp {
    /// Per-frame tract-model update: refresh the VTL estimate from
    /// identity-grade frames, then invert the current display-grade formants
    /// to (q1, q2). Anything that fails the reliability gate leaves the
    /// shape HELD — visibly grey, never animating on unreliable data.
    fn update_tract_model(&mut self, now: f64) {
        self.tract_live = false;
        let p = &self.current_profile;
        let grade = crate::math::formant_grade(&p.formants, p.formants_f0);
        if !p.valid || grade == crate::math::FormantGrade::Reject {
            return;
        }

        // VTL: anatomy accumulates slowly, and only from identity-grade
        // frames — the same rule as the voiceprint, for the same reason.
        if grade == crate::math::FormantGrade::Identity
            && let Some(l) =
                crate::tract::vtl_from_formants(p.formants[1].frequency, p.formants[2].frequency)
        {
            self.vtl_est_cm = Some(match self.vtl_est_cm {
                Some(prev) => prev + VTL_EMA_ALPHA * (l - prev),
                None => l,
            });
        }

        let basis = crate::tract::basis_for_vtl(self.vtl_est_cm);
        let Some(grid) = tract_grid_for(basis) else {
            return; // still building, first frames only
        };
        // Formants scale ~1/L: map the measured pair into the basis's length
        // reference before lookup (uniform-scaling approximation; the
        // oral/pharyngeal cavities actually scale differently — Fant 1966 —
        // which is part of why this is a model fit, not a measurement).
        let scale = self.vtl_est_cm.map_or(1.0, |l| l / basis.vtl_cm);
        let (f1, f2) = (
            p.formants[0].frequency * scale,
            p.formants[1].frequency * scale,
        );
        if let Some((q1, q2)) = grid.invert(f1, f2) {
            let (dq1, dq2) = match self.tract_q {
                Some((a, b)) => (
                    a + TRACT_Q_EMA_ALPHA * (q1 - a),
                    b + TRACT_Q_EMA_ALPHA * (q2 - b),
                ),
                None => (q1, q2),
            };
            self.tract_q = Some((dq1, dq2));
            self.tract_live = true;

            // The articulatory log: one sample whenever a frame clears every
            // gate (voiced, SNR over floor, formant grade, in vowel space),
            // decimated so a session-length log stays small. Nothing that
            // failed a gate is ever logged — the log's value IS the gating.
            if now - self.tract_log_last_t >= TRACT_LOG_MIN_DT {
                self.tract_log_last_t = now;
                if self.tract_log.len() == TRACT_LOG_CAP {
                    self.tract_log.pop_front();
                }
                self.tract_log.push_back((now, dq1, dq2));
            }
        }
    }

    /// 2.5D pseudo-midsagittal tract profile: the model diameter function
    /// D(i) drawn as a ribbon along a quarter-turn centerline (pharynx
    /// vertical, glottis at bottom; oral cavity horizontal, lips at right) —
    /// the same projection Story uses. Teal while LIVE, grey while HELD or
    /// idle; labeled a model throughout, because that is what it is.
    fn tract_view(&self, ui: &mut egui::Ui) {
        let basis = crate::tract::basis_for_vtl(self.vtl_est_cm);
        let grid_ready = tract_grid_for(basis).is_some();

        let (state, accent) = if !grid_ready {
            ("CALIBRATING", ink(115))
        } else if self.tract_live {
            ("LIVE", TEAL)
        } else if self.current_profile.metrics.voiced_but_noisy {
            // Periodicity present but the room is too loud to measure it
            // honestly: say so, instead of silently holding.
            ("NOISY", AMBER_TEXT)
        } else if self.tract_q.is_some() {
            ("HELD", AMBER_TEXT)
        } else {
            ("—", ink(115))
        };

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("TRACT MODEL")
                    .font(FontId::monospace(9.5))
                    .color(ink(115)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(state)
                        .font(FontId::monospace(10.5))
                        .color(accent)
                        .strong(),
                );
            });
        });
        ui.add_space(4.0);

        let (rect, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), TRACT_VIEW_H), Sense::hover());
        let painter = ui.painter();

        // Layout: profile ribbon on the left, the vowel-space log map as a
        // square on the right, caption strip along the bottom.
        let caption_h = 16.0;
        let map_side = (rect.height() - caption_h).min(rect.width() * 0.42);
        let map = Rect::from_min_size(
            pos2(rect.right() - map_side, rect.top()),
            vec2(map_side, map_side),
        );
        let rib = Rect::from_min_max(rect.min, pos2(map.left() - 10.0, rect.bottom() - caption_h));

        // Neutral shape as idle context (the model's own rest posture — a
        // labeled model state, not user data); live/held coefficients
        // otherwise.
        let (q1, q2) = self.tract_q.unwrap_or((0.0, 0.0));
        let d = crate::tract::diameters(basis, q1, q2);
        let n = crate::tract::N_SECTIONS;

        // Quarter-turn centerline. Roughly Story's pseudo-midsagittal split:
        // lower-pharynx run vertical, a 90° velar bend, oral run horizontal.
        let margin = 26.0f32;
        let usable_w = (rib.width() - 2.0 * margin).max(60.0);
        let usable_h = (rib.height() - 2.0 * margin).max(60.0);
        // Path budget: BEND_START straight + arc + rest straight. Fit step so
        // the whole polyline spans the rect.
        let arc_secs = (TRACT_BEND_END - TRACT_BEND_START) as f32;
        let horiz_secs = (n - TRACT_BEND_END) as f32;
        let vert_secs = TRACT_BEND_START as f32;
        // Extents in step units: height = vert + arc radius contribution,
        // width = horiz + radius. r = arc_len / (pi/2), arc_len = arc_secs.
        let r_units = arc_secs / std::f32::consts::FRAC_PI_2;
        let h_units = vert_secs + r_units;
        let w_units = horiz_secs + r_units;
        let step = (usable_h / h_units).min(usable_w / w_units);
        let origin = pos2(rib.left() + margin, rib.bottom() - margin);

        // Centerline points + unit normals.
        let mut pts = Vec::with_capacity(n + 1);
        let mut normals = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let s_units = i as f32;
            let (x, y, tangent) = if s_units <= vert_secs {
                (0.0, -s_units, (0.0f32, -1.0f32))
            } else if s_units <= vert_secs + arc_secs {
                let a = (s_units - vert_secs) / r_units; // 0..pi/2
                (
                    r_units - r_units * a.cos(),
                    -(vert_secs + r_units * a.sin()),
                    (a.sin(), -a.cos()),
                )
            } else {
                (
                    r_units + (s_units - vert_secs - arc_secs),
                    -(vert_secs + r_units),
                    (1.0, 0.0),
                )
            };
            pts.push(pos2(origin.x + x * step, origin.y + y * step));
            // Normal = tangent rotated 90°.
            normals.push(vec2(-tangent.1, tangent.0));
        }

        let px_per_cm = step / (basis.vtl_cm / n as f32) * 0.5;
        let fill = if self.tract_live { teal_a(56) } else { ink(26) };
        let edge = if self.tract_live {
            Stroke::new(1.5, TEAL_DARK)
        } else {
            Stroke::new(1.2, ink(90))
        };

        let mut upper = Vec::with_capacity(n + 1);
        let mut lower = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let di = d[i.min(n - 1)];
            let w = (di * px_per_cm * 0.5).clamp(1.0, step * 2.6);
            upper.push(pts[i] + normals[i] * w);
            lower.push(pts[i] - normals[i] * w);
        }
        for i in 0..n {
            painter.add(Shape::convex_polygon(
                vec![upper[i], upper[i + 1], lower[i + 1], lower[i]],
                fill,
                Stroke::NONE,
            ));
        }
        painter.add(Shape::line(upper, edge));
        painter.add(Shape::line(lower, edge));

        // End labels.
        painter.text(
            pts[0] + vec2(0.0, 12.0),
            Align2::CENTER_TOP,
            "glottis",
            FontId::monospace(8.5),
            ink(115),
        );
        painter.text(
            pts[n] + vec2(6.0, 0.0),
            Align2::LEFT_CENTER,
            "lips",
            FontId::monospace(8.5),
            ink(115),
        );

        // ── Vowel-space log map: the session's articulatory history ──────
        // Axes are the model's mode coefficients (q1 →, q2 ↑). Every dot is
        // one fully-gated moment of the singer's session; the trail fades
        // with age. Anchors are the published Table II vowels — the model's
        // landmarks, not measurements of this singer.
        painter.rect(
            map,
            6.0,
            white(120),
            Stroke::new(1.0, ink(28)),
            StrokeKind::Inside,
        );
        let to_map = |q1: f32, q2: f32| -> Pos2 {
            let tx = (q1 - crate::tract::Q1_MIN) / (crate::tract::Q1_MAX - crate::tract::Q1_MIN);
            let ty = (q2 - crate::tract::Q2_MIN) / (crate::tract::Q2_MAX - crate::tract::Q2_MIN);
            pos2(
                map.left() + tx.clamp(0.0, 1.0) * map.width(),
                map.bottom() - ty.clamp(0.0, 1.0) * map.height(),
            )
        };
        // Zero-axes, faint.
        let z = to_map(0.0, 0.0);
        painter.line_segment(
            [pos2(map.left(), z.y), pos2(map.right(), z.y)],
            Stroke::new(0.5, ink(18)),
        );
        painter.line_segment(
            [pos2(z.x, map.top()), pos2(z.x, map.bottom())],
            Stroke::new(0.5, ink(18)),
        );
        for (name, vq1, vq2) in crate::tract::VOWEL_ANCHORS {
            painter.text(
                to_map(vq1, vq2),
                Align2::CENTER_CENTER,
                name,
                FontId::monospace(9.0),
                ink(96),
            );
        }
        let trail_n = self.tract_log.len().min(TRACT_MAP_TRAIL);
        for (age, &(_, lq1, lq2)) in self.tract_log.iter().rev().take(trail_n).enumerate() {
            // Newest opaque, oldest nearly gone.
            let a = (200.0 * (1.0 - age as f32 / trail_n as f32)).max(14.0) as u8;
            painter.circle_filled(to_map(lq1, lq2), 1.6, teal_a(a));
        }
        if self.tract_live
            && let Some((cq1, cq2)) = self.tract_q
        {
            let c = to_map(cq1, cq2);
            painter.circle_filled(c, 3.0, TEAL);
            painter.circle_stroke(c, 4.5, Stroke::new(1.0, teal_a(120)));
        }

        // Caption: what this is, and what it is scaled to. Never "your
        // vocal tract" — see tract_data's honesty boundary. Log size shows
        // the data-log nature of the card: it only ever counts fully-gated
        // moments.
        let log_txt = if self.tract_log.is_empty() {
            String::new()
        } else {
            let span_s = self.tract_log.back().map(|b| b.0).unwrap_or(0.0)
                - self.tract_log.front().map(|f| f.0).unwrap_or(0.0);
            format!(
                " · log {} pts / {:.0} min",
                self.tract_log.len(),
                (span_s / 60.0).max(0.0)
            )
        };
        let caption = match self.vtl_est_cm {
            Some(l) => format!("model tract · est. length ≈ {l:.1} cm{log_txt}"),
            None => format!("model tract · assumed {:.1} cm{log_txt}", basis.vtl_cm),
        };
        painter.text(
            pos2(rect.center().x, rect.bottom() - 4.0),
            Align2::CENTER_BOTTOM,
            caption,
            FontId::monospace(8.5),
            ink(115),
        );
    }
}

/// Lazily built inversion grids, one per basis, built off the UI thread on
/// native (the UI shows CALIBRATING for the first moments of the first
/// session). On wasm — which has no analysis thread and thus never measures
/// formants — the build would run inline if ever requested.
fn tract_grid_for(
    basis: &'static crate::tract::TractBasis,
) -> Option<&'static crate::tract::TractGrid> {
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicBool, Ordering};
    static GRID_M: OnceLock<crate::tract::TractGrid> = OnceLock::new();
    static GRID_F: OnceLock<crate::tract::TractGrid> = OnceLock::new();
    static BUILDING_M: AtomicBool = AtomicBool::new(false);
    static BUILDING_F: AtomicBool = AtomicBool::new(false);

    let female = (basis.vtl_cm - 15.53).abs() < 0.1;
    let (cell, building) = if female {
        (&GRID_F, &BUILDING_F)
    } else {
        (&GRID_M, &BUILDING_M)
    };
    if let Some(g) = cell.get() {
        return Some(g);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if !building.swap(true, Ordering::Relaxed) {
            std::thread::spawn(move || {
                let g = crate::tract::TractGrid::build(
                    basis,
                    crate::tract::GRID_N,
                    crate::tract::GRID_N,
                );
                let _ = cell.set(g);
            });
        }
        None
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = building;
        let _ = cell.set(crate::tract::TractGrid::build(
            basis,
            crate::tract::GRID_N,
            crate::tract::GRID_N,
        ));
        cell.get()
    }
}

/// One amber status line in a glass card, shown above whichever screen is up.
/// Returns true when a dismissible banner's ✕ was clicked this frame.
fn notice_banner(ui: &mut egui::Ui, text: &str, dismissible: bool) -> bool {
    let mut dismissed = false;
    glass(16.0).show(ui, |ui| {
        ui.horizontal(|ui| {
            let close_w = if dismissible { 26.0 } else { 0.0 };
            ui.allocate_ui_with_layout(
                vec2(ui.available_width() - close_w, 0.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.add(
                        egui::Label::new(RichText::new(text).size(12.0).color(AMBER_TEXT)).wrap(),
                    );
                },
            );
            if dismissible {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let resp = ui
                        .add(
                            egui::Label::new(RichText::new("✕").size(13.0).color(ink(140)))
                                .sense(Sense::click()),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    dismissed = resp.clicked();
                });
            }
        });
    });
    dismissed
}

// ── background ───────────────────────────────────────────────────────────────

fn paint_background(painter: &egui::Painter, rect: Rect) {
    painter.rect_filled(rect, 0.0, BG_BASE);
    // Soft radial washes (layered translucent circles stand in for gradients).
    let tr = pos2(rect.right() + 60.0, rect.top() - 60.0);
    let bl = pos2(rect.left() - 80.0, rect.bottom() - 120.0);
    for (i, r) in [150.0f32, 110.0, 70.0].iter().enumerate() {
        let a = 8 + i as u8 * 6;
        painter.circle_filled(
            tr,
            *r * 2.0,
            Color32::from_rgba_unmultiplied(34, 211, 238, a),
        );
        painter.circle_filled(
            bl,
            *r * 2.3,
            Color32::from_rgba_unmultiplied(14, 148, 136, a / 2),
        );
    }
}

// ── shared widgets ───────────────────────────────────────────────────────────

impl DashboardApp {
    fn screen_kicker(&self, ui: &mut egui::Ui, kicker: &str, title: &str) {
        ui.label(
            RichText::new(kicker)
                .font(FontId::proportional(11.0))
                .color(TEAL)
                .strong(),
        );
        ui.add_space(3.0);
        ui.label(
            RichText::new(title)
                .font(FontId::proportional(30.0))
                .color(INK)
                .strong(),
        );
    }
}

fn pill_badge(ui: &mut egui::Ui, text: &str, bg: Color32, fg: Color32) {
    let font = FontId::proportional(12.0);
    let galley = ui.painter().layout_no_wrap(text.into(), font.clone(), fg);
    let size = vec2(galley.size().x + 22.0, 24.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, 12.0, bg);
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, text, font, fg);
}

/// Rounded pill button; returns true on click.
fn pill_button(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    text: &str,
    fill: Color32,
    fg: Color32,
    stroke: Stroke,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let fill = if resp.is_pointer_button_down_on() {
        fill.linear_multiply(0.9)
    } else {
        fill
    };
    ui.painter()
        .rect(rect, size.y / 2.0, fill, stroke, StrokeKind::Inside);
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        FontId::proportional(14.5),
        fg,
    );
    resp.clicked()
}

/// Circular progress ring ("match %" gauge). `pct` in 0..=100.
fn ring_gauge(
    painter: &egui::Painter,
    rect: Rect,
    pct: f32,
    color: Color32,
    value: &str,
    sub: Option<&str>,
) {
    let c = rect.center();
    let r = rect.width().min(rect.height()) * 0.4375; // 42/96 of the viewbox
    let w = rect.width() / 12.0; // stroke 8/96
    painter.circle_stroke(c, r, Stroke::new(w, teal_a(31)));

    let frac = (pct / 100.0).clamp(0.0, 1.0);
    if frac > 0.005 {
        let start = -std::f32::consts::FRAC_PI_2;
        let sweep = frac * std::f32::consts::TAU;
        let n = 64;
        let points: Vec<Pos2> = (0..=n)
            .map(|i| {
                let a = start + sweep * i as f32 / n as f32;
                pos2(c.x + r * a.cos(), c.y + r * a.sin())
            })
            .collect();
        // Round caps faked with end dots.
        let (first, last) = (points[0], points[n]);
        painter.add(Shape::line(points, Stroke::new(w, color)));
        painter.circle_filled(first, w / 2.0, color);
        painter.circle_filled(last, w / 2.0, color);
    }

    if let Some(sub) = sub {
        painter.text(
            pos2(c.x, c.y - 3.0),
            Align2::CENTER_BOTTOM,
            value,
            FontId::monospace(rect.width() * 0.2),
            INK,
        );
        painter.text(
            pos2(c.x, c.y + 4.0),
            Align2::CENTER_TOP,
            sub,
            FontId::proportional(rect.width() * 0.1),
            ink(128),
        );
    } else {
        painter.text(
            c,
            Align2::CENTER_CENTER,
            value,
            FontId::monospace(rect.width() * 0.2),
            INK,
        );
    }
}

fn sparkline(painter: &egui::Painter, rect: Rect, pts: &[(f32, f32)]) {
    let points: Vec<Pos2> = pts
        .iter()
        .map(|(x, y)| {
            pos2(
                rect.left() + x / 100.0 * rect.width(),
                rect.top() + y / 24.0 * rect.height(),
            )
        })
        .collect();
    painter.add(Shape::line(points, Stroke::new(1.8, teal_a(204))));
}

/// The prototype's capture-row glyph: a tiny symmetric waveform.
fn waveform_glyph(painter: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 16.0;
    let o = rect.min;
    let seg = |x1: f32, y1: f32, x2: f32, y2: f32| {
        [
            pos2(o.x + x1 * s, o.y + y1 * s),
            pos2(o.x + x2 * s, o.y + y2 * s),
        ]
    };
    let stroke = Stroke::new(1.6, color);
    painter.line_segment(seg(2.0, 8.0, 3.5, 8.0), stroke);
    painter.line_segment(seg(5.0, 5.0, 5.0, 11.0), stroke);
    painter.line_segment(seg(8.0, 2.5, 8.0, 13.5), stroke);
    painter.line_segment(seg(11.0, 5.0, 11.0, 11.0), stroke);
    painter.line_segment(seg(14.0, 8.0, 12.5, 8.0), stroke);
}

/// Microphone glyph scaled into `rect` (design geometry is a 24 px viewbox).
fn mic_glyph(painter: &egui::Painter, rect: Rect, color: Color32, filled_body: bool) {
    let s = rect.width() / 24.0;
    let o = rect.min;
    let p = |x: f32, y: f32| pos2(o.x + x * s, o.y + y * s);
    let body = Rect::from_min_max(p(9.0, 3.0), p(15.0, 14.0));
    if filled_body {
        painter.rect_filled(body, 3.0 * s, color);
    } else {
        painter.rect(
            body,
            3.0 * s,
            Color32::TRANSPARENT,
            Stroke::new(1.8 * s, color),
            StrokeKind::Middle,
        );
    }
    // Lower semicircle from (5.5,11) to (18.5,11) through (12,17.5).
    let c = p(12.0, 11.0);
    let r = 6.5 * s;
    let arc: Vec<Pos2> = (0..=24)
        .map(|i| {
            let a = std::f32::consts::PI * i as f32 / 24.0;
            pos2(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect();
    painter.add(Shape::line(arc, Stroke::new(1.9 * s, color)));
    painter.line_segment([p(12.0, 17.5), p(12.0, 21.0)], Stroke::new(1.9 * s, color));
}

// ── screens ──────────────────────────────────────────────────────────────────

impl DashboardApp {
    fn screen_overview(&mut self, ui: &mut egui::Ui) {
        // Header row.
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.screen_kicker(ui, "VOXLAB · ACOUSTIC BIOMETRICS", "Overview");
            });
            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::hover());
                ui.painter().circle_filled(rect.center(), 20.0, white(179));
                ui.painter()
                    .circle_stroke(rect.center(), 20.0, Stroke::new(1.0, white(230)));
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "AK",
                    FontId::proportional(13.0),
                    TEAL_DARK,
                );
            });
        });
        ui.add_space(18.0);

        // Reference-voiceprint hero card. Reflects real enrollment state: the
        // ring shows the latest capture's real match; the label is the enrolled
        // reference ID and true session count. Before enrollment it invites a
        // first capture rather than showing a fake subject.
        let latest = self.sessions.first().cloned();
        let count = self.sessions.len();
        let id_label = self.enrolled_id.clone();
        let reenroll_armed = self.reenroll_armed;
        let mut reenroll_clicked = false;
        glass(24.0).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(96.0, 96.0), Sense::hover());
                let ring_pct = latest.as_ref().and_then(|s| s.match_pct).unwrap_or(0.0);
                let ring_val = latest
                    .as_ref()
                    .map(|s| Self::match_text(s.match_pct))
                    .unwrap_or_else(|| "—".into());
                ring_gauge(
                    ui.painter(),
                    rect,
                    ring_pct,
                    TEAL,
                    &ring_val,
                    Some("MATCH %"),
                );
                ui.add_space(14.0);
                ui.vertical(|ui| match &id_label {
                    Some(id) => {
                        ui.label(
                            RichText::new("Reference voiceprint")
                                .size(12.0)
                                .color(ink(140)),
                        );
                        ui.add_space(3.0);
                        ui.label(
                            RichText::new(id)
                                .font(FontId::monospace(20.0))
                                .color(INK)
                                .strong(),
                        );
                        ui.add_space(6.0);
                        let plural = if count == 1 { "" } else { "s" };
                        ui.label(
                            RichText::new(format!("Enrolled · {count} session{plural}"))
                                .size(12.0)
                                .color(ink(140)),
                        );
                        ui.add_space(9.0);
                        if let Some(s) = &latest {
                            let (badge, bg, fg) = Self::badge_style(s.match_pct);
                            pill_badge(ui, badge, bg, fg);
                        }
                        ui.add_space(8.0);
                        // Two-tap re-enroll: a bad reference is recoverable
                        // without hand-deleting archive.json. Past sessions
                        // keep their historical scores.
                        let (relabel, recolor) = if reenroll_armed {
                            ("Tap again to clear the reference", AMBER_TEXT)
                        } else {
                            ("Re-enroll…", ink(128))
                        };
                        let resp = ui
                            .add(
                                egui::Label::new(RichText::new(relabel).size(11.0).color(recolor))
                                    .sense(Sense::click()),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        if resp.clicked() {
                            reenroll_clicked = true;
                        }
                    }
                    None => {
                        ui.label(
                            RichText::new("No voiceprint enrolled")
                                .size(12.0)
                                .color(ink(140)),
                        );
                        ui.add_space(3.0);
                        ui.label(
                            RichText::new("—")
                                .font(FontId::monospace(20.0))
                                .color(INK)
                                .strong(),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Capture and save to enroll your reference")
                                .size(12.0)
                                .color(ink(140)),
                        );
                    }
                });
            });
        });
        if reenroll_clicked {
            if self.reenroll_armed {
                // Confirmed: clear the reference. The next capture that passes
                // the enrollment gate becomes the new one on save.
                self.enrolled = None;
                self.enrolled_id = None;
                self.reenroll_armed = false;
                self.persist_state();
            } else {
                self.reenroll_armed = true;
            }
        }
        ui.add_space(12.0);

        // Metric tiles (2 × 2) — real aggregates across the saved archive.
        // Sessions are stored newest-first; `.rev()` gives oldest→newest for
        // the trend sparkline.
        let vals_f0: Vec<f32> = self.sessions.iter().rev().map(|s| s.f0).collect();
        let vals_jit: Vec<f32> = self
            .sessions
            .iter()
            .rev()
            .filter_map(|s| s.jitter_pct)
            .collect();
        let vals_shim: Vec<f32> = self
            .sessions
            .iter()
            .rev()
            .filter_map(|s| s.shimmer_db)
            .collect();
        let vals_hnr: Vec<f32> = self
            .sessions
            .iter()
            .rev()
            .filter_map(|s| s.hnr_db)
            .collect();

        let mean = |v: &[f32]| (!v.is_empty()).then(|| v.iter().sum::<f32>() / v.len() as f32);
        // Last up-to-8 values → sparkline points (higher value = toward top).
        let spark = |v: &[f32]| -> Vec<(f32, f32)> {
            let w = &v[v.len().saturating_sub(8)..];
            if w.len() < 2 {
                return Vec::new();
            }
            let (mn, mx) = w
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
            let range = (mx - mn).max(1e-6);
            w.iter()
                .enumerate()
                .map(|(i, &x)| {
                    let px = i as f32 / (w.len() - 1) as f32 * 100.0;
                    let py = 22.0 - (x - mn) / range * 20.0;
                    (px, py)
                })
                .collect()
        };
        let fmt = |v: Option<f32>, d: usize| match v {
            Some(x) => format!("{x:.*}", d),
            None => "—".into(),
        };
        let tiles = [
            (
                "F0 MEAN",
                fmt(mean(&vals_f0), 1),
                "Hz",
                "across saved captures",
                spark(&vals_f0),
            ),
            (
                "JITTER",
                fmt(mean(&vals_jit), 2),
                "%",
                "ref < 1.04 %",
                spark(&vals_jit),
            ),
            (
                "SHIMMER",
                fmt(mean(&vals_shim), 2),
                "dB",
                "ref < 0.35 dB",
                spark(&vals_shim),
            ),
            (
                "HNR",
                fmt(mean(&vals_hnr), 1),
                "dB",
                "ref > 17 dB",
                spark(&vals_hnr),
            ),
        ];
        for row in tiles.chunks(2) {
            ui.columns(2, |cols| {
                for (col, (label, value, unit, reference, pts)) in cols.iter_mut().zip(row) {
                    glass(20.0).show(col, |ui| {
                        ui.label(
                            RichText::new(*label)
                                .font(FontId::monospace(10.0))
                                .color(ink(128)),
                        );
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(value)
                                    .font(FontId::monospace(22.0))
                                    .color(INK)
                                    .strong(),
                            );
                            ui.label(RichText::new(*unit).size(12.0).color(ink(128)));
                        });
                        ui.add_space(7.0);
                        let (rect, _) = ui
                            .allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
                        sparkline(ui.painter(), rect, pts);
                        ui.add_space(6.0);
                        ui.label(RichText::new(*reference).size(10.5).color(ink(115)));
                    });
                }
            });
            ui.add_space(12.0);
        }
        ui.add_space(8.0);

        // Recent captures.
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Recent captures")
                    .size(14.0)
                    .color(INK)
                    .strong(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let resp = ui
                    .label(RichText::new("View all").size(12.5).color(TEAL).strong())
                    .interact(Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() {
                    self.screen = Screen::Sessions;
                }
            });
        });
        ui.add_space(10.0);

        if self.sessions.is_empty() {
            ui.label(RichText::new("No captures yet.").size(12.5).color(ink(120)));
        }
        let recent: Vec<(usize, Session)> =
            self.sessions.iter().take(3).cloned().enumerate().collect();
        for (i, s) in recent {
            if self.session_row_compact(ui, &s, i) {
                self.selected = Some(i);
                self.export_queued = false;
                self.screen = Screen::Detail;
            }
            ui.add_space(9.0);
        }
    }

    /// Overview "recent capture" row; returns true when clicked.
    fn session_row_compact(&self, ui: &mut egui::Ui, s: &Session, i: usize) -> bool {
        let (_, badge_bg, badge_fg) = Self::badge_style(s.match_pct);
        let ir = glass(18.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
                ui.painter().rect_filled(rect, 12.0, teal_a(26));
                waveform_glyph(
                    ui.painter(),
                    Rect::from_center_size(rect.center(), vec2(16.0, 16.0)),
                    TEAL,
                );
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&s.id)
                            .font(FontId::monospace(13.5))
                            .color(INK)
                            .strong(),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(format!("{} · {}", s.subj, s.date))
                            .size(11.5)
                            .color(ink(128)),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let text = Self::match_text(s.match_pct);
                    let font = FontId::monospace(12.0);
                    let galley = ui
                        .painter()
                        .layout_no_wrap(text.clone(), font.clone(), badge_fg);
                    let size = vec2(galley.size().x + 20.0, 24.0);
                    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                    ui.painter().rect_filled(rect, 12.0, badge_bg);
                    ui.painter()
                        .text(rect.center(), Align2::CENTER_CENTER, text, font, badge_fg);
                });
            });
        });
        ui.interact(
            ir.response.rect,
            ui.id().with("recent").with(i),
            Sense::click(),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    }

    fn screen_capture(&mut self, ui: &mut egui::Ui, now: f64) {
        let chip = self
            .enrolled_id
            .clone()
            .unwrap_or_else(|| "Not enrolled".into());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.screen_kicker(ui, "LIVE ACQUISITION", "Capture");
            });
            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                let font = FontId::monospace(12.5);
                let galley = ui.painter().layout_no_wrap(chip.clone(), font.clone(), INK);
                let size = vec2(galley.size().x + 38.0, 30.0);
                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                ui.painter().rect(
                    rect,
                    15.0,
                    white(179),
                    Stroke::new(1.0, white(230)),
                    StrokeKind::Inside,
                );
                ui.painter()
                    .circle_filled(pos2(rect.left() + 15.0, rect.center().y), 3.5, TEAL);
                ui.painter().text(
                    pos2(rect.left() + 26.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    &chip,
                    font,
                    INK,
                );
            });
        });
        ui.add_space(16.0);

        let recording = matches!(self.rec, RecState::Recording { .. });

        // Vocal tract card.
        glass(24.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Vocal tract · articulatory model")
                        .size(13.0)
                        .color(INK)
                        .strong(),
                );
                if recording {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let blink = if (now * 1.0).fract() < 0.5 { 255 } else { 64 };
                        ui.label(
                            RichText::new("LIVE")
                                .font(FontId::monospace(10.0))
                                .color(CYAN_DEEP),
                        );
                        let (dot, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                        ui.painter().circle_filled(
                            dot.center(),
                            3.0,
                            Color32::from_rgba_unmultiplied(6, 182, 212, blink),
                        );
                    });
                }
            });
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 236.0), Sense::hover());
            paint_tract(ui.painter(), rect, (now * 1000.0) as f32, recording);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("SAGITTAL MESH · 44 SECTIONS")
                        .font(FontId::monospace(9.5))
                        .color(ink(107)),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("AREA FN · MRI FIT")
                            .font(FontId::monospace(9.5))
                            .color(ink(107)),
                    );
                });
            });
        });
        ui.add_space(12.0);

        // Waveform + live readouts.
        glass(24.0).show(ui, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::hover());
            paint_wave(ui.painter(), rect, &self.wave);
            ui.add_space(11.0);
            let sep = ui.available_rect_before_wrap();
            ui.painter().line_segment(
                [pos2(sep.left(), sep.top()), pos2(sep.right(), sep.top())],
                Stroke::new(1.0, ink(18)),
            );
            ui.add_space(11.0);

            let f0 = if recording && self.current_profile.valid {
                format!("{:.1} Hz", self.current_profile.f0)
            } else {
                "—".into()
            };
            let level = if recording {
                let rms = self.input_rms().max(1e-6);
                format!("{:.1} dB", (20.0 * rms.log10()).clamp(-90.0, 0.0))
            } else {
                "—".into()
            };
            let elapsed = match self.rec {
                RecState::Recording { start } => now - start,
                RecState::Analyzing { elapsed, .. } | RecState::Done { elapsed } => elapsed,
                RecState::Idle => 0.0,
            };
            let timer = format!("{}:{:04.1}", (elapsed / 60.0) as u32, elapsed % 60.0);
            ui.columns(3, |cols| {
                for (col, (label, value)) in
                    cols.iter_mut()
                        .zip([("F0", f0), ("LEVEL", level), ("ELAPSED", timer)])
                {
                    col.label(
                        RichText::new(label)
                            .font(FontId::monospace(9.5))
                            .color(ink(115)),
                    );
                    col.add_space(3.0);
                    col.label(
                        RichText::new(value)
                            .font(FontId::monospace(16.0))
                            .color(INK)
                            .strong(),
                    );
                }
            });
        });
        ui.add_space(14.0);

        // Harmonic series (live, musician-facing).
        self.harmonics_card(ui);
        ui.add_space(14.0);

        // Analyzing card.
        if let RecState::Analyzing { start, .. } = self.rec {
            glass(22.0).show(ui, |ui| {
                ui.label(
                    RichText::new("Analyzing capture")
                        .size(14.0)
                        .color(INK)
                        .strong(),
                );
                ui.add_space(10.0);
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::hover());
                ui.painter().rect_filled(rect, 3.0, teal_a(31));
                let frac = (((now - start) / 2.0).clamp(0.04, 1.0)) as f32;
                let bar = Rect::from_min_size(rect.min, vec2(rect.width() * frac, 6.0));
                ui.painter().rect_filled(bar, 3.0, TEAL);
                ui.add_space(10.0);
                let analyzing_line = match &self.enrolled_id {
                    Some(id) => format!("EXTRACTING VOICEPRINT · SCORING VS {id}"),
                    None => "EXTRACTING VOICEPRINT · NO REFERENCE YET".to_string(),
                };
                ui.label(
                    RichText::new(analyzing_line)
                        .font(FontId::monospace(10.5))
                        .color(ink(128)),
                );
            });
            ui.add_space(14.0);
        }

        // Result card.
        if matches!(self.rec, RecState::Done { .. })
            && let Some(res) = &self.result
        {
            let is_ref = res.is_reference;
            let (badge, badge_bg, badge_fg) = if is_ref {
                ("Reference", teal_a(31), TEAL_DARK)
            } else {
                Self::badge_style(res.match_pct)
            };
            let match_str = Self::match_text(res.match_pct);
            let heading = match (&self.enrolled_id, is_ref) {
                (_, true) => "First capture — enrolls your reference voiceprint".to_string(),
                (Some(id), false) => format!("Similarity vs voiceprint {id}"),
                (None, false) => "No reference enrolled yet".to_string(),
            };
            let blocked = res.blocked.clone();
            let mut save = false;
            let mut discard = false;
            glass(24.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(heading).size(13.0).color(ink(140)));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        pill_badge(ui, badge, badge_bg, badge_fg);
                    });
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(match_str)
                            .font(FontId::monospace(44.0))
                            .color(INK)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("%")
                            .font(FontId::monospace(18.0))
                            .color(ink(128)),
                    );
                });
                ui.add_space(14.0);
                if let Some(reason) = &blocked {
                    // Save is withheld (G2): say why inline; only Discard.
                    ui.label(RichText::new(reason.as_str()).size(12.0).color(AMBER_TEXT));
                    ui.add_space(10.0);
                    discard = pill_button(
                        ui,
                        vec2(ui.available_width(), 46.0),
                        "Discard",
                        white(179),
                        INK,
                        Stroke::new(1.0, ink(31)),
                    );
                } else {
                    ui.horizontal(|ui| {
                        let w = (ui.available_width() - 10.0) / 2.0;
                        save = pill_button(
                            ui,
                            vec2(w, 46.0),
                            "Save session",
                            Color32::from_rgb(17, 166, 166),
                            Color32::WHITE,
                            Stroke::NONE,
                        );
                        ui.add_space(10.0);
                        discard = pill_button(
                            ui,
                            vec2(w, 46.0),
                            "Discard",
                            white(179),
                            INK,
                            Stroke::new(1.0, ink(31)),
                        );
                    });
                }
            });
            if save {
                self.save_session();
            }
            if discard {
                self.rec = RecState::Idle;
                self.result = None;
            }
        }

        // Record button.
        if matches!(self.rec, RecState::Idle | RecState::Recording { .. }) {
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                let (rect, resp) = ui.allocate_exact_size(vec2(100.0, 100.0), Sense::click());
                let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                let c = rect.center();

                if recording {
                    // Two expanding pulse rings, 1.8 s period, 0.9 s apart.
                    for phase in [0.0, 0.5] {
                        let t = ((now / 1.8 + phase).fract()) as f32;
                        let r = 46.0 * (0.9 + 0.65 * t);
                        let a = (0.7 * (1.0 - t) * 255.0) as u8;
                        ui.painter().circle_stroke(
                            c,
                            r,
                            Stroke::new(2.0, Color32::from_rgba_unmultiplied(6, 182, 212, a / 2)),
                        );
                    }
                }

                // Button disc (flat stand-in for the teal→cyan gradient).
                ui.painter()
                    .circle_filled(c, 42.0, Color32::from_rgb(18, 168, 178));
                ui.painter()
                    .circle_stroke(c, 42.0, Stroke::new(1.0, white(120)));
                if recording {
                    let sq = Rect::from_center_size(c, vec2(26.0, 26.0));
                    ui.painter().rect_filled(sq, 7.0, Color32::WHITE);
                } else {
                    mic_glyph(
                        ui.painter(),
                        Rect::from_center_size(c, vec2(30.0, 30.0)),
                        Color32::WHITE,
                        true,
                    );
                }

                if resp.clicked() {
                    if recording {
                        self.stop_rec(now);
                    } else {
                        self.start_rec(now);
                    }
                }

                ui.add_space(12.0);
                let hint = if let RecState::Recording { start } = self.rec {
                    format!(
                        "Recording {:.0} s — tap to stop (auto-stops at {MAX_REC_SECS:.0} s)",
                        now - start
                    )
                } else {
                    "Tap to begin capture · sustained /a/ · 8 s min".to_string()
                };
                ui.label(
                    RichText::new(hint)
                        .font(FontId::monospace(11.0))
                        .color(ink(128)),
                );
            });
        }
    }

    /// Live harmonic-series card: per-partial ladder (dB bars, note names,
    /// cents) plus the timbre metrics strip (tilt, even/odd, singer's formant).
    /// Bozeman's "turning over" gauge, driven by the A2/A1 harmonic ratio —
    /// his own observable for the event (Journal of Singing, 2010): H2
    /// dominant (open timbre, *voce aperta*) on one side, the dominant-
    /// harmonic switch at 0 dB, H1 dominant (turned over, *voce chiusa*) on
    /// the other. The gauge shows sign and crossing only — no target value:
    /// whether singers deliberately tune the crossing is disputed (Sundberg,
    /// Lã & Gill 2013 found no systematic tuning in male opera singers), so
    /// nothing here is presented as a number to hold.
    /// Cents tuning needle: ±50¢ around the current note, glowing teal within
    /// ±5¢ ("in tune"), amber otherwise. Ported from the Personal Harmonic
    /// Identifier prototype's needle meter — previously cents were text-only.
    fn cents_needle_gauge(&self, ui: &mut egui::Ui) {
        const RANGE: f32 = 50.0;
        let valid = self.current_profile.valid && self.current_profile.f0 > 0.0;
        let cents = if valid {
            crate::math::freq_to_note(self.current_profile.f0).map(|n| n.cents)
        } else {
            None
        };
        let in_tune = cents.is_some_and(|c| c.abs() <= 5.0);
        let color = if in_tune { TEAL } else { AMBER };

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("TUNING")
                    .font(FontId::monospace(9.5))
                    .color(ink(115)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let text = match cents {
                    Some(c) if in_tune => format!("IN TUNE  {c:+.0}¢"),
                    Some(c) => format!("{c:+.0}¢"),
                    None => "—".into(),
                };
                ui.label(
                    RichText::new(text)
                        .font(FontId::monospace(10.5))
                        .color(color)
                        .strong(),
                );
            });
        });
        ui.add_space(5.0);

        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 14.0), Sense::hover());
        let track = Rect::from_min_size(
            pos2(rect.left(), rect.center().y - 3.0),
            vec2(rect.width(), 6.0),
        );
        let to_x = |c: f32| -> f32 {
            let t = ((c + RANGE) / (2.0 * RANGE)).clamp(0.0, 1.0);
            track.left() + t * track.width()
        };
        ui.painter().rect_filled(track, 3.0, ink(10));
        let cx = to_x(0.0);
        ui.painter().line_segment(
            [pos2(cx, track.top() - 3.0), pos2(cx, track.bottom() + 3.0)],
            Stroke::new(1.5, ink(90)),
        );
        if let Some(c) = cents {
            let mx = to_x(c.clamp(-RANGE, RANGE));
            let my = track.center().y;
            ui.painter().circle_filled(pos2(mx, my), 5.0, color);
            ui.painter()
                .circle_stroke(pos2(mx, my), 5.0, Stroke::new(1.0, white(230)));
        }
    }

    fn turning_over_gauge(&self, ui: &mut egui::Ui) {
        const RANGE: f32 = 12.0; // displayed span, ± dB of A2/A1
        // Half-width of the amber "turning" band. Display convention, not a
        // measurement claim: ±3 dB (half power) keeps the needle from
        // flickering between states around the dominant-harmonic switch.
        const TURN_ZONE: f32 = 3.0;

        // Positive A2/A1 = H2 dominant = open timbre. The axis keeps the
        // established left-open / right-closed reading, so the needle plots
        // the *negated* value.
        let a2a1 = self.turnover_disp;
        let (state, color) = match a2a1 {
            Some(v) if v > TURN_ZONE => ("OPEN", TEAL),
            Some(v) if v < -TURN_ZONE => ("CLOSED", CYAN_DEEP),
            Some(_) => ("TURNING", AMBER),
            None => ("—", ink(115)),
        };

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("A2 / A1 · PASSAGGIO")
                    .font(FontId::monospace(9.5))
                    .color(ink(115)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let numeric = a2a1
                    .map(|v| format!("{v:+.1} dB"))
                    .unwrap_or_else(|| "—".into());
                ui.label(
                    RichText::new(format!("{state}  {numeric}"))
                        .font(FontId::monospace(10.5))
                        .color(color)
                        .strong(),
                );
            });
        });
        ui.add_space(5.0);

        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
        let track = Rect::from_min_size(
            pos2(rect.left(), rect.center().y - 4.0),
            vec2(rect.width(), 8.0),
        );
        let to_x = |st: f32| -> f32 {
            let t = ((st + RANGE) / (2.0 * RANGE)).clamp(0.0, 1.0);
            track.left() + t * track.width()
        };

        ui.painter().rect_filled(track, 4.0, ink(10));
        // Amber "turning" zone band, centered on the crossing.
        let zone = Rect::from_min_max(
            pos2(to_x(-TURN_ZONE), track.top()),
            pos2(to_x(TURN_ZONE), track.bottom()),
        );
        ui.painter()
            .rect_filled(zone, 4.0, Color32::from_rgba_unmultiplied(217, 119, 6, 46));
        // Crossing tick at 0 dB — the dominant-harmonic switch.
        let cx = to_x(0.0);
        ui.painter().line_segment(
            [pos2(cx, track.top() - 3.0), pos2(cx, track.bottom() + 3.0)],
            Stroke::new(1.5, ink(90)),
        );

        if let Some(v) = a2a1 {
            let mx = to_x((-v).clamp(-RANGE, RANGE));
            let my = track.center().y;
            ui.painter().circle_filled(pos2(mx, my), 6.0, color);
            ui.painter()
                .circle_stroke(pos2(mx, my), 6.0, Stroke::new(1.0, white(230)));
        }
    }

    /// Segmented control selecting which visualization fills the switchable
    /// region: `Ladder | Radial | Spectro | Scope`.
    fn viz_mode_switch(&mut self, ui: &mut egui::Ui) {
        let modes = [
            (VizMode::Radial, "Radial"),
            (VizMode::Spectrogram, "Spectro"),
            (VizMode::Scope, "Scope"),
            (VizMode::Tract, "Tract"),
        ];
        let gap = 4.0;
        let seg_w = (ui.available_width() - gap * (modes.len() - 1) as f32) / modes.len() as f32;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (mode, label) in modes {
                let active = self.viz_mode == mode;
                let (rect, resp) = ui.allocate_exact_size(vec2(seg_w, 26.0), Sense::click());
                let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                let (fill, fg, stroke) = if active {
                    (teal_a(31), TEAL_DARK, Stroke::new(1.0, teal_a(64)))
                } else {
                    (white(120), ink(120), Stroke::new(1.0, ink(20)))
                };
                ui.painter()
                    .rect(rect, 7.0, fill, stroke, StrokeKind::Inside);
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::monospace(10.5),
                    fg,
                );
                if resp.clicked() {
                    self.viz_mode = mode;
                }
            }
        });
    }

    /// Scrolling spectrogram waterfall: time on X (newest at right),
    /// log-frequency (50–5000 Hz) on Y, Viridis intensity. Ingests one new
    /// column per repaint from the latest published magnitude frame, then
    /// composes and uploads the RGBA texture. Renderer ported from Resonator's
    /// `draw_waterfall`.
    fn spectrogram_view(&mut self, ui: &mut egui::Ui) {
        // Pull the latest magnitude frame into the reused scratch buffer.
        {
            let mags = self.spectrum_rx.read();
            self.wf_scratch.clear();
            self.wf_scratch.extend_from_slice(mags);
        }
        // Advance the ring head and rewrite that column along the log-freq axis.
        self.wf_head = (self.wf_head + 1) % SPEC_TIME_COLS;
        {
            let Self {
                wf_cols,
                wf_freq_map,
                wf_scratch,
                wf_head,
                ..
            } = self;
            let col = &mut wf_cols[*wf_head];
            for (cell, span) in col.iter_mut().zip(wf_freq_map.iter()) {
                *cell = spec_db_to_byte(span.sample(wf_scratch));
            }
        }
        // Compose RGBA: display x = time (oldest left, newest right), y = freq.
        let (cols, rows) = (SPEC_TIME_COLS, SPEC_FREQ_ROWS);
        {
            let Self {
                wf_pixels,
                wf_cols,
                wf_lut,
                wf_head,
                ..
            } = self;
            for x in 0..cols {
                let ring_idx = (*wf_head + 1 + x) % cols; // x = cols-1 → newest
                let column = &wf_cols[ring_idx];
                for (y, &byte) in column.iter().enumerate() {
                    let c = wf_lut[byte as usize];
                    let p = (y * cols + x) * 4;
                    wf_pixels[p] = c.r();
                    wf_pixels[p + 1] = c.g();
                    wf_pixels[p + 2] = c.b();
                    wf_pixels[p + 3] = 255;
                }
            }
        }
        let image = ColorImage::from_rgba_unmultiplied([cols, rows], &self.wf_pixels);
        let tex = self.wf_tex.get_or_insert_with(|| {
            ui.ctx()
                .load_texture("voxlab_waterfall", image.clone(), TextureOptions::LINEAR)
        });
        tex.set(image, TextureOptions::LINEAR);

        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 210.0), Sense::hover());
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        ui.painter().rect(
            rect,
            6.0,
            Color32::TRANSPARENT,
            Stroke::new(1.0, ink(30)),
            StrokeKind::Inside,
        );
        // Log-frequency axis labels (white reads over the dark low-magnitude field).
        for (hz, lbl) in [
            (100.0, "100"),
            (300.0, "300"),
            (1000.0, "1k"),
            (3000.0, "3k"),
        ] {
            let t = (SPEC_FMAX_HZ / hz).ln() / (SPEC_FMAX_HZ / SPEC_FMIN_HZ).ln();
            let y = rect.top() + t * rect.height();
            ui.painter().text(
                pos2(rect.left() + 4.0, y),
                Align2::LEFT_CENTER,
                lbl,
                FontId::monospace(8.5),
                white(205),
            );
        }
    }

    /// Oscilloscope: zero-crossing-triggered time-domain trace of the most
    /// recent raw frame, ~3 periods wide when voiced.
    fn scope_view(&mut self, ui: &mut egui::Ui, valid: bool, f0: f32) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 130.0), Sense::hover());
        ui.painter().rect(
            rect,
            6.0,
            ink(6),
            Stroke::new(1.0, ink(20)),
            StrokeKind::Inside,
        );
        ui.painter().line_segment(
            [
                pos2(rect.left(), rect.center().y),
                pos2(rect.right(), rect.center().y),
            ],
            Stroke::new(1.0, ink(18)),
        );

        let sr = self.sample_rate;
        let buf = self.scope_rx.read();
        let n = buf.len();
        if n < 8 {
            return;
        }
        // Trigger on the first rising zero-crossing in the first half.
        let mut start = 0;
        for i in 1..n / 2 {
            if buf[i - 1] <= 0.0 && buf[i] > 0.0 {
                start = i;
                break;
            }
        }
        // ~3.2 periods when voiced, else a fixed window.
        let span = if valid && f0 > 0.0 {
            ((sr / f0 * 3.2) as usize).clamp(64, n - start)
        } else {
            1024.min(n - start)
        };
        if span < 2 {
            return;
        }
        let pts: Vec<Pos2> = (0..span)
            .map(|i| {
                let x = rect.left() + (i as f32 / span as f32) * rect.width();
                let y = rect.center().y - buf[start + i] * rect.height() * 0.46;
                pos2(x, y)
            })
            .collect();
        ui.painter().add(Shape::line(pts, Stroke::new(1.5, TEAL)));
    }

    fn harmonics_card(&mut self, ui: &mut egui::Ui) {
        let valid = self.current_profile.valid && self.current_profile.f0 > 0.0;
        let f0 = self.current_profile.f0;

        glass(24.0).show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            // Header: title + live fundamental note readout.
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Harmonic series")
                        .size(13.0)
                        .color(INK)
                        .strong(),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut readout = if valid {
                        crate::math::freq_to_note(f0)
                            .map(|n| format!("{}{} · {:+.0}¢", n.name, n.octave, n.cents))
                            .unwrap_or_else(|| "—".into())
                    } else {
                        "—".into()
                    };
                    if let Some(v) = self.current_profile.metrics.vibrato {
                        readout += &format!(" · vib {:.1} Hz ±{:.0}¢", v.rate_hz, v.extent_cents);
                    }
                    ui.label(
                        RichText::new(readout)
                            .font(FontId::monospace(13.0))
                            .color(TEAL_DARK)
                            .strong(),
                    );
                });
            });
            ui.add_space(10.0);

            self.cents_needle_gauge(ui);
            ui.add_space(12.0);

            self.turning_over_gauge(ui);
            ui.add_space(12.0);

            // The harmonic ladder is always shown.
            harmonic_ladder(ui, &self.harm_ema, valid, f0);
            ui.add_space(12.0);

            // Secondary switchable visualization below the ladder: radial
            // signature, spectrogram waterfall, or oscilloscope.
            self.viz_mode_switch(ui);
            ui.add_space(10.0);
            match self.viz_mode {
                VizMode::Radial => radial_view(ui, &self.harm_ema, valid),
                VizMode::Spectrogram => self.spectrogram_view(ui),
                VizMode::Scope => self.scope_view(ui, valid, f0),
                VizMode::Tract => self.tract_view(ui),
            }

            // Metrics strip, same pattern as the F0/LEVEL/ELAPSED readouts.
            ui.add_space(9.0);
            let sep = ui.available_rect_before_wrap();
            ui.painter().line_segment(
                [pos2(sep.left(), sep.top()), pos2(sep.right(), sep.top())],
                Stroke::new(1.0, ink(18)),
            );
            ui.add_space(11.0);

            let (tilt, balance, formant_pct) = if valid {
                (
                    crate::math::spectral_tilt_db_per_octave(&self.harm_ema),
                    crate::math::even_odd_balance_db(&self.harm_ema),
                    crate::math::singers_formant_pct(&self.harm_ema, f0),
                )
            } else {
                (None, None, None)
            };
            let fmt = |v: Option<f32>, unit: &str| {
                v.map(|v| format!("{v:+.1} {unit}"))
                    .unwrap_or_else(|| "—".into())
            };
            let steadiness = self.current_profile.metrics.steadiness_cents;
            let rows = [
                [
                    ("TILT", fmt(tilt, "dB/oct")),
                    ("EVEN/ODD", fmt(balance, "dB")),
                    (
                        "SINGER'S FMT",
                        formant_pct
                            .map(|p| format!("{p:.0} %"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                ],
                [
                    ("H1–H2", fmt(self.h1h2_disp, "dB")),
                    (
                        "HNR",
                        self.hnr_disp
                            .map(|v| format!("{v:.0} dB"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                    (
                        "STEADINESS",
                        steadiness
                            .map(|s| format!("{s:.0} ¢"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                ],
                [
                    (
                        "JITTER",
                        self.jitter_disp
                            .map(|v| format!("{v:.2} %"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                    (
                        "SHIMMER",
                        self.shimmer_disp
                            .map(|v| format!("{v:.2} dB"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                    (
                        "CPP",
                        self.cpp_disp
                            .map(|v| format!("{v:.0} dB"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                ],
                [
                    (
                        "CENTROID",
                        self.centroid_disp
                            .map(|v| format!("{v:.0} Hz"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                    (
                        "VOICE CLASS",
                        if valid {
                            crate::math::voice_class(f0).to_string()
                        } else {
                            "—".into()
                        },
                    ),
                    (
                        "BRIGHTNESS",
                        self.centroid_disp
                            .map(crate::math::brightness_class)
                            .unwrap_or("—")
                            .to_string(),
                    ),
                ],
            ];
            for (i, row) in rows.into_iter().enumerate() {
                if i > 0 {
                    ui.add_space(10.0);
                }
                ui.columns(3, |cols| {
                    for (col, (label, value)) in cols.iter_mut().zip(row) {
                        col.label(
                            RichText::new(label)
                                .font(FontId::monospace(9.5))
                                .color(ink(115)),
                        );
                        col.add_space(3.0);
                        col.label(
                            RichText::new(value)
                                .font(FontId::monospace(13.0))
                                .color(INK)
                                .strong(),
                        );
                    }
                });
            }
        });
    }

    fn screen_sessions(&mut self, ui: &mut egui::Ui) {
        self.screen_kicker(ui, "CAPTURE ARCHIVE", "Sessions");
        ui.add_space(16.0);

        // Filter chips.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for (key, label) in [
                (Filter::All, "All"),
                (Filter::Verified, "Verified"),
                (Filter::Flagged, "Flagged"),
            ] {
                let active = self.filter == key;
                let (fill, fg, stroke) = if active {
                    (TEAL, Color32::WHITE, Stroke::NONE)
                } else {
                    (white(158), ink(158), Stroke::new(1.0, white(230)))
                };
                let font = FontId::proportional(13.0);
                let galley = ui.painter().layout_no_wrap(label.into(), font.clone(), fg);
                let size = vec2(galley.size().x + 32.0, 33.0);
                let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
                let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                ui.painter()
                    .rect(rect, size.y / 2.0, fill, stroke, StrokeKind::Inside);
                ui.painter()
                    .text(rect.center(), Align2::CENTER_CENTER, label, font, fg);
                if resp.clicked() {
                    self.filter = key;
                }
            }
        });
        ui.add_space(16.0);

        // Column headers.
        let header_font = FontId::monospace(9.5);
        ui.horizontal(|ui| {
            ui.add_space(GLASS_INSET);
            for (w, label) in [
                (SESSIONS_ID_W, "ID"),
                (0.0, "SUBJECT"),
                (SESSIONS_F0_W, "F0 HZ"),
            ] {
                let text = RichText::new(label)
                    .font(header_font.clone())
                    .color(ink(115));
                let w = if w > 0.0 {
                    w
                } else {
                    // The rows are inset by the card on BOTH sides; the header
                    // only pads its left above, so it gives back the right-hand
                    // inset here to land on the same grid.
                    sessions_subject_w(ui.available_width(), GLASS_INSET)
                };
                ui.allocate_ui_with_layout(
                    vec2(w, 14.0),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.set_width(w);
                        ui.label(text);
                    },
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(GLASS_INSET);
                ui.label(
                    RichText::new("MATCH")
                        .font(header_font.clone())
                        .color(ink(115)),
                );
            });
        });
        ui.add_space(8.0);

        // Rows.
        let rows: Vec<(usize, Session)> = self
            .sessions
            .iter()
            .enumerate()
            .filter(|(_, s)| match self.filter {
                Filter::All => true,
                Filter::Verified => Self::verified(s.match_pct),
                Filter::Flagged => !Self::verified(s.match_pct),
            })
            .map(|(i, s)| (i, s.clone()))
            .collect();

        if rows.is_empty() {
            ui.add_space(24.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(if self.sessions.is_empty() {
                        "No captures yet — record on the Capture tab.\nYour first save enrolls your reference voiceprint."
                    } else {
                        "No sessions match this filter."
                    })
                    .size(12.5)
                    .color(ink(120)),
                );
            });
        }

        for (i, s) in rows {
            let (_, badge_bg, badge_fg) = Self::badge_style(s.match_pct);
            let ir = glass(18.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        vec2(SESSIONS_ID_W, 34.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_width(SESSIONS_ID_W);
                            ui.label(
                                RichText::new(&s.id)
                                    .font(FontId::monospace(12.5))
                                    .color(INK)
                                    .strong(),
                            );
                        },
                    );
                    let subj_w = sessions_subject_w(ui.available_width(), 0.0);
                    ui.allocate_ui_with_layout(
                        vec2(subj_w, 34.0),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.set_width(subj_w);
                            ui.label(RichText::new(&s.subj).size(13.0).color(INK).strong());
                            ui.label(RichText::new(&s.date).size(10.5).color(ink(115)));
                        },
                    );
                    ui.allocate_ui_with_layout(
                        vec2(SESSIONS_F0_W, 34.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_width(SESSIONS_F0_W);
                            ui.label(
                                RichText::new(format!("{:.0}", s.f0))
                                    .font(FontId::monospace(12.5))
                                    .color(ink(179)),
                            );
                        },
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let text = Self::match_text(s.match_pct);
                        let font = FontId::monospace(11.5);
                        let galley =
                            ui.painter()
                                .layout_no_wrap(text.clone(), font.clone(), badge_fg);
                        let size = vec2(galley.size().x + 18.0, 23.0);
                        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                        ui.painter().rect_filled(rect, 11.5, badge_bg);
                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            text,
                            font,
                            badge_fg,
                        );
                    });
                });
            });
            let clicked = ui
                .interact(
                    ir.response.rect,
                    ui.id().with("session").with(i),
                    Sense::click(),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked();
            if clicked {
                self.selected = Some(i);
                self.export_queued = false;
                self.screen = Screen::Detail;
            }
            ui.add_space(8.0);
        }
    }

    fn screen_detail(&mut self, ui: &mut egui::Ui) {
        // Resolve the selected session; fall back to Sessions if the archive is
        // empty or the selection is stale (no unwrap on an empty vec).
        let sel = match self.selected.and_then(|i| self.sessions.get(i)) {
            Some(s) => s.clone(),
            None => {
                self.screen = Screen::Sessions;
                return;
            }
        };
        let (badge, badge_bg, badge_fg) = Self::badge_style(sel.match_pct);
        let ring_color = if Self::verified(sel.match_pct) {
            TEAL
        } else {
            AMBER
        };

        // Back + title.
        ui.horizontal(|ui| {
            let (rect, resp) = ui.allocate_exact_size(vec2(38.0, 38.0), Sense::click());
            let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            ui.painter().circle_filled(rect.center(), 19.0, white(179));
            ui.painter()
                .circle_stroke(rect.center(), 19.0, Stroke::new(1.0, white(230)));
            let c = rect.center();
            let stroke = Stroke::new(2.4, INK);
            ui.painter()
                .line_segment([pos2(c.x + 3.0, c.y - 7.0), pos2(c.x - 3.0, c.y)], stroke);
            ui.painter()
                .line_segment([pos2(c.x - 3.0, c.y), pos2(c.x + 3.0, c.y + 7.0)], stroke);
            if resp.clicked() {
                self.screen = Screen::Sessions;
            }
            ui.add_space(12.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(&sel.id)
                        .font(FontId::monospace(20.0))
                        .color(INK)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{} · {}", sel.subj, sel.date))
                        .size(12.0)
                        .color(ink(128)),
                );
            });
        });
        ui.add_space(16.0);

        // Verification score card.
        glass(24.0).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(86.0, 86.0), Sense::hover());
                ring_gauge(
                    ui.painter(),
                    rect,
                    sel.match_pct.unwrap_or(0.0),
                    ring_color,
                    &Self::match_text(sel.match_pct),
                    None,
                );
                ui.add_space(14.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("Match score").size(12.0).color(ink(140)));
                    ui.add_space(3.0);
                    let vp_line = self
                        .enrolled_id
                        .as_ref()
                        .map(|id| format!("vs voiceprint {id}"))
                        .unwrap_or_else(|| "vs reference voiceprint".into());
                    ui.label(RichText::new(vp_line).size(15.0).color(INK).strong());
                    ui.add_space(8.0);
                    pill_badge(ui, badge, badge_bg, badge_fg);
                });
            });
        });
        ui.add_space(14.0);

        // Acoustic parameters — measured values only (means / stop-time
        // snapshot from the capture; static demo values for seeded rows).
        // Reference ranges are musician guidance, not diagnosis; the neutral
        // dot means "not measured".
        let dot = |ok: bool| if ok { TEAL } else { AMBER };
        let opt_dot = |v: Option<f32>, ok: fn(f32) -> bool| match v {
            Some(v) => dot(ok(v)),
            None => ink(64),
        };
        let opt_val = |v: Option<f32>, unit: &str, digits: usize| match v {
            Some(v) => format!("{v:.digits$} {unit}"),
            None => "—".into(),
        };
        let params: [(String, String, &str, Color32); 8] = [
            (
                "F0 mean".into(),
                format!("{:.1} Hz", sel.f0),
                "85–255 Hz",
                dot((85.0..=255.0).contains(&sel.f0)),
            ),
            (
                "Jitter (local)".into(),
                opt_val(sel.jitter_pct, "%", 2),
                "< 1.04 %",
                opt_dot(sel.jitter_pct, |v| v < 1.04),
            ),
            (
                "Shimmer".into(),
                opt_val(sel.shimmer_db, "dB", 2),
                "< 0.35 dB",
                opt_dot(sel.shimmer_db, |v| v < 0.35),
            ),
            (
                "HNR".into(),
                opt_val(sel.hnr_db, "dB", 1),
                "> 17 dB",
                opt_dot(sel.hnr_db, |v| v > 17.0),
            ),
            (
                "CPP".into(),
                opt_val(sel.cpp_db, "dB", 1),
                "> 11 dB",
                opt_dot(sel.cpp_db, |v| v > 11.0),
            ),
            (
                "H1–H2".into(),
                opt_val(sel.h1_h2_db, "dB", 1),
                "0–10 dB",
                opt_dot(sel.h1_h2_db, |v| (0.0..=10.0).contains(&v)),
            ),
            (
                "Vibrato".into(),
                sel.vibrato
                    .map(|v| format!("{:.1} Hz ±{:.0}¢", v.rate_hz, v.extent_cents))
                    .unwrap_or_else(|| "—".into()),
                "4.5–6.5 Hz",
                match sel.vibrato {
                    Some(v) => dot((4.5..=6.5).contains(&v.rate_hz)),
                    None => ink(64),
                },
            ),
            (
                "Steadiness".into(),
                opt_val(sel.steadiness_cents, "¢", 0),
                "< 15 ¢",
                opt_dot(sel.steadiness_cents, |v| v < 15.0),
            ),
        ];

        // Formant chips: measured values only — an unresolved formant reads
        // "—", and there is no F4 chip (the engine extracts three formants;
        // deriving a fourth from f0 was prototype residue, not a measurement).
        let chips: Vec<(String, Option<f32>)> = match sel.formants {
            Some(f) => f
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    (
                        format!("F{}", i + 1),
                        (f.frequency > 0.0).then_some(f.frequency),
                    )
                })
                .collect(),
            None => (1..=3).map(|i| (format!("F{i}"), None)).collect(),
        };

        glass(24.0).show(ui, |ui| {
            ui.label(
                RichText::new("Acoustic parameters")
                    .size(14.0)
                    .color(INK)
                    .strong(),
            );
            ui.add_space(4.0);
            for (name, value, reference, dot_color) in &params {
                ui.add_space(11.0);
                ui.horizontal(|ui| {
                    // Value column sized for the widest case ("5.1 Hz ±55¢").
                    let name_w = ui.available_width() - 104.0 - 84.0 - 12.0 - 24.0;
                    ui.allocate_ui_with_layout(
                        vec2(name_w, 18.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_width(name_w);
                            ui.label(RichText::new(name).size(13.0).color(ink(191)));
                        },
                    );
                    ui.allocate_ui_with_layout(
                        vec2(104.0, 18.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_width(104.0);
                            ui.label(
                                RichText::new(value)
                                    .font(FontId::monospace(13.0))
                                    .color(INK)
                                    .strong(),
                            );
                        },
                    );
                    ui.allocate_ui_with_layout(
                        vec2(84.0, 18.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_width(84.0);
                            ui.label(
                                RichText::new(*reference)
                                    .font(FontId::monospace(10.5))
                                    .color(ink(107)),
                            );
                        },
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (rect, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                        ui.painter().circle_filled(rect.center(), 4.0, *dot_color);
                    });
                });
                ui.add_space(11.0);
                let sep = ui.available_rect_before_wrap();
                ui.painter().line_segment(
                    [pos2(sep.left(), sep.top()), pos2(sep.right(), sep.top())],
                    Stroke::new(1.0, ink(15)),
                );
            }
            ui.add_space(14.0);
            let n = chips.len();
            ui.columns(n, |cols| {
                for (col, (label, freq)) in cols.iter_mut().zip(&chips) {
                    let rect = col.available_rect_before_wrap();
                    let chip = Rect::from_min_size(rect.min, vec2(rect.width(), 52.0));
                    col.painter().rect(
                        chip,
                        14.0,
                        teal_a(18),
                        Stroke::new(1.0, teal_a(31)),
                        StrokeKind::Inside,
                    );
                    col.painter().text(
                        pos2(chip.center().x, chip.top() + 12.0),
                        Align2::CENTER_CENTER,
                        label,
                        FontId::monospace(9.5),
                        ink(128),
                    );
                    col.painter().text(
                        pos2(chip.center().x, chip.top() + 32.0),
                        Align2::CENTER_CENTER,
                        match freq {
                            Some(v) => format!("{v:.0}"),
                            None => "—".to_string(),
                        },
                        FontId::monospace(13.0),
                        TEAL_DARK,
                    );
                    col.allocate_exact_size(vec2(rect.width(), 52.0), Sense::hover());
                }
            });

            // Personal Harmonic Identifier port: plain-language classifiers
            // derived at render time from the session's stored profile, not
            // pre-rendered strings.
            ui.add_space(10.0);
            let even_odd = crate::math::even_odd_balance_db(&sel.profile);
            let timbre = crate::math::timbre_description(&sel.profile, even_odd);
            let voice_class = crate::math::voice_class(sel.f0);
            let brightness = sel
                .centroid_hz
                .map(crate::math::brightness_class)
                .unwrap_or("—");
            ui.label(
                RichText::new(format!(
                    "Voice Class: {voice_class}  ·  Brightness: {brightness}  ·  Timbre: {timbre}"
                ))
                .size(11.5)
                .color(ink(140)),
            );
        });
        ui.add_space(14.0);

        let label = if self.export_queued {
            "Report queued"
        } else {
            "Export report (PDF)"
        };
        if pill_button(
            ui,
            vec2(ui.available_width(), 48.0),
            label,
            white(179),
            TEAL_DARK,
            Stroke::new(1.0, teal_a(64)),
        ) {
            self.export_queued = true;
        }
    }

    // ── floating tab bar ─────────────────────────────────────────────────────

    fn tab_bar(&mut self, ctx: egui::Context, now: f64) {
        egui::Area::new(egui::Id::new("voxlab_tab_bar"))
            .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -16.0 - BOTTOM_INSET))
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                egui::Frame::default()
                    .fill(white(200))
                    .stroke(Stroke::new(1.0, white(235)))
                    .corner_radius(28.0)
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            for (screen, label) in [
                                (Screen::Overview, "Overview"),
                                (Screen::Capture, "Capture"),
                                (Screen::Sessions, "Sessions"),
                            ] {
                                let active = self.screen == screen
                                    || (screen == Screen::Sessions
                                        && self.screen == Screen::Detail);
                                let fg = if active { TEAL } else { ink(107) };
                                let (rect, resp) =
                                    ui.allocate_exact_size(vec2(86.0, 44.0), Sense::click());
                                let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                                if active {
                                    ui.painter().rect_filled(rect, 22.0, teal_a(26));
                                }
                                let icon = Rect::from_center_size(
                                    pos2(rect.center().x, rect.top() + 15.0),
                                    vec2(20.0, 20.0),
                                );
                                match screen {
                                    Screen::Overview => grid_glyph(ui.painter(), icon, fg),
                                    Screen::Capture => mic_glyph(ui.painter(), icon, fg, false),
                                    _ => list_glyph(ui.painter(), icon, fg),
                                }
                                // Live-capture dot: recording must stay visible
                                // from every screen, not just Capture.
                                if screen == Screen::Capture
                                    && matches!(self.rec, RecState::Recording { .. })
                                {
                                    let pulse = (0.55 + 0.45 * (now * 4.0).sin().abs()) as f32;
                                    ui.painter().circle_filled(
                                        pos2(icon.right() + 3.0, icon.top() + 1.0),
                                        3.5,
                                        Color32::from_rgba_unmultiplied(
                                            239,
                                            68,
                                            68,
                                            (pulse * 255.0) as u8,
                                        ),
                                    );
                                }
                                ui.painter().text(
                                    pos2(rect.center().x, rect.bottom() - 9.0),
                                    Align2::CENTER_CENTER,
                                    label,
                                    FontId::proportional(10.5),
                                    fg,
                                );
                                if resp.clicked() {
                                    self.screen = screen;
                                    // Leaving Overview disarms a half-confirmed
                                    // re-enroll.
                                    self.reenroll_armed = false;
                                }
                            }
                        });
                    });
            });
    }
}

fn grid_glyph(painter: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 20.0;
    let o = rect.min;
    let stroke = Stroke::new(1.7 * s, color);
    for (x, y) in [(2.5, 2.5), (11.5, 2.5), (2.5, 11.5), (11.5, 11.5)] {
        let r = Rect::from_min_size(pos2(o.x + x * s, o.y + y * s), vec2(6.0 * s, 6.0 * s));
        painter.rect(r, 2.0 * s, Color32::TRANSPARENT, stroke, StrokeKind::Middle);
    }
}

fn list_glyph(painter: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 20.0;
    let o = rect.min;
    let stroke = Stroke::new(1.7 * s, color);
    let seg = |x1: f32, y1: f32, x2: f32| {
        [
            pos2(o.x + x1 * s, o.y + y1 * s),
            pos2(o.x + x2 * s, o.y + y1 * s),
        ]
    };
    painter.line_segment(seg(3.0, 5.0, 17.0), stroke);
    painter.line_segment(seg(3.0, 10.0, 17.0), stroke);
    painter.line_segment(seg(3.0, 15.0, 12.0), stroke);
}

// ── canvases (ports of the prototype's <canvas> drawing) ────────────────────

/// 3D wireframe of the vocal tract: 44 cross-section rings swept along a
/// bent centerline (pharynx up, bend at the velum, oral cavity forward),
/// rotated around Y and projected with a simple perspective divide. While
/// recording, the area function ripples. Direct port of the JS `drawTract`.
fn paint_tract(painter: &egui::Painter, rect: Rect, time_ms: f32, rec: bool) {
    const N: usize = 44;
    const M: usize = 18;
    let ang = 0.55 + time_ms * 0.000_32;
    let (cos_a, sin_a) = (ang.cos(), ang.sin());
    let (w, h) = (rect.width(), rect.height());
    let cx = rect.left() + w / 2.0 - 6.0;
    let cy = rect.top() + h / 2.0;
    let (f, s_scale) = (300.0f32, 1.32f32);

    struct Ring {
        pts: Vec<(f32, f32, f32)>, // x, y, depth
        z_avg: f32,
        s: f32,
    }
    let mut rings: Vec<Ring> = Vec::with_capacity(N);

    for i in 0..N {
        let s = i as f32 / (N - 1) as f32;
        let (px, py, tx, ty);
        if s < 0.45 {
            let u = s / 0.45;
            px = 0.0;
            py = u * 90.0;
            tx = 0.0;
            ty = 1.0;
        } else if s < 0.62 {
            let u = (s - 0.45) / 0.17;
            let a = std::f32::consts::PI - u * std::f32::consts::FRAC_PI_2;
            px = 38.0 + 38.0 * a.cos();
            py = 52.0 + 38.0 * a.sin();
            tx = a.sin();
            ty = -a.cos();
        } else {
            let u = (s - 0.62) / 0.38;
            px = 38.0 + u * 74.0;
            py = 90.0;
            tx = 1.0;
            ty = 0.0;
        }

        // Area-function radius, rippling while recording.
        let mut r = 10.5 + 7.0 * (s * std::f32::consts::PI).sin() + 3.0 * (s * 6.3 + 1.7).sin();
        if s > 0.9 {
            r *= 1.0 - (s - 0.9) * 3.2;
        }
        if s < 0.06 {
            r *= 0.55 + s * 7.0;
        }
        if rec {
            r += 2.2 * (time_ms * 0.012 + s * 9.0).sin() + 1.2 * (time_ms * 0.027 + s * 17.0).sin();
        }
        r = r.max(4.0);

        let (nx, ny) = (-ty, tx);
        let (mx, my) = (px - 40.0, py - 46.0);
        let mut pts = Vec::with_capacity(M);
        let mut z_sum = 0.0;
        for j in 0..M {
            let th = j as f32 / M as f32 * std::f32::consts::TAU;
            let x3 = mx + r * th.cos() * nx;
            let y3 = my + r * th.cos() * ny;
            let z3 = r * th.sin();
            let xr = x3 * cos_a + z3 * sin_a;
            let zr = -x3 * sin_a + z3 * cos_a;
            let k = f / (f + zr);
            pts.push((cx + xr * k * s_scale, cy - y3 * k * s_scale, zr));
            z_sum += zr;
        }
        rings.push(Ring {
            pts,
            z_avg: z_sum / M as f32,
            s,
        });
    }

    // Longitudinal lines (every 3rd meridian).
    for j in (0..M).step_by(3) {
        let points: Vec<Pos2> = rings.iter().map(|r| pos2(r.pts[j].0, r.pts[j].1)).collect();
        painter.add(Shape::line(
            points,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(8, 145, 178, 46)),
        ));
    }

    // Rings painted far → near, alpha by depth; every 6th emphasized.
    let mut order: Vec<usize> = (0..N).collect();
    order.sort_by(|&a, &b| {
        rings[b]
            .z_avg
            .partial_cmp(&rings[a].z_avg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for idx in order {
        let ring = &rings[idx];
        let depth = ((ring.z_avg + 30.0) / 60.0).clamp(0.0, 1.0);
        let alpha = 0.10 + depth * 0.42;
        let emph = ((ring.s * (N - 1) as f32).round() as usize).is_multiple_of(6);
        let points: Vec<Pos2> = ring.pts.iter().map(|p| pos2(p.0, p.1)).collect();
        let (color, width) = if emph {
            (
                Color32::from_rgba_unmultiplied(
                    13,
                    148,
                    136,
                    (((alpha + 0.22).min(1.0)) * 255.0) as u8,
                ),
                1.5,
            )
        } else {
            (
                Color32::from_rgba_unmultiplied(8, 145, 178, (alpha * 255.0) as u8),
                1.0,
            )
        };
        painter.add(Shape::closed_line(points, Stroke::new(width, color)));
    }
}

/// Scrolling bar waveform, teal fading up to cyan at the live (right) edge.
fn paint_wave(painter: &egui::Painter, rect: Rect, samples: &[f32]) {
    let n = samples.len().max(1);
    let step = rect.width() / n as f32;
    let mid = rect.center().y;
    for (i, &a) in samples.iter().enumerate() {
        let x = rect.left() + i as f32 * step;
        let t = i as f32 / n as f32;
        // teal 25% → teal 90% → cyan, matching the canvas gradient stops.
        let color = if t < 0.75 {
            let a8 = (64.0 + (t / 0.75) * 166.0) as u8;
            Color32::from_rgba_unmultiplied(14, 148, 136, a8)
        } else {
            let u = (t - 0.75) / 0.25;
            Color32::from_rgb(
                (14.0 + u * 20.0) as u8,
                (148.0 + u * 63.0) as u8,
                (136.0 + u * 102.0) as u8,
            )
        };
        let bar = Rect::from_min_max(
            pos2(x + step * 0.18, mid - a),
            pos2(x + step * 0.82, mid + a),
        );
        painter.rect_filled(bar, 2.0, color);
    }
}

/// Harmonic-amplitude ladder: H1..H16 as horizontal bars in dB relative to the
/// strongest partial (floored at −48 dB), with note + cents per harmonic. This
/// is the default (`VizMode::Ladder`) view; the code is unchanged from when it
/// lived inline in `harmonics_card`.
fn harmonic_ladder(ui: &mut egui::Ui, harm_ema: &[f32], valid: bool, f0: f32) {
    const ROWS: usize = 16;
    let max_amp = harm_ema.iter().take(ROWS).cloned().fold(0.0f32, f32::max);
    let audible = max_amp > 1e-5;
    let label_font = FontId::monospace(10.0);
    for k in 0..ROWS {
        let amp = harm_ema.get(k).copied().unwrap_or(0.0);
        let db = if audible && amp > 1e-6 {
            (20.0 * (amp / max_amp).log10()).max(-48.0)
        } else {
            -48.0
        };
        let frac = 1.0 + db / 48.0;

        ui.horizontal(|ui| {
            let (hr, _) = ui.allocate_exact_size(vec2(28.0, 16.0), Sense::hover());
            ui.painter().text(
                hr.left_center(),
                Align2::LEFT_CENTER,
                format!("H{}", k + 1),
                label_font.clone(),
                ink(115),
            );

            let note = if valid {
                crate::math::freq_to_note((k + 1) as f32 * f0)
                    .map(|n| format!("{}{} {:+.0}¢", n.name, n.octave, n.cents))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            let (nr, _) = ui.allocate_exact_size(vec2(76.0, 16.0), Sense::hover());
            ui.painter().text(
                nr.left_center(),
                Align2::LEFT_CENTER,
                note,
                label_font.clone(),
                ink(166),
            );

            let db_w = 46.0;
            let bar_w = (ui.available_width() - db_w).max(20.0);
            let (br, _) = ui.allocate_exact_size(vec2(bar_w, 16.0), Sense::hover());
            let track = Rect::from_min_size(pos2(br.left(), br.center().y - 4.0), vec2(bar_w, 8.0));
            ui.painter().rect_filled(track, 4.0, ink(10));
            if frac > 0.02 {
                let t = k as f32 / (ROWS - 1) as f32;
                let color = Color32::from_rgb(
                    (14.0 + t * 20.0) as u8,
                    (148.0 + t * 63.0) as u8,
                    (136.0 + t * 102.0) as u8,
                );
                let fill =
                    Rect::from_min_size(track.min, vec2(track.width() * frac.clamp(0.0, 1.0), 8.0));
                ui.painter().rect_filled(fill, 4.0, color);
            }

            let (dr, _) = ui.allocate_exact_size(vec2(db_w, 16.0), Sense::hover());
            let db_str = if audible && db > -48.0 {
                format!("{db:+.0} dB")
            } else {
                "—".into()
            };
            ui.painter().text(
                dr.right_center(),
                Align2::RIGHT_CENTER,
                db_str,
                FontId::monospace(9.5),
                ink(128),
            );
        });
        ui.add_space(2.0);
    }
}

/// Radial harmonic signature: 16 spokes at `2π·i/16 − π/2` (12 o'clock start),
/// radius = amplitude relative to the strongest partial. A closed teal polygon
/// (triangle-fan filled so a concave signature renders correctly) with labeled
/// nodes; idle "—" when unvoiced.
fn radial_view(ui: &mut egui::Ui, harm_ema: &[f32], valid: bool) {
    use std::f32::consts::{FRAC_PI_2, TAU};
    const N: usize = 16;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 240.0), Sense::hover());
    let c = rect.center();
    let radius = rect.height() * 0.40;

    for r in 1..=4 {
        ui.painter()
            .circle_stroke(c, radius * r as f32 / 4.0, Stroke::new(1.0, ink(12)));
    }
    let spoke = |i: usize| i as f32 / N as f32 * TAU - FRAC_PI_2;
    for i in 0..N {
        let a = spoke(i);
        ui.painter().line_segment(
            [c, pos2(c.x + a.cos() * radius, c.y + a.sin() * radius)],
            Stroke::new(1.0, ink(10)),
        );
    }

    let max_amp = harm_ema.iter().take(N).cloned().fold(0.0f32, f32::max);
    if !valid || max_amp <= 1e-5 {
        ui.painter().text(
            c,
            Align2::CENTER_CENTER,
            "—",
            FontId::monospace(20.0),
            ink(90),
        );
        return;
    }

    let pts: Vec<Pos2> = (0..N)
        .map(|i| {
            let a = spoke(i);
            let rel = (harm_ema[i] / max_amp).clamp(0.02, 1.0);
            pos2(c.x + a.cos() * radius * rel, c.y + a.sin() * radius * rel)
        })
        .collect();
    for i in 0..N {
        let j = (i + 1) % N;
        ui.painter().add(Shape::convex_polygon(
            vec![c, pts[i], pts[j]],
            teal_a(28),
            Stroke::NONE,
        ));
    }
    ui.painter()
        .add(Shape::closed_line(pts.clone(), Stroke::new(1.8, TEAL)));
    for (i, p) in pts.iter().enumerate() {
        let (col, r) = if i == 0 { (AMBER, 3.5) } else { (TEAL, 2.2) };
        ui.painter().circle_filled(*p, r, col);
    }
    for i in 0..N {
        let a = spoke(i);
        let lr = radius + 12.0;
        ui.painter().text(
            pos2(c.x + a.cos() * lr, c.y + a.sin() * lr),
            Align2::CENTER_CENTER,
            format!("H{}", i + 1),
            FontId::monospace(8.5),
            ink(120),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pixel tolerance for a column edge: the header and the row reach the same
    /// x by different arithmetic, so the last float bit may differ.
    const ALIGN_EPS: f32 = 0.01;

    fn assert_aligned(label: &str, header: f32, row: f32, col: f32) {
        assert!(
            (header - row).abs() < ALIGN_EPS,
            "{label} misaligned at column width {col}: header {header}, row {row}"
        );
    }

    /// The Sessions header sits on the panel; each row sits inside a `glass`
    /// card that insets it on both sides. Two different expressions therefore
    /// compute what has to be one grid — this pins the invariant they exist to
    /// satisfy. Before the shared constants the SUBJECT column was 4 px out,
    /// and nothing said so.
    #[test]
    fn sessions_header_and_rows_share_one_column_grid() {
        for col in [320.0f32, COL_WIDTH, 900.0] {
            // Header: spans the full content column, padding itself by the
            // card inset at each end.
            let head_id_x = GLASS_INSET;
            let head_subj_x = head_id_x + SESSIONS_ID_W;
            let head_subj_w = sessions_subject_w(col - head_subj_x, GLASS_INSET);
            let head_f0_x = head_subj_x + head_subj_w;
            let head_right = col - GLASS_INSET;

            // Row: the card supplies the inset, so its content box is narrower
            // and its subject column subtracts one fewer term.
            let row_w = col - 2.0 * GLASS_INSET;
            let row_id_x = GLASS_INSET;
            let row_subj_x = row_id_x + SESSIONS_ID_W;
            let row_subj_w = sessions_subject_w(row_w - SESSIONS_ID_W, 0.0);
            let row_f0_x = row_subj_x + row_subj_w;
            let row_right = GLASS_INSET + row_w;

            assert_aligned("ID left edge", head_id_x, row_id_x, col);
            assert_aligned("SUBJECT left edge", head_subj_x, row_subj_x, col);
            assert_aligned("SUBJECT width", head_subj_w, row_subj_w, col);
            assert_aligned("F0 left edge", head_f0_x, row_f0_x, col);
            assert_aligned("MATCH right edge", head_right, row_right, col);
        }
    }

    /// The flexible column has to survive the narrowest layout the app can be
    /// shown at; a negative width would invert the allocation.
    #[test]
    fn subject_column_stays_positive_at_the_narrowest_layout() {
        let row_w = 320.0f32 - 2.0 * GLASS_INSET;
        let subj_w = sessions_subject_w(row_w - SESSIONS_ID_W, 0.0);
        assert!(subj_w > 0.0, "SUBJECT column collapsed to {subj_w}");
    }
}
