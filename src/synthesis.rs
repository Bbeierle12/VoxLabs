use crate::config::SynthesisConfig;
use crate::config::consts::{MILLIS_PER_SECOND, SQUARED};
use crate::types::{Formant, MAX_PARTIALS, N_FORMANTS, VocalProfile};
use std::f32::consts::TAU;

/// Stage configuration (see `config::SynthesisConfig` and `pipeline.toml`).
const SYNTH: SynthesisConfig = SynthesisConfig::DEFAULT;

pub struct OscillatorBank {
    sample_rate: f32,

    // Engine State
    harmonic_count: usize,
    delta_f: f32,

    // Targets (From Profile)
    target_f0: f32,
    target_formants: [Formant; N_FORMANTS],

    // Smoothed values
    current_f0: f32,
    current_formants: [Formant; N_FORMANTS],

    // Phases
    phase_l: [f32; MAX_PARTIALS],
    phase_r: [f32; MAX_PARTIALS],

    // Smoothing coeff
    alpha: f32,
}

impl OscillatorBank {
    pub fn new(sample_rate: f32, tau_glide_ms: f32) -> Self {
        let tau_sec = tau_glide_ms / MILLIS_PER_SECOND;
        let alpha = 1.0 - (-1.0 / (tau_sec * sample_rate)).exp();

        let default_formants = SYNTH.default_formants();

        Self {
            sample_rate,
            harmonic_count: SYNTH.default_harmonic_count,
            delta_f: SYNTH.default_delta_f_hz,
            target_f0: SYNTH.default_f0_hz,
            target_formants: default_formants,
            current_f0: SYNTH.default_f0_hz,
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

    /// Adopts a measured profile as the new glide target.
    ///
    /// Non-finite values are refused rather than stored: `current_f0` and the
    /// current formants converge toward their targets on *every* sample, so a
    /// single NaN or infinity in a target poisons the oscillator state for the
    /// rest of the stream — there is no frame boundary to recover at. The
    /// upstream DSP already gates its outputs (see `math`'s finiteness
    /// guards), but this is the real-time audio callback, so it does not
    /// assume that.
    pub fn set_profile(&mut self, profile: &VocalProfile) {
        if profile.valid && profile.f0.is_finite() && profile.f0 > SYNTH.min_target_f0_hz {
            self.target_f0 = profile.f0;
            for (target, measured) in self.target_formants.iter_mut().zip(&profile.formants) {
                if measured.frequency.is_finite() && measured.bandwidth.is_finite() {
                    *target = *measured;
                }
            }
        }
    }

    // A simple resonance curve to approximate a formant filter
    #[inline(always)]
    fn evaluate_formants(freq: f32, formants: &[Formant; N_FORMANTS]) -> f32 {
        let mut gain = 0.0;
        for f in formants {
            if f.frequency > 0.0 && f.bandwidth > 0.0 {
                // Simple bandpass magnitude approximation
                let q = f.frequency / f.bandwidth;
                let omega = freq / f.frequency;
                let denom =
                    ((1.0 - omega * omega).powi(SQUARED) + (omega / q).powi(SQUARED)).sqrt();
                gain += 1.0 / (1.0 + denom * SYNTH.resonance_scale); // scaled to avoid blowing up
            }
        }
        // Base glottal rolloff (-12dB/octave roughly)
        let rolloff = 1.0 / (1.0 + freq / SYNTH.rolloff_corner_hz);
        (gain + SYNTH.base_gain) * rolloff
    }

    #[inline(always)]
    pub fn process_sample(&mut self) -> (f32, f32) {
        // 1. Smooth target parameters
        self.current_f0 += self.alpha * (self.target_f0 - self.current_f0);

        for i in 0..N_FORMANTS {
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

            if amp < SYNTH.amp_cutoff {
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

        (out_l * SYNTH.headroom, out_r * SYNTH.headroom) // -6dB headroom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::VoiceMetrics;

    const SR: f32 = 48_000.0;
    const GLIDE_MS: f32 = 20.0;

    /// `process_sample` divides by the summed harmonic amplitudes and then
    /// applies -6 dB headroom, so no sample may leave ±0.5. The tolerance
    /// absorbs float rounding in that division, nothing more.
    const PEAK_CEILING: f32 = 0.5;
    const PEAK_TOLERANCE: f32 = 1e-5;

    /// Enough samples for the 20 ms glide to fully converge on the target
    /// (~960 samples at 48 kHz), so a target that poisons the state has time
    /// to show up in the output.
    const SETTLE_SAMPLES: usize = 4800;

    fn formants(spec: [(f32, f32); 3]) -> [Formant; 3] {
        spec.map(|(frequency, bandwidth)| Formant {
            frequency,
            bandwidth,
        })
    }

    fn profile(f0: f32, formants: [Formant; 3]) -> VocalProfile {
        VocalProfile {
            f0,
            formants,
            formants_f0: f0,
            partial_amplitudes: [0.0; MAX_PARTIALS],
            metrics: VoiceMetrics::default(),
            valid: true,
        }
    }

    /// Runs the bank and asserts every sample is finite and inside the
    /// headroom bound. This is the whole safety net: the oscillator feeds a
    /// real-time output callback, where a NaN is an audible fault and an
    /// out-of-range sample is a clip.
    fn drive(bank: &mut OscillatorBank, samples: usize, case: &str) {
        for i in 0..samples {
            let (l, r) = bank.process_sample();
            assert!(
                l.is_finite() && r.is_finite(),
                "{case}: sample {i} = ({l}, {r}) is not finite"
            );
            assert!(
                l.abs() <= PEAK_CEILING + PEAK_TOLERANCE
                    && r.abs() <= PEAK_CEILING + PEAK_TOLERANCE,
                "{case}: sample {i} = ({l}, {r}) exceeds ±{PEAK_CEILING}"
            );
        }
    }

    #[test]
    fn default_bank_is_finite_and_bounded() {
        let mut bank = OscillatorBank::new(SR, GLIDE_MS);
        drive(&mut bank, SETTLE_SAMPLES, "default");
    }

    #[test]
    fn all_zero_profile_is_finite_and_bounded() {
        let mut bank = OscillatorBank::new(SR, GLIDE_MS);
        // The unvoiced/pre-signal profile: nothing measured yet. The bank must
        // keep its last good target rather than glide to silence-by-zero.
        bank.set_profile(&VocalProfile::default());
        drive(&mut bank, SETTLE_SAMPLES, "default profile");

        // Same shape but flagged valid, with every field zeroed.
        bank.set_profile(&profile(0.0, formants([(0.0, 0.0); 3])));
        drive(&mut bank, SETTLE_SAMPLES, "all-zero profile");
    }

    #[test]
    fn zero_formants_with_real_f0_is_finite_and_bounded() {
        let mut bank = OscillatorBank::new(SR, GLIDE_MS);
        // Voiced frame whose LPC root-solve resolved nothing: f0 is real, the
        // spectral envelope is empty. `evaluate_formants` must fall through to
        // the glottal rolloff instead of dividing by a zero-width resonance.
        bank.set_profile(&profile(220.0, formants([(0.0, 0.0); 3])));
        drive(&mut bank, SETTLE_SAMPLES, "zero formants");
    }

    #[test]
    fn max_harmonic_count_is_finite_and_bounded() {
        let mut bank = OscillatorBank::new(SR, GLIDE_MS);
        // Above MAX_PARTIALS: must clamp, not index out of the phase arrays.
        bank.set_harmonic_count(usize::MAX);
        // Low f0 so all MAX_PARTIALS harmonics land in band and contribute.
        bank.set_profile(&profile(
            80.0,
            formants([(500.0, 80.0), (1500.0, 120.0), (2500.0, 160.0)]),
        ));
        drive(&mut bank, SETTLE_SAMPLES, "max harmonics");
    }

    #[test]
    fn non_finite_profiles_never_reach_the_oscillator() {
        let good = profile(
            220.0,
            formants([(500.0, 80.0), (1500.0, 120.0), (2500.0, 160.0)]),
        );
        let poison = [
            ("nan f0", profile(f32::NAN, good.formants)),
            ("inf f0", profile(f32::INFINITY, good.formants)),
            ("neg inf f0", profile(f32::NEG_INFINITY, good.formants)),
            (
                "nan formants",
                profile(220.0, formants([(f32::NAN, f32::NAN); 3])),
            ),
            (
                "inf formants",
                profile(220.0, formants([(f32::INFINITY, f32::INFINITY); 3])),
            ),
            (
                "mixed",
                profile(
                    220.0,
                    formants([(500.0, 80.0), (f32::NAN, 120.0), (2500.0, f32::INFINITY)]),
                ),
            ),
        ];

        for (case, bad) in poison {
            let mut bank = OscillatorBank::new(SR, GLIDE_MS);
            bank.set_profile(&good);
            drive(&mut bank, SETTLE_SAMPLES, case);
            bank.set_profile(&bad);
            drive(&mut bank, SETTLE_SAMPLES, case);
        }
    }
}
