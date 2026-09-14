//! Stage configuration — every tunable number the DSP modules use, defined
//! once here and mirrored in `pipeline.toml` at the repository root.
//!
//! Plan v3 §4: "`const` defaults in code + one `pipeline.toml` per mode. No
//! env vars, no runtime override system." Each stage owns one `*Config`
//! struct declared with [`stage_config!`]; its `DEFAULT` is the value the
//! code runs with today, and the test at the bottom of this file proves that
//! `pipeline.toml` carries exactly the same numbers, so the two can never
//! drift apart silently. Phase 1 threads these structs through `Stage::init`;
//! until then every DSP module reads its own `const CFG: XConfig =
//! XConfig::DEFAULT`.
//!
//! What is *not* here: definitional constants (the 2 in 2πf, the 20 in
//! 20·log10, the 12 semitones of an octave) live in [`consts`] — they are
//! named and documented, but they are mathematics, not settings, and are
//! not in `pipeline.toml`. Published data tables (the Story 2018 tract
//! bases) stay in `tract_data.rs`.
//!
//! Every field carries a doc comment stating its purpose and valid range.
//! Phase 0 rule: no behavior change — the `DEFAULT`s are the literals that
//! were in the code, moved, not retuned.

pub mod consts;
pub mod contour;
pub mod diagnostics;
pub mod fach;
pub mod formants;
pub mod harmonics;
pub mod pitch;
pub mod spatial;
pub mod spectrogram;
pub mod stream;
pub mod synthesis;
pub mod tolerance;
pub mod tract;
pub mod voiceprint;

pub use contour::VibratoConfig;
pub use diagnostics::DiagnosticsConfig;
pub use fach::FachConfig;
pub use formants::{FormantConfig, LpcConfig};
pub use harmonics::{
    CentroidConfig, CppConfig, HarmonicsConfig, HnrConfig, PerturbationConfig, TimbreConfig,
    TuningConfig, VoiceClassConfig,
};
pub use pitch::{NoiseFloorConfig, RoomCalibrationConfig, VoicingConfig, YinConfig};
pub use spatial::SpatialConfig;
pub use spectrogram::{ResampleConfig, SpectrogramConfig};
pub use stream::StreamConfig;
pub use synthesis::SynthesisConfig;
pub use tolerance::ToleranceConfig;
pub use tract::{InverseConfig, TractConfig};
pub use voiceprint::VoiceprintConfig;

/// Every stage's parameters together — the shape of `pipeline.toml`. This is
/// what the pipeline builder hands each stage's `init`; a mode file
/// (`pipelines/live_model.toml`) may override any key under `[params]`.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PipelineParams {
    pub stream: StreamConfig,
    pub yin: YinConfig,
    pub voicing: VoicingConfig,
    pub noise_floor: NoiseFloorConfig,
    pub room_calibration: RoomCalibrationConfig,
    pub lpc: LpcConfig,
    pub formants: FormantConfig,
    pub harmonics: HarmonicsConfig,
    pub hnr: HnrConfig,
    pub perturbation: PerturbationConfig,
    pub cpp: CppConfig,
    pub centroid: CentroidConfig,
    pub timbre: TimbreConfig,
    pub voice_class: VoiceClassConfig,
    pub tuning: TuningConfig,
    pub voiceprint: VoiceprintConfig,
    pub vibrato: VibratoConfig,
    pub spectrogram: SpectrogramConfig,
    pub resample: ResampleConfig,
    pub synthesis: SynthesisConfig,
    pub fach: FachConfig,
    pub spatial: SpatialConfig,
    pub tract: TractConfig,
    pub inverse: InverseConfig,
    pub diagnostics: DiagnosticsConfig,
    pub tolerance: ToleranceConfig,
}

