//! `goertzel`: the amplitude of each harmonic k·f0 (Goertzel at the
//! harmonic frequencies, `math::harmonic_amplitudes`) on voiced frames,
//! zeros otherwise — as the analyzers compute it.

use crate::config::{HarmonicsConfig, PipelineParams};
use crate::math;

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{AudioFrame, F0Track, HarmonicSeries};
use super::require_default;

pub struct HarmonicsStage;

impl Stage for HarmonicsStage {
    type In<'a> = (&'a AudioFrame, &'a F0Track);
    type Out = HarmonicSeries;
    const NAME: &'static str = "harmonics";
    const BACKEND: &'static str = "goertzel";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, _fmt: &StreamFormat) -> Result<Self, StageError> {
        require_default(
            "harmonics",
            Self::NAME,
            &params.harmonics,
            &HarmonicsConfig::DEFAULT,
        )?;
        Ok(Self)
    }

    fn process(
        &mut self,
        (frame, f0): (&AudioFrame, &F0Track),
        out: &mut HarmonicSeries,
    ) -> Result<(), StageError> {
        if f0.voiced {
            out.amplitudes = math::harmonic_amplitudes(&frame.samples, frame.sample_rate, f0.hz);
            out.f0_hz = f0.hz;
            out.voiced = true;
        } else {
            *out = HarmonicSeries::default();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::ANALYSIS_FRAME;
    use crate::types::MAX_PARTIALS;
    use std::f32::consts::TAU;

    const SR: f32 = 48_000.0;

    fn fmt() -> StreamFormat {
        StreamFormat {
            sample_rate_hz: SR,
            frame_samples: ANALYSIS_FRAME,
            hop: ANALYSIS_FRAME / 2,
        }
    }

    #[test]
    fn stage_matches_direct_call_and_is_silent_when_unvoiced() {
        let mut s = HarmonicsStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let frame = AudioFrame {
            samples: (0..ANALYSIS_FRAME)
                .map(|i| {
                    let t = i as f32 / SR;
                    (1..=6)
                        .map(|k| (0.5f32).powi(k - 1) * (TAU * 150.0 * k as f32 * t).sin())
                        .sum::<f32>()
                })
                .collect(),
            sample_rate: SR,
            frame_index: 0,
        };
        let voiced = F0Track {
            hz: 150.0,
            confidence: 0.95,
            voiced: true,
            ..Default::default()
        };
        let mut out = HarmonicSeries::default();
        s.process((&frame, &voiced), &mut out).unwrap();
        assert_eq!(
            out.amplitudes,
            math::harmonic_amplitudes(&frame.samples, SR, 150.0)
        );
        assert!(out.voiced && out.f0_hz == 150.0);
        assert!(out.amplitudes[0] > out.amplitudes[1], "H1 dominates");
        s.process((&frame, &F0Track::default()), &mut out).unwrap();
        assert_eq!(out.amplitudes, [0.0; MAX_PARTIALS]);
        assert!(!out.voiced);
    }
}
