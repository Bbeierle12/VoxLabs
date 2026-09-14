//! `labeler`: Coral's section labeler as the `satb` stage — the
//! `NoteSet` voices → `SectionLabels`, confidence-weighted by the
//! detector's salience (no singer's-formant tiebreaker, as in Coral's
//! production path).

use crate::choir::labeler::label_sections;
use crate::config::{ChoirLabelerConfig, PipelineParams};

use super::super::stage::{Stage, StageError, StreamFormat};
use super::super::types::{NoteSet, SectionLabels};

pub struct SatbStage {
    cfg: ChoirLabelerConfig,
    midis: Vec<i32>,
}

impl Stage for SatbStage {
    type In<'a> = &'a NoteSet;
    type Out = SectionLabels;
    const NAME: &'static str = "satb";
    const BACKEND: &'static str = "labeler";
    const VERSION: &'static str = "1.0.0";

    fn init(params: &PipelineParams, _fmt: &StreamFormat) -> Result<Self, StageError> {
        Ok(Self {
            cfg: params.choir_labeler,
            midis: Vec::with_capacity(128),
        })
    }

    fn process(&mut self, n: &NoteSet, out: &mut SectionLabels) -> Result<(), StageError> {
        self.midis.clear();
        self.midis.extend(n.voices.iter().map(|v| v.midi));
        let conf = |midi: i32| n.voices.iter().find(|v| v.midi == midi).map(|v| v.salience);
        let labels = label_sections(&self.midis, Some(&conf), None, &self.cfg);
        out.labels.clear();
        out.labels.extend(labels);
        out.frame_index = n.frame_index;
        Ok(())
    }
}
