//! Tap-by-tap comparison of two runs within the D5 tolerance bands
//! (`[tolerance]` in `pipeline.toml`). Used by `vox-validation` for the
//! provenance round-trip and, from Phase 5a, for the arm64-vs-x86_64 and
//! backend-vs-backend comparisons. Every difference is reported by wire
//! type with the worst value seen, so the bands can be set from data.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::ToleranceConfig;
use crate::types::VoiceMetrics;

use super::types::{
    AreaFunction, F0Track, FormantTrack, HarmonicSeries, Spectrum, TractGeometry, TractParams,
    Wire, WireType,
};

/// The worst disagreement seen for one wire type, and whether it stayed
/// inside its band.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WireDelta {
    pub compared: u64,
    /// The largest normalized excess over the band (≤ 1 means inside).
    pub worst_ratio: f32,
    /// Human-readable field and value of the worst excess.
    pub worst: String,
    pub outside_band: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CompareReport {
    pub hops_compared: u64,
    pub mismatched_types: u64,
    pub per_type: BTreeMap<String, WireDelta>,
}

impl CompareReport {
    pub fn within_bands(&self) -> bool {
        self.mismatched_types == 0 && self.per_type.values().all(|d| d.outside_band == 0)
    }
}

pub struct Comparer {
    tol: ToleranceConfig,
    report: CompareReport,
}

impl Comparer {
    pub fn new(tol: ToleranceConfig) -> Self {
        Self {
            tol,
            report: CompareReport::default(),
        }
    }

    pub fn finish(self) -> CompareReport {
        self.report
    }

    /// Compares one hop's taps: `a` and `b` must be the same wires in the
    /// same order (a type mismatch is counted, never compared).
    pub fn hop(&mut self, a: &[Wire], b: &[Wire]) {
        self.report.hops_compared += 1;
        for (x, y) in a.iter().zip(b) {
            if x.wire_type() != y.wire_type() {
                self.report.mismatched_types += 1;
                continue;
            }
            let mut worst = Excess::default();
            match (x, y) {
                (Wire::AudioFrame(_), Wire::AudioFrame(_)) => continue,
                (Wire::F0Track(p), Wire::F0Track(q)) => self.f0(p, q, &mut worst),
                (Wire::Spectrum(p), Wire::Spectrum(q)) => self.spectrum(p, q, &mut worst),
                (Wire::HarmonicSeries(p), Wire::HarmonicSeries(q)) => {
                    self.harmonics(p, q, &mut worst)
                }
                (Wire::VoiceMetrics(p), Wire::VoiceMetrics(q)) => self.metrics(p, q, &mut worst),
                (Wire::FormantTrack(p), Wire::FormantTrack(q)) => self.formants(p, q, &mut worst),
                (Wire::TractParams(p), Wire::TractParams(q)) => self.tract(p, q, &mut worst),
                (Wire::AreaFunction(p), Wire::AreaFunction(q)) => self.area(p, q, &mut worst),
                (Wire::TractGeometry(p), Wire::TractGeometry(q)) => self.geometry(p, q, &mut worst),
                _ => {
                    self.report.mismatched_types += 1;
                    continue;
                }
            }
            let entry = self
                .report
                .per_type
                .entry(x.wire_type().to_string())
                .or_default();
            entry.compared += 1;
            if worst.ratio > entry.worst_ratio {
                entry.worst_ratio = worst.ratio;
                entry.worst = worst.what;
            }
            if worst.ratio > 1.0 {
                entry.outside_band += 1;
            }
        }
    }

    fn f0(&self, p: &F0Track, q: &F0Track, w: &mut Excess) {
        w.flag("voiced", p.voiced != q.voiced);
        w.flag("rejected", p.rejected != q.rejected);
        w.abs("hz", p.hz, q.hz, self.tol.f0_hz);
        w.abs(
            "confidence",
            p.confidence,
            q.confidence,
            self.tol.confidence,
        );
        w.opt("snr_db", p.snr_db, q.snr_db, self.tol.metric_db);
    }

    fn spectrum(&self, p: &Spectrum, q: &Spectrum, w: &mut Excess) {
        w.flag("bins", p.magnitude.len() != q.magnitude.len());
        let peak = p
            .magnitude
            .iter()
            .cloned()
            .fold(0.0f32, f32::max)
            .max(f32::MIN_POSITIVE);
        for (i, (a, b)) in p.magnitude.iter().zip(&q.magnitude).enumerate() {
            w.abs(
                &format!("magnitude[{i}]/peak"),
                a / peak,
                b / peak,
                self.tol.spectrum_rel,
            );
        }
    }

