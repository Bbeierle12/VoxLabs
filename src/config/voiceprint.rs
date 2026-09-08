//! Voiceprint (classical similarity) stage configuration.

use super::stage_config;

/// Scalar voiceprint features, in order: F1, F2, F3, centroid, tilt, VTL.
pub const N_SCALAR_FEATURES: usize = 6;

stage_config! {
    /// Pitch-invariant voiceprint and its similarity score.
    pub struct VoiceprintConfig, section = "voiceprint" {
        /// Rough adult-voice population mean of each scalar feature, in
        /// feature order (Hz, Hz, Hz, Hz, dB/oct, cm). Range: physiological.
        scalar_means: [f32; N_SCALAR_FEATURES] = [500.0, 1500.0, 2600.0, 1800.0, -9.0, 16.6],
        /// Population standard deviation of each scalar feature, same order;
        /// z-scoring divisor. VTL: Story 2018's adult cohort pooled across the
        /// sexes. Range: > 0.
        scalar_stds: [f32; N_SCALAR_FEATURES] = [150.0, 350.0, 400.0, 700.0, 4.0, 1.4],
        /// Weight of the scalar (resonance/brightness) part when both parts
        /// are measurable. Range: 0.0..=1.0; timbre_weight = 1 − this.
        scalar_weight: f32 = 0.6,
        /// Weight of the timbre (harmonic-profile cosine) part. Range: 0.0..=1.0.
        timbre_weight: f32 = 0.4,
        /// A harmonic profile whose norm is at or below this is unmeasured.
        /// Range: 1e-12..=1e-6.
        profile_norm_eps: f32 = 1e-9,
    }
}
