//! The local diagnostics store: append-only JSONL files under
//! `<files dir>/diagnostics/` — one session file per launch, one shared
//! refinement file, and a pending-crash marker consumed by the next launch.
//! A port of `DiagnosticStore.kt`: same file names, caps, pruning, JSON
//! keys and bundle layout.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use super::core::{
    Assessment, CFG, Event, Finding, Metrics, RefinementRecord, SelfTestResult, Severity,
};

/// The export bundle's schema tag. The layout is the source app's
/// `vocaltract3d.diagnostic-bundle/1.1`; the name says whose data it is.
pub const BUNDLE_SCHEMA: &str = "voxlabs.diagnostic-bundle/1.1";
pub const REFINEMENT_SCHEMA: &str = "voxlabs.refinement/1.0";
/// Download folder shown to the user (`Downloads/VoxLabs`).
pub const DOWNLOAD_SUBDIR: &str = "VoxLabs";

pub struct Store {
    directory: PathBuf,
    session_file: PathBuf,
    refinement_file: PathBuf,
    pending_crash_file: PathBuf,
    session_id: String,
}

/// A written bundle: the file's display name, where it went (a content URI
/// on Android, a path elsewhere), and its size.
#[derive(Clone, Debug, PartialEq)]
pub struct Export {
    pub display_name: String,
    pub uri: String,
    pub bytes: usize,
}

/// Room-calibration outcome kept for the console's "Singer calibration"
/// section and the bundle.
#[derive(Clone, Debug, PartialEq)]
pub struct CalibrationReport {
    pub schema_version: String,
    pub started_at_millis: u64,
    pub completed_at_millis: u64,
    pub steps: Vec<CalibrationStep>,
    pub raw_audio_stored: bool,
    pub automatic_model_mutation: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CalibrationStep {
    pub id: String,
    pub frames: u32,
    pub voiced_frames: u32,
    pub mean_f0_hz: Option<f32>,
    pub mean_formants_hz: Vec<f32>,
    pub mean_snr_db: f32,
    pub mean_tract_confidence: f32,
    pub abstained_frames: u32,
}

/// What the bundle records about the build and the device.
pub struct BundleContext {
    pub app: Value,
    pub device: Value,
}

impl Store {
    /// `files_dir/diagnostics/`; prunes old session files on open.
    pub fn open(files_dir: &Path, session_id: String) -> std::io::Result<Self> {
        if !valid_session_id(&session_id) {
            return Err(std::io::Error::other("Invalid diagnostics session ID"));
        }
        let directory = files_dir.join("diagnostics");
        fs::create_dir_all(&directory)?;
        let store = Self {
            session_file: directory.join(format!("session-{session_id}.jsonl")),
            refinement_file: directory.join("refinements.jsonl"),
            pending_crash_file: directory.join("pending-crash.json"),
            directory,
            session_id,
        };
        store.prune();
        Ok(store)
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// `<millis>-<8 hex>`.
    pub fn new_session_id(now_millis: u64) -> String {
        let hex = super::ids::random_hex(CFG.short_id_chars);
        format!("{now_millis}-{hex}")
    }

    /// Appends one event; `false` when the session file is at its cap.
    pub fn append(&self, event: &Event) -> std::io::Result<bool> {
        if file_len(&self.session_file) >= CFG.max_session_bytes as u64 {
            return Ok(false);
        }
        fs::create_dir_all(&self.directory)?;
        let line = event_to_json(event).to_string();
        append_line(&self.session_file, &line)?;
        if event.category == "crash" {
            fs::write(&self.pending_crash_file, &line)?;
        }
        Ok(true)
    }

    pub fn append_refinement(&self, record: &RefinementRecord) -> Result<(), String> {
        fs::create_dir_all(&self.directory).map_err(|e| e.to_string())?;
        if file_len(&self.refinement_file) >= CFG.max_refinement_bytes as u64 {
            return Err(
                "Refinement store is full; export and clear local diagnostics before adding more"
                    .into(),
            );
        }
        append_line(
            &self.refinement_file,
            &refinement_to_json(record).to_string(),
        )
        .map_err(|e| e.to_string())
    }

    /// The crash event the previous launch left behind, if any (deleted once read).
    pub fn consume_pending_crash(&self) -> Option<Event> {
        if !self.pending_crash_file.is_file() {
            return None;
        }
        let text = fs::read_to_string(&self.pending_crash_file).ok()?;
        let event = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| event_from_json(&v));
        if event.is_some() {
            let _ = fs::remove_file(&self.pending_crash_file);
        }
        event
    }

