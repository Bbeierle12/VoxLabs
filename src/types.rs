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

/// A pitch-invariant classical voiceprint: vocal-tract resonances (formants),
/// spectral brightness/slope, and timbre shape. f0 is deliberately excluded so
/// the print identifies the *voice* regardless of the note sung. Built from a
/// capture's averaged features; compared via `math::voiceprint_similarity`.
///
/// This is a real, deterministic voice-*similarity* representation — good for
/// tracking a singer's own consistency — not a forensic biometric identity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voiceprint {
    /// F1, F2, F3 in Hz (0.0 = unresolved).
    pub formants: [f32; 3],
    /// Spectral centroid (Hz).
    pub centroid_hz: f32,
    /// Spectral tilt (dB/octave).
    pub tilt_db_oct: f32,
    /// Mean relative harmonic profile, H1..H16 (normalized to the strongest).
    pub profile: [f32; 16],
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
    /// Spectral centroid (Hz): full-spectrum power-weighted mean frequency,
    /// driving the Dark..Brilliant brightness classification.
    pub centroid_hz: Option<f32>,
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
