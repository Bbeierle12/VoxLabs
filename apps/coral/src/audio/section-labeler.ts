/**
 * SATB section labeler (MVP). Maps a set of detected MIDI notes to voice
 * sections — Soprano / Alto / Tenor / Bass — using pitch-range priors, with an
 * explicit `A/T?` ambiguity class for the alto/tenor overlap register.
 *
 * This is **pitch-range labeling (A)**, the honest MVP: it assigns by where a
 * note sits and how the chord is ordered, NOT by separating voices. The
 * physics is unambiguous on its limits (docs/vocal-acoustics-research.md):
 * alto and tenor ranges overlap around G3–A4 and are near-chance to tell apart
 * monaurally, so a mid note that pitch-order can't pin is labeled `A/T?` rather
 * than guessed. An optional singer's-formant (SPR) tiebreaker — injected by the
 * caller, since computing it needs the spectrum — can resolve the `A/T?` zone;
 * whether it actually helps is for P3 to measure.
 *
 * Assignment is a small monotonic DP over per-note range fit (labels stay
 * non-decreasing in pitch; a divisi penalty discourages collapsing distinct
 * voices into one section). Confidence (0..1) folds range fit, the detector's
 * per-note confidence, and (in the overlap) SPR clarity — a soft value.
 */

export type Section = "S" | "A" | "T" | "B";
export type Label = Section | "A/T?";

export interface SectionLabel {
  midi: number;
  label: Label;
  /** The DP's provisional section — the card a `A/T?` note is shown on. */
  guess: Section;
  /** 0..1 — soft. Low in the A/T overlap; lower still for `A/T?`. */
  confidence: number;
}

export interface LabelOptions {
  /**
   * Per-note detector confidence (e.g. `NoteDetector.lastConfidences()`):
   * smoothed salience / threshold, so ≥1 means active. Scales confidence.
   */
  confidence?: ReadonlyMap<number, number>;
  /**
   * Singer's-formant lean for a mid-register note, in [-1, 1]: +1 = clearly
   * alto-bright, -1 = clearly tenor-dark, 0 = ambiguous. Used ONLY to break the
   * A/T overlap. Omit to fall back to pitch-order bracketing.
   */
  spr?: (midi: number) => number;
}

/** Typical SATB MIDI ranges (overlapping), from the vocal-acoustics brief. */
const RANGES: Record<Section, readonly [number, number]> = {
  B: [40, 64], // E2–E4
  T: [48, 69], // C3–A4
  A: [53, 77], // F3–F5
  S: [60, 84], // C4–C6
};
/** Sections low→high; DP labels stay non-decreasing along this order. */
const ASC: readonly Section[] = ["B", "T", "A", "S"];
const RANK: Record<Section, number> = { B: 0, T: 1, A: 2, S: 3 };
/** Alto/tenor overlap register where the two are near-chance to distinguish. */
const AT_LO = 55; // G3
const AT_HI = 69; // A4
/** |SPR lean| below this is too weak to resolve A vs T. */
const SPR_MARGIN = 0.3;
/** Fit penalty for assigning two notes to the same section (discourages divisi). */
const REPEAT_PENALTY = 0.5;

function rangeFit(s: Section, m: number): number {
  const [lo, hi] = RANGES[s];
  if (m < lo || m > hi) {
    const over = m < lo ? lo - m : m - hi;
    return Math.max(0.02, 0.25 * Math.exp(-over / 6)); // outside range: small, decaying
  }
  const center = (lo + hi) / 2;
  const half = (hi - lo) / 2;
  return Math.max(0.3, 1 - Math.abs(m - center) / half);
}

/**
 * Label detected notes by SATB section. `notes` is any iterable of MIDI numbers
 * (deduped + sorted internally).
 */
export function labelSections(
  notes: Iterable<number>,
  opts: LabelOptions = {},
): SectionLabel[] {
  const sorted = Array.from(new Set(notes)).sort((a, b) => a - b);
  const n = sorted.length;
  if (n === 0) return [];
  const NS = ASC.length;

  // Monotonic DP: dp[s] = best total range-fit ending with this note in section
  // s, given the previous note was in some section ≤ s.
  let dp = new Float64Array(NS);
  for (let s = 0; s < NS; s++)
    dp[s] = rangeFit(ASC[s] as Section, sorted[0] as number);
  const back: Int32Array[] = [new Int32Array(NS).fill(-1)];

  for (let i = 1; i < n; i++) {
    const cur = new Float64Array(NS);
    const bk = new Int32Array(NS);
    for (let s = 0; s < NS; s++) {
      let bestPrev = -Infinity;
      let bestS = 0;
      for (let sp = 0; sp <= s; sp++) {
        const val = (dp[sp] as number) - (sp === s ? REPEAT_PENALTY : 0);
        if (val > bestPrev) {
          bestPrev = val;
          bestS = sp;
        }
      }
      cur[s] = rangeFit(ASC[s] as Section, sorted[i] as number) + bestPrev;
      bk[s] = bestS;
    }
    dp = cur;
    back.push(bk);
  }

  // Reconstruct.
  let s = 0;
  for (let k = 1; k < NS; k++) if ((dp[k] as number) > (dp[s] as number)) s = k;
  const provisional: Section[] = new Array(n);
  for (let i = n - 1; i >= 0; i--) {
    provisional[i] = ASC[s] as Section;
    s = (back[i] as Int32Array)[s] as number;
    if (s < 0) s = 0;
  }

  return sorted.map((m, i) => refine(m, i, provisional, opts));
}

