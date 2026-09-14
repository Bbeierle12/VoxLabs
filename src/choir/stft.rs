//! Coral's STFT (`streaming-stft.ts`, `window.ts`): periodic Hann
//! (`denom = N`, librosa/scipy `fftbins=True`), one-sided magnitudes
//! scaled by `2 / Σw` so a unit sine centred on a bin peaks at 0 dB. The
//! pipeline's `Framer` does the sliding; this is the per-frame kernel the
//! `stft/coral_hann` stage wraps. Coral's `numBins` is `N / 2`; VoxLabs'
//! `Spectrum` carries `N / 2 + 1`, the extra Nyquist bin computed the same
//! way. Contract: the librosa oracle fixtures in
//! `apps/coral/src/audio/__fixtures__/oracle/` within Coral's
//! cross-implementation tier (rtol 1e-4, atol 1e-7, `docs/TOLERANCES.md`).

use std::f32::consts::TAU;
use std::sync::Arc;

use rustfft::{Fft, FftPlanner, num_complex::Complex};

use crate::config::consts::{HANN_A0, TWO};

/// Periodic Hann window of `size` samples.
pub fn periodic_hann(size: usize) -> Vec<f32> {
    (0..size)
        .map(|i| HANN_A0 * (1.0 - (TAU * i as f32 / size as f32).cos()))
        .collect()
}

pub struct CoralStft {
    fft: Arc<dyn Fft<f32>>,
    window: Vec<f32>,
    scratch: Vec<Complex<f32>>,
    inv_scale: f32,
}

impl CoralStft {
    /// `fft_size` must be a power of two (Coral's contract).
    pub fn new(fft_size: usize) -> Result<Self, String> {
        if fft_size == 0 || !fft_size.is_power_of_two() {
            return Err(format!(
                "CoralStft: fftSize must be a power of 2, got {fft_size}"
            ));
        }
        let window = periodic_hann(fft_size);
        let window_sum: f32 = window.iter().sum();
        let mut planner = FftPlanner::<f32>::new();
        Ok(Self {
            fft: planner.plan_fft_forward(fft_size),
            window,
            scratch: vec![Complex::new(0.0, 0.0); fft_size],
            inv_scale: TWO / window_sum,
        })
    }

    pub fn fft_size(&self) -> usize {
        self.window.len()
    }

    /// Coral's `numBins` (`fftSize / 2`).
    pub fn num_bins(&self) -> usize {
        self.window.len() / 2
    }

    /// Magnitudes of one frame into `out` (`out.len()` bins, at most
    /// `fft_size / 2 + 1`).
    pub fn magnitudes(&mut self, frame: &[f32], out: &mut [f32]) -> Result<(), String> {
        let n = self.window.len();
        if frame.len() != n {
            return Err(format!(
                "CoralStft: frame of {} samples, expected {n}",
                frame.len()
            ));
        }
        if out.len() > n / 2 + 1 {
            return Err(format!(
                "CoralStft: {} bins requested of {}",
                out.len(),
                n / 2 + 1
            ));
        }
        for (i, (s, w)) in frame.iter().zip(&self.window).enumerate() {
            self.scratch[i] = Complex::new(s * w, 0.0);
        }
        self.fft.process(&mut self.scratch);
        for (o, c) in out.iter_mut().zip(&self.scratch) {
            *o = (c.re * c.re + c.im * c.im).sqrt() * self.inv_scale;
        }
        Ok(())
    }
}

