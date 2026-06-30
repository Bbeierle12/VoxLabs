use crate::concurrency::{EngineEvent, Telemetry};
use crate::synthesis::OscillatorBank;
use crate::types::VocalProfile;
use cpal::Stream;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rtrb::Consumer;
use std::sync::Arc;
use triple_buffer::Output;

pub struct AudioEngine {
    _input_stream: Stream,
    _output_stream: Stream,
}

impl AudioEngine {
    pub fn start(
        mut profile_rx: Output<VocalProfile>,
        mut event_rx: Consumer<EngineEvent>,
        mut audio_tx: rtrb::Producer<f32>,
        telemetry: Arc<Telemetry>,
    ) -> Result<Self, anyhow::Error> {
        let host = cpal::default_host();

        let input_device = host
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No input device found"))?;
        let output_device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No output device found"))?;

        let input_config = input_device.default_input_config()?.config();
        let output_config = output_device.default_output_config()?.config();

        let sample_rate = output_config.sample_rate as f32;
        let mut osc_bank = OscillatorBank::new(sample_rate, 20.0); // 20ms parameter glide

        let channels = output_config.channels as usize;
        let in_channels = input_config.channels as usize;

        // Input callback needs its own handle; the output closure moves `telemetry`.
        let telemetry_in = telemetry.clone();

        // Output stream (Synthesis)
        let _output_stream = output_device.build_output_stream(
            output_config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Read latest profile (non-blocking)
                let profile = profile_rx.read();
                osc_bank.set_profile(profile);

                // Process events bounded
                for _ in 0..10 {
                    if let Ok(event) = event_rx.pop() {
                        match event {
                            EngineEvent::SetHarmonicCount(c) => osc_bank.set_harmonic_count(c),
                            EngineEvent::SetDeltaF(df) => osc_bank.set_delta_f(df),
                            _ => {}
                        }
                    } else {
                        break;
                    }
                }

                // Render audio block
                for frame in data.chunks_mut(channels) {
                    let (l, r) = osc_bank.process_sample();
                    if channels >= 2 {
                        frame[0] = l;
                        frame[1] = r;
                    } else if channels == 1 {
                        frame[0] = (l + r) * 0.5;
                    }
                }

                // Update telemetry
                telemetry
                    .consumed_frames
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            },
            move |err| {
                eprintln!("Output stream error: {}", err);
            },
            None, // Timeout
        )?;

        // Input stream (Analysis gathering)
        let _input_stream = input_device.build_input_stream(
            input_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Downmix and push to the analysis ring buffer. A failed push means
                // the ring is full — the analysis thread fell behind and this input
                // is dropped. Count it as an overrun (xrun) so the UI telemetry
                // reflects lost input instead of silently reading zero.
                let mut overran = false;
                for frame in data.chunks(in_channels) {
                    let mono = frame.iter().sum::<f32>() / (in_channels as f32);
                    if audio_tx.push(mono).is_err() {
                        overran = true;
                    }
                }
                if overran {
                    telemetry_in
                        .xruns
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            },
            move |err| {
                eprintln!("Input stream error: {}", err);
            },
            None,
        )?;

        _output_stream.play()?;
        _input_stream.play()?;

        Ok(Self {
            _input_stream,
            _output_stream,
        })
    }
}
