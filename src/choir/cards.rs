//! Coral's stateful `HarmonyAnalyzer` (`harmony.ts`): the fixed S/A/T/B
//! cards with stickiness, onset debounce, hold and the three-phase claim
//! (continuity, free cards, orphan escape), the chord hold, the
//! leaky-memory drift readout and the per-section scatter. One instance
//! per pipeline; `now` is the caller's clock in ms (injectable).

use crate::config::ChoirHarmonyConfig;

use super::harmony::{
    Band, ChordId, HarmonyConfig, Pair, Profile, Target, Voice, VoiceIn, band, cents,
    consonance_index, hann_sigma_bins, identify_chord, note_info, pair_analysis, scatter_band,
    section_scatter, targets,
};
use super::labeler::{Label, Section};

#[derive(Clone, Debug)]
struct Card {
    f: Option<f32>,
    hist: Vec<f32>,
    last: f32,
    active: bool,
    hits: u32,
    since: f32,
    scat: Vec<f32>,
    scatter: Option<f32>,
    scatter_k: Option<u32>,
    midi: Option<i32>,
    label: Option<Label>,
    salience: f32,
    section_conf: f32,
    pend_n: u32,
}

impl Card {
    fn new() -> Self {
        Self {
            f: None,
            hist: Vec::new(),
            last: 0.0,
            active: false,
            hits: 0,
            since: 0.0,
            scat: Vec::new(),
            scatter: None,
            scatter_k: None,
            midi: None,
            label: None,
            salience: 0.0,
            section_conf: 0.0,
            pend_n: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HarmonyResult {
    pub voices: Vec<Voice>,
    pub id: Option<ChordId>,
    pub tg: Vec<Option<Target>>,
    pub pairs: Vec<Pair>,
    pub cons: Option<i32>,
    pub counts: [usize; 4],
    pub chord_held_ms: f32,
    pub settled: bool,
    pub drift: Option<f32>,
    pub tau: Option<f32>,
    pub profile: Profile,
    pub a4: f32,
}

impl HarmonyResult {
    pub fn band_of(&self, i: usize, cfg: &HarmonyConfig, hc: &ChoirHarmonyConfig) -> Option<Band> {
        let t = self.tg.get(i)?.as_ref()?;
        Some(band(t.err, Some(self.voices[i].held_ms), cfg, hc))
    }
}

pub struct HarmonyAnalyzer {
    pub cfg: HarmonyConfig,
    hc: ChoirHarmonyConfig,
    cards: [Card; 4],
    drift_r: Option<f32>,
    drift_r0: Option<f32>,
    drift_last_key: Option<String>,
    drift_last_t: f32,
    drift_tau: Option<f32>,
    hold_key: Option<String>,
    hold_start: f32,
    hann_sigma: f32,
}

fn median(v: &[f32]) -> f32 {
    let mut s = v.to_vec();
    s.sort_by(f32::total_cmp);
    s[s.len() / 2]
}

impl HarmonyAnalyzer {
    pub fn new(cfg: HarmonyConfig, hc: ChoirHarmonyConfig) -> Self {
        Self {
            cfg,
            hc,
            cards: [Card::new(), Card::new(), Card::new(), Card::new()],
            drift_r: None,
            drift_r0: None,
            drift_last_key: None,
            drift_last_t: 0.0,
            drift_tau: None,
            hold_key: None,
            hold_start: 0.0,
            hann_sigma: hann_sigma_bins(&hc),
        }
    }

    pub fn set_config(&mut self, next: HarmonyConfig) {
        if next.profile != self.cfg.profile || next.a4 != self.cfg.a4 {
            self.reset_drift();
        }
        self.cfg = next;
    }

    pub fn reset_cards(&mut self) {
        self.cards = [Card::new(), Card::new(), Card::new(), Card::new()];
    }

    pub fn reset_drift(&mut self) {
        self.drift_r = None;
        self.drift_r0 = None;
        self.drift_last_key = None;
        self.drift_last_t = 0.0;
        self.drift_tau = None;
    }

    pub fn reset(&mut self) {
        self.reset_cards();
        self.reset_drift();
        self.hold_key = None;
        self.hold_start = 0.0;
    }

    fn suspect(&self, f0: f32) -> bool {
        for c in &self.cards {
            let Some(f) = c.f else { continue };
            for k in 2..=self.hc.suspect_k_max {
                if cents(f * k as f32, f0).abs() <= self.hc.suspect_cents {
                    return true;
                }
            }
        }
        false
    }

    /// Cards are fixed per section; one voice per section, stickiest wins.
    fn update_cards(&mut self, input: &[VoiceIn], now: f32) {
        let hc = self.hc;
        let score = |x: &VoiceIn| x.salience * x.section_conf.max(hc.claim_confidence_floor);
        let mut labelled: [Option<usize>; 4] = [None; 4];
        for (vi, v) in input.iter().enumerate() {
            if !self.cfg.section_on(v.section) {
                continue;
            }
            let r = v.section.rank();
            match labelled[r] {
                Some(cur) if score(v) <= score(&input[cur]) => {}
                _ => labelled[r] = Some(vi),
            }
        }
        let mut by_section: [Option<usize>; 4] = [None; 4];
        let mut claimed = vec![false; input.len()];
        // Phase A — continuity.
        for s in Section::ASC {
            let r = s.rank();
            let Some(held) = self.cards[r].f else {
                continue;
            };
            if !self.cfg.section_on(s) {
                continue;
            }
            let mut best: Option<usize> = None;
            for (vi, v) in input.iter().enumerate() {
                if claimed[vi] || cents(held, v.f0_hz).abs() > hc.continuity_cents {
                    continue;
                }
                let better = match best {
                    None => true,
                    Some(b) => {
                        let bv = &input[b];
                        (v.section == s && bv.section != s)
                            || (v.section == bv.section && score(v) > score(bv))
                    }
                };
                if better {
                    best = Some(vi);
                }
            }
            if let Some(b) = best {
                by_section[r] = Some(b);
                claimed[b] = true;
            }
        }
        // Phase B — free cards take their labelled voice.
        for s in Section::ASC {
            let r = s.rank();
            if !self.cfg.section_on(s) || by_section[r].is_some() {
                continue;
            }
            let lab = labelled[r].filter(|&l| !claimed[l]);
            let Some(l) = lab else {
                self.cards[r].pend_n = 0;
                continue;
            };
            let is_suspect = self.suspect(input[l].f0_hz);
            let c = &mut self.cards[r];
            if c.f.is_none() {
                if is_suspect {
                    c.pend_n += 1;
                    if c.pend_n < hc.suspect_frames {
                        continue;
                    }
                }
            } else if is_suspect {
                continue;
            } else {
                c.pend_n += 1;
                if c.pend_n < hc.onset_frames {
                    continue;
                }
            }
            c.pend_n = 0;
            c.hist.clear();
            c.since = now;
            c.scat.clear();
            by_section[r] = Some(l);
            claimed[l] = true;
        }
        // Phase C — orphan escape.
        let held_f: Vec<f32> = self.cards.iter().filter_map(|c| c.f).collect();
        let lo = held_f.iter().cloned().fold(f32::INFINITY, f32::min);
        let hi = held_f.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let lowest = Section::ASC
            .iter()
            .copied()
            .find(|&s| self.cfg.section_on(s));
        let highest = Section::ASC
            .iter()
            .rev()
            .copied()
            .find(|&s| self.cfg.section_on(s));
        let orphans: Vec<usize> = (0..input.len())
            .filter(|&vi| {
                !claimed[vi]
                    && self.cfg.section_on(input[vi].section)
                    && input[vi].section_conf >= hc.orphan_confidence_min
            })
            .collect();
        let mut bumped = [false; 4];
        for o in orphans {
            let ov = input[o];
            let o_suspect = self.suspect(ov.f0_hz);
            let mut target: Option<usize> = None;
            let mut best = f32::INFINITY;
            for s in Section::ASC {
                let r = s.rank();
                let Some(cf) = self.cards[r].f else { continue };
                if !self.cfg.section_on(s) || bumped[r] {
                    continue;
                }
                let held_suspect = self.suspect(cf);
                if o_suspect {
                    if !held_suspect || labelled[r] != Some(o) {
                        continue;
                    }
                } else {
                    let outside = (ov.f0_hz < lo && Some(s) == lowest)
                        || (ov.f0_hz > hi && Some(s) == highest);
                    if !outside && !held_suspect {
                        continue;
                    }
                }
                let d = cents(cf, ov.f0_hz).abs();
                if d < best {
                    best = d;
                    target = Some(r);
                }
            }
            let Some(r) = target else { continue };
            bumped[r] = true;
            let c = &mut self.cards[r];
            c.pend_n += 1;
            if c.pend_n < hc.relabel_frames {
                continue;
            }
            c.pend_n = 0;
            c.hist.clear();
            c.since = now;
            c.scat.clear();
            if let Some(prev) = by_section[r] {
                claimed[prev] = false;
            }
            by_section[r] = Some(o);
            claimed[o] = true;
        }
        for r in 0..4 {
            if !bumped[r] && by_section[r].is_some() {
                self.cards[r].pend_n = 0;
            }
        }
        let mut seen = [false; 4];
        for (r, slot) in by_section.iter().enumerate() {
            let Some(vi) = *slot else { continue };
            let v = input[vi];
            let c = &mut self.cards[r];
            seen[r] = true;
            if c.hist.is_empty() {
                c.since = now;
            }
            c.hist.push(v.f0_hz);
            if c.hist.len() > hc.history_len {
                c.hist.remove(0);
            }
            c.f = Some(median(&c.hist));
            c.last = now;
            c.hits += 1;
            if c.hits >= hc.onset_frames {
                c.active = true;
            }
            c.midi = Some(v.midi);
            c.label = Some(v.label);
            c.salience = v.salience;
            c.section_conf = v.section_conf;
        }
        for (r, &was_seen) in seen.iter().enumerate() {
            if was_seen {
                continue;
            }
            if self.cards[r].f.is_some() && now - self.cards[r].last > hc.hold_ms {
                self.cards[r] = Card::new();
            }
        }
    }

    /// `spec_db`: the detection spectrum in dB (for scatter), or `None`.
    pub fn analyze(
        &mut self,
        input: &[VoiceIn],
        spec_db: Option<&[f32]>,
        bin_hz: f32,
        floor_db: f32,
        now: f32,
    ) -> HarmonyResult {
        let cfg = self.cfg;
        let hc = self.hc;
        self.update_cards(input, now);
        let mut voices: Vec<Voice> = Vec::new();
        for s in Section::ASC {
            let c = &self.cards[s.rank()];
            let (true, Some(f)) = (c.active, c.f) else {
                continue;
            };
            voices.push(Voice {
                info: note_info(f, cfg.a4),
                f,
                section: s,
                label: c.label.unwrap_or(Label::Section(s)),
                salience: c.salience,
                section_conf: c.section_conf,
                held_ms: now - c.since,
                scatter: None,
                scatter_k: None,
                scatter_band: None,
            });
        }
        voices.sort_by(|a, b| a.f.total_cmp(&b.f));
        if let Some(spec_db) = spec_db {
            let f0s: Vec<f32> = voices.iter().map(|v| v.f).collect();
            for v in &mut voices {
                let others: Vec<f32> = f0s.iter().copied().filter(|&o| o != v.f).collect();
                let c = &mut self.cards[v.section.rank()];
                if let Some((cents, k)) = section_scatter(
                    spec_db,
                    bin_hz,
                    v.f,
                    &others,
                    floor_db,
                    self.hann_sigma,
                    &hc,
                ) {
                    c.scat.push(cents);
                    if c.scat.len() > hc.history_len {
                        c.scat.remove(0);
                    }
                    c.scatter = Some(median(&c.scat));
                    c.scatter_k = Some(k);
                }
                v.scatter = c.scatter;
                v.scatter_k = c.scatter_k;
                v.scatter_band = scatter_band(c.scatter, &hc);
            }
        }
        let pcs: Vec<usize> = voices.iter().map(|v| v.info.pc).collect();
        let id = identify_chord(&pcs, &hc);
        let key = id.map(|id| {
            if id.cluster {
                "cluster".to_string()
            } else {
                format!("{}:{}:{}", id.root, id.chord.name, voices.len())
            }
        });
        if key != self.hold_key {
            self.hold_key = key.clone();
            self.hold_start = now;
        }
        let chord_held_ms = if key.is_some() {
            now - self.hold_start
        } else {
            0.0
        };
        let tg = targets(&voices, id.as_ref(), &cfg);
        let pairs = pair_analysis(&voices, &tg, &hc);
        let cons = consonance_index(&pairs, &hc);
        let settled = chord_held_ms >= hc.chord_hold_ms;
        if let (Some(id), Some(k)) = (id, &key)
            && !id.cluster
            && settled
        {
            self.drift_step(&voices, &tg, k, now);
        }
        let mut counts = [0usize; 4];
        for v in &voices {
            counts[v.section.rank()] += 1;
        }
        HarmonyResult {
            voices,
            id,
            tg,
            pairs,
            cons,
            counts,
            chord_held_ms,
            settled,
            drift: match (self.drift_r, self.drift_r0) {
                (Some(r), Some(r0)) => Some(r - r0),
                _ => None,
            },
            tau: self.drift_tau,
            profile: cfg.profile,
            a4: cfg.a4,
        }
    }

    /// τ = mean(ET deviation − profile offset); r ← m·r + (1 − m)·τ per
    /// chord change or every `drift_interval_ms`.
    fn drift_step(&mut self, voices: &[Voice], tg: &[Option<Target>], chord_key: &str, now: f32) {
        let vals: Vec<f32> = voices
            .iter()
            .zip(tg)
            .filter_map(|(v, t)| t.as_ref().map(|t| v.info.cents - t.phi))
            .collect();
        if vals.is_empty() {
            return;
        }
        let tau = vals.iter().sum::<f32>() / vals.len() as f32;
        self.drift_tau = Some(tau);
        let due = self.drift_last_key.as_deref() != Some(chord_key)
            || now - self.drift_last_t > self.hc.drift_interval_ms;
        if !due {
            return;
        }
        self.drift_last_key = Some(chord_key.to_string());
        self.drift_last_t = now;
        match self.drift_r {
            None => {
                self.drift_r = Some(tau);
                self.drift_r0 = Some(tau);
            }
            Some(r) => {
                self.drift_r = Some(self.hc.drift_memory * r + (1.0 - self.hc.drift_memory) * tau)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::choir::harmony::midi_to_hz;

    const A4: f32 = 440.0;

    fn v(section: Section, pc: i32, oct: i32, c: f32) -> VoiceIn {
        let f0 = midi_to_hz((12 * (oct + 1) + pc) as f32, A4) * 2f32.powf(c / 1200.0);
        VoiceIn {
            midi: note_info(f0, A4).midi,
            f0_hz: f0,
            salience: 2.0,
            section,
            label: Label::Section(section),
            section_conf: 0.8,
        }
    }

    fn run(h: &mut HarmonyAnalyzer, voices: &[VoiceIn], frames: usize, t0: f32) -> HarmonyResult {
        let mut r = h.analyze(voices, None, 44_100.0 / 8192.0, -120.0, t0);
        for i in 1..frames {
            r = h.analyze(
                voices,
                None,
                44_100.0 / 8192.0,
                -120.0,
                t0 + i as f32 * 47.0,
            );
        }
        r
    }

    fn cmaj_pure() -> Vec<VoiceIn> {
        vec![
            v(Section::B, 0, 3, 0.0),
            v(Section::T, 7, 3, 2.0),
            v(Section::A, 4, 4, -13.7),
            v(Section::S, 0, 5, 0.0),
        ]
    }

    fn analyzer() -> HarmonyAnalyzer {
        HarmonyAnalyzer::new(HarmonyConfig::default(), ChoirHarmonyConfig::DEFAULT)
    }

    /// Coral's `harmony.test.ts`: targets, profiles, anchoring.
    #[test]
    fn pure_c_major_reads_zero_error_and_ensemble_minus_7_7_on_the_third() {
        let mut h = analyzer();
        h.set_config(HarmonyConfig {
            profile: Profile::Pure,
            ..HarmonyConfig::default()
        });
        let r = run(&mut h, &cmaj_pure(), 12, 1000.0);
        assert_eq!(r.id.unwrap().chord.name, "Major");
        assert!(r.settled);
        for t in &r.tg {
            assert!(
                t.as_ref().map(|t| t.err.abs()).unwrap_or(99.0) < 0.5,
                "{:?}",
                r.tg
            );
        }
        h.set_config(HarmonyConfig {
            profile: Profile::Ensemble,
            ..HarmonyConfig::default()
        });
        let r2 = run(&mut h, &cmaj_pure(), 12, 1000.0);
        let alto = r2
            .voices
            .iter()
            .position(|x| x.section == Section::A)
            .unwrap();
        assert!((r2.tg[alto].as_ref().unwrap().err - -7.7).abs() < 0.5);
    }

    #[test]
    fn anchors_on_the_bass() {
        let mut h = analyzer();
        h.set_config(HarmonyConfig {
            profile: Profile::Pure,
            ..HarmonyConfig::default()
        });
        let mut det = cmaj_pure();
        det[0].f0_hz *= 2f32.powf(10.0 / 1200.0);
        let r = run(&mut h, &det, 12, 1000.0);
        assert!(r.tg[0].as_ref().unwrap().anchor);
        assert!((r.voices[0].info.cents - 10.0).abs() < 0.1);
        for i in 1..4 {
            assert!((r.tg[i].as_ref().unwrap().err - -10.0).abs() < 0.5);
        }
    }

    #[test]
    fn dominant_seventh_sizes_the_fifth_to_seventh_pair_as_7_to_6() {
        let mut h = analyzer();
        let r = run(
            &mut h,
            &[
                v(Section::B, 7, 2, 0.0),
                v(Section::T, 2, 4, 2.0),
                v(Section::A, 5, 4, -31.2),
                v(Section::S, 11, 4, -13.7),
            ],
            12,
            1000.0,
        );
        assert_eq!(r.id.unwrap().chord.name, "Dom 7");
        let p = r
            .pairs
            .iter()
            .find(|p| r.voices[p.a].info.name == "D" && r.voices[p.b].info.name == "F")
            .unwrap();
        assert_eq!(p.ratio, "7:6");
    }

    #[test]
    fn settles_after_300_ms_and_reports_drift_after_flat_updates() {
        let mut h = analyzer();
        assert!(!run(&mut h, &cmaj_pure(), 4, 1000.0).settled);
        run(&mut h, &cmaj_pure(), 12, 5000.0);
        let flat: Vec<VoiceIn> = cmaj_pure()
            .into_iter()
            .map(|x| VoiceIn {
                f0_hz: x.f0_hz * 2f32.powf(-30.0 / 1200.0),
                ..x
            })
            .collect();
        let mut r = h.analyze(&flat, None, 44_100.0 / 8192.0, -120.0, 20_000.0);
        for i in 1..22 {
            r = h.analyze(
                &flat,
                None,
                44_100.0 / 8192.0,
                -120.0,
                20_000.0 + i as f32 * 2100.0,
            );
        }
        let want = -30.0 * (1.0 - 0.85f32.powi(20));
        assert!(
            (r.drift.unwrap() - want).abs() < 0.5,
            "{:?} vs {want}",
            r.drift
        );
    }

    #[test]
    fn drops_voices_whose_section_is_off() {
        let mut h = analyzer();
        h.set_config(HarmonyConfig {
            sections: [true, true, false, false],
            ..HarmonyConfig::default()
        });
        let r = run(&mut h, &cmaj_pure(), 12, 1000.0);
        assert_eq!(
            r.voices.iter().map(|x| x.section).collect::<Vec<_>>(),
            [Section::B, Section::T]
        );
    }
}
