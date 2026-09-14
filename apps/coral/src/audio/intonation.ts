/**
 * Intonation analysis: turn F0 estimates into equal-temperament note names +
 * signed cents deviation, and summarize a clip's tuning. ET-only for v1 (the
 * neutral reference a piano-trained choir tunes against); just-intonation is
 * deferred. Sharps for accidentals.
 */

const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

export interface NoteCents {
  midi: number;
  /** e.g. "A4". */
  noteName: string;
  /** Signed deviation from the nearest ET note; + = sharp. */
  cents: number;
}

/** Frequency → nearest ET note + cents deviation (A4 = 440, MIDI 69). */
export function freqToNoteCents(freqHz: number): NoteCents {
  const midiFloat = 69 + 12 * Math.log2(freqHz / 440);
  const midi = Math.round(midiFloat);
  const cents = (midiFloat - midi) * 100;
  const pc = ((midi % 12) + 12) % 12;
  const octave = Math.floor(midi / 12) - 1;
  return { midi, noteName: `${NOTE_NAMES[pc]}${octave}`, cents };
}

export interface IntonationSummary {
  voicedFraction: number;
  /** [lowest, highest] note names over voiced frames, or null if none. */
  pitchRange: [string, string] | null;
  /** Signed mean cents-from-nearest over voiced frames. */
  meanCents: number;
  rmsCents: number;
  /** Detected vibrato rate (3–8 Hz band), or null. */
  vibratoRateHz: number | null;
}

function mean(xs: number[]): number {
  return xs.length ? xs.reduce((s, v) => s + v, 0) / xs.length : 0;
}

function median(xs: number[]): number {
  const s = [...xs].sort((a, b) => a - b);
  return s.length ? (s[s.length >> 1] as number) : 0;
}

/**
 * Summarize a sequence of per-frame F0 values (null = unvoiced) at the given
 * frame rate (sampleRate / hopSize).
 */
export function summarize(
  frames: ReadonlyArray<number | null>,
  hopSize: number,
  sampleRate: number
): IntonationSummary {
  const voiced: number[] = [];
  for (const f of frames) if (f != null && f > 0) voiced.push(f);
  const voicedFraction = frames.length ? voiced.length / frames.length : 0;
  if (voiced.length === 0) {
    return { voicedFraction: 0, pitchRange: null, meanCents: 0, rmsCents: 0, vibratoRateHz: null };
  }

  let lo = Infinity;
  let hi = -Infinity;
  const cents: number[] = [];
  for (const f of voiced) {
    if (f < lo) lo = f;
    if (f > hi) hi = f;
    cents.push(freqToNoteCents(f).cents);
  }
  const meanCents = mean(cents);
  const rmsCents = Math.sqrt(mean(cents.map((c) => c * c)));
  const pitchRange: [string, string] = [
    freqToNoteCents(lo).noteName,
    freqToNoteCents(hi).noteName,
  ];
  const vibratoRateHz = detectVibrato(voiced, hopSize, sampleRate);
  return { voicedFraction, pitchRange, meanCents, rmsCents, vibratoRateHz };
}

/**
 * Vibrato rate via autocorrelation of the (detrended) cents-vs-median signal,
 * peak-picked in the 3–8 Hz band. Null if too short or not periodic enough.
 */
function detectVibrato(f0s: number[], hopSize: number, sampleRate: number): number | null {
  if (f0s.length < 24) return null;
  const med = median(f0s);
  if (!(med > 0)) return null;
  const dev = f0s.map((f) => 1200 * Math.log2(f / med));
  const m = mean(dev);
  const x = dev.map((d) => d - m);
  const energy = x.reduce((s, v) => s + v * v, 0);
  if (energy <= 0) return null;

  const frameRate = sampleRate / hopSize;
  const minLag = Math.max(1, Math.ceil(frameRate / 8));
  const maxLag = Math.min(x.length - 1, Math.floor(frameRate / 3));

  const corrAt = (lag: number): number => {
    if (lag < 1 || lag >= x.length) return 0;
    let c = 0;
    for (let i = 0; i + lag < x.length; i++) c += (x[i] as number) * (x[i + lag] as number);
    return c / energy;
  };

  let bestLag = -1;
  let bestCorr = 0;
  for (let lag = minLag; lag <= maxLag; lag++) {
    const norm = corrAt(lag);
    if (norm > bestCorr) {
      bestCorr = norm;
      bestLag = lag;
    }
  }
  if (bestLag < 0 || bestCorr < 0.3) return null;

  // Guard against period-multiple aliasing (e.g. 9 Hz masquerading as 4.5 Hz).
  if (bestLag >= 2) {
    const halfLagLower = Math.floor(bestLag / 2);
    const halfLagUpper = Math.ceil(bestLag / 2);
    const halfCorr = Math.max(corrAt(halfLagLower), corrAt(halfLagUpper));
    if (halfCorr > 0.8 * bestCorr) {
      return null;
    }
  }

  return frameRate / bestLag;
}
