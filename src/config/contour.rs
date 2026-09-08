//! f0-contour (vibrato / steadiness) stage configuration.

use super::stage_config;

stage_config! {
    /// Vibrato detection and sustain steadiness over the recent f0 contour
    /// (`metrics::F0Contour`).
    pub struct VibratoConfig, section = "vibrato" {
        /// Low edge of the vibrato rate search band, Hz. Range: 2.0..=5.0.
        rate_min_hz: f32 = 3.0,
        /// High edge of the search band, Hz (contour Nyquist is ~10 Hz).
        /// Range: 6.0..=10.0.
        rate_max_hz: f32 = 9.0,
        /// DFT sweep step across the band, Hz. Range: 0.05..=0.5.
        rate_step_hz: f32 = 0.25,
        /// Rates within this of a band edge are drift/noise, not vibrato.
        /// Range: 0.0..=1.0.
        edge_margin_hz: f32 = 0.25,
        /// Minimum fraction of contour variance the fitted sinusoid must
        /// explain. Range: 0.2..=0.8.
        min_explained: f32 = 0.4,
        /// Minimum extent (± cents) to call vibrato rather than jitter.
        /// Range: 3.0..=20.0.
        min_extent_cents: f32 = 8.0,
        /// Rolling window of voiced contour, seconds. Range: 1.0..=4.0.
        window_secs: f32 = 2.0,
        /// Continuous voicing required before anything is reported, seconds.
        /// Range: 0.5..=window_secs.
        min_secs: f32 = 1.0,
        /// Unvoiced gap tolerated before the window resets, seconds.
        /// Range: 0.1..=0.5.
        max_gap_secs: f32 = 0.25,
        /// Floor on the window and minimum lengths, in contour samples, so a
        /// very low contour rate still yields a usable buffer. Range: 4..=16.
        min_buffer_samples: usize = 8,
        /// Contour variance (cents²) below this is a dead-flat sustain: no
        /// vibrato, steadiness = its RMS. Range: 1e-6..=1e-2.
        flat_variance: f32 = 1e-4,
        /// Slack added to the band's top edge so the sweep's last step is
        /// not lost to rounding. Range: 1e-9..=1e-3.
        sweep_end_eps: f32 = 1e-6,
        /// `|y0 − 2·y1 + y2|` below this skips parabolic rate refinement.
        /// Range: 1e-15..=1e-9.
        parabolic_flat_eps: f32 = 1e-12,
        /// Detrend regression denominator below this is degenerate.
        /// Range: 1e-12..=1e-6.
        detrend_eps: f32 = 1e-9,
        /// Sinusoid-fit normal-equation determinant below this is singular.
        /// Range: 1e-12..=1e-6.
        fit_det_eps: f32 = 1e-9,
    }
}
