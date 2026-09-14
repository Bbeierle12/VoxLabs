//! The process-wide diagnostics runtime: one session store, the latest
//! derived metrics, the offline assistant, self-tests, refinements,
//! calibration, export and the debug-overlay preference. A port of
//! `DiagnosticsRuntime.kt`; every function here is one of its members.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::Value;

use super::core::{
    Assessment, CFG, Event, Metrics, RefinementDraft, RefinementRecord, SelfTestResult, Severity,
    assess,
};
use super::ids::{now_millis, uuid_like};
use super::store::{BundleContext, CalibrationReport, Export, Store};

const OVERLAY_PREFS_FILE: &str = "diagnostics_preferences.json";
const OVERLAY_KEY: &str = "show_debug_overlay";

struct State {
    store: Store,
    files_dir: PathBuf,
    metrics: Metrics,
    last_sample_millis: u64,
    last_logged_drops: Option<u64>,
    self_tests: Vec<SelfTestResult>,
    calibration_report: Option<CalibrationReport>,
    shared_model_snapshot: Option<Value>,
    /// The latest provenance record (D11), as written to the store.
    provenance: Option<Value>,
    /// The last validation report and its verdict (`record_validation`).
    validation: Option<(Value, bool)>,
    /// Evidence of the last `runtime/pipeline_report` event.
    last_pipeline_report: Option<BTreeMap<String, String>>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with_state<T>(f: impl FnOnce(&mut State) -> T) -> Option<T> {
    let mut guard = match STATE.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.as_mut().map(f)
}

/// Opens the store under `files_dir/diagnostics/`, logs the session start,
/// and reports a crash the previous launch left behind. A second call in
/// the same process is a no-op.
pub fn initialize(files_dir: &Path) -> Result<(), String> {
    let mut guard = match STATE.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    if guard.is_some() {
        return Ok(());
    }
    let store =
        Store::open(files_dir, Store::new_session_id(now_millis())).map_err(|e| e.to_string())?;
    let pending = store.consume_pending_crash();
    let mut state = State {
        store,
        files_dir: files_dir.to_path_buf(),
        metrics: Metrics::default(),
        last_sample_millis: 0,
        last_logged_drops: None,
        self_tests: Vec::new(),
        calibration_report: None,
        shared_model_snapshot: None,
        provenance: None,
        validation: None,
        last_pipeline_report: None,
    };
    log_locked(
        &state,
        "lifecycle",
        "process_started",
        "Diagnostics session started",
        BTreeMap::from([
            ("privacy".to_string(), "derived_metrics_only".to_string()),
            ("network".to_string(), "disabled".to_string()),
        ]),
        Severity::Info,
    );
    if let Some(crash) = pending {
        let mut evidence = crash.evidence.clone();
        evidence.insert("previous_session".into(), crash.session_id.clone());
        evidence.insert(
            "crash_timestamp_epoch_ms".into(),
            crash.timestamp_millis.to_string(),
        );
        log_locked(
            &state,
            "recovery",
            "previous_crash_recovered",
            &crash.message,
            evidence,
            Severity::Error,
        );
    }
    state.metrics.renderer_mode = renderer_mode_name().into();
    *guard = Some(state);
    Ok(())
}

pub fn is_initialized() -> bool {
    with_state(|_| ()).is_some()
}

/// `glow_gles` on Android (eframe's glow renderer), `wgpu` elsewhere.
pub fn renderer_mode_name() -> &'static str {
    if cfg!(target_os = "android") {
        "glow_gles"
    } else {
        "wgpu"
    }
}

pub fn session_id() -> String {
    with_state(|s| s.store.session_id().to_string()).unwrap_or_else(|| "not-initialized".into())
}

pub fn current_metrics() -> Metrics {
    with_state(|s| s.metrics.clone()).unwrap_or_default()
}

pub fn current_assessment() -> Assessment {
    let m = current_metrics();
    assess(&m, now_millis())
}

pub fn current_calibration_report() -> Option<CalibrationReport> {
    with_state(|s| s.calibration_report.clone()).flatten()
}

pub fn recent_events(limit: usize) -> Vec<Event> {
    with_state(|s| s.store.recent_events(limit)).unwrap_or_default()
}

pub fn self_tests() -> Vec<SelfTestResult> {
    with_state(|s| s.self_tests.clone()).unwrap_or_default()
}

