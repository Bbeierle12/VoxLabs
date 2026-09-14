import {
  synthesizeChoirChord,
  type Section,
  type ChoirChordSpec,
} from "./choir-synth";

/**
 * Synthetic multi-F0 benchmark + scorer — the measurement backbone for the
 * detector-robustness workstream. Pure: it takes a `detect` function (the
 * caller wires the real pipeline), runs it over a tagged battery of SATB
 * chords, and reports note-set precision/recall/F1 overall AND grouped by
 * condition tag (wide/close/cluster/crossing/octave-doubling/register/spread).
 *
 * Every tier (T1 sum-in-band, T2 cancellation, T3 CQT, T4 temporal) is gated
 * against this same battery so improvements and regressions are visible per
 * condition, not just in aggregate.
 */

export interface BenchVoice {
  section: Section;
  midi: number;
  /** Explicit per-harmonic dB profile (see VoiceSpec.harmonicDb). */
  harmonicDb?: readonly number[];
  /** Singers in the section (default 8; 1 = a soloist). */
  singers?: number;
  /** Per-singer detune SD, cents (voice-level override). */
  detuneCents?: number;
  /** Vibrato depth, cents. */
  vibratoCents?: number;
  /** Linear gain relative to other voices. */
  gain?: number;
}

export interface BenchCase {
  name: string;
  voices: BenchVoice[];
  /** Per-section detune SD override, cents (for spread-sensitivity cases). */
  detuneCents?: number;
  /** Condition tags for grouped scoring. */
  tags: string[];
  /** Chord-level synth overrides (noise floor, formant, roll-off, …). */
  synth?: Partial<
    Omit<ChoirChordSpec, "sampleRate" | "durationSec" | "voices">
  >;
}

export interface Metrics {
  tp: number;
  fp: number;
  fn: number;
  precision: number;
  recall: number;
  f1: number;
}

export interface CaseResult {
  name: string;
  tags: string[];
  gt: number[];
  detected: number[];
  tp: number;
  fp: number;
  fn: number;
}

export interface BenchReport {
  overall: Metrics;
  byTag: Record<string, Metrics>;
  cases: CaseResult[];
}

const v = (
  section: Section,
  midi: number,
  extra: Partial<BenchVoice> = {},
): BenchVoice => ({ section, midi, ...extra });
const SATB = (b: number, t: number, a: number, s: number): BenchVoice[] => [
  v("B", b),
  v("T", t),
  v("A", a),
  v("S", s),
];

function gtOf(voices: BenchVoice[]): number[] {
  return Array.from(new Set(voices.map((x) => x.midi))).sort((a, b) => a - b);
}

