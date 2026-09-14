//! `f0_contour`: vibrato and sustain steadiness from the f0 history
//! (`metrics::F0Contour`), merged into the frame's `VoiceMetrics`. One
//! contour sample per hop, so the contour rate is `sample_rate / hop` —
//! at hop 1024 twice the pre-pipeline rate, which the tracker's window
//! and gap lengths scale with (they are in seconds).

use crate::config::{PipelineParams, VibratoConfig};
use crate::metrics::F0Contour;
use crate::types::VoiceMetrics;

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::F0Track;
use super::require_default;

pub struct ContourStage {
    contour: F0Contour,
}

impl Stage for ContourStage {
    type In<'a> = (&'a F0Track, &'a VoiceMetrics);
    type Out = VoiceMetrics;
    const NAME: &'static str = "contour";
    const BACKEND: &'static str = "f0_contour";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, fmt: &StreamFormat) -> Result<Self, StageError> {
        require_default(
            "vibrato",
            Self::NAME,
            &params.vibrato,
            &VibratoConfig::DEFAULT,
        )?;
        if fmt.hop == 0 {
            return Err(StageError::Init("hop must be positive".into()));
        }
        Ok(Self {
            contour: F0Contour::new(fmt.sample_rate_hz / fmt.hop as f32),
        })
    }

    fn process(
        &mut self,
        (f0, metrics): (&F0Track, &VoiceMetrics),
        out: &mut VoiceMetrics,
    ) -> Result<(), StageError> {
        self.contour.push(f0.hz, f0.voiced);
        let (vibrato, steadiness) = self.contour.analyze();
        *out = *metrics;
        out.vibrato = vibrato;
        out.steadiness_cents = steadiness;
        Ok(())
    }
}
