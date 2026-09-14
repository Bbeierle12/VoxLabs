//! The Engineering Console — Vocal Tract Lab 0.10.0's diagnostics page
//! (`AdminConsoleActivity.kt`) as a VoxLabs screen: the same sections in
//! the same order with the same copy, buttons and behaviours, fed by the
//! `diagnostics` runtime. Opened from the DIAGNOSTICS chip in every
//! screen's header; "Return to VoxLabs" goes back where the user was.
//!
//! Also here: the per-repaint metrics feed (`feed_diagnostics`) and the
//! optional live overlay line the console's toggle enables.

use super::*;

#[cfg(not(target_arch = "wasm32"))]
use crate::diagnostics::{Metrics, RefinementDraft, Severity, TractSnapshot, runtime};

/// Console-page colours and type sizes, as the source app draws them.
const CONSOLE_BG: Color32 = Color32::from_rgb(244, 248, 252);
const TITLE_INK: Color32 = Color32::from_rgb(20, 48, 68);
const BODY_INK: Color32 = Color32::from_rgb(38, 60, 72);
const CHECK_INK: Color32 = Color32::from_rgb(35, 55, 70);
const TITLE_SIZE: f32 = 27.0;
const SECTION_SIZE: f32 = 19.0;
const BODY_SIZE: f32 = 14.0;
const BUTTON_HEIGHT: f32 = 40.0;

/// Console state the screen keeps between repaints: the cached section
/// texts (refreshed on demand, as the source page does), the form, the
/// clear-confirmation dialog and the toast.
pub(super) struct Console {
    pub(super) return_to: Screen,
    atlas_text: String,
    assistant_text: String,
    calibration_text: String,
    self_test_text: String,
    evidence_text: String,
    event_text: String,
    /// Status of the last fixture-tap record/export (Phase 5a, D5).
    fixture_status: String,
    overlay_enabled: bool,
    expected_f0: String,
    expected_vowel: String,
    expected_resonances: String,
    notes: String,
    approve_training: bool,
    confirm_clear: bool,
    toast: Option<(String, f64)>,
    /// `analysis_frames` when the runtime was last fed.
    last_fed_frames: u32,
}

impl Default for Console {
    fn default() -> Self {
        Self {
            return_to: Screen::Overview,
            atlas_text: String::new(),
            assistant_text: "Analyzing latest metrics…".into(),
            calibration_text: "No completed calibration in this session".into(),
            self_test_text: "Not run in this session".into(),
            evidence_text: String::new(),
            event_text: "No events".into(),
            fixture_status: String::new(),
            overlay_enabled: false,
            expected_f0: String::new(),
            expected_vowel: String::new(),
            expected_resonances: String::new(),
            notes: String::new(),
            approve_training: false,
            confirm_clear: false,
            toast: None,
            last_fed_frames: 0,
        }
    }
}

