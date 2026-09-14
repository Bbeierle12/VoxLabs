//! The §2 type universe. Keep it small: adding a variant here requires a
//! line in the plan's table. A stage declares exactly which of these it
//! consumes and produces; the builder refuses to connect anything else.

pub use crate::atlas::reduced_model::{AbstainReason, MAX_MODES};
use crate::config::FormantConfig;
use crate::tract::{self, ADULT_FEMALE, ADULT_MALE, N_SECTIONS, TractBasis};
use crate::types::{Formant, MAX_PARTIALS, N_FORMANTS, VoiceMetrics};

use super::stage::StageError;

/// Every wire type in Plan v3 §2, whether or not Phase 1 produces it. The
/// builder validates against this whole table so a later stage's wiring
/// error still names the right type.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum WireType {
    AudioFrame,
    Spectrum,
    F0Track,
    HarmonicSeries,
    VoiceMetrics,
    FormantTrack,
    TractParams,
    AreaFunction,
    TractGeometry,
    Loudness,
    NoteSet,
    SectionLabels,
    SpatialCal,
}

impl WireType {
    /// The table, in the plan's order.
    pub const ALL: &'static [WireType] = &[
        WireType::AudioFrame,
        WireType::Spectrum,
        WireType::F0Track,
        WireType::HarmonicSeries,
        WireType::VoiceMetrics,
        WireType::FormantTrack,
        WireType::TractParams,
        WireType::AreaFunction,
        WireType::TractGeometry,
        WireType::Loudness,
        WireType::NoteSet,
        WireType::SectionLabels,
        WireType::SpatialCal,
    ];

    pub fn name(self) -> &'static str {
        match self {
            WireType::AudioFrame => "AudioFrame",
            WireType::Spectrum => "Spectrum",
            WireType::F0Track => "F0Track",
            WireType::HarmonicSeries => "HarmonicSeries",
            WireType::VoiceMetrics => "VoiceMetrics",
            WireType::FormantTrack => "FormantTrack",
            WireType::TractParams => "TractParams",
            WireType::AreaFunction => "AreaFunction",
            WireType::TractGeometry => "TractGeometry",
            WireType::Loudness => "Loudness",
            WireType::NoteSet => "NoteSet",
            WireType::SectionLabels => "SectionLabels",
            WireType::SpatialCal => "SpatialCal",
        }
    }
}

impl std::fmt::Display for WireType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// One analysis frame from the ring-buffer reader: `frame_samples` mono
/// samples ending at the hop boundary, the rate they were captured at, and
/// the hop index (frame 0 is the first full frame).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AudioFrame {
    pub samples: Vec<f32>,
    pub sample_rate: f32,
    pub frame_index: u64,
}

impl AudioFrame {
    /// A zero frame of the stream's length — the preallocated source wire.
    pub fn preallocated(frame_samples: usize, sample_rate: f32) -> Self {
        Self {
            samples: vec![0.0; frame_samples],
            sample_rate,
            frame_index: 0,
        }
    }
}

/// Fundamental frequency for one frame. `voiced` is the pitch stage's own
/// gate (confidence and range); the SNR and hum gates are a later stage.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct F0Track {
    pub hz: f32,
    pub confidence: f32,
    pub voiced: bool,
    /// Frame SNR over the learned ambient floor, dB, once the `voicing`
    /// stage has run; `None` from the raw estimator or while the floor
    /// warms up. A frame property, present on unvoiced frames too.
    pub snr_db: Option<f32>,
    /// The estimator found a period the SNR/hum gates rejected: there IS
    /// a periodic source, but the room is too loud to measure it honestly.
    pub rejected: bool,
}

/// One-sided magnitude spectrum of the current frame (Hann-windowed FFT
/// over the whole frame), in linear magnitude and dB.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Spectrum {
    pub magnitude: Vec<f32>,
    pub magnitudes_db: Vec<f32>,
    /// Hz per bin (`sample_rate / fft_size`).
    pub bin_hz: f32,
    pub fft_size: usize,
    pub frame_index: u64,
}

impl Spectrum {
    pub fn preallocated(fft_size: usize, sample_rate: f32, db_floor: f32) -> Self {
        let bins = fft_size / crate::config::consts::TWO_USIZE + 1;
        Self {
            magnitude: vec![0.0; bins],
            magnitudes_db: vec![db_floor; bins],
            bin_hz: sample_rate / fft_size as f32,
            fft_size,
            frame_index: 0,
        }
    }
}

