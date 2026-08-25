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

// GPU-compute analysis (wgpu YIN): desktop only. Android uses the CPU DSP path
// in the `android` module, so wgpu is never compiled for Android.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
mod analysis;

// Android entry point: `android_main`, wired to android-activity's
// NativeActivity glue. Gated so desktop and web are untouched.
#[cfg(target_os = "android")]
mod android;

mod concurrency;
// Capture-stack capability probe (UNPROCESSED support, microphone inventory,
// channel independence). Report/analysis types compile everywhere so the UI
// and tests are cross-target; the JNI half is Android-only. The dead_code
// allow is desktop-only: there the UI matches on these types but nothing
// constructs them (only the Android probe does), which is exactly the shape
// dead-code analysis flags.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
mod device_probe;
mod math;
// TV-path spatial calibration + live path-consistency scoring. The math is
// cross-target and unit-tested; the continuous two-channel capture thread
// is Android-only (same dead-code shape as device_probe on desktop).
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
mod spatial;
// Disk persistence for the enrolled reference + session archive. Compiled on
// all targets (ui uses it); each entry point supplies the store path (or None
// on web, which has no disk).
mod persist;
// f0-contour metrics (vibrato/steadiness): used by the desktop and Android
// analysis loops; the web target has no analysis thread yet.
#[cfg(not(target_arch = "wasm32"))]
mod metrics;
// Scrolling-spectrogram STFT. Compiled on all targets (its consts size the
// UI's waterfall buffers); the engine itself is driven only by the desktop
// and Android analysis loops.
mod spectrogram;
mod synthesis;
// Story two-mode vocal tract model: runtime (tract) + published basis data
// (tract_data). Compiled on all targets — pure math, no threads or I/O.
mod tract;
mod tract_data;
mod types;
mod ui;

pub use concurrency::ConcurrencyBridges;
pub use ui::DashboardApp;

/// Consecutive `process_frame` failures tolerated before the desktop analysis
/// loop gives up and reports itself stopped. A single failure can be a
/// transient GPU submit hiccup; three in a row means the device is gone.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
const MAX_CONSECUTIVE_FRAME_ERRORS: u32 = 3;

/// Idle sleep between ring-buffer drains on the analysis thread.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
const ANALYSIS_POLL_MS: u64 = 5;

/// Desktop native entry point. Spawns the GPU-accelerated analysis thread,
/// starts the cpal audio engine, and runs the egui dashboard. This is the
/// former `fn main` body verbatim (now returning to `main.rs`), so desktop
/// behaviour is unchanged.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn run() -> anyhow::Result<()> {
    use crate::analysis::AnalysisEngine;
    use crate::audio::AudioEngine;
    use crate::concurrency::AnalysisState;
    use cpal::traits::{DeviceTrait, HostTrait};
    use std::panic::AssertUnwindSafe;
    use std::thread;

    env_logger::init();

    println!("Starting Voice Harmonic Engine...");

    let bridges = ConcurrencyBridges::new();
    let profile_tx = bridges.profile_tx;

    let mut audio_rx = bridges.audio_rx;
    let ui_profile_tx = bridges.ui_profile_tx;
    let spectrum_tx = bridges.spectrum_tx;
    let spectrum_rx = bridges.spectrum_rx;
    let scope_tx = bridges.scope_tx;
    let scope_rx = bridges.scope_rx;

    // Determine the real microphone sample rate up front so the analysis DSP
    // scales its frequencies correctly. Previously process_frame was fed a
    // hardcoded 44100.0, which mis-scaled f0/formants on any other device rate.
    let input_sample_rate = {
        let host = cpal::default_host();
        let dev = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device found"))?;
        dev.default_input_config()?.config().sample_rate as f32
    };
    println!("Microphone sample rate: {input_sample_rate} Hz");

    // Start background analysis thread. Nothing on this thread may panic the
    // process or die silently: initialization failure, a panic inside the loop
    // and repeated frame errors each land in `Telemetry` so the UI can say
    // which one happened instead of showing a frozen readout forever.
    let analysis_telemetry = bridges.telemetry.clone();
    thread::spawn(move || {
        let telemetry = analysis_telemetry;
        let mut engine = match pollster::block_on(AnalysisEngine::new(
            profile_tx,
            ui_profile_tx,
            spectrum_tx,
            scope_tx,
            telemetry.clone(),
        )) {
            Ok(engine) => engine,
            Err(e) => {
                // Almost always "no usable GPU adapter". Per the phase-F
                // decision (gate G3) we report it rather than falling back to
                // a desktop CPU analysis path.
                log::error!("analysis engine failed to initialize: {e:?}");
                telemetry.set_analysis_state(AnalysisState::Unavailable);
                return;
            }
        };
        telemetry.set_analysis_state(AnalysisState::Running);

        let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| {
            analysis_loop(&mut engine, &mut audio_rx, input_sample_rate, &telemetry);
        }));
        match outcome {
            Ok(()) => log::error!("analysis loop exited; live analysis has stopped"),
            Err(_) => log::error!("analysis loop panicked; live analysis has stopped"),
        }
        telemetry.set_analysis_state(AnalysisState::Stopped);
    });

    let _audio_engine = AudioEngine::start(
        bridges.profile_rx,
        bridges.event_rx,
        bridges.audio_tx,
        bridges.telemetry.clone(),
    )?;

    println!("Audio engine running. Starting GUI...");

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    let store_path = persist::default_store_path();

    eframe::run_native(
        "Voice Harmonic Engine",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(DashboardApp::new(
                cc,
                bridges.event_tx,
                bridges.telemetry.clone(),
                bridges.ui_profile_rx,
                spectrum_rx,
                scope_rx,
                input_sample_rate,
                store_path,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {:?}", e))?;

    Ok(())
}

/// Desktop analysis loop body, split out of [`run`] so the spawning thread can
/// wrap it in `catch_unwind` and report its death.
///
/// Returns only when analysis can no longer proceed: the caller then marks the
/// engine `Stopped`. The accumulator is persistent — each tick drains
/// *everything* available from the ring buffer, processes as many whole frames
/// as it has, and carries the leftover samples into the next tick.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
fn analysis_loop(
    engine: &mut analysis::AnalysisEngine,
    audio_rx: &mut rtrb::Consumer<f32>,
    input_sample_rate: f32,
    telemetry: &concurrency::Telemetry,
) {
    let mut accumulator: Vec<f32> = Vec::with_capacity(analysis::ANALYSIS_FRAME * 4);
    let mut consecutive_errors = 0u32;

    loop {
        while let Ok(sample) = audio_rx.pop() {
            accumulator.push(sample);
        }

        while accumulator.len() >= analysis::ANALYSIS_FRAME {
            match engine.process_frame(&accumulator[..analysis::ANALYSIS_FRAME], input_sample_rate)
            {
                Ok(()) => {
                    consecutive_errors = 0;
                    telemetry.note_analysis_frame();
                }
                Err(e) => {
                    consecutive_errors += 1;
                    log::error!(
                        "process_frame failed ({consecutive_errors}/{MAX_CONSECUTIVE_FRAME_ERRORS}): {e:?}"
                    );
                    if consecutive_errors >= MAX_CONSECUTIVE_FRAME_ERRORS {
                        return;
                    }
                }
            }
            accumulator.drain(..analysis::ANALYSIS_FRAME);
        }

        std::thread::sleep(std::time::Duration::from_millis(ANALYSIS_POLL_MS));
    }
}