    /// The newest `limit` events (oldest first), `limit` clamped to
    /// `1..=max_exported_events`.
    pub fn recent_events(&self, limit: usize) -> Vec<Event> {
        let limit = limit.clamp(1, CFG.max_exported_events);
        let all: Vec<Event> = read_lines(&self.session_file)
            .into_iter()
            .filter_map(|l| serde_json::from_str::<Value>(&l).ok())
            .filter_map(|v| event_from_json(&v))
            .collect();
        let skip = all.len().saturating_sub(limit);
        all.into_iter().skip(skip).collect()
    }

    /// The newest `limit` refinement records as raw JSON.
    pub fn refinements(&self, limit: usize) -> Vec<Value> {
        let limit = limit.clamp(1, CFG.max_exported_refinements);
        let all: Vec<Value> = read_lines(&self.refinement_file)
            .into_iter()
            .filter_map(|l| serde_json::from_str::<Value>(&l).ok())
            .collect();
        let skip = all.len().saturating_sub(limit);
        all.into_iter().skip(skip).collect()
    }

    /// Deletes every file directly in the diagnostics directory.
    pub fn clear(&self) {
        if let Ok(entries) = fs::read_dir(&self.directory) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let _ = fs::remove_file(p);
                }
            }
        }
    }

    /// The pretty-printed bundle JSON and its file name.
    #[allow(clippy::too_many_arguments)]
    pub fn build_bundle(
        &self,
        ctx: &BundleContext,
        metrics: &Metrics,
        assessment: &Assessment,
        self_tests: &[SelfTestResult],
        calibration: Option<&CalibrationReport>,
        shared_model: Option<&Value>,
        now_millis: u64,
    ) -> (String, String) {
        let bundle = json!({
            "schema_version": BUNDLE_SCHEMA,
            "generated_at_epoch_ms": now_millis,
            "session_id": self.session_id,
            "privacy": {
                "raw_audio_included": false,
                "internet_upload_performed": false,
                "contents": "derived metrics, app events, explicit user corrections, and self-test results",
            },
            "app": ctx.app,
            "device": ctx.device,
            "latest_metrics": metrics_to_json(metrics),
            "assistant_assessment": assessment_to_json(assessment),
            "self_tests": self_tests.iter().map(self_test_to_json).collect::<Vec<_>>(),
            "singer_calibration": calibration.map(calibration_to_json).unwrap_or(Value::Null),
            "shared_model": shared_model.cloned().unwrap_or(Value::Null),
            "events": self.recent_events(CFG.max_exported_events).iter().map(event_to_json).collect::<Vec<_>>(),
            "refinements": self.refinements(CFG.max_exported_refinements),
        });
        let text = serde_json::to_string_pretty(&bundle).unwrap_or_else(|_| "{}".into());
        let name = format!("voxlabs-diagnostics-{}.json", self.session_id);
        (name, text)
    }

    fn prune(&self) {
        let Ok(entries) = fs::read_dir(&self.directory) else {
            return;
        };
        let mut sessions: Vec<(std::time::SystemTime, PathBuf)> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with("session-") && n.ends_with(".jsonl"))
            })
            .map(|p| {
                let mtime = fs::metadata(&p)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::UNIX_EPOCH);
                (mtime, p)
            })
            .collect();
        sessions.sort_by(|a, b| b.0.cmp(&a.0));
        for (_, p) in sessions.into_iter().skip(CFG.max_session_files) {
            let _ = fs::remove_file(p);
        }
    }
}

