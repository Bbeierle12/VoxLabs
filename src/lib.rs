//! Voice Harmonic Engine — crate root.
//!
//! This library owns every module and the shared native (desktop) runner. The
//! three entry points live at the edges:
//!   * desktop  — `main.rs`'s `main()` calls [`run`];
//!   * web/wasm — `main.rs`'s wasm `main()` drives eframe's `WebRunner`;
//!   * Android  — the `android` module's `android_main` (compiled into the
//!     cdylib the APK loads).
//!
//! All three launch the same [`DashboardApp`] egui UI.

// Real audio I/O (cpal): desktop + Android; gated out of wasm (Web Audio there).
#[cfg(not(target_arch = "wasm32"))]
mod audio;

// The shell side shared by the phone and the desktop: the switchable live
// engine over the runner, and the hop consumers the egui screens read.
#[cfg(not(target_arch = "wasm32"))]
pub mod shell;

// Android entry point: `android_main`, wired to android-activity's
// NativeActivity glue. Gated so desktop and web are untouched.
#[cfg(target_os = "android")]
mod android;
// "Share → VoxLabs": reads the launching intent's audio stream into the
// import folder. JNI, Android-only.
#[cfg(target_os = "android")]
mod share_intent;
// Runtime RECORD_AUDIO request + grant polling. JNI, Android-only.
#[cfg(target_os = "android")]
mod permission;

// Audio-file decoding + resampling for the in-app import path and the
// `voxlab` harness. Native only (the web build has no files).
#[cfg(not(target_arch = "wasm32"))]
pub mod audio_file;
// Raw-audio export of a running capture (WAV mirror of the analysis
// frames), so a phone capture can go through the `voxlab` harness like a
// dataset file. Desktop + Android; the web target has no disk.
#[cfg(not(target_arch = "wasm32"))]
pub mod capture_log;
mod concurrency;
// The Engineering Console's back end (events, assistant, self-tests,
// export): the diagnostics stack ported from Vocal Tract Lab. Native only
// (it owns files).
#[cfg(not(target_arch = "wasm32"))]
pub mod diagnostics;
// Routes `log` warnings/errors into the diagnostics session log.
#[cfg(not(target_arch = "wasm32"))]
pub mod diag;
// Stage configuration: every tunable number, mirrored in pipeline.toml.
pub mod config;
// The analysis pipeline (Plan v3 Phase 1): Stage trait, builder, runner,
// taps, and the wrapped kernels. Cross-target except the runner thread.
pub mod pipeline;
// In-app file import: a file through the capture pipeline. Cross-target
// (inert on web).
// (The job and its messages are only constructed on native targets; the web
// build compiles the types for the UI and never starts one.)
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
mod import;
// Voice-part (Fach) measurements: FHE, LTAS, cluster stats, tessitura,
// turnover, dominant harmonic, register events. Pure math, cross-target,
// public for the study harness.
pub mod fach;
// Capture-stack capability probe (UNPROCESSED support, microphone inventory,
// channel independence). Report/analysis types compile everywhere so the UI
// and tests are cross-target; the JNI half is Android-only. The dead_code
// allow is desktop-only: there the UI matches on these types but nothing
// constructs them (only the Android probe does), which is exactly the shape
// dead-code analysis flags.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
mod device_probe;
// SHA-256 for provenance digests (no dependency).
pub mod atlas;
pub mod choir;
pub mod hash;
pub mod math;
// The room as calibration learned it (floor seed, hum), shared between the
// calibration pass and the `voicing` stage.
pub mod room;
// TV-path spatial calibration + live path-consistency scoring. The math is
// cross-target and unit-tested; the continuous two-channel capture thread
// is Android-only (same dead-code shape as device_probe on desktop).
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
mod spatial;
// Disk persistence for the enrolled reference + session archive. Compiled on
// all targets (ui uses it); each entry point supplies the store path (or None
// on web, which has no disk).
mod persist;
// f0-contour metrics (vibrato/steadiness): the `contour` stage's kernel.
// Pure math, every target.
pub mod metrics;
// The shared per-frame CPU pipeline (Android loop + the `voxlab` study
// harness). Public so the harness binary can drive it.
#[cfg(not(target_arch = "wasm32"))]
pub mod frame;
// Scrolling-spectrogram STFT. Compiled on all targets (its consts size the
// UI's waterfall buffers); the engine itself is driven only by the desktop
// and Android analysis loops.
// The reference STFT (the `stft` stage's contract) and the waterfall's bin
// constants; nothing runs it live any more (D15).
#[allow(dead_code)]
mod spectrogram;
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
mod synthesis;
// Story two-mode vocal tract model: runtime (tract) + published basis data
// (tract_data). Compiled on all targets — pure math, no threads or I/O.
pub mod tract;
mod tract_data;
pub mod types;
mod ui;