/** The benchmark battery. Deterministic; extend freely (keep tags meaningful). */
export const BENCH_CASES: BenchCase[] = [
  // Well-voiced chords across keys/registers.
  { name: "Cmaj wide", voices: SATB(48, 55, 64, 72), tags: ["wide", "mid"] },
  { name: "Fmaj close", voices: SATB(53, 57, 60, 65), tags: ["close", "mid"] },
  { name: "Gmaj wide", voices: SATB(55, 59, 62, 67), tags: ["wide", "mid"] },
  { name: "Dmin wide", voices: SATB(50, 57, 62, 69), tags: ["wide", "mid"] },
  { name: "Amaj close", voices: SATB(57, 61, 64, 69), tags: ["close", "mid"] },
  {
    name: "CEG triad",
    voices: [v("B", 60), v("T", 64), v("A", 67)],
    tags: ["close", "triad"],
  },
  // Register stress.
  {
    name: "high chord",
    voices: SATB(60, 67, 72, 79),
    tags: ["high", "soprano"],
  },
  { name: "low chord", voices: SATB(40, 48, 55, 60), tags: ["low", "bass"] },
  // Harmonic-interval doublings (the hard cases).
  {
    name: "octave B+S",
    voices: [v("B", 48), v("S", 60)],
    tags: ["octave-dbl"],
  },
  { name: "double-oct", voices: [v("B", 48), v("S", 72)], tags: ["dbl-2oct"] },
  { name: "twelfth", voices: [v("B", 48), v("S", 67)], tags: ["twelfth"] },
  {
    name: "oct-stack G2-G4",
    voices: SATB(43, 55, 62, 67),
    tags: ["octave-dbl", "wide"],
  },
  // Cluster & crossing.
  { name: "cluster CDEF", voices: SATB(60, 62, 64, 65), tags: ["cluster"] },
  { name: "crossing A>S", voices: SATB(55, 60, 69, 67), tags: ["crossing"] },
  // Unison doubling and a sanity single note.
  {
    name: "unison T=A",
    voices: [v("B", 48), v("T", 60), v("A", 60), v("S", 72)],
    tags: ["unison"],
  },
  { name: "single A4", voices: [v("A", 69)], tags: ["single"] },
  // Spread sensitivity (same chord, three detune levels).
  {
    name: "Cmaj wide d15",
    voices: SATB(48, 55, 64, 72),
    detuneCents: 15,
    tags: ["spread", "spread15"],
  },
  {
    name: "Cmaj wide d25",
    voices: SATB(48, 55, 64, 72),
    detuneCents: 25,
    tags: ["spread", "spread25"],
  },
  {
    name: "Cmaj wide d35",
    voices: SATB(48, 55, 64, 72),
    detuneCents: 35,
    tags: ["spread", "spread35"],
  },

  // ---- Single voices with realistic harmonic profiles (added in the Analyzer merge). ----
  // A chest-voice baritone on an open vowel puts H2/H3 ABOVE H1; the roll-off model above never
  // renders that, which is why the detector's octave/twelfth phantoms on one real singer went unseen.
  {
    name: "solo baritone A2 chest",
    voices: [
      v("B", 45, {
        harmonicDb: [-20, -8, -9, -16, -12, -22, -26, -30, -34, -38],
        singers: 1,
        detuneCents: 0,
      }),
    ],
    tags: ["single", "strong-h2"],
  },
  {
    name: "solo baritone D3 H2+8",
    voices: [
      v("B", 50, {
        harmonicDb: [-14, -6, -12, -10, -18, -24, -28, -32],
        singers: 1,
        detuneCents: 0,
      }),
    ],
    tags: ["single", "strong-h2"],
  },
  {
    name: "solo G3 H2+12",
    voices: [
      v("T", 55, {
        harmonicDb: [-16, -4, -10, -12, -14, -20, -24, -28],
        singers: 1,
        detuneCents: 0,
      }),
    ],
    tags: ["single", "strong-h2"],
  },
  {
    name: "solo B2 formant on H4",
    voices: [
      v("B", 47, {
        harmonicDb: [-18, -9, -14, -6, -20, -26, -30, -34],
        singers: 1,
        detuneCents: 0,
      }),
    ],
    tags: ["single", "formant"],
  },
  {
    name: "solo A2 H2=H1",
    voices: [
      v("B", 45, {
        harmonicDb: [-10, -10, -14, -18, -22, -26, -30, -34],
        singers: 1,
        detuneCents: 0,
      }),
    ],
    tags: ["single", "strong-h2"],
  },
  {
    name: "solo A4 soprano",
    voices: [
      v("S", 69, {
        harmonicDb: [-8, -14, -20, -26, -32, -38],
        singers: 1,
        detuneCents: 0,
      }),
    ],
    tags: ["single", "soprano"],
  },
  // Section of four with chest-voice profile (spread + strong H2 together).
  {
    name: "tenor section E3 chest",
    voices: [
      v("T", 52, {
        harmonicDb: [-16, -6, -9, -14, -18, -24, -28, -32],
        singers: 4,
      }),
    ],
    tags: ["single-section", "strong-h2"],
  },
  // Breath / room noise over a chest-voice baritone.
  {
    name: "baritone + breath −35 dB",
    voices: [
      v("B", 45, {
        harmonicDb: [-20, -8, -9, -16, -12, -22, -26, -30],
        singers: 1,
        detuneCents: 0,
      }),
    ],
    synth: { noiseFloor: 0.018 },
    tags: ["single", "breath"],
  },
  {
    name: "baritone + room −45 dB",
    voices: [
      v("B", 45, {
        harmonicDb: [-20, -8, -9, -16, -12, -22, -26, -30],
        singers: 1,
        detuneCents: 0,
      }),
    ],
    synth: { noiseFloor: 0.0056 },
    tags: ["single", "breath"],
  },
  // Octave partner at three levels relative to the bass fundamental.
  {
    name: "bass C3 + sop C5 (0 dB)",
    voices: [
      v("B", 48, {
        harmonicDb: [-10, -14, -18, -24, -28, -32, -36, -40],
        singers: 1,
        detuneCents: 0,
      }),
      v("S", 72, {
        harmonicDb: [-10, -19, -28, -37],
        singers: 1,
        detuneCents: 0,
      }),
    ],
    tags: ["oct-partner", "octave-dbl"],
  },
  {
    name: "bass C3 + sop C5 (−6 dB)",
    voices: [
      v("B", 48, {
        harmonicDb: [-10, -14, -18, -24, -28, -32, -36, -40],
        singers: 1,
        detuneCents: 0,
      }),
      v("S", 72, {
        harmonicDb: [-16, -25, -34, -43],
        singers: 1,
        detuneCents: 0,
      }),
    ],
    tags: ["oct-partner", "octave-dbl"],
  },
  {
    name: "bass A2 chest + tenor A3",
    voices: [
      v("B", 45, {
        harmonicDb: [-20, -8, -9, -16, -12, -22, -26, -30],
        singers: 1,
        detuneCents: 0,
      }),
      v("T", 57, {
        harmonicDb: [-12, -8, -10, -14, -18, -22, -26, -30],
        singers: 1,
        detuneCents: 0,
      }),
    ],
    tags: ["oct-partner", "octave-dbl", "strong-h2"],
  },
  {
    name: "bass A2 chest + tenor E3",
    voices: [
      v("B", 45, {
        harmonicDb: [-20, -8, -9, -16, -12, -22, -26, -30],
        singers: 1,
        detuneCents: 0,
      }),
      v("T", 52, {
        harmonicDb: [-12, -8, -10, -14, -18, -22, -26, -30],
        singers: 1,
        detuneCents: 0,
      }),
    ],
    tags: ["twelfth-partner", "strong-h2"],
  },
];

