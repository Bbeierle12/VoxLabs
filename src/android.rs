//! Android entry point.
//!
//! android-activity's NativeActivity glue (pulled in via winit's
//! `android-native-activity` backend) loads the cdylib in the Activity's
//! `onCreate` and then calls the `#[unsafe(no_mangle)] fn android_main` symbol
//! defined here on a dedicated thread. We wire up the *same* [`DashboardApp`]
//! egui app the desktop and web targets use.
//!
//! Two deliberate de-risking choices for this first on-device build:
//!   1. **CPU DSP only.** Analysis runs `math::yin_pitch` + LPC formants on the
//!      CPU (see [`cpu_analysis_loop`]); it never constructs the wgpu `GpuYin`
//!      compute path, so a missing/limited Vulkan driver can't break launch.
//!   2. **glow (GLES/EGL) renderer**, not wgpu/Vulkan — forced via
//!      `NativeOptions.renderer`.
//!
//! Microphone capture needs the `RECORD_AUDIO` runtime permission. It is
//! declared in the manifest, but Android only grants it once the user (or
//! `adb shell pm grant`) approves it. Until then cpal's input stream fails to
//! open; we treat that as non-fatal so the UI still launches (it just shows
//! "SEARCHING"). See docs/android-build.md.

use crate::concurrency::ConcurrencyBridges;
use crate::types::{Formant, VocalProfile};
use crate::ui::DashboardApp;
use rtrb::Consumer;
use std::thread;
use triple_buffer::Input;
use winit::platform::android::activity::AndroidApp;

/// Samples per analysis frame. Mirrors `analysis::ANALYSIS_FRAME` (2048 @ 44.1
/// kHz ≈ 46 ms) — kept local so the GPU analysis module stays desktop-only.
const ANALYSIS_FRAME: usize = 2048;

/// Fallback microphone rate when the input device can't be queried yet (e.g.
/// `RECORD_AUDIO` not granted at launch). 48 kHz is the near-universal Android
/// capture rate.
const FALLBACK_SAMPLE_RATE: f32 = 48_000.0;

/// Spectral envelope held before the first voiced frame and through unvoiced
/// gaps, so synthesis never collapses. Mirrors `analysis::DEFAULT_FORMANTS`.
const DEFAULT_FORMANTS: [Formant; 3] = [
    Formant {
        frequency: 500.0,
        bandwidth: 80.0,
    },
    Formant {
        frequency: 1500.0,
        bandwidth: 120.0,
    },
    Formant {
        frequency: 2500.0,
        bandwidth: 160.0,
    },
];

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    log::info!("android_main: starting Voice Harmonic Engine");

    let bridges = ConcurrencyBridges::new();

    // Analysis thread inputs.
    let profile_tx = bridges.profile_tx;
    let ui_profile_tx = bridges.ui_profile_tx;
    let audio_rx = bridges.audio_rx;

    // Query the mic rate up front so DSP frequency scaling is correct; fall back
    // if unavailable (permission not yet granted / no input device).
    let input_sample_rate = query_input_sample_rate().unwrap_or(FALLBACK_SAMPLE_RATE);
    log::info!("android input sample rate: {input_sample_rate} Hz");

    // CPU YIN + LPC on the non-real-time analysis thread.
    thread::spawn(move || {
        cpu_analysis_loop(profile_tx, ui_profile_tx, audio_rx, input_sample_rate);
    });

    // cpal AAudio engine. Non-fatal on failure: without RECORD_AUDIO the input
    // stream can't open, but we still want the UI up so the user can grant it.
    let _audio_engine = match crate::audio::AudioEngine::start(
        bridges.profile_rx,
        bridges.event_rx,
        bridges.audio_tx,
        bridges.telemetry.clone(),
    ) {
        Ok(engine) => Some(engine),
        Err(e) => {
            log::error!(
                "audio engine failed to start (is RECORD_AUDIO granted?): {e:?}; UI will run without audio"
            );
            None
        }
    };

    let event_tx = bridges.event_tx;
    let telemetry = bridges.telemetry.clone();
    let ui_profile_rx = bridges.ui_profile_rx;

    let native_options = eframe::NativeOptions {
        android_app: Some(app),
        // glow/EGL (GLES) rather than wgpu/Vulkan for the first build.
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Voice Harmonic Engine",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(DashboardApp::new(
                cc,
                event_tx,
                telemetry,
                ui_profile_rx,
            )))
        }),
    ) {
        log::error!("eframe exited with error: {e:?}");
    }
}

