//! `posterior_pca4`: Vocal Tract Lab's posterior inverse as `inverse`
//! backend #2 (Plan v3 D10). `FormantTrack` → four atlas mode
//! coefficients with confidence, abstention and the temporal filter,
//! exactly as the app's `FrameAnalyzer` drives `ReducedModelAsset.inferArea`
//! and `TemporalAtlasFilter.update` (research/vocal-tract-lab-0.10.0/):
//!
//! 1. acoustic evidence = clamp(w_f0 · f0 confidence + w_h · harmonicity +
//!    w_snr · clamp((snr + 5) / 25)) — the app's harmonicity is its own
//!    0..1 measure; here HNR is mapped onto 0..1 over
//!    `posterior.harmonicity_hnr_span_db` (stated, not hidden);
//!
//! 2. a high-f0 penalty (sparse harmonics under-sample the envelope);
//!
//! 3. evidence = clamp(0.78 · acoustic + 0.05 − penalty, 0.03, 0.82);
//!
//! 4. raw coefficients from the clamped formant→mode map, from F1..F4
//!    when the LPC stage resolved a fourth formant and F1..F3 otherwise
//!    (the map is the inverse of a 4×4 Jacobian: without F4 it is a
//!    different, much worse map — see docs/phase3-gate.md);
//!
//! 5. the filter: follow on voiced frames above the model's abstention
//!    threshold, decay otherwise; relative area std = (1 − conf) · 0.53
//!    + 0.12.
//!
//! The model is 16 kHz / 1024 / 512 in the app; the inverse consumes
//! formants only, so it is rate-agnostic. Not ported: the app's
//! `SharedTractModel.fit` refinement (its own tube solver), whose absence
//! is reported through `AbstainReason::ModelMismatch` never being raised.

use crate::atlas::reduced_model::{self, MAX_MODES, ReducedModel, TemporalAtlasFilter};
use crate::config::{PipelineParams, PosteriorConfig};
use crate::types::VoiceMetrics;

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{BasisId, F0Track, FormantTrack, TractModelId, TractParams};

pub struct PosteriorInverseStage {
    cfg: PosteriorConfig,
    model: &'static ReducedModel,
    filter: TemporalAtlasFilter,
    raw: [f32; MAX_MODES],
}

impl PosteriorInverseStage {
    /// The app's evidence score for one frame, from what VoxLabs measures.
    pub fn evidence(cfg: &PosteriorConfig, f0: &F0Track, vm: &VoiceMetrics, voiced: bool) -> f32 {
        let f0_conf = f0.confidence.clamp(0.0, 1.0);
        let harmonicity = vm
            .hnr_db
            .map(|h| (h / cfg.harmonicity_hnr_span_db).clamp(0.0, 1.0))
            .unwrap_or(0.0);
        let snr = f0
            .snr_db
            .or(vm.snr_db)
            .map(|s| ((s + cfg.evidence_snr_offset_db) / cfg.evidence_snr_span_db).clamp(0.0, 1.0))
            .unwrap_or(0.0);
        let acoustic = (cfg.evidence_f0_confidence_weight * f0_conf
            + cfg.evidence_harmonicity_weight * harmonicity
            + cfg.evidence_snr_weight * snr)
            .clamp(0.0, 1.0);
        let penalty = if voiced {
            ((f0.hz - cfg.f0_penalty_start_hz) / cfg.f0_penalty_span_hz)
                .clamp(0.0, cfg.f0_penalty_max)
        } else {
            0.0
        };
        (cfg.evidence_gain * acoustic + cfg.evidence_offset - penalty)
            .clamp(cfg.evidence_min, cfg.evidence_max)
    }
}

