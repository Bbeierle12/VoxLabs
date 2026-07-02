#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Formant {
    pub frequency: f32,
    pub bandwidth: f32,
}

pub const MAX_PARTIALS: usize = 32;

#[derive(Debug, Clone, Copy)]
pub struct VocalProfile {
    pub f0: f32,
    pub formants: [Formant; 3], // F1, F2, F3
    /// Measured amplitude of each harmonic k·f0 (linear peak, Goertzel at the
    /// harmonic frequencies). Zeroed when the frame is unvoiced.
    pub partial_amplitudes: [f32; MAX_PARTIALS],
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
            valid: false,
        }
    }
}
