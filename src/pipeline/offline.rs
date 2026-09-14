//! Headless driver: a whole signal through a pipeline on the calling
//! thread, framed exactly as the live runner frames the ring buffer, with
//! every hop's wires handed to a sink. What `vox-harness` and
//! `vox-validation` run; what the study's Python calls through them.

use super::builder::{Pipeline, WiringError, build};
use super::definition::PipelineDefinition;
use super::framing::Framer;
use super::stage::{StageError, StreamFormat};
use super::types::Wire;

/// One hop's outputs as the sink sees them.
pub struct HopOutput<'a> {
    pub hop: u64,
    pub wires: &'a [Wire],
    /// Stage indices whose output the mode file taps.
    pub taps: &'a [usize],
}

#[derive(Debug)]
pub enum OfflineError {
    Wiring(WiringError),
    Stage {
        hop: u64,
        stage: usize,
        source: StageError,
    },
}

impl std::fmt::Display for OfflineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OfflineError::Wiring(e) => write!(f, "{e}"),
            OfflineError::Stage { hop, stage, source } => {
                write!(f, "hop {hop}: stage {stage} failed: {source}")
            }
        }
    }
}

impl std::error::Error for OfflineError {}

/// Builds `def` for `sample_rate_hz` and runs `samples` through it, calling
/// `sink` once per hop. Returns the pipeline (for its provenance) and the
/// hop count.
pub fn run_offline(
    def: &PipelineDefinition,
    sample_rate_hz: f32,
    samples: &[f32],
    mut sink: impl FnMut(HopOutput<'_>),
) -> Result<(Pipeline, u64), OfflineError> {
    let format: StreamFormat = def.format(Some(sample_rate_hz));
    let mut pipeline = build(def, format).map_err(OfflineError::Wiring)?;
    let mut framer = Framer::new(&format, def.runner.pending_capacity_frames);
    let mut hop: u64 = 0;
    for &s in samples {
        framer.push(s);
        while framer.advance() {
            if let Wire::AudioFrame(src) = &mut pipeline.wires[0] {
                src.samples.copy_from_slice(framer.frame());
                src.frame_index = hop;
                src.sample_rate = format.sample_rate_hz;
            }
            pipeline
                .run_hop()
                .map_err(|(stage, source)| OfflineError::Stage { hop, stage, source })?;
            sink(HopOutput {
                hop,
                wires: &pipeline.wires,
                taps: &pipeline.taps,
            });
            hop += 1;
        }
    }
    Ok((pipeline, hop))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::contract::{SR, live_model_coarse, vowel};
    use crate::pipeline::types::Wire;

    /// The offline driver produces the same hop count and the same taps
    /// the live runner does for the same signal.
    #[test]
    fn offline_hops_match_the_framing_rule_and_taps_are_live_values() {
        crate::room::reset();
        let def = live_model_coarse().unwrap();
        let signal = vowel(120.0, 3.86, 1.35, SR as usize);
        let mut hops = Vec::new();
        let (pipeline, n) = run_offline(&def, SR, &signal, |h| {
            if let Some(Wire::F0Track(f0)) = h.wires.get(2) {
                hops.push((h.hop, f0.voiced, f0.hz));
            }
        })
        .unwrap();
        let format = def.format(Some(SR));
        assert_eq!(n, Framer::hops_for(&format, signal.len()));
        assert_eq!(hops.len() as u64, n);
        assert_eq!(pipeline.taps.len(), def.taps.len());
        let mid = &hops[hops.len() / 2];
        assert!(mid.1 && (mid.2 - 120.0).abs() < 3.0, "{mid:?}");
    }
}
