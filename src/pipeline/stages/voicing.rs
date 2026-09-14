//! `snr_hum`: the voicing gates on top of the raw estimate — the learned
//! ambient floor's SNR gate and the calibrated-interferer (room hum) gate,
//! exactly as `frame::FrameAnalyzer` applies them. Periodicity alone is not
//! voice: a YIN-voiced frame that fails either gate is demoted to unvoiced
//! here, protecting every downstream consumer at one point. The floor
//! learns only from unvoiced frames, so sustained singing cannot raise it.

use crate::config::{NoiseFloorConfig, PipelineParams, RoomCalibrationConfig, VoicingConfig};
use crate::math::{self, Interferer, NoiseFloor};
use crate::room;

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{AudioFrame, F0Track};
use super::require_default;

pub struct VoicingStage {
    voicing: VoicingConfig,
    floor: NoiseFloor,
    interferer: Option<Interferer>,
    room_generation: u64,
}

impl VoicingStage {
    /// Picks up a calibration the room slot received since the last hop.
    fn sync_room(&mut self) {
        let r = room::snapshot();
        if r.generation == self.room_generation {
            return;
        }
        self.room_generation = r.generation;
        if let Some(seed) = r.floor_seed {
            self.floor.seed(seed);
        }
        self.interferer = r.interferer;
    }
}

