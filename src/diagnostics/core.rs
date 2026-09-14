//! Diagnostics core: the derived-metrics snapshot, the offline rule-based
//! assistant, findings/assessments, events, refinement drafts and records,
//! self-test results. A port of Vocal Tract Lab 0.10.0's `DiagnosticsCore.kt`
//! (same codes, titles, evidence strings, recommended actions, severities,
//! confidences, ordering and health arithmetic), with VoxLabs' measurements
//! feeding the same fields.

use std::collections::BTreeMap;

use crate::config::DiagnosticsConfig;
use crate::config::consts::PERCENT;

pub(crate) const CFG: DiagnosticsConfig = DiagnosticsConfig::DEFAULT;

/// Severity of a finding or event. Declaration order is the sort order
/// (`ERROR` outranks `WARNING` outranks `INFO`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl Severity {
    pub fn name(self) -> &'static str {
        match self {
            Severity::Info => "INFO",
            Severity::Warning => "WARNING",
            Severity::Error => "ERROR",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "INFO" => Some(Severity::Info),
            "WARNING" => Some(Severity::Warning),
            "ERROR" => Some(Severity::Error),
            _ => None,
        }
    }
}

/// The latest derived metrics the assistant reasons over. Field names and
/// defaults follow `DiagnosticMetrics` in the source app; the doc on each
/// says which VoxLabs measurement fills it.
#[derive(Clone, Debug, PartialEq)]
pub struct Metrics {
    /// Analysis frames published so far.
    pub sequence: u64,
    /// `idle` until audio is up, then `live`, or `file` while a file import runs.
    pub source: String,
    /// Wall time of the last pipeline hop, ms.
    pub processing_ms: f32,
    /// The hop budget, ms.
    pub frame_budget_ms: f32,
    /// Ring-buffer overruns plus dropped tap messages.
    pub dropped_frames: u64,
    /// The frame passed every voicing gate.
    pub voiced: bool,
    /// Accepted f0, Hz.
    pub f0_hz: Option<f32>,
    /// YIN periodicity confidence of the accepted estimate.
    pub f0_confidence: f32,
    /// Not measured in VoxLabs (LPC reports point estimates); stays 0.
    pub mean_formant_std_hz: f32,
    /// Formant reliability grade as a confidence (see `runtime::feed`).
    pub tract_confidence: f32,
    /// Not measured in VoxLabs; stays 0.
    pub relative_area_std: f32,
    /// `glow_gles` on Android, `wgpu` on desktop.
    pub renderer_mode: String,
    pub microphone_granted: bool,
    /// The output (monitor) stream is open.
    pub synthesizer_active: bool,
    /// YIN's estimate before the SNR/hum gates.
    pub raw_f0_hz: Option<f32>,
    /// `accepted`, `gated_noise`, `unvoiced`, or `unprocessed`.
    pub pitch_decision: String,
    /// YIN found a period the gates rejected.
    pub pitch_rejected: bool,
    /// Harmonics-to-noise ratio, dB, when voiced.
    pub harmonicity: f32,
    pub snr_db: f32,
    pub noise_floor_db: f32,
    /// Room-calibration state: `learning`, `calibrating`, `calibrated`,
    /// `calibration_failed_voice`.
    pub noise_state: String,
    pub noise_confidence: f32,
    /// Not detected in VoxLabs; stays false.
    pub background_changed: bool,
    pub noise_bands_db: Vec<f32>,
    /// The tract shape on screen is held, not from this frame.
    pub posterior_abstained: bool,
    pub abstention_reason: String,
    /// F1..F3, Hz.
    pub formant_candidates_hz: Vec<f32>,
    /// Story mode coefficients and length (VoxLabs' articulatory state).
    pub tract: TractSnapshot,
    pub analysis_window_ms: f32,
    pub analysis_hop_ms: f32,
}

/// VoxLabs' stand-in for the source app's articulator posterior.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TractSnapshot {
    pub q1: f32,
    pub q2: f32,
    pub vtl_cm: Option<f32>,
    pub basis: String,
    pub live: bool,
    pub interpretation: String,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            sequence: 0,
            source: "idle".into(),
            processing_ms: 0.0,
            frame_budget_ms: 0.0,
            dropped_frames: 0,
            voiced: false,
            f0_hz: None,
            f0_confidence: 0.0,
            mean_formant_std_hz: 0.0,
            tract_confidence: 0.0,
            relative_area_std: 1.0,
            renderer_mode: "initializing".into(),
            microphone_granted: false,
            synthesizer_active: false,
            raw_f0_hz: None,
            pitch_decision: "unprocessed".into(),
            pitch_rejected: false,
            harmonicity: 0.0,
            snr_db: 30.0,
            noise_floor_db: -120.0,
            noise_state: "stable".into(),
            noise_confidence: 1.0,
            background_changed: false,
            noise_bands_db: Vec::new(),
            posterior_abstained: false,
            abstention_reason: "none".into(),
            formant_candidates_hz: Vec::new(),
            tract: TractSnapshot {
                interpretation:
                    "Story two-mode coefficients; the shape is a fit, not observed anatomy".into(),
                ..Default::default()
            },
            analysis_window_ms: 64.0,
            analysis_hop_ms: 64.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub code: String,
    pub severity: Severity,
    pub title: String,
    pub evidence: String,
    pub recommended_action: String,
    pub confidence: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assessment {
    pub provider_id: String,
    pub generated_at_millis: u64,
    pub health_score: u32,
    pub summary: String,
    pub findings: Vec<Finding>,
    pub automatic_model_mutation_allowed: bool,
}

