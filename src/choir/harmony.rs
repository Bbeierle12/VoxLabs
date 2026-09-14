//! Coral's rehearsal harmony maths (`harmony.ts`, itself ported from the
//! Choral Harmony Analyzer): note info, chord identification over pitch
//! classes, bass-anchored tuning targets sized by a profile, pair
//! intervals with beat rates, the consonance index, perception bands and
//! the section-scatter measure. Pure functions; the stateful card /
//! drift / hold analyser is `cards.rs`. The tuning profiles, JI ratio
//! table and chord templates are the app's published tables and stay
//! here as consts.

use crate::config::ChoirHarmonyConfig;

use super::labeler::{Label, Section};

pub const NOTE_NAMES: [&str; 12] = [
    "C", "C♯", "D", "E♭", "E", "F", "F♯", "G", "A♭", "A", "B♭", "B",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Profile {
    Pure,
    Ensemble,
    Equal,
}

impl Profile {
    pub fn name(self) -> &'static str {
        match self {
            Profile::Pure => "Pure (just)",
            Profile::Ensemble => "Ensemble",
            Profile::Equal => "Equal (12-TET)",
        }
    }

    /// Cents above the chord root for interval classes 0..11.
    pub fn cents(self) -> &'static [f32; 12] {
        match self {
            Profile::Pure => &[
                0.0, 112.0, 204.0, 316.0, 386.0, 498.0, 583.0, 702.0, 814.0, 884.0, 969.0, 1088.0,
            ],
            Profile::Ensemble => &[
                0.0, 108.0, 202.0, 302.0, 394.0, 499.0, 590.0, 703.0, 806.0, 898.0, 985.0, 1092.0,
            ],
            Profile::Equal => &[
                0.0, 100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0, 1000.0, 1100.0,
            ],
        }
    }
}

/// One detected voice as the pipeline emits it (`DspVoice`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoiceIn {
    pub midi: i32,
    pub f0_hz: f32,
    pub salience: f32,
    pub section: Section,
    pub label: Label,
    pub section_conf: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HarmonyConfig {
    pub a4: f32,
    pub profile: Profile,
    pub vibrato_heavy: bool,
    pub sections: [bool; 4],
}

impl Default for HarmonyConfig {
    fn default() -> Self {
        Self {
            a4: ChoirHarmonyConfig::DEFAULT.a4_hz,
            profile: Profile::Ensemble,
            vibrato_heavy: false,
            sections: [true; 4],
        }
    }
}

impl HarmonyConfig {
    pub fn section_on(&self, s: Section) -> bool {
        self.sections[s.rank()]
    }
}

