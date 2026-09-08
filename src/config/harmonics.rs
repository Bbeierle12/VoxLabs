//! Harmonic-series, voice-quality, timbre, and classification stage
//! configuration (the per-frame measures in `math.rs`).

use super::stage_config;
stage_config! {
    /// Harmonic amplitudes at k·f0 by Hann-windowed Goertzel evaluation.
    pub struct HarmonicsConfig, section = "harmonics" {
        /// Frames shorter than this (samples) return all-zero amplitudes.
        /// Range: 16..=256.
        min_frame_samples: usize = 32,
    }
}

stage_config! {
    /// Harmonics-to-noise ratio (Boersma/Praat style autocorrelation).
    pub struct HnrConfig, section = "hnr" {
        /// Frames shorter than this (samples) are not measured. Range: 32..=512.
        min_frame_samples: usize = 64,
        /// Pitch period in samples must be at least this (else no lag
        /// neighbourhood exists). Range: exactly 2.
        min_period_samples: usize = 2,
        /// The period plus this margin must fit in the frame, so the
        /// parabolic neighbours `lag ± 1` are in bounds. Range: exactly 2.
        period_margin_samples: usize = 2,
        /// Zero-lag autocorrelation at or below this is silence. Range: 1e-15..=1e-9.
        energy_eps: f32 = 1e-12,
        /// `|y0 − 2·y1 + y2|` below this skips the parabolic peak refinement.
        /// Range: 1e-15..=1e-9.
        parabolic_flat_eps: f32 = 1e-12,
        /// Normalized correlation is clamped to ±this before the log mapping
        /// so `r / (1 − r)` stays finite. Range: 0.99..=0.999999.
        r_clamp: f32 = 0.9999,
        /// Output is clamped to ±this many dB; a non-positive correlation
        /// reports the negative limit. Range: 20.0..=60.0.
        db_limit: f32 = 40.0,
    }
}

stage_config! {
    /// Jitter and shimmer from peak-picked cycle marks.
    pub struct PerturbationConfig, section = "perturbation" {
        /// Frames shorter than this (samples) are not measured. Range: 32..=512.
        min_frame_samples: usize = 64,
        /// Periods shorter than this (samples) are too short to mark.
        /// Range: 4.0..=16.0.
        min_period_samples: f32 = 8.0,
        /// The first mark is the strongest sample within this many periods
        /// from the frame start. Range: 1.0..=2.0.
        first_search_periods: f32 = 1.5,
        /// Each next mark is sought from this fraction of a period after the
        /// previous one (−30 %). Range: 0.5..=0.9.
        search_window_lo: f32 = 0.7,
        /// …to this fraction (+30 %); intervals outside the window reject the
        /// frame. Range: 1.1..=1.5.
        search_window_hi: f32 = 1.3,
        /// Fewest marks (cycles) for a measurement. Range: 3..=10.
        min_cycles: usize = 5,
        /// Jitter above this percentage means the marks are not tracking real
        /// glottal cycles (severe pathology is ~2–3 %). Range: 2.0..=10.0.
        max_jitter_pct: f32 = 5.0,
        /// A mark whose height is at or below this is unusable for shimmer.
        /// Range: 1e-9..=1e-3.
        amp_eps: f32 = 1e-6,
        /// `|y0 − 2·y1 + y2|` below this skips sub-sample refinement of a
        /// mark. Range: 1e-15..=1e-9.
        parabolic_flat_eps: f32 = 1e-12,
    }
}

stage_config! {
    /// Cepstral peak prominence (Hillenbrand-style).
    pub struct CppConfig, section = "cpp" {
        /// Frames shorter than this (samples) are not measured. Range: 128..=1024.
        min_frame_samples: usize = 256,
        /// Frame energy (sum of squares) below this is silence. Range: 1e-12..=1e-6.
        silence_energy: f32 = 1e-9,
        /// Added to power before `log10` so silent bins stay finite (dB
        /// floor of −120). Range: 1e-15..=1e-9.
        log_floor: f32 = 1e-12,
        /// Low edge of the voice-pitch quefrency band, Hz (long quefrency).
        /// Range: 40.0..=100.0.
        f0_band_lo_hz: f32 = 60.0,
        /// High edge of the quefrency band, Hz (short quefrency).
        /// Range: 300.0..=800.0.
        f0_band_hi_hz: f32 = 500.0,
        /// The band must span at least this many quefrency bins. Range: 2..=16.
        min_band_bins: usize = 4,
        /// Regression denominator below this is degenerate. Range: 1e-12..=1e-6.
        regression_eps: f32 = 1e-9,
    }
}

