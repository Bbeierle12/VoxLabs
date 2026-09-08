//! `story_two_mode`: the Story two-mode area function as a stage.
//! Coefficients in, diameters and areas out — `tract::diameters` and
//! `tract::area_function`, wrapped. An invalid (gated-out) frame holds the
//! last valid shape and reports it as not live, the way the hero card does.

use crate::config::{PipelineParams, TractConfig};

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{AreaFunction, BasisId, TractParams};
use super::require_default;

pub struct StoryTractStage {
    /// Last valid coefficients and basis, for the HELD shape.
    held: Option<(BasisId, f32, f32)>,
}

impl Stage for StoryTractStage {
    type In<'a> = &'a TractParams;
    type Out = AreaFunction;
    const NAME: &'static str = "tract";
    const BACKEND: &'static str = "story_two_mode";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, _fmt: &StreamFormat) -> Result<Self, StageError> {
        let mut given = params.tract;
        given.grid_n = TractConfig::DEFAULT.grid_n;
        require_default("tract", Self::NAME, &given, &TractConfig::DEFAULT)?;
        Ok(Self { held: None })
    }

    fn process(&mut self, p: &TractParams, out: &mut AreaFunction) -> Result<(), StageError> {
        if p.valid {
            if !(p.q1.is_finite() && p.q2.is_finite()) {
                return Err(StageError::Process(format!(
                    "non-finite coefficients ({}, {})",
                    p.q1, p.q2
                )));
            }
            self.held = Some((p.basis, p.q1, p.q2));
        }
        *out = match self.held {
            Some((basis, q1, q2)) => AreaFunction::from_coefficients(basis, q1, q2, p.valid),
            None => AreaFunction::neutral(p.basis),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tract::{self, ADULT_MALE, N_SECTIONS};

    fn fmt() -> StreamFormat {
        StreamFormat {
            sample_rate_hz: 48_000.0,
            frame_samples: 2048,
            hop: 1024,
        }
    }

    /// Contract: the stage's diameters and areas are `tract::diameters` and
    /// `tract::area_function` of the same coefficients.
    #[test]
    fn stage_matches_direct_area_function() {
        let mut s = StoryTractStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let p = TractParams {
            q1: -5.10,
            q2: 0.88,
            uncertainty: None,
            valid: true,
            basis: BasisId::AdultMale,
            vtl_est_cm: None,
        };
        let mut out = AreaFunction::neutral(BasisId::AdultMale);
        s.process(&p, &mut out).unwrap();
        assert_eq!(out.diameters_cm, tract::diameters(&ADULT_MALE, -5.10, 0.88));
        assert_eq!(
            out.areas_cm2,
            tract::area_function(&ADULT_MALE, -5.10, 0.88)
        );
        assert!(out.live);
        assert_eq!(out.sections, N_SECTIONS);
        assert!(out.areas_cm2.iter().all(|&a| a.is_finite() && a > 0.0));
    }

    /// The tube moves: two vowels give two different shapes, and an invalid
    /// frame holds the last one, not live.
    #[test]
    fn different_vowels_move_the_tube_and_invalid_frames_hold() {
        let mut s = StoryTractStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let mut i = TractParams {
            q1: -5.10,
            q2: 0.88,
            uncertainty: None,
            valid: true,
            basis: BasisId::AdultMale,
            vtl_est_cm: None,
        };
        let mut out = AreaFunction::neutral(BasisId::AdultMale);
        s.process(&i, &mut out).unwrap();
        let shape_i = out.diameters_cm;
        i.q1 = 3.86;
        i.q2 = 1.35;
        s.process(&i, &mut out).unwrap();
        let shape_a = out.diameters_cm;
        assert_ne!(shape_i, shape_a);
        i.valid = false;
        s.process(&i, &mut out).unwrap();
        assert_eq!(out.diameters_cm, shape_a, "held shape");
        assert!(!out.live);
    }

    #[test]
    fn before_any_valid_frame_the_shape_is_neutral_and_not_live() {
        let mut s = StoryTractStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let mut out = AreaFunction::from_coefficients(BasisId::AdultMale, 1.0, 1.0, true);
        s.process(&TractParams::default(), &mut out).unwrap();
        assert_eq!(out, AreaFunction::neutral(BasisId::AdultMale));
    }
}