impl Stage for VoicingStage {
    type In<'a> = (&'a AudioFrame, &'a F0Track);
    type Out = F0Track;
    const NAME: &'static str = "voicing";
    const BACKEND: &'static str = "snr_hum";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, _fmt: &StreamFormat) -> Result<Self, StageError> {
        require_default(
            "noise_floor",
            Self::NAME,
            &params.noise_floor,
            &NoiseFloorConfig::DEFAULT,
        )?;
        require_default(
            "room_calibration",
            Self::NAME,
            &params.room_calibration,
            &RoomCalibrationConfig::DEFAULT,
        )?;
        require_default(
            "voicing",
            Self::NAME,
            &params.voicing,
            &VoicingConfig::DEFAULT,
        )?;
        let mut stage = Self {
            voicing: params.voicing,
            floor: NoiseFloor::new(),
            interferer: None,
            // Force a load on the first hop so a calibration done before
            // the pipeline was built still applies.
            room_generation: u64::MAX,
        };
        stage.sync_room();
        Ok(stage)
    }

    fn process(
        &mut self,
        (frame, raw): (&AudioFrame, &F0Track),
        out: &mut F0Track,
    ) -> Result<(), StageError> {
        self.sync_room();
        let rms = math::frame_rms(&frame.samples);
        let snr_db = self.floor.snr_db(rms);
        let snr_ok = snr_db.is_none_or(|s| s >= self.voicing.voiced_min_snr_db);
        let hum = self
            .interferer
            .is_some_and(|i| math::interferer_match(raw.hz, rms, &i));
        let voiced = raw.voiced && snr_ok && !hum;
        if !raw.voiced {
            self.floor.push_unvoiced(rms);
        }
        *out = F0Track {
            hz: if voiced { raw.hz } else { 0.0 },
            confidence: raw.confidence,
            voiced,
            snr_db,
            rejected: raw.voiced && (!snr_ok || hum),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::yin::YinStage;
    use super::*;
    use crate::frame::{ANALYSIS_FRAME, FrameAnalyzer};
    use std::f32::consts::TAU;

    const SR: f32 = 48_000.0;

    fn fmt() -> StreamFormat {
        StreamFormat {
            sample_rate_hz: SR,
            frame_samples: ANALYSIS_FRAME,
            hop: ANALYSIS_FRAME,
        }
    }

    fn tone(f0: f32, amp: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| amp * (TAU * f0 * i as f32 / SR).sin())
            .collect()
    }

    fn noise(amp: f32, n: usize, seed: &mut u64) -> Vec<f32> {
        (0..n)
            .map(|_| {
                *seed ^= *seed << 13;
                *seed ^= *seed >> 7;
                *seed ^= *seed << 17;
                amp * ((*seed >> 40) as f32 / (1u64 << 24) as f32 - 0.5)
            })
            .collect()
    }

    /// Contract: on the same frame sequence the stage's verdict (voiced,
    /// f0, SNR, rejected) is `FrameAnalyzer`'s, frame for frame.
    #[test]
    fn stage_matches_frame_analyzer_gates() {
        room::reset();
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        // Forty quiet frames teach the floor, then a loud tone, then a tone
        // barely above the noise (rejected by SNR), then silence.
        let mut sig: Vec<f32> = noise(0.002, ANALYSIS_FRAME * 40, &mut seed);
        sig.extend(tone(180.0, 0.3, ANALYSIS_FRAME * 4));
        sig.extend(
            tone(180.0, 0.004, ANALYSIS_FRAME * 4)
                .iter()
                .zip(noise(0.002, ANALYSIS_FRAME * 4, &mut seed))
                .map(|(a, b)| a + b),
        );
        sig.extend(std::iter::repeat_n(0.0, ANALYSIS_FRAME * 2));

        let mut analyzer = FrameAnalyzer::new(SR);
        let mut yin = YinStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let mut gate = VoicingStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let mut saw_voiced = false;
        let mut saw_rejected = false;
        for (i, chunk) in sig.chunks_exact(ANALYSIS_FRAME).enumerate() {
            let expected = analyzer.analyze(chunk);
            let frame = AudioFrame {
                samples: chunk.to_vec(),
                sample_rate: SR,
                frame_index: i as u64,
            };
            let mut raw = F0Track::default();
            yin.process(&frame, &mut raw).unwrap();
            let mut gated = F0Track::default();
            gate.process((&frame, &raw), &mut gated).unwrap();
            assert_eq!(gated.voiced, expected.profile.valid, "frame {i} voiced");
            assert_eq!(gated.hz, expected.profile.f0, "frame {i} f0");
            assert_eq!(
                gated.snr_db, expected.profile.metrics.snr_db,
                "frame {i} snr"
            );
            assert_eq!(
                gated.rejected, expected.profile.metrics.voiced_but_noisy,
                "frame {i} rejected"
            );
            saw_voiced |= gated.voiced;
            saw_rejected |= gated.rejected;
        }
        assert!(saw_voiced, "the loud tone must pass");
        assert!(saw_rejected, "the faint tone must be rejected by SNR");
    }

    /// A calibration installed through the room slot reaches the stage on
    /// the next hop: a tone at the fingerprinted hum's pitch and level is
    /// gated out; a louder one on the same pitch passes.
    #[test]
    fn a_calibrated_hum_is_gated_and_a_louder_voice_passes() {
        room::reset();
        let mut gate = VoicingStage::init(&PipelineParams::DEFAULT, &fmt()).unwrap();
        let hum = Interferer {
            f0_hz: 120.0,
            rms: 0.02,
        };
        room::set_calibration(0.001, Some(hum));
        let raw = F0Track {
            hz: 120.0,
            confidence: 0.9,
            voiced: true,
            ..Default::default()
        };
        let mut out = F0Track::default();
        let quiet = AudioFrame {
            samples: tone(120.0, 0.02 * std::f32::consts::SQRT_2, ANALYSIS_FRAME),
            sample_rate: SR,
            frame_index: 0,
        };
        gate.process((&quiet, &raw), &mut out).unwrap();
        assert!(!out.voiced && out.rejected, "the hum itself: {out:?}");
        let loud = AudioFrame {
            samples: tone(120.0, 0.5, ANALYSIS_FRAME),
            sample_rate: SR,
            frame_index: 1,
        };
        gate.process((&loud, &raw), &mut out).unwrap();
        assert!(out.voiced, "a louder voice on the hum's pitch: {out:?}");
        room::reset();
    }
}
