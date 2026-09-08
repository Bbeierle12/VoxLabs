//! TV-path spatial calibration and scoring configuration (`spatial.rs`).

use super::stage_config;

stage_config! {
    /// Two-channel relative-transfer-path learning and contrastive scoring.
    pub struct SpatialConfig, section = "spatial" {
        /// STFT frame length, samples (2048 @ 48 kHz ≈ 43 ms, 23.4 Hz bins).
        /// Range: power of two, 1024..=8192.
        fft_n: usize = 2048,
        /// STFT hop, samples (50 % overlap). Range: fft_n/4..=fft_n.
        hop: usize = 1024,
        /// Microphone channels in the pair. Range: exactly 2.
        channels: usize = 2,
        /// Accumulator capacity, in frames, reserved per channel. Range: 1..=8.
        accumulator_frames: usize = 2,
        /// Usable (non-quiet) frames a calibration accumulates (512 hops ≈ 11 s).
        /// Range: 64..=2048.
        calib_frames: usize = 512,
        /// Low edge of the calibrated/scored band, Hz (port roll-off below).
        /// Range: 100.0..=400.0.
        band_lo_hz: f32 = 200.0,
        /// High edge of the band, Hz (TV content is sparse above the codec
        /// ceiling). Range: 4000.0..=Nyquist.
        band_hi_hz: f32 = 8000.0,
        /// Frame RMS (dBFS, both channels) below which no score is computed.
        /// Range: -90.0..=-40.0.
        quiet_dbfs: f32 = -65.0,
        /// Smallest mean-square passed to `log10` in the quiet gate.
        /// Range: 1e-24..=1e-12.
        energy_floor: f64 = 1e-18,
        /// EMA coefficient for displayed scores (per scored frame, ~21 ms).
        /// Range: 0.01..=0.5.
        score_ema_alpha: f32 = 0.1,
        /// A bin with coherence γ² below this is uncovered. Range: 0.2..=0.8.
        covered_coherence: f32 = 0.5,
        /// Energy-weighted mean λ2/λ1 above which the calibration warns of a
        /// second source or heavy reverberation. Range: 0.05..=0.5.
        rank_warn: f32 = 0.2,
        /// Calibration fails if fewer than calib_frames usable frames arrive
        /// in calib_frames × this many wall frames. Range: 2..=10.
        calib_max_wall_factor: usize = 3,
        /// Single-path score at or above this reads "TV". Range: 0.6..=0.95.
        score_tv: f32 = 0.85,
        /// Single-path score at or below this reads "not TV". Range: 0.1..=score_tv.
        score_not_tv: f32 = 0.40,
        /// Single-path band coverage under which the score is weak evidence.
        /// Range: 0.01..=0.5.
        single_weak_coverage: f32 = 0.10,
        /// A bin separates the paths when 1 − |ĥ_Tᴴĥ_U|² reaches this.
        /// Range: 0.05..=0.5.
        sep_min: f32 = 0.2,
        /// Fraction of band bins that must separate before the contrast
        /// score counts as evidence. Range: 0.01..=0.3.
        disc_min: f32 = 0.05,
        /// Contrast at or above this reads "TV-like". Range: 0.1..=0.8.
        contrast_tv: f32 = 0.3,
        /// Contrast at or below this reads "you-like". Range: -0.8..=-0.1.
        contrast_user: f32 = -0.3,
        /// TV-path weight below which a bin is skipped in scoring. Range: 1e-6..=1e-1.
        weight_eps: f64 = 1e-3,
        /// Joint (separation) weight below which a bin is skipped in the
        /// contrast score. Range: 1e-6..=1e-2.
        joint_weight_eps: f64 = 1e-4,
        /// Weighted-energy denominator below which no score is reported.
        /// Range: 1e-15..=1e-9.
        denominator_eps: f64 = 1e-12,
        /// Covariance / eigenvector norm floor (f64), guarding divisions in
        /// the closed-form 2×2 eigen-decomposition. Range: 1e-40..=1e-20.
        eigen_eps: f64 = 1e-30,
    }
}
