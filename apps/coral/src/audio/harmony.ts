/**
 * Harmony analysis for the rehearsal view (ported from the Choral Harmony
 * Analyzer's analysis.js, 2026-09-04). Pure functions over the voices the DSP
 * worker emits per detection frame: chord identification, bass-anchored
 * tuning targets sized by a selectable profile, pair intervals with beat
 * rates, a consonance index, perception-based tolerance bands, a leaky-memory
 * drift readout, per-section scatter from a high partial, and the fixed
 * S/A/T/B card state (stickiness, onset debounce, hold) that stops the cards
 * from flickering.
 *
 * Research basis: memo 2 (intonation reference) for the profiles, anchoring,
 * bands and drift; memo 3 (blend) for the scatter measure. No DOM, no audio —
 * runs unchanged in the worker, in Vitest and in Node.
 */

import type { Section, Label } from "./section-labeler";

export const NOTE_NAMES = [
  "C",
  "C♯",
  "D",
  "E♭",
  "E",
  "F",
  "F♯",
  "G",
  "A♭",
  "A",
  "B♭",
  "B",
] as const;

/** One detected voice as emitted by the worker (see DspVoice in dsp-protocol). */
export interface VoiceIn {
  midi: number;
  /** Sub-bin refined fundamental, Hz. */
  f0Hz: number;
  /** Detector confidence (salience / threshold; ≥1 = active). */
  salience: number;
  /** Section the labeler chose (its provisional guess when the label is `A/T?`). */
  section: Section;
  label: Label;
  sectionConf: number;
}

export type ProfileId = "pure" | "ensemble" | "equal";

/**
 * Target profiles: cents above the chord root for interval classes 0..11.
 * pure     = 5-limit just intonation (7:4 for the minor seventh)
 * ensemble = centre of what measured a cappella ensembles produce and expert
 *            listeners prefer (M3 392–405 measured, 395 preferred; m3 ≈ ET;
 *            P5 702–706; m7 977–985)
 * equal    = 12-TET
 */
export const PROFILES: Record<
  ProfileId,
  { name: string; c: readonly number[] }
> = {
  pure: {
    name: "Pure (just)",
    c: [0, 112, 204, 316, 386, 498, 583, 702, 814, 884, 969, 1088],
  },
  ensemble: {
    name: "Ensemble",
    c: [0, 108, 202, 302, 394, 499, 590, 703, 806, 898, 985, 1092],
  },
  equal: {
    name: "Equal (12-TET)",
    c: [0, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100],
  },
};

export interface HarmonyConfig {
  a4: number;
  profile: ProfileId;
  /** Widen the flat-side tolerance by 10 c (vibrato-heavy ensembles). */
  vibratoHeavy: boolean;
  /** Sections currently singing; voices labelled with a section that is off are dropped. */
  sections: Record<Section, boolean>;
}

export const DEFAULT_HARMONY_CONFIG: HarmonyConfig = {
  a4: 440,
  profile: "ensemble",
  vibratoHeavy: false,
  sections: { B: true, T: true, A: true, S: true },
};

/* ---------- pitch math ---------- */
export const cents = (f1: number, f2: number): number =>
  1200 * Math.log2(f2 / f1);
export const hzToMidi = (f: number, a4: number): number =>
  69 + 12 * Math.log2(f / a4);
export const midiToHz = (m: number, a4: number): number =>
  a4 * Math.pow(2, (m - 69) / 12);

export interface NoteInfo {
  midi: number;
  pc: number;
  name: string;
  oct: number;
  /** Deviation from the nearest ET note at the configured A4, cents. */
  cents: number;
  etHz: number;
}
export function noteInfo(f: number, a4: number): NoteInfo {
  const m = hzToMidi(f, a4);
  const r = Math.round(m);
  const pc = ((r % 12) + 12) % 12;
  return {
    midi: r,
    pc,
    name: NOTE_NAMES[pc] as string,
    oct: Math.floor(r / 12) - 1,
    cents: (m - r) * 100,
    etHz: midiToHz(r, a4),
  };
}

