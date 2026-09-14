//! The per-mode pipeline file (D9): `pipelines/live_model.toml` and its kin.
//!
//! A mode file selects the stream format, the ordered stage list with
//! backend ids, the taps the shell renders, the runner's limits, and any
//! parameter overrides on top of `pipeline.toml`. It is loaded, not
//! generated: the runner builds from what it says. Every numeric value the
//! runner uses lives here or in a `StageConfig` — none in code.

use std::path::Path;

use crate::config::PipelineParams;

use super::stage::StreamFormat;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineDefinition {
    /// The mode this file defines ("live_model", "fingerprint", …).
    pub name: String,
    pub format: FormatDef,
    pub runner: RunnerConfig,
    /// The ordered stage list. Each input must be produced by an earlier
    /// stage (or be the source `AudioFrame`).
    #[serde(rename = "stage")]
    pub stages: Vec<StageDef>,
    /// Stage names whose output the shell receives every hop.
    pub taps: Vec<String>,
    /// Overrides on top of `pipeline.toml`, table for table.
    #[serde(default)]
    pub params: toml::Table,
}

/// Nominal stream format. The shell replaces the sample rate with the
/// device's real rate before building; frame and hop are as written.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatDef {
    pub sample_rate_hz: f32,
    pub frame_samples: usize,
    pub hop: usize,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageDef {
    pub name: String,
    pub backend: String,
}

/// Runner limits and reporting cadence. No defaults in code: a mode file
/// that omits one fails to load.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunnerConfig {
    /// Idle sleep between ring-buffer drains on the worker, ms.
    pub poll_ms: u64,
    /// Bounded tap channel depth, in messages; a full channel drops the
    /// newest and counts it.
    pub tap_capacity: usize,
    /// Log a timing summary every this many hops.
    pub report_every_hops: u64,
    /// Gate: the worst hop must stay under this fraction of the hop budget.
    pub hop_budget_fraction_max: f32,
    /// Gate: mic-to-render latency threshold, ms.
    pub mic_to_render_max_ms: f32,
    /// How long the shell waits for the worker's `init` (grid builds)
    /// before starting audio anyway, ms.
    pub init_timeout_ms: u64,
    /// The shell's not-yet-wrapped per-frame work runs every this many hops
    /// (2 = the pre-pipeline 2048-sample cadence at hop 1024).
    pub legacy_frame_every_hops: u64,
    /// Samples the re-framer reserves for drained-but-unconsumed audio, in
    /// frames; growth beyond it allocates on the worker.
    pub pending_capacity_frames: usize,
}

#[derive(Debug)]
pub enum DefinitionError {
    Parse(toml::de::Error),
    Io(std::io::Error),
    /// A `[params]` override names a table or key no config struct has, or
    /// the merged parameters failed to deserialize.
    Params(String),
    /// A structural problem the schema could not catch.
    Invalid(String),
}

impl std::fmt::Display for DefinitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinitionError::Parse(e) => write!(f, "pipeline definition does not parse: {e}"),
            DefinitionError::Io(e) => write!(f, "pipeline definition unreadable: {e}"),
            DefinitionError::Params(m) => write!(f, "pipeline [params]: {m}"),
            DefinitionError::Invalid(m) => write!(f, "pipeline definition invalid: {m}"),
        }
    }
}

impl std::error::Error for DefinitionError {}

impl PipelineDefinition {
    /// The Live Model mode, compiled in so the APK carries it.
    pub const LIVE_MODEL: &'static str = include_str!("../../pipelines/live_model.toml");

    pub fn live_model() -> Result<Self, DefinitionError> {
        Self::from_toml(Self::LIVE_MODEL)
    }

    /// The Fingerprint mode (enrollment/match), compiled in.
    pub const FINGERPRINT: &'static str = include_str!("../../pipelines/fingerprint.toml");
    /// The Calibrate mode (the room pass), compiled in.
    pub const CALIBRATE: &'static str = include_str!("../../pipelines/calibrate.toml");
    /// Phase 3: the Vocal Tract Lab chain (posterior inverse, MRI-reduced
    /// area function, lumen mesh).
    pub const ATLAS: &'static str = include_str!("../../pipelines/atlas.toml");
    /// Phase 4: Coral's choir branch (Coral STFT, QIFFT, multi-F0, SATB).
    pub const CHOIR: &'static str = include_str!("../../pipelines/choir.toml");
    /// Phase 6: every tap exposed, two inverse backends side by side.
    pub const WORKBENCH: &'static str = include_str!("../../pipelines/workbench.toml");