/// Measured amplitude of each harmonic k·f0 (linear peak); zeroed when
/// the frame is unvoiced.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HarmonicSeries {
    pub amplitudes: [f32; MAX_PARTIALS],
    pub f0_hz: f32,
    pub voiced: bool,
}

impl Default for HarmonicSeries {
    fn default() -> Self {
        Self {
            amplitudes: [0.0; MAX_PARTIALS],
            f0_hz: 0.0,
            voiced: false,
        }
    }
}

/// Formants for one frame. Held across unvoiced frames exactly as the
/// existing analyzers hold them, so `measured_f0` is the f0 of the frame
/// they were measured on — what reliability grading must judge by.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FormantTrack {
    pub formants: [Formant; N_FORMANTS],
    /// f0 of the frame the formants were measured on; 0.0 = never measured
    /// (the configured default envelope).
    pub measured_f0: f32,
    /// 1.0 when F1 resolved on this frame, 0.0 when held. The LPC backend
    /// has no finer confidence; a later backend may.
    pub confidence: f32,
    /// True when this frame produced a new measurement.
    pub fresh: bool,
    /// The fourth in-band LPC pole when the frame resolved one (the
    /// posterior inverse's map is built on four formants); held with the
    /// rest. `None` when the frame had only three.
    #[serde(default)]
    pub f4: Option<Formant>,
}

impl FormantTrack {
    pub fn held_default(cfg: &FormantConfig) -> Self {
        Self {
            formants: cfg.default_formants(),
            measured_f0: 0.0,
            confidence: 0.0,
            fresh: false,
            f4: None,
        }
    }
}

/// Which published basis a tract parameter vector refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BasisId {
    AdultMale,
    AdultFemale,
    /// The Vocal Tract Lab atlas: the frozen five-subject metric-MRI mean
    /// (`assets/vocal_tract_lab/PROVENANCE.md`), not a Story basis.
    MriAtlasMean,
}

impl BasisId {
    /// The Story basis table, for the two Story bases; the atlas has none.
    pub fn basis(self) -> Option<&'static TractBasis> {
        match self {
            BasisId::AdultMale => Some(&ADULT_MALE),
            BasisId::AdultFemale => Some(&ADULT_FEMALE),
            BasisId::MriAtlasMean => None,
        }
    }

    pub fn of(basis: &'static TractBasis) -> Self {
        if std::ptr::eq(basis, &ADULT_FEMALE) {
            BasisId::AdultFemale
        } else {
            BasisId::AdultMale
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            BasisId::AdultMale => "adult male",
            BasisId::AdultFemale => "adult female",
            BasisId::MriAtlasMean => "MRI atlas mean",
        }
    }
}

/// Which tract model a parameter vector or area function belongs to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TractModelId {
    /// Story (2018) two-mode area function on a published basis.
    #[default]
    StoryTwoMode,
    /// The Vocal Tract Lab four-mode log-area PCA atlas
    /// (`vt3d-frozen-mri-pca-v0.7.0`).
    MriPca4,
}

impl TractModelId {
    pub fn name(self) -> &'static str {
        match self {
            TractModelId::StoryTwoMode => "story_two_mode",
            TractModelId::MriPca4 => "mri_pca4",
        }
    }
}

/// Model-specific tract parameters. `valid` is false when the frame did
/// not clear the inversion's gates (or the posterior abstained) — the
/// coefficients then still hold the backend's last state, for a HELD view.
///
/// Two backends write this: the Story grid inverse fills `q1`/`q2` (and
/// mirrors them into `modes[..2]`); the posterior inverse fills `modes`
/// (four atlas coefficients, in standard deviations) with its confidence
/// and abstention, and leaves `q1`/`q2` at 0.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TractParams {
    pub model: TractModelId,
    pub q1: f32,
    pub q2: f32,
    /// The model's coefficient vector; `n_modes` of them are meaningful.
    pub modes: [f32; MAX_MODES],
    pub n_modes: usize,
    /// The backend's own uncertainty measure, when it has one: the
    /// posterior's relative area standard deviation. The grid backend has
    /// none (`invert` returns only weighted coefficients) — `None`, never a
    /// made-up number.
    pub uncertainty: Option<f32>,
    /// The posterior's filtered confidence (0..1); `None` from a backend
    /// that does not compute one.
    pub confidence: Option<f32>,
    /// The posterior abstained on this frame (decaying toward the mean).
    pub abstained: bool,
    pub reason: AbstainReason,
    pub valid: bool,
    pub basis: BasisId,
    /// Slow estimate of the singer's tract length, cm, once measured.
    pub vtl_est_cm: Option<f32>,
}