/* ---------- interval classes and chord templates ---------- */
interface IntervalClass {
  p: number;
  q: number;
  name: string;
  /** Consonance weight for the index. */
  w: number;
}
export const JI: Record<number, IntervalClass> = {
  0: { p: 1, q: 1, name: "Unison", w: 1.0 },
  1: { p: 16, q: 15, name: "m2", w: 0.15 },
  2: { p: 9, q: 8, name: "M2", w: 0.3 },
  3: { p: 6, q: 5, name: "m3", w: 0.7 },
  4: { p: 5, q: 4, name: "M3", w: 0.7 },
  5: { p: 4, q: 3, name: "P4", w: 0.8 },
  6: { p: 7, q: 5, name: "TT", w: 0.2 },
  7: { p: 3, q: 2, name: "P5", w: 0.9 },
  8: { p: 8, q: 5, name: "m6", w: 0.65 },
  9: { p: 5, q: 3, name: "M6", w: 0.65 },
  10: { p: 7, q: 4, name: "m7", w: 0.35 },
  11: { p: 15, q: 8, name: "M7", w: 0.15 },
};

export interface ChordTemplate {
  name: string;
  /** Semitone offsets from the root. */
  iv: readonly number[];
  /** Pure integer ratio per chord tone (root first). */
  ints: readonly number[];
  ratio: string;
}
const T = (name: string, iv: number[], ints: number[]): ChordTemplate => ({
  name,
  iv,
  ints,
  ratio: ints.join(":"),
});
export const CHORDS: readonly ChordTemplate[] = [
  T("Major", [0, 4, 7], [4, 5, 6]),
  T("Minor", [0, 3, 7], [10, 12, 15]),
  T("Dom 7", [0, 4, 7, 10], [4, 5, 6, 7]),
  T("Major 7", [0, 4, 7, 11], [8, 10, 12, 15]),
  T("Minor 7", [0, 3, 7, 10], [10, 12, 15, 18]),
  T("Dim", [0, 3, 6], [5, 6, 7]),
  T("Sus4", [0, 5, 7], [6, 8, 9]),
  T("Sus2", [0, 2, 7], [8, 9, 12]),
  T("Open 5th", [0, 7], [2, 3]),
  T("Major 3rd", [0, 4], [4, 5]),
  T("Minor 3rd", [0, 3], [5, 6]),
  T("Perfect 4th", [0, 5], [3, 4]),
  T("Unison/8ve", [0], [1]),
];
const CLUSTER: ChordTemplate = {
  name: "Cluster / Polyphony",
  iv: [],
  ints: [],
  ratio: "—",
};

export const gcd = (a: number, b: number): number => (b ? gcd(b, a % b) : a);
export const ratioText = (p: number, q: number): string => {
  const g = gcd(p, q);
  return `${p / g}/${q / g}`;
};

export interface Voice extends NoteInfo {
  f: number;
  section: Section;
  label: Label;
  salience: number;
  sectionConf: number;
  /** ms this card has held the current note. */
  heldMs: number;
  scatter: number | null;
  scatterK: number | null;
  scatterBand: ScatterBand | null;
}

export interface ChordId {
  chord: ChordTemplate;
  root: number;
  missing: number;
  cluster: boolean;
}

export function identifyChord(
  voices: ReadonlyArray<{ pc: number }>,
): ChordId | null {
  if (!voices.length) return null;
  const pcs = [...new Set(voices.map((v) => v.pc))];
  let best: (ChordId & { score: number }) | null = null;
  for (const c of CHORDS) {
    for (const root of pcs) {
      const set = c.iv.map((i) => (root + i) % 12);
      const covered = pcs.filter((p) => set.includes(p)).length;
      const missing = c.iv.length - set.filter((p) => pcs.includes(p)).length;
      if (covered !== pcs.length) continue;
      const score =
        covered * 10 -
        missing * 4 +
        ((voices[0] as { pc: number }).pc === root ? 2 : 0) -
        c.iv.length * 0.1;
      if (!best || score > best.score)
        best = { chord: c, root, missing, cluster: false, score };
    }
  }
  if (!best)
    return {
      chord: CLUSTER,
      root: (voices[0] as { pc: number }).pc,
      missing: 0,
      cluster: true,
    };
  const { score: _s, ...id } = best;
  void _s;
  return id;
}

