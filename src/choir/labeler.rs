//! Coral's SATB section labeler (`section-labeler.ts`): pitch-range
//! priors, a monotonic DP over the sorted notes (labels non-decreasing
//! in pitch, a divisi penalty for reusing a section), and an explicit
//! `A/T?` abstention in the alto/tenor overlap where the two are
//! near-chance to tell apart monaurally. The singer's-formant tiebreaker
//! (`spr_lean`) is ported too, and, as in Coral, not used by the
//! production path (it measured worse — see the TypeScript header).

use crate::config::ChoirLabelerConfig;

/// Sections low → high; DP labels stay non-decreasing along this order.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Section {
    B,
    T,
    A,
    S,
}

impl Section {
    pub const ASC: [Section; 4] = [Section::B, Section::T, Section::A, Section::S];

    pub fn rank(self) -> usize {
        match self {
            Section::B => 0,
            Section::T => 1,
            Section::A => 2,
            Section::S => 3,
        }
    }

    pub fn letter(self) -> &'static str {
        match self {
            Section::B => "B",
            Section::T => "T",
            Section::A => "A",
            Section::S => "S",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Section::B => "Bass",
            Section::T => "Tenor",
            Section::A => "Alto",
            Section::S => "Soprano",
        }
    }

    fn range(self, cfg: &ChoirLabelerConfig) -> (i32, i32) {
        match self {
            Section::B => (cfg.bass_lo, cfg.bass_hi),
            Section::T => (cfg.tenor_lo, cfg.tenor_hi),
            Section::A => (cfg.alto_lo, cfg.alto_hi),
            Section::S => (cfg.soprano_lo, cfg.soprano_hi),
        }
    }
}

/// A section, or the honest abstention in the A/T overlap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Label {
    Section(Section),
    /// `A/T?`
    AmbiguousAT,
}

