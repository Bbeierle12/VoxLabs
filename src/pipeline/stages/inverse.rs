//! `grid_story`: the 41×41 (q1, q2) grid inverse wrapped as a stage. The
//! logic is the hero card's `update_tract_model`, moved off the UI thread:
//! grade the formants by the f0 they were measured at, accumulate the
//! tract-length estimate from identity-grade frames, pick the basis, scale
//! the pair into the basis's length reference, invert.
//!
//! The grids are built once per process and shared with the UI, so the
//! stage's `init` and the hero card never build the same 1681 solves twice.

use std::sync::OnceLock;

use crate::config::{InverseConfig, PipelineParams, TractConfig};
use crate::math::{self, FormantGrade};
use crate::tract::{self, ADULT_FEMALE, ADULT_MALE, TractBasis, TractGrid};

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{BasisId, F0Track, FormantTrack, TractParams};
use super::require_default;

static GRID_MALE: OnceLock<TractGrid> = OnceLock::new();
static GRID_FEMALE: OnceLock<TractGrid> = OnceLock::new();

fn cell(basis: &'static TractBasis) -> &'static OnceLock<TractGrid> {
    if std::ptr::eq(basis, &ADULT_FEMALE) {
        &GRID_FEMALE
    } else {
        &GRID_MALE
    }
}

/// The shared grid for a basis, built on this thread if nobody has yet
/// (blocking while another thread builds it).
pub fn shared_grid(basis: &'static TractBasis, grid_n: usize) -> &'static TractGrid {
    cell(basis).get_or_init(|| TractGrid::build(basis, grid_n, grid_n))
}

/// The shared grid if it is already built.
pub fn shared_grid_if_built(basis: &'static TractBasis) -> Option<&'static TractGrid> {
    cell(basis).get()
}

pub struct GridInverseStage {
    cfg: InverseConfig,
    grid_n: usize,
    vtl_est_cm: Option<f32>,
}

impl Stage for GridInverseStage {
    type In<'a> = (&'a F0Track, &'a FormantTrack);
    type Out = TractParams;
    const NAME: &'static str = "inverse";
    const BACKEND: &'static str = "grid_story";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, _fmt: &StreamFormat) -> Result<Self, StageError> {
        // `grid_n` is passed through explicitly; everything else in
        // [tract] is read by the kernels from their compiled defaults.
        let mut given = params.tract;
        given.grid_n = TractConfig::DEFAULT.grid_n;
        require_default("tract", Self::NAME, &given, &TractConfig::DEFAULT)?;
        if params.tract.grid_n < params.tract.grid_min_n {
            return Err(StageError::Init(format!(
                "tract.grid_n {} is below grid_min_n {}",
                params.tract.grid_n, params.tract.grid_min_n
            )));
        }
        // All allocation here: both grids, shared with the UI.
        shared_grid(&ADULT_MALE, params.tract.grid_n);
        shared_grid(&ADULT_FEMALE, params.tract.grid_n);
        Ok(Self {
            cfg: params.inverse,
            grid_n: params.tract.grid_n,
            vtl_est_cm: None,
        })
    }

    fn process(
        &mut self,
        (f0, ft): (&F0Track, &FormantTrack),
        out: &mut TractParams,
    ) -> Result<(), StageError> {
        out.valid = false;
        out.uncertainty = None;
        let grade = math::formant_grade(&ft.formants, ft.measured_f0);
        if !f0.voiced || grade == FormantGrade::Reject {
            out.vtl_est_cm = self.vtl_est_cm;
            return Ok(());
        }
        let [f1_meas, f2_meas, f3_meas] = ft.formants;
        if grade == FormantGrade::Identity
            && let Some(l) = tract::vtl_from_formants(f2_meas.frequency, f3_meas.frequency)
        {
            self.vtl_est_cm = Some(match self.vtl_est_cm {
                Some(prev) => prev + self.cfg.vtl_ema_alpha * (l - prev),
                None => l,
            });
        }
        let basis = tract::basis_for_vtl(self.vtl_est_cm);
        let grid = shared_grid(basis, self.grid_n);
        let scale = self.vtl_est_cm.map_or(1.0, |l| l / basis.vtl_cm);
        let (f1, f2) = (f1_meas.frequency * scale, f2_meas.frequency * scale);
        out.basis = BasisId::of(basis);
        out.vtl_est_cm = self.vtl_est_cm;
        if let Some((q1, q2)) = grid.invert(f1, f2) {
            out.q1 = q1;
            out.q2 = q2;
            out.valid = true;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Formant;

    fn params() -> PipelineParams {
        let mut p = PipelineParams::DEFAULT;
        // Coarser than runtime, as tract.rs's own tests do, to keep this fast.
        p.tract.grid_n = 21;
        p
    }

    fn fmt() -> StreamFormat {
        StreamFormat {
            sample_rate_hz: 48_000.0,
            frame_samples: 2048,
            hop: 1024,
        }
    }

    fn track(f: [f32; 3], measured_f0: f32) -> FormantTrack {
        FormantTrack {
            formants: f.map(|frequency| Formant {
                frequency,
                bandwidth: 80.0,
            }),
            measured_f0,
            confidence: 1.0,
            fresh: true,
        }
    }

    /// Contract: the published vowels round-trip through the stage as they
    /// do through `TractGrid::invert` directly (tract.rs's own test).
    #[test]
    fn published_vowels_round_trip_through_the_stage() {
        let mut s = GridInverseStage::init(&params(), &fmt()).unwrap();
        let f0 = F0Track {
            hz: 120.0,
            confidence: 0.9,
            voiced: true,
            ..Default::default()
        };
        for (name, q1, q2) in [("i", -5.10, 0.88), ("ɑ", 3.86, 1.35), ("u", -3.48, -1.70)] {
            let areas = tract::area_function(&ADULT_MALE, q1, q2);
            let [r1, r2, r3] = tract::resonances(&areas, ADULT_MALE.vtl_cm).unwrap();
            // measured_f0 above the identity gate keeps the VTL estimate off,
            // so no length rescaling enters and the direct call is the oracle.
            // It must also keep F1 and F2 out of the ±15 % harmonic-suspect
            // bands, which depends on the vowel: pick the first display-grade
            // f0 the grader accepts.
            let measured_f0 = [210.0f32, 230.0, 260.0, 290.0, 320.0, 345.0]
                .into_iter()
                .find(|&f| {
                    math::formant_grade(&track([r1, r2, r3], f).formants, f)
                        == FormantGrade::DisplayOnly
                })
                .expect("a display-grade f0 clear of the suspect bands");
            let ft = track([r1, r2, r3], measured_f0);
            let mut out = TractParams::default();
            s.process((&f0, &ft), &mut out).unwrap();
            let direct = shared_grid(&ADULT_MALE, 21).invert(r1, r2).unwrap();
            assert!(out.valid, "vowel {name} did not invert");
            assert_eq!((out.q1, out.q2), direct, "vowel {name}");
            assert!(
                (out.q1 - q1).abs() < 0.6 && (out.q2 - q2).abs() < 0.5,
                "vowel {name}"
            );
            assert_eq!(out.uncertainty, None);
        }
    }

    #[test]
    fn unvoiced_or_rejected_frames_are_not_valid() {
        let mut s = GridInverseStage::init(&params(), &fmt()).unwrap();
        let ft = track([700.0, 1200.0, 2500.0], 120.0);
        let mut out = TractParams::default();
        s.process((&F0Track::default(), &ft), &mut out).unwrap();
        assert!(!out.valid);
        // f0 above the display gate: rejected by grade.
        let voiced = F0Track {
            hz: 400.0,
            confidence: 0.9,
            voiced: true,
            ..Default::default()
        };
        s.process((&voiced, &track([700.0, 1200.0, 2500.0], 400.0)), &mut out)
            .unwrap();
        assert!(!out.valid);
    }

    #[test]
    fn out_of_space_formants_refuse_to_invert() {
        let mut s = GridInverseStage::init(&params(), &fmt()).unwrap();
        let voiced = F0Track {
            hz: 120.0,
            confidence: 0.9,
            voiced: true,
            ..Default::default()
        };
        let mut out = TractParams::default();
        s.process((&voiced, &track([3000.0, 600.0, 2500.0], 250.0)), &mut out)
            .unwrap();
        assert!(!out.valid);
    }
}
