//! Coral's interval maths (`harmony.ts`, second half): pair intervals
//! with pure ratios and beat rates, the consonance index, the perception
//! bands, and the section-scatter measure from the width of a high
//! partial.

use crate::config::ChoirHarmonyConfig;

use super::harmony::{HarmonyConfig, JI, Target, Voice, cents, gcd};

#[derive(Clone, Debug, PartialEq)]
pub struct Pair {
    pub a: usize,
    pub b: usize,
    pub cls: usize,
    pub name: String,
    pub ratio: String,
    pub cents: f32,
    pub err: f32,
    /// Beat rate between the coinciding harmonics, Hz.
    pub beat: f32,
    pub tune: f32,
    pub w: f32,
}

pub fn pair_analysis(
    voices: &[Voice],
    tg: &[Option<Target>],
    cfg: &ChoirHarmonyConfig,
) -> Vec<Pair> {
    let mut out = Vec::new();
    for i in 0..voices.len() {
        for j in i + 1..voices.len() {
            let (a, b) = (&voices[i], &voices[j]);
            let c = cents(a.f, b.f);
            let semis = (c / 100.0).round() as i32;
            let k = semis.rem_euclid(12) as usize;
            let oct = semis.div_euclid(12);
            let ji = JI[k];
            let (mut p, mut q, pure_c) = match (&tg[i], &tg[j]) {
                (Some(ta), Some(tb)) => {
                    let n =
                        ((tb.tgt / ta.tgt * ta.int as f32 / tb.int as f32).log2()).round() as i32;
                    let mut p = tb.int as u64;
                    let mut q = ta.int as u64;
                    if n > 0 {
                        p <<= n as u32;
                    } else {
                        q <<= (-n) as u32;
                    }
                    (p, q, cents(ta.tgt, tb.tgt))
                }
                _ => {
                    let p = (ji.p as u64) << oct.max(0) as u32;
                    let q = ji.q as u64;
                    (p, q, 1200.0 * (p as f32 / q as f32).log2())
                }
            };
            let g = gcd(p, q).max(1);
            p /= g;
            q /= g;
            let err = c - pure_c;
            let beat = (q as f32 * b.f - p as f32 * a.f).abs();
            let tune = (-(err / cfg.tune_sigma_cents).powi(2)).exp();
            out.push(Pair {
                a: i,
                b: j,
                cls: k,
                name: if oct != 0 {
                    format!("{}+{oct}8ve", ji.name)
                } else {
                    ji.name.into()
                },
                ratio: format!("{p}:{q}"),
                cents: c,
                err,
                beat,
                tune,
                w: ji.w,
            });
        }
    }
    out
}

