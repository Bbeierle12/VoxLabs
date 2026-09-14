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

// Per-screen modules. Every one is an `impl DashboardApp` slice plus the
// helpers only it needs; `use super::*` in each gives it this module's
// imports and every sibling's `pub(super)` items.
mod app;
mod capture;
mod detail;
mod diagnostics;
mod files;
mod gauges;
mod live;
mod model;
mod nav;
mod overview;
#[cfg(not(target_arch = "wasm32"))]
mod pipeline_panel;
mod recording;
mod room;
mod room_probe;
mod room_tv_path;
mod sessions;
mod theme;
mod tract_card;
mod viz;

use self::app::*;
pub(crate) use self::model::Session;
use self::model::*;
use self::theme::*;
use self::tract_card::*;
use self::viz::*;

use crate::concurrency::{AnalysisState, EngineEvent, Telemetry};
use crate::types::{Formant, MAX_PARTIALS, Vibrato, VocalProfile, Voiceprint};
use eframe::egui::{
    self, Align, Align2, Color32, ColorImage, FontId, Layout, Pos2, Rect, RichText, Sense, Shape,
    Stroke, StrokeKind, TextureHandle, TextureOptions, pos2, vec2,
};
use rtrb::Producer;
use std::path::PathBuf;

use crate::import;

// Raw-audio capture export: the real sink on native targets, an inert
// stand-in on web (no disk), so the capture screen compiles unchanged.
#[cfg(not(target_arch = "wasm32"))]
use crate::capture_log::{self as capture_export, CaptureFile};
#[cfg(target_arch = "wasm32")]
use capture_export::CaptureFile;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use triple_buffer::Output;
#[cfg(target_arch = "wasm32")]
mod capture_export {
    use std::path::{Path, PathBuf};

    #[derive(Clone, Debug, PartialEq)]
    pub struct CaptureFile {
        pub path: PathBuf,
        pub seconds: f32,
        pub peak_dbfs: f32,
        pub clipped: usize,
    }

    impl CaptureFile {
        pub fn file_name(&self) -> String {
            self.path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        }
    }

    pub fn arm(_dir: &Path, _sample_rate: u32) -> std::io::Result<PathBuf> {
        Err(std::io::Error::other("no filesystem on web"))
    }

    pub fn disarm() -> Option<CaptureFile> {
        None
    }

    pub fn is_armed() -> bool {
        false
    }
}

/// Where the app keeps things on disk. Any of them may be `None` (the web
/// build has no disk at all); each feature that needs a path degrades on
/// its own — no archive, no raw-capture export, no file import.
#[derive(Clone, Debug, Default)]
pub struct AppPaths {
    /// The reference + session archive (`archive.json`).
    pub store: Option<PathBuf>,
    /// Raw-capture WAV exports.
    pub captures: Option<PathBuf>,
    /// The import folder the Sessions screen lists and analyzes.
    pub imports: Option<PathBuf>,
}

#[derive(PartialEq, Clone, Copy)]
enum Screen {
    Overview,
    Capture,
    Sessions,
    Detail,
    /// Room-noise measurement: ambient floor, calibration, persistent tones.
    /// Everything DETERMINISTIC about the space lives here — fans, HVAC, the
    /// TV's hardware hum. Program audio is out of scope by design.
    Room,
    /// The Engineering Console (see `diagnostics`), opened from the
    /// DIAGNOSTICS chip in every header; not in the tab bar.
    Diagnostics,
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
    /// Coverage counters, incremented once per FRESH analysis profile (not
    /// per repaint): frames with any periodicity, frames surviving the
    /// voicing gates (SNR + hum), and frames meeting every identity bar.
    /// Session-lifetime; the Room screen reports the ratios.
    cov_periodic: u32,
    cov_voiced: u32,
    cov_identity: u32,
    /// Same pair scoped to the current recording, for the stored per-capture
    /// coverage statistic.
    rec_cov_voiced: u32,
    rec_cov_identity: u32,
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
    /// Where raw capture WAVs are written (`None` = no export on this
    /// platform). See `capture_log`.
    capture_dir: Option<PathBuf>,
    /// The import folder: audio files here can be analyzed as captures
    /// (`None` = no file import on this platform). See `import`.
    import_dir: Option<PathBuf>,
    /// A file analysis in flight; while it runs, the capture accumulators
    /// take the file's frames and ignore the microphone.
    import_job: Option<import::Job>,
    /// Cached listing of `import_dir`, refreshed every couple of seconds
    /// while the Sessions screen is up (the folder is small; a stat per
    /// repaint would still be wasteful).
    import_list: Vec<import::Entry>,
    import_list_at: f64,
    /// A file handed over before the first frame (share sheet); started on
    /// the first repaint, when `now` exists.
    pending_import: Option<PathBuf>,
    /// User switch for the raw-audio export (default on: the study needs
    /// the files, and the one-minute cap bounds the size).
    export_audio: bool,
    /// The most recent finished capture file, shown on the result card and
    /// linked into the session on save.
    last_capture: Option<CaptureFile>,
    /// Why the last export could not be started, if it could not.
    capture_error: Option<String>,
    rng: u64,
    /// The pipeline runner's taps and stats, once a shell attaches one
    /// (Android; see `pipeline_panel`). None on desktop until Phase 5c.
    #[cfg(not(target_arch = "wasm32"))]
    pipeline: Option<pipeline_panel::PipelineShell>,
    /// Engineering Console state (see `diagnostics`).
    console: diagnostics::Console,
}

impl eframe::App for DashboardApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Continuous repaint: canvases + live readouts animate every frame.
        ui.ctx().request_repaint();
        let now = ui.input(|i| i.time);

        self.watch_engine(now);
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_pipeline();
        self.advance(now);
        if let Some(path) = self.pending_import.take() {
            self.start_import(path, now);
        }

        let fresh = self.ui_profile_rx.updated();
        let live = if fresh {
            *self.ui_profile_rx.read()
        } else {
            self.current_profile
        };
        if self.import_job.is_some() {
            // A file is being analyzed: its frames drive everything and the
            // microphone is ignored until it ends.
            self.poll_import(now);
        } else {
            self.ingest_profile(live, fresh);
        }
        self.feed_diagnostics();
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
                    // The overlay line and the DIAGNOSTICS chip, above
                    // every screen but the console itself.
                    self.console_header(ui);
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
                        Screen::Room => self.screen_room(ui),
                        Screen::Diagnostics => self.screen_diagnostics(ui, now),
                    }
                    // Clearance for the floating tab bar.
                    ui.add_space(104.0 + BOTTOM_INSET);
                });
            });

        self.tab_bar(ui.ctx().clone(), now);
    }
}