    fn harmonics(&self, p: &HarmonicSeries, q: &HarmonicSeries, w: &mut Excess) {
        w.flag("voiced", p.voiced != q.voiced);
        w.abs("f0_hz", p.f0_hz, q.f0_hz, self.tol.f0_hz);
        let peak = p
            .amplitudes
            .iter()
            .cloned()
            .fold(0.0f32, f32::max)
            .max(f32::MIN_POSITIVE);
        for (i, (a, b)) in p.amplitudes.iter().zip(&q.amplitudes).enumerate() {
            w.abs(
                &format!("H{}/peak", i + 1),
                a / peak,
                b / peak,
                self.tol.harmonic_rel,
            );
        }
    }

    fn metrics(&self, p: &VoiceMetrics, q: &VoiceMetrics, w: &mut Excess) {
        let t = &self.tol;
        w.opt("hnr_db", p.hnr_db, q.hnr_db, t.metric_db);
        w.opt("h1_h2_db", p.h1_h2_db, q.h1_h2_db, t.metric_db);
        w.opt("jitter_pct", p.jitter_pct, q.jitter_pct, t.jitter_pct);
        w.opt("shimmer_db", p.shimmer_db, q.shimmer_db, t.metric_db);
        w.opt("cpp_db", p.cpp_db, q.cpp_db, t.metric_db);
        w.opt("centroid_hz", p.centroid_hz, q.centroid_hz, t.centroid_hz);
        w.opt("snr_db", p.snr_db, q.snr_db, t.metric_db);
        w.opt(
            "steadiness_cents",
            p.steadiness_cents,
            q.steadiness_cents,
            t.cents,
        );
        w.flag("voiced_but_noisy", p.voiced_but_noisy != q.voiced_but_noisy);
        match (p.vibrato, q.vibrato) {
            (None, None) => {}
            (Some(a), Some(b)) => {
                w.abs("vibrato.rate_hz", a.rate_hz, b.rate_hz, t.vibrato_rate_hz);
                w.abs(
                    "vibrato.extent_cents",
                    a.extent_cents,
                    b.extent_cents,
                    t.cents,
                );
            }
            _ => w.flag("vibrato presence", true),
        }
    }

    fn formants(&self, p: &FormantTrack, q: &FormantTrack, w: &mut Excess) {
        w.flag("fresh", p.fresh != q.fresh);
        w.opt(
            "F4",
            p.f4.map(|f| f.frequency),
            q.f4.map(|f| f.frequency),
            self.tol.formant_hz,
        );
        w.opt(
            "B4",
            p.f4.map(|f| f.bandwidth),
            q.f4.map(|f| f.bandwidth),
            self.tol.bandwidth_hz,
        );
        w.abs("measured_f0", p.measured_f0, q.measured_f0, self.tol.f0_hz);
        for (i, (a, b)) in p.formants.iter().zip(&q.formants).enumerate() {
            w.abs(
                &format!("F{}", i + 1),
                a.frequency,
                b.frequency,
                self.tol.formant_hz,
            );
            w.abs(
                &format!("B{}", i + 1),
                a.bandwidth,
                b.bandwidth,
                self.tol.bandwidth_hz,
            );
        }
    }

    fn tract(&self, p: &TractParams, q: &TractParams, w: &mut Excess) {
        w.flag("valid", p.valid != q.valid);
        w.flag("basis", p.basis != q.basis);
        w.flag("model", p.model != q.model);
        w.flag("abstained", p.abstained != q.abstained);
        w.flag("reason", p.reason != q.reason);
        w.abs("q1", p.q1, q.q1, self.tol.tract_q);
        w.abs("q2", p.q2, q.q2, self.tol.tract_q);
        for (i, (a, b)) in p.modes.iter().zip(&q.modes).enumerate() {
            w.abs(&format!("mode[{i}]"), *a, *b, self.tol.mode_sd);
        }
        w.opt(
            "confidence",
            p.confidence,
            q.confidence,
            self.tol.confidence,
        );
        w.opt(
            "uncertainty",
            p.uncertainty,
            q.uncertainty,
            self.tol.confidence,
        );
        w.opt("vtl_est_cm", p.vtl_est_cm, q.vtl_est_cm, self.tol.vtl_cm);
    }