pub fn cents(f1: f32, f2: f32) -> f32 {
    1200.0 * (f2 / f1).log2()
}
pub fn hz_to_midi(f: f32, a4: f32) -> f32 {
    69.0 + 12.0 * (f / a4).log2()
}
pub fn midi_to_hz(m: f32, a4: f32) -> f32 {
    a4 * 2f32.powf((m - 69.0) / 12.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteInfo {
    pub midi: i32,
    pub pc: usize,
    pub name: &'static str,
    pub oct: i32,
    pub cents: f32,
    pub et_hz: f32,
}

pub fn note_info(f: f32, a4: f32) -> NoteInfo {
    let m = hz_to_midi(f, a4);
    let r = m.round() as i32;
    let pc = r.rem_euclid(12) as usize;
    NoteInfo {
        midi: r,
        pc,
        name: NOTE_NAMES[pc],
        oct: r.div_euclid(12) - 1,
        cents: (m - r as f32) * 100.0,
        et_hz: midi_to_hz(r as f32, a4),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct IntervalClass {
    pub p: u32,
    pub q: u32,
    pub name: &'static str,
    pub w: f32,
}

pub const JI: [IntervalClass; 12] = [
    IntervalClass {
        p: 1,
        q: 1,
        name: "Unison",
        w: 1.0,
    },
    IntervalClass {
        p: 16,
        q: 15,
        name: "m2",
        w: 0.15,
    },
    IntervalClass {
        p: 9,
        q: 8,
        name: "M2",
        w: 0.3,
    },
    IntervalClass {
        p: 6,
        q: 5,
        name: "m3",
        w: 0.7,
    },
    IntervalClass {
        p: 5,
        q: 4,
        name: "M3",
        w: 0.7,
    },
    IntervalClass {
        p: 4,
        q: 3,
        name: "P4",
        w: 0.8,
    },
    IntervalClass {
        p: 7,
        q: 5,
        name: "TT",
        w: 0.2,
    },
    IntervalClass {
        p: 3,
        q: 2,
        name: "P5",
        w: 0.9,
    },
    IntervalClass {
        p: 8,
        q: 5,
        name: "m6",
        w: 0.65,
    },
    IntervalClass {
        p: 5,
        q: 3,
        name: "M6",
        w: 0.65,
    },
    IntervalClass {
        p: 7,
        q: 4,
        name: "m7",
        w: 0.35,
    },
    IntervalClass {
        p: 15,
        q: 8,
        name: "M7",
        w: 0.15,
    },
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChordTemplate {
    pub name: &'static str,
    /// Semitone offsets from the root.
    pub iv: &'static [usize],
    /// Pure integer ratio per chord tone (root first).
    pub ints: &'static [u32],
}

pub const CHORDS: [ChordTemplate; 13] = [
    ChordTemplate {
        name: "Major",
        iv: &[0, 4, 7],
        ints: &[4, 5, 6],
    },
    ChordTemplate {
        name: "Minor",
        iv: &[0, 3, 7],
        ints: &[10, 12, 15],
    },
    ChordTemplate {
        name: "Dom 7",
        iv: &[0, 4, 7, 10],
        ints: &[4, 5, 6, 7],
    },
    ChordTemplate {
        name: "Major 7",
        iv: &[0, 4, 7, 11],
        ints: &[8, 10, 12, 15],
    },
    ChordTemplate {
        name: "Minor 7",
        iv: &[0, 3, 7, 10],
        ints: &[10, 12, 15, 18],
    },
    ChordTemplate {
        name: "Dim",
        iv: &[0, 3, 6],
        ints: &[5, 6, 7],
    },
    ChordTemplate {
        name: "Sus4",
        iv: &[0, 5, 7],
        ints: &[6, 8, 9],
    },
    ChordTemplate {
        name: "Sus2",
        iv: &[0, 2, 7],
        ints: &[8, 9, 12],
    },
    ChordTemplate {
        name: "Open 5th",
        iv: &[0, 7],
        ints: &[2, 3],
    },
    ChordTemplate {
        name: "Major 3rd",
        iv: &[0, 4],
        ints: &[4, 5],
    },
    ChordTemplate {
        name: "Minor 3rd",
        iv: &[0, 3],
        ints: &[5, 6],
    },
    ChordTemplate {
        name: "Perfect 4th",
        iv: &[0, 5],
        ints: &[3, 4],
    },
    ChordTemplate {
        name: "Unison/8ve",
        iv: &[0],
        ints: &[1],
    },
];
pub const CLUSTER: ChordTemplate = ChordTemplate {
    name: "Cluster / Polyphony",
    iv: &[],
    ints: &[],
};

impl ChordTemplate {
    pub fn ratio_text(&self) -> String {
        if self.ints.is_empty() {
            "—".into()
        } else {
            self.ints
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(":")
        }
    }
}

pub fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

pub fn ratio_text(p: u32, q: u32) -> String {
    let g = gcd(p as u64, q as u64).max(1) as u32;
    format!("{}/{}", p / g, q / g)
}

/// A card's voice as the analyser reports it.
#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    pub info: NoteInfo,
    pub f: f32,
    pub section: Section,
    pub label: Label,
    pub salience: f32,
    pub section_conf: f32,
    pub held_ms: f32,
    pub scatter: Option<f32>,
    pub scatter_k: Option<u32>,
    pub scatter_band: Option<ScatterBand>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChordId {
    pub chord: ChordTemplate,
    pub root: usize,
    pub missing: usize,
    pub cluster: bool,
}

pub fn identify_chord(pcs_in: &[usize], cfg: &ChoirHarmonyConfig) -> Option<ChordId> {
    if pcs_in.is_empty() {
        return None;
    }
    let mut pcs: Vec<usize> = Vec::new();
    for &p in pcs_in {
        if !pcs.contains(&p) {
            pcs.push(p);
        }
    }
    let mut best: Option<(ChordId, f32)> = None;
    for c in &CHORDS {
        for &root in &pcs {
            let set: Vec<usize> = c.iv.iter().map(|i| (root + i) % 12).collect();
            let covered = pcs.iter().filter(|p| set.contains(p)).count();
            let missing = c.iv.len() - set.iter().filter(|p| pcs.contains(p)).count();
            if covered != pcs.len() {
                continue;
            }
            let score = covered as f32 * cfg.chord_score_covered
                - missing as f32 * cfg.chord_score_missing
                + if pcs_in[0] == root {
                    cfg.chord_score_root_bonus
                } else {
                    0.0
                }
                - c.iv.len() as f32 * cfg.chord_score_size_penalty;
            if best.is_none_or(|(_, s)| score > s) {
                best = Some((
                    ChordId {
                        chord: *c,
                        root,
                        missing,
                        cluster: false,
                    },
                    score,
                ));
            }
        }
    }
    Some(best.map(|(id, _)| id).unwrap_or(ChordId {
        chord: CLUSTER,
        root: pcs_in[0],
        missing: 0,
        cluster: true,
    }))
}

#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub tgt: f32,
    pub ji_cents: f32,
    /// Singer − target, cents (0 for the anchor).
    pub err: f32,
    /// Profile offset from ET for this chord tone.
    pub phi: f32,
    pub alt_pure: f32,
    pub alt_equal: f32,
    pub anchor: bool,
    pub k: usize,
    pub int: u32,
    pub role: String,
    pub ratio_txt: String,
}

/// Targets anchored on the lowest sounding chord tone.
pub fn targets(voices: &[Voice], id: Option<&ChordId>, cfg: &HarmonyConfig) -> Vec<Option<Target>> {
    let Some(id) = id.filter(|id| !id.cluster) else {
        return voices.iter().map(|_| None).collect();
    };
    let prof = cfg.profile.cents();
    let cls: Vec<Option<usize>> = voices
        .iter()
        .map(|v| {
            let rel = (v.info.pc + 12 - id.root) % 12;
            id.chord
                .iv
                .iter()
                .position(|&i| i == rel)
                .map(|k| id.chord.iv[k])
        })
        .collect();
    let Some(ai) = cls.iter().position(|c| c.is_some()) else {
        return voices.iter().map(|_| None).collect();
    };
    let a = &voices[ai];
    let cls_a = cls[ai].unwrap();
    let root_hz = a.f * 2f32.powf(-prof[cls_a] / 1200.0);
    let root_hz_of = |p: Profile| a.f * 2f32.powf(-p.cents()[cls_a] / 1200.0);
    voices
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let c = cls[i]?;
            let k = id.chord.iv.iter().position(|&x| x == c).unwrap_or(0);
            let near = |hz: f32| -> f32 {
                let mut t = hz;
                while t / v.f > std::f32::consts::SQRT_2 {
                    t /= 2.0;
                }
                while v.f / t > std::f32::consts::SQRT_2 {
                    t *= 2.0;
                }
                t
            };
            let tgt = near(root_hz * 2f32.powf(prof[c] / 1200.0));
            let alt_pure = cents(
                v.info.et_hz,
                near(root_hz_of(Profile::Pure) * 2f32.powf(Profile::Pure.cents()[c] / 1200.0)),
            );
            let alt_equal = cents(
                v.info.et_hz,
                near(root_hz_of(Profile::Equal) * 2f32.powf(Profile::Equal.cents()[c] / 1200.0)),
            );
            Some(Target {
                tgt,
                ji_cents: cents(v.info.et_hz, tgt),
                err: if i == ai { 0.0 } else { cents(tgt, v.f) },
                phi: prof[c] - 100.0 * c as f32,
                alt_pure,
                alt_equal,
                anchor: i == ai,
                k,
                int: id.chord.ints[k],
                role: if k == 0 {
                    "root".into()
                } else {
                    JI[c].name.into()
                },
                ratio_txt: ratio_text(id.chord.ints[k], id.chord.ints[0]),
            })
        })
        .collect()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chord_identification_and_note_info() {
        let cfg = ChoirHarmonyConfig::DEFAULT;
        let c = identify_chord(&[0, 4, 7], &cfg).unwrap();
        assert_eq!((c.chord.name, c.root, c.cluster), ("Major", 0, false));
        let d7 = identify_chord(&[7, 2, 5, 11], &cfg).unwrap();
        assert_eq!((d7.chord.name, d7.root), ("Dom 7", 7));
        assert!(identify_chord(&[0, 1, 6], &cfg).unwrap().cluster);
        assert!(identify_chord(&[], &cfg).is_none());
        let n = note_info(440.0 * 2f32.powf(10.0 / 1200.0), 440.0);
        assert_eq!((n.midi, n.name, n.oct), (69, "A", 4));
        assert!((n.cents - 10.0).abs() < 1e-3);
        assert_eq!(ratio_text(6, 4), "3/2");
        assert!(
            (hann_sigma_bins(&cfg) - 0.63).abs() < 0.1,
            "{}",
            hann_sigma_bins(&cfg)
        );
    }
}
