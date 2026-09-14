//! `classic`: the per-frame voice-quality metrics — HNR, H1–H2, jitter,
//! shimmer, CPP, spectral centroid — on voiced frames, plus the SNR and
//! noisy-rejection verdicts the `voicing` stage already decided. Vibrato
//! and steadiness are the `contour` stage's (they need the f0 history).
//! Every kernel is the analyzers' own.

use crate::config::{
    CentroidConfig, CppConfig, HnrConfig, PerturbationConfig, PipelineParams, TimbreConfig,
};
use crate::math;
use crate::types::VoiceMetrics;

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{AudioFrame, F0Track, HarmonicSeries};
use super::require_default;

pub struct MetricsStage;

impl Stage for MetricsStage {
    type In<'a> = (&'a AudioFrame, &'a F0Track, &'a HarmonicSeries);
    type Out = VoiceMetrics;
    const NAME: &'static str = "metrics";
    const BACKEND: &'static str = "classic";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, _fmt: &StreamFormat) -> Result<Self, StageError> {
        require_default("hnr", Self::NAME, &params.hnr, &HnrConfig::DEFAULT)?;
        require_default(
            "perturbation",
            Self::NAME,
            &params.perturbation,
            &PerturbationConfig::DEFAULT,
        )?;
        require_default("cpp", Self::NAME, &params.cpp, &CppConfig::DEFAULT)?;
        require_default(
            "centroid",
            Self::NAME,
            &params.centroid,
            &CentroidConfig::DEFAULT,
        )?;
        require_default("timbre", Self::NAME, &params.timbre, &TimbreConfig::DEFAULT)?;
        Ok(Self)
    }

    fn process(
        &mut self,
        (frame, f0, harmonics): (&AudioFrame, &F0Track, &HarmonicSeries),
        out: &mut VoiceMetrics,
    ) -> Result<(), StageError> {
        let samples = &frame.samples;
        let sr = frame.sample_rate;
        let perturbation = if f0.voiced {
            math::cycle_perturbation(samples, sr, f0.hz)
        } else {
            None
        };
        *out = VoiceMetrics {
            hnr_db: if f0.voiced {
                math::hnr_db(samples, sr, f0.hz)
            } else {
                None
            },
            h1_h2_db: if f0.voiced {
                math::h1_h2_db(&harmonics.amplitudes)
            } else {
                None
            },
            vibrato: None,
            steadiness_cents: None,
            jitter_pct: perturbation.map(|p| p.jitter_pct),
            shimmer_db: perturbation.map(|p| p.shimmer_db),
            cpp_db: if f0.voiced {
                math::cpp_db(samples, sr)
            } else {
                None
            },
            centroid_hz: if f0.voiced {
                math::spectral_centroid(samples, sr)
            } else {
                None
            },
            snr_db: f0.snr_db,
            voiced_but_noisy: f0.rejected,
        };
        Ok(())
    }
}