function bracketed(i: number, provisional: Section[]): boolean {
  const rank = RANK[provisional[i] as Section];
  let below = false;
  let above = false;
  for (let j = 0; j < i; j++)
    if (RANK[provisional[j] as Section] < rank) below = true;
  for (let j = i + 1; j < provisional.length; j++) {
    if (RANK[provisional[j] as Section] > rank) above = true;
  }
  return below && above;
}

function refine(
  m: number,
  i: number,
  provisional: Section[],
  opts: LabelOptions,
): SectionLabel {
  const guess = provisional[i] as Section;
  const detFactor = opts.confidence
    ? Math.max(0.3, Math.min(1, (opts.confidence.get(m) ?? 1) / 2))
    : 1;
  const inAtZone = m >= AT_LO && m <= AT_HI && (guess === "A" || guess === "T");

  if (!inAtZone) {
    return {
      midi: m,
      label: guess,
      guess,
      confidence: round2(rangeFit(guess, m) * detFactor),
    };
  }

  // A/T overlap: SPR first, then pitch-order bracketing, else declare ambiguous.
  if (opts.spr) {
    const lean = opts.spr(m); // +alto / -tenor
    if (Math.abs(lean) >= SPR_MARGIN) {
      const label: Section = lean > 0 ? "A" : "T";
      return {
        midi: m,
        label,
        guess: label,
        confidence: round2(Math.min(0.9, Math.abs(lean)) * detFactor),
      };
    }
    return {
      midi: m,
      label: "A/T?",
      guess,
      confidence: round2(0.3 * detFactor),
    };
  }
  if (bracketed(i, provisional)) {
    // Pitch order pins it (a section strictly below AND above) — keep the
    // guess, but it's the hard register, so discount.
    return {
      midi: m,
      label: guess,
      guess,
      confidence: round2(0.6 * rangeFit(guess, m) * detFactor),
    };
  }
  return { midi: m, label: "A/T?", guess, confidence: round2(0.3 * detFactor) };
}

function round2(x: number): number {
  return Math.round(x * 100) / 100;
}

/**
 * Per-note singer's-formant lean for the A/T tiebreaker, in [-1, 1]
 * (+alto / -tenor). Measures the energy-weighted frequency centroid of the
 * note's harmonics inside the 2–4 kHz singer's-formant band relative to the
 * tenor/alto formant midpoint (~2800 Hz): alto's formant sits higher (~2900)
 * than tenor's (~2705), so a higher centroid leans alto.
 *
 * Uses the RAW magnitude spectrum (NOT the whitened one — whitening flattens
 * exactly the formant this reads). Returns 0 (ambiguous) when the note has no
 * energy in the band.
 *
 * EVALUATED (P3) and NOT adopted: feeding this as the labeler's `spr` made
 * A/T-zone labels WORSE (correct 56%→31%, wrong 6%→25%) — in a monaural mixture
 * the band is contaminated by other voices' harmonics, exactly as the brief
 * warned. The production path supplies NO spr (pitch-order + `A/T?` abstention
 * scores 70% correct / 4% wrong). Retained as a reference for a future,
 * better-isolated SPR feature.
 */
export function sprLean(
  midi: number,
  mags: Float32Array,
  sampleRate: number,
): number {
  const f0 = 440 * Math.pow(2, (midi - 69) / 12);
  const binFreqHz = sampleRate / (2 * mags.length);
  const kLo = Math.max(1, Math.ceil(2000 / f0));
  const kHi = Math.floor(4000 / f0);
  let wsum = 0;
  let esum = 0;
  for (let k = kLo; k <= kHi; k++) {
    const fk = k * f0;
    const center = Math.round(fk / binFreqHz);
    let peak = 0;
    for (
      let b = Math.max(0, center - 2);
      b <= Math.min(mags.length - 1, center + 2);
      b++
    ) {
      const m = mags[b] as number;
      if (m > peak) peak = m;
    }
    wsum += fk * peak;
    esum += peak;
  }
  if (esum <= 0) return 0;
  const centroid = wsum / esum;
  return Math.max(-1, Math.min(1, (centroid - 2800) / 250));
}