/// Best-effort read of the default input device's sample rate via cpal.
fn query_input_sample_rate() -> Option<f32> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    let dev = host.default_input_device()?;
    let config = dev.default_input_config().ok()?;
    Some(config.config().sample_rate as f32)
}

/// CPU-only analysis loop: drains the input ring buffer, and for each whole
/// frame estimates f0 (YIN) and — on voiced frames — the formants (decimate →
/// LPC → root-solve), publishing the profile to synthesis + UI. This mirrors
/// `AnalysisEngine::process_frame`'s CPU path but never touches wgpu.
fn cpu_analysis_loop(
    mut profile_tx: Input<VocalProfile>,
    mut ui_profile_tx: Input<VocalProfile>,
    mut audio_rx: Consumer<f32>,
    sample_rate: f32,
) {
    use crate::math;

    let mut last_formants = DEFAULT_FORMANTS;
    let mut accumulator: Vec<f32> = Vec::with_capacity(ANALYSIS_FRAME * 4);
    // f0-contour tracker for vibrato/steadiness, mirroring the desktop path.
    let mut contour = crate::metrics::F0Contour::new(sample_rate / ANALYSIS_FRAME as f32);

    loop {
        while let Ok(sample) = audio_rx.pop() {
            accumulator.push(sample);
        }

        while accumulator.len() >= ANALYSIS_FRAME {
            let frame = &accumulator[..ANALYSIS_FRAME];

            // --- Pitch (f0): pure-CPU YIN ---
            let pitch = math::yin_pitch(frame, sample_rate);
            let (f0, voiced) = match pitch {
                Some(p) if p.confidence > 0.4 && (50.0..=1000.0).contains(&p.f0) => (p.f0, true),
                _ => (0.0, false),
            };

            // --- Formants via LPC on a decimated signal (voiced frames only) ---
            if voiced {
                let m = ((sample_rate / 11_025.0).round() as usize).max(1);
                let fs_dec = sample_rate / m as f32;
                let order = (2 + (fs_dec / 1000.0) as usize).clamp(8, 20);

                let decimated = math::decimate(frame, m);
                let lpc = math::lpc_coefficients(&decimated, order, 0.97);
                let measured = math::formants_from_lpc(&lpc, fs_dec);
                if measured[0].frequency > 0.0 {
                    last_formants = measured;
                }
            }

            // Harmonic series at k·f0, mirroring the desktop analysis path.
            let partial_amplitudes = if voiced {
                math::harmonic_amplitudes(frame, sample_rate, f0)
            } else {
                [0.0; crate::types::MAX_PARTIALS]
            };

            // Voice-quality metrics, mirroring the desktop analysis path.
            contour.push(f0, voiced);
            let (vibrato, steadiness_cents) = contour.analyze();
            let metrics = crate::types::VoiceMetrics {
                hnr_db: if voiced {
                    math::hnr_db(frame, sample_rate, f0)
                } else {
                    None
                },
                h1_h2_db: if voiced {
                    math::h1_h2_db(&partial_amplitudes)
                } else {
                    None
                },
                vibrato,
                steadiness_cents,
            };

            let profile = VocalProfile {
                f0,
                formants: last_formants,
                partial_amplitudes,
                metrics,
                valid: voiced,
            };
            profile_tx.write(profile);
            ui_profile_tx.write(profile);

            accumulator.drain(..ANALYSIS_FRAME);
        }

        thread::sleep(std::time::Duration::from_millis(5));
    }
}