impl PipelineParams {
    /// The code's defaults, section by section; identical to `pipeline.toml`.
    pub const DEFAULT: Self = Self {
        stream: StreamConfig::DEFAULT,
        yin: YinConfig::DEFAULT,
        voicing: VoicingConfig::DEFAULT,
        noise_floor: NoiseFloorConfig::DEFAULT,
        room_calibration: RoomCalibrationConfig::DEFAULT,
        lpc: LpcConfig::DEFAULT,
        formants: FormantConfig::DEFAULT,
        harmonics: HarmonicsConfig::DEFAULT,
        hnr: HnrConfig::DEFAULT,
        perturbation: PerturbationConfig::DEFAULT,
        cpp: CppConfig::DEFAULT,
        centroid: CentroidConfig::DEFAULT,
        timbre: TimbreConfig::DEFAULT,
        voice_class: VoiceClassConfig::DEFAULT,
        tuning: TuningConfig::DEFAULT,
        voiceprint: VoiceprintConfig::DEFAULT,
        vibrato: VibratoConfig::DEFAULT,
        spectrogram: SpectrogramConfig::DEFAULT,
        resample: ResampleConfig::DEFAULT,
        synthesis: SynthesisConfig::DEFAULT,
        fach: FachConfig::DEFAULT,
        spatial: SpatialConfig::DEFAULT,
        tract: TractConfig::DEFAULT,
        inverse: InverseConfig::DEFAULT,
        diagnostics: DiagnosticsConfig::DEFAULT,
        tolerance: ToleranceConfig::DEFAULT,
    };

    /// The repository's `pipeline.toml`, compiled in so every target — the
    /// Android APK has no source tree — loads the same parameters.
    pub const TOML: &'static str = include_str!("../../pipeline.toml");

    /// Parses a `pipeline.toml` text. Unknown tables or keys are an error,
    /// not a warning: a misspelled key that silently fell back to its
    /// default would be exactly the drift this file exists to prevent.
    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }
}

impl Default for PipelineParams {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// One configuration value as it appears in `pipeline.toml`, used by the
/// parity test to compare a struct's defaults against the file.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f32),
    Float64(f64),
    Floats(Vec<f32>),
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Float(v)
    }
}
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float64(v)
    }
}
impl From<isize> for Value {
    fn from(v: isize) -> Self {
        Value::Int(v as i64)
    }
}
impl From<usize> for Value {
    fn from(v: usize) -> Self {
        Value::Int(v as i64)
    }
}
impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::Int(v as i64)
    }
}
impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Int(v as i64)
    }
}
impl<const N: usize> From<[f32; N]> for Value {
    fn from(v: [f32; N]) -> Self {
        Value::Floats(v.to_vec())
    }
}

