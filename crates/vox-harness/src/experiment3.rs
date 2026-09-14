//! Experiment 3 (Plan v3 Phase 3 gate): held-out (formants, tract
//! parameter) pairs from the forward model, inverted, re-synthesized, and
//! the round-trip error, parameter RMS and per-frame inverse latency
//! reported against the `[validation]` bands. Two backends, the same
//! protocol:
//!
//! - `grid_story`: (q1, q2) uniform over the `[tract]` grid span →
//!   `tract::area_function` → `tract::resonances` → `TractGrid::invert`.
//! - `posterior_pca4`: four atlas coefficients uniform within the model's
//!   limits → `area_from_coefficients` (32 sections) → resampled to the
//!   solver's 44 sections → `tract::resonances_up_to` for F1..F4 (the
//!   lumen centerline as the tract length) → `infer_coefficients` (the
//!   app's clamped linear map, on all four formants as the app feeds it).
//!   The gate is scored on F1..F3 as the plan states it; F4's error is
//!   reported beside it. The forward model is VoxLabs' chain-matrix
//!   solver, not the app's `TubeGrid`, against which the app calibrated
//!   its Jacobian; the report states this and the gate is applied to what
//!   it measures.
//!
//! Frozen with provenance: seed, counts, model ids and digests, bands.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use serde::Serialize;

use vox_core::atlas::{lumen, reduced_model};
use vox_core::config::{PosteriorConfig, TractConfig, ValidationConfig};
use vox_core::pipeline::stages::inverse::shared_grid;
use vox_core::tract::{self, ADULT_MALE, N_SECTIONS};

/// Deterministic xorshift32 — no dependency, reproducible across targets.
struct Rng(u32);

impl Rng {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    fn uniform(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * (self.next_u32() as f32 / u32::MAX as f32)
    }
}

#[derive(Serialize)]
struct Pair {
    index: usize,
    params: Vec<f32>,
    formants_hz: Vec<f32>,
    inverted: Vec<f32>,
    achieved_hz: Vec<f32>,
    rel_error: f32,
    abs_error_hz: f32,
}

#[derive(Serialize, Clone, Debug)]
pub struct BackendReport {
    pub backend: String,
    pub pairs_requested: usize,
    /// Forward model had no solution (resonances out of the sweep band).
    pub unsolved: usize,
    /// The inverse refused (grid distance beyond `tract.invert_max_dist`).
    pub refused: usize,
    pub pairs_evaluated: usize,
    pub rel_error_p50: f32,
    pub rel_error_p95: f32,
    pub abs_error_hz_p50: f32,
    pub abs_error_hz_p95: f32,
    pub param_rms: f32,
    /// F4 round-trip error, Hz, p95, when the backend inverts four formants.
    pub f4_abs_error_hz_p95: Option<f32>,
    pub latency_ms_p50: f32,
    pub latency_ms_p95: f32,
    pub latency_ms_max: f32,
    pub accuracy_gate_met: bool,
    pub latency_gate_met: bool,
    pub pass: bool,
    pub note: String,
}

#[derive(Serialize)]
pub struct Report {
    pub schema_version: String,
    pub seed: u32,
    pub pairs_per_backend: usize,
    pub bands: ValidationConfig,
    pub host: String,
    pub data_files: Vec<vox_core::atlas::DataFileProvenance>,
    pub jacobian: JacobianReport,
    pub backends: Vec<BackendReport>,
}

pub const SCHEMA: &str = "voxlabs.experiment3/1.0";

