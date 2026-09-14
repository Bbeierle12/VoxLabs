//! Cross-target tolerance bands (Plan v3 D5): numeric comparisons between
//! runs, targets and backends use these bands, never bit-equality. Kernel
//! parity on one machine is expected to land far inside them; the bands
//! exist for arm64-vs-x86_64, wasm and float-order differences. Set from
//! data when Phase 5a measures them; these are the plan's starting values.

use super::stage_config;

stage_config! {
    /// D5 tolerance bands for tap-by-tap comparison (`pipeline::compare`).
    pub struct ToleranceConfig, section = "tolerance" {
        /// f0 agreement, Hz. Range: 0.01..=5.0.
        f0_hz: f32 = 0.1,
        /// Estimator confidence agreement (unit scale). Range: 1e-4..=0.1.
        confidence: f32 = 1e-3,
        /// Formant frequency agreement, Hz (kernel parity: tenths of a Hz).
        /// Range: 0.01..=50.0.
        formant_hz: f32 = 0.1,
        /// Formant bandwidth agreement, Hz. Range: 0.01..=50.0.
        bandwidth_hz: f32 = 0.5,
        /// Harmonic amplitude agreement, relative to the frame's strongest
        /// harmonic. Range: 1e-5..=0.1.
        harmonic_rel: f32 = 1e-3,
        /// Any dB-valued metric (HNR, H1–H2, CPP, shimmer, SNR, spectrum
        /// bins), dB. Range: 0.01..=3.0.
        metric_db: f32 = 0.1,
        /// Jitter agreement, percent. Range: 1e-3..=1.0.
        jitter_pct: f32 = 0.01,
        /// Spectral centroid agreement, Hz. Range: 0.1..=100.0.
        centroid_hz: f32 = 1.0,
        /// Vibrato rate agreement, Hz. Range: 0.01..=1.0.
        vibrato_rate_hz: f32 = 0.05,
        /// Vibrato extent and steadiness agreement, cents. Range: 0.1..=20.0.
        cents: f32 = 1.0,
        /// Story mode coefficient agreement. Range: 1e-4..=0.5.
        tract_q: f32 = 1e-3,
        /// Atlas mode coefficient agreement, standard deviations (the
        /// posterior inverse's `modes`). Range: 1e-4..=0.5.
        mode_sd: f32 = 1e-3,
        /// Mesh vertex agreement, mm per coordinate. Range: 1e-3..=1.0.
        geometry_mm: f32 = 0.01,
        /// Vocal-tract length agreement, cm. Range: 0.01..=1.0.
        vtl_cm: f32 = 0.05,
        /// Area-function agreement, cm² per section. Range: 1e-4..=0.5.
        area_cm2: f32 = 1e-3,
        /// Diameter agreement, cm per section. Range: 1e-4..=0.5.
        diameter_cm: f32 = 1e-3,
        /// Linear spectrum magnitude agreement, relative to the frame's
        /// peak bin. Range: 1e-5..=0.1.
        spectrum_rel: f32 = 1e-3,
    }
}
