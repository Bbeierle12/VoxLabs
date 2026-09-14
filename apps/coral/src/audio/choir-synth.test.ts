import { describe, it, expect } from 'vitest';
import { synthesizeChoirChord, type ChoirChordSpec } from './choir-synth';
import { StreamingStft } from './streaming-stft';

const SR = 44100;
const FFT = 2048;
const HOP = 512;

/** MIDI note → frequency (A4 = 440 Hz). */
function midiToFreq(midi: number): number {
  return 440 * Math.pow(2, (midi - 69) / 12);
}

/** Steady-state magnitude frame: the last frame the real STFT emits. */
function steadyFrame(samples: Float32Array): Float32Array {
  const stft = new StreamingStft(FFT, HOP);
  const frames = stft.pushSamples(samples);
  expect(frames.length).toBeGreaterThan(0);
  return frames[frames.length - 1] as Float32Array;
}

/** Peak magnitude within ±`pad` bins of a frequency's center bin. */
function peakNear(frame: Float32Array, freq: number, pad = 2): number {
  const binFreqHz = SR / FFT;
  const center = Math.round(freq / binFreqHz);
  let peak = 0;
  for (let b = Math.max(1, center - pad); b <= Math.min(frame.length - 1, center + pad); b++) {
    const m = frame[b] as number;
    if (m > peak) peak = m;
  }
  return peak;
}

function median(frame: Float32Array): number {
  // Skip DC (bin 0); it carries no musical information.
  const vals = Array.from(frame.subarray(1)).sort((a, b) => a - b);
  return vals[vals.length >> 1] as number;
}

// C-major triad voiced SATB: B=C3, T=G3, A=E4, S=C5.
const SATB_CHORD: ChoirChordSpec = {
  sampleRate: SR,
  durationSec: 0.5,
  seed: 1,
  voices: [
    { section: 'B', midi: 48 },
    { section: 'T', midi: 55 },
    { section: 'A', midi: 64 },
    { section: 'S', midi: 72 },
  ],
};

describe('synthesizeChoirChord', () => {
  it('places clear spectral energy at every chord fundamental', () => {
    const { samples, groundTruthMidi } = synthesizeChoirChord(SATB_CHORD);
    const frame = steadyFrame(samples);
    const med = median(frame);
    expect(med).toBeGreaterThan(0);

    for (const midi of groundTruthMidi) {
      const peak = peakNear(frame, midiToFreq(midi));
      // Fundamentals stand well clear of the inter-harmonic noise floor.
      expect(peak).toBeGreaterThan(med * 4);
    }
  });

  it('reports ground truth as distinct ascending MIDI + a section map', () => {
    const { groundTruthMidi, sections } = synthesizeChoirChord(SATB_CHORD);
    expect(groundTruthMidi).toEqual([48, 55, 64, 72]);
    expect(sections).toEqual([
      { section: 'B', midi: 48 },
      { section: 'T', midi: 55 },
      { section: 'A', midi: 64 },
      { section: 'S', midi: 72 },
    ]);
  });

  it('collapses a unison/doubling to one ground-truth note but keeps both sections', () => {
    const { groundTruthMidi, sections } = synthesizeChoirChord({
      ...SATB_CHORD,
      voices: [
        { section: 'B', midi: 48 },
        { section: 'T', midi: 60 },
        { section: 'A', midi: 60 }, // Alto doubles Tenor at the unison
        { section: 'S', midi: 72 },
      ],
    });
    expect(groundTruthMidi).toEqual([48, 60, 72]);
    expect(sections.length).toBe(4);
  });

  it('is deterministic for a fixed seed and varies with the seed', () => {
    const a = synthesizeChoirChord(SATB_CHORD).samples;
    const b = synthesizeChoirChord(SATB_CHORD).samples;
    expect(a.length).toBe(b.length);
    let identical = true;
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== b[i]) {
        identical = false;
        break;
      }
    }
    expect(identical).toBe(true);

    const c = synthesizeChoirChord({ ...SATB_CHORD, seed: 2 }).samples;
    let differs = false;
    for (let i = 0; i < a.length; i++) {
      if (a[i] !== c[i]) {
        differs = true;
        break;
      }
    }
    expect(differs).toBe(true);
  });

  it('produces finite samples within range (normalized, no clipping)', () => {
    const { samples } = synthesizeChoirChord(SATB_CHORD);
    let peak = 0;
    for (let i = 0; i < samples.length; i++) {
      const v = samples[i] as number;
      expect(Number.isFinite(v)).toBe(true);
      const a = Math.abs(v);
      if (a > peak) peak = a;
    }
    expect(peak).toBeGreaterThan(0.5);
    expect(peak).toBeLessThanOrEqual(0.9 + 1e-6);
  });

  it('rejects an empty voice list and bad timing', () => {
    expect(() => synthesizeChoirChord({ ...SATB_CHORD, voices: [] })).toThrow();
    expect(() => synthesizeChoirChord({ ...SATB_CHORD, durationSec: 0 })).toThrow();
    expect(() => synthesizeChoirChord({ ...SATB_CHORD, sampleRate: -1 })).toThrow();
  });
});