    /// Every compiled-in mode, by name.
    pub const MODES: &'static [(&'static str, &'static str)] = &[
        ("live_model", Self::LIVE_MODEL),
        ("fingerprint", Self::FINGERPRINT),
        ("calibrate", Self::CALIBRATE),
        ("atlas", Self::ATLAS),
        ("choir", Self::CHOIR),
        ("workbench", Self::WORKBENCH),
    ];

    /// A compiled-in mode by name, or a `.toml` path.
    pub fn by_name_or_path(name: &str) -> Result<Self, DefinitionError> {
        match Self::MODES.iter().find(|(n, _)| *n == name) {
            Some((_, text)) => Self::from_toml(text),
            None => Self::from_path(Path::new(name)),
        }
    }

    pub fn from_toml(text: &str) -> Result<Self, DefinitionError> {
        let def: Self = toml::from_str(text).map_err(DefinitionError::Parse)?;
        def.validate()?;
        Ok(def)
    }

    pub fn from_path(path: &Path) -> Result<Self, DefinitionError> {
        let text = std::fs::read_to_string(path).map_err(DefinitionError::Io)?;
        Self::from_toml(&text)
    }

    fn validate(&self) -> Result<(), DefinitionError> {
        if self.name.is_empty() {
            return Err(DefinitionError::Invalid("`name` is empty".into()));
        }
        if self.format.frame_samples == 0 || self.format.hop == 0 {
            return Err(DefinitionError::Invalid(
                "format.frame_samples and format.hop must be positive".into(),
            ));
        }
        if self.format.hop > self.format.frame_samples {
            return Err(DefinitionError::Invalid(format!(
                "format.hop ({}) exceeds format.frame_samples ({})",
                self.format.hop, self.format.frame_samples
            )));
        }
        if !(self.format.sample_rate_hz.is_finite() && self.format.sample_rate_hz > 0.0) {
            return Err(DefinitionError::Invalid(
                "format.sample_rate_hz must be positive".into(),
            ));
        }
        let r = &self.runner;
        if r.tap_capacity == 0
            || r.report_every_hops == 0
            || r.legacy_frame_every_hops == 0
            || r.pending_capacity_frames == 0
        {
            return Err(DefinitionError::Invalid(
                "runner.tap_capacity, report_every_hops, legacy_frame_every_hops and \
                 pending_capacity_frames must be positive"
                    .into(),
            ));
        }
        if !(r.hop_budget_fraction_max > 0.0 && r.hop_budget_fraction_max <= 1.0) {
            return Err(DefinitionError::Invalid(
                "runner.hop_budget_fraction_max must be in (0, 1]".into(),
            ));
        }
        Ok(())
    }

    /// The stream format to build for: the file's frame and hop, and the
    /// device's rate when the shell knows it.
    pub fn format(&self, actual_sample_rate_hz: Option<f32>) -> StreamFormat {
        StreamFormat {
            sample_rate_hz: actual_sample_rate_hz.unwrap_or(self.format.sample_rate_hz),
            frame_samples: self.format.frame_samples,
            hop: self.format.hop,
        }
    }

    /// `pipeline.toml` with this mode's `[params]` overrides merged over it,
    /// as the typed bundle every stage's `init` receives. Overrides are
    /// checked key by key: a table or key no config struct declares is an
    /// error, not a silently ignored typo.
    pub fn params(&self) -> Result<PipelineParams, DefinitionError> {
        let mut base: toml::Table = PipelineParams::TOML
            .parse()
            .map_err(|e: toml::de::Error| DefinitionError::Params(e.to_string()))?;
        for (section, value) in &self.params {
            let overrides = value.as_table().ok_or_else(|| {
                DefinitionError::Params(format!("[params.{section}] must be a table"))
            })?;
            let target = base
                .get_mut(section)
                .and_then(toml::Value::as_table_mut)
                .ok_or_else(|| {
                    DefinitionError::Params(format!("no stage config section named `{section}`"))
                })?;
            for (key, v) in overrides {
                if !target.contains_key(key) {
                    return Err(DefinitionError::Params(format!(
                        "[{section}] has no key `{key}`"
                    )));
                }
                target.insert(key.clone(), v.clone());
            }
        }
        let merged = toml::Value::Table(base);
        merged
            .try_into()
            .map_err(|e: toml::de::Error| DefinitionError::Params(e.to_string()))
    }
}

#[cfg(test)]
mod mode_tests {
    use super::PipelineDefinition;
    use crate::pipeline::builder::build;

    /// Every compiled-in mode file loads and builds (with the coarse grid
    /// where a tract stage is present, for speed).
    #[test]
    fn every_compiled_in_mode_builds() {
        for (name, text) in PipelineDefinition::MODES {
            let text = if text.contains("story_two_mode") {
                format!("{text}\n[params.tract]\ngrid_n = 21\n")
            } else {
                text.to_string()
            };
            let def =
                PipelineDefinition::from_toml(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(&def.name, name);
            let p =
                build(&def, def.format(Some(48_000.0))).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(p.taps.len(), def.taps.len(), "{name}");
        }
        assert!(PipelineDefinition::by_name_or_path("no_such_mode").is_err());
    }
}
