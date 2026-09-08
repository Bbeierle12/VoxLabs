//! LPC front-end and formant-extraction stage configuration.

use super::stage_config;
use crate::types::{Formant, N_FORMANTS};

stage_config! {
    /// Linear-prediction front-end: decimation to the formant band,
    /// pre-emphasis, Hamming window, autocorrelation, Levinson-Durbin.
    pub struct LpcConfig, section = "lpc" {
        /// Target sample rate after decimation, Hz; the decimation factor
        /// is `round(sample_rate / this)`. ~11 kHz keeps 0–5.5 kHz, where
        /// the formants are. Range: 8000.0..=16000.0.
        decimation_target_hz: f32 = 11_025.0,
        /// LPC order rule: `order = order_base + fs_dec / order_hz_per_pole`,
        /// clamped to [order_min, order_max]. This is the base term.
        /// Range: 0..=4.
        order_base: usize = 2,
        /// Hertz of decimated bandwidth per prediction pole ("2 + fs/1000").
        /// Range: 500.0..=2000.0.
        order_hz_per_pole: f32 = 1000.0,
        /// Lower clamp on the LPC order. Range: 4..=order_max.
        order_min: usize = 8,
        /// Upper clamp on the LPC order. Range: order_min..=32.
        order_max: usize = 20,
        /// Pre-emphasis coefficient `y[n] = x[n] − α·x[n−1]`, flattening the
        /// −12 dB/oct glottal tilt. Range: 0.9..=0.99.
        preemphasis: f32 = 0.97,
        /// Anti-alias lowpass cutoff before decimation, as a fraction of the
        /// *new* Nyquist (0.45 ≈ 90 % of it). Range: 0.3..=0.5.
        decimation_cutoff_of_nyquist: f32 = 0.45,
        /// Windowed-sinc lowpass length in taps (odd). Range: 15..=63.
        decimation_fir_taps: usize = 31,
        /// A sinc argument closer to zero than this takes the limit value.
        /// Range: 1e-9..=1e-3.
        sinc_center_eps: f32 = 1e-6,
        /// FIR DC gain below this is left unnormalized (degenerate cutoff).
        /// Range: 1e-12..=1e-6.
        fir_gain_eps: f32 = 1e-9,
        /// Zero-lag autocorrelation at or below this is a silent frame and
        /// yields the all-pass polynomial. Range: 1e-12..=1e-6.
        silence_energy: f32 = 1e-9,
        /// Levinson-Durbin stops when the prediction error falls below this
        /// (degenerate frame). Range: 1e-12..=1e-6.
        levinson_error_floor: f32 = 1e-9,
    }
}

stage_config! {
    /// Formant extraction from the LPC polynomial's roots, plus the
    /// reliability grades applied to the result.
    pub struct FormantConfig, section = "formants" {
        /// Aberth root-finder iteration cap. Range: 20..=200.
        aberth_max_iterations: u32 = 50,
        /// Aberth convergence tolerance. Range: 1e-12..=1e-6.
        aberth_epsilon: f64 = 1e-9,
        /// Lowest pole frequency accepted as a formant, Hz. Range: 50.0..=150.0.
        band_lo_hz: f64 = 90.0,
        /// Highest pole frequency accepted as a formant, Hz; the spectrogram
        /// view's ceiling matches it. Range: 3500.0..=decimated Nyquist.
        band_hi_hz: f64 = 5000.0,
        /// Poles wider than this (Hz) are not formants. Range: 200.0..=1000.0.
        max_bandwidth_hz: f64 = 500.0,
        /// Fewest polynomial coefficients that can hold a complex pole pair.
        /// Range: exactly 3.
        min_coefficients: usize = 3,
        /// Spectral envelope held before the first voiced frame and through
        /// unvoiced gaps — frequencies, Hz (F1, F2, F3). Range: rising,
        /// inside [band_lo_hz, band_hi_hz].
        default_frequencies_hz: [f32; N_FORMANTS] = [500.0, 1500.0, 2500.0],
        /// Bandwidths, Hz, of the held default envelope. Range: 40.0..=300.0.
        default_bandwidths_hz: [f32; N_FORMANTS] = [80.0, 120.0, 160.0],
        /// Above this measured f0, formants are excluded from identity
        /// features (Chen, Whalen & Shadle 2019). Range: 150.0..=250.0.
        identity_f0_max_hz: f32 = 200.0,
        /// Above this measured f0, formants are neither stored nor shown
        /// (Monsen & Engebretson 1983). Range: identity_f0_max_hz..=500.0.
        display_f0_max_hz: f32 = 350.0,
        /// Half-width of the harmonic-proximity suspect band as a fraction of
        /// the harmonic (Boë et al. 2023: ±15 %). Range: 0.05..=0.25.
        harmonic_suspect_frac: f32 = 0.15,
        /// F2 is checked against this multiple of f0 (the second harmonic).
        /// Range: exactly 2.
        f2_suspect_harmonic: f32 = 2.0,
    }
}

impl FormantConfig {
    /// The held default envelope as `Formant`s.
    pub const fn default_formants(&self) -> [Formant; N_FORMANTS] {
        let mut out = [Formant {
            frequency: 0.0,
            bandwidth: 0.0,
        }; N_FORMANTS];
        let mut i = 0;
        while i < N_FORMANTS {
            out[i] = Formant {
                frequency: self.default_frequencies_hz[i],
                bandwidth: self.default_bandwidths_hz[i],
            };
            i += 1;
        }
        out
    }
}
