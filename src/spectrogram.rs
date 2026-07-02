//! Hop-based Short-Time Fourier Transform for the scrolling spectrogram view.
//!
//! Ported from the Resonator project's `amp-analysis::Spectrogram` (same
//! author, MIT/Apache-2.0), adapted to use this crate's existing `rustfft`
//! dependency instead of `realfft` — the math is identical, `realfft` is only
//! a real-input optimization we don't need here. The engine buffers input,
//! emits one Hann-windowed magnitude frame every `HOP` samples, and keeps the
//! most recent frame available for the UI to sample each repaint. It runs on
//! the non-real-time analysis thread, never in an audio callback.

use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::f32::consts::PI;
use std::sync::Arc;

/// FFT window length. 2048 matches `analysis::ANALYSIS_FRAME`, so the analysis
/// loop can feed the engine the same frame slices it feeds YIN.
pub const FFT_SIZE: usize = 2048;
/// Samples advanced between frames. 512 → four magnitude frames per 2048-sample
/// analysis frame, so the waterfall stays fresh regardless of the YIN cadence.
pub const HOP: usize = 512;
/// One-sided magnitude bin count for a real signal: `FFT_SIZE / 2 + 1`.
pub const N_BINS: usize = FFT_SIZE / 2 + 1;
/// Magnitude floor (dB) reported for silent / empty bins.
pub const DB_FLOOR: f32 = -120.0;

pub struct Spectrogram {
    fft: Arc<dyn Fft<f32>>,
    window: Vec<f32>,
    input_buffer: Vec<f32>,
    in_pos: usize,
    scratch: Vec<Complex<f32>>,
    magnitudes_db: Vec<f32>,
    hop: usize,
}

impl Default for Spectrogram {
    fn default() -> Self {
        Self::new()
    }
}

impl Spectrogram {
    pub fn new() -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        Self {
            fft,
            window: hann_window(FFT_SIZE),
            input_buffer: vec![0.0; FFT_SIZE],
            in_pos: 0,
            scratch: vec![Complex::new(0.0, 0.0); FFT_SIZE],
            magnitudes_db: vec![DB_FLOOR; N_BINS],
            hop: HOP,
        }
    }

    /// Feed a block of samples (any length). Returns the number of new FFT
    /// frames computed; the most recent is available via [`magnitudes_db`].
    pub fn process_block(&mut self, samples: &[f32]) -> usize {
        let mut new_frames = 0;
        for &s in samples {
            self.input_buffer[self.in_pos] = s;
            self.in_pos += 1;
            if self.in_pos >= self.input_buffer.len() {
                self.compute_frame();
                new_frames += 1;
                let len = self.input_buffer.len();
                self.input_buffer.copy_within(self.hop..len, 0);
                self.in_pos = len - self.hop;
            }
        }
        new_frames
    }

    fn compute_frame(&mut self) {
        let n = self.input_buffer.len();
        for i in 0..n {
            self.scratch[i] = Complex::new(self.input_buffer[i] * self.window[i], 0.0);
        }
        self.fft.process(&mut self.scratch);
        let scale = 1.0 / n as f32;
        for (bin, c) in self.scratch.iter().take(N_BINS).enumerate() {
            let mag = c.norm() * scale;
            self.magnitudes_db[bin] = 20.0 * mag.max(1e-12).log10();
        }
    }

    pub fn magnitudes_db(&self) -> &[f32] {
        &self.magnitudes_db
    }
}

fn hann_window(n: usize) -> Vec<f32> {
    if n <= 1 {
        return vec![1.0; n];
    }
    let m = (n - 1) as f32;
    (0..n)
        .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / m).cos()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f32 = 48_000.0;

    fn sine(freq: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| 0.5 * (2.0 * PI * freq * i as f32 / SR).sin())
            .collect()
    }

    #[test]
    fn bin_count_is_half_fft_plus_one() {
        let s = Spectrogram::new();
        assert_eq!(s.magnitudes_db().len(), N_BINS);
        assert_eq!(N_BINS, FFT_SIZE / 2 + 1);
    }

    #[test]
    fn peak_bin_matches_input_frequency() {
        let mut s = Spectrogram::new();
        let target = 1_000.0f32;
        s.process_block(&sine(target, 8_192));

        let peak_bin = s
            .magnitudes_db()
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        let bin_hz = peak_bin as f32 * SR / FFT_SIZE as f32;
        let bin_width = SR / FFT_SIZE as f32;
        assert!(
            (bin_hz - target).abs() < bin_width,
            "expected peak near {target} Hz, got bin {peak_bin} = {bin_hz:.1} Hz"
        );
    }

    #[test]
    fn silence_yields_floor() {
        let mut s = Spectrogram::new();
        s.process_block(&vec![0.0f32; 4_096]);
        let peak = s
            .magnitudes_db()
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            peak < -100.0,
            "silence should be near floor, got {peak:.1} dB"
        );
    }
}