impl Stage for PosteriorInverseStage {
    type In<'a> = (&'a F0Track, &'a FormantTrack, &'a VoiceMetrics);
    type Out = TractParams;
    const NAME: &'static str = "inverse";
    const BACKEND: &'static str = "posterior_pca4";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, _fmt: &StreamFormat) -> Result<Self, StageError> {
        let model = reduced_model::shared().map_err(StageError::Init)?;
        if model.n_modes() > MAX_MODES {
            return Err(StageError::Init(format!(
                "reduced model has {} modes; TractParams carries {MAX_MODES}",
                model.n_modes()
            )));
        }
        Ok(Self {
            cfg: params.posterior,
            model,
            filter: TemporalAtlasFilter::new(model, params.posterior),
            raw: [0.0; MAX_MODES],
        })
    }

    fn process(
        &mut self,
        (f0, ft, vm): (&F0Track, &FormantTrack, &VoiceMetrics),
        out: &mut TractParams,
    ) -> Result<(), StageError> {
        let voiced = f0.voiced && !f0.rejected;
        let evidence = Self::evidence(&self.cfg, f0, vm, voiced);
        let formants = [
            ft.formants[0].frequency,
            ft.formants[1].frequency,
            ft.formants[2].frequency,
        ];
        if formants.iter().any(|f| !f.is_finite()) {
            return Err(StageError::Process(format!(
                "non-finite formants {formants:?}"
            )));
        }
        self.model.infer_coefficients(&formants, &mut self.raw);
        let est = self.filter.update(&self.raw, evidence, voiced);
        out.model = TractModelId::MriPca4;
        out.basis = BasisId::MriAtlasMean;
        out.q1 = 0.0;
        out.q2 = 0.0;
        out.modes = est.coefficients;
        out.n_modes = self.model.n_modes();
        out.confidence = Some(est.confidence);
        out.abstained = est.abstained;
        out.reason = est.reason;
        out.valid = !est.abstained;
        out.uncertainty =
            Some((1.0 - est.confidence) * self.cfg.area_std_gain + self.cfg.area_std_offset);
        out.vtl_est_cm = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::types::AbstainReason;
    use crate::types::Formant;

    fn fmt() -> StreamFormat {
        StreamFormat {
            sample_rate_hz: 48_000.0,
            frame_samples: 2048,
            hop: 1024,
        }
    }

    fn track(f: [f32; 3]) -> FormantTrack {
        FormantTrack {
            formants: f.map(|frequency| Formant {
                frequency,
                bandwidth: 80.0,
            }),
            measured_f0: 120.0,
            confidence: 1.0,
            fresh: true,
            f4: None,
        }
    }

    fn strong() -> (F0Track, VoiceMetrics) {
        (
            F0Track {
                hz: 120.0,
                confidence: 0.95,
                voiced: true,
                snr_db: Some(30.0),
                rejected: false,
            },
            VoiceMetrics {
                hnr_db: Some(25.0),
                ..Default::default()
            },
        )
    }

    /// Contract (`FrameAnalyzer`): the evidence formula, hand-computed.
    #[test]
    fn evidence_is_the_apps_weighted_clamped_score() {
        let cfg = PosteriorConfig::DEFAULT;
        let (f0, vm) = strong();
        // f0 conf 0.95, harmonicity 1.0 (25/20 clamped), snr (30+5)/25 clamped → 1.
        let acoustic = (0.42 * 0.95f32 + 0.30 + 0.28).min(1.0);
        let want = (0.78 * acoustic + 0.05f32).clamp(0.03, 0.82);
        assert!((PosteriorInverseStage::evidence(&cfg, &f0, &vm, true) - want).abs() < 1e-6);
        // High f0: penalized by (720 − 420) / 600 = 0.5, capped at 0.28.
        let high = F0Track { hz: 720.0, ..f0 };
        let want_high = (0.78 * acoustic + 0.05 - 0.28f32).clamp(0.03, 0.82);
        assert!((PosteriorInverseStage::evidence(&cfg, &high, &vm, true) - want_high).abs() < 1e-6);
        // Nothing measured: only the offset survives (above the 0.03 floor).
        assert!(
            (PosteriorInverseStage::evidence(
                &cfg,
                &F0Track::default(),
                &VoiceMetrics::default(),
                false
            ) - 0.05)
                .abs()
                < 1e-6
        );
    }

    /// Contract: the stage's coefficients after one strong frame are the
    /// clamped map's raw estimate scaled by the filter's follow rate, the
    /// confidence is 0.35 of the evidence, and the uncertainty readout is
    /// (1 − conf) · 0.53 + 0.12. Unvoiced frames abstain and decay.
    #[test]
    fn one_frame_follows_the_map_and_unvoiced_frames_abstain() {
        let mut s = PosteriorInverseStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let (f0, vm) = strong();
        let ft = track([816.0, 2022.0, 3174.0]);
        let mut out = TractParams::default();
        s.process((&f0, &ft, &vm), &mut out).unwrap();
        let m = reduced_model::shared().unwrap();
        let mut raw = [0.0; MAX_MODES];
        m.infer_coefficients(&[816.0, 2022.0, 3174.0], &mut raw);
        let e = PosteriorInverseStage::evidence(&PosteriorConfig::DEFAULT, &f0, &vm, true);
        let alpha = (0.38 * e + 0.12f32).clamp(0.12, 0.46);
        assert_eq!(out.model, TractModelId::MriPca4);
        assert_eq!(out.basis, BasisId::MriAtlasMean);
        assert_eq!(out.n_modes, 4);
        assert!(out.valid && !out.abstained);
        for (m, r) in out.modes.iter().zip(&raw) {
            assert!((m - (r * alpha).clamp(-2.0, 2.0)).abs() < 1e-6, "{out:?}");
        }
        let conf = e * 0.35;
        assert!((out.confidence.unwrap() - conf).abs() < 1e-6);
        assert!((out.uncertainty.unwrap() - ((1.0 - conf) * 0.53 + 0.12)).abs() < 1e-6);
        let held = out.modes;
        s.process(
            (&F0Track::default(), &ft, &VoiceMetrics::default()),
            &mut out,
        )
        .unwrap();
        assert!(out.abstained && !out.valid);
        assert_eq!(out.reason, AbstainReason::Unvoiced);
        for (m, h) in out.modes.iter().zip(&held) {
            assert!((m - h * 0.9).abs() < 1e-6);
        }
        // Rejected by the room gates counts as unvoiced.
        let rejected = F0Track {
            rejected: true,
            ..strong().0
        };
        s.process((&rejected, &ft, &vm), &mut out).unwrap();
        assert_eq!(out.reason, AbstainReason::Unvoiced);
    }

    #[test]
    fn weak_evidence_on_a_voiced_frame_is_insufficient_evidence() {
        let mut s = PosteriorInverseStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let weak = F0Track {
            hz: 120.0,
            confidence: 0.1,
            voiced: true,
            snr_db: Some(-5.0),
            rejected: false,
        };
        let mut out = TractParams::default();
        s.process(
            (
                &weak,
                &track([700.0, 1200.0, 2500.0]),
                &VoiceMetrics::default(),
            ),
            &mut out,
        )
        .unwrap();
        assert_eq!(out.reason, AbstainReason::InsufficientEvidence);
        assert!(out.abstained);
    }
}
