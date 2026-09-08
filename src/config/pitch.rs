//! Pitch, voicing, ambient-floor, and room-calibration stage configuration.

use super::stage_config;

stage_config! {
    /// YIN fundamental-frequency estimator (de Cheveigné & Kawahara 2002),
    /// shared by the CPU reference path and the desktop GPU path.
    pub struct YinConfig, section = "yin" {
        /// Analysis window length in samples: the number of samples compared
        /// per lag in the difference function. The analysis frame must be at
        /// least this plus the maximum search lag. Range: 256..=frame/2;
        /// the GPU shader sizes its buffers from this, so change both.
        window: usize = 1024,
        /// Lowest fundamental searched, Hz. Sets the maximum lag
        /// (`sample_rate / f0_min_hz`). Range: 20.0..=200.0.
        f0_min_hz: f32 = 50.0,
        /// Highest fundamental searched, Hz. Sets the minimum lag.
        /// Range: f0_min_hz..=sample_rate/4.
        f0_max_hz: f32 = 1000.0,
        /// Absolute threshold on the cumulative-mean-normalized difference:
        /// the first dip below it is the period. Range: 0.05..=0.3 (the
        /// paper's 0.1; higher accepts weaker periodicity).
        threshold: f32 = 0.12,
        /// Smallest maximum lag worth searching, in samples; below it the
        /// frame is "too short to analyze". Range: 2..=8.
        min_max_lag: usize = 3,
        /// Smallest minimum lag, in samples (f0 cannot exceed sample_rate/2).
        /// Range: exactly 2 unless the search bounds change.
        min_tau: usize = 2,
        /// Smallest usable window, in samples. Range: 2..=16.
        min_window: usize = 2,
        /// `|s0 − 2·s1 + s2|` below this is a flat neighbourhood and the
        /// parabolic lag refinement is skipped. Range: 1e-9..=1e-3.
        parabolic_flat_eps: f32 = 1e-6,
    }
}

stage_config! {
    /// Voicing decision on top of YIN: confidence and range gates, then the
    /// SNR and calibrated-hum gates.
    pub struct VoicingConfig, section = "voicing" {
        /// YIN confidence (`1 − d'(τ)`) a frame must exceed to be voiced.
        /// Range: 0.2..=0.8.
        min_confidence: f32 = 0.4,
        /// Lowest f0 accepted as voice, Hz. Range: yin.f0_min_hz..=100.0.
        f0_min_hz: f32 = 50.0,
        /// Highest f0 accepted as voice, Hz. Range: 500.0..=yin.f0_max_hz.
        f0_max_hz: f32 = 1000.0,
        /// Minimum frame-over-floor SNR (dB) for a YIN-voiced frame to count
        /// as voice at all; below it the frame is demoted to unvoiced.
        /// Engineering floor. Range: 6.0..=25.0.
        voiced_min_snr_db: f32 = 15.0,
        /// Minimum SNR (dB) for a frame to contribute identity data
        /// (voiceprint formants, VTL): the ASHA measurement-grade floor
        /// (Patel et al. 2018). Range: 20.0..=40.0.
        identity_min_snr_db: f32 = 30.0,
    }
}

stage_config! {
    /// Ambient-noise floor tracker (`math::NoiseFloor`): learns the room
    /// from frames the pitch detector calls unvoiced.
    pub struct NoiseFloorConfig, section = "noise_floor" {
        /// Ring capacity in unvoiced frames (~6 s at ~21.5 frames/s).
        /// Range: 32..=1024; must be ≥ ring_min.
        ring_len: usize = 128,
        /// Unvoiced frames observed before the floor is trusted (~1.5 s).
        /// Until then no SNR is reported and nothing is gated.
        /// Range: 8..=ring_len.
        ring_min: usize = 32,
        /// Percentile of the ring taken as the floor — low, so breaths and
        /// consonants do not drag it toward the voice. Range: 0.02..=0.5.
        floor_percentile: f32 = 0.10,
        /// Smallest floor used in the SNR division (linear RMS), so digital
        /// silence yields a large finite SNR, not infinity. Range: 1e-9..=1e-5.
        floor_min_rms: f32 = 1e-7,
    }
}

stage_config! {
    /// One room-calibration pass (`math::RoomCalibrator`) and the
    /// calibrated-interferer gate.
    pub struct RoomCalibrationConfig, section = "room_calibration" {
        /// Frames in one pass (~4.6 s at ~21.5 frames/s): long enough for a
        /// stable level and hum-pitch estimate. Range: 40..=400.
        frames: u32 = 100,
        /// A pass with fewer than `frames / too_short_divisor` frames is
        /// rejected as too short. Range: 1..=4.
        too_short_divisor: u32 = 2,
        /// Fractional f0 tolerance for matching a live detection against the
        /// calibrated interferer (±4 % ≈ two thirds of a semitone).
        /// Range: 0.01..=0.1.
        interferer_f0_tol: f32 = 0.04,
        /// A frame louder than the calibrated interferer by this many dB is
        /// the singer, even at the hum's pitch. Range: 3.0..=20.0.
        interferer_level_margin_db: f32 = 10.0,
        /// Smallest interferer level used in the match (linear RMS).
        /// Range: 1e-9..=1e-5.
        interferer_min_rms: f32 = 1e-7,
        /// Voiced fraction of the window above which an *unstable* pitch
        /// means someone was talking — the pass fails. Range: 0.1..=0.6.
        voiced_fail_fraction: f32 = 0.3,
        /// Voiced fraction above which a *stable* pitch is fingerprinted as
        /// a real periodic interferer. Range: 0.05..=voiced_fail_fraction.
        interferer_min_fraction: f32 = 0.15,
        /// Relative f0 standard deviation separating machine hum (below)
        /// from anything vocal (above). Range: 0.005..=0.05.
        stable_rel_std: f32 = 0.02,
        /// Smallest mean f0 (Hz) used as the relative-std denominator.
        /// Range: 0.1..=10.0.
        rel_std_min_mean_hz: f32 = 1.0,
    }
}