    fn geometry(&self, p: &TractGeometry, q: &TractGeometry, w: &mut Excess) {
        w.flag("live", p.live != q.live);
        w.flag("model_id", p.model_id != q.model_id);
        w.flag("topology", p.triangles != q.triangles);
        w.flag("vertex count", p.vertices_mm.len() != q.vertices_mm.len());
        w.abs(
            "relative_area_std",
            p.relative_area_std,
            q.relative_area_std,
            self.tol.confidence,
        );
        for (i, (a, b)) in p.vertices_mm.iter().zip(&q.vertices_mm).enumerate() {
            w.abs(
                &format!("vertex[{}].{}", i / 3, i % 3),
                *a,
                *b,
                self.tol.geometry_mm,
            );
        }
        for (i, (a, b)) in p
            .uncertainty_vertices_mm
            .iter()
            .zip(&q.uncertainty_vertices_mm)
            .enumerate()
        {
            w.abs(
                &format!("uncertainty_vertex[{}].{}", i / 3, i % 3),
                *a,
                *b,
                self.tol.geometry_mm,
            );
        }
    }

    fn area(&self, p: &AreaFunction, q: &AreaFunction, w: &mut Excess) {
        w.flag("live", p.live != q.live);
        w.flag("basis", p.basis != q.basis);
        w.flag("model", p.model != q.model);
        w.flag("sections", p.sections != q.sections);
        w.abs("vtl_cm", p.vtl_cm, q.vtl_cm, self.tol.vtl_cm);
        for (i, (a, b)) in p.areas_cm2.iter().zip(&q.areas_cm2).enumerate() {
            w.abs(&format!("area[{i}]"), *a, *b, self.tol.area_cm2);
        }
        for (i, (a, b)) in p.diameters_cm.iter().zip(&q.diameters_cm).enumerate() {
            w.abs(&format!("diameter[{i}]"), *a, *b, self.tol.diameter_cm);
        }
    }
}

/// The worst normalized excess over a band within one wire.
#[derive(Default)]
struct Excess {
    ratio: f32,
    what: String,
}

impl Excess {
    fn note(&mut self, what: &str, ratio: f32) {
        if ratio > self.ratio {
            self.ratio = ratio;
            self.what = what.to_string();
        }
    }

    fn abs(&mut self, what: &str, a: f32, b: f32, band: f32) {
        let d = (a - b).abs();
        let ratio = if d == 0.0 {
            0.0
        } else if !d.is_finite() {
            f32::INFINITY
        } else {
            d / band.max(f32::MIN_POSITIVE)
        };
        self.note(&format!("{what}: {a} vs {b}"), ratio);
    }

    fn opt(&mut self, what: &str, a: Option<f32>, b: Option<f32>, band: f32) {
        match (a, b) {
            (None, None) => {}
            (Some(a), Some(b)) => self.abs(what, a, b, band),
            _ => self.note(&format!("{what}: {a:?} vs {b:?}"), f32::INFINITY),
        }
    }

    fn flag(&mut self, what: &str, differs: bool) {
        if differs {
            self.note(&format!("{what} differs"), f32::INFINITY);
        }
    }
}

/// The wire types a report covers, for the Evidence rows.
pub fn covered_types(report: &CompareReport) -> Vec<WireType> {
    WireType::ALL
        .iter()
        .copied()
        .filter(|t| report.per_type.contains_key(t.name()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::types::BasisId;

    #[test]
    fn identical_runs_are_inside_and_a_shifted_f0_is_outside() {
        let mut c = Comparer::new(ToleranceConfig::DEFAULT);
        let a = vec![
            Wire::F0Track(F0Track {
                hz: 220.0,
                confidence: 0.9,
                voiced: true,
                ..Default::default()
            }),
            Wire::AreaFunction(AreaFunction::neutral(BasisId::AdultMale)),
        ];
        c.hop(&a, &a);
        let mut b = a.clone();
        if let Wire::F0Track(t) = &mut b[0] {
            t.hz += 1.0;
        }
        c.hop(&a, &b);
        let r = c.finish();
        assert!(!r.within_bands());
        assert_eq!(r.hops_compared, 2);
        assert_eq!(r.per_type["F0Track"].outside_band, 1);
        assert_eq!(r.per_type["AreaFunction"].outside_band, 0);
        assert!(r.per_type["F0Track"].worst.starts_with("hz:"));
    }
}