export interface Target {
  tgt: number;
  /** Target's offset from the voice's ET pitch, cents (gauge marker). */
  jiCents: number;
  /** Singer − target, cents (0 for the anchor). */
  err: number;
  /** Profile offset from ET for this chord tone (drift). */
  phi: number;
  alt: { pure: number; equal: number };
  anchor: boolean;
  k: number;
  int: number;
  role: string;
  ratioTxt: string;
}

/**
 * Targets anchored on the lowest sounding chord tone and sized by the profile.
 * Singers tune to the bass (Devaney 2012; barbershop pedagogy); the anchor
 * never gets a "move" instruction.
 */
export function targets(
  voices: ReadonlyArray<Voice>,
  id: ChordId | null,
  cfg: HarmonyConfig,
): Array<Target | null> {
  if (!id || id.cluster) return voices.map(() => null);
  const prof = PROFILES[cfg.profile].c;
  const cls = voices.map((v) => {
    const k = id.chord.iv.indexOf((((v.pc - id.root) % 12) + 12) % 12);
    return k < 0 ? null : (id.chord.iv[k] as number);
  });
  const ai = cls.findIndex((c) => c != null);
  if (ai < 0) return voices.map(() => null);
  const A = voices[ai] as Voice;
  const clsA = cls[ai] as number;
  const rootHz = A.f * Math.pow(2, -(prof[clsA] as number) / 1200);
  const rootHzOf = (p: ProfileId) =>
    A.f * Math.pow(2, -(PROFILES[p].c[clsA] as number) / 1200);
  return voices.map((v, i) => {
    const c = cls[i];
    if (c == null) return null;
    const k = id.chord.iv.indexOf(c);
    const near = (hz: number): number => {
      let t = hz;
      while (t / v.f > Math.SQRT2) t /= 2;
      while (v.f / t > Math.SQRT2) t *= 2;
      return t;
    };
    const tgt = near(rootHz * Math.pow(2, (prof[c] as number) / 1200));
    const alt = {
      pure: cents(
        v.etHz,
        near(
          rootHzOf("pure") * Math.pow(2, (PROFILES.pure.c[c] as number) / 1200),
        ),
      ),
      equal: cents(
        v.etHz,
        near(
          rootHzOf("equal") *
            Math.pow(2, (PROFILES.equal.c[c] as number) / 1200),
        ),
      ),
    };
    return {
      tgt,
      jiCents: cents(v.etHz, tgt),
      err: i === ai ? 0 : cents(tgt, v.f),
      phi: (prof[c] as number) - 100 * c,
      alt,
      anchor: i === ai,
      k,
      int: id.chord.ints[k] as number,
      role: k === 0 ? "root" : (JI[c]?.name ?? ""),
      ratioTxt: ratioText(
        id.chord.ints[k] as number,
        id.chord.ints[0] as number,
      ),
    };
  });
}

export interface Pair {
  a: Voice;
  b: Voice;
  cls: number;
  name: string;
  ratio: string;
  cents: number;
  err: number;
  /** Beat rate between the coinciding harmonics, Hz. */
  beat: number;
  tune: number;
  w: number;
}

export function pairAnalysis(
  voices: ReadonlyArray<Voice>,
  tg: ReadonlyArray<Target | null>,
): Pair[] {
  const out: Pair[] = [];
  for (let i = 0; i < voices.length; i++) {
    for (let j = i + 1; j < voices.length; j++) {
      const a = voices[i] as Voice;
      const b = voices[j] as Voice;
      const c = cents(a.f, b.f);
      const semis = Math.round(c / 100);
      const k = ((semis % 12) + 12) % 12;
      const oct = Math.floor(semis / 12);
      const ji = JI[k] as IntervalClass;
      let P: number;
      let Q: number;
      let pureC: number;
      const ta = tg[i];
      const tb = tg[j];
      if (ta && tb) {
        const n = Math.round(Math.log2(((tb.tgt / ta.tgt) * ta.int) / tb.int));
        P = tb.int;
        Q = ta.int;
        if (n > 0) P *= Math.pow(2, n);
        else Q *= Math.pow(2, -n);
        pureC = cents(ta.tgt, tb.tgt);
      } else {
        P = ji.p * Math.pow(2, oct);
        Q = ji.q;
        pureC = 1200 * Math.log2(P / Q);
      }
      const g = gcd(P, Q);
      P /= g;
      Q /= g;
      const err = c - pureC;
      const beat = Math.abs(Q * b.f - P * a.f);
      const tune = Math.exp(-Math.pow(err / 20, 2));
      out.push({
        a,
        b,
        cls: k,
        name: ji.name + (oct ? `+${oct}8ve` : ""),
        ratio: `${P}:${Q}`,
        cents: c,
        err,
        beat,
        tune,
        w: ji.w,
      });
    }
  }
  return out;
}

