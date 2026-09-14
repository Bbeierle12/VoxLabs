//! The provenance record (Plan v3 D11): what ran, on what, with which
//! parameters, on which build — written by the runner when a pipeline is
//! built and by the harness for every offline run. `vox-validation` reads
//! it, rebuilds the pipeline from the embedded definition, re-runs the
//! input, and compares every tap within the D5 bands; the Evidence output
//! is built from that.

use serde::{Deserialize, Serialize};

use crate::config::PipelineParams;
use crate::hash::sha256_hex;

use super::builder::Pipeline;
use super::definition::PipelineDefinition;
use super::stage::StreamFormat;

pub const SCHEMA: &str = "voxlabs.provenance/1.0";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    pub schema_version: String,
    pub generated_at_epoch_ms: u64,
    /// The mode file as loaded (name, format, stages, taps, runner limits,
    /// `[params]` overrides) — enough to rebuild the same pipeline.
    pub definition: PipelineDefinition,
    /// SHA-256 of the definition serialized as TOML.
    pub definition_sha256: String,
    /// The format the pipeline was actually built for (the device's real
    /// sample rate substituted).
    pub format: StreamFormat,
    /// Every stage as built: name, backend id, implementation version.
    pub stages: Vec<StageProvenance>,
    /// SHA-256 of the effective parameters (pipeline.toml plus overrides),
    /// serialized as TOML.
    pub params_sha256: String,
    pub build: BuildProvenance,
    pub input: InputProvenance,
    /// The compiled-in data files (the Vocal Tract Lab atlas) with their
    /// digests and their own release-readiness statements, whatever mode
    /// ran — provenance lists what the binary carries.
    #[serde(default)]
    pub data_files: Vec<crate::atlas::DataFileProvenance>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StageProvenance {
    pub name: String,
    pub backend: String,
    pub version: String,
    pub inputs: Vec<String>,
    pub output: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BuildProvenance {
    pub crate_name: String,
    pub crate_version: String,
    pub target_os: String,
    pub target_arch: String,
    pub profile: String,
    /// SHA-256 of the compiled-in `pipeline.toml` text.
    pub pipeline_toml_sha256: String,
}

/// What went in: the live microphone, or a file with its digest.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputProvenance {
    /// `live` or `file`.
    pub kind: String,
    pub path: Option<String>,
    /// SHA-256 of the file's bytes (as read from disk, before decoding).
    pub sha256: Option<String>,
    pub sample_rate_hz: f32,
    /// Mono samples handed to the pipeline (after decode and resample).
    pub samples: Option<usize>,
}

impl InputProvenance {
    pub fn live(sample_rate_hz: f32) -> Self {
        Self {
            kind: "live".into(),
            path: None,
            sha256: None,
            sample_rate_hz,
            samples: None,
        }
    }

    pub fn file(path: &str, bytes: &[u8], sample_rate_hz: f32, samples: usize) -> Self {
        Self {
            kind: "file".into(),
            path: Some(path.into()),
            sha256: Some(sha256_hex(bytes)),
            sample_rate_hz,
            samples: Some(samples),
        }
    }
}

impl StreamFormat {
    fn as_record(&self) -> Self {
        *self
    }
}

impl serde::Serialize for StreamFormat {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("StreamFormat", 3)?;
        st.serialize_field("sample_rate_hz", &self.sample_rate_hz)?;
        st.serialize_field("frame_samples", &self.frame_samples)?;
        st.serialize_field("hop", &self.hop)?;
        st.end()
    }
}

impl<'de> serde::Deserialize<'de> for StreamFormat {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            sample_rate_hz: f32,
            frame_samples: usize,
            hop: usize,
        }
        let r = Raw::deserialize(d)?;
        Ok(StreamFormat {
            sample_rate_hz: r.sample_rate_hz,
            frame_samples: r.frame_samples,
            hop: r.hop,
        })
    }
}

/// Epoch milliseconds; 0 on the web target, which has no system clock here.
fn now_millis() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// SHA-256 of a parameter set serialized as TOML.
pub fn params_digest(params: &PipelineParams) -> String {
    let text = toml::to_string(params).unwrap_or_default();
    sha256_hex(text.as_bytes())
}

pub fn definition_digest(def: &PipelineDefinition) -> String {
    let text = toml::to_string(def).unwrap_or_default();
    sha256_hex(text.as_bytes())
}

impl ProvenanceRecord {
    pub fn for_pipeline(
        def: &PipelineDefinition,
        pipeline: &Pipeline,
        input: InputProvenance,
    ) -> Self {
        let stages = pipeline
            .stages
            .iter()
            .map(|s| {
                let ident = s.stage.ident();
                StageProvenance {
                    name: s.name.clone(),
                    backend: ident.backend.into(),
                    version: ident.version.into(),
                    inputs: s.stage.inputs().iter().map(|t| t.to_string()).collect(),
                    output: s.stage.output().to_string(),
                }
            })
            .collect();
        Self {
            schema_version: SCHEMA.into(),
            generated_at_epoch_ms: now_millis(),
            definition: def.clone(),
            definition_sha256: definition_digest(def),
            format: pipeline.format.as_record(),
            stages,
            params_sha256: params_digest(&pipeline.params),
            build: BuildProvenance {
                crate_name: env!("CARGO_PKG_NAME").into(),
                crate_version: env!("CARGO_PKG_VERSION").into(),
                target_os: std::env::consts::OS.into(),
                target_arch: std::env::consts::ARCH.into(),
                profile: if cfg!(debug_assertions) {
                    "debug".into()
                } else {
                    "release".into()
                },
                pipeline_toml_sha256: sha256_hex(PipelineParams::TOML.as_bytes()),
            },
            input,
            data_files: crate::atlas::data_provenance(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::builder::build;

    #[test]
    fn a_record_round_trips_and_names_every_stage() {
        let def = crate::pipeline::contract::live_model_coarse().unwrap();
        let format = def.format(Some(48_000.0));
        let p = build(&def, format).unwrap();
        let rec = ProvenanceRecord::for_pipeline(
            &def,
            &p,
            InputProvenance::file("fixture.wav", b"bytes", 48_000.0, 96_000),
        );
        assert_eq!(rec.stages.len(), def.stages.len());
        assert_eq!(rec.stages[0].backend, "yin_cpu");
        assert_eq!(rec.stages[0].output, "F0Track");
        assert_eq!(rec.input.sha256.as_deref(), Some(&sha256_hex(b"bytes")[..]));
        let back = ProvenanceRecord::from_json(&rec.to_json()).unwrap();
        assert_eq!(back, rec);
        // The digests pin the definition and the parameters.
        assert_eq!(back.definition_sha256, definition_digest(&def));
        assert_eq!(back.params_sha256, params_digest(&p.params));
        let rebuilt = build(&back.definition, back.format).unwrap();
        assert_eq!(params_digest(&rebuilt.params), back.params_sha256);
    }
}
