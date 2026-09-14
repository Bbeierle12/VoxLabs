//! `rustfft`: one Hann-windowed FFT over the current frame per hop, as
//! `spectrogram::Spectrogram` computes it — the same window, scale and
//! floors — so the waterfall reads the `Spectrum` tap instead of its own
//! STFT. At hop 1024 over a 2048 frame that is 50 % overlap (D14: the
//! spectrogram moves to the pipeline hop; `overlap` is the frame/hop ratio
//! the mode file sets, not a stage parameter).

use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::f32::consts::TAU;
use std::sync::Arc;

use crate::config::consts::{DB_PER_DECADE_AMPLITUDE, HANN_A0};
use crate::config::{PipelineParams, SpectrogramConfig};

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{AudioFrame, Spectrum};
use super::require_default;

pub struct StftStage {
    fft: Arc<dyn Fft<f32>>,
    window: Vec<f32>,
    scratch: Vec<Complex<f32>>,
    magnitude_floor: f32,
}

impl Stage for StftStage {
    type In<'a> = &'a AudioFrame;
    type Out = Spectrum;
    const NAME: &'static str = "stft";
    const BACKEND: &'static str = "rustfft";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, fmt: &StreamFormat) -> Result<Self, StageError> {
        require_default(
            "spectrogram",
            Self::NAME,
            &params.spectrogram,
            &SpectrogramConfig::DEFAULT,
        )?;
        if fmt.frame_samples != params.spectrogram.fft_size {
            return Err(StageError::Init(format!(
                "frame of {} samples is not the spectrogram fft_size {}",
                fmt.frame_samples, params.spectrogram.fft_size
            )));
        }
        let n = fmt.frame_samples;
        let mut planner = FftPlanner::<f32>::new();
        Ok(Self {
            fft: planner.plan_fft_forward(n),
            window: hann_window(n),
            scratch: vec![Complex::new(0.0, 0.0); n],
            magnitude_floor: params.spectrogram.magnitude_floor,
        })
    }

    fn process(&mut self, frame: &AudioFrame, out: &mut Spectrum) -> Result<(), StageError> {
        let n = self.window.len();
        if frame.samples.len() != n {
            return Err(StageError::Process(format!(
                "frame of {} samples; the stage was built for {n}",
                frame.samples.len()
            )));
        }
        for (i, (s, w)) in frame.samples.iter().zip(&self.window).enumerate() {
            self.scratch[i] = Complex::new(s * w, 0.0);
        }
        self.fft.process(&mut self.scratch);
        let scale = 1.0 / n as f32;
        for (bin, c) in self.scratch.iter().take(out.magnitude.len()).enumerate() {
            let mag = c.norm() * scale;
            out.magnitude[bin] = mag;
            out.magnitudes_db[bin] =
                DB_PER_DECADE_AMPLITUDE * mag.max(self.magnitude_floor).log10();
        }
        out.bin_hz = frame.sample_rate / n as f32;
        out.fft_size = n;
        out.frame_index = frame.frame_index;
        Ok(())
    }
}

fn hann_window(n: usize) -> Vec<f32> {
    if n <= 1 {
        return vec![1.0; n];
    }
    let m = (n - 1) as f32;
    (0..n)
        .map(|i| HANN_A0 * (1.0 - (TAU * i as f32 / m).cos()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spectrogram::{FFT_SIZE, HOP, N_BINS, Spectrogram};

    const SR: f32 = 48_000.0;

    fn fmt() -> StreamFormat {
        StreamFormat {
            sample_rate_hz: SR,
            frame_samples: FFT_SIZE,
            hop: FFT_SIZE / 2,
        }
    }

    /// Contract: over the same stream, the stage's spectrum at hop k equals
    /// the scrolling spectrogram's frame ending at the same sample.
    #[test]
    fn stage_matches_spectrogram_frames() {
        let hop = fmt().hop;
        let n = FFT_SIZE + hop * 6;
        let sig: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / SR;
                0.4 * (TAU * 220.0 * t).sin() + 0.2 * (TAU * 1375.0 * t).sin()
            })
            .collect();
        let mut stage = StftStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let mut out = Spectrum::preallocated(FFT_SIZE, SR, crate::spectrogram::DB_FLOOR);
        // The reference advances HOP (512) samples per frame; the stage
        // advances the pipeline hop (1024), so every second reference
        // frame is the stage's.
        assert_eq!(hop % HOP, 0);
        let stride = hop / HOP;
        let mut reference = Spectrogram::new();
        let mut compared = 0;
        let mut fed = 0;
        let mut k = 0;
        while fed + FFT_SIZE <= n {
            let end = fed + FFT_SIZE;
            // Feed the reference up to `end` and take its latest frame.
            reference.process_block(&sig[reference_fed(k, hop)..end]);
            let frame = AudioFrame {
                samples: sig[fed..end].to_vec(),
                sample_rate: SR,
                frame_index: k as u64,
            };
            stage.process(&frame, &mut out).unwrap();
            if k % stride == 0 || stride == 1 {
                for (b, (a, r)) in out
                    .magnitudes_db
                    .iter()
                    .zip(reference.magnitudes_db())
                    .enumerate()
                {
                    assert!((a - r).abs() < 1e-3, "hop {k} bin {b}: {a} vs {r}");
                }
                compared += 1;
            }
            assert_eq!(out.magnitude.len(), N_BINS);
            fed += hop;
            k += 1;
        }
        assert!(compared >= 3);

        fn reference_fed(k: usize, hop: usize) -> usize {
            if k == 0 { 0 } else { FFT_SIZE + (k - 1) * hop }
        }
    }

    #[test]
    fn a_pure_tone_peaks_in_its_bin() {
        let mut stage = StftStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let mut out = Spectrum::preallocated(FFT_SIZE, SR, crate::spectrogram::DB_FLOOR);
        let f = 1000.0;
        let frame = AudioFrame {
            samples: (0..FFT_SIZE)
                .map(|i| (TAU * f * i as f32 / SR).sin())
                .collect(),
            sample_rate: SR,
            frame_index: 3,
        };
        stage.process(&frame, &mut out).unwrap();
        let peak = out
            .magnitude
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap()
            .0;
        assert_eq!(peak, (f / out.bin_hz).round() as usize);
        assert_eq!(out.frame_index, 3);
    }
}
