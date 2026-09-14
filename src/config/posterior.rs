//! The Vocal Tract Lab posterior inverse (`pipeline::stages::posterior`),
//! the articulator readout and the Phase 3 validation bands. The model's
//! own numbers (modes, limits, abstention threshold) live in
//! `assets/vocal_tract_lab/reduced_model.json`, a published data table;
//! the numbers here are the app's fixed literals around it, moved out of
//! the decompiled `FrameAnalyzer`, `TemporalAtlasFilter`, `TractMeshCpu`
//! and `ArticulatorPosterior` (research/vocal-tract-lab-0.10.0/).

use super::stage_config;

stage_config! {
    /// `inverse/posterior_pca4`: evidence confidence, the temporal atlas
    /// filter, the area clamp and the uncertainty readout, as Vocal Tract
    /// Lab 0.10.0 computes them.
    pub struct PosteriorConfig, section = "posterior" {
        /// Weight of the f0 estimator's confidence (clamped to 0..1) in the
        /// acoustic-evidence score. Range: 0.0..=1.0; the three weights sum to 1.
        evidence_f0_confidence_weight: f32 = 0.42,
        /// Weight of harmonicity (0..1) in the acoustic-evidence score.
        /// Range: 0.0..=1.0.
        evidence_harmonicity_weight: f32 = 0.30,
        /// Weight of the SNR term in the acoustic-evidence score. Range: 0.0..=1.0.
        evidence_snr_weight: f32 = 0.28,
        /// SNR term: (snr_db + offset) / span, clamped to 0..1. The offset, dB.
        /// Range: 0.0..=20.0.
        evidence_snr_offset_db: f32 = 5.0,
        /// SNR term span, dB. Range: 5.0..=60.0.
        evidence_snr_span_db: f32 = 25.0,
        /// The app's harmonicity is its own 0..1 measure that VoxLabs does not
        /// compute; HNR (dB) is mapped linearly onto 0..1 over this span.
        /// Range: 5.0..=40.0.
        harmonicity_hnr_span_db: f32 = 20.0,
        /// High-f0 penalty: (f0 − start) / span, clamped to 0..max. Sparse
        /// harmonics above this f0, Hz, weaken the formant evidence.
        /// Range: 200.0..=800.0.
        f0_penalty_start_hz: f32 = 420.0,
        /// High-f0 penalty span, Hz. Range: 100.0..=2000.0.
        f0_penalty_span_hz: f32 = 600.0,
        /// High-f0 penalty ceiling. Range: 0.0..=1.0.
        f0_penalty_max: f32 = 0.28,
        /// Evidence = clamp(gain · acoustic + offset − penalty, min, max). The
        /// gain. Range: 0.0..=1.0.
        evidence_gain: f32 = 0.78,
        /// Evidence offset. Range: 0.0..=0.5.
        evidence_offset: f32 = 0.05,
        /// Evidence floor. Range: 0.0..=0.5.
        evidence_min: f32 = 0.03,
        /// Evidence ceiling. Range: 0.5..=1.0.
        evidence_max: f32 = 0.82,
        /// Temporal filter: the follow rate is clamp(gain · evidence + offset,
        /// min, max) when the frame is voiced and the evidence clears the
        /// model's abstention threshold. The gain. Range: 0.0..=1.0.
        filter_alpha_gain: f32 = 0.38,
        /// Follow-rate offset. Range: 0.0..=1.0.
        filter_alpha_offset: f32 = 0.12,
        /// Follow-rate floor. Range: 0.0..=1.0.
        filter_alpha_min: f32 = 0.12,
        /// Follow-rate ceiling. Range: filter_alpha_min..=1.0.
        filter_alpha_max: f32 = 0.46,
        /// Confidence follows the evidence at this rate on evidence frames.
        /// Range: 0.0..=1.0.
        confidence_follow: f32 = 0.35,
        /// Coefficients decay toward the atlas mean by this factor per
        /// abstained frame. Range: 0.0..=1.0.
        decay_coefficients: f32 = 0.9,
        /// Confidence decays by this factor per abstained frame. Range: 0.0..=1.0.
        decay_confidence: f32 = 0.82,
        /// Relative area standard deviation = (1 − confidence) · gain + offset
        /// (the app's uncertainty readout, `TractPosterior.relativeAreaStd`).
        /// The gain. Range: 0.0..=2.0.
        area_std_gain: f32 = 0.53,
        /// Relative area std offset (the floor at full confidence). Range: 0.0..=1.0.
        area_std_offset: f32 = 0.12,
        /// Synthesized section area is clamped to this multiple of the atlas
        /// mean from below. Range: 0.01..=1.0.
        area_floor_ratio: f32 = 0.2,
        /// … and to this multiple from above. Range: 1.0..=20.0.
        area_ceiling_ratio: f32 = 5.0,
        /// The mesh's uncertainty envelope scales ring offsets by
        /// sqrt(1 + clamp(rel_std, 0, this)). Range: 0.0..=4.0.
        uncertainty_expand_max: f32 = 1.5,
        /// Resampled areas handed to the mesh are floored here, cm².
        /// Range: 1e-4..=0.1.
        resample_min_area_cm2: f32 = 0.001,
    }
}