/// Coral's u8 wire quantization of one magnitude bin over
/// `[floor_db, ceil_db]` (`dsp-core.ts::quantize`).
pub fn quantize_db(magnitude: f32, floor_db: f32, ceil_db: f32, eps: f32) -> u8 {
    let db = crate::config::consts::DB_PER_DECADE_AMPLITUDE * (magnitude + eps).log10();
    let t = ((db - floor_db) / (ceil_db - floor_db)).clamp(0.0, 1.0);
    (t * 255.0 + 0.5) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Oracle {
        name: String,
        sample_rate: f32,
        fft_size: usize,
        hop_size: usize,
        num_bins: usize,
        frames: usize,
        librosa_version: String,
        samples: Vec<f32>,
        expected_magnitudes: Vec<Vec<f32>>,
    }

    const FIXTURES: &[&str] = &[
        include_str!("../../apps/coral/src/audio/__fixtures__/oracle/sine-bin100-44k1-2048.json"),
        include_str!("../../apps/coral/src/audio/__fixtures__/oracle/sine-440p7-44k1-2048.json"),
        include_str!("../../apps/coral/src/audio/__fixtures__/oracle/twotone-noise-44k1-2048.json"),
        include_str!("../../apps/coral/src/audio/__fixtures__/oracle/chirp-100-4k-44k1-2048.json"),
        include_str!("../../apps/coral/src/audio/__fixtures__/oracle/sine-bin100-48k-2048.json"),
        include_str!("../../apps/coral/src/audio/__fixtures__/oracle/sine-440p7-44k1-4096.json"),
    ];
    const RTOL: f32 = 1e-4;
    const ATOL: f32 = 1e-7;

    /// Coral's `oracle-fixtures.test.ts`: librosa magnitudes on bit-identical
    /// float32 input, bin 0 excluded (Coral's documented DC wart), within the
    /// cross-implementation tier. Framed as the pipeline frames: the first
    /// frame after `fftSize` samples, then one per hop.
    #[test]
    fn librosa_oracle_fixtures_within_the_cross_implementation_tier() {
        for text in FIXTURES {
            let fx: Oracle = serde_json::from_str(text).unwrap();
            let mut stft = CoralStft::new(fx.fft_size).unwrap();
            let mut got = vec![0.0f32; fx.num_bins];
            let mut max_rel = 0.0f32;
            let mut frames = 0;
            let mut start = 0;
            while start + fx.fft_size <= fx.samples.len() {
                stft.magnitudes(&fx.samples[start..start + fx.fft_size], &mut got)
                    .unwrap();
                let exp = &fx.expected_magnitudes[frames];
                for b in 1..fx.num_bins {
                    let err = (got[b] - exp[b]).abs();
                    assert!(
                        err <= ATOL + RTOL * exp[b].abs(),
                        "{} frame {frames} bin {b}: got {}, librosa {} {}",
                        fx.name,
                        got[b],
                        exp[b],
                        fx.librosa_version
                    );
                    if exp[b].abs() > ATOL / RTOL {
                        max_rel = max_rel.max(err / exp[b].abs());
                    }
                }
                frames += 1;
                start += fx.hop_size;
            }
            assert_eq!(frames, fx.frames, "{}", fx.name);
            assert!(max_rel < RTOL, "{}: max rel {max_rel:e}", fx.name);
            let _ = fx.sample_rate;
        }
    }

    /// Calibration: a unit sine on bin 100 at 48 kHz peaks at 0 dB ± 0.3.
    #[test]
    fn a_bin_centred_unit_sine_peaks_at_zero_db() {
        let n = 2048;
        let sr = 48_000.0f32;
        let f = 100.0 * sr / n as f32;
        let frame: Vec<f32> = (0..n).map(|i| (TAU * f * i as f32 / sr).sin()).collect();
        let mut stft = CoralStft::new(n).unwrap();
        let mut out = vec![0.0f32; n / 2 + 1];
        stft.magnitudes(&frame, &mut out).unwrap();
        let db = 20.0 * out[100].log10();
        assert!(db.abs() < 0.3, "{db} dB");
        assert_eq!(
            quantize_db(1.0, -140.0, 12.0, 1e-10),
            ((140.0 / 152.0) * 255.0 + 0.5) as u8
        );
        assert!(CoralStft::new(1000).is_err());
    }
}