/** 0–100: 60 % tuning purity of the pairs, 40 % consonance of the interval classes. */
export function consonanceIndex(pairs: ReadonlyArray<Pair>): number | null {
  if (!pairs.length) return null;
  const tune = pairs.reduce((t, p) => t + p.tune, 0) / pairs.length;
  const w = pairs.reduce((t, p) => t + p.w, 0) / pairs.length;
  return Math.round(100 * (0.6 * tune + 0.4 * w));
}

export type Band = "in" | "marginal" | "out";
/** ±10 c in tune, 10–20 marginal, >20 out; flat side +10 c when vibrato-heavy; ±20/30 for notes < 250 ms. */
export function band(
  err: number,
  heldMs: number | null,
  cfg: HarmonyConfig,
): Band {
  let tin = 10;
  let tout = 20;
  if (heldMs != null && heldMs < 250) {
    tin = 20;
    tout = 30;
  } else if (cfg.vibratoHeavy && err < 0) {
    tin = 20;
    tout = 30;
  }
  const a = Math.abs(err);
  return a <= tin ? "in" : a <= tout ? "marginal" : "out";
}

/* ---------- section scatter from the width of a high partial ---------- */
export type ScatterBand = "tight" | "typical" | "loose" | "scattered";
export const scatterBand = (c: number | null): ScatterBand | null =>
  c == null
    ? null
    : c < 10
      ? "tight"
      : c < 15
        ? "typical"
        : c < 30
          ? "loose"
          : "scattered";

/** RMS width of the Hann power main lobe in bins, sampled at bin spacing. */
const HANN_SIGMA_BINS = (() => {
  let num = 0;
  let den = 0;
  const D = (b: number) =>
    Math.abs(b) < 1e-9 ? 1 : Math.sin(Math.PI * b) / (Math.PI * b);
  for (let b = -3; b <= 3; b += 0.01) {
    const W = 0.5 * D(b) + 0.25 * (D(b - 1) + D(b + 1));
    const p = W * W;
    num += p * b * b;
    den += p;
  }
  return Math.sqrt(num / den);
})();

/**
 * Energy-weighted RMS width of the highest isolated partial k ≤ 10 (≥ 500 Hz,
 * not within a quarter tone of any other voice's partial), window width
 * removed in quadrature, in cents at k·f0. `specDb` is the detection
 * spectrum in dB. Null when no partial qualifies.
 */
