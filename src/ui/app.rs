//! Construction, tuning constants, persistence, and small state helpers of the dashboard.

use super::*;

/// Sessions scoring at or above this are "Verified", below are "Flagged"
/// (the prototype's `matchThreshold` prop, default 85).
pub(super) const MATCH_THRESHOLD: f32 = 85.0;

/// Minimum wall-clock length and voiced fraction a capture needs before it may
/// enroll as the reference voiceprint — the Capture hint's "8 s min", enforced.
/// A bad reference poisons every later match, so enrollment is gated harder
/// than ordinary session saves.
pub(super) const MIN_ENROLL_SECS: f64 = 8.0;
pub(super) const MIN_ENROLL_VOICED_FRACTION: f32 = 0.5;

/// Recording auto-stops here so a capture left running off-screen can't grow
/// its accumulators unbounded.
pub(super) const MAX_REC_SECS: f64 = 60.0;
/// File import: frames folded into the capture accumulators per repaint.
/// 128 frames is 5.5 s of audio per tick, so a three-minute file lands in
/// under a second of repaints while the screen stays responsive.
pub(super) const IMPORT_FRAMES_PER_TICK: usize = 128;
/// How often the import-folder listing is re-read while it is on screen.
pub(super) const IMPORT_LIST_REFRESH_SECS: f64 = 2.0;
/// Import-folder rows shown on the Sessions screen (newest first).
pub(super) const IMPORT_LIST_MAX: usize = 12;

/// How long the analysis thread may go without publishing a frame — while the
/// microphone is demonstrably live — before the UI calls it stalled. One
/// analysis frame is ~46 ms, so 2 s is ~40 missed frames: far past scheduling
/// jitter, well short of a length a user would sit through wondering.
pub(super) const ANALYSIS_STALL_SECS: f64 = 2.0;

/// Input RMS above which the microphone counts as delivering signal, for the
/// staleness check above. A silent room reads well below this; a stalled
/// analysis thread with a live mic reads well above it.
pub(super) const ANALYSIS_STALL_RMS_FLOOR: f32 = 0.002;

pub(super) const TRACT_Q_EMA_ALPHA: f32 = 0.2;
pub(super) const VTL_EMA_ALPHA: f32 = 0.05;

/// Approximate seconds per analysis frame (2048 samples at ~44.1–48 kHz),
/// for the calibration progress readout only — cosmetic, not used in DSP.
pub(super) const ANALYSIS_FRAME_SECS_APPROX: f32 = 0.046;

/// Articulatory-log ring: capacity and decimation. 4096 samples at one per
/// 150 ms is ~10 minutes of continuous fully-gated phonation; a fixed cap
/// keeps a day-long session's memory bounded.
pub(super) const TRACT_LOG_CAP: usize = 4096;
pub(super) const TRACT_LOG_MIN_DT: f64 = 0.15;
/// Trail dots drawn in the vowel map (newest of the log; older ones fade).
pub(super) const TRACT_MAP_TRAIL: usize = 240;

/// How long a cpal stream error keeps its banner up after the last occurrence.
/// Stream errors are often transient, so the notice expires on its own rather
/// than pinning a warning for the rest of the session.
pub(super) const AUDIO_ERROR_NOTICE_SECS: f64 = 10.0;

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
        paths: AppPaths,
    ) -> Self {
        let AppPaths {
            store: store_path,
            captures: capture_dir,
            imports: import_dir,
        } = paths;
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
            cov_periodic: 0,
            cov_voiced: 0,
            cov_identity: 0,
            rec_cov_voiced: 0,
            rec_cov_identity: 0,
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
            capture_dir,
            import_dir,
            import_job: None,
            import_list: Vec::new(),
            import_list_at: -1e9,
            pending_import: None,
            export_audio: true,
            last_capture: None,
            capture_error: None,
            rng: 0x9E37_79B9_7F4A_7C15,
        }
    }

    /// Analyze a file at startup (the Android share sheet hands one over
    /// before the UI exists). Runs on the first repaint.
    pub fn queue_import(&mut self, path: PathBuf) {
        self.pending_import = Some(path);
    }

    /// Write the current reference + archive to `store_path` (no-op on web).
    /// Called whenever the archive changes. A failed write surfaces in the
    /// notice banner — the UI must never claim a save that didn't stick.
    pub(super) fn persist_state(&mut self) {
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

    pub(super) fn rand01(&mut self) -> f32 {
        // xorshift64 — no rng dependency, prototype-grade randomness.
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u64 << 24) as f32
    }

    pub(super) fn verified(match_pct: Option<f32>) -> bool {
        match_pct.is_some_and(|m| m >= MATCH_THRESHOLD)
    }

    pub(super) fn badge_style(match_pct: Option<f32>) -> (&'static str, Color32, Color32) {
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
    pub(super) fn match_text(match_pct: Option<f32>) -> String {
        match_pct.map_or_else(|| "—".into(), |m| format!("{m:.1}"))
    }

    pub(super) fn input_rms(&self) -> f32 {
        f32::from_bits(self.telemetry.input_rms.load(Ordering::Relaxed))
    }
}
