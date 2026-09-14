//! The Engineering Console's back end — a port of Vocal Tract Lab 0.10.0's
//! diagnostics stack (`DiagnosticsCore.kt`, `DiagnosticStore.kt`,
//! `DiagnosticsRuntime.kt`) into VoxLabs:
//!
//! * [`core`] — the derived-metrics snapshot, the offline rule-based
//!   assistant (same rules, codes, wording and health arithmetic),
//!   events, refinement drafts/records, self-test results;
//! * [`store`] — append-only JSONL under `<files>/diagnostics/`, pruning,
//!   the pending-crash marker, and the export bundle's JSON;
//! * [`runtime`] — the process-wide singleton the shell feeds and the
//!   console reads;
//! * [`self_test`] — the nine on-device checks;
//! * [`export`] — the Downloads write and the share sheet (Android) or a
//!   plain file (desktop), plus app/device facts for the bundle.
//!
//! Privacy stance carried over unchanged: derived metrics only, no raw
//! audio, no upload, no automatic model mutation.

pub mod core;
pub mod export;
pub mod ids;
pub mod runtime;
pub mod self_test;
pub mod store;

pub use core::{
    Assessment, Event, Metrics, RefinementDraft, SelfTestResult, Severity, TractSnapshot,
};
pub use store::{CalibrationReport, CalibrationStep, Export};

/// The room-calibration pass as a calibration report: one derived-only
/// step (ambient floor plus any fingerprinted steady tone), no raw audio,
/// no model change — what the console's calibration section and the bundle
/// carry.
pub fn room_calibration_report(ambient_rms: f32, hum_hz: Option<f32>) -> CalibrationReport {
    let now = ids::now_millis();
    let frames = crate::math::CALIB_FRAMES;
    let floor_db = crate::config::consts::DB_PER_DECADE_AMPLITUDE
        * ambient_rms
            .max(crate::config::NoiseFloorConfig::DEFAULT.floor_min_rms)
            .log10();
    CalibrationReport {
        schema_version: "voxlabs.room-calibration/1.0".into(),
        started_at_millis: now,
        completed_at_millis: now,
        steps: vec![CalibrationStep {
            id: "room_floor".into(),
            frames,
            voiced_frames: 0,
            mean_f0_hz: hum_hz,
            mean_formants_hz: Vec::new(),
            mean_snr_db: floor_db,
            mean_tract_confidence: 0.0,
            abstained_frames: frames,
        }],
        raw_audio_stored: false,
        automatic_model_mutation: false,
    }
}

/// Installs a panic hook that records the panic as a `crash` event (the
/// source app's uncaught-exception handler) before the previous hook runs.
/// The pending-crash marker it leaves is reported by the next launch.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "no message".into());
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown".into());
        let thread = std::thread::current()
            .name()
            .unwrap_or("unnamed")
            .to_string();
        let backtrace = std::backtrace::Backtrace::force_capture().to_string();
        let top_frames: Vec<&str> = backtrace
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .take(core::CFG.crash_top_frames)
            .collect();
        runtime::log_crash(
            &format!("panic: {message}"),
            std::collections::BTreeMap::from([
                ("thread".to_string(), thread),
                ("location".to_string(), location),
                ("top_frames".to_string(), top_frames.join(" | ")),
            ]),
        );
        previous(info);
    }));
}
