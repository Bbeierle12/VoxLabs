//! Builds a [`Pipeline`] from a definition: validates every wire against
//! the §2 table, names the adapter chain when one is missing, instantiates
//! the stages, and preallocates the wires.

use std::collections::BTreeSet;

use crate::config::PipelineParams;

use super::definition::{DefinitionError, PipelineDefinition};
use super::stage::{DynStage, StageDescriptor, StageError, StreamFormat};
use super::stages::registry;
use super::types::{Wire, WireType};

/// One built stage: its wire indices and the erased implementation.
pub struct BuiltStage {
    pub name: String,
    pub stage: Box<dyn DynStage>,
    /// Wire index per declared input, in declaration order.
    pub inputs: Vec<usize>,
    /// The wire this stage writes.
    pub out: usize,
}

/// A validated, instantiated, preallocated pipeline. Wire 0 is the source
/// `AudioFrame`; wire `i + 1` is stage `i`'s output.
pub struct Pipeline {
    pub name: String,
    pub format: StreamFormat,
    pub params: PipelineParams,
    pub stages: Vec<BuiltStage>,
    pub wires: Vec<Wire>,
    /// Stage indices whose output is published as a tap, in tap-list order.
    pub taps: Vec<usize>,
}

/// One suggested adapter: a registry backend that would produce a missing
/// type from what is available.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterHint {
    pub name: &'static str,
    pub backend: &'static str,
    pub inputs: Vec<WireType>,
    pub output: WireType,
}

#[derive(Debug)]
pub enum WiringError {
    Definition(DefinitionError),
    Empty,
    DuplicateStage(String),
    UnknownBackend {
        stage: String,
        backend: String,
        known: Vec<&'static str>,
    },
    /// A stage needs a type nothing before it produces. `adapter_chain` is
    /// the shortest registry chain that would produce it, in insertion
    /// order, or empty when no registered backend can.
    MissingInput {
        position: usize,
        stage: String,
        backend: &'static str,
        needed: WireType,
        available: Vec<(WireType, String)>,
        adapter_chain: Vec<AdapterHint>,
    },
    UnknownTap {
        tap: String,
        stages: Vec<String>,
    },
    Init {
        stage: String,
        source: StageError,
    },
    Preallocate(StageError),
}

impl std::fmt::Display for WiringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WiringError::Definition(e) => write!(f, "{e}"),
            WiringError::Empty => write!(f, "pipeline has no stages"),
            WiringError::DuplicateStage(n) => write!(f, "stage name `{n}` is used twice"),
            WiringError::UnknownBackend {
                stage,
                backend,
                known,
            } => write!(
                f,
                "stage `{stage}`: unknown backend `{backend}`; known backends: {}",
                known.join(", ")
            ),
            WiringError::MissingInput {
                position,
                stage,
                backend,
                needed,
                available,
                adapter_chain,
            } => {
                writeln!(
                    f,
                    "stage {position} `{stage}` ({backend}) needs {needed}, but nothing before it produces {needed}."
                )?;
                let avail: Vec<String> = available
                    .iter()
                    .map(|(t, from)| format!("{t} ({from})"))
                    .collect();
                writeln!(f, "  available at that point: {}", avail.join(", "))?;
                if adapter_chain.is_empty() {
                    write!(f, "  no registered backend produces {needed}")
                } else {
                    let chain: Vec<String> = adapter_chain
                        .iter()
                        .map(|a| {
                            let ins: Vec<&str> = a.inputs.iter().map(|t| t.name()).collect();
                            format!("{} ({} → {})", a.name, ins.join(" + "), a.output)
                        })
                        .collect();
                    write!(
                        f,
                        "  adapter chain to insert before `{stage}`: {}",
                        chain.join(", ")
                    )
                }
            }
            WiringError::UnknownTap { tap, stages } => write!(
                f,
                "tap `{tap}` names no stage; stages are: {}",
                stages.join(", ")
            ),
            WiringError::Init { stage, source } => {
                write!(f, "stage `{stage}` failed to init: {source}")
            }
            WiringError::Preallocate(e) => write!(f, "cannot preallocate wire: {e}"),
        }
    }
}

impl std::error::Error for WiringError {}

impl From<DefinitionError> for WiringError {
    fn from(e: DefinitionError) -> Self {
        WiringError::Definition(e)
    }
}

/// Shortest registry chain that makes `needed` producible from `available`,
/// in the order the stages would have to be inserted. Empty when no chain
/// exists. Breadth-first over the (tiny) registry: repeatedly add every
/// backend whose inputs are all available, then keep only what `needed`
/// depends on.
pub fn adapter_chain(available: &[WireType], needed: WireType) -> Vec<AdapterHint> {
    let reg = registry();
    let mut have: BTreeSet<WireType> = available.iter().copied().collect();
    let mut order: Vec<&StageDescriptor> = Vec::new();
    let mut progress = true;
    while progress && !have.contains(&needed) {
        progress = false;
        for d in reg {
            if have.contains(&d.output) {
                continue;
            }
            if d.inputs.iter().all(|t| have.contains(t)) {
                have.insert(d.output);
                order.push(d);
                progress = true;
            }
        }
    }
    if !have.contains(&needed) {
        return Vec::new();
    }
    // Dependency closure of `needed` over the discovered producers.
    let original: BTreeSet<WireType> = available.iter().copied().collect();
    let mut wanted: BTreeSet<WireType> = BTreeSet::new();
    let mut frontier = vec![needed];
    while let Some(t) = frontier.pop() {
        if original.contains(&t) || !wanted.insert(t) {
            continue;
        }
        if let Some(d) = order.iter().find(|d| d.output == t) {
            frontier.extend(d.inputs.iter().copied());
        }
    }
    order
        .iter()
        .filter(|d| wanted.contains(&d.output))
        .map(|d| AdapterHint {
            name: d.name,
            backend: d.backend,
            inputs: d.inputs.to_vec(),
            output: d.output,
        })
        .collect()
}

