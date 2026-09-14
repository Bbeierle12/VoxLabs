//! `qifft`: Coral's single-F0 estimator as a `pitch` stage —
//! `Spectrum` → `F0Track` (the intonation trail). Coral ran it on its
//! display STFT (2048); here it reads whatever spectrum the mode taps.

use crate::choir::qifft::estimate_f0;
use crate::config::{PipelineParams, QifftConfig};

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{F0Track, Spectrum};

pub struct QifftStage {
    cfg: QifftConfig,
    sample_rate: f32,
}

impl Stage for QifftStage {
    type In<'a> = &'a Spectrum;
    type Out = F0Track;
    const NAME: &'static str = "pitch";
    const BACKEND: &'static str = "qifft";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, fmt: &StreamFormat) -> Result<Self, StageError> {
        Ok(Self {
            cfg: params.qifft,
            sample_rate: fmt.sample_rate_hz,
        })
    }

    fn process(&mut self, s: &Spectrum, out: &mut F0Track) -> Result<(), StageError> {
        // Coral's numBins = fftSize / 2 (no Nyquist bin).
        let mags = &s.magnitude[..s.fft_size / 2];
        match estimate_f0(mags, self.sample_rate, s.fft_size, &self.cfg) {
            Some(e) => {
                out.hz = e.frequency_hz;
                out.confidence = e.confidence;
                out.voiced = true;
            }
            None => {
                out.hz = 0.0;
                out.confidence = 0.0;
                out.voiced = false;
            }
        }
        out.snr_db = None;
        out.rejected = false;
        Ok(())
    }
}
