//! The `Stage` contract (Plan v3 §3) and its type-erased form for the runner.
//!
//! A typed stage names its input (one wire, or a tuple) through a generic
//! associated type so a two-input stage can borrow both wires at once; the
//! plan's sketch used a plain `In`, which cannot express that borrow. Every
//! other clause of the sketch holds: `init` once, all allocation there;
//! `process` per hop, no I/O, no panics on bad input — `Err`.

use crate::config::PipelineParams;

use super::types::{FromWires, Wire, WireType, WireValue};

/// The stream the pipeline is built for. The mode file carries the nominal
/// rate; the shell substitutes the device's real rate before building.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StreamFormat {
    pub sample_rate_hz: f32,
    pub frame_samples: usize,
    pub hop: usize,
}

impl StreamFormat {
    /// Seconds of audio one hop advances — the per-hop budget.
    pub fn hop_seconds(&self) -> f32 {
        self.hop as f32 / self.sample_rate_hz
    }
}

/// Identity for provenance: name, semver, backend id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageIdent {
    pub name: &'static str,
    pub version: &'static str,
    pub backend: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StageError {
    /// `init` refused: a format or parameter the backend cannot honor.
    Init(String),
    /// The runner handed the stage a wire of the wrong type. The builder
    /// prevents this; seeing it at runtime is a bug, reported not hidden.
    WrongInput { expected: WireType, got: WireType },
    /// `process` refused this hop's input.
    Process(String),
}

impl std::fmt::Display for StageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StageError::Init(m) => write!(f, "init: {m}"),
            StageError::WrongInput { expected, got } => {
                write!(f, "wired a {got} where {expected} was expected")
            }
            StageError::Process(m) => write!(f, "process: {m}"),
        }
    }
}

impl std::error::Error for StageError {}

/// One stage of the analysis chain.
pub trait Stage: Send + 'static {
    /// What it consumes: `&'a AudioFrame`, `(&'a F0Track, &'a FormantTrack)`, …
    type In<'a>: FromWires<'a>
    where
        Self: 'a;
    /// What it produces, written in place into the preallocated wire.
    type Out: WireValue;

    const NAME: &'static str;
    const BACKEND: &'static str;
    const VERSION: &'static str;

    /// Called once. All allocation happens here. Config is immutable after.
    fn init(params: &PipelineParams, fmt: &StreamFormat) -> Result<Self, StageError>
    where
        Self: Sized;

    /// Called per hop on the worker. No allocation, no I/O, no panics on bad
    /// input — return `Err`.
    fn process(&mut self, input: Self::In<'_>, out: &mut Self::Out) -> Result<(), StageError>;

    fn ident(&self) -> StageIdent {
        StageIdent {
            name: Self::NAME,
            version: Self::VERSION,
            backend: Self::BACKEND,
        }
    }
}

/// The runner's view of a stage: wire indices in, one wire index out.
pub trait DynStage: Send {
    fn ident(&self) -> StageIdent;
    fn inputs(&self) -> &'static [WireType];
    fn output(&self) -> WireType;
    /// Runs one hop. `inputs` index wires before `out`; the builder
    /// guarantees the order, so the slice can be split at `out`.
    fn run(&mut self, wires: &mut [Wire], inputs: &[usize], out: usize) -> Result<(), StageError>;
}

/// Wraps a typed [`Stage`] as a [`DynStage`].
pub struct Erased<S: Stage>(pub S);

impl<S: Stage> DynStage for Erased<S> {
    fn ident(&self) -> StageIdent {
        self.0.ident()
    }

    fn inputs(&self) -> &'static [WireType] {
        <S::In<'static> as FromWires<'static>>::TYPES
    }

    fn output(&self) -> WireType {
        S::Out::TYPE
    }

    fn run(&mut self, wires: &mut [Wire], inputs: &[usize], out: usize) -> Result<(), StageError> {
        if inputs.iter().any(|&i| i >= out) {
            return Err(StageError::Process(format!(
                "stage `{}` wired to a wire at or after its own output",
                S::NAME
            )));
        }
        let (before, from_out) = wires.split_at_mut(out);
        let slot = from_out
            .first_mut()
            .ok_or_else(|| StageError::Process(format!("output wire {out} out of range")))?;
        let output = S::Out::from_wire_mut(slot).ok_or(StageError::WrongInput {
            expected: S::Out::TYPE,
            got: WireType::AudioFrame,
        })?;
        let input = S::In::from_wires(before, inputs)?;
        self.0.process(input, output)
    }
}

/// A backend the builder can instantiate: its identity, its wire signature,
/// and its constructor. The registry of these is the whole adapter search
/// space when a wiring error needs to name the missing chain.
pub struct StageDescriptor {
    pub name: &'static str,
    pub backend: &'static str,
    pub inputs: &'static [WireType],
    pub output: WireType,
    pub make: fn(&PipelineParams, &StreamFormat) -> Result<Box<dyn DynStage>, StageError>,
}

/// The constructor for a typed stage, in the registry's shape.
pub fn make_erased<S: Stage>(
    params: &PipelineParams,
    fmt: &StreamFormat,
) -> Result<Box<dyn DynStage>, StageError> {
    Ok(Box::new(Erased(S::init(params, fmt)?)))
}

/// Descriptor for a typed stage.
pub const fn describe<S: Stage>() -> StageDescriptor {
    StageDescriptor {
        name: S::NAME,
        backend: S::BACKEND,
        inputs: <S::In<'static> as FromWires<'static>>::TYPES,
        output: S::Out::TYPE,
        make: make_erased::<S>,
    }
}
