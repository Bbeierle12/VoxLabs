import { describe, it, expect } from 'vitest';
import { NoteDetector } from './note-detector';

const SAMPLE_RATE = 44100;
const NUM_BINS = 1024; // fftSize 2048
const BIN_FREQ_HZ = SAMPLE_RATE / (2 * NUM_BINS); // ≈ 21.533

const DEFAULT_OPTS = {
  numBins: NUM_BINS,
  sampleRate: SAMPLE_RATE,
  minFreqHz: 50,
  maxFreqHz: 8000,
};

/** Magnitudes array with one bin set; everything else zero. */
function spikeAt(bin: number, value: number): Float32Array {
  const m = new Float32Array(NUM_BINS);
  m[bin] = value;
  return m;
}

/** Multi-bin spike — for polyphony tests. */
function spikes(pairs: ReadonlyArray<readonly [number, number]>): Float32Array {
  const m = new Float32Array(NUM_BINS);
  for (const [bin, value] of pairs) m[bin] = value;
  return m;
}

describe('NoteDetector', () => {
  // Test 1
  it('maps A4 (440 Hz) to MIDI 69', () => {
    const d = new NoteDetector(DEFAULT_OPTS);
    expect(d.midiMin).toBeLessThanOrEqual(69);
    expect(d.midiMax).toBeGreaterThanOrEqual(69);
    const range = d.binRangeForMidi(69);
    expect(range).not.toBeNull();
    // 440 Hz / binFreqHz ≈ 20.43, so the central bin is 20.
    const a4Bin = Math.round(440 / BIN_FREQ_HZ);
    expect(range![0]).toBeLessThanOrEqual(a4Bin);
    expect(range![1]).toBeGreaterThanOrEqual(a4Bin);
  });

  // Test 2 — bin range follows the formula [floor(f/hs / bw), ceil(f*hs / bw)].
  it('computes per-MIDI bin ranges from the ±quarter-tone formula', () => {
    const d = new NoteDetector(DEFAULT_OPTS);
    const halfStep = Math.pow(2, 1 / 24);
    for (const midi of [60, 69, 72, 96]) {
      // C4, A4, C5, C7
      const freq = 440 * Math.pow(2, (midi - 69) / 12);
      const expectedLo = Math.max(0, Math.floor(freq / halfStep / BIN_FREQ_HZ));
      const expectedHi = Math.min(
        NUM_BINS - 1,
        Math.ceil((freq * halfStep) / BIN_FREQ_HZ)
      );
      const range = d.binRangeForMidi(midi);
      expect(range).not.toBeNull();
      expect(range![0]).toBe(expectedLo);
      expect(range![1]).toBe(expectedHi);
    }
  });

  // Test 3
  it('excludes notes outside [minFreqHz, maxFreqHz]', () => {
    const narrow = new NoteDetector({
      ...DEFAULT_OPTS,
      minFreqHz: 200,
      maxFreqHz: 2000,
    });
    // A2 = 110 Hz → MIDI 45 → below 200 Hz, excluded.
    expect(narrow.binRangeForMidi(45)).toBeNull();
    // A6 = 1760 Hz → MIDI 93 → inside, included.
    expect(narrow.binRangeForMidi(93)).not.toBeNull();
    // Verify in-range detection works.
    const wide = new NoteDetector(DEFAULT_OPTS);
    expect(wide.binRangeForMidi(45)).not.toBeNull();
  });

  // Test 4
  it('returns an empty set for silence', () => {
    const d = new NoteDetector(DEFAULT_OPTS);
    const active = d.analyze(new Float32Array(NUM_BINS));
    expect(active.size).toBe(0);
  });

  // Test 5 — threshold boundary at -50 dB (default).
  it('triggers above the threshold, not below', () => {
    // Bin 97 is in C7's range [94,101] only (not in B6 or C#7).
    const C7_ONLY_BIN = 97;
    const C7_MIDI = 96;

    // Just above threshold: -49.5 dB → 10^(-49.5/20) ≈ 0.003350.
    const above = new NoteDetector(DEFAULT_OPTS);
    const activeAbove = above.analyze(spikeAt(C7_ONLY_BIN, Math.pow(10, -49.5 / 20)));
    expect(activeAbove.has(C7_MIDI)).toBe(true);

    // Just below threshold: -50.5 dB → 10^(-50.5/20) ≈ 0.002985.
    const below = new NoteDetector(DEFAULT_OPTS);
    const activeBelow = below.analyze(spikeAt(C7_ONLY_BIN, Math.pow(10, -50.5 / 20)));
    expect(activeBelow.has(C7_MIDI)).toBe(false);
  });

  // Test 6 — a single bin lit, inside exactly one note's range, lights that note.
  it('activates a single note when energy is in its band', () => {
    const d = new NoteDetector(DEFAULT_OPTS);
    // Bin 97 is uniquely inside C7's band at this fft size.
    const active = d.analyze(spikeAt(97, 1.0));
    expect(active.has(96)).toBe(true);
    // Adjacent semitones whose bands don't overlap bin 97 should NOT
    // activate — B6 covers [89,95], C#7 covers [100,106].
    expect(active.has(95)).toBe(false); // B6
    expect(active.has(97)).toBe(false); // C#7
  });

  // Test 7 — three independent peaks light three independent notes.
  it('detects a polyphonic chord (C7-E7-G7)', () => {
    const d = new NoteDetector(DEFAULT_OPTS);
    // Bins chosen to fall uniquely inside each note's band at
    // numBins=1024 / sampleRate=44100: C7=bin 97, E7=bin 122, G7=bin 145.
    const active = d.analyze(spikes([
      [97, 1.0],
      [122, 1.0],
      [145, 1.0],
    ]));
    expect(active.has(96)).toBe(true); // C7
    expect(active.has(100)).toBe(true); // E7
    expect(active.has(103)).toBe(true); // G7
  });

  // Test 8 — decay rate gives the expected release time.
  it('decays a previously-active note over the expected number of frames', () => {
    const d = new NoteDetector(DEFAULT_OPTS);
    // Loud opening frame: peak 0.5 at C7, ≈ -6 dBFS.
    d.analyze(spikeAt(97, 0.5));
    // Count silent frames until C7 leaves the active set.
    const silent = new Float32Array(NUM_BINS);
    let framesUntilSilent = 0;
    const C7_MIDI = 96;
    while (framesUntilSilent < 200) {
      const active = d.analyze(silent);
      if (!active.has(C7_MIDI)) break;
      framesUntilSilent++;
    }
    // Algebra: 0.5 * 0.92^n < 10^(-50/20) ≈ 0.003162 →
    // n > log(0.006325)/log(0.92) ≈ 60.7. At default cadence (~86 fps)
    // that's ~700 ms. Allow a generous tolerance to absorb float precision.
    expect(framesUntilSilent).toBeGreaterThan(55);
    expect(framesUntilSilent).toBeLessThan(70);
  });

  // Test 9 — reset() clears decay state.
  it('clears decay state on reset()', () => {
    const d = new NoteDetector(DEFAULT_OPTS);
    // Activate C7.
    const first = d.analyze(spikeAt(97, 1.0));
    expect(first.has(96)).toBe(true);
    d.reset();
    // Without reset(), the next silent frame would still find C7 above
    // threshold (since 1.0 × 0.92 = 0.92, still loud). After reset, the
    // energy buffer starts at zero, so a silent frame yields nothing.
    const afterReset = d.analyze(new Float32Array(NUM_BINS));
    expect(afterReset.size).toBe(0);
  });

  it('rejects invalid construction options', () => {
    expect(() => new NoteDetector({ ...DEFAULT_OPTS, numBins: 0 })).toThrow();
    expect(() => new NoteDetector({ ...DEFAULT_OPTS, sampleRate: -1 })).toThrow();
    expect(() => new NoteDetector({ ...DEFAULT_OPTS, minFreqHz: 0 })).toThrow();
    expect(() => new NoteDetector({ ...DEFAULT_OPTS, maxFreqHz: 50 })).toThrow();
    expect(
      () => new NoteDetector({ ...DEFAULT_OPTS, decayPerFrame: 1.1 })
    ).toThrow();
  });
});