pub use concurrency::ConcurrencyBridges;
pub use ui::{AppPaths, DashboardApp};

/// Desktop native entry point. Starts the pipeline runner on the live audio
/// stream (the same `shell::engine` the phone uses — D15: the desktop is
/// the CPU pipeline), the cpal audio engine, and the egui dashboard.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn run() -> anyhow::Result<()> {
    use crate::audio::AudioEngine;
    use crate::concurrency::AnalysisState;
    use crate::shell::consumers::WireConsumers;
    use crate::shell::engine::{self, LiveEngine};
    use cpal::traits::{DeviceTrait, HostTrait};
    use std::path::Path;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex};

    let logger = env_logger::Builder::from_default_env().build();
    log::set_max_level(logger.filter());
    let _ = log::set_boxed_logger(Box::new(diag::Tee { inner: logger }));

    println!("Starting Voice Harmonic Engine...");

    // The Engineering Console's session log, beside the archive.
    let store_path = persist::default_store_path();
    let data_dir = store_path
        .as_ref()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    if let Some(files) = data_dir.as_deref() {
        match diagnostics::runtime::initialize(files) {
            Ok(()) => diagnostics::install_panic_hook(),
            Err(e) => log::error!("diagnostics store did not open: {e}"),
        }
    }
    diagnostics::runtime::update_renderer_mode(diagnostics::runtime::renderer_mode_name());
    diagnostics::runtime::set_shared_model_snapshot(
        serde_json::to_value(crate::atlas::data_provenance()).unwrap_or(serde_json::Value::Null),
    );
    // Desktop has no runtime permission model: the microphone is available
    // whenever a device is.
    diagnostics::runtime::update_microphone_granted(true);

    let bridges = ConcurrencyBridges::new();
    let spectrum_rx = bridges.spectrum_rx;
    let scope_rx = bridges.scope_rx;

    // The real microphone sample rate, so the DSP scales its frequencies
    // correctly.
    let input_sample_rate = {
        let host = cpal::default_host();
        let dev = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device found"))?;
        dev.default_input_config()?.config().sample_rate as f32
    };
    println!("Microphone sample rate: {input_sample_rate} Hz");

    // The runner on the live stream, in the mode last chosen (mode.txt
    // beside the archive; default live_model).
    let shutdown = Arc::new(AtomicBool::new(false));
    let mode = engine::saved_mode(data_dir.as_deref());
    let consumers = WireConsumers::new(
        bridges.profile_tx,
        bridges.ui_profile_tx,
        bridges.spectrum_tx,
        bridges.scope_tx,
        bridges.telemetry.clone(),
        1,
        1,
    );
    let mut live = LiveEngine::new(
        input_sample_rate,
        bridges.audio_rx,
        Some(Box::new(consumers)),
        bridges.telemetry.clone(),
    );
    let shell = match live.start(&mode) {
        Ok(handle) => Some(handle),
        Err(e) => {
            log::error!("{e}; live analysis unavailable");
            bridges
                .telemetry
                .set_analysis_state(AnalysisState::Unavailable);
            None
        }
    };
    let live = Arc::new(Mutex::new(live));
    engine::spawn_watcher(live.clone(), shutdown.clone(), data_dir.clone());

    let _audio_engine = AudioEngine::start(
        bridges.profile_rx,
        bridges.event_rx,
        bridges.audio_tx,
        bridges.telemetry.clone(),
    )?;

    println!("Audio engine running. Starting GUI...");
    diagnostics::runtime::update_synthesizer_active(true);

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    // Raw captures and the import folder live beside the archive:
    // `<data dir>/VoxLabs/{captures,import}/`.
    let paths = AppPaths {
        store: store_path,
        captures: data_dir.as_ref().map(|d| d.join("captures")),
        imports: data_dir.as_ref().map(|d| d.join("import")),
    };

    let result = eframe::run_native(
        "Voice Harmonic Engine",
        native_options,
        Box::new(move |cc| {
            let mut app = DashboardApp::new(
                cc,
                bridges.event_tx,
                bridges.telemetry.clone(),
                bridges.ui_profile_rx,
                spectrum_rx,
                scope_rx,
                input_sample_rate,
                paths,
            );
            if let Some(shell) = shell {
                app.attach_pipeline(shell);
            }
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {:?}", e));
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
    if let Ok(mut e) = live.lock() {
        e.retire(
            std::time::Duration::from_millis(500),
            std::time::Duration::from_millis(5),
        );
    }
    result
}
