//! Coral's DSP parameters (`apps/coral/src/config/*.ts` and the local
//! defaults in `src/audio/*.ts`), moved here for the Rust port in
//! `crate::choir`. Every number Coral's worker used is a field; the
//! published tables (JI ratios, tuning profiles, chord templates, SATB
//! ranges) stay as documented consts in `choir::harmony` and
//! `choir::labeler`, as `tract_data.rs` does for Story's bases.

use super::stage_config;

stage_config! {
    /// Coral's STFT framing and wire encoding (`FFT_CONFIG`,
    /// `DETECTOR_CONFIG.fftSize`, `spectrogram-encoding.ts`). The pipeline's
    /// `choir` mode frames at the DETECTION size; the display size is
    /// recorded for the renderer.
    pub struct ChoirStftConfig, section = "choir_stft" {
        /// Detection STFT size (decoupled from display in Coral: resolves bass
        /// semitones, ~5.4 Hz bins at 44.1 kHz). Range: power of two, 2048..=16384.
        detection_fft_size: usize = 8192,
        /// Detection hop, samples (fftSize / 4). Range: 1..=detection_fft_size.
        detection_hop: usize = 2048,
        /// Display STFT size Coral's waterfall used. Range: power of two.
        display_fft_size: usize = 2048,
        /// Display hop, samples. Range: 1..=display_fft_size.
        display_hop: usize = 512,
        /// u8 wire encoding floor, dBFS. Range: -200.0..=-60.0.
        src_floor_db: f32 = -140.0,
        /// u8 wire encoding ceiling, dBFS (+12 dB headroom for coherent sums).
        /// Range: 0.0..=24.0.
        src_ceil_db: f32 = 12.0,
        /// Epsilon added before the log. Range: 1e-15..=1e-6.
        magnitude_epsilon: f32 = 1.0e-10,
        /// Level meter cadence, ms. Range: 10..=500.
        level_interval_ms: f32 = 50.0,
    }
}

stage_config! {
    /// Coral's multi-F0 note detector (`DETECTOR_CONFIG` plus the
    /// `NoteDetector` local defaults). Detection runs on the whitened
    /// spectrum by iterative harmonic cancellation.
    pub struct ChoirDetectorConfig, section = "choir_detector" {
        /// Lowest candidate frequency, Hz. Range: 20.0..=200.0.
        min_freq_hz: f32 = 50.0,
        /// Highest candidate frequency, Hz. Range: 1000.0..=Nyquist.
        max_freq_hz: f32 = 8000.0,
        /// The rehearsal detector's candidate ceiling, Hz (just above C6).
        /// Range: 800.0..=2000.0.
        harmony_max_hz: f32 = 1150.0,
        /// Legacy path dBFS threshold; also the whitening noise gate. Range: -100.0..=0.0.
        threshold_db: f32 = -50.0,
        /// Harmonics summed per candidate (K), fundamental included. Range: 1..=32.
        harmonic_count: usize = 9,
        /// Unitless salience floor (flat whitened region = 1.0). Range: 1.0..=10.0.
        salience_threshold: f32 = 3.0,
        /// Whitening moving-average width, Hz (→ bins at the detection STFT).
        /// Range: 200.0..=8000.0.
        whitening_window_hz: f32 = 2000.0,
        /// Max simultaneous F0s the canceller extracts per frame. Range: 1..=16.
        max_polyphony: usize = 8,
        /// Fraction of the modelled amplitude removed per cancellation. Range: 0.0 < x ≤ 1.0.
        cancel_factor: f32 = 0.9,
        /// Collapse runs of active semitones with gaps ≤ this to the centroid. Range: 0..=3.
        merge_radius: usize = 1,
        /// Per-frame multiplicative decay of smoothed salience. Range: 0.0 < x < 1.0.
        decay_per_frame: f32 = 0.92,
        /// Harmonic-family margin, dB (0 = off; 8 = solo rehearsal). Range: 0.0..=20.0.
        family_margin_db: f32 = 0.0,
        /// Harmonics cancelled per accepted note (Kc). Range: 1..=32.
        cancel_harmonic_count: usize = 16,
        /// Headroom multiplier on the neighbour envelope during cancellation. Range: ≥ 1.0.
        cancel_headroom: f32 = 1.0,
        /// Minimum whitened peak in the fundamental band for eligibility. Range: 0.0..=5.0.
        fundamental_floor: f32 = 1.0,
        /// Partial levels below this, dB, count as absent. Range: -140.0..=-40.0.
        partial_floor_db: f32 = -90.0,
        /// Family test: smallest harmonic number a child can sit on. Range: 2..=4.
        family_k_min: usize = 2,
        /// Family test: largest harmonic number. Range: family_k_min..=32.
        family_k_max: usize = 12,
        /// Family test: a candidate is "on" a harmonic within this many cents. Range: 10.0..=100.0.
        family_cents: f32 = 50.0,
        /// Envelope fit: points more than this above the trend are dropped, dB. Range: 1.0..=12.0.
        fit_outlier_db: f32 = 4.0,
        /// Envelope fit default slope, dB per octave of k. Range: -12.0..=0.0.
        fit_default_slope: f32 = -6.0,
        /// Envelope fit default intercept when no points, dB. Range: -120.0..=0.0.
        fit_default_intercept: f32 = -60.0,
        /// Chest-voice prior: H2 may sit this far above H1, dB. Range: 0.0..=12.0.
        prior_h2_db: f32 = 6.0,
        /// Chest-voice prior: H3 may sit this far above H1, dB. Range: 0.0..=12.0.
        prior_h3_db: f32 = 3.0,
        /// Whitening envelope floor (division guard). Range: 1e-15..=1e-6.
        envelope_floor: f32 = 1.0e-12,
    }
}

