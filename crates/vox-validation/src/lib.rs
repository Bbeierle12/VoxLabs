//! `vox-validation` — the provenance round-trip (Plan v3 D11, Phase 2):
//! read a run's provenance record, rebuild the pipeline from the embedded
//! definition, re-run the recorded input, and compare every tap of the
//! recorded run within the D5 tolerance bands (`[tolerance]` in
//! `pipeline.toml`). Emits the comparison report and the Evidence-format
//! summary the Engineering Console's bundle carries.
//!
//! The recorded run is the harness's `voxlab run` output: a
//! `<stem>.provenance.json` and a `<stem>.taps.jsonl` side by side.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use vox_core::audio_file;
use vox_core::config::ToleranceConfig;
use vox_core::hash::sha256_hex;
use vox_core::pipeline::compare::{CompareReport, Comparer};
use vox_core::pipeline::offline::run_offline;
use vox_core::pipeline::provenance::{ProvenanceRecord, params_digest};
use vox_core::pipeline::types::Wire;

pub use vox_core::pipeline::record::TapLine;

/// The round-trip's outcome.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub schema_version: String,
    pub provenance_path: String,
    pub input_path: String,
    /// The recorded input digest matched the file on disk.
    pub input_digest_matched: bool,
    /// The rebuilt pipeline's parameter digest matched the record.
    pub params_digest_matched: bool,
    pub recorded_hops: u64,
    pub rerun_hops: u64,
    pub compare: CompareReport,
    pub pass: bool,
}

pub const SCHEMA: &str = "voxlabs.validation/1.0";

#[derive(Debug)]
pub struct ValidationError(pub String);

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ValidationError {}

fn err<E: std::fmt::Display>(e: E) -> ValidationError {
    ValidationError(e.to_string())
}

/// Records one run of `mode` over an audio file: decodes, resamples to
/// `sr`, runs the pipeline offline, writes `<out>/<stem>.taps.jsonl` and
/// `<out>/<stem>.provenance.json` (the input path recorded relative to
/// `out` when it can be, so a moved results folder still validates).
/// Returns the hop count and the provenance path.
pub fn record_run(
    mode: &vox_core::pipeline::PipelineDefinition,
    path: &Path,
    out: &Path,
    sr: f32,
) -> Result<(u64, PathBuf), ValidationError> {
    use std::io::Write;
    use vox_core::pipeline::provenance::{InputProvenance, ProvenanceRecord};
    let bytes = fs::read(path).map_err(err)?;
    let dec = audio_file::decode(path).map_err(err)?;
    let samples = audio_file::resample(&dec.samples, dec.sample_rate as f32, sr);
    fs::create_dir_all(out).map_err(err)?;
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("input")
        .to_string();
    let taps_path = out.join(format!("{stem}.taps.jsonl"));
    let mut taps_file = std::io::BufWriter::new(fs::File::create(&taps_path).map_err(err)?);
    let names: Vec<String> = mode.stages.iter().map(|s| s.name.clone()).collect();
    vox_core::room::reset();
    let mut io_error: Option<std::io::Error> = None;
    let (pipeline, hops) = run_offline(mode, sr, &samples, |h| {
        if io_error.is_some() {
            return;
        }
        let mut taps: BTreeMap<String, Wire> = BTreeMap::new();
        for &idx in h.taps {
            taps.insert(names[idx].clone(), h.wires[idx + 1].clone());
        }
        let line = TapLine { hop: h.hop, taps };
        let r = serde_json::to_writer(&mut taps_file, &line)
            .map_err(std::io::Error::other)
            .and_then(|()| taps_file.write_all(b"\n"));
        if let Err(e) = r {
            io_error = Some(e);
        }
    })
    .map_err(err)?;
    if let Some(e) = io_error {
        return Err(err(e));
    }
    taps_file.flush().map_err(err)?;
    let rel = relative_path(out, path).unwrap_or_else(|| path.display().to_string());
    let record = ProvenanceRecord::for_pipeline(
        mode,
        &pipeline,
        InputProvenance::file(&rel, &bytes, sr, samples.len()),
    );
    let provenance_path = out.join(format!("{stem}.provenance.json"));
    fs::write(&provenance_path, record.to_json()).map_err(err)?;
    Ok((hops, provenance_path))
}

/// `to` relative to `from` (both canonicalized); `None` when either cannot
/// be resolved.
fn relative_path(from: &Path, to: &Path) -> Option<String> {
    let from = fs::canonicalize(from).ok()?;
    let to = fs::canonicalize(to).ok()?;
    let mut f = from.components().peekable();
    let mut t = to.components().peekable();
    while let (Some(a), Some(b)) = (f.peek(), t.peek()) {
        if a == b {
            f.next();
            t.next();
        } else {
            break;
        }
    }
    let mut rel = PathBuf::new();
    for _ in f {
        rel.push("..");
    }
    for c in t {
        rel.push(c);
    }
    Some(rel.display().to_string())
}