/// D11: files the record beside the session log
/// (`provenance-<pipeline>-<ms>.json`), keeps it for the bundle, and logs
/// `provenance/pipeline_built` with the digests as evidence.
pub fn record_provenance(record: &crate::pipeline::provenance::ProvenanceRecord) {
    let json = record.to_json();
    with_state(|s| {
        let name = format!(
            "provenance-{}-{}.json",
            record.definition.name, record.generated_at_epoch_ms
        );
        let path = s.store.directory().join(&name);
        let written = std::fs::write(&path, &json).is_ok();
        s.provenance = serde_json::from_str(&json).ok();
        let stages: Vec<String> = record
            .stages
            .iter()
            .map(|st| format!("{}/{}@{}", st.name, st.backend, st.version))
            .collect();
        log_locked(
            s,
            "provenance",
            "pipeline_built",
            &format!(
                "pipeline `{}` built: {} stages, {} Hz, frame {} hop {}",
                record.definition.name,
                record.stages.len(),
                record.format.sample_rate_hz,
                record.format.frame_samples,
                record.format.hop
            ),
            BTreeMap::from([
                (
                    "definition_sha256".to_string(),
                    record.definition_sha256.clone(),
                ),
                ("params_sha256".to_string(), record.params_sha256.clone()),
                (
                    "pipeline_toml_sha256".to_string(),
                    record.build.pipeline_toml_sha256.clone(),
                ),
                ("stages".to_string(), stages.join(", ")),
                ("input".to_string(), record.input.kind.clone()),
                (
                    "file".to_string(),
                    if written { name } else { "not written".into() },
                ),
            ]),
            Severity::Info,
        );
    });
}

pub fn current_provenance() -> Option<Value> {
    with_state(|s| s.provenance.clone()).flatten()
}

pub fn set_shared_model_snapshot(snapshot: Value) {
    with_state(|s| s.shared_model_snapshot = Some(snapshot));
}

pub fn update_renderer_mode(mode: &str) {
    with_state(|s| {
        s.metrics.renderer_mode = mode.to_string();
        log_locked(
            s,
            "renderer",
            "renderer_mode",
            "Renderer mode changed",
            BTreeMap::from([("mode".to_string(), mode.to_string())]),
            Severity::Info,
        );
    });
}

pub fn update_microphone_granted(granted: bool) {
    with_state(|s| s.metrics.microphone_granted = granted);
}

pub fn update_synthesizer_active(active: bool) {
    with_state(|s| {
        s.metrics.synthesizer_active = active;
        log_locked(
            s,
            "audio",
            "synth_state",
            if active {
                "Synthesizer started"
            } else {
                "Synthesizer stopped"
            },
            BTreeMap::new(),
            Severity::Info,
        );
    });
}

/// Takes the shell's latest measurements (everything but the renderer,
/// permission and synthesizer flags, which the runtime keeps), then — at
/// most every `sample_interval_ms`, or at once when the drop count changed
/// — assesses them and logs a `derived_state_sample` event.
pub fn update_state(sample: Metrics) {
    with_state(|s| {
        let renderer = std::mem::take(&mut s.metrics.renderer_mode);
        let granted = s.metrics.microphone_granted;
        let synth = s.metrics.synthesizer_active;
        s.metrics = sample;
        s.metrics.renderer_mode = renderer;
        s.metrics.microphone_granted = granted;
        s.metrics.synthesizer_active = synth;

        let now = now_millis();
        let drops = s.metrics.dropped_frames;
        let drops_unchanged = s.last_logged_drops == Some(drops);
        s.last_logged_drops = Some(drops);
        if drops_unchanged
            && now.saturating_sub(s.last_sample_millis) < u64::from(CFG.sample_interval_ms)
        {
            return;
        }
        s.last_sample_millis = now;
        let a = assess(&s.metrics, now);
        let severity = a
            .findings
            .first()
            .map(|f| f.severity)
            .unwrap_or(Severity::Info);
        let m = &s.metrics;
        let opt = |v: Option<f32>| v.map(|x| x.to_string()).unwrap_or_else(|| "null".into());
        let evidence = BTreeMap::from([
            ("source".to_string(), m.source.clone()),
            ("processing_ms".to_string(), m.processing_ms.to_string()),
            ("dropped_frames".to_string(), m.dropped_frames.to_string()),
            ("f0_hz".to_string(), opt(m.f0_hz)),
            ("raw_f0_hz".to_string(), opt(m.raw_f0_hz)),
            ("pitch_decision".to_string(), m.pitch_decision.clone()),
            ("f0_confidence".to_string(), m.f0_confidence.to_string()),
            ("snr_db".to_string(), m.snr_db.to_string()),
            ("noise_state".to_string(), m.noise_state.clone()),
            (
                "background_changed".to_string(),
                m.background_changed.to_string(),
            ),
            (
                "tract_confidence".to_string(),
                m.tract_confidence.to_string(),
            ),
            (
                "posterior_abstained".to_string(),
                m.posterior_abstained.to_string(),
            ),
            ("renderer_mode".to_string(), m.renderer_mode.clone()),
        ]);
        log_locked(
            s,
            "runtime",
            "derived_state_sample",
            &a.summary,
            evidence,
            severity,
        );
    });
}