stage_config! {
    /// Coral's single-F0 estimator (`qifft.ts`, `INTONATION_CONFIG`):
    /// parabolic interpolation of the magnitude peak in the vocal band.
    pub struct QifftConfig, section = "qifft" {
        /// Lowest fundamental searched, Hz (~C2). Range: 30.0..=200.0.
        min_f0_hz: f32 = 65.0,
        /// Highest fundamental searched, Hz (~C#6). Range: 500.0..=2000.0.
        max_f0_hz: f32 = 1100.0,
        /// Voiced if peak / mean magnitude ≥ this. Range: 1.0..=20.0.
        voiced_ratio: f32 = 5.0,
        /// Sub-octave check: a peak at half the bin at least this fraction of
        /// the global peak (and a local maximum) is the fundamental. Range: 0.0..=1.0.
        sub_octave_ratio: f32 = 0.3,
        /// Confidence = min(1, peak/mean / (this · voiced_ratio)). Range: 1.0..=4.0.
        confidence_divisor: f32 = 2.0,
        /// Epsilon under the log. Range: 1e-15..=1e-6.
        log_epsilon: f32 = 1.0e-12,
    }
}

stage_config! {
    /// Coral's SATB section labeler (`section-labeler.ts`): pitch-range
    /// priors, a monotonic DP over the sorted notes, and an `A/T?`
    /// abstention class in the alto/tenor overlap.
    pub struct ChoirLabelerConfig, section = "choir_labeler" {
        /// Bass range, MIDI, low (E2). Range: 30..=50.
        bass_lo: i32 = 40,
        /// Bass range high (E4). Range: bass_lo..=70.
        bass_hi: i32 = 64,
        /// Tenor range low (C3). Range: 40..=55.
        tenor_lo: i32 = 48,
        /// Tenor range high (A4). Range: tenor_lo..=75.
        tenor_hi: i32 = 69,
        /// Alto range low (F3). Range: 45..=60.
        alto_lo: i32 = 53,
        /// Alto range high (F5). Range: alto_lo..=84.
        alto_hi: i32 = 77,
        /// Soprano range low (C4). Range: 55..=65.
        soprano_lo: i32 = 60,
        /// Soprano range high (C6). Range: soprano_lo..=96.
        soprano_hi: i32 = 84,
        /// Alto/tenor overlap register, low (G3). Range: 48..=60.
        at_zone_lo: i32 = 55,
        /// Alto/tenor overlap register, high (A4). Range: at_zone_lo..=72.
        at_zone_hi: i32 = 69,
        /// |SPR lean| below this cannot resolve A vs T. Range: 0.0..=1.0.
        spr_margin: f32 = 0.3,
        /// Fit penalty for two notes in one section (divisi). Range: 0.0..=2.0.
        repeat_penalty: f32 = 0.5,
        /// Range fit outside a range: max(floor, scale · exp(−over / decay)). The floor.
        /// Range: 0.0..=0.2.
        outside_floor: f32 = 0.02,
        /// Outside-range scale. Range: 0.0..=1.0.
        outside_scale: f32 = 0.25,
        /// Outside-range decay, semitones. Range: 1.0..=24.0.
        outside_decay_semitones: f32 = 6.0,
        /// Range fit inside a range: max(this, 1 − |m − centre| / half). Range: 0.0..=1.0.
        inside_floor: f32 = 0.3,
        /// Detector factor = clamp(confidence / this, min, 1). The divisor. Range: 1.0..=8.0.
        detector_factor_divisor: f32 = 2.0,
        /// Detector factor floor. Range: 0.0..=1.0.
        detector_factor_min: f32 = 0.3,
        /// Confidence cap when the SPR lean decides. Range: 0.0..=1.0.
        spr_confidence_max: f32 = 0.9,
        /// Confidence multiplier for a pitch-order-bracketed overlap note. Range: 0.0..=1.0.
        bracketed_discount: f32 = 0.6,
        /// Confidence of an `A/T?` abstention. Range: 0.0..=1.0.
        ambiguous_confidence: f32 = 0.3,
        /// Singer's-formant band for the SPR lean, low, Hz. Range: 1000.0..=3000.0.
        spr_band_lo_hz: f32 = 2000.0,
        /// SPR band high, Hz. Range: spr_band_lo_hz..=6000.0.
        spr_band_hi_hz: f32 = 4000.0,
        /// Tenor/alto formant midpoint, Hz. Range: 2000.0..=3500.0.
        spr_center_hz: f32 = 2800.0,
        /// Lean = clamp((centroid − centre) / this, −1, 1), Hz. Range: 50.0..=1000.0.
        spr_scale_hz: f32 = 250.0,
        /// Peak search half-width around each partial, bins. Range: 0..=8.
        spr_peak_bins: usize = 2,
    }
}

