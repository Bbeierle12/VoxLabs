//! Recorded runs as files: one `TapLine` per hop in `<stem>.taps.jsonl`
//! beside a provenance record — what `voxlab run` writes and
//! `vox-validation` reads — and the compiled-in fixture runs both sides
//! of a cross-target comparison record (Plan v3 D5, Phase 5a): the phone
//! records `fixture-<mode>.taps.jsonl` from the Engineering Console, the
//! host records the same from `voxlab fixture-taps`, and
//! `compare_tap_files` reports every wire's worst delta so the
//! `[tolerance]` bands can be set from data.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::compare::{CompareReport, Comparer};
use super::definition::PipelineDefinition;
use super::offline::run_offline;
use super::provenance::{InputProvenance, ProvenanceRecord};
use super::types::Wire;
use crate::config::ToleranceConfig;

/// One line of a `.taps.jsonl` file: the hop and the tapped wires by stage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TapLine {
    pub hop: u64,
    pub taps: BTreeMap<String, Wire>,
}

/// The fixture rate every mode is recorded at (the Pixel's capture rate).
pub const FIXTURE_SR: f32 = 48_000.0;
/// Fixture length, seconds.
pub const FIXTURE_SECS: f32 = 1.0;

/// The deterministic signal a mode's fixture run uses: the contract
/// checks' vowel for the voice modes, a synthetic SATB chord for `choir`.
pub fn fixture_signal(mode: &str) -> Result<Vec<f32>, String> {
    let n = (FIXTURE_SR * FIXTURE_SECS) as usize;
    if mode == "choir" {
        use crate::choir::labeler::Section;
        use crate::choir::synth::{ChordSpec, VoiceSpec, synthesize_choir_chord};
        let mut spec = ChordSpec::new(
            f64::from(FIXTURE_SR),
            f64::from(FIXTURE_SECS),
            vec![
                VoiceSpec::new(Section::B, 48),
                VoiceSpec::new(Section::T, 55),
                VoiceSpec::new(Section::A, 64),
                VoiceSpec::new(Section::S, 72),
            ],
        );
        spec.seed = 7;
        return Ok(synthesize_choir_chord(&spec)?.samples);
    }
    Ok(super::contract::vowel(120.0, 3.86, 1.35, n))
}

fn samples_bytes(samples: &[f32]) -> Vec<u8> {
    samples.iter().flat_map(|s| s.to_le_bytes()).collect()
}

/// Runs `def` over `samples`, writing the taps and the provenance record.
/// Returns the hop count.
pub fn record_taps(
    def: &PipelineDefinition,
    sr: f32,
    samples: &[f32],
    input_name: &str,
    taps_path: &Path,
    provenance_path: &Path,
) -> Result<u64, String> {
    let mut file = std::io::BufWriter::new(fs::File::create(taps_path).map_err(|e| e.to_string())?);
    let names: Vec<String> = def.stages.iter().map(|s| s.name.clone()).collect();
    crate::room::reset();
    let mut io_error: Option<String> = None;
    let (pipeline, hops) = run_offline(def, sr, samples, |h| {
        if io_error.is_some() {
            return;
        }
        let mut taps: BTreeMap<String, Wire> = BTreeMap::new();
        for &idx in h.taps {
            taps.insert(names[idx].clone(), h.wires[idx + 1].clone());
        }
        let line = TapLine { hop: h.hop, taps };
        let r = serde_json::to_writer(&mut file, &line)
            .map_err(|e| e.to_string())
            .and_then(|()| file.write_all(b"\n").map_err(|e| e.to_string()));
        if let Err(e) = r {
            io_error = Some(e);
        }
    })
    .map_err(|e| e.to_string())?;
    if let Some(e) = io_error {
        return Err(e);
    }
    file.flush().map_err(|e| e.to_string())?;
    let record = ProvenanceRecord::for_pipeline(
        def,
        &pipeline,
        InputProvenance::file(input_name, &samples_bytes(samples), sr, samples.len()),
    );
    fs::write(provenance_path, record.to_json()).map_err(|e| e.to_string())?;
    Ok(hops)
}

/// Records `mode`'s fixture run into `dir` as `fixture-<mode>.taps.jsonl`
/// and `fixture-<mode>.provenance.json`.
pub fn record_fixture_taps(mode: &str, dir: &Path) -> Result<(PathBuf, PathBuf, u64), String> {
    let def = PipelineDefinition::by_name_or_path(mode).map_err(|e| format!("{mode}: {e}"))?;
    let samples = fixture_signal(mode)?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let taps = dir.join(format!("fixture-{mode}.taps.jsonl"));
    let prov = dir.join(format!("fixture-{mode}.provenance.json"));
    let hops = record_taps(
        &def,
        FIXTURE_SR,
        &samples,
        &format!("fixture:{mode}"),
        &taps,
        &prov,
    )?;
    Ok((taps, prov, hops))
}

/// Reads a `.taps.jsonl` file.
pub fn read_tap_lines(path: &Path) -> Result<Vec<TapLine>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<TapLine>(l).map_err(|e| e.to_string()))
        .collect()
}

/// Compares two tap files hop by hop (same stage names) within the bands.
pub fn compare_tap_files(
    a: &Path,
    b: &Path,
    tol: ToleranceConfig,
) -> Result<CompareReport, String> {
    let la = read_tap_lines(a)?;
    let lb = read_tap_lines(b)?;
    if la.len() != lb.len() {
        return Err(format!("{} hops vs {} hops", la.len(), lb.len()));
    }
    let mut c = Comparer::new(tol);
    for (x, y) in la.iter().zip(&lb) {
        let mut wa = Vec::new();
        let mut wb = Vec::new();
        for (name, w) in &x.taps {
            let Some(v) = y.taps.get(name) else {
                return Err(format!("hop {}: stage `{name}` missing on one side", x.hop));
            };
            wa.push(w.clone());
            wb.push(v.clone());
        }
        c.hop(&wa, &wb);
    }
    Ok(c.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fixture_run_records_and_compares_with_itself_inside_the_bands() {
        let dir = std::env::temp_dir().join(format!("vox-record-{}", std::process::id()));
        let (taps, prov, hops) = record_fixture_taps("calibrate", &dir).unwrap();
        assert!(hops > 10);
        assert!(taps.is_file() && prov.is_file());
        let lines = read_tap_lines(&taps).unwrap();
        assert_eq!(lines.len() as u64, hops);
        let r = compare_tap_files(&taps, &taps, ToleranceConfig::DEFAULT).unwrap();
        assert!(r.within_bands());
        assert_eq!(r.hops_compared, hops);
        assert!(fixture_signal("choir").unwrap().len() == 48_000);
        let _ = fs::remove_dir_all(dir);
    }
}