fn truncate(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

fn log_locked(
    state: &State,
    category: &str,
    code: &str,
    message: &str,
    evidence: BTreeMap<String, String>,
    severity: Severity,
) {
    let event = Event {
        timestamp_millis: now_millis(),
        session_id: state.store.session_id().to_string(),
        category: truncate(category, CFG.category_max_chars),
        code: truncate(code, CFG.code_max_chars),
        severity,
        message: truncate(message, CFG.message_max_chars),
        evidence: evidence
            .into_iter()
            .map(|(k, v)| (k, truncate(&v, CFG.message_max_chars)))
            .collect(),
    };
    let _ = state.store.append(&event);
}

/// Appends one event to the session log. Silent before `initialize`.
pub fn log(
    category: &str,
    code: &str,
    message: &str,
    evidence: BTreeMap<String, String>,
    severity: Severity,
) {
    with_state(|s| {
        if category == "runtime" && code == "pipeline_report" {
            s.last_pipeline_report = Some(evidence.clone());
        }
        log_locked(s, category, code, message, evidence, severity);
    });
}

/// A `vox-validation` verdict for the Evidence output and the bundle.
pub fn record_validation(report: Value, pass: bool) {
    with_state(|s| {
        log_locked(
            s,
            "validation",
            "provenance_round_trip",
            if pass {
                "Provenance round-trip within tolerance bands"
            } else {
                "Provenance round-trip outside tolerance bands"
            },
            BTreeMap::from([("pass".to_string(), pass.to_string())]),
            if pass {
                Severity::Info
            } else {
                Severity::Warning
            },
        );
        s.validation = Some((report, pass));
    });
}

/// The Evidence output assembled from what the runtime knows now.
pub fn current_evidence() -> super::evidence::Evidence {
    with_state(|s| {
        super::evidence::assemble(&super::evidence::EvidenceInputs {
            self_tests: &s.self_tests,
            provenance: s.provenance.as_ref(),
            calibrated: s.calibration_report.is_some(),
            validation: s.validation.as_ref().map(|(v, p)| (v, *p)),
            last_pipeline_report: s.last_pipeline_report.as_ref(),
        })
    })
    .unwrap_or_else(|| {
        super::evidence::assemble(&super::evidence::EvidenceInputs {
            self_tests: &[],
            provenance: None,
            calibrated: false,
            validation: None,
            last_pipeline_report: None,
        })
    })
}

/// `log` with no evidence at `INFO`.
pub fn note(category: &str, code: &str, message: &str) {
    log(category, code, message, BTreeMap::new(), Severity::Info);
}

/// The crash path: never blocks. A panic while the runtime lock is held
/// (or before initialization) is dropped rather than deadlocked on.
pub fn log_crash(message: &str, evidence: BTreeMap<String, String>) {
    let Ok(mut guard) = STATE.try_lock() else {
        return;
    };
    if let Some(s) = guard.as_mut() {
        log_locked(
            s,
            "crash",
            "uncaught_exception",
            message,
            evidence,
            Severity::Error,
        );
    }
}

/// Validates and stores a correction as evidence; the model is untouched.
pub fn record_refinement(draft: &RefinementDraft) -> Result<RefinementRecord, String> {
    let validated = draft.validated()?;
    with_state(|s| {
        let record = RefinementRecord {
            record_id: uuid_like(),
            timestamp_millis: now_millis(),
            session_id: s.store.session_id().to_string(),
            observed: s.metrics.clone(),
            correction: validated,
            status: "proposed".into(),
        };
        s.store.append_refinement(&record)?;
        log_locked(
            s,
            "refinement",
            "correction_proposed",
            "A user correction was saved without mutating the model",
            BTreeMap::from([(
                "approved_for_future_training".to_string(),
                record.correction.approved_for_future_training.to_string(),
            )]),
            Severity::Info,
        );
        Ok(record)
    })
    .unwrap_or_else(|| Err("Diagnostics are not initialized".into()))
}

pub fn record_calibration(report: CalibrationReport) {
    with_state(|s| {
        log_locked(
            s,
            "calibration",
            "guided_calibration_completed",
            "Derived-only room calibration completed",
            BTreeMap::from([
                ("steps".to_string(), report.steps.len().to_string()),
                (
                    "raw_audio_stored".to_string(),
                    report.raw_audio_stored.to_string(),
                ),
                (
                    "automatic_model_mutation".to_string(),
                    report.automatic_model_mutation.to_string(),
                ),
            ]),
            Severity::Info,
        );
        s.calibration_report = Some(report);
    });
}

/// Runs every self-test, keeps the results for the bundle, and logs the
/// summary (`WARNING` when any check failed).
pub fn run_self_tests() -> Vec<SelfTestResult> {
    let files_dir = with_state(|s| s.files_dir.clone());
    let results = super::self_test::run(files_dir.as_deref());
    let passed = results.iter().filter(|r| r.passed).count();
    let failed: Vec<&str> = results
        .iter()
        .filter(|r| !r.passed)
        .map(|r| r.code.as_str())
        .collect();
    let severity = if failed.is_empty() {
        Severity::Info
    } else {
        Severity::Warning
    };
    with_state(|s| {
        s.self_tests = results.clone();
        log_locked(
            s,
            "self_test",
            "self_test_completed",
            &format!("Self-test completed: {passed}/{} passed", results.len()),
            BTreeMap::from([("failed".to_string(), failed.join(", "))]),
            severity,
        );
    });
    results
}

/// Writes the privacy-safe bundle to the Downloads folder and logs it.
pub fn export_bundle() -> Result<Export, String> {
    if !is_initialized() {
        return Err("Diagnostics are not initialized".into());
    }
    if self_tests().is_empty() {
        run_self_tests();
    }
    let ctx = BundleContext {
        app: super::export::app_json(),
        device: super::export::device_json(),
    };
    let evidence = super::evidence::to_json(&current_evidence());
    let (name, text) = with_state(|s| {
        let assessment = assess(&s.metrics, now_millis());
        s.store.build_bundle(
            &ctx,
            &s.metrics,
            &assessment,
            &s.self_tests,
            s.calibration_report.as_ref(),
            s.shared_model_snapshot.as_ref(),
            s.provenance.as_ref(),
            Some(&evidence),
            now_millis(),
        )
    })
    .ok_or_else(|| "Diagnostics are not initialized".to_string())?;
    let export = super::export::write_download(&name, text.as_bytes())?;
    log(
        "export",
        "bundle_exported",
        "Privacy-safe diagnostic bundle exported",
        BTreeMap::from([
            ("display_name".to_string(), export.display_name.clone()),
            ("bytes".to_string(), export.bytes.to_string()),
        ]),
        Severity::Info,
    );
    Ok(export)
}

/// Deletes the local history (events, corrections, self-test results).
pub fn clear_logs() {
    with_state(|s| {
        s.store.clear();
        s.self_tests.clear();
        s.calibration_report = None;
        log_locked(
            s,
            "privacy",
            "logs_cleared",
            "Local diagnostic history was cleared by the user",
            BTreeMap::new(),
            Severity::Info,
        );
    });
}

fn prefs_path() -> Option<PathBuf> {
    with_state(|s| s.files_dir.join(OVERLAY_PREFS_FILE))
}

pub fn debug_overlay_enabled() -> bool {
    let Some(path) = prefs_path() else {
        return false;
    };
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| v.get(OVERLAY_KEY).and_then(Value::as_bool))
        .unwrap_or(false)
}

pub fn set_debug_overlay_enabled(enabled: bool) {
    if let Some(path) = prefs_path() {
        let _ = std::fs::write(
            path,
            serde_json::json!({ OVERLAY_KEY: enabled }).to_string(),
        );
    }
    note(
        "ui",
        "debug_overlay",
        &format!(
            "Live debug overlay {}",
            if enabled { "enabled" } else { "disabled" }
        ),
    );
}
