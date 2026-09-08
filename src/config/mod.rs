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
pub mod fach;
pub mod formants;
pub mod harmonics;
pub mod pitch;
pub mod spatial;
pub mod spectrogram;
pub mod stream;
pub mod synthesis;
pub mod tract;
pub mod voiceprint;

pub use contour::VibratoConfig;
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
pub use tract::TractConfig;
pub use voiceprint::VoiceprintConfig;

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
        #[derive(Clone, Copy, Debug, PartialEq)]
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
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// The file the code's defaults must match, byte for byte at the number
    /// level. Read at compile time so the test cannot silently pass against
    /// a missing file.
    const PIPELINE_TOML: &str = include_str!("../../pipeline.toml");

    /// A deliberately tiny reader for the subset of TOML `pipeline.toml`
    /// uses: `[table]` headers, `key = <number | [numbers]>` lines, `#`
    /// comments. Anything else fails the test — the file is ours, and a
    /// parsing gap means a syntax the parity check would not see.
    fn parse(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
        let mut out: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        let mut section = String::new();
        for (n, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                section = name.trim().to_string();
                assert!(
                    out.insert(section.clone(), BTreeMap::new()).is_none(),
                    "pipeline.toml:{}: duplicate table [{section}]",
                    n + 1
                );
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .unwrap_or_else(|| panic!("pipeline.toml:{}: not `key = value`: {raw}", n + 1));
            assert!(
                !section.is_empty(),
                "pipeline.toml:{}: key before any table",
                n + 1
            );
            let prev = out
                .get_mut(&section)
                .unwrap()
                .insert(key.trim().to_string(), value.trim().to_string());
            assert!(
                prev.is_none(),
                "pipeline.toml:{}: duplicate key {key}",
                n + 1
            );
        }
        out
    }

    fn parse_f32(s: &str) -> f32 {
        s.replace('_', "")
            .parse::<f32>()
            .unwrap_or_else(|e| panic!("not a number: {s:?} ({e})"))
    }

    fn matches(text: &str, value: &Value) -> bool {
        match value {
            Value::Int(i) => text.replace('_', "").parse::<i64>().ok() == Some(*i),
            Value::Float(f) => parse_f32(text) == *f,
            Value::Float64(f) => text.replace('_', "").parse::<f64>().ok() == Some(*f),
            Value::Floats(fs) => {
                let inner = text
                    .strip_prefix('[')
                    .and_then(|t| t.strip_suffix(']'))
                    .unwrap_or_else(|| panic!("not an array: {text}"));
                let parsed: Vec<f32> = inner
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(parse_f32)
                    .collect();
                parsed == *fs
            }
        }
    }

    /// Every default in code appears in `pipeline.toml` with the same value,
    /// every key in the file belongs to a field, and no table is orphaned.
    #[test]
    fn pipeline_toml_matches_code_defaults() {
        let file = parse(PIPELINE_TOML);
        let mut seen = std::collections::BTreeSet::new();
        for (section, entries) in all_sections() {
            let table = file
                .get(section)
                .unwrap_or_else(|| panic!("pipeline.toml has no [{section}] table"));
            assert!(
                seen.insert(section),
                "section [{section}] declared twice in code"
            );
            for (key, value) in &entries {
                let text = table.get(*key).unwrap_or_else(|| {
                    panic!("pipeline.toml [{section}] is missing `{key}` (code default {value:?})")
                });
                assert!(
                    matches(text, value),
                    "pipeline.toml [{section}] {key} = {text} but the code default is {value:?}"
                );
            }
            for key in table.keys() {
                assert!(
                    entries.iter().any(|(k, _)| k == key),
                    "pipeline.toml [{section}] has `{key}`, which no config field declares"
                );
            }
        }
        for section in file.keys() {
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
    }
}