impl Default for TractParams {
    fn default() -> Self {
        Self {
            model: TractModelId::StoryTwoMode,
            q1: 0.0,
            q2: 0.0,
            modes: [0.0; MAX_MODES],
            n_modes: 2,
            uncertainty: None,
            confidence: None,
            abstained: false,
            reason: AbstainReason::None,
            valid: false,
            basis: BasisId::AdultMale,
            vtl_est_cm: None,
        }
    }
}

/// The area function: `sections` equal-length sections from glottis to
/// lips (`N_SECTIONS` for the Story model, 32 for the atlas; the arrays
/// beyond `sections` are 0), as the diameters the display renders and the
/// areas the resonance solver takes. `live` mirrors `TractParams::valid`.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AreaFunction {
    pub model: TractModelId,
    pub sections: usize,
    pub section_len_cm: f32,
    #[serde(with = "sections_array")]
    pub diameters_cm: [f32; N_SECTIONS],
    #[serde(with = "sections_array")]
    pub areas_cm2: [f32; N_SECTIONS],
    /// The basis's tract length, cm (the model's, not the singer's).
    pub vtl_cm: f32,
    pub basis: BasisId,
    pub live: bool,
}

impl AreaFunction {
    /// The neutral shape of a basis (q1 = q2 = 0), not live.
    pub fn neutral(basis: BasisId) -> Self {
        Self::from_coefficients(basis, 0.0, 0.0, false)
    }

    /// A Story shape. The atlas basis has no Story table: a caller that
    /// asks for one gets the neutral adult-male shape flagged not live,
    /// and the stages refuse the mismatch before it gets here.
    pub fn from_coefficients(basis: BasisId, q1: f32, q2: f32, live: bool) -> Self {
        let Some(b) = basis.basis() else {
            let mut n = Self::from_coefficients(BasisId::AdultMale, 0.0, 0.0, false);
            n.basis = basis;
            return n;
        };
        Self {
            model: TractModelId::StoryTwoMode,
            sections: N_SECTIONS,
            section_len_cm: b.vtl_cm / N_SECTIONS as f32,
            diameters_cm: tract::diameters(b, q1, q2),
            areas_cm2: tract::area_function(b, q1, q2),
            vtl_cm: b.vtl_cm,
            basis,
            live,
        }
    }
}

/// The lumen surface mesh for one frame: the atlas mesh morphed to the
/// area function, its vertex normals, and the same mesh expanded by the
/// uncertainty envelope. `frame` states the coordinate frame the vertices
/// are in (`atlas::lumen::FRAME`: the renderer's mm frame; the transform
/// to the VTL canonical frame is unknown and is said so).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TractGeometry {
    pub model_id: String,
    pub frame: String,
    pub sections: usize,
    pub angular: usize,
    pub vertex_count: usize,
    /// `vertex_count × 3`, mm.
    pub vertices_mm: Vec<f32>,
    /// `vertex_count × 3`, unit normals.
    pub normals: Vec<f32>,
    /// `vertex_count × 3`, mm: the uncertainty-expanded mesh.
    pub uncertainty_vertices_mm: Vec<f32>,
    /// `sections × 3`, mm.
    pub centerline_mm: Vec<f32>,
    /// Vertex indices, three per triangle.
    pub triangles: Vec<u16>,
    pub relative_area_std: f32,
    pub live: bool,
}

impl TractGeometry {
    /// The mesh at its stored (mean) shape, not live — the preallocated
    /// wire, sized once from the compiled-in asset.
    pub fn preallocated() -> Result<Self, StageError> {
        let l = crate::atlas::lumen::shared().map_err(StageError::Init)?;
        let n = l.vertex_count * 3;
        let mut vertices_mm = vec![0.0; n];
        l.morph_into(&l.reference_area_cm2, &mut vertices_mm)
            .map_err(StageError::Init)?;
        let mut normals = vec![0.0; n];
        crate::atlas::lumen::vertex_normals_into(&vertices_mm, &l.triangle_indices, &mut normals)
            .map_err(StageError::Init)?;
        Ok(Self {
            model_id: l.model_id.into(),
            frame: crate::atlas::lumen::FRAME.into(),
            sections: l.sections,
            angular: l.angular,
            vertex_count: l.vertex_count,
            uncertainty_vertices_mm: vertices_mm.clone(),
            vertices_mm,
            normals,
            centerline_mm: l.centerline_mm.clone(),
            triangles: l.triangle_indices.clone(),
            relative_area_std: 0.0,
            live: false,
        })
    }
}