fn valid_session_id(id: &str) -> bool {
    (8..=80).contains(&id.len())
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn file_len(p: &Path) -> u64 {
    fs::metadata(p).map(|m| m.len()).unwrap_or(0)
}

fn append_line(p: &Path, line: &str) -> std::io::Result<()> {
    let mut f = fs::OpenOptions::new().create(true).append(true).open(p)?;
    f.write_all(line.as_bytes())?;
    f.write_all(b"\n")
}

fn read_lines(p: &Path) -> Vec<String> {
    let Ok(f) = fs::File::open(p) else {
        return Vec::new();
    };
    BufReader::new(f).lines().map_while(Result::ok).collect()
}

fn opt_f32(v: Option<f32>) -> Value {
    v.map(|x| json!(x)).unwrap_or(Value::Null)
}

pub fn event_to_json(e: &Event) -> Value {
    let evidence: Map<String, Value> = e
        .evidence
        .iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect();
    json!({
        "timestamp_epoch_ms": e.timestamp_millis,
        "session_id": e.session_id,
        "category": e.category,
        "code": e.code,
        "severity": e.severity.name(),
        "message": e.message,
        "evidence": evidence,
    })
}

pub fn event_from_json(v: &Value) -> Option<Event> {
    let evidence: BTreeMap<String, String> = v
        .get("evidence")
        .and_then(Value::as_object)
        .map(|o| {
            o.iter()
                .map(|(k, val)| {
                    let s = match val {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    (k.clone(), s)
                })
                .collect()
        })
        .unwrap_or_default();
    let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_string);
    Some(Event {
        timestamp_millis: v.get("timestamp_epoch_ms")?.as_u64()?,
        session_id: s("session_id")?,
        category: s("category")?,
        code: s("code")?,
        severity: Severity::parse(v.get("severity")?.as_str()?)?,
        message: s("message")?,
        evidence,
    })
}

pub fn metrics_to_json(m: &Metrics) -> Value {
    json!({
        "sequence": m.sequence,
        "source": m.source,
        "processing_ms": m.processing_ms,
        "frame_budget_ms": m.frame_budget_ms,
        "dropped_frames": m.dropped_frames,
        "voiced": m.voiced,
        "f0_hz": opt_f32(m.f0_hz),
        "f0_confidence": m.f0_confidence,
        "raw_f0_hz": opt_f32(m.raw_f0_hz),
        "pitch_decision": m.pitch_decision,
        "pitch_rejected": m.pitch_rejected,
        "harmonicity": m.harmonicity,
        "snr_db": m.snr_db,
        "noise_floor_db": m.noise_floor_db,
        "noise_state": m.noise_state,
        "noise_confidence": m.noise_confidence,
        "background_changed": m.background_changed,
        "noise_bands_db": m.noise_bands_db,
        "mean_formant_std_hz": m.mean_formant_std_hz,
        "formant_candidates_hz": m.formant_candidates_hz,
        "tract_confidence": m.tract_confidence,
        "relative_area_std": m.relative_area_std,
        "posterior_abstained": m.posterior_abstained,
        "abstention_reason": m.abstention_reason,
        "analysis_window_ms": m.analysis_window_ms,
        "analysis_hop_ms": m.analysis_hop_ms,
        "articulators": Value::Null,
        "tract_params": {
            "q1": m.tract.q1,
            "q2": m.tract.q2,
            "vtl_cm": opt_f32(m.tract.vtl_cm),
            "basis": m.tract.basis,
            "live": m.tract.live,
            "interpretation": m.tract.interpretation,
        },
        "renderer_mode": m.renderer_mode,
        "microphone_granted": m.microphone_granted,
        "synthesizer_active": m.synthesizer_active,
    })
}

fn finding_to_json(f: &Finding) -> Value {
    json!({
        "code": f.code,
        "severity": f.severity.name(),
        "title": f.title,
        "evidence": f.evidence,
        "recommended_action": f.recommended_action,
        "confidence": f.confidence,
    })
}

pub fn assessment_to_json(a: &Assessment) -> Value {
    json!({
        "provider_id": a.provider_id,
        "generated_at_epoch_ms": a.generated_at_millis,
        "health_score": a.health_score,
        "summary": a.summary,
        "automatic_model_mutation_allowed": a.automatic_model_mutation_allowed,
        "findings": a.findings.iter().map(finding_to_json).collect::<Vec<_>>(),
    })
}

pub fn self_test_to_json(t: &SelfTestResult) -> Value {
    json!({ "code": t.code, "passed": t.passed, "message": t.message })
}

pub fn calibration_to_json(c: &CalibrationReport) -> Value {
    json!({
        "schema_version": c.schema_version,
        "started_at_epoch_ms": c.started_at_millis,
        "completed_at_epoch_ms": c.completed_at_millis,
        "raw_audio_stored": c.raw_audio_stored,
        "automatic_model_mutation": c.automatic_model_mutation,
        "steps": c.steps.iter().map(|s| json!({
            "id": s.id,
            "frames": s.frames,
            "voiced_frames": s.voiced_frames,
            "mean_f0_hz": opt_f32(s.mean_f0_hz),
            "mean_formants_hz": s.mean_formants_hz,
            "mean_snr_db": s.mean_snr_db,
            "mean_tract_confidence": s.mean_tract_confidence,
            "abstained_frames": s.abstained_frames,
        })).collect::<Vec<_>>(),
    })
}

pub fn refinement_to_json(r: &RefinementRecord) -> Value {
    json!({
        "schema_version": REFINEMENT_SCHEMA,
        "record_id": r.record_id,
        "timestamp_epoch_ms": r.timestamp_millis,
        "session_id": r.session_id,
        "status": r.status,
        "observed": metrics_to_json(&r.observed),
        "correction": {
            "expected_f0_hz": opt_f32(r.correction.expected_f0_hz),
            "expected_vowel": r.correction.expected_vowel.clone().map(Value::String).unwrap_or(Value::Null),
            "expected_resonances_hz": r.correction.expected_resonances_hz,
            "notes": r.correction.notes,
            "approved_for_future_training": r.correction.approved_for_future_training,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "voxlabs-diag-{tag}-{}-{}",
            std::process::id(),
            super::super::ids::random_hex(8)
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn event(store: &Store, category: &str, code: &str) -> Event {
        Event {
            timestamp_millis: 1,
            session_id: store.session_id().to_string(),
            category: category.into(),
            code: code.into(),
            severity: Severity::Info,
            message: "m".into(),
            evidence: BTreeMap::from([("k".to_string(), "v".to_string())]),
        }
    }

    #[test]
    fn events_round_trip_and_tail_in_order() {
        let dir = temp_dir("events");
        let store = Store::open(&dir, Store::new_session_id(5)).unwrap();
        for i in 0..5 {
            assert!(store.append(&event(&store, "c", &format!("e{i}"))).unwrap());
        }
        let last = store.recent_events(2);
        assert_eq!(last.len(), 2);
        assert_eq!(last[0].code, "e3");
        assert_eq!(last[1].code, "e4");
        assert_eq!(last[1].evidence["k"], "v");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn crash_events_leave_a_pending_marker_consumed_once() {
        let dir = temp_dir("crash");
        let store = Store::open(&dir, Store::new_session_id(5)).unwrap();
        store.append(&event(&store, "crash", "boom")).unwrap();
        let again = Store::open(&dir, Store::new_session_id(6)).unwrap();
        let crash = again.consume_pending_crash().unwrap();
        assert_eq!(crash.code, "boom");
        assert!(again.consume_pending_crash().is_none());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn clear_removes_files_and_bundle_has_the_schema() {
        let dir = temp_dir("clear");
        let store = Store::open(&dir, Store::new_session_id(5)).unwrap();
        store.append(&event(&store, "c", "e")).unwrap();
        let ctx = BundleContext {
            app: json!({"package": "test"}),
            device: json!({}),
        };
        let (name, text) = store.build_bundle(
            &ctx,
            &Metrics::default(),
            &super::super::core::assess(&Metrics::default(), 0),
            &[],
            None,
            None,
            0,
        );
        assert!(name.starts_with("voxlabs-diagnostics-"));
        let v: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["schema_version"], BUNDLE_SCHEMA);
        assert_eq!(v["events"].as_array().unwrap().len(), 1);
        assert_eq!(v["privacy"]["raw_audio_included"], false);
        store.clear();
        assert!(store.recent_events(10).is_empty());
        let _ = fs::remove_dir_all(dir);
    }
}