/// Validates, instantiates, and preallocates `def` for `format`.
pub fn build(def: &PipelineDefinition, format: StreamFormat) -> Result<Pipeline, WiringError> {
    if def.stages.is_empty() {
        return Err(WiringError::Empty);
    }
    let params = def.params()?;
    let reg = registry();
    let known: Vec<&'static str> = reg.iter().map(|d| d.backend).collect();

    // Wire 0 is the source frame. `producers` maps each available type to
    // the wire index of its most recent producer.
    let mut wires: Vec<Wire> = vec![
        Wire::preallocate(
            WireType::AudioFrame,
            format.frame_samples,
            format.sample_rate_hz,
            &params.formants,
        )
        .map_err(WiringError::Preallocate)?,
    ];
    let mut producers: Vec<(WireType, usize, String)> =
        vec![(WireType::AudioFrame, 0, "source".to_string())];
    let mut names: BTreeSet<&str> = BTreeSet::new();
    let mut stages: Vec<BuiltStage> = Vec::with_capacity(def.stages.len());

    for (position, sd) in def.stages.iter().enumerate() {
        if !names.insert(sd.name.as_str()) {
            return Err(WiringError::DuplicateStage(sd.name.clone()));
        }
        let desc = reg
            .iter()
            .find(|d| d.backend == sd.backend)
            .ok_or_else(|| WiringError::UnknownBackend {
                stage: sd.name.clone(),
                backend: sd.backend.clone(),
                known: known.clone(),
            })?;
        let mut inputs = Vec::with_capacity(desc.inputs.len());
        for &needed in desc.inputs {
            match producers.iter().rev().find(|(t, _, _)| *t == needed) {
                Some((_, idx, _)) => inputs.push(*idx),
                None => {
                    let available: Vec<WireType> = producers.iter().map(|(t, _, _)| *t).collect();
                    return Err(WiringError::MissingInput {
                        position,
                        stage: sd.name.clone(),
                        backend: desc.backend,
                        needed,
                        available: producers
                            .iter()
                            .map(|(t, _, from)| (*t, from.clone()))
                            .collect(),
                        adapter_chain: adapter_chain(&available, needed),
                    });
                }
            }
        }
        let stage = (desc.make)(&params, &format).map_err(|source| WiringError::Init {
            stage: sd.name.clone(),
            source,
        })?;
        let out = wires.len();
        wires.push(
            Wire::preallocate(
                desc.output,
                format.frame_samples,
                format.sample_rate_hz,
                &params.formants,
            )
            .map_err(WiringError::Preallocate)?,
        );
        producers.push((desc.output, out, sd.name.clone()));
        stages.push(BuiltStage {
            name: sd.name.clone(),
            stage,
            inputs,
            out,
        });
    }

    let mut taps = Vec::with_capacity(def.taps.len());
    for tap in &def.taps {
        let idx =
            stages
                .iter()
                .position(|s| s.name == *tap)
                .ok_or_else(|| WiringError::UnknownTap {
                    tap: tap.clone(),
                    stages: def.stages.iter().map(|s| s.name.clone()).collect(),
                })?;
        taps.push(idx);
    }

    Ok(Pipeline {
        name: def.name.clone(),
        format,
        params,
        stages,
        wires,
        taps,
    })
}

impl Pipeline {
    /// Runs every stage once over the current source wire, in order. Stops
    /// at the first failing stage and returns which one.
    pub fn run_hop(&mut self) -> Result<(), (usize, StageError)> {
        for (i, s) in self.stages.iter_mut().enumerate() {
            s.stage
                .run(&mut self.wires, &s.inputs, s.out)
                .map_err(|e| (i, e))?;
        }
        Ok(())
    }

    /// Runs the stages, recording each stage's wall time into `timings`
    /// (one entry per stage, seconds). Same stop-at-first-error contract.
    pub fn run_hop_timed(&mut self, timings: &mut [f64]) -> Result<(), (usize, StageError)> {
        for (i, s) in self.stages.iter_mut().enumerate() {
            let t = std::time::Instant::now();
            let r = s.stage.run(&mut self.wires, &s.inputs, s.out);
            if let Some(slot) = timings.get_mut(i) {
                *slot = t.elapsed().as_secs_f64();
            }
            r.map_err(|e| (i, e))?;
        }
        Ok(())
    }

    pub fn stage_names(&self) -> Vec<String> {
        self.stages.iter().map(|s| s.name.clone()).collect()
    }
}
