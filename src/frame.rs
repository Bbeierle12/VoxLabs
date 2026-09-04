//! The per-frame CPU analysis pipeline, shared by every consumer that is
//! not the desktop GPU engine: the Android capture loop and the `voxlab`
//! study harness. One implementation, so a number measured on a file in
//! the lab and a number measured on the phone come from the same code —
//! which is the whole point of studying the dataset through the app's own
//! DSP rather than through some other tool.
//!
//! What this owns: YIN pitch with the confidence/range gate, the SNR and
//! calibrated-hum voicing gates (with their noise-floor learning rule),
//! LPC formants on the decimated frame with measurement-time f0 latched,
//! harmonic amplitudes, and the per-frame voice-quality metrics. What it
//! does NOT own: room-calibration bookkeeping (telemetry-driven, stays in
//! the Android loop, which seeds the floor and interferer through the two
//! setters here), spectrogram/scope feeds, and telemetry heartbeats.

use crate::math;
use crate::types::{Formant, MAX_PARTIALS, VocalProfile, VoiceMetrics};

/// Analysis frame length in samples, on every CPU consumer. 2048 at 48 kHz
/// is 42.7 ms.
pub const ANALYSIS_FRAME: usize = 2048;

/// Spectral envelope held before the first voiced frame and through
/// unvoiced gaps, so synthesis never collapses. Mirrors
/// `analysis::DEFAULT_FORMANTS` on the desktop GPU path.
pub const DEFAULT_FORMANTS: [Formant; 3] = [
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

/// YIN confidence below which a frame is unvoiced, and the f0 range the
/// engine accepts at all (the study harness reports frames outside it as
/// unvoiced, exactly as the phone does).
pub const YIN_MIN_CONFIDENCE: f32 = 0.4;
pub const F0_MIN_HZ: f32 = 50.0;
pub const F0_MAX_HZ: f32 = 1000.0;

/// Everything one frame produced, plus the raw facts the caller needs for
/// its own bookkeeping (calibration wants the pre-gate periodicity).
#[derive(Clone, Debug)]
pub struct FrameResult {
    pub profile: VocalProfile,
    /// Frame RMS (linear), pre-gate.
    pub rms: f32,
    /// f0 YIN reported before the SNR/hum gates, if it reported one.
    pub yin_f0: Option<f32>,
}

/// Stateful analyzer: holds the latched formants, the f0-contour tracker,
/// the learned noise floor, and the calibrated interferer.
pub struct FrameAnalyzer {
    sample_rate: f32,
    last_formants: [Formant; 3],
    /// f0 of the frame `last_formants` was measured on (0.0 = never), so the
    /// formant-reliability gate judges by measurement conditions.
    last_formants_f0: f32,
    contour: crate::metrics::F0Contour,
    noise_floor: math::NoiseFloor,
    room_interferer: Option<math::Interferer>,
}

impl FrameAnalyzer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            last_formants: DEFAULT_FORMANTS,
            last_formants_f0: 0.0,
            contour: crate::metrics::F0Contour::new(sample_rate / ANALYSIS_FRAME as f32),
            noise_floor: math::NoiseFloor::new(),
            room_interferer: None,
        }
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Seed the ambient floor from a calibration pass (see
    /// `math::NoiseFloor::seed`).
    pub fn seed_floor(&mut self, ambient_rms: f32) {
        self.noise_floor.seed(ambient_rms);
    }

    /// Install (or clear) the calibrated steady interferer the hum gate
    /// matches against.
    pub fn set_interferer(&mut self, interferer: Option<math::Interferer>) {
        self.room_interferer = interferer;
    }

    /// Analyze one frame of exactly [`ANALYSIS_FRAME`] samples (a shorter
    /// final frame is analyzed as-is; callers normally never send one).
    pub fn analyze(&mut self, frame: &[f32]) -> FrameResult {
        let sample_rate = self.sample_rate;

        // --- Pitch (f0): pure-CPU YIN ---
        let pitch = math::yin_pitch(frame, sample_rate);
        let (f0_raw, yin_voiced) = match pitch {
            Some(p)
                if p.confidence > YIN_MIN_CONFIDENCE && (F0_MIN_HZ..=F0_MAX_HZ).contains(&p.f0) =>
            {
                (p.f0, true)
            }
            _ => (0.0, false),
        };

        // SNR voicing gate: periodicity alone is not voice — a YIN-voiced
        // frame must also clear the learned ambient floor by
        // VOICED_MIN_SNR_DB or it is demoted to unvoiced here, protecting
        // every downstream consumer at one point. The floor learns from
        // unvoiced frames only, so sustained singing cannot raise it.
        let rms = math::frame_rms(frame);
        let snr_db = self.noise_floor.snr_db(rms);
        let snr_ok = snr_db.is_none_or(|s| s >= math::VOICED_MIN_SNR_DB);
        // Calibrated-interferer gate: a steady room tone at this pitch and
        // level is not the singer (a louder singer on the same pitch passes).
        let hum = self
            .room_interferer
            .is_some_and(|i| math::interferer_match(f0_raw, rms, &i));
        let voiced = yin_voiced && snr_ok && !hum;
        let voiced_but_noisy = yin_voiced && (!snr_ok || hum);
        if !yin_voiced {
            self.noise_floor.push_unvoiced(rms);
        }
        let f0 = if voiced { f0_raw } else { 0.0 };

        // --- Formants via LPC on a decimated signal (voiced frames only) ---
        if voiced {
            let m = ((sample_rate / 11_025.0).round() as usize).max(1);
            let fs_dec = sample_rate / m as f32;
            let order = (2 + (fs_dec / 1000.0) as usize).clamp(8, 20);

            let decimated = math::decimate(frame, m);
            let lpc = math::lpc_coefficients(&decimated, order, 0.97);
            let measured = math::formants_from_lpc(&lpc, fs_dec);
            if measured[0].frequency > 0.0 {
                self.last_formants = measured;
                self.last_formants_f0 = f0;
            }
        }

        // Harmonic series at k·f0.
        let partial_amplitudes = if voiced {
            math::harmonic_amplitudes(frame, sample_rate, f0)
        } else {
            [0.0; MAX_PARTIALS]
        };

        // Voice-quality metrics.
        self.contour.push(f0, voiced);
        let (vibrato, steadiness_cents) = self.contour.analyze();
        let perturbation = if voiced {
            math::cycle_perturbation(frame, sample_rate, f0)
        } else {
            None
        };
        let metrics = VoiceMetrics {
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
            jitter_pct: perturbation.map(|p| p.jitter_pct),
            shimmer_db: perturbation.map(|p| p.shimmer_db),
            cpp_db: if voiced {
                math::cpp_db(frame, sample_rate)
            } else {
                None
            },
            centroid_hz: if voiced {
                math::spectral_centroid(frame, sample_rate)
            } else {
                None
            },
            snr_db,
            voiced_but_noisy,
        };

        FrameResult {
            profile: VocalProfile {
                f0,
                formants: self.last_formants,
                formants_f0: self.last_formants_f0,
                partial_amplitudes,
                metrics,
                valid: voiced,
            },
            rms,
            yin_f0: yin_voiced.then_some(f0_raw),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A harmonic-rich synthetic vowel at a known f0.
    fn synth(f0: f32, sr: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| {
                let t = i as f32 / sr;
                (1i32..=10)
                    .map(|k| {
                        (0.6f32).powi(k - 1)
                            * (2.0 * std::f32::consts::PI * f0 * k as f32 * t).sin()
                    })
                    .sum::<f32>()
                    * 0.2
            })
            .collect()
    }

    #[test]
    fn voiced_frame_yields_f0_and_harmonics() {
        let sr = 48_000.0;
        let mut a = FrameAnalyzer::new(sr);
        let sig = synth(140.0, sr, ANALYSIS_FRAME * 4);
        let mut last = None;
        for frame in sig.chunks_exact(ANALYSIS_FRAME) {
            last = Some(a.analyze(frame));
        }
        let r = last.unwrap();
        assert!(r.profile.valid, "synthetic vowel should be voiced");
        assert!((r.profile.f0 - 140.0).abs() < 3.0, "f0 {}", r.profile.f0);
        assert!(r.profile.partial_amplitudes[0] > 0.0);
        assert_eq!(r.yin_f0.map(|f| f.round()), Some(140.0));
    }

    #[test]
    fn silence_is_unvoiced_and_learns_the_floor() {
        let sr = 48_000.0;
        let mut a = FrameAnalyzer::new(sr);
        let quiet = vec![0.0f32; ANALYSIS_FRAME];
        let r = a.analyze(&quiet);
        assert!(!r.profile.valid);
        assert!(r.yin_f0.is_none());
        assert_eq!(r.profile.partial_amplitudes, [0.0; MAX_PARTIALS]);
    }
}
