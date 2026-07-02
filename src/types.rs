#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Formant {
    pub frequency: f32,
    pub bandwidth: f32,
}

/// A detected vibrato: oscillation rate and extent (± cents around the mean).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vibrato {
    pub rate_hz: f32,
    pub extent_cents: f32,
}

/// Per-frame voice-quality metrics. All `None` until the analysis has enough
/// voiced signal to say something honest.
#[derive(Debug, Clone, Copy, Default)]
pub struct VoiceMetrics {
    /// Harmonics-to-noise ratio (dB); breathiness/clarity.
    pub hnr_db: Option<f32>,
    /// H1–H2 level difference (dB); phonation type (breathy vs pressed).
    pub h1_h2_db: Option<f32>,
    /// Detected vibrato; `None` when no stable 3–9 Hz oscillation dominates.
    pub vibrato: Option<Vibrato>,
    /// RMS cents deviation of the sustain after drift + vibrato removal.
    pub steadiness_cents: Option<f32>,
    /// Jitter (local, %): cycle-to-cycle period perturbation. Needs ≥ 5
    /// cycles per frame, so `None` below ~112 Hz.
    pub jitter_pct: Option<f32>,
    /// Shimmer (local, dB): cycle-to-cycle amplitude perturbation.
    pub shimmer_db: Option<f32>,
    /// Cepstral peak prominence (dB): periodicity strength / breathiness.
    pub cpp_db: Option<f32>,
}

pub const MAX_PARTIALS: usize = 32;

#[derive(Debug, Clone, Copy)]
pub struct VocalProfile {
    pub f0: f32,
    pub formants: [Formant; 3], // F1, F2, F3
    /// Measured amplitude of each harmonic k·f0 (linear peak, Goertzel at the
    /// harmonic frequencies). Zeroed when the frame is unvoiced.
    pub partial_amplitudes: [f32; MAX_PARTIALS],
    /// Voice-quality metrics (HNR, H1–H2, vibrato, steadiness).
    pub metrics: VoiceMetrics,
    pub valid: bool,
}

impl Default for VocalProfile {
    fn default() -> Self {
        Self {
            f0: 0.0,
            formants: [Formant {
                frequency: 0.0,
                bandwidth: 0.0,
            }; 3],
            partial_amplitudes: [0.0; MAX_PARTIALS],
            metrics: VoiceMetrics::default(),
            valid: false,
        }
    }
}