stage_config! {
    /// Spectral centroid ("brightness") over the full spectrum.
    pub struct CentroidConfig, section = "centroid" {
        /// Frames shorter than this (samples) are not measured. Range: 128..=1024.
        min_frame_samples: usize = 256,
        /// Frame energy (sum of squares) below this is silence. Range: 1e-12..=1e-6.
        silence_energy: f32 = 1e-9,
        /// Low edge of the analysed band, Hz. Range: 20.0..=200.0.
        band_lo_hz: f32 = 80.0,
        /// High edge of the analysed band, Hz (capped at Nyquist). The
        /// brightness thresholds were tuned against this band.
        /// Range: 4000.0..=16000.0.
        band_hi_hz: f32 = 8000.0,
        /// Total magnitude below this yields no centroid. Range: 1e-12..=1e-6.
        magnitude_eps: f32 = 1e-9,
    }
}

stage_config! {
    /// Harmonic-shape measures: H1–H2, A2/A1, tilt, even/odd balance,
    /// singer's-formant share, and the plain-language timbre label.
    pub struct TimbreConfig, section = "timbre" {
        /// Partials more than this many dB below the strongest are numerical
        /// noise and are excluded (H1–H2 refuses; tilt drops them; A2/A1
        /// floors A2 here). Range: -80.0..=-30.0.
        relative_floor_db: f32 = -48.0,
        /// Strongest partial at or below this (linear) is silence. Range: 1e-9..=1e-3.
        amp_eps: f32 = 1e-6,
        /// Fewest partials above the floor for a tilt fit. Range: exactly 2.
        tilt_min_points: usize = 2,
        /// Tilt regression denominator below this is degenerate. Range: 1e-12..=1e-6.
        tilt_regression_eps: f32 = 1e-9,
        /// Partials H2..=this take part in the even/odd balance. Range: 4..=32.
        even_odd_partials: usize = 16,
        /// Singer's-formant band, low edge, Hz. Range: 2000.0..=3000.0.
        singers_formant_lo_hz: f32 = 2_800.0,
        /// Singer's-formant band, high edge, Hz. Range: 3000.0..=4000.0.
        singers_formant_hi_hz: f32 = 3_400.0,
        /// Total harmonic energy below this yields no singer's-formant share.
        /// Range: 1e-15..=1e-9.
        energy_eps: f32 = 1e-12,
        /// Even/odd balance below this many dB reads "hollow". Range: -12.0..=-3.0.
        hollow_even_odd_db: f32 = -6.0,
        /// A partial above this fraction of the strongest counts as strong.
        /// Range: 0.1..=0.5.
        strong_partial_rel_amp: f32 = 0.25,
        /// At least this many strong partials reads "rich". Range: 4..=10.
        rich_min_strong: usize = 6,
        /// At most this many strong partials reads "pure". Range: 0..=3.
        pure_max_strong: usize = 2,
    }
}

stage_config! {
    /// Coarse voice-range and brightness labels (Personal Harmonic
    /// Identifier prototype thresholds).
    pub struct VoiceClassConfig, section = "voice_class" {
        /// Mean f0 below this is "Bass", Hz. Range: 100.0..=150.0.
        bass_max_hz: f32 = 130.0,
        /// …below this "Baritone". Range: bass_max_hz..=200.0.
        baritone_max_hz: f32 = 175.0,
        /// …below this "Tenor". Range: baritone_max_hz..=250.0.
        tenor_max_hz: f32 = 220.0,
        /// …below this "Alto". Range: tenor_max_hz..=320.0.
        alto_max_hz: f32 = 290.0,
        /// …below this "Mezzo-Soprano", else "Soprano". Range: alto_max_hz..=420.0.
        mezzo_max_hz: f32 = 370.0,
        /// Centroid below this is "Dark", Hz. Range: 800.0..=1600.0.
        dark_max_hz: f32 = 1200.0,
        /// …below this "Warm". Range: dark_max_hz..=2600.0.
        warm_max_hz: f32 = 2200.0,
        /// …below this "Balanced". Range: warm_max_hz..=3600.0.
        balanced_max_hz: f32 = 3200.0,
        /// …below this "Bright", else "Brilliant". Range: balanced_max_hz..=5000.0.
        bright_max_hz: f32 = 4400.0,
    }
}

stage_config! {
    /// Musical reference tuning.
    pub struct TuningConfig, section = "tuning" {
        /// Frequency of A4, Hz — the anchor of every note name and semitone
        /// distance. Range: 415.0..=466.0 (concert pitch conventions).
        a4_hz: f32 = 440.0,
    }
}