function metricsOf(tp: number, fp: number, fn: number): Metrics {
  const precision = tp + fp > 0 ? tp / (tp + fp) : 0;
  const recall = tp + fn > 0 ? tp / (tp + fn) : 0;
  const f1 =
    precision + recall > 0
      ? (2 * precision * recall) / (precision + recall)
      : 0;
  return { tp, fp, fn, precision, recall, f1 };
}

/**
 * Run `detect` (caller-supplied; wires the real pipeline) over the cases and
 * score note-set accuracy overall + by tag.
 */
export function runBench(
  cases: BenchCase[],
  detect: (samples: Float32Array) => Set<number>,
  opts: { sampleRate?: number; durationSec?: number; seed?: number } = {},
): BenchReport {
  const sampleRate = opts.sampleRate ?? 44100;
  const durationSec = opts.durationSec ?? 0.4;
  const seed = opts.seed ?? 7;

  const results: CaseResult[] = [];
  const tagAcc = new Map<string, { tp: number; fp: number; fn: number }>();
  let TP = 0;
  let FP = 0;
  let FN = 0;

  for (const c of cases) {
    const voices = c.voices.map((x) => ({
      ...x,
      detuneCents: x.detuneCents ?? c.detuneCents,
    }));
    const { samples } = synthesizeChoirChord({
      sampleRate,
      durationSec,
      seed,
      voices,
      ...(c.synth ?? {}),
    });
    const det = detect(samples);
    const gt = gtOf(c.voices);
    const gtSet = new Set(gt);
    let tp = 0;
    let fp = 0;
    for (const m of det) gtSet.has(m) ? tp++ : fp++;
    const fn = gt.length - tp;

    results.push({
      name: c.name,
      tags: c.tags,
      gt,
      detected: Array.from(det).sort((a, b) => a - b),
      tp,
      fp,
      fn,
    });
    TP += tp;
    FP += fp;
    FN += fn;
    for (const tag of c.tags) {
      let a = tagAcc.get(tag);
      if (!a) {
        a = { tp: 0, fp: 0, fn: 0 };
        tagAcc.set(tag, a);
      }
      a.tp += tp;
      a.fp += fp;
      a.fn += fn;
    }
  }

  const byTag: Record<string, Metrics> = {};
  for (const [tag, a] of tagAcc) byTag[tag] = metricsOf(a.tp, a.fp, a.fn);
  return { overall: metricsOf(TP, FP, FN), byTag, cases: results };
}

/** Human-readable report (per-case detail + overall + by-tag) for on-demand logging. */
export function formatReport(rep: BenchReport): string {
  const lines: string[] = [];
  for (const c of rep.cases) {
    const gtSet = new Set(c.gt);
    const detSet = new Set(c.detected);
    const ph = c.detected.filter((m) => !gtSet.has(m));
    const miss = c.gt.filter((m) => !detSet.has(m));
    lines.push(
      `${c.name.padEnd(20)} [${c.gt.join(",")}] -> [${c.detected.join(",")}]` +
        (ph.length ? ` +ph[${ph.join(",")}]` : "") +
        (miss.length ? ` -miss[${miss.join(",")}]` : ""),
    );
  }
  const fmt = (m: Metrics) =>
    `P=${m.precision.toFixed(3)} R=${m.recall.toFixed(3)} F1=${m.f1.toFixed(3)} (tp=${m.tp} fp=${m.fp} fn=${m.fn})`;
  lines.push("");
  lines.push(`OVERALL  ${fmt(rep.overall)}`);
  lines.push("BY TAG:");
  for (const tag of Object.keys(rep.byTag).sort()) {
    lines.push(`  ${tag.padEnd(12)} ${fmt(rep.byTag[tag] as Metrics)}`);
  }
  return lines.join("\n");
}