/// FNV-1a over `text`, as 16 hex characters — the "source lock" shown for
/// each compiled-in model file.
fn source_lock(text: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

impl DashboardApp {
    /// Opens the console from wherever the user is.
    pub(super) fn open_console(&mut self) {
        if self.screen != Screen::Diagnostics {
            self.console.return_to = self.screen;
        }
        self.screen = Screen::Diagnostics;
        #[cfg(not(target_arch = "wasm32"))]
        {
            runtime::note("lifecycle", "admin_opened", "Engineering console opened");
            self.console_refresh();
        }
    }

    fn close_console(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        runtime::note("lifecycle", "admin_closed", "Engineering console closed");
        self.screen = self.console.return_to;
    }

    /// The header-bar row every screen shows: the overlay line (when
    /// enabled) on the left, the DIAGNOSTICS chip on the right.
    pub(super) fn console_header(&mut self, ui: &mut egui::Ui) {
        if self.screen == Screen::Diagnostics {
            return;
        }
        let mut open = false;
        ui.horizontal(|ui| {
            if self.console.overlay_enabled {
                ui.label(
                    RichText::new(self.overlay_line())
                        .font(FontId::monospace(9.5))
                        .color(ink(140)),
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                open = probe_chip(ui, "DIAGNOSTICS");
            });
        });
        ui.add_space(8.0);
        if open {
            self.open_console();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl DashboardApp {
    /// The source app's debug overlay: model id · source lock · DSP time ·
    /// confidence.
    fn overlay_line(&self) -> String {
        let m = runtime::current_metrics();
        format!(
            "{} · {} · DSP {:.1} ms · confidence {}%",
            model_id(),
            &source_lock(crate::pipeline::PipelineDefinition::LIVE_MODEL)[..12],
            m.processing_ms,
            (m.tract_confidence * crate::config::consts::PERCENT).round()
        )
    }

    /// Feeds the runtime the latest derived metrics: once per new analysis
    /// frame, never per repaint. The runtime rate-limits its own sampling.
    pub(super) fn feed_diagnostics(&mut self) {
        let frames = self.telemetry.analysis_frames.load(Ordering::Relaxed);
        if frames == self.console.last_fed_frames {
            return;
        }
        self.console.last_fed_frames = frames;
        runtime::update_state(self.derived_metrics(frames));
    }

    fn derived_metrics(&self, frames: u32) -> Metrics {
        use crate::concurrency::CalibState;
        use crate::math::FormantGrade;
        let p = &self.current_profile;
        let ms = crate::config::consts::MILLIS_PER_SECOND;

        let source = if self.import_job.is_some() {
            "file"
        } else if self.telemetry.audio_unavailable() {
            "idle"
        } else {
            "live"
        };

        // Timing and drops from the runner when one is attached (Android);
        // the desktop engine reports neither.
        let (processing_ms, budget_ms, hop_ms, tap_drops, yin_tap) = match self.pipeline.as_ref() {
            Some(shell) => {
                let load = |a: &std::sync::atomic::AtomicU64| a.load(Ordering::Relaxed) as f32;
                let yin = shell
                    .handle
                    .stages
                    .iter()
                    .position(|(name, _)| name == "yin")
                    .and_then(|i| shell.latest.get(i))
                    .and_then(|m| m.as_ref())
                    .and_then(|m| match &m.value {
                        crate::pipeline::types::Wire::F0Track(t) => Some(*t),
                        _ => None,
                    });
                (
                    load(&shell.handle.stats.last_hop_us) / ms,
                    load(&shell.handle.stats.hop_budget_us) / ms,
                    shell.handle.format.hop as f32 / self.sample_rate * ms,
                    load(&shell.handle.stats.tap_drops) as u64,
                    yin,
                )
            }
            None => {
                let frame_ms = crate::frame::ANALYSIS_FRAME as f32 / self.sample_rate * ms;
                (0.0, frame_ms, frame_ms, 0, None)
            }
        };
        let xruns = u64::from(self.telemetry.xruns.load(Ordering::Relaxed));

        let grade = crate::math::formant_grade(&p.formants, p.formants_f0);
        let (tract_confidence, abstained, reason) = if !p.valid {
            (
                0.0,
                true,
                if self.tract_q.is_some() {
                    "held"
                } else {
                    "unvoiced"
                },
            )
        } else {
            match grade {
                FormantGrade::Identity => (1.0, false, "none"),
                FormantGrade::DisplayOnly => (0.6, false, "none"),
                FormantGrade::Reject => (0.1, true, "formants_suspect"),
            }
        };
        let noisy = p.metrics.voiced_but_noisy;
        let (raw_f0, f0_confidence) = match yin_tap {
            Some(t) => (t.voiced.then_some(t.hz), t.confidence),
            None => (p.valid.then_some(p.f0), if p.valid { 1.0 } else { 0.0 }),
        };
        let pitch_decision = if p.valid {
            "accepted"
        } else if noisy {
            "gated_noise"
        } else if raw_f0.is_some() {
            "gated"
        } else {
            "unvoiced"
        };

        let calib = self.telemetry.calib_state();
        let (ambient, _) = self.telemetry.calibration_summary();
        let noise_state = match calib {
            CalibState::Idle => "learning",
            CalibState::Running => "calibrating",
            CalibState::Done => "calibrated",
            CalibState::FailedVoice => "calibration_failed_voice",
        };
        let noise_confidence = match calib {
            CalibState::Done => 1.0,
            CalibState::Running => {
                1.0 - self.telemetry.calib_frames_left() as f32
                    / crate::math::CALIB_FRAMES.max(1) as f32
            }
            _ => 0.0,
        };
        let noise_floor_db = if calib == CalibState::Done && ambient > 0.0 {
            crate::config::consts::DB_PER_DECADE_AMPLITUDE * ambient.log10()
        } else {
            Metrics::default().noise_floor_db
        };

        let basis = crate::tract::basis_for_vtl(self.vtl_est_cm);
        let (q1, q2) = self.tract_q.unwrap_or((0.0, 0.0));
        Metrics {
            sequence: u64::from(frames),
            source: source.into(),
            processing_ms,
            frame_budget_ms: budget_ms,
            dropped_frames: xruns + tap_drops,
            voiced: p.valid,
            f0_hz: p.valid.then_some(p.f0),
            f0_confidence,
            mean_formant_std_hz: 0.0,
            tract_confidence,
            relative_area_std: 0.0,
            renderer_mode: String::new(),
            microphone_granted: false,
            synthesizer_active: false,
            raw_f0_hz: raw_f0,
            pitch_decision: pitch_decision.into(),
            pitch_rejected: noisy,
            harmonicity: p.metrics.hnr_db.unwrap_or(0.0),
            snr_db: p.metrics.snr_db.unwrap_or(Metrics::default().snr_db),
            noise_floor_db,
            noise_state: noise_state.into(),
            noise_confidence,
            background_changed: false,
            noise_bands_db: Vec::new(),
            posterior_abstained: abstained,
            abstention_reason: reason.into(),
            formant_candidates_hz: p.formants.iter().map(|f| f.frequency).collect(),
            tract: TractSnapshot {
                q1,
                q2,
                vtl_cm: self.vtl_est_cm,
                basis: crate::pipeline::types::BasisId::of(basis).name().into(),
                live: self.tract_live,
                ..Default::default()
            },
            analysis_window_ms: crate::frame::ANALYSIS_FRAME as f32 / self.sample_rate * ms,
            analysis_hop_ms: hop_ms,
        }
    }

    // ── the page ─────────────────────────────────────────────────────────────

    pub(super) fn screen_diagnostics(&mut self, ui: &mut egui::Ui, now: f64) {
        // The source page's flat light background over the app's washes.
        ui.painter().rect_filled(ui.max_rect(), 0.0, CONSOLE_BG);

        title(ui, "Engineering Console", TITLE_SIZE);
        body(
            ui,
            &format!(
                "Session {}\nDerived metrics only • no raw audio • no automatic upload • no automatic model mutation",
                runtime::session_id()
            ),
        );

        section(ui, "Active model");
        body(ui, &self.console.atlas_text);

        section(ui, "Offline diagnostic assistant");
        body(ui, &self.console.assistant_text);
        #[cfg(target_os = "android")]
        if runtime::current_metrics().source == "live"
            && !runtime::current_metrics().microphone_granted
        {
            ui.add_space(4.0);
            if action(
                ui,
                "Open app settings (Permissions → Microphone)",
                full_width(ui),
            ) && !crate::permission::open_app_settings()
            {
                self.toast(
                    "Could not open Settings from here; open it from the launcher.",
                    now,
                );
            }
        }
        let (refresh, self_test) = two_up(ui, "Refresh advice", "Run self-test");
        if refresh {
            self.console_refresh();
        }
        if self_test {
            self.console_run_self_tests();
        }
        let overlay_label = if self.console.overlay_enabled {
            "Disable live overlay"
        } else {
            "Enable live overlay"
        };
        if action(ui, overlay_label, full_width(ui)) {
            runtime::set_debug_overlay_enabled(!self.console.overlay_enabled);
            self.console.overlay_enabled = runtime::debug_overlay_enabled();
        }

        section(ui, "Room calibration");
        body(ui, &self.console.calibration_text);

        section(ui, "Self-test results");
        body(ui, &self.console.self_test_text);

        section(ui, "Evidence");
        body(ui, &self.console.evidence_text);

        section(ui, "Propose a refinement");
        body(
            ui,
            "Corrections are appended as evidence. They never alter the active model. Only enter a target you know independently, such as a reference-tone F0.",
        );
        input(
            ui,
            &mut self.console.expected_f0,
            "Expected F0 Hz (optional)",
        );
        input(
            ui,
            &mut self.console.expected_vowel,
            "Expected vowel label (optional)",
        );
        input(
            ui,
            &mut self.console.expected_resonances,
            "Expected R1–R6 Hz, comma separated (optional)",
        );
        input(
            ui,
            &mut self.console.notes,
            "Observation, environment, or failure notes",
        );
        ui.add_space(4.0);
        ui.checkbox(
            &mut self.console.approve_training,
            RichText::new("Approved for later reviewed training")
                .size(BODY_SIZE)
                .color(CHECK_INK),
        );
        ui.add_space(4.0);
        if action(ui, "Save proposed correction", full_width(ui)) {
            self.console_save_correction(now);
        }

        section(ui, "Cross-target tolerance (D5)");
        body(
            ui,
            "Record runs every mode over its compiled-in fixture on this phone and keeps the taps; export copies them to Downloads/VoxLabs. On the host, `voxlab fixture-taps` records the same and `voxlab compare-taps` sets the [tolerance] bands from the two.",
        );
        let (record, export_taps) = two_up(ui, "Record fixture taps", "Export fixture taps");
        if record {
            match runtime::record_fixture_taps() {
                Ok(lines) => self.console.fixture_status = lines.join("\n"),
                Err(e) => self.console.fixture_status = format!("Record failed: {e}"),
            }
        }
        if export_taps {
            match runtime::export_fixture_taps() {
                Ok(names) => {
                    self.console.fixture_status = format!("Exported: {}", names.join(", "))
                }
                Err(e) => self.console.fixture_status = format!("Export failed: {e}"),
            }
        }
        if !self.console.fixture_status.is_empty() {
            body(ui, &self.console.fixture_status);
        }

        section(ui, "Progress and review bundle");
        body(
            ui,
            "Export writes a JSON bundle to Downloads/VoxLabs and opens Android sharing so it can be attached directly. This app has no Internet permission.",
        );
        if action(ui, "Export and share AI review bundle", full_width(ui)) {
            self.console_export_bundle(now);
        }

        section(ui, "Recent session events");
        body(ui, &self.console.event_text);
        let (refresh_log, clear) = two_up(ui, "Refresh log", "Clear local logs");
        if refresh_log {
            self.console_refresh_events();
        }
        if clear {
            self.console.confirm_clear = true;
        }
        ui.add_space(6.0);
        if action(ui, "Return to VoxLabs", full_width(ui)) {
            self.close_console();
        }

        self.console_confirm_clear_dialog(ui);
        self.console_toast(ui, now);
    }

    fn console_refresh(&mut self) {
        let metrics = runtime::current_metrics();
        let assessment = runtime::current_assessment();
        let findings = if assessment.findings.is_empty() {
            "No current findings.".to_string()
        } else {
            assessment
                .findings
                .iter()
                .map(|f| {
                    format!(
                        "{}: {}\nEvidence: {}\nAction: {}",
                        f.severity.name(),
                        f.title,
                        f.evidence,
                        f.recommended_action
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        };
        let f0 = |v: Option<f32>| {
            v.map(crate::diagnostics::core::one_decimal)
                .unwrap_or_else(|| "null".into())
        };
        let _ = f0;
        self.console.assistant_text = format!(
            "Provider: {} (offline, deterministic)\nHealth score: {}/100\n{}\n\nLatest: source={}, DSP={} ms, drops={}, renderer={}, tract confidence={}%\nSNR={} dB, noise={}, pitch={}, abstained={}\n\n{}",
            assessment.provider_id,
            assessment.health_score,
            assessment.summary,
            metrics.source,
            crate::diagnostics::core::one_decimal(metrics.processing_ms),
            metrics.dropped_frames,
            metrics.renderer_mode,
            (metrics.tract_confidence * crate::config::consts::PERCENT) as i64,
            crate::diagnostics::core::one_decimal(metrics.snr_db),
            metrics.noise_state,
            metrics.pitch_decision,
            metrics.posterior_abstained,
            findings
        );
        self.console.calibration_text = match runtime::current_calibration_report() {
            Some(report) => {
                let frames: u32 = report.steps.iter().map(|s| s.frames).sum();
                format!(
                    "Completed {} derived-only steps • {} frames • no raw audio • model unchanged",
                    report.steps.len(),
                    frames
                )
            }
            None => "No completed calibration in this session".into(),
        };
        self.console.atlas_text = atlas_text();
        let evidence = runtime::current_evidence();
        self.console.evidence_text = format!(
            "{} — {}\n\nBaseline acceptance gates\n{}\n\n{}",
            evidence.banner,
            evidence.banner_text,
            evidence
                .gates
                .iter()
                .map(|g| format!("• {}: {}", g.name, g.status))
                .collect::<Vec<_>>()
                .join("\n"),
            evidence.statement
        );
        self.console.overlay_enabled = runtime::debug_overlay_enabled();
        self.console_refresh_events();
    }

    fn console_refresh_events(&mut self) {
        let events = runtime::recent_events(crate::diagnostics::core::CFG.recent_events_shown);
        self.console.event_text = if events.is_empty() {
            "No events".into()
        } else {
            events
                .iter()
                .rev()
                .map(|e| {
                    format!(
                        "{} {} {}/{}: {}",
                        crate::diagnostics::ids::clock_utc(e.timestamp_millis),
                        e.severity.name(),
                        e.category,
                        e.code,
                        e.message
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
    }

    fn console_run_self_tests(&mut self) {
        let results = runtime::run_self_tests();
        self.console.self_test_text = results
            .iter()
            .map(|r| {
                format!(
                    "{}  {}: {}",
                    if r.passed { "PASS" } else { "CHECK" },
                    r.code,
                    r.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        self.console_refresh();
    }

    fn console_save_correction(&mut self, now: f64) {
        let parse = |s: &str| -> Result<f32, String> {
            s.trim()
                .parse::<f32>()
                .map_err(|_| format!("For input string: \"{}\"", s.trim()))
        };
        let outcome = (|| -> Result<String, String> {
            let f0 = self.console.expected_f0.trim();
            let expected_f0_hz = if f0.is_empty() {
                None
            } else {
                Some(parse(f0)?)
            };
            let resonances = self
                .console
                .expected_resonances
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|t| !t.trim().is_empty())
                .map(parse)
                .collect::<Result<Vec<f32>, String>>()?;
            let draft = RefinementDraft {
                expected_f0_hz,
                expected_vowel: Some(self.console.expected_vowel.clone()),
                expected_resonances_hz: resonances,
                notes: self.console.notes.clone(),
                approved_for_future_training: self.console.approve_training,
            };
            let record = runtime::record_refinement(&draft)?;
            let short: String = record
                .record_id
                .chars()
                .take(crate::diagnostics::core::CFG.short_id_chars)
                .collect();
            Ok(format!("Saved proposal {short}; model unchanged"))
        })();
        match outcome {
            Ok(msg) => {
                self.console.expected_f0.clear();
                self.console.expected_vowel.clear();
                self.console.expected_resonances.clear();
                self.console.notes.clear();
                self.console.approve_training = false;
                self.toast(&msg, now);
                self.console_refresh();
            }
            Err(e) => {
                let msg = if e.is_empty() {
                    "Invalid correction".to_string()
                } else {
                    e
                };
                self.toast(&msg, now);
            }
        }
    }

    fn console_export_bundle(&mut self, now: f64) {
        match runtime::export_bundle()
            .and_then(|export| crate::diagnostics::export::share(&export).map(|()| export))
        {
            Ok(export) => {
                self.toast(
                    &format!(
                        "Saved {} to Downloads/VoxLabs and opened sharing",
                        export.display_name
                    ),
                    now,
                );
                self.console_refresh_events();
            }
            Err(e) => {
                runtime::log(
                    "export",
                    "bundle_export_failed",
                    &e,
                    std::collections::BTreeMap::new(),
                    Severity::Error,
                );
                self.toast(&format!("Export failed: {e}"), now);
            }
        }
    }

    fn console_confirm_clear_dialog(&mut self, ui: &mut egui::Ui) {
        if !self.console.confirm_clear {
            return;
        }
        let mut open = true;
        let mut clear = false;
        let mut cancel = false;
        egui::Window::new("Clear local diagnostics?")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ui.set_max_width(COL_WIDTH - 60.0);
                ui.label(
                    RichText::new(
                        "This removes stored events, corrections, and self-test history from the app. Export first if needed.",
                    )
                    .size(BODY_SIZE)
                    .color(BODY_INK),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        clear = ui.button("Clear").clicked();
                        cancel = ui.button("Cancel").clicked();
                    });
                });
            });
        if clear {
            runtime::clear_logs();
            self.console_refresh();
        }
        if clear || cancel || !open {
            self.console.confirm_clear = false;
        }
    }

    fn toast(&mut self, text: &str, now: f64) {
        self.console.toast = Some((text.to_string(), now));
    }

    fn console_toast(&mut self, ui: &mut egui::Ui, now: f64) {
        let Some((text, since)) = self.console.toast.clone() else {
            return;
        };
        if now - since > crate::diagnostics::core::CFG.toast_secs {
            self.console.toast = None;
            return;
        }
        egui::Area::new(egui::Id::new("voxlab_console_toast"))
            .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -40.0 - BOTTOM_INSET))
            .order(egui::Order::Tooltip)
            .show(ui.ctx(), |ui| {
                egui::Frame::default()
                    .fill(Color32::from_rgba_unmultiplied(20, 48, 68, 230))
                    .corner_radius(18.0)
                    .inner_margin(vec2(16.0, 10.0))
                    .show(ui, |ui| {
                        ui.set_max_width(COL_WIDTH - 40.0);
                        ui.label(RichText::new(text).size(13.0).color(Color32::WHITE));
                    });
            });
    }
}

/// `<crate> <mode> <version>` — the "model id" of what is analyzing.
#[cfg(not(target_arch = "wasm32"))]
fn model_id() -> String {
    let mode = crate::pipeline::PipelineDefinition::live_model()
        .map(|d| d.name)
        .unwrap_or_else(|_| "live_model (failed to load)".into());
    format!(
        "{} {mode} {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )
}

/// The "Active model" section: what the source page says about its atlas,
/// said about VoxLabs' compiled-in mode file and parameters.
#[cfg(not(target_arch = "wasm32"))]
fn atlas_text() -> String {
    match crate::pipeline::PipelineDefinition::live_model() {
        Ok(def) => {
            let stages = def
                .stages
                .iter()
                .map(|s| format!("{}/{}", s.name, s.backend))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{}\nStages {stages}; frame {} hop {}; parameters from pipeline.toml\nSource locks: {}, {}",
                model_id(),
                def.format.frame_samples,
                def.format.hop,
                &source_lock(crate::pipeline::PipelineDefinition::LIVE_MODEL)[..12],
                &source_lock(crate::config::PipelineParams::TOML)[..12]
            )
        }
        Err(e) => format!("Model unavailable: {e}"),
    }
}

fn title(ui: &mut egui::Ui, text: &str, size: f32) {
    ui.label(RichText::new(text).size(size).color(TITLE_INK));
    ui.add_space(8.0);
}

fn section(ui: &mut egui::Ui, text: &str) {
    ui.add_space(18.0);
    ui.label(RichText::new(text).size(SECTION_SIZE).color(TITLE_INK));
    ui.add_space(6.0);
}

fn body(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.add(egui::Label::new(RichText::new(text).size(BODY_SIZE).color(BODY_INK)).wrap());
    ui.add_space(6.0);
}

fn input(ui: &mut egui::Ui, text: &mut String, hint: &str) {
    ui.add_space(4.0);
    ui.add(
        egui::TextEdit::multiline(text)
            .hint_text(hint)
            .desired_rows(1)
            .desired_width(f32::INFINITY)
            .font(FontId::proportional(BODY_SIZE)),
    );
}

fn full_width(ui: &egui::Ui) -> f32 {
    ui.available_width()
}

/// One console button of the given width; returns whether it was clicked.
fn action(ui: &mut egui::Ui, label: &str, width: f32) -> bool {
    ui.add_space(4.0);
    ui.add_sized(
        vec2(width, BUTTON_HEIGHT),
        egui::Button::new(RichText::new(label).size(BODY_SIZE)),
    )
    .clicked()
}

/// Two equal-weight buttons on one row.
fn two_up(ui: &mut egui::Ui, left: &str, right: &str) -> (bool, bool) {
    let gap = 8.0;
    let w = (ui.available_width() - gap) / 2.0;
    let mut out = (false, false);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        out.0 = ui
            .add_sized(
                vec2(w, BUTTON_HEIGHT),
                egui::Button::new(RichText::new(left).size(BODY_SIZE)),
            )
            .clicked();
        out.1 = ui
            .add_sized(
                vec2(w, BUTTON_HEIGHT),
                egui::Button::new(RichText::new(right).size(BODY_SIZE)),
            )
            .clicked();
    });
    out
}

#[cfg(target_arch = "wasm32")]
impl DashboardApp {
    fn overlay_line(&self) -> String {
        String::new()
    }

    pub(super) fn feed_diagnostics(&mut self) {}

    pub(super) fn screen_diagnostics(&mut self, ui: &mut egui::Ui, _now: f64) {
        title(ui, "Engineering Console", TITLE_SIZE);
        body(
            ui,
            "Diagnostics need a data folder; the web build has none. Use the desktop or Android build.",
        );
        if action(ui, "Return to VoxLabs", full_width(ui)) {
            self.close_console();
        }
    }
}
