//! `yin_cpu`: `math::yin_pitch` wrapped as a stage, with the confidence and
//! range gate the CPU analyzers apply (`frame::FrameAnalyzer`).

use crate::config::{PipelineParams, VoicingConfig, YinConfig};
use crate::math;

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{AudioFrame, F0Track};
use super::require_default;

pub struct YinStage {
    voicing: VoicingConfig,
}

impl Stage for YinStage {
    type In<'a> = &'a AudioFrame;
    type Out = F0Track;
    const NAME: &'static str = "yin";
    const BACKEND: &'static str = "yin_cpu";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, fmt: &StreamFormat) -> Result<Self, StageError> {
        require_default("yin", Self::NAME, &params.yin, &YinConfig::DEFAULT)?;
        let max_lag = (fmt.sample_rate_hz / params.yin.f0_min_hz) as usize;
        let needed = params.yin.window + max_lag;
        if fmt.frame_samples < needed {
            return Err(StageError::Init(format!(
                "frame of {} samples cannot hold yin.window {} + max lag {} at {:.0} Hz",
                fmt.frame_samples, params.yin.window, max_lag, fmt.sample_rate_hz
            )));
        }
        Ok(Self {
            voicing: params.voicing,
        })
    }

    fn process(&mut self, frame: &AudioFrame, out: &mut F0Track) -> Result<(), StageError> {
        if !(frame.sample_rate.is_finite() && frame.sample_rate > 0.0) {
            return Err(StageError::Process(format!(
                "frame sample rate {} is not positive",
                frame.sample_rate
            )));
        }
        *out = match math::yin_pitch(&frame.samples, frame.sample_rate) {
            Some(p) => F0Track {
                hz: p.f0,
                confidence: p.confidence,
                voiced: p.confidence > self.voicing.min_confidence
                    && (self.voicing.f0_min_hz..=self.voicing.f0_max_hz).contains(&p.f0),
                snr_db: None,
                rejected: false,
            },
            None => F0Track::default(),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::types::Wire;
    use super::*;
    use crate::pipeline::stage::{DynStage, Erased};
    use std::f32::consts::TAU;

    const SR: f32 = 48_000.0;
    const FRAME: usize = 2048;

    fn fmt() -> StreamFormat {
        StreamFormat {
            sample_rate_hz: SR,
            frame_samples: FRAME,
            hop: FRAME / 2,
        }
    }

    fn sine(f0: f32, n: usize) -> Vec<f32> {
        (0..n).map(|i| (TAU * f0 * i as f32 / SR).sin()).collect()
    }

    /// Contract: the stage reports exactly what `math::yin_pitch` reports,
    /// plus the analyzers' voicing gate.
    #[test]
    fn stage_matches_direct_yin_call() {
        let mut s = YinStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let frame = AudioFrame {
            samples: sine(220.0, FRAME),
            sample_rate: SR,
            frame_index: 0,
        };
        let mut out = F0Track::default();
        s.process(&frame, &mut out).unwrap();
        let direct = math::yin_pitch(&frame.samples, SR).unwrap();
        assert_eq!(out.hz, direct.f0);
        assert_eq!(out.confidence, direct.confidence);
        assert!(out.voiced);
        assert!((out.hz - 220.0).abs() < 2.0, "f0 {}", out.hz);
    }

    #[test]
    fn silence_is_unvoiced() {
        let mut s = YinStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let frame = AudioFrame::preallocated(FRAME, SR);
        let mut out = F0Track {
            hz: 1.0,
            confidence: 1.0,
            voiced: true,
            ..Default::default()
        };
        s.process(&frame, &mut out).unwrap();
        assert!(!out.voiced);
    }

    #[test]
    fn too_short_a_frame_is_refused_at_init() {
        let short = StreamFormat {
            sample_rate_hz: SR,
            frame_samples: 1024,
            hop: 512,
        };
        assert!(matches!(
            YinStage::init(&PipelineParams::DEFAULT, &short),
            Err(StageError::Init(_))
        ));
    }

    #[test]
    fn a_yin_override_is_refused_loudly() {
        let mut p = PipelineParams::DEFAULT;
        p.yin.threshold = 0.2;
        let err = YinStage::init(&p, &fmt()).err().expect("refused");
        assert!(err.to_string().contains("[params.yin]"), "{err}");
    }

    /// The erased form hands the right wires through.
    #[test]
    fn erased_stage_runs_over_wires() {
        let mut s = Erased(YinStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap());
        let mut wires = vec![
            Wire::AudioFrame(AudioFrame {
                samples: sine(140.0, FRAME),
                sample_rate: SR,
                frame_index: 0,
            }),
            Wire::F0Track(F0Track::default()),
        ];
        s.run(&mut wires, &[0], 1).unwrap();
        let Wire::F0Track(t) = &wires[1] else {
            panic!()
        };
        assert!((t.hz - 140.0).abs() < 2.0);
        // A wrong wire is an error, not a silent read.
        let mut bad = vec![
            Wire::F0Track(F0Track::default()),
            Wire::F0Track(F0Track::default()),
        ];
        assert!(matches!(
            s.run(&mut bad, &[0], 1),
            Err(StageError::WrongInput { .. })
        ));
    }
}
