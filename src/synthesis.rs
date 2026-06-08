use crate::types::{Formant, MAX_PARTIALS, VocalProfile};
use std::f32::consts::PI;

const TAU: f32 = 2.0 * PI;

pub struct OscillatorBank {
    sample_rate: f32,

    // Engine State
    harmonic_count: usize,
    delta_f: f32,

    // Targets (From Profile)
    target_f0: f32,
    target_formants: [Formant; 3],

    // Smoothed values
    current_f0: f32,
    current_formants: [Formant; 3],

    // Phases
    phase_l: [f32; MAX_PARTIALS],
    phase_r: [f32; MAX_PARTIALS],

    // Smoothing coeff
    alpha: f32,
}

impl OscillatorBank {
    pub fn new(sample_rate: f32, tau_glide_ms: f32) -> Self {
        let tau_sec = tau_glide_ms / 1000.0;
        let alpha = 1.0 - (-1.0 / (tau_sec * sample_rate)).exp();

        let default_formants = [
            Formant {
                frequency: 500.0,
                bandwidth: 50.0,
            },
            Formant {
                frequency: 1500.0,
                bandwidth: 100.0,
            },
            Formant {
                frequency: 2500.0,
                bandwidth: 150.0,
            },
        ];

        Self {
            sample_rate,
            harmonic_count: 5,
            delta_f: 6.0,
            target_f0: 150.0,
            target_formants: default_formants,
            current_f0: 150.0,
            current_formants: default_formants,
            phase_l: [0.0; MAX_PARTIALS],
            phase_r: [0.0; MAX_PARTIALS],
            alpha,
        }
    }

    pub fn set_harmonic_count(&mut self, count: usize) {
        self.harmonic_count = count.clamp(1, MAX_PARTIALS);
    }

    pub fn set_delta_f(&mut self, delta_f: f32) {
        self.delta_f = delta_f;
    }

    pub fn set_profile(&mut self, profile: &VocalProfile) {
        if profile.valid && profile.f0 > 20.0 {
            self.target_f0 = profile.f0;
            self.target_formants = profile.formants;
        }
    }

    // A simple resonance curve to approximate a formant filter
    #[inline(always)]
    fn evaluate_formants(freq: f32, formants: &[Formant; 3]) -> f32 {
        let mut gain = 0.0;
        for f in formants {
            if f.frequency > 0.0 && f.bandwidth > 0.0 {
                // Simple bandpass magnitude approximation
                let q = f.frequency / f.bandwidth;
                let omega = freq / f.frequency;
                let denom = ((1.0 - omega * omega).powi(2) + (omega / q).powi(2)).sqrt();
                gain += 1.0 / (1.0 + denom * 10.0); // scaled to avoid blowing up
            }
        }
        // Base glottal rolloff (-12dB/octave roughly)
        let rolloff = 1.0 / (1.0 + freq / 100.0);
        (gain + 0.1) * rolloff
    }

    #[inline(always)]
    pub fn process_sample(&mut self) -> (f32, f32) {
        // 1. Smooth target parameters
        self.current_f0 += self.alpha * (self.target_f0 - self.current_f0);

        for i in 0..3 {
            self.current_formants[i].frequency += self.alpha
                * (self.target_formants[i].frequency - self.current_formants[i].frequency);
            self.current_formants[i].bandwidth += self.alpha
                * (self.target_formants[i].bandwidth - self.current_formants[i].bandwidth);
        }

        let mut out_l = 0.0;
        let mut out_r = 0.0;
        let mut total_amp = 0.0;

        // 2. Generate additive stack
        for n in 1..=self.harmonic_count {
            let idx = n - 1;
            let n_f32 = n as f32;

            // Frequencies
            let f_l = n_f32 * self.current_f0;
            let f_r = n_f32 * self.current_f0 + self.delta_f;

            // Amplitude based on spectral envelope at f_l
            let amp = Self::evaluate_formants(f_l, &self.current_formants);

            if amp < 0.0001 {
                continue;
            }

            total_amp += amp;

            // Phase increments (normalized 0..1)
            let dp_l = f_l / self.sample_rate;
            let dp_r = f_r / self.sample_rate;

            // Accumulate phase
            self.phase_l[idx] += dp_l;
            if self.phase_l[idx] >= 1.0 {
                self.phase_l[idx] -= 1.0;
            }

            self.phase_r[idx] += dp_r;
            if self.phase_r[idx] >= 1.0 {
                self.phase_r[idx] -= 1.0;
            }

            // Synthesize
            out_l += amp * (self.phase_l[idx] * TAU).sin();
            out_r += amp * (self.phase_r[idx] * TAU).sin();
        }

        if total_amp > 0.0 {
            out_l /= total_amp;
            out_r /= total_amp;
        }

        (out_l * 0.5, out_r * 0.5) // -6dB headroom
    }
}
