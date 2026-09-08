//! Voice-part (Fach) measurement configuration (`fach.rs`).

use super::stage_config;

/// ISO third-octave bands reported by the LTAS, 100 Hz – 8 kHz.
pub const N_THIRD_OCTAVE_BANDS: usize = 20;

stage_config! {
    /// The "Fach, Measured" quantities: FHE bands, singer's-formant cluster,
    /// LTAS bands, tessitura, turnover, dominant harmonic, register events.
    pub struct FachConfig, section = "fach" {
        /// Singer's-formant analysis band for male voices, Hz [lo, hi]
        /// (Müller 2022). Range: lo < hi within 1500.0..=5000.0.
        fhe_band_male_hz: [f32; 2] = [2000.0, 3600.0],
        /// Singer's-formant analysis band for female voices, Hz [lo, hi].
        /// Range: lo < hi within 1500.0..=6000.0.
        fhe_band_female_hz: [f32; 2] = [2300.0, 4500.0],
        /// Cluster search window, low edge, Hz. Range: 1800.0..=2600.0.
        cluster_lo_hz: f32 = 2200.0,
        /// Cluster search window, high edge, Hz. Range: cluster_lo_hz..=4000.0.
        cluster_hi_hz: f32 = 3400.0,
        /// The valley below the cluster is sought from here up, Hz.
        /// Range: 800.0..=cluster_lo_hz.
        cluster_valley_from_hz: f32 = 1500.0,
        /// The −3 dB width walk stops at this frequency, Hz.
        /// Range: cluster_hi_hz..=8000.0.
        cluster_width_ceil_hz: f32 = 5000.0,
        /// Width is measured this many dB down from the cluster peak.
        /// Range: 1.0..=6.0.
        cluster_width_db: f32 = 3.0,
        /// A cluster peak at or below this level (dB re linear power 1) is
        /// silence, no cluster. Range: -200.0..=-100.0.
        cluster_silent_peak_db: f32 = -170.0,
        /// ISO third-octave band centres, Hz. Range: the standard series.
        third_octave_centers_hz: [f32; N_THIRD_OCTAVE_BANDS] = [
            100.0, 125.0, 160.0, 200.0, 250.0, 315.0, 400.0, 500.0, 630.0, 800.0, 1000.0, 1250.0,
            1600.0, 2000.0, 2500.0, 3150.0, 4000.0, 5000.0, 6300.0, 8000.0,
        ],
        /// Turnover hysteresis: A2/A1 must sit at or beyond ±this (dB) to
        /// count as below/above, so a wobble around 0 dB fires once.
        /// Range: 1.0..=6.0.
        turnover_hysteresis_db: f32 = 3.0,
        /// Register event: minimum f0 jump between consecutive voiced
        /// frames, semitones. Range: 1.0..=4.0.
        register_min_jump_st: f32 = 2.0,
        /// Register event: |ΔH1–H2| that counts as a source change, dB.
        /// Range: 2.0..=8.0.
        register_h1h2_delta_db: f32 = 4.0,
        /// Register event: CPP drop that counts as a source change, dB.
        /// Range: 1.0..=4.0.
        register_cpp_drop_db: f32 = 2.0,
        /// Register event: jitter that counts as a source change, %.
        /// Range: 0.5..=3.0.
        register_jitter_pct: f32 = 1.5,
        /// Frames shorter than this (samples) yield no power spectrum.
        /// Range: 128..=1024.
        min_frame_samples: usize = 256,
        /// Frame energy (sum of squares) below this is silence. Range: 1e-15..=1e-9.
        silence_energy: f32 = 1e-12,
        /// Band power (linear, f64 sum) at or below this is empty. Range: 1e-24..=1e-12.
        band_energy_eps: f64 = 1e-18,
        /// Smallest mean power passed to `log10` in the third-octave levels
        /// (f64 accumulator; −180 dB). Range: 1e-24..=1e-12.
        ltas_power_floor: f64 = 1e-18,
        /// Smallest power passed to `log10` in the cluster statistics (f32;
        /// −180 dB). Range: 1e-24..=1e-12.
        power_floor: f32 = 1e-18,
        /// Fewest voiced frames for a tessitura. Range: 5..=50.
        tessitura_min_frames: usize = 10,
        /// The tessitura's reported percentiles: p10, p25, p50, p75, p90.
        /// Range: exactly these unless the struct's fields change.
        tessitura_percentiles: [f32; 5] = [10.0, 25.0, 50.0, 75.0, 90.0],
        /// Robust low extreme percentile. Range: 0.0..=10.0.
        tessitura_extreme_lo_pct: f32 = 2.0,
        /// Robust high extreme percentile. Range: 90.0..=100.0.
        tessitura_extreme_hi_pct: f32 = 98.0,
        /// Harmonics considered for the dominant-harmonic measure (H1..=Hn).
        /// Range: 2..=4.
        dominant_harmonic_candidates: usize = 3,
        /// `|r − r_prev|` below this treats a sign change as a midpoint
        /// crossing (turnover interpolation). Range: 1e-9..=1e-3.
        turnover_flat_eps: f32 = 1e-6,
    }
}