fn percentile(sorted: &[f32], p: f32) -> f32 {
    if sorted.is_empty() {
        return f32::NAN;
    }
    let i = ((sorted.len() - 1) as f32 * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

/// Worst relative and absolute error over the first `scored` formants.
fn errors(target: &[f32], achieved: &[f32], scored: usize) -> (f32, f32) {
    let mut rel = 0.0f32;
    let mut abs = 0.0f32;
    for (t, a) in target.iter().zip(achieved).take(scored) {
        let d = (a - t).abs();
        abs = abs.max(d);
        rel = rel.max(d / t.max(f32::MIN_POSITIVE));
    }
    (rel, abs)
}

struct Collector {
    pairs: Vec<Pair>,
    rel: Vec<f32>,
    abs: Vec<f32>,
    latency_ms: Vec<f32>,
    f4_abs: Vec<f32>,
    param_sq: f64,
    param_count: usize,
    unsolved: usize,
    refused: usize,
}

impl Collector {
    fn new() -> Self {
        Self {
            pairs: Vec::new(),
            rel: Vec::new(),
            abs: Vec::new(),
            latency_ms: Vec::new(),
            f4_abs: Vec::new(),
            param_sq: 0.0,
            param_count: 0,
            unsolved: 0,
            refused: 0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        index: usize,
        params: Vec<f32>,
        formants: Vec<f32>,
        inverted: Vec<f32>,
        achieved: Vec<f32>,
        latency_ms: f32,
        scored: usize,
    ) {
        let (rel, abs) = errors(&formants, &achieved, scored);
        if formants.len() > scored {
            let (_, abs4) = errors(&formants[scored..], &achieved[scored..], 1);
            self.f4_abs.push(abs4);
        }
        for (p, q) in params.iter().zip(&inverted) {
            self.param_sq += ((p - q) as f64).powi(2);
            self.param_count += 1;
        }
        self.rel.push(rel);
        self.abs.push(abs);
        self.latency_ms.push(latency_ms);
        self.pairs.push(Pair {
            index,
            params,
            formants_hz: formants,
            inverted,
            achieved_hz: achieved,
            rel_error: rel,
            abs_error_hz: abs,
        });
    }

    fn report(
        mut self,
        backend: &str,
        requested: usize,
        bands: &ValidationConfig,
        note: &str,
    ) -> BackendReport {
        self.rel.sort_by(f32::total_cmp);
        self.abs.sort_by(f32::total_cmp);
        self.latency_ms.sort_by(f32::total_cmp);
        self.f4_abs.sort_by(f32::total_cmp);
        let rel95 = percentile(&self.rel, 0.95);
        let abs95 = percentile(&self.abs, 0.95);
        let lat95 = percentile(&self.latency_ms, 0.95);
        let accuracy = !self.rel.is_empty()
            && (rel95 < bands.p95_relative_error_max || abs95 < bands.p95_absolute_error_hz_max);
        let latency = !self.latency_ms.is_empty() && lat95 < bands.inverse_latency_ms_max;
        BackendReport {
            backend: backend.into(),
            pairs_requested: requested,
            unsolved: self.unsolved,
            refused: self.refused,
            pairs_evaluated: self.pairs.len(),
            rel_error_p50: percentile(&self.rel, 0.5),
            rel_error_p95: rel95,
            abs_error_hz_p50: percentile(&self.abs, 0.5),
            abs_error_hz_p95: abs95,
            param_rms: if self.param_count == 0 {
                f32::NAN
            } else {
                (self.param_sq / self.param_count as f64).sqrt() as f32
            },
            f4_abs_error_hz_p95: (!self.f4_abs.is_empty()).then(|| percentile(&self.f4_abs, 0.95)),
            latency_ms_p50: percentile(&self.latency_ms, 0.5),
            latency_ms_p95: lat95,
            latency_ms_max: self.latency_ms.last().copied().unwrap_or(f32::NAN),
            accuracy_gate_met: accuracy,
            latency_gate_met: latency,
            pass: accuracy && latency,
            note: note.into(),
        }
    }
}

fn write_pairs(path: &Path, pairs: &[Pair]) -> anyhow::Result<()> {
    let mut f = std::io::BufWriter::new(fs::File::create(path)?);
    for p in pairs {
        serde_json::to_writer(&mut f, p)?;
        f.write_all(b"\n")?;
    }
    f.flush()?;
    Ok(())
}

/// The atlas forward model: coefficients → 32-section area → resampled to
/// the solver's 44 sections → resonances, with the lumen centerline as
/// the tract length.
struct Forward {
    m: &'static reduced_model::ReducedModel,
    cfg: PosteriorConfig,
    vtl_cm: f32,
    target_pos: Vec<f32>,
    area32: Vec<f32>,
    area44: [f32; N_SECTIONS],
}

impl Forward {
    fn new() -> anyhow::Result<Self> {
        let m = reduced_model::shared().map_err(anyhow::Error::msg)?;
        let l = lumen::shared().map_err(anyhow::Error::msg)?;
        Ok(Self {
            m,
            cfg: PosteriorConfig::DEFAULT,
            vtl_cm: l.centerline_length_mm() / vox_core::config::consts::MM_PER_CM,
            target_pos: (0..N_SECTIONS)
                .map(|i| i as f32 / (N_SECTIONS - 1) as f32)
                .collect(),
            area32: vec![0.0; m.n_sections()],
            area44: [0.0; N_SECTIONS],
        })
    }

    /// F1..F4, or `None` when fewer than four lie under the sweep ceiling.
    fn resonances(&mut self, coef: &[f32]) -> anyhow::Result<Option<[f32; 4]>> {
        self.m
            .area_from_coefficients(coef, &self.cfg, &mut self.area32)
            .map_err(anyhow::Error::msg)?;
        lumen::resample_area_into(
            &self.m.section_position,
            &self.area32,
            &self.target_pos,
            self.cfg.resample_min_area_cm2,
            &mut self.area44,
        )
        .map_err(anyhow::Error::msg)?;
        let mut out = [0.0f32; 4];
        let found = tract::resonances_up_to(
            &self.area44,
            self.vtl_cm,
            ValidationConfig::DEFAULT.forward_sweep_hi_hz,
            &mut out,
        );
        Ok((found == 4).then_some(out))
    }
}

/// The forward model's Jacobian at the atlas mean (central differences,
/// Hz per SD, F1..F4 × modes) beside the JSON's own `jacobian_hz_per_sd`
/// (F1..F4 × modes): how far VoxLabs' solver is from the tube model the
/// app linearized. The Frobenius figure is over F1..F3, the rows the gate
/// scores.
#[derive(Serialize, Clone, Debug)]
pub struct JacobianReport {
    pub step_sd: f32,
    pub app_hz_per_sd: Vec<Vec<f32>>,
    pub voxlabs_hz_per_sd: Vec<Vec<f32>>,
    /// ‖J_vox − J_app‖_F / ‖J_app‖_F over the F1..F3 rows.
    pub relative_frobenius_difference_f1_f3: f32,
}

fn jacobian(bands: &ValidationConfig) -> anyhow::Result<JacobianReport> {
    let mut fwd = Forward::new()?;
    let n = fwd.m.n_modes();
    let h = bands.jacobian_step_sd;
    let mut vox = vec![vec![0.0f32; n]; 4];
    for i in 0..n {
        let mut plus = vec![0.0f32; n];
        let mut minus = vec![0.0f32; n];
        plus[i] = h;
        minus[i] = -h;
        let (Some(fp), Some(fm)) = (fwd.resonances(&plus)?, fwd.resonances(&minus)?) else {
            anyhow::bail!("no resonances at ±{h} SD of mode {i}");
        };
        for (j, row) in vox.iter_mut().enumerate() {
            row[i] = (fp[j] - fm[j]) / (2.0 * h);
        }
    }
    let app = fwd.m.acoustic_inverse.jacobian_hz_per_sd.clone();
    let (mut num, mut den) = (0.0f64, 0.0f64);
    for j in 0..3.min(app.len()) {
        for i in 0..n.min(app[j].len()) {
            num += ((vox[j][i] - app[j][i]) as f64).powi(2);
            den += (app[j][i] as f64).powi(2);
        }
    }
    Ok(JacobianReport {
        step_sd: h,
        app_hz_per_sd: app,
        voxlabs_hz_per_sd: vox,
        relative_frobenius_difference_f1_f3: if den > 0.0 {
            (num / den).sqrt() as f32
        } else {
            f32::NAN
        },
    })
}

/// The posterior's clamped linear map, coefficients sampled within
/// `span` (None: the model's own limits).
fn posterior(
    n: usize,
    seed: u32,
    bands: &ValidationConfig,
    span: Option<f32>,
) -> anyhow::Result<(BackendReport, Vec<Pair>)> {
    let mut fwd = Forward::new()?;
    let m = fwd.m;
    let limits: Vec<[f32; 2]> = m
        .limits()
        .iter()
        .take(m.n_modes())
        .map(|l| match span {
            Some(s) => [l[0].max(-s), l[1].min(s)],
            None => *l,
        })
        .collect();
    let mut rng = Rng(seed.wrapping_mul(2_654_435_761) | 1);
    let mut c = Collector::new();
    for i in 0..n {
        let coef: Vec<f32> = (0..m.n_modes())
            .map(|k| rng.uniform(limits[k][0], limits[k][1]))
            .collect();
        let Some(res) = fwd.resonances(&coef)? else {
            c.unsolved += 1;
            continue;
        };
        let t0 = Instant::now();
        let mut inv = [0.0f32; reduced_model::MAX_MODES];
        m.infer_coefficients(&res, &mut inv);
        let latency = t0.elapsed().as_secs_f32() * 1000.0;
        let inv = inv[..m.n_modes()].to_vec();
        let Some(achieved) = fwd.resonances(&inv)? else {
            c.unsolved += 1;
            continue;
        };
        c.push(i, coef, res.to_vec(), inv, achieved.to_vec(), latency, 3);
    }
    let pairs = std::mem::take(&mut c.pairs);
    let mut keep = Collector::new();
    keep.rel = c.rel;
    keep.abs = c.abs;
    keep.latency_ms = c.latency_ms;
    keep.f4_abs = c.f4_abs;
    keep.param_sq = c.param_sq;
    keep.param_count = c.param_count;
    keep.unsolved = c.unsolved;
    keep.refused = c.refused;
    keep.pairs = Vec::with_capacity(0);
    let evaluated = pairs.len();
    let (name, regime) = match span {
        Some(s) => (
            format!("posterior_pca4 ±{s} SD"),
            format!("coefficients within ±{s} SD of the mean"),
        ),
        None => (
            "posterior_pca4".to_string(),
            "coefficients uniform over the model's ±2 SD limits".to_string(),
        ),
    };
    let mut r = keep.report(
        &name,
        n,
        bands,
        &format!(
            "F1–F3 scored (F4 reported) after inverting F1..F4 through the app's clamped linear formant→mode map, {regime}; forward model tract::resonances_up_to on the atlas area function resampled to 44 sections with the lumen centerline length — not the app's TubeGrid"
        ),
    );
    r.pairs_evaluated = evaluated;
    Ok((r, pairs))
}

/// Runs both backends and writes `inverse_fixtures_<backend>.jsonl` and
/// `inverse_eval.json` under `out`.
pub fn run(out: &Path, n: usize, seed: u32) -> anyhow::Result<Report> {
    let bands = ValidationConfig::DEFAULT;
    fs::create_dir_all(out)?;
    let (story_report, _) = story_with_pairs(
        n,
        seed,
        &bands,
        &out.join("inverse_fixtures_grid_story.jsonl"),
    )?;
    let (posterior_report, pairs) = posterior(n, seed, &bands, None)?;
    write_pairs(&out.join("inverse_fixtures_posterior_pca4.jsonl"), &pairs)?;
    let (local_report, local_pairs) =
        posterior(n, seed, &bands, Some(bands.inverse_local_span_sd))?;
    write_pairs(
        &out.join("inverse_fixtures_posterior_pca4_local.jsonl"),
        &local_pairs,
    )?;
    let report = Report {
        schema_version: SCHEMA.into(),
        seed,
        pairs_per_backend: n,
        bands,
        host: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        data_files: vox_core::atlas::data_provenance(),
        jacobian: jacobian(&bands)?,
        backends: vec![story_report, posterior_report, local_report],
    };
    fs::write(
        out.join("inverse_eval.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    Ok(report)
}

fn story_with_pairs(
    n: usize,
    seed: u32,
    bands: &ValidationConfig,
    path: &Path,
) -> anyhow::Result<(BackendReport, ())> {
    let t = TractConfig::DEFAULT;
    let grid = shared_grid(&ADULT_MALE, t.grid_n);
    let mut rng = Rng(seed | 1);
    let mut c = Collector::new();
    for i in 0..n {
        let q1 = rng.uniform(t.q1_min, t.q1_max);
        let q2 = rng.uniform(t.q2_min, t.q2_max);
        let areas = tract::area_function(&ADULT_MALE, q1, q2);
        let Some(res) = tract::resonances(&areas, ADULT_MALE.vtl_cm) else {
            c.unsolved += 1;
            continue;
        };
        let t0 = Instant::now();
        let inv = grid.invert(res[0], res[1]);
        let latency = t0.elapsed().as_secs_f32() * 1000.0;
        let Some((p1, p2)) = inv else {
            c.refused += 1;
            continue;
        };
        let back = tract::area_function(&ADULT_MALE, p1, p2);
        let Some(achieved) = tract::resonances(&back, ADULT_MALE.vtl_cm) else {
            c.unsolved += 1;
            continue;
        };
        c.push(
            i,
            vec![q1, q2],
            res[..2].to_vec(),
            vec![p1, p2],
            achieved[..2].to_vec(),
            latency,
            2,
        );
    }
    write_pairs(path, &c.pairs)?;
    let evaluated = c.pairs.len();
    c.pairs = Vec::new();
    let mut r = c.report(
        "grid_story",
        n,
        bands,
        "F1–F2 round trip (the grid inverts two formants); forward model tract::resonances on the adult-male Story basis",
    );
    r.pairs_evaluated = evaluated;
    Ok((r, ()))
}

pub fn print(report: &Report) {
    println!(
        "Experiment 3 · seed {} · {} pairs per backend · {}",
        report.seed, report.pairs_per_backend, report.host
    );
    let j = &report.jacobian;
    println!(
        "Jacobian at the mean (Hz/SD, step {} SD): app F1..F3 {:?} · voxlabs {:?} · relative Frobenius difference {:.3}",
        j.step_sd,
        &j.app_hz_per_sd[..j.app_hz_per_sd.len().min(3)],
        j.voxlabs_hz_per_sd,
        j.relative_frobenius_difference_f1_f3
    );
    for b in &report.backends {
        println!(
            "{:<22} {:>5} eval ({} unsolved, {} refused) · rel p50 {:.4} p95 {:.4} · abs p50 {:.1} p95 {:.1} Hz · F4 p95 {} · param RMS {:.3} · latency p50 {:.4} p95 {:.4} max {:.4} ms · accuracy {} · latency {} · {}",
            b.backend,
            b.pairs_evaluated,
            b.unsolved,
            b.refused,
            b.rel_error_p50,
            b.rel_error_p95,
            b.abs_error_hz_p50,
            b.abs_error_hz_p95,
            b.f4_abs_error_hz_p95
                .map(|v| format!("{v:.1} Hz"))
                .unwrap_or_else(|| "—".into()),
            b.param_rms,
            b.latency_ms_p50,
            b.latency_ms_p95,
            b.latency_ms_max,
            if b.accuracy_gate_met {
                "MET"
            } else {
                "NOT MET"
            },
            if b.latency_gate_met { "MET" } else { "NOT MET" },
            if b.pass { "PASS" } else { "FAIL" }
        );
    }
}
