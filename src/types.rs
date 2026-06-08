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
    pub active_partials: usize,
    /// Reserved: per-harmonic amplitudes for a future richer synthesis envelope.
    #[allow(dead_code)]
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
            active_partials: 0,
            partial_amplitudes: [0.0; MAX_PARTIALS],
            valid: false,
        }
    }
}
