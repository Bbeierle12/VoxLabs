//! Voice Harmonic Engine — crate root.
//!
//! This library owns every module and the shared native (desktop) runner. The
//! three entry points live at the edges:
//!   * desktop  — `main.rs`'s `main()` calls [`run`];
//!   * web/wasm — `main.rs`'s wasm `main()` drives eframe's `WebRunner`;
//!   * Android  — the `android` module's `android_main` (compiled into the
//!     cdylib the APK loads).
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
mod math;
mod synthesis;
mod types;
mod ui;

pub use concurrency::ConcurrencyBridges;
pub use ui::DashboardApp;

/// Desktop native entry point. Spawns the GPU-accelerated analysis thread,
/// starts the cpal audio engine, and runs the egui dashboard. This is the
/// former `fn main` body verbatim (now returning to `main.rs`), so desktop
/// behaviour is unchanged.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn run() -> anyhow::Result<()> {
    use crate::analysis::{self, AnalysisEngine};
    use crate::audio::AudioEngine;
    use cpal::traits::{DeviceTrait, HostTrait};
    use std::thread;

    env_logger::init();

    println!("Starting Voice Harmonic Engine...");

    let bridges = ConcurrencyBridges::new();
    let profile_tx = bridges.profile_tx;

    let mut audio_rx = bridges.audio_rx;
    let ui_profile_tx = bridges.ui_profile_tx;

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

    // Start background analysis thread
    thread::spawn(move || {
        let mut engine = pollster::block_on(AnalysisEngine::new(profile_tx, ui_profile_tx))
            .expect("Failed to init AnalysisEngine");

        // Persistent accumulator. Each tick we drain *everything* available from
        // the ring buffer, then process as many whole frames as we have, carrying
        // the leftover samples into the next tick. The previous loop reset its
        // index every iteration and silently discarded any partial (<1024) read.
        let mut accumulator: Vec<f32> = Vec::with_capacity(analysis::ANALYSIS_FRAME * 4);
        loop {
            while let Ok(sample) = audio_rx.pop() {
                accumulator.push(sample);
            }

            while accumulator.len() >= analysis::ANALYSIS_FRAME {
                engine
                    .process_frame(&accumulator[..analysis::ANALYSIS_FRAME], input_sample_rate)
                    .expect("process_frame failed");
                accumulator.drain(..analysis::ANALYSIS_FRAME);
            }

            std::thread::sleep(std::time::Duration::from_millis(5));
        }
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

    eframe::run_native(
        "Voice Harmonic Engine",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(DashboardApp::new(
                cc,
                bridges.event_tx,
                bridges.telemetry.clone(),
                bridges.ui_profile_rx,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {:?}", e))?;

    Ok(())
}