/// One line of the session log.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub timestamp_millis: u64,
    pub session_id: String,
    pub category: String,
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub evidence: BTreeMap<String, String>,
}

/// What the "Propose a refinement" form submits.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RefinementDraft {
    pub expected_f0_hz: Option<f32>,
    pub expected_vowel: Option<String>,
    pub expected_resonances_hz: Vec<f32>,
    pub notes: String,
    pub approved_for_future_training: bool,
}

impl RefinementDraft {
    /// The source app's `validated()`: range checks, then "needs an expected
    /// value or a note", then trimming.
    pub fn validated(&self) -> Result<Self, String> {
        if let Some(f0) = self.expected_f0_hz
            && (!f0.is_finite() || !(CFG.expected_f0_min_hz..=CFG.expected_f0_max_hz).contains(&f0))
        {
            return Err(format!(
                "Expected F0 must be between {} and {} Hz",
                CFG.expected_f0_min_hz as i64, CFG.expected_f0_max_hz as i64
            ));
        }
        if self.expected_resonances_hz.len() > CFG.expected_resonances_max {
            return Err("At most six expected resonances are supported".into());
        }
        if self
            .expected_resonances_hz
            .iter()
            .any(|r| !r.is_finite() || !(CFG.resonance_min_hz..=CFG.resonance_max_hz).contains(r))
        {
            return Err(format!(
                "Expected resonances must be between {} and {} Hz",
                CFG.resonance_min_hz as i64, CFG.resonance_max_hz as i64
            ));
        }
        let vowel_blank = self
            .expected_vowel
            .as_deref()
            .is_none_or(|v| v.trim().is_empty());
        if self.expected_f0_hz.is_none()
            && vowel_blank
            && self.expected_resonances_hz.is_empty()
            && self.notes.trim().is_empty()
        {
            return Err("A correction needs an expected value or a note".into());
        }
        Ok(Self {
            expected_f0_hz: self.expected_f0_hz,
            expected_vowel: self
                .expected_vowel
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string),
            expected_resonances_hz: self.expected_resonances_hz.clone(),
            notes: self.notes.trim().to_string(),
            approved_for_future_training: self.approved_for_future_training,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RefinementRecord {
    pub record_id: String,
    pub timestamp_millis: u64,
    pub session_id: String,
    pub observed: Metrics,
    pub correction: RefinementDraft,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelfTestResult {
    pub code: String,
    pub passed: bool,
    pub message: String,
}

/// One decimal, as the source app prints floats (`12.3`, `0.0`).
pub fn one_decimal(v: f32) -> String {
    format!("{:.1}", (v * 10.0).round() / 10.0)
}

/// 0..=100 percent of a unit value.
pub fn percent(v: f32) -> i64 {
    (v.clamp(0.0, 1.0) * PERCENT).round() as i64
}

fn opt_one_decimal(v: Option<f32>) -> String {
    v.map(one_decimal).unwrap_or_else(|| "null".into())
}

pub const PROVIDER_ID: &str = "offline_diagnostic_assistant_v1";

/// The deterministic rule set of `OfflineDiagnosticAssistant.assess`.
pub fn assess(m: &Metrics, now_millis: u64) -> Assessment {
    let mut findings: Vec<Finding> = Vec::new();
    let mut add = |code: &str,
                   severity: Severity,
                   title: &str,
                   evidence: String,
                   action: &str,
                   confidence: f32| {
        findings.push(Finding {
            code: code.into(),
            severity,
            title: title.into(),
            evidence,
            recommended_action: action.into(),
            confidence: confidence.clamp(0.0, 1.0),
        });
    };

    if m.renderer_mode == "2d_fallback" {
        add(
            "renderer_fallback",
            Severity::Error,
            "3D renderer is in fallback mode",
            "renderer_mode=2d_fallback".into(),
            "Record the status message, restart once, and export this bundle for shader/context review.",
            0.99,
        );
    }
    if m.processing_ms > m.frame_budget_ms {
        add(
            "frame_deadline_missed",
            Severity::Error,
            "DSP exceeded its frame deadline",
            format!(
                "{} ms > {} ms",
                one_decimal(m.processing_ms),
                one_decimal(m.frame_budget_ms)
            ),
            "Stop other audio modes, repeat the fixture, and inspect sustained device timing and thermals.",
            0.98,
        );
    } else if m.processing_ms > m.frame_budget_ms * CFG.budget_pressure_fraction {
        add(
            "frame_budget_pressure",
            Severity::Warning,
            "DSP is using more than half its frame budget",
            format!(
                "{} of {} ms",
                one_decimal(m.processing_ms),
                one_decimal(m.frame_budget_ms)
            ),
            "Run a sustained test and watch queue drops before accepting this device configuration.",
            0.92,
        );
    }
    if m.dropped_frames > 0 {
        let severity = if m.dropped_frames >= u64::from(CFG.drops_error_threshold) {
            Severity::Error
        } else {
            Severity::Warning
        };
        add(
            "queue_drops",
            severity,
            "Audio frames were dropped",
            format!("dropped_frames={}", m.dropped_frames),
            "Close competing audio workloads and compare drops against processing time and thermal state.",
            0.97,
        );
    }
    if m.source == "live" && !m.microphone_granted {
        add(
            "microphone_permission",
            Severity::Error,
            "Live mode lacks microphone permission",
            "permission=denied".into(),
            "Grant microphone access in Android settings or use the offline demo.",
            1.0,
        );
    }
    if m.voiced && m.f0_hz.is_none() {
        add(
            "voiced_without_f0",
            Severity::Error,
            "Voicing and F0 disagree",
            "voiced=true, f0=null".into(),
            "Capture a diagnostic fixture; this is an internal estimator-consistency error.",
            1.0,
        );
    } else if m.voiced && m.f0_confidence < CFG.f0_confidence_warn {
        add(
            "low_f0_confidence",
            Severity::Warning,
            "Pitch estimate is uncertain",
            format!("f0_confidence={}%", percent(m.f0_confidence)),
            "Reduce background sound, move closer, sustain one pitch, and save the known F0 as a correction.",
            0.9,
        );
    }
    if m.pitch_rejected {
        add(
            "pitch_continuity_intervention",
            Severity::Warning,
            "A pitch discontinuity was corrected or held",
            format!(
                "raw_f0={} Hz; accepted_f0={} Hz; decision={}",
                opt_one_decimal(m.raw_f0_hz),
                opt_one_decimal(m.f0_hz),
                m.pitch_decision
            ),
            "Check the accepted pitch against a reference tone before using this frame as correction evidence.",
            0.96,
        );
    }
    if m.voiced && m.snr_db < CFG.snr_low_db {
        add(
            "low_signal_to_noise",
            Severity::Warning,
            "Singer-to-background ratio is low",
            format!(
                "snr={} dB; noise_state={}",
                one_decimal(m.snr_db),
                m.noise_state
            ),
            "Repeat the background calibration with the fan running, then sing closer to the phone.",
            0.93,
        );
    }
    if m.background_changed {
        add(
            "background_changed",
            Severity::Warning,
            "The stationary background changed",
            format!(
                "noise_state={}; confidence={}%",
                m.noise_state,
                percent(m.noise_confidence)
            ),
            "Pause singing and relearn the background, especially after a fan speed or position change.",
            0.95,
        );
    }
    if m.posterior_abstained {
        add(
            "posterior_abstained",
            Severity::Warning,
            "The 3D posterior abstained",
            format!(
                "reason={}; tract_confidence={}%",
                m.abstention_reason,
                percent(m.tract_confidence)
            ),
            "Treat the mean-shape display as uncertainty, not as the singer's observed anatomy.",
            0.99,
        );
    }
    if let Some(f0) = m.f0_hz
        && !(CFG.f0_unusual_min_hz..=CFG.f0_unusual_max_hz).contains(&f0)
    {
        add(
            "unusual_f0",
            Severity::Warning,
            "Pitch is outside the normal analysis range",
            format!("f0={} Hz", one_decimal(f0)),
            "Confirm the octave against a reference tone before accepting or correcting the estimate.",
            0.8,
        );
    }
    if m.mean_formant_std_hz > CFG.formant_std_warn_hz {
        add(
            "uncertain_resonances",
            Severity::Warning,
            "Resonance estimates are broad",
            format!("mean_std={} Hz", one_decimal(m.mean_formant_std_hz)),
            "Hold a steadier vowel and separate room/device calibration from the singer estimate.",
            0.88,
        );
    }
    if m.tract_confidence < CFG.tract_confidence_error {
        add(
            "very_low_tract_confidence",
            Severity::Error,
            "Tract estimate should not be trusted",
            format!("tract_confidence={}%", percent(m.tract_confidence)),
            "Treat the mesh as abstained; do not use this frame as refinement data.",
            0.98,
        );
    } else if m.tract_confidence < CFG.tract_confidence_warn {
        add(
            "low_tract_confidence",
            Severity::Warning,
            "Tract posterior is weakly constrained",
            format!("tract_confidence={}%", percent(m.tract_confidence)),
            "Repeat with a sustained vowel and only save a correction if the target is independently known.",
            0.94,
        );
    }
    if m.relative_area_std > CFG.area_std_warn {
        add(
            "large_area_uncertainty",
            Severity::Warning,
            "Airway-area uncertainty is large",
            format!("relative_area_std={}%", percent(m.relative_area_std)),
            "Do not interpret local constrictions literally; gather a cleaner or calibrated observation.",
            0.93,
        );
    }

    // Severity descending, then confidence descending (stable).
    findings.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| b.confidence.total_cmp(&a.confidence))
    });

    let penalty: u32 = findings
        .iter()
        .map(|f| match f.severity {
            Severity::Error => CFG.penalty_error,
            Severity::Warning => CFG.penalty_warning,
            Severity::Info => CFG.penalty_info,
        })
        .sum();
    let health_score = CFG.health_max.saturating_sub(penalty).min(CFG.health_max);
    let errors = findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .count();
    let summary = if errors > 0 {
        format!("Action required: {errors} blocking diagnostic finding(s).")
    } else if !findings.is_empty() {
        format!(
            "Review {} warning(s) before accepting a refinement.",
            findings.len()
        )
    } else {
        "No rule-based anomaly is visible in the latest derived metrics.".to_string()
    };
    Assessment {
        provider_id: PROVIDER_ID.into(),
        generated_at_millis: now_millis,
        health_score,
        summary,
        findings,
        automatic_model_mutation_allowed: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy() -> Metrics {
        Metrics {
            source: "live".into(),
            processing_ms: 2.0,
            frame_budget_ms: 21.3,
            voiced: true,
            f0_hz: Some(220.0),
            f0_confidence: 0.9,
            tract_confidence: 0.8,
            relative_area_std: 0.1,
            renderer_mode: "glow_gles".into(),
            microphone_granted: true,
            snr_db: 25.0,
            ..Metrics::default()
        }
    }

    #[test]
    fn healthy_metrics_have_no_findings_and_full_score() {
        let a = assess(&healthy(), 0);
        assert!(a.findings.is_empty());
        assert_eq!(a.health_score, 100);
        assert_eq!(
            a.summary,
            "No rule-based anomaly is visible in the latest derived metrics."
        );
    }

    #[test]
    fn errors_outrank_warnings_and_cost_more() {
        let m = Metrics {
            microphone_granted: false,
            f0_confidence: 0.3,
            ..healthy()
        };
        let a = assess(&m, 0);
        assert_eq!(a.findings[0].code, "microphone_permission");
        assert_eq!(a.findings[0].severity, Severity::Error);
        assert_eq!(a.findings[1].code, "low_f0_confidence");
        assert_eq!(a.health_score, 100 - 25 - 10);
        assert_eq!(
            a.summary,
            "Action required: 1 blocking diagnostic finding(s)."
        );
    }

    #[test]
    fn budget_pressure_then_deadline() {
        let mut m = healthy();
        m.processing_ms = 12.0;
        assert_eq!(assess(&m, 0).findings[0].code, "frame_budget_pressure");
        m.processing_ms = 30.0;
        let a = assess(&m, 0);
        assert_eq!(a.findings[0].code, "frame_deadline_missed");
        assert_eq!(a.findings[0].evidence, "30.0 ms > 21.3 ms");
    }

    #[test]
    fn drafts_are_validated_like_the_source_app() {
        let empty = RefinementDraft::default();
        assert_eq!(
            empty.validated().unwrap_err(),
            "A correction needs an expected value or a note"
        );
        let bad_f0 = RefinementDraft {
            expected_f0_hz: Some(10.0),
            ..Default::default()
        };
        assert_eq!(
            bad_f0.validated().unwrap_err(),
            "Expected F0 must be between 40 and 2000 Hz"
        );
        let ok = RefinementDraft {
            expected_vowel: Some("  a  ".into()),
            notes: " steady ".into(),
            ..Default::default()
        }
        .validated()
        .unwrap();
        assert_eq!(ok.expected_vowel.as_deref(), Some("a"));
        assert_eq!(ok.notes, "steady");
    }

    #[test]
    fn number_formatting() {
        assert_eq!(one_decimal(12.34), "12.3");
        assert_eq!(one_decimal(0.0), "0.0");
        assert_eq!(percent(0.456), 46);
        assert_eq!(percent(2.0), 100);
    }
}
