//! `coral_hann`: Coral's STFT as `stft` backend #2 — periodic Hann,
//! `2 / Σw` one-sided scaling (a unit sine at a bin centre reads 0 dB),
//! dB with Coral's epsilon. The frame is the mode's `frame_samples`
//! (`choir.toml`: 8192, hop 2048 — Coral's detection framing).

use crate::choir::stft::CoralStft;
use crate::config::consts::DB_PER_DECADE_AMPLITUDE;
use crate::config::{ChoirStftConfig, PipelineParams};

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{AudioFrame, Spectrum};

pub struct CoralStftStage {
    stft: CoralStft,
    cfg: ChoirStftConfig,
}

impl Stage for CoralStftStage {
    type In<'a> = &'a AudioFrame;
    type Out = Spectrum;
    const NAME: &'static str = "stft";
    const BACKEND: &'static str = "coral_hann";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, fmt: &StreamFormat) -> Result<Self, StageError> {
        Ok(Self {
            stft: CoralStft::new(fmt.frame_samples).map_err(StageError::Init)?,
            cfg: params.choir_stft,
        })
    }

    fn process(&mut self, frame: &AudioFrame, out: &mut Spectrum) -> Result<(), StageError> {
        let n = self.stft.fft_size();
        if out.magnitude.len() != n / 2 + 1 {
            return Err(StageError::Process(format!(
                "spectrum of {} bins for fft {n}",
                out.magnitude.len()
            )));
        }
        self.stft
            .magnitudes(&frame.samples, &mut out.magnitude)
            .map_err(StageError::Process)?;
        for (db, m) in out.magnitudes_db.iter_mut().zip(&out.magnitude) {
            *db = DB_PER_DECADE_AMPLITUDE * (m + self.cfg.magnitude_epsilon).log10();
        }
        out.bin_hz = frame.sample_rate / n as f32;
        out.fft_size = n;
        out.frame_index = frame.frame_index;
        Ok(())
    }
}
