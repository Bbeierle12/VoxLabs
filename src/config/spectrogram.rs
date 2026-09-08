//! Spectrogram STFT and file-import resampler configuration.

use super::stage_config;

stage_config! {
    /// Hop-based STFT feeding the scrolling waterfall (`spectrogram.rs`).
    pub struct SpectrogramConfig, section = "spectrogram" {
        /// FFT window length, samples; matches the analysis frame so the
        /// loop can feed the same slices. Range: power of two, 512..=8192.
        fft_size: usize = 2048,
        /// Samples advanced between frames (512 → four frames per 2048-sample
        /// analysis frame). Range: 64..=fft_size.
        hop: usize = 512,
        /// Magnitude floor (dB) reported for silent / empty bins.
        /// Range: -160.0..=-60.0.
        db_floor: f32 = -120.0,
        /// Smallest magnitude passed to `log10` (−240 dB). Range: 1e-15..=1e-9.
        magnitude_floor: f32 = 1e-12,
    }
}

stage_config! {
    /// Windowed-sinc resampler used when an imported file's rate differs
    /// from the analysis rate (`audio_file::resample`).
    pub struct ResampleConfig, section = "resample" {
        /// Rates closer than this (Hz) are treated as equal: no resampling.
        /// Range: 0.0..=1.0.
        same_rate_tol_hz: f32 = 0.5,
        /// Lowpass cutoff as a fraction of the lower of the two rates
        /// (0.45 ≈ 90 % of the new Nyquist). Range: 0.3..=0.5.
        cutoff_of_rate: f32 = 0.45,
        /// Half-length of the sinc kernel, in input samples on each side.
        /// Range: 8..=128.
        half_taps: isize = 32,
        /// A sinc argument closer to zero than this takes the limit value.
        /// Range: 1e-9..=1e-3.
        sinc_center_eps: f32 = 1e-6,
    }
}