/// One detected voice in a `NoteSet`.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DetectedNote {
    pub midi: i32,
    /// Parabola-refined fundamental on the detection spectrum, Hz.
    pub f0_hz: f32,
    /// Detector confidence: smoothed salience / threshold (≥ 1 active).
    pub salience: f32,
}

/// Coral's per-detection-frame note sets (`multi_f0` stage): the active
/// MIDI numbers over the display range, and the rehearsal voices (the
/// same detector bound to sung fundamentals, ≤ `choir_detector.harmony_max_hz`)
/// with their sub-bin pitch and confidence. Vectors are sized once at
/// `init` to the detector's note count and only cleared per hop.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NoteSet {
    pub active_midi: Vec<i32>,
    pub voices: Vec<DetectedNote>,
    pub frame_index: u64,
}

/// Coral's SATB labels for the `NoteSet` voices (`satb` stage), in the
/// voices' order.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SectionLabels {
    pub labels: Vec<crate::choir::labeler::SectionLabel>,
    pub frame_index: u64,
}

/// One preallocated wire slot. The runner owns one per stage output plus
/// the source frame; stages read inputs and write their output in place.
// The variants differ in size by design: wires are preallocated once and
// cloned only into tap messages, so the largest variant costs nothing per hop.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Wire {
    AudioFrame(AudioFrame),
    Spectrum(Spectrum),
    F0Track(F0Track),
    HarmonicSeries(HarmonicSeries),
    VoiceMetrics(VoiceMetrics),
    FormantTrack(FormantTrack),
    TractParams(TractParams),
    AreaFunction(AreaFunction),
    TractGeometry(TractGeometry),
    NoteSet(NoteSet),
    SectionLabels(SectionLabels),
}

impl Wire {
    pub fn wire_type(&self) -> WireType {
        match self {
            Wire::AudioFrame(_) => WireType::AudioFrame,
            Wire::Spectrum(_) => WireType::Spectrum,
            Wire::F0Track(_) => WireType::F0Track,
            Wire::HarmonicSeries(_) => WireType::HarmonicSeries,
            Wire::VoiceMetrics(_) => WireType::VoiceMetrics,
            Wire::FormantTrack(_) => WireType::FormantTrack,
            Wire::TractParams(_) => WireType::TractParams,
            Wire::AreaFunction(_) => WireType::AreaFunction,
            Wire::TractGeometry(_) => WireType::TractGeometry,
            Wire::NoteSet(_) => WireType::NoteSet,
            Wire::SectionLabels(_) => WireType::SectionLabels,
        }
    }

    /// A preallocated slot for `t`. Fails loud for a table type no Phase-1
    /// stage can produce — there is nothing honest to put in it.
    pub fn preallocate(
        t: WireType,
        frame_samples: usize,
        sample_rate: f32,
        formants: &FormantConfig,
    ) -> Result<Wire, StageError> {
        Ok(match t {
            WireType::AudioFrame => {
                Wire::AudioFrame(AudioFrame::preallocated(frame_samples, sample_rate))
            }
            WireType::Spectrum => Wire::Spectrum(Spectrum::preallocated(
                frame_samples,
                sample_rate,
                crate::config::SpectrogramConfig::DEFAULT.db_floor,
            )),
            WireType::F0Track => Wire::F0Track(F0Track::default()),
            WireType::HarmonicSeries => Wire::HarmonicSeries(HarmonicSeries::default()),
            WireType::VoiceMetrics => Wire::VoiceMetrics(VoiceMetrics::default()),
            WireType::FormantTrack => Wire::FormantTrack(FormantTrack::held_default(formants)),
            WireType::TractParams => Wire::TractParams(TractParams::default()),
            WireType::AreaFunction => Wire::AreaFunction(AreaFunction::neutral(BasisId::AdultMale)),
            WireType::TractGeometry => Wire::TractGeometry(TractGeometry::preallocated()?),
            WireType::NoteSet => Wire::NoteSet(NoteSet {
                active_midi: Vec::with_capacity(128),
                voices: Vec::with_capacity(128),
                frame_index: 0,
            }),
            WireType::SectionLabels => Wire::SectionLabels(SectionLabels {
                labels: Vec::with_capacity(128),
                frame_index: 0,
            }),
            other => {
                return Err(StageError::Init(format!(
                    "no payload for wire type {other}; no stage produces it yet"
                )));
            }
        })
    }
}