stage_config! {
    /// Coral's rehearsal harmony analysis (`harmony.ts`): card state,
    /// tolerance bands, drift memory, chord hold, scatter measure and the
    /// chord-identification score.
    pub struct ChoirHarmonyConfig, section = "choir_harmony" {
        /// Reference A4, Hz. Range: 415.0..=466.0.
        a4_hz: f32 = 440.0,
        /// A card keeps a note this long after it was last seen, ms. Range: 50.0..=2000.0.
        hold_ms: f32 = 400.0,
        /// Detection frames before a card lights (≈140 ms). Range: 1..=20.
        onset_frames: u32 = 3,
        /// Frames of consistent relabelling before a card abandons a note (≈0.6 s). Range: 1..=60.
        relabel_frames: u32 = 12,
        /// Frames a suspect (partial-of-a-held-note) candidate must persist. Range: 1..=60.
        suspect_frames: u32 = 6,
        /// A chord is "settled" after this long, ms. Range: 50.0..=2000.0.
        chord_hold_ms: f32 = 300.0,
        /// Drift memory: r ← memory · r + (1 − memory) · τ. Range: 0.0..=1.0.
        drift_memory: f32 = 0.85,
        /// Drift updates at least every this many ms without a chord change. Range: 200.0..=10000.0.
        drift_interval_ms: f32 = 2000.0,
        /// In-tune band, cents. Range: 1.0..=30.0.
        band_in_cents: f32 = 10.0,
        /// Marginal band edge, cents. Range: band_in_cents..=60.0.
        band_out_cents: f32 = 20.0,
        /// In-tune band for short notes or the flat side when vibrato-heavy. Range: 1.0..=40.0.
        band_wide_in_cents: f32 = 20.0,
        /// Marginal edge for the widened band. Range: band_wide_in_cents..=80.0.
        band_wide_out_cents: f32 = 30.0,
        /// Notes held shorter than this get the widened band, ms. Range: 50.0..=1000.0.
        short_note_ms: f32 = 250.0,
        /// Median window of recent pitches per card, frames. Range: 1..=15.
        history_len: usize = 5,
        /// A candidate within this of partial 2–4 of a held note is a suspect, cents. Range: 5.0..=100.0.
        suspect_cents: f32 = 40.0,
        /// A card follows the nearest voice within this, cents. Range: 20.0..=200.0.
        continuity_cents: f32 = 80.0,
        /// Highest partial a suspect is checked against. Range: 2..=8.
        suspect_k_max: u32 = 4,
        /// Orphans need at least this section confidence to displace a card. Range: 0.0..=1.0.
        orphan_confidence_min: f32 = 0.2,
        /// Card-claim score = salience · max(this, section confidence). Range: 0.0..=1.0.
        claim_confidence_floor: f32 = 0.05,
        /// Scatter band edges, cents: tight below this. Range: 1.0..=30.0.
        scatter_tight_cents: f32 = 10.0,
        /// Typical below this. Range: scatter_tight_cents..=40.0.
        scatter_typical_cents: f32 = 15.0,
        /// Loose below this; scattered above. Range: scatter_typical_cents..=80.0.
        scatter_loose_cents: f32 = 30.0,
        /// Highest partial examined for scatter. Range: 2..=20.
        scatter_k_max: u32 = 10,
        /// Lowest partial examined for scatter. Range: 2..=scatter_k_max.
        scatter_k_min: u32 = 2,
        /// A partial below this frequency is not used for scatter, Hz. Range: 100.0..=2000.0.
        scatter_min_hz: f32 = 500.0,
        /// Scatter search half-width around the partial, cents. Range: 10.0..=200.0.
        scatter_half_cents: f32 = 60.0,
        /// Scatter search half-width floor, bins. Range: 1..=10.
        scatter_half_bins_min: u32 = 3,
        /// A partial must sit this far above the floor, dB. Range: 5.0..=60.0.
        scatter_min_above_floor_db: f32 = 20.0,
        /// Width cut: this far below the peak, dB. Range: 3.0..=30.0.
        scatter_cut_below_peak_db: f32 = 12.0,
        /// … but never lower than this above the floor, dB. Range: 0.0..=20.0.
        scatter_cut_above_floor_db: f32 = 6.0,
        /// Isolation: another voice's partial within this many cents disqualifies. Range: 10.0..=100.0.
        scatter_isolation_cents: f32 = 50.0,
        /// Pair tune = exp(−(err / this)²), cents. Range: 5.0..=60.0.
        tune_sigma_cents: f32 = 20.0,
        /// Consonance index weight of pair tuning. Range: 0.0..=1.0.
        consonance_tune_weight: f32 = 0.6,
        /// Consonance index weight of interval-class consonance. Range: 0.0..=1.0.
        consonance_class_weight: f32 = 0.4,
        /// Chord score: points per covered pitch class. Range: 1.0..=20.0.
        chord_score_covered: f32 = 10.0,
        /// Chord score: points off per missing chord tone. Range: 0.0..=10.0.
        chord_score_missing: f32 = 4.0,
        /// Chord score: bonus when the lowest voice is the root. Range: 0.0..=5.0.
        chord_score_root_bonus: f32 = 2.0,
        /// Chord score: penalty per template tone (prefers smaller templates). Range: 0.0..=1.0.
        chord_score_size_penalty: f32 = 0.1,
        /// Hann main-lobe RMS width integration range, bins. Range: 1.0..=6.0.
        hann_sigma_range_bins: f32 = 3.0,
        /// … and step, bins. Range: 0.001..=0.1.
        hann_sigma_step_bins: f32 = 0.01,
        /// Level meter epsilon under the log. Range: 1e-15..=1e-6.
        level_epsilon: f32 = 1.0e-10,
        /// Scatter/detection-spectrum floor estimate samples every this many bins. Range: 1..=64.
        floor_sample_stride: usize = 8,
        /// Floor fallback when the spectrum is empty, dB. Range: -200.0..=-60.0.
        floor_fallback_db: f32 = -120.0,
    }
}