/** Place peaks at k·f0 for k=1..count, amplitude amp/k. */
function harmonicStack(f0: number, count: number, amp = 1): Float32Array {
  const m = new Float32Array(NUM_BINS);
  for (let k = 1; k <= count; k++) {
    const bin = Math.round((k * f0) / BIN_FREQ_HZ);
    if (bin >= 0 && bin < NUM_BINS) m[bin] = amp / k;
  }
  return m;
}

describe('NoteDetector — harmonic mode', () => {
  const HARMONIC_OPTS = { ...DEFAULT_OPTS, harmonic: true };

  // The headline behavior: a single voice's overtone stack reads as ONE
  // fundamental, with the octave-above (its 2nd harmonic) suppressed
  // instead of reported as a separate note.
  it('detects a harmonic stack as one fundamental, not its octave', () => {
    const d = new NoteDetector(HARMONIC_OPTS);
    // A3 = 220 Hz → MIDI 57. 2nd harmonic = A4 = 440 Hz → MIDI 69.
    const active = d.analyze(harmonicStack(220, 7));
    expect(active.has(57)).toBe(true); // fundamental detected
    expect(active.has(69)).toBe(false); // octave phantom suppressed
  });

  // Whitening + per-note normalization means featureless energy, however
  // loud, sits at the flat baseline (1.0) and never crosses the threshold.
  // The legacy path, judging raw magnitude, lights every band.
  it('whitening rejects a flat spectrum that the legacy path flags', () => {
    const flat = new Float32Array(NUM_BINS).fill(0.5); // ≈ -6 dBFS everywhere

    const legacy = new NoteDetector(DEFAULT_OPTS);
    expect(legacy.analyze(flat).size).toBeGreaterThan(0);

    const harmonic = new NoteDetector(HARMONIC_OPTS);
    expect(harmonic.analyze(flat).size).toBe(0);
  });

  it('rejects invalid harmonic options (only when harmonic is on)', () => {
    expect(
      () => new NoteDetector({ ...HARMONIC_OPTS, harmonicCount: 0 })
    ).toThrow();
    expect(
      () => new NoteDetector({ ...HARMONIC_OPTS, salienceThreshold: 0 })
    ).toThrow();
    expect(
      () => new NoteDetector({ ...HARMONIC_OPTS, whiteningWindowBins: 0 })
    ).toThrow();
    // The same out-of-range values are inert in legacy mode.
    expect(
      () => new NoteDetector({ ...DEFAULT_OPTS, harmonicCount: 0 })
    ).not.toThrow();
  });

  it('rejects input magnitude frames that are too short', () => {
    const d = new NoteDetector(DEFAULT_OPTS);
    expect(() => d.analyze(new Float32Array(5))).toThrow();
  });

  it('exposes harmonic bin ranges in harmonic mode only', () => {
    const legacy = new NoteDetector(DEFAULT_OPTS);
    expect(legacy.harmonicBinsForMidi(57)).toBeNull();

    const d = new NoteDetector(HARMONIC_OPTS);
    const ranges = d.harmonicBinsForMidi(57);
    expect(ranges).not.toBeNull();
    expect(ranges!.length).toBe(9);
    // Harmonic 1 is the fundamental band — same as the legacy band.
    expect(ranges![0]).toEqual(d.binRangeForMidi(57));
  });

  it('reset() clears harmonic salience state', () => {
    const d = new NoteDetector(HARMONIC_OPTS);
    expect(d.analyze(harmonicStack(220, 7)).has(57)).toBe(true);
    d.reset();
    expect(d.analyze(new Float32Array(NUM_BINS)).size).toBe(0);
  });

  it('reports per-note confidence for the emitted notes', () => {
    const d = new NoteDetector(HARMONIC_OPTS);
    const active = d.analyze(harmonicStack(220, 7));
    const conf = d.lastConfidences();
    // Confidence keys match the emitted set; each is ≥ 1 (at/above threshold).
    expect(new Set(conf.keys())).toEqual(active);
    for (const c of conf.values()) expect(c).toBeGreaterThanOrEqual(1);
  });
});