/// A payload that lives in a [`Wire`].
pub trait WireValue: Sized + Clone + Send + 'static {
    const TYPE: WireType;
    fn from_wire(w: &Wire) -> Option<&Self>;
    fn from_wire_mut(w: &mut Wire) -> Option<&mut Self>;
}

macro_rules! wire_value {
    ($ty:ident) => {
        impl WireValue for $ty {
            const TYPE: WireType = WireType::$ty;
            fn from_wire(w: &Wire) -> Option<&Self> {
                match w {
                    Wire::$ty(v) => Some(v),
                    _ => None,
                }
            }
            fn from_wire_mut(w: &mut Wire) -> Option<&mut Self> {
                match w {
                    Wire::$ty(v) => Some(v),
                    _ => None,
                }
            }
        }
    };
}
wire_value!(AudioFrame);
wire_value!(Spectrum);
wire_value!(F0Track);
wire_value!(HarmonicSeries);
wire_value!(VoiceMetrics);
wire_value!(FormantTrack);
wire_value!(TractParams);
wire_value!(AreaFunction);
wire_value!(TractGeometry);
wire_value!(NoteSet);
wire_value!(SectionLabels);

/// A stage's input: one wire value, or a tuple of them, borrowed from the
/// runner's wire slots for the duration of `process`.
pub trait FromWires<'a>: Sized {
    /// The wire types, in the order `from_wires` expects the indices.
    const TYPES: &'static [WireType];
    fn from_wires(wires: &'a [Wire], idx: &[usize]) -> Result<Self, StageError>;
}

fn take<'a, A: WireValue>(
    wires: &'a [Wire],
    idx: &[usize],
    slot: usize,
) -> Result<&'a A, StageError> {
    let i = *idx.get(slot).ok_or_else(|| {
        StageError::Process(format!("input slot {slot} ({}) was not wired", A::TYPE))
    })?;
    let w = wires
        .get(i)
        .ok_or_else(|| StageError::Process(format!("input slot {slot}: wire {i} out of range")))?;
    A::from_wire(w).ok_or(StageError::WrongInput {
        expected: A::TYPE,
        got: w.wire_type(),
    })
}

impl<'a, A: WireValue> FromWires<'a> for &'a A {
    const TYPES: &'static [WireType] = &[A::TYPE];
    fn from_wires(wires: &'a [Wire], idx: &[usize]) -> Result<Self, StageError> {
        take::<A>(wires, idx, 0)
    }
}

impl<'a, A: WireValue, B: WireValue> FromWires<'a> for (&'a A, &'a B) {
    const TYPES: &'static [WireType] = &[A::TYPE, B::TYPE];
    fn from_wires(wires: &'a [Wire], idx: &[usize]) -> Result<Self, StageError> {
        Ok((take::<A>(wires, idx, 0)?, take::<B>(wires, idx, 1)?))
    }
}

impl<'a, A: WireValue, B: WireValue, C: WireValue> FromWires<'a> for (&'a A, &'a B, &'a C) {
    const TYPES: &'static [WireType] = &[A::TYPE, B::TYPE, C::TYPE];
    fn from_wires(wires: &'a [Wire], idx: &[usize]) -> Result<Self, StageError> {
        Ok((
            take::<A>(wires, idx, 0)?,
            take::<B>(wires, idx, 1)?,
            take::<C>(wires, idx, 2)?,
        ))
    }
}

/// serde for the 44-section arrays (serde's array impls stop at 32).
mod sections_array {
    use super::N_SECTIONS;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &[f32; N_SECTIONS], s: S) -> Result<S::Ok, S::Error> {
        v.as_slice().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[f32; N_SECTIONS], D::Error> {
        let v: Vec<f32> = Vec::deserialize(d)?;
        v.try_into().map_err(|v: Vec<f32>| {
            serde::de::Error::custom(format!("expected {N_SECTIONS} sections, got {}", v.len()))
        })
    }
}