/// 0–100: tuning purity of the pairs and consonance of the interval classes.
pub fn consonance_index(pairs: &[Pair], cfg: &ChoirHarmonyConfig) -> Option<i32> {
    if pairs.is_empty() {
        return None;
    }
    let n = pairs.len() as f32;
    let tune = pairs.iter().map(|p| p.tune).sum::<f32>() / n;
    let w = pairs.iter().map(|p| p.w).sum::<f32>() / n;
    Some(
        (100.0 * (cfg.consonance_tune_weight * tune + cfg.consonance_class_weight * w)).round()
            as i32,
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Band {
    In,
    Marginal,
    Out,
}

pub fn band(err: f32, held_ms: Option<f32>, cfg: &HarmonyConfig, hc: &ChoirHarmonyConfig) -> Band {
    let (tin, tout) =
        if held_ms.is_some_and(|h| h < hc.short_note_ms) || (cfg.vibrato_heavy && err < 0.0) {
            (hc.band_wide_in_cents, hc.band_wide_out_cents)
        } else {
            (hc.band_in_cents, hc.band_out_cents)
        };
    let a = err.abs();
    if a <= tin {
        Band::In
    } else if a <= tout {
        Band::Marginal
    } else {
        Band::Out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScatterBand {
    Tight,
    Typical,
    Loose,
    Scattered,
}

pub fn scatter_band(c: Option<f32>, cfg: &ChoirHarmonyConfig) -> Option<ScatterBand> {
    c.map(|c| {
        if c < cfg.scatter_tight_cents {
            ScatterBand::Tight
        } else if c < cfg.scatter_typical_cents {
            ScatterBand::Typical
        } else if c < cfg.scatter_loose_cents {
            ScatterBand::Loose
        } else {
            ScatterBand::Scattered
        }
    })
}

/// RMS width of the Hann power main lobe in bins.
pub fn hann_sigma_bins(cfg: &ChoirHarmonyConfig) -> f32 {
    let d = |b: f32| {
        if b.abs() < 1e-9 {
            1.0
        } else {
            (std::f32::consts::PI * b).sin() / (std::f32::consts::PI * b)
        }
    };
    let (mut num, mut den) = (0.0f64, 0.0f64);
    let mut b = -cfg.hann_sigma_range_bins;
    while b <= cfg.hann_sigma_range_bins {
        let w = 0.5 * d(b) + 0.25 * (d(b - 1.0) + d(b + 1.0));
        let p = (w * w) as f64;
        num += p * (b * b) as f64;
        den += p;
        b += cfg.hann_sigma_step_bins;
    }
    (num / den).sqrt() as f32
}

/// Energy-weighted RMS width of the highest isolated partial, in cents.
pub fn section_scatter(
    spec_db: &[f32],
    bin_hz: f32,
    f0: f32,
    other_f0s: &[f32],
    floor_db: f32,
    hann_sigma: f32,
    cfg: &ChoirHarmonyConfig,
) -> Option<(f32, u32)> {
    let n = spec_db.len();
    let isolated = |fk: f32| {
        other_f0s.iter().all(|&o| {
            let m = (fk / o).round();
            m < 1.0 || (1200.0 * (fk / (m * o)).log2()).abs() > cfg.scatter_isolation_cents
        })
    };
    let mut k = cfg.scatter_k_max;
    while k >= cfg.scatter_k_min {
        let fk = k as f32 * f0;
        k -= 1;
        if fk > bin_hz * (n as f32 - 4.0) {
            continue;
        }
        if fk < cfg.scatter_min_hz {
            break;
        }
        if !isolated(fk) {
            continue;
        }
        let x0 = fk / bin_hz;
        let half = (x0 * (2f32.powf(cfg.scatter_half_cents / 1200.0) - 1.0))
            .max(cfg.scatter_half_bins_min as f32);
        let lo = ((x0 - half).floor() as usize).max(1);
        let hi = ((x0 + half).ceil() as usize).min(n - 2);
        let mut pk = lo;
        for i in lo..=hi {
            if spec_db[i] > spec_db[pk] {
                pk = i;
            }
        }
        if spec_db[pk] < floor_db + cfg.scatter_min_above_floor_db {
            continue;
        }
        let cut = (spec_db[pk] - cfg.scatter_cut_below_peak_db)
            .max(floor_db + cfg.scatter_cut_above_floor_db);
        let mut a = pk;
        while a > lo && spec_db[a - 1] > cut {
            a -= 1;
        }
        let mut b = pk;
        while b < hi && spec_db[b + 1] > cut {
            b += 1;
        }
        let pf = 10f32.powf(cut / 10.0);
        let (mut den, mut mean) = (0.0f32, 0.0f32);
        for (i, &db) in spec_db.iter().enumerate().take(b + 1).skip(a) {
            let p = (10f32.powf(db / 10.0) - pf).max(0.0);
            den += p;
            mean += p * i as f32;
        }
        if den <= 0.0 {
            continue;
        }
        mean /= den;
        let mut num = 0.0f32;
        for (i, &db) in spec_db.iter().enumerate().take(b + 1).skip(a) {
            let p = (10f32.powf(db / 10.0) - pf).max(0.0);
            num += p * (i as f32 - mean) * (i as f32 - mean);
        }
        let sig_bins = (num / den - hann_sigma * hann_sigma).max(0.0).sqrt();
        return Some((1200.0 * (1.0 + sig_bins / mean).log2(), k + 1));
    }
    None
}
