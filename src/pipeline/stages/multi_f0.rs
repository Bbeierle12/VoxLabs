//! `harmonic_cancellation`: Coral's note detector as the `multi_f0`
//! stage — `Spectrum` → `NoteSet`. Two detectors as in `dsp-core.ts`:
//! the display one over `[min_freq_hz, max_freq_hz]` gives `active_midi`;
//! the rehearsal one, capped at `harmony_max_hz`, gives the `voices`,
//! each with its parabola-refined fundamental (log magnitudes in its
//! fundamental band) and its confidence.

use crate::choir::note_detector::{DetectorOptions, NoteDetector};
use crate::choir::qifft::parabolic_peak;
use crate::config::{ChoirDetectorConfig, ChoirStftConfig, PipelineParams};

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{DetectedNote, NoteSet, Spectrum};

pub struct MultiF0Stage {
    display: NoteDetector,
    harmony: NoteDetector,
    eps: f32,
    sample_rate: f32,
    num_bins: usize,
    voice_midis: Vec<i32>,
}

impl Stage for MultiF0Stage {
    type In<'a> = &'a Spectrum;
    type Out = NoteSet;
    const NAME: &'static str = "multi_f0";
    const BACKEND: &'static str = "harmonic_cancellation";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, fmt: &StreamFormat) -> Result<Self, StageError> {
        let c: ChoirDetectorConfig = params.choir_detector;
        let s: ChoirStftConfig = params.choir_stft;
        let num_bins = fmt.frame_samples / 2;
        let build = |max: f32| {
            NoteDetector::new(DetectorOptions {
                num_bins,
                sample_rate: fmt.sample_rate_hz,
                min_freq_hz: c.min_freq_hz,
                max_freq_hz: max,
                harmonic: true,
                cfg: c,
            })
            .map_err(StageError::Init)
        };
        Ok(Self {
            display: build(c.max_freq_hz)?,
            harmony: build(c.max_freq_hz.min(c.harmony_max_hz))?,
            eps: s.magnitude_epsilon,
            sample_rate: fmt.sample_rate_hz,
            num_bins,
            voice_midis: Vec::with_capacity(128),
        })
    }

    fn process(&mut self, s: &Spectrum, out: &mut NoteSet) -> Result<(), StageError> {
        if s.magnitude.len() < self.num_bins {
            return Err(StageError::Process(format!(
                "spectrum of {} bins; the detector needs {}",
                s.magnitude.len(),
                self.num_bins
            )));
        }
        let mags = &s.magnitude[..self.num_bins];
        let active = self.display.analyze(mags).map_err(StageError::Process)?;
        out.active_midi.clear();
        out.active_midi.extend_from_slice(active);
        let voices = self.harmony.analyze(mags).map_err(StageError::Process)?;
        self.voice_midis.clear();
        self.voice_midis.extend_from_slice(voices);
        let bin_hz = self.sample_rate / s.fft_size as f32;
        out.voices.clear();
        for &midi in &self.voice_midis {
            let Some((lo, hi)) = self.harmony.bin_range_for_midi(midi) else {
                continue;
            };
            let mut pk = lo as usize;
            for b in lo as usize..=hi as usize {
                if mags[b] > mags[pk] {
                    pk = b;
                }
            }
            let mut f0 = pk as f32 * bin_hz;
            if pk > 0 && pk < mags.len() - 1 {
                let l = (mags[pk - 1] + self.eps).ln();
                let c = (mags[pk] + self.eps).ln();
                let r = (mags[pk + 1] + self.eps).ln();
                f0 = (pk as f32 + parabolic_peak(l, c, r)) * bin_hz;
            }
            out.voices.push(DetectedNote {
                midi,
                f0_hz: f0,
                salience: self.harmony.confidence_of(midi).unwrap_or(1.0),
            });
        }
        out.frame_index = s.frame_index;
        Ok(())
    }
}
