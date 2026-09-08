//! Additive synthesis (`synthesis::OscillatorBank`) configuration.

use super::stage_config;
use crate::types::{Formant, N_FORMANTS};

stage_config! {
    /// Additive stereo oscillator bank driven by the measured profile.
    pub struct SynthesisConfig, section = "synthesis" {
        /// Parameter glide time constant, ms (one-pole smoothing of f0 and
        /// formants toward their targets). Range: 5.0..=200.0.
        glide_ms: f32 = 20.0,
        /// Initial spectral-envelope target — formant frequencies, Hz.
        /// Range: rising, 200.0..=4000.0.
        default_frequencies_hz: [f32; N_FORMANTS] = [500.0, 1500.0, 2500.0],
        /// Bandwidths, Hz, of the initial envelope. Range: 20.0..=300.0.
        default_bandwidths_hz: [f32; N_FORMANTS] = [50.0, 100.0, 150.0],
        /// Initial f0 target, Hz. Range: 50.0..=500.0.
        default_f0_hz: f32 = 150.0,
        /// Harmonics rendered until the UI says otherwise. Range: 1..=MAX_PARTIALS.
        default_harmonic_count: usize = 5,
        /// Right-channel detune added to every partial, Hz (binaural beat).
        /// Range: 0.0..=20.0.
        default_delta_f_hz: f32 = 6.0,
        /// Measured f0 at or below this is refused as a target. Range: 10.0..=50.0.
        min_target_f0_hz: f32 = 20.0,
        /// Scales the resonance denominator in the bandpass approximation so
        /// the peak gain stays bounded. Range: 1.0..=50.0.
        resonance_scale: f32 = 10.0,
        /// Corner of the glottal roll-off `1 / (1 + f / this)`, Hz (≈ −12
        /// dB/oct above it). Range: 50.0..=300.0.
        rolloff_corner_hz: f32 = 100.0,
        /// Gain floor added under the formant sum so partials between
        /// formants are not silent. Range: 0.0..=0.5.
        base_gain: f32 = 0.1,
        /// Partials whose envelope gain is below this are skipped.
        /// Range: 1e-6..=1e-2.
        amp_cutoff: f32 = 0.0001,
        /// Output scale after normalization (0.5 = −6 dB headroom).
        /// Range: 0.1..=1.0.
        headroom: f32 = 0.5,
    }
}

impl SynthesisConfig {
    /// The initial envelope as `Formant`s.
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