impl Label {
    pub fn text(self) -> &'static str {
        match self {
            Label::Section(s) => s.letter(),
            Label::AmbiguousAT => "A/T?",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SectionLabel {
    pub midi: i32,
    pub label: Label,
    /// The DP's provisional section — the card an `A/T?` note is shown on.
    pub guess: Section,
    /// 0..1, soft; low in the A/T overlap, lower still for `A/T?`.
    pub confidence: f32,
}

fn range_fit(s: Section, m: i32, cfg: &ChoirLabelerConfig) -> f32 {
    let (lo, hi) = s.range(cfg);
    if m < lo || m > hi {
        let over = if m < lo { lo - m } else { m - hi } as f32;
        return (cfg.outside_scale * (-over / cfg.outside_decay_semitones).exp())
            .max(cfg.outside_floor);
    }
    let center = (lo + hi) as f32 / 2.0;
    let half = (hi - lo) as f32 / 2.0;
    (1.0 - (m as f32 - center).abs() / half).max(cfg.inside_floor)
}

fn round2(x: f32) -> f32 {
    (x * 100.0).round() / 100.0
}

/// Label detected notes by section. `notes` is deduplicated and sorted;
/// `confidence(midi)` is the detector's smoothed salience / threshold
/// (≥ 1 active) when known; `spr(midi)` the singer's-formant lean
/// (+alto / −tenor) when supplied.
pub fn label_sections(
    notes: &[i32],
    confidence: Option<&dyn Fn(i32) -> Option<f32>>,
    spr: Option<&dyn Fn(i32) -> f32>,
    cfg: &ChoirLabelerConfig,
) -> Vec<SectionLabel> {
    let mut sorted: Vec<i32> = notes.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let n = sorted.len();
    if n == 0 {
        return Vec::new();
    }
    let ns = Section::ASC.len();
    let mut dp: Vec<f64> = Section::ASC
        .iter()
        .map(|&s| range_fit(s, sorted[0], cfg) as f64)
        .collect();
    let mut back: Vec<[i32; 4]> = vec![[-1; 4]];
    for &m in sorted.iter().skip(1) {
        let mut cur = vec![0.0f64; ns];
        let mut bk = [0i32; 4];
        for s in 0..ns {
            let mut best_prev = f64::NEG_INFINITY;
            let mut best_s = 0;
            for (sp, &d) in dp.iter().enumerate().take(s + 1) {
                let val = d - if sp == s {
                    cfg.repeat_penalty as f64
                } else {
                    0.0
                };
                if val > best_prev {
                    best_prev = val;
                    best_s = sp;
                }
            }
            cur[s] = range_fit(Section::ASC[s], m, cfg) as f64 + best_prev;
            bk[s] = best_s as i32;
        }
        dp = cur;
        back.push(bk);
    }
    let mut s = 0;
    for k in 1..ns {
        if dp[k] > dp[s] {
            s = k;
        }
    }
    let mut provisional = vec![Section::B; n];
    for i in (0..n).rev() {
        provisional[i] = Section::ASC[s];
        let prev = back[i][s];
        s = if prev < 0 { 0 } else { prev as usize };
    }
    sorted
        .iter()
        .enumerate()
        .map(|(i, &m)| refine(m, i, &provisional, confidence, spr, cfg))
        .collect()
}

fn bracketed(i: usize, provisional: &[Section]) -> bool {
    let rank = provisional[i].rank();
    let below = provisional[..i].iter().any(|p| p.rank() < rank);
    let above = provisional[i + 1..].iter().any(|p| p.rank() > rank);
    below && above
}

fn refine(
    m: i32,
    i: usize,
    provisional: &[Section],
    confidence: Option<&dyn Fn(i32) -> Option<f32>>,
    spr: Option<&dyn Fn(i32) -> f32>,
    cfg: &ChoirLabelerConfig,
) -> SectionLabel {
    let guess = provisional[i];
    let det_factor = match confidence {
        Some(c) => (c(m).unwrap_or(1.0) / cfg.detector_factor_divisor)
            .min(1.0)
            .max(cfg.detector_factor_min),
        None => 1.0,
    };
    let in_at_zone =
        m >= cfg.at_zone_lo && m <= cfg.at_zone_hi && (guess == Section::A || guess == Section::T);
    if !in_at_zone {
        return SectionLabel {
            midi: m,
            label: Label::Section(guess),
            guess,
            confidence: round2(range_fit(guess, m, cfg) * det_factor),
        };
    }
    if let Some(spr) = spr {
        let lean = spr(m);
        if lean.abs() >= cfg.spr_margin {
            let label = if lean > 0.0 { Section::A } else { Section::T };
            return SectionLabel {
                midi: m,
                label: Label::Section(label),
                guess: label,
                confidence: round2(lean.abs().min(cfg.spr_confidence_max) * det_factor),
            };
        }
        return SectionLabel {
            midi: m,
            label: Label::AmbiguousAT,
            guess,
            confidence: round2(cfg.ambiguous_confidence * det_factor),
        };
    }
    if bracketed(i, provisional) {
        return SectionLabel {
            midi: m,
            label: Label::Section(guess),
            guess,
            confidence: round2(cfg.bracketed_discount * range_fit(guess, m, cfg) * det_factor),
        };
    }
    SectionLabel {
        midi: m,
        label: Label::AmbiguousAT,
        guess,
        confidence: round2(cfg.ambiguous_confidence * det_factor),
    }
}

/// Singer's-formant lean for the A/T tiebreaker (+alto / −tenor) from the
/// RAW magnitude spectrum (`mags.len()` = Coral's `numBins`). Ported, not
/// adopted (see the module doc).
pub fn spr_lean(midi: i32, mags: &[f32], sample_rate: f32, cfg: &ChoirLabelerConfig) -> f32 {
    let f0 = 440.0 * 2f32.powf((midi - 69) as f32 / 12.0);
    let bin_hz = sample_rate / (2.0 * mags.len() as f32);
    let k_lo = ((cfg.spr_band_lo_hz / f0).ceil() as i32).max(1);
    let k_hi = (cfg.spr_band_hi_hz / f0).floor() as i32;
    let mut wsum = 0.0f32;
    let mut esum = 0.0f32;
    for k in k_lo..=k_hi {
        let fk = k as f32 * f0;
        let center = (fk / bin_hz).round() as i64;
        let lo = (center - cfg.spr_peak_bins as i64).max(0) as usize;
        let hi = ((center + cfg.spr_peak_bins as i64) as usize).min(mags.len() - 1);
        let peak = mags[lo..=hi].iter().cloned().fold(0.0f32, f32::max);
        wsum += fk * peak;
        esum += peak;
    }
    if esum <= 0.0 {
        return 0.0;
    }
    ((wsum / esum - cfg.spr_center_hz) / cfg.spr_scale_hz).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(notes: &[i32]) -> Vec<&'static str> {
        label_sections(notes, None, None, &ChoirLabelerConfig::DEFAULT)
            .iter()
            .map(|l| l.label.text())
            .collect()
    }

    /// Coral's `section-labeler.test.ts`, the pure-function half.
    #[test]
    fn labels_by_pitch_order_and_abstains_in_the_overlap() {
        assert!(labels(&[]).is_empty());
        assert_eq!(labels(&[48, 55, 64, 72]), ["B", "T", "A", "S"]);
        assert_eq!(labels(&[40]), ["B"]);
        assert_eq!(labels(&[82]), ["S"]);
        assert_eq!(labels(&[62]), ["A/T?"]);
        assert_eq!(labels(&[48, 60]), ["B", "A/T?"]);
        let cfg = ChoirLabelerConfig::DEFAULT;
        let bright = |_: i32| 0.8f32;
        let dark = |_: i32| -0.8f32;
        let weak = |_: i32| 0.1f32;
        assert_eq!(
            label_sections(&[62], None, Some(&bright), &cfg)[0]
                .label
                .text(),
            "A"
        );
        assert_eq!(
            label_sections(&[62], None, Some(&dark), &cfg)[0]
                .label
                .text(),
            "T"
        );
        assert_eq!(
            label_sections(&[62], None, Some(&weak), &cfg)[0]
                .label
                .text(),
            "A/T?"
        );
        let out = label_sections(&[64, 60, 62, 65, 60], None, None, &cfg);
        assert_eq!(
            out.iter().map(|x| x.midi).collect::<Vec<_>>(),
            [60, 62, 64, 65]
        );
        for r in &out {
            assert!(r.confidence > 0.0 && r.confidence <= 1.0);
        }
        let weak_c = |_: i32| Some(1.0f32);
        let strong_c = |_: i32| Some(4.0f32);
        let w = label_sections(&[72], Some(&weak_c), None, &cfg)[0].confidence;
        let s = label_sections(&[72], Some(&strong_c), None, &cfg)[0].confidence;
        assert!(s > w);
        let sop = label_sections(&[72], None, None, &cfg)[0].confidence;
        let mid = label_sections(&[62], None, None, &cfg)[0].confidence;
        assert!(mid < sop);
    }
}