/// Declares one stage's configuration struct.
///
/// ```ignore
/// stage_config! {
///     /// Docs for the struct.
///     pub struct YinConfig, section = "yin" {
///         /// Docs for the field: purpose, and valid range.
///         threshold: f32 = 0.12,
///     }
/// }
/// ```
///
/// Generates: the struct (all fields `pub`), `DEFAULT` (a `const`), `SECTION`
/// (the `[table]` name in `pipeline.toml`), a `Default` impl, and `entries()`
/// — the field list as [`Value`]s for the parity test.
macro_rules! stage_config {
    (
        $(#[$meta:meta])*
        pub struct $name:ident, section = $section:literal {
            $(
                $(#[$fmeta:meta])*
                $field:ident : $ty:ty = $default:expr
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
        #[serde(default, deny_unknown_fields)]
        pub struct $name {
            $(
                $(#[$fmeta])*
                pub $field: $ty,
            )*
        }

        impl $name {
            /// The values the code ships with; identical to `pipeline.toml`.
            pub const DEFAULT: Self = Self { $($field: $default,)* };
            /// The `[table]` this struct is written under in `pipeline.toml`.
            pub const SECTION: &'static str = $section;
            /// Field names and values, in declaration order.
            pub fn entries(&self) -> Vec<(&'static str, $crate::config::Value)> {
                vec![ $( (stringify!($field), $crate::config::Value::from(self.$field)), )* ]
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::DEFAULT
            }
        }
    };
}
pub(crate) use stage_config;

/// Every stage config, as `(section, entries)`, for the parity test and for
/// anything that wants to enumerate the whole configuration (provenance in
/// Phase 2 will).
pub fn all_sections() -> Vec<(&'static str, Vec<(&'static str, Value)>)> {
    vec![
        (StreamConfig::SECTION, StreamConfig::DEFAULT.entries()),
        (YinConfig::SECTION, YinConfig::DEFAULT.entries()),
        (VoicingConfig::SECTION, VoicingConfig::DEFAULT.entries()),
        (
            NoiseFloorConfig::SECTION,
            NoiseFloorConfig::DEFAULT.entries(),
        ),
        (
            RoomCalibrationConfig::SECTION,
            RoomCalibrationConfig::DEFAULT.entries(),
        ),
        (LpcConfig::SECTION, LpcConfig::DEFAULT.entries()),
        (FormantConfig::SECTION, FormantConfig::DEFAULT.entries()),
        (HarmonicsConfig::SECTION, HarmonicsConfig::DEFAULT.entries()),
        (HnrConfig::SECTION, HnrConfig::DEFAULT.entries()),
        (
            PerturbationConfig::SECTION,
            PerturbationConfig::DEFAULT.entries(),
        ),
        (CppConfig::SECTION, CppConfig::DEFAULT.entries()),
        (CentroidConfig::SECTION, CentroidConfig::DEFAULT.entries()),
        (TimbreConfig::SECTION, TimbreConfig::DEFAULT.entries()),
        (
            VoiceClassConfig::SECTION,
            VoiceClassConfig::DEFAULT.entries(),
        ),
        (TuningConfig::SECTION, TuningConfig::DEFAULT.entries()),
        (
            VoiceprintConfig::SECTION,
            VoiceprintConfig::DEFAULT.entries(),
        ),
        (VibratoConfig::SECTION, VibratoConfig::DEFAULT.entries()),
        (
            SpectrogramConfig::SECTION,
            SpectrogramConfig::DEFAULT.entries(),
        ),
        (ResampleConfig::SECTION, ResampleConfig::DEFAULT.entries()),
        (SynthesisConfig::SECTION, SynthesisConfig::DEFAULT.entries()),
        (FachConfig::SECTION, FachConfig::DEFAULT.entries()),
        (SpatialConfig::SECTION, SpatialConfig::DEFAULT.entries()),
        (TractConfig::SECTION, TractConfig::DEFAULT.entries()),
        (InverseConfig::SECTION, InverseConfig::DEFAULT.entries()),
        (
            DiagnosticsConfig::SECTION,
            DiagnosticsConfig::DEFAULT.entries(),
        ),
        (ToleranceConfig::SECTION, ToleranceConfig::DEFAULT.entries()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every default in code equals `pipeline.toml` (parsed by the `toml`
    /// crate; unknown keys are rejected by `deny_unknown_fields`), and every
    /// field of every section is present in the file — a missing key would
    /// otherwise deserialize to its default and hide the drift.
    #[test]
    fn pipeline_toml_matches_code_defaults() {
        let parsed = PipelineParams::from_toml(PipelineParams::TOML)
            .unwrap_or_else(|e| panic!("pipeline.toml does not parse: {e}"));
        assert_eq!(
            parsed,
            PipelineParams::DEFAULT,
            "pipeline.toml drifted from the code defaults"
        );

        let table: toml::Table = PipelineParams::TOML
            .parse()
            .expect("pipeline.toml as a table");
        let mut seen = std::collections::BTreeSet::new();
        for (section, entries) in all_sections() {
            assert!(
                seen.insert(section),
                "section [{section}] declared twice in code"
            );
            let sub = table
                .get(section)
                .and_then(toml::Value::as_table)
                .unwrap_or_else(|| panic!("pipeline.toml has no [{section}] table"));
            for (key, _) in &entries {
                assert!(
                    sub.contains_key(*key),
                    "pipeline.toml [{section}] is missing `{key}`"
                );
            }
        }
        for section in table.keys() {
            assert!(
                seen.contains(section.as_str()),
                "pipeline.toml table [{section}] belongs to no config struct"
            );
        }
    }

    #[test]
    fn default_trait_equals_default_const() {
        assert_eq!(YinConfig::default(), YinConfig::DEFAULT);
        assert_eq!(TractConfig::default(), TractConfig::DEFAULT);
        assert_eq!(PipelineParams::default(), PipelineParams::DEFAULT);
    }

    #[test]
    fn unknown_keys_are_rejected() {
        let text = format!("{}\n[yin]\nthreshhold = 0.2\n", "");
        assert!(
            PipelineParams::from_toml(&text).is_err(),
            "typo must not fall back to default"
        );
    }
}