export function sectionScatter(
  specDb: Float32Array,
  binHz: number,
  f0: number,
  otherF0s: ReadonlyArray<number>,
  floorDb: number,
): { cents: number; k: number } | null {
  const n = specDb.length;
  const isolated = (fk: number) =>
    otherF0s.every((o) => {
      const m = Math.round(fk / o);
      return m < 1 || Math.abs(1200 * Math.log2(fk / (m * o))) > 50;
    });
  for (let k = 10; k >= 2; k--) {
    const fk = k * f0;
    if (fk > binHz * (n - 4)) continue;
    if (fk < 500) break;
    if (!isolated(fk)) continue;
    const x0 = fk / binHz;
    const half = Math.max(3, x0 * (Math.pow(2, 60 / 1200) - 1));
    const lo = Math.max(1, Math.floor(x0 - half));
    const hi = Math.min(n - 2, Math.ceil(x0 + half));
    let pk = lo;
    for (let i = lo; i <= hi; i++)
      if ((specDb[i] as number) > (specDb[pk] as number)) pk = i;
    if ((specDb[pk] as number) < floorDb + 20) continue;
    const cut = Math.max((specDb[pk] as number) - 12, floorDb + 6);
    let a = pk;
    while (a > lo && (specDb[a - 1] as number) > cut) a--;
    let b = pk;
    while (b < hi && (specDb[b + 1] as number) > cut) b++;
    const pf = Math.pow(10, cut / 10);
    let den = 0;
    let mean = 0;
    for (let i = a; i <= b; i++) {
      const p = Math.max(0, Math.pow(10, (specDb[i] as number) / 10) - pf);
      den += p;
      mean += p * i;
    }
    if (den <= 0) continue;
    mean /= den;
    let num = 0;
    for (let i = a; i <= b; i++) {
      const p = Math.max(0, Math.pow(10, (specDb[i] as number) / 10) - pf);
      num += p * (i - mean) * (i - mean);
    }
    const sigBins = Math.sqrt(
      Math.max(0, num / den - HANN_SIGMA_BINS * HANN_SIGMA_BINS),
    );
    return {
      cents: 1200 * Math.log2(1 + (sigBins * binHz) / (mean * binHz)),
      k,
    };
  }
  return null;
}

/* ---------- card state, drift memory, chord hold ---------- */
const SECTIONS: readonly Section[] = ["B", "T", "A", "S"];
export const HOLD_MS = 400;
export const ONSET_FRAMES = 3; // detection frames (~21/s) before a card lights: ≈140 ms
/** Detection frames of consistent relabelling before a card abandons a note something still continues: ≈0.6 s. */
export const RELABEL_FRAMES = 12;
/** Detection frames a suspect (partial-of-a-held-note) candidate must persist before it takes a free card. */
export const SUSPECT_FRAMES = 6;
export const CHORD_HOLD_MS = 300;

interface CardState {
  f: number | null;
  hist: number[];
  last: number;
  active: boolean;
  hits: number;
  since: number;
  scat: number[];
  scatter: number | null;
  scatterK: number | null;
  midi: number | null;
  label: Label | null;
  salience: number;
  sectionConf: number;
  /** Frames in a row the labeler's candidate has disagreed (>80 c) with the held note. */
  pendN: number;
}
const newCard = (): CardState => ({
  f: null,
  hist: [],
  last: 0,
  active: false,
  hits: 0,
  since: 0,
  scat: [],
  scatter: null,
  scatterK: null,
  midi: null,
  label: null,
  salience: 0,
  sectionConf: 0,
  pendN: 0,
});

export interface HarmonyResult {
  voices: Voice[];
  id: ChordId | null;
  tg: Array<Target | null>;
  pairs: Pair[];
  cons: number | null;
  counts: Record<Section, number>;
  chordHeldMs: number;
  settled: boolean;
  /** Drift since start, cents, or null before the first chord. */
  drift: number | null;
  tau: number | null;
  profile: ProfileId;
  a4: number;
}

/**
 * Stateful analyser: feed it the worker's voices per detection frame (plus the
 * detection spectrum for scatter) and read back everything the rehearsal
 * view shows. One instance per pipeline; reset on start.
 */
export class HarmonyAnalyzer {
  cfg: HarmonyConfig;
  private cards: Record<Section, CardState> = {
    B: newCard(),
    T: newCard(),
    A: newCard(),
    S: newCard(),
  };
  private drift = {
    r: null as number | null,
    r0: null as number | null,
    lastKey: null as string | null,
    lastT: 0,
    tau: null as number | null,
  };
  private hold = { key: null as string | null, start: 0 };

  constructor(cfg: HarmonyConfig = DEFAULT_HARMONY_CONFIG) {
    this.cfg = { ...cfg, sections: { ...cfg.sections } };
  }

  setConfig(
    partial: Partial<Omit<HarmonyConfig, "sections">> & {
      sections?: Partial<Record<Section, boolean>>;
    },
  ): void {
    const next: HarmonyConfig = {
      ...this.cfg,
      ...partial,
      sections: { ...this.cfg.sections, ...(partial.sections ?? {}) },
    };
    if (next.profile !== this.cfg.profile || next.a4 !== this.cfg.a4)
      this.resetDrift();
    this.cfg = next;
  }

