// Native-only modules: real audio I/O (cpal) and GPU compute (wgpu). The web
// build (P2) replaces these with a Web Audio + CPU-DSP layer, so they are gated
// out of the wasm target entirely.
#[cfg(not(target_arch = "wasm32"))]
mod analysis;
#[cfg(not(target_arch = "wasm32"))]
mod audio;

mod concurrency;
mod math;
mod synthesis;
mod types;
mod ui;

use concurrency::ConcurrencyBridges;
use ui::DashboardApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    use analysis::AnalysisEngine;
    use audio::AudioEngine;
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

// Web entry point. P1 renders the existing dashboard in the browser with the DSP
// stubbed: the concurrency bridges are built so the UI has its event/telemetry/
// profile handles, but nothing writes the profile yet, so the dashboard sits in
// "SEARCHING" until P2 wires the Web Audio capture/synthesis layer.
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    console_error_panic_hook::set_once();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no global window")
            .document()
            .expect("no document on window");
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("missing element #the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#the_canvas_id is not a <canvas>");

        let bridges = ConcurrencyBridges::new();
        let event_tx = bridges.event_tx;
        let telemetry = bridges.telemetry.clone();
        let ui_profile_rx = bridges.ui_profile_rx;

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(move |cc| {
                    Ok(Box::new(DashboardApp::new(
                        cc,
                        event_tx,
                        telemetry,
                        ui_profile_rx,
                    )))
                }),
            )
            .await
            .expect("failed to start eframe web runner");
    });
}