/// The taps file that belongs to a provenance file (`x.provenance.json`
/// → `x.taps.jsonl`).
pub fn taps_path_for(provenance: &Path) -> PathBuf {
    let name = provenance
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let stem = name.strip_suffix(".provenance.json").unwrap_or(name);
    provenance.with_file_name(format!("{stem}.taps.jsonl"))
}

/// Reads a `.taps.jsonl` file.
pub fn read_taps(path: &Path) -> Result<Vec<TapLine>, ValidationError> {
    let text = fs::read_to_string(path).map_err(err)?;
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<TapLine>(l).map_err(err))
        .collect()
}

/// Runs the round-trip for one recorded run. The input path in the record
/// is resolved relative to the record's directory when it is not absolute.
pub fn round_trip(provenance_path: &Path) -> Result<ValidationReport, ValidationError> {
    let record_text = fs::read_to_string(provenance_path).map_err(err)?;
    let record = ProvenanceRecord::from_json(&record_text).map_err(ValidationError)?;
    let recorded = read_taps(&taps_path_for(provenance_path))?;

    let input_rel = record
        .input
        .path
        .as_deref()
        .ok_or_else(|| ValidationError("the record has no input file".into()))?;
    let mut input_path = PathBuf::from(input_rel);
    if input_path.is_relative()
        && let Some(dir) = provenance_path.parent()
    {
        input_path = dir.join(&input_path);
    }
    let bytes = fs::read(&input_path).map_err(|e| err(format!("{}: {e}", input_path.display())))?;
    let input_digest_matched = record.input.sha256.as_deref() == Some(sha256_hex(&bytes).as_str());

    let decoded = audio_file::decode(&input_path).map_err(err)?;
    let sr = record.input.sample_rate_hz;
    let samples = audio_file::resample(&decoded.samples, decoded.sample_rate as f32, sr);

    vox_core::room::reset();
    let stage_names: Vec<String> = record
        .definition
        .stages
        .iter()
        .map(|s| s.name.clone())
        .collect();
    let mut comparer = Comparer::new(ToleranceConfig::DEFAULT);
    let mut rerun_hops = 0u64;
    let mut missing = 0u64;
    let (pipeline, _) = run_offline(&record.definition, sr, &samples, |h| {
        rerun_hops += 1;
        let Some(line) = recorded.get(h.hop as usize) else {
            missing += 1;
            return;
        };
        let mut a = Vec::with_capacity(h.taps.len());
        let mut b = Vec::with_capacity(h.taps.len());
        for &idx in h.taps {
            let name = &stage_names[idx];
            let Some(rec) = line.taps.get(name) else {
                missing += 1;
                continue;
            };
            a.push(h.wires[idx + 1].clone());
            b.push(rec.clone());
        }
        comparer.hop(&a, &b);
    })
    .map_err(err)?;
    let params_digest_matched = params_digest(&pipeline.params) == record.params_sha256;
    let compare = comparer.finish();
    let pass = input_digest_matched
        && params_digest_matched
        && missing == 0
        && rerun_hops == recorded.len() as u64
        && compare.within_bands();
    Ok(ValidationReport {
        schema_version: SCHEMA.into(),
        provenance_path: provenance_path.display().to_string(),
        input_path: input_path.display().to_string(),
        input_digest_matched,
        params_digest_matched,
        recorded_hops: recorded.len() as u64,
        rerun_hops,
        compare,
        pass,
    })
}

/// Runs the round-trip and writes `<stem>.validation.json` and
/// `<stem>.evidence.json` beside the record.
pub fn validate_and_write(provenance_path: &Path) -> Result<ValidationReport, ValidationError> {
    let report = round_trip(provenance_path)?;
    let name = provenance_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let stem = name.strip_suffix(".provenance.json").unwrap_or(name);
    let report_json = serde_json::to_value(&report).map_err(err)?;
    fs::write(
        provenance_path.with_file_name(format!("{stem}.validation.json")),
        serde_json::to_string_pretty(&report_json).map_err(err)?,
    )
    .map_err(err)?;
    let evidence = vox_core::diagnostics::evidence::assemble(
        &vox_core::diagnostics::evidence::EvidenceInputs {
            self_tests: &[],
            provenance: serde_json::from_str::<serde_json::Value>(
                &fs::read_to_string(provenance_path).map_err(err)?,
            )
            .ok()
            .as_ref(),
            calibrated: false,
            validation: Some((&report_json, report.pass)),
            last_pipeline_report: None,
        },
    );
    fs::write(
        provenance_path.with_file_name(format!("{stem}.evidence.json")),
        serde_json::to_string_pretty(&vox_core::diagnostics::evidence::to_json(&evidence))
            .map_err(err)?,
    )
    .map_err(err)?;
    Ok(report)
}