  resetCards(): void {
    for (const s of SECTIONS) this.cards[s] = newCard();
  }
  resetDrift(): void {
    this.drift = { r: null, r0: null, lastKey: null, lastT: 0, tau: null };
  }
  reset(): void {
    this.resetCards();
    this.resetDrift();
    this.hold = { key: null, start: 0 };
  }

  /** Cards are fixed per section; one voice per section, stickiest wins. */
  private updateCards(input: ReadonlyArray<VoiceIn>, now: number): void {
    const seen = new Set<Section>();
    // Card claim: detector confidence weighted by how well the note fits the
    // section, so a strong out-of-range phantom does not displace a real voice.
    const score = (x: VoiceIn) => x.salience * Math.max(0.05, x.sectionConf);
    const labelled = new Map<Section, VoiceIn>();
    for (const v of input) {
      if (!this.cfg.sections[v.section]) continue;
      const cur = labelled.get(v.section);
      if (!cur || score(v) > score(cur)) labelled.set(v.section, v);
    }
    // A candidate sitting on partial 2–4 of a note some card already holds
    // is a suspect: it may take a free card (a real octave doubling) or
    // continue a note, but it never makes a card abandon a held note.
    const suspect = (v: VoiceIn): boolean => {
      for (const s of SECTIONS) {
        const f = this.cards[s].f;
        if (f == null) continue;
        for (let k = 2; k <= 4; k++)
          if (Math.abs(cents(f * k, v.f0Hz)) <= 40) return true;
      }
      return false;
    };
    const bySection = new Map<Section, VoiceIn>();
    const claimed = new Set<VoiceIn>();
    // Phase A — continuity. A card holding a note follows it: the nearest
    // unclaimed voice within 80 c, whatever the labeler called it this
    // frame. A transient extra candidate (typically a lower voice's octave
    // partial) shifts the monotonic section labels by one; without this the
    // tenor card jumped to the phantom for a frame and the chord hold reset
    // (measured on the synthetic 4×4 C-major chord).
    for (const s of SECTIONS) {
      const c = this.cards[s];
      if (c.f == null || !this.cfg.sections[s]) continue;
      const held = c.f;
      let best: VoiceIn | null = null;
      for (const v of input) {
        if (claimed.has(v) || Math.abs(cents(held, v.f0Hz)) > 80) continue;
        if (
          !best ||
          (v.section === s && best.section !== s) ||
          (v.section === best.section && score(v) > score(best))
        )
          best = v;
      }
      if (best) {
        bySection.set(s, best);
        claimed.add(best);
      }
    }
    // Phase B — free cards take their labelled voice; a suspect must persist
    // SUSPECT_FRAMES first (a real octave doubling still gets its card, a
    // one-frame partial does not). A held card with nothing continuing its
    // note jumps to its labelled voice after ONSET_FRAMES.
    for (const s of SECTIONS) {
      if (!this.cfg.sections[s]) continue;
      const c = this.cards[s];
      const lab = labelled.get(s);
      if (bySection.has(s)) continue;
      if (!lab || claimed.has(lab)) {
        c.pendN = 0;
        continue;
      }
      if (c.f == null) {
        if (suspect(lab) && ++c.pendN < SUSPECT_FRAMES) continue;
      } else if (suspect(lab) || ++c.pendN < ONSET_FRAMES) continue;
      c.pendN = 0;
      c.hist = [];
      c.since = now;
      c.scat = [];
      bySection.set(s, lab);
      claimed.add(lab);
    }
    // Phase C — orphan escape. A real (non-suspect) voice nobody claimed
    // means the cards are mis-set — typically a card locked onto a phantom
    // at onset. It displaces, after RELABEL_FRAMES, the nearest held card
    // whose own note is suspect, or the extreme card when it lies outside
    // the span of all held notes. Otherwise a held note is never abandoned.
    const heldF = SECTIONS.map((s) => this.cards[s].f).filter(
      (f): f is number => f != null,
    );
    const lo = Math.min(...heldF);
    const hi = Math.max(...heldF);
    const lowest = SECTIONS.find((s) => this.cfg.sections[s]);
    const highest = [...SECTIONS].reverse().find((s) => this.cfg.sections[s]);
    // A suspect orphan (e.g. a real octave doubling) may only displace a
    // card whose held note is also suspect and whose labelled voice it is —
    // the labeler breaks the tie between two partial-shaped candidates.
    const orphans = input.filter(
      (v) =>
        !claimed.has(v) && this.cfg.sections[v.section] && v.sectionConf >= 0.2,
    );
    const bumped = new Set<Section>();
    for (const o of orphans) {
      const oSuspect = suspect(o);
      let target: Section | null = null;
      let best = Infinity;
      for (const s of SECTIONS) {
        const c = this.cards[s];
        if (c.f == null || !this.cfg.sections[s] || bumped.has(s)) continue;
        const heldSuspect = suspect({ ...o, f0Hz: c.f });
        if (oSuspect) {
          if (!heldSuspect || labelled.get(s) !== o) continue;
        } else {
          const outside =
            (o.f0Hz < lo && s === lowest) || (o.f0Hz > hi && s === highest);
          if (!outside && !heldSuspect) continue;
        }
        const d = Math.abs(cents(c.f, o.f0Hz));
        if (d < best) {
          best = d;
          target = s;
        }
      }
      if (!target) continue;
      const c = this.cards[target];
      bumped.add(target);
      if (++c.pendN < RELABEL_FRAMES) continue;
      c.pendN = 0;
      c.hist = [];
      c.since = now;
      c.scat = [];
      const prev = bySection.get(target);
      if (prev) claimed.delete(prev);
      bySection.set(target, o);
      claimed.add(o);
    }
    for (const s of SECTIONS)
      if (!bumped.has(s) && bySection.has(s)) this.cards[s].pendN = 0;
    for (const [s, v] of bySection) {
      const c = this.cards[s];
      seen.add(s);
      if (!c.hist.length) c.since = now;
      c.hist.push(v.f0Hz);
      if (c.hist.length > 5) c.hist.shift();
      const sorted = [...c.hist].sort((a, b) => a - b);
      c.f = sorted[Math.floor(sorted.length / 2)] as number;
      c.last = now;
      c.hits++;
      if (c.hits >= ONSET_FRAMES) c.active = true;
      c.midi = v.midi;
      c.label = v.label;
      c.salience = v.salience;
      c.sectionConf = v.sectionConf;
    }
    for (const s of SECTIONS) {
      if (seen.has(s)) continue;
      const c = this.cards[s];
      // A free card keeps its pendN (the suspect-persistence counter).
      if (c.f != null && now - c.last > HOLD_MS) this.cards[s] = newCard();
    }
  }