stage_config! {
    /// The articulator readout (`atlas::articulators`, the app's
    /// `ArticulatorPosterior.infer`): mean log area ratio over six
    /// glottis-to-lips regions of the normalized tract, clamped to ±1.
    pub struct ArticulatorConfig, section = "articulators" {
        /// Epilarynx region (fraction of tract length from the glottis), start.
        /// Range: 0.0..=1.0.
        epilarynx_lo: f32 = 0.0,
        /// Epilarynx region end. Range: epilarynx_lo..=1.0.
        epilarynx_hi: f32 = 0.16,
        /// Pharynx region start. Range: 0.0..=1.0.
        pharynx_lo: f32 = 0.16,
        /// Pharynx region end. Range: pharynx_lo..=1.0.
        pharynx_hi: f32 = 0.45,
        /// Tongue-root region start. Range: 0.0..=1.0.
        tongue_root_lo: f32 = 0.28,
        /// Tongue-root region end. Range: tongue_root_lo..=1.0.
        tongue_root_hi: f32 = 0.52,
        /// Tongue-dorsum region start. Range: 0.0..=1.0.
        tongue_dorsum_lo: f32 = 0.48,
        /// Tongue-dorsum region end. Range: tongue_dorsum_lo..=1.0.
        tongue_dorsum_hi: f32 = 0.72,
        /// Front-of-tongue region start. Range: 0.0..=1.0.
        tongue_front_lo: f32 = 0.68,
        /// Front-of-tongue region end. Range: tongue_front_lo..=1.0.
        tongue_front_hi: f32 = 0.88,
        /// Lip region start. Range: 0.0..=1.0.
        lips_lo: f32 = 0.88,
        /// Lip region end. Range: lips_lo..=1.0.
        lips_hi: f32 = 1.0,
        /// Jaw opening = clamp(w_lips · lips + w_front · front). Lip weight.
        /// Range: 0.0..=1.0.
        jaw_lip_weight: f32 = 0.55,
        /// Jaw opening front-of-tongue weight. Range: 0.0..=1.0.
        jaw_front_weight: f32 = 0.45,
        /// Area ratios are floored here before the log. Range: 1e-6..=1e-2.
        ratio_floor: f32 = 1.0e-4,
    }
}

stage_config! {
    /// Phase 3 validation bands (Experiment 3, `voxlab gen-inverse-fixtures`)
    /// and the atlas figures the Evidence output reports.
    pub struct ValidationConfig, section = "validation" {
        /// Held-out (formants, tract-parameter) pairs per backend in the
        /// inverse fixture set. Range: 100..=100000.
        inverse_fixture_pairs: usize = 1000,
        /// Seed of the fixture sampler (deterministic xorshift). Range: any.
        inverse_fixture_seed: u32 = 20260914,
        /// A second posterior row samples coefficients within ±this many SD
        /// of the atlas mean — the local regime the app's linear map was
        /// built for. Range: 0.05..=2.0.
        inverse_local_span_sd: f32 = 0.5,
        /// Experiment 3 sweeps the forward model up to this frequency, Hz, so
        /// it can return F4 (the atlas map's fourth input; the runtime
        /// solver stops at `tract.sweep_hi_hz`). Range: 4500.0..=8000.0.
        forward_sweep_hi_hz: f32 = 6000.0,
        /// Central-difference step, SD, for the forward-model Jacobian that
        /// is compared against the JSON's `jacobian_hz_per_sd`. Range: 0.01..=0.5.
        jacobian_step_sd: f32 = 0.1,
        /// Gate: 95th-percentile worst-of-F1..F3 round-trip error, relative.
        /// Range: 0.001..=0.5.
        p95_relative_error_max: f32 = 0.05,
        /// Gate: the same, absolute, Hz (either band satisfies the gate).
        /// Range: 1.0..=500.0.
        p95_absolute_error_hz_max: f32 = 50.0,
        /// Gate: per-frame inverse latency, ms. Range: 0.1..=50.0.
        inverse_latency_ms_max: f32 = 5.0,
        /// The atlas's held-out surface-error target, mm (the app's Evidence
        /// tab). Range: 0.5..=20.0.
        surface_error_target_mm: f32 = 4.0,
        /// The held-out median surface error the atlas reports, mm
        /// ("Median 4.781 mm · target ≤4 mm · not met"). Range: 0.0..=50.0.
        surface_error_reported_mm: f32 = 4.781,
    }
}
