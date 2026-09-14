//! `mri_pca4`: the Vocal Tract Lab reduced model as `tract` backend #2.
//! Four atlas coefficients → the 32-section area function
//! (`ReducedModelAsset.areaFromCoefficients`: exp(ln mean + Σ c·mode),
//! clamped to [0.2, 5] × mean). The sections sit at the model's
//! normalized positions (uniform 0..1); the tract length is the lumen
//! centerline's arc length (`tract_lumen_v2.bin`, the same frozen mean),
//! so `section_len_cm` is that length over 32. There is no HELD state:
//! the posterior's filter already decays toward the mean on abstained
//! frames, and `live` reports whether the frame was abstained.

use crate::atlas::{lumen, reduced_model};
use crate::config::{PipelineParams, PosteriorConfig, consts};
use crate::tract::N_SECTIONS;

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{AreaFunction, BasisId, TractModelId, TractParams};

pub struct MriTractStage {
    cfg: PosteriorConfig,
    model: &'static reduced_model::ReducedModel,
    vtl_cm: f32,
    areas: [f32; N_SECTIONS],
}

impl Stage for MriTractStage {
    type In<'a> = &'a TractParams;
    type Out = AreaFunction;
    const NAME: &'static str = "tract";
    const BACKEND: &'static str = "mri_pca4";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, _fmt: &StreamFormat) -> Result<Self, StageError> {
        let model = reduced_model::shared().map_err(StageError::Init)?;
        let mesh = lumen::shared().map_err(StageError::Init)?;
        if model.n_sections() > N_SECTIONS {
            return Err(StageError::Init(format!(
                "reduced model has {} sections; AreaFunction holds {N_SECTIONS}",
                model.n_sections()
            )));
        }
        Ok(Self {
            cfg: params.posterior,
            model,
            vtl_cm: mesh.centerline_length_mm() / consts::MM_PER_CM,
            areas: [0.0; N_SECTIONS],
        })
    }

    fn process(&mut self, p: &TractParams, out: &mut AreaFunction) -> Result<(), StageError> {
        if p.model != TractModelId::MriPca4 {
            return Err(StageError::Process(format!(
                "mri_pca4 cannot shape {} parameters (wire `inverse` to posterior_pca4)",
                p.model.name()
            )));
        }
        let n = self.model.n_sections();
        self.model
            .area_from_coefficients(
                &p.modes[..p.n_modes.max(self.model.n_modes())],
                &self.cfg,
                &mut self.areas[..n],
            )
            .map_err(StageError::Process)?;
        out.model = TractModelId::MriPca4;
        out.basis = BasisId::MriAtlasMean;
        out.sections = n;
        out.vtl_cm = self.vtl_cm;
        out.section_len_cm = self.vtl_cm / n as f32;
        out.live = p.valid && !p.abstained;
        for k in 0..N_SECTIONS {
            let a = if k < n { self.areas[k] } else { 0.0 };
            out.areas_cm2[k] = a;
            out.diameters_cm[k] = consts::TWO * (a / std::f32::consts::PI).sqrt();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt() -> StreamFormat {
        StreamFormat {
            sample_rate_hz: 48_000.0,
            frame_samples: 2048,
            hop: 1024,
        }
    }

    /// Contract: the stage's areas are `area_from_coefficients` of the
    /// same coefficients, 32 sections, the rest 0, diameters from the areas.
    #[test]
    fn stage_matches_the_model_synthesis() {
        let mut s = MriTractStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let p = TractParams {
            model: TractModelId::MriPca4,
            modes: [0.7, -0.4, 1.1, 0.2],
            n_modes: 4,
            valid: true,
            basis: BasisId::MriAtlasMean,
            ..Default::default()
        };
        let mut out = AreaFunction::neutral(BasisId::AdultMale);
        s.process(&p, &mut out).unwrap();
        let m = reduced_model::shared().unwrap();
        let mut want = [0.0f32; 32];
        m.area_from_coefficients(&p.modes, &PosteriorConfig::DEFAULT, &mut want)
            .unwrap();
        assert_eq!(out.sections, 32);
        assert_eq!(&out.areas_cm2[..32], &want);
        assert!(out.areas_cm2[32..].iter().all(|&a| a == 0.0));
        assert!((out.diameters_cm[5] - 2.0 * (want[5] / std::f32::consts::PI).sqrt()).abs() < 1e-6);
        assert_eq!(out.model, TractModelId::MriPca4);
        assert!(out.live);
        assert!(out.vtl_cm > 10.0 && out.vtl_cm < 25.0, "{}", out.vtl_cm);
        assert!((out.section_len_cm * 32.0 - out.vtl_cm).abs() < 1e-4);
        let abstained = TractParams {
            abstained: true,
            valid: false,
            ..p
        };
        s.process(&abstained, &mut out).unwrap();
        assert!(!out.live);
        assert!(
            s.process(&TractParams::default(), &mut out).is_err(),
            "Story params refused"
        );
    }
}