  analyze(
    input: ReadonlyArray<VoiceIn>,
    specDb: Float32Array | null,
    binHz: number,
    floorDb: number,
    now: number,
  ): HarmonyResult {
    const cfg = this.cfg;
    this.updateCards(input, now);
    const voices: Voice[] = [];
    for (const s of SECTIONS) {
      const c = this.cards[s];
      if (!c.active || c.f == null) continue;
      const ni = noteInfo(c.f, cfg.a4);
      voices.push({
        ...ni,
        f: c.f,
        section: s,
        label: c.label ?? s,
        salience: c.salience,
        sectionConf: c.sectionConf,
        heldMs: now - c.since,
        scatter: null,
        scatterK: null,
        scatterBand: null,
      });
    }
    voices.sort((a, b) => a.f - b.f);
    if (specDb) {
      const f0s = voices.map((v) => v.f);
      for (const v of voices) {
        const c = this.cards[v.section];
        const s = sectionScatter(
          specDb,
          binHz,
          v.f,
          f0s.filter((o) => o !== v.f),
          floorDb,
        );
        if (s) {
          c.scat.push(s.cents);
          if (c.scat.length > 5) c.scat.shift();
          const so = [...c.scat].sort((a, b) => a - b);
          c.scatter = so[Math.floor(so.length / 2)] as number;
          c.scatterK = s.k;
        }
        v.scatter = c.scatter;
        v.scatterK = c.scatterK;
        v.scatterBand = scatterBand(c.scatter);
      }
    }
    const id = identifyChord(voices);
    const key = id
      ? id.cluster
        ? "cluster"
        : `${id.root}:${id.chord.name}:${voices.length}`
      : null;
    if (key !== this.hold.key) this.hold = { key, start: now };
    const chordHeldMs = key ? now - this.hold.start : 0;
    const tg = targets(voices, id, cfg);
    const pairs = pairAnalysis(voices, tg);
    const cons = consonanceIndex(pairs);
    const settled = chordHeldMs >= CHORD_HOLD_MS;
    if (id && !id.cluster && settled && key)
      this.driftStep(voices, tg, key, now);
    const counts: Record<Section, number> = { B: 0, T: 0, A: 0, S: 0 };
    for (const v of voices) counts[v.section]++;
    const d = this.drift;
    return {
      voices,
      id,
      tg,
      pairs,
      cons,
      counts,
      chordHeldMs,
      settled,
      drift: d.r == null || d.r0 == null ? null : d.r - d.r0,
      tau: d.tau,
      profile: cfg.profile,
      a4: cfg.a4,
    };
  }

