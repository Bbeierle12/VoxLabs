//! `lpc_levinson`: decimate → pre-emphasis + Hamming → autocorrelation →
//! Levinson-Durbin → Aberth root-solve → formants, on voiced frames, held
//! across unvoiced ones — the exact sequence `frame::FrameAnalyzer` and the
//! desktop engine run, wrapped.

use crate::config::{FormantConfig, LpcConfig, PipelineParams};
use crate::math;

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{AudioFrame, F0Track, FormantTrack};
use super::require_default;

pub struct LpcStage {
    lpc: LpcConfig,
    held: FormantTrack,
}

impl LpcStage {
    /// The analyzers' decimation factor and LPC order for a sample rate.
    fn plan(&self, sample_rate: f32) -> (usize, usize, f32) {
        let m = ((sample_rate / self.lpc.decimation_target_hz).round() as usize).max(1);
        let fs_dec = sample_rate / m as f32;
        let order = (self.lpc.order_base + (fs_dec / self.lpc.order_hz_per_pole) as usize)
            .clamp(self.lpc.order_min, self.lpc.order_max);
        (m, order, fs_dec)
    }
}

impl Stage for LpcStage {
    type In<'a> = (&'a AudioFrame, &'a F0Track);
    type Out = FormantTrack;
    const NAME: &'static str = "lpc";
    const BACKEND: &'static str = "lpc_levinson";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, _fmt: &StreamFormat) -> Result<Self, StageError> {
        require_default("lpc", Self::NAME, &params.lpc, &LpcConfig::DEFAULT)?;
        require_default(
            "formants",
            Self::NAME,
            &params.formants,
            &FormantConfig::DEFAULT,
        )?;
        Ok(Self {
            lpc: params.lpc,
            held: FormantTrack::held_default(&params.formants),
        })
    }

    fn process(
        &mut self,
        (frame, f0): (&AudioFrame, &F0Track),
        out: &mut FormantTrack,
    ) -> Result<(), StageError> {
        self.held.fresh = false;
        if f0.voiced {
            let (m, order, fs_dec) = self.plan(frame.sample_rate);
            let decimated = math::decimate(&frame.samples, m);
            let lpc = math::lpc_coefficients(&decimated, order, self.lpc.preemphasis);
            let measured = math::formants_from_lpc(&lpc, fs_dec);
            if measured[0].frequency > 0.0 {
                self.held = FormantTrack {
                    formants: measured,
                    measured_f0: f0.hz,
                    confidence: 1.0,
                    fresh: true,
                };
            }
        }
        *out = self.held;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{ANALYSIS_FRAME, FrameAnalyzer};
    use std::f32::consts::TAU;

    const SR: f32 = 48_000.0;

    fn fmt() -> StreamFormat {
        StreamFormat {
            sample_rate_hz: SR,
            frame_samples: ANALYSIS_FRAME,
            hop: ANALYSIS_FRAME / 2,
        }
    }

    /// The `frame.rs` test's harmonic-rich vowel.
    fn synth(f0: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| {
                let t = i as f32 / SR;
                (1i32..=10)
                    .map(|k| (0.6f32).powi(k - 1) * (TAU * f0 * k as f32 * t).sin())
                    .sum::<f32>()
                    * 0.2
            })
            .collect()
    }

    /// Contract: on the same frames, the stage holds exactly the formants
    /// `FrameAnalyzer` holds (same kernels, same gate, same hold rule).
    #[test]
    fn stage_matches_frame_analyzer_formants() {
        let sig = synth(140.0, ANALYSIS_FRAME * 4);
        let mut analyzer = FrameAnalyzer::new(SR);
        let mut stage = LpcStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let mut yin = super::super::yin::YinStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        for (i, chunk) in sig.chunks_exact(ANALYSIS_FRAME).enumerate() {
            let expected = analyzer.analyze(chunk).profile;
            let frame = AudioFrame {
                samples: chunk.to_vec(),
                sample_rate: SR,
                frame_index: i as u64,
            };
            let mut f0 = F0Track::default();
            yin.process(&frame, &mut f0).unwrap();
            let mut ft = FormantTrack::held_default(&PipelineParams::DEFAULT.formants);
            stage.process((&frame, &f0), &mut ft).unwrap();
            assert_eq!(ft.formants, expected.formants, "frame {i}");
            assert_eq!(ft.measured_f0, expected.formants_f0, "frame {i}");
        }
    }

    #[test]
    fn unvoiced_frames_hold_the_default_envelope() {
        let mut stage = LpcStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let frame = AudioFrame::preallocated(ANALYSIS_FRAME, SR);
        let mut ft = FormantTrack::held_default(&PipelineParams::DEFAULT.formants);
        stage
            .process((&frame, &F0Track::default()), &mut ft)
            .unwrap();
        assert_eq!(
            ft.formants,
            PipelineParams::DEFAULT.formants.default_formants()
        );
        assert!(!ft.fresh);
        assert_eq!(ft.confidence, 0.0);
    }
}
