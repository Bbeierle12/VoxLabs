//! `lumen_v2`: the `mesh` stage. `(TractParams, AreaFunction)` →
//! `TractGeometry`: the area function resampled onto the lumen's section
//! positions (`TractMeshCpu.resampleArea`), the mesh morphed
//! (`TractLumenAsset.morph`), vertex normals, and the uncertainty
//! envelope from the posterior's relative area std
//! (`TractMeshCpu.expandUncertainty`). A Story area function meshes too
//! (its equal-length sections map to positions 0..1); with no uncertainty
//! from the grid inverse the envelope then equals the mesh.

use crate::atlas::lumen::{self, Lumen, resample_area_into, vertex_normals_into};
use crate::config::{PipelineParams, PosteriorConfig};

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{AreaFunction, TractGeometry, TractModelId, TractParams};

pub struct LumenMeshStage {
    cfg: PosteriorConfig,
    lumen: &'static Lumen,
    /// Source positions of the incoming area function, per model.
    story_positions: Vec<f32>,
    atlas_positions: Vec<f32>,
    resampled: Vec<f32>,
}

impl Stage for LumenMeshStage {
    type In<'a> = (&'a TractParams, &'a AreaFunction);
    type Out = TractGeometry;
    const NAME: &'static str = "mesh";
    const BACKEND: &'static str = "lumen_v2";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, _fmt: &StreamFormat) -> Result<Self, StageError> {
        let l = lumen::shared().map_err(StageError::Init)?;
        let model = crate::atlas::reduced_model::shared().map_err(StageError::Init)?;
        let n = crate::tract::N_SECTIONS;
        Ok(Self {
            cfg: params.posterior,
            lumen: l,
            story_positions: (0..n).map(|i| i as f32 / (n - 1) as f32).collect(),
            atlas_positions: model.section_position.clone(),
            resampled: vec![0.0; l.sections],
        })
    }

    fn process(
        &mut self,
        (p, af): (&TractParams, &AreaFunction),
        out: &mut TractGeometry,
    ) -> Result<(), StageError> {
        let positions: &[f32] = match af.model {
            TractModelId::StoryTwoMode => &self.story_positions,
            TractModelId::MriPca4 => &self.atlas_positions,
        };
        if af.sections != positions.len() {
            return Err(StageError::Process(format!(
                "{} area function has {} sections, expected {}",
                af.model.name(),
                af.sections,
                positions.len()
            )));
        }
        resample_area_into(
            positions,
            &af.areas_cm2[..af.sections],
            &self.lumen.section_position,
            self.cfg.resample_min_area_cm2,
            &mut self.resampled,
        )
        .map_err(StageError::Process)?;
        self.lumen
            .morph_into(&self.resampled, &mut out.vertices_mm)
            .map_err(StageError::Process)?;
        vertex_normals_into(&out.vertices_mm, &out.triangles, &mut out.normals)
            .map_err(StageError::Process)?;
        let rel_std = p.uncertainty.unwrap_or(0.0);
        self.lumen
            .expand_uncertainty_into(
                &out.vertices_mm,
                rel_std,
                self.cfg.uncertainty_expand_max,
                &mut out.uncertainty_vertices_mm,
            )
            .map_err(StageError::Process)?;
        out.relative_area_std = rel_std;
        out.live = af.live;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::types::BasisId;

    fn fmt() -> StreamFormat {
        StreamFormat {
            sample_rate_hz: 48_000.0,
            frame_samples: 2048,
            hop: 1024,
        }
    }

    /// Contract: with the atlas mean area function the stage's vertices
    /// are the lumen morphed to the reference-position resample of the
    /// mean; with rel σ the envelope scales offsets by sqrt(1 + σ).
    #[test]
    fn mean_shape_morphs_to_the_resampled_mean_and_uncertainty_expands() {
        let mut s = LumenMeshStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let model = crate::atlas::reduced_model::shared().unwrap();
        let l = lumen::shared().unwrap();
        let mut af = AreaFunction::neutral(BasisId::AdultMale);
        af.model = TractModelId::MriPca4;
        af.sections = 32;
        af.live = true;
        for k in 0..32 {
            af.areas_cm2[k] = model.area_mean_cm2[k];
        }
        let p = TractParams {
            model: TractModelId::MriPca4,
            uncertainty: Some(0.5),
            valid: true,
            ..Default::default()
        };
        let mut out = TractGeometry::preallocated().unwrap();
        s.process((&p, &af), &mut out).unwrap();
        let mut want_area = vec![0.0f32; 32];
        resample_area_into(
            &model.section_position,
            &model.area_mean_cm2,
            &l.section_position,
            0.001,
            &mut want_area,
        )
        .unwrap();
        let mut want = vec![0.0f32; l.vertex_count * 3];
        l.morph_into(&want_area, &mut want).unwrap();
        assert_eq!(out.vertices_mm, want);
        assert!(out.live);
        assert_eq!(out.relative_area_std, 0.5);
        let idx = (4 * l.angular + 2) * 3;
        let c = l.centerline_mm[12];
        let want_u = c + (out.vertices_mm[idx] - c) * 1.5f32.sqrt();
        assert!((out.uncertainty_vertices_mm[idx] - want_u).abs() < 1e-4);
        assert!(out.vertices_mm.iter().all(|v| v.is_finite()));
        assert!(out.normals.iter().all(|v| v.is_finite()));
        // A Story shape meshes too, without an envelope.
        let story = AreaFunction::from_coefficients(BasisId::AdultMale, 3.86, 1.35, true);
        s.process((&TractParams::default(), &story), &mut out)
            .unwrap();
        assert_eq!(out.uncertainty_vertices_mm, out.vertices_mm);
        assert!(out.live);
    }
}