  /** τ = mean(ET deviation − profile offset); r ← 0.85·r + 0.15·τ per chord change or every 2 s. */
  private driftStep(
    voices: ReadonlyArray<Voice>,
    tg: ReadonlyArray<Target | null>,
    chordKey: string,
    now: number,
  ): void {
    const vals: number[] = [];
    voices.forEach((v, i) => {
      const t = tg[i];
      if (t) vals.push(v.cents - t.phi);
    });
    if (!vals.length) return;
    const tau = vals.reduce((a, b) => a + b, 0) / vals.length;
    const d = this.drift;
    d.tau = tau;
    const due = chordKey !== d.lastKey || now - d.lastT > 2000;
    if (!due) return;
    d.lastKey = chordKey;
    d.lastT = now;
    if (d.r == null) {
      d.r = tau;
      d.r0 = tau;
    } else d.r = 0.85 * d.r + 0.15 * tau;
  }
}

/** Plain-text snapshot of a result, for the rehearsal log / clipboard. */
export function snapshotText(
  r: HarmonyResult,
  when: number,
  cfg: HarmonyConfig,
): string | null {
  if (!r.voices.length || !r.id) return null;
  const t = new Date(when);
  const sec: Record<Section, string> = {
    B: "Bass",
    T: "Tenor",
    A: "Alto",
    S: "Soprano",
  };
  const L: string[] = [];
  const d = r.drift;
  L.push(
    `[${t.toLocaleTimeString()}] ${r.id.cluster ? "Cluster" : `${NOTE_NAMES[r.id.root]} ${r.id.chord.name}`}${r.id.chord.ratio !== "—" ? ` (${r.id.chord.ratio})` : ""} · consonance ${r.cons == null ? "—" : `${r.cons}%`} · target ${PROFILES[cfg.profile].name} · A4=${cfg.a4} · drift ${d == null ? "—" : `${d >= 0 ? "+" : ""}${d.toFixed(1)}c`}`,
  );
  r.voices.forEach((v, i) => {
    const j = r.tg[i];
    L.push(
      `  ${sec[v.section].padEnd(7)} ${v.name}${v.oct} ${v.f.toFixed(2)} Hz  ET ${v.cents >= 0 ? "+" : ""}${v.cents.toFixed(1)}c${v.scatter != null ? `  scatter ±${v.scatter.toFixed(0)}c` : ""}${j ? (j.anchor ? `  ANCHOR (${j.role})` : `  target ${j.err >= 0 ? "+" : ""}${j.err.toFixed(1)}c (${j.role} ${j.ratioTxt} → ${j.tgt.toFixed(1)} Hz)`) : ""}`,
    );
  });
  for (const p of r.pairs)
    L.push(
      `  ${p.a.name}${p.a.oct}–${p.b.name}${p.b.oct} ${p.name} ${p.ratio}: ${p.cents.toFixed(1)}c (err ${p.err >= 0 ? "+" : ""}${p.err.toFixed(1)}c) beats ${p.beat.toFixed(2)} Hz`,
    );
  return L.join("\n");
}
