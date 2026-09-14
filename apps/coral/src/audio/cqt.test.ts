import { describe, it, expect } from 'vitest';
import { StreamingCqt } from './cqt';
import { NoteDetector } from './note-detector';

const SR = 44100;

function sine(freq: number, n: number): Float32Array {
  const out = new Float32Array(n);
  const w = (2 * Math.PI * freq) / SR;
  for (let i = 0; i < n; i++) out[i] = Math.sin(w * i);
  return out;
}

/** Sum of `k` harmonics of f0 at amplitude 1/h — gives the detector something to score. */
function harmonicTone(f0: number, k: number, n: number): Float32Array {
  const out = new Float32Array(n);
  for (let h = 1; h <= k; h++) {
    const w = (2 * Math.PI * h * f0) / SR;
    const a = 1 / h;
    for (let i = 0; i < n; i++) out[i] = (out[i] as number) + a * Math.sin(w * i);
  }
  return out;
}

function argmax(a: Float32Array): number {
  let am = 0;
  let mx = -Infinity;
  for (let k = 0; k < a.length; k++) {
    if ((a[k] as number) > mx) {
      mx = a[k] as number;
      am = k;
    }
  }
  return am;
}

describe('StreamingCqt', () => {
  const make = () =>
    new StreamingCqt({ sampleRate: SR, minFreqHz: 50, maxFreqHz: 20000, binsPerOctave: 36, hopSize: 4096 });

  it('peaks at the bin nearest a pure tone (A4 = 440)', () => {
    const cqt = make();
    const n = cqt.fftSize + SR; // ≥ one full window
    const frames = cqt.pushSamples(sine(440, n));
    expect(frames.length).toBeGreaterThan(0);
    const last = frames[frames.length - 1] as Float32Array;
    const f = cqt.binFreqs[argmax(last)] as number;
    // Within a quarter-tone of 440 Hz.
    expect(f).toBeGreaterThan(440 / 1.03);
    expect(f).toBeLessThan(440 * 1.03);
  });

  it('resolves two tones a semitone apart into two peaks', () => {
    const cqt = make();
    const n = cqt.fftSize + SR;
    const mix = sine(440, n);
    const b = sine(466.16, n); // A#4
    for (let i = 0; i < n; i++) mix[i] = (mix[i] as number) + (b[i] as number);
    const frames = cqt.pushSamples(mix);
    const last = frames[frames.length - 1] as Float32Array;
    // Energy near 440 and near 466 should both clearly exceed energy a
    // quarter-tone below 440 (a frequency neither tone occupies).
    const near = (hz: number) => {
      let best = 0;
      for (let k = 0; k < cqt.numBins; k++) {
        const ratio = (cqt.binFreqs[k] as number) / hz;
        if (ratio > 1 / 1.02 && ratio < 1.02) best = Math.max(best, last[k] as number);
      }
      return best;
    };
    const valley = near(440 / 1.06); // ~quarter-tone below A4, between nothing
    expect(near(440)).toBeGreaterThan(valley * 3);
    expect(near(466.16)).toBeGreaterThan(valley * 3);
  });

  it('exposes geometrically spaced bins (octave = binsPerOctave apart)', () => {
    const cqt = make();
    const r = (cqt.binFreqs[36] as number) / (cqt.binFreqs[0] as number);
    expect(r).toBeGreaterThan(1.99);
    expect(r).toBeLessThan(2.01);
  });

  it('drives a NoteDetector via the binFreqs (CQT) option to detect a sustained note', () => {
    const cqt = make();
    const det = new NoteDetector({
      numBins: cqt.numBins,
      binFreqs: cqt.binFreqs,
      sampleRate: SR,
      minFreqHz: 50,
      maxFreqHz: 8000,
      harmonic: true,
      salienceThreshold: 2.5,
      whiteningWindowBins: 30,
    });
    const n = cqt.fftSize + SR;
    let active = new Set<number>();
    for (const f of cqt.pushSamples(harmonicTone(220, 6, n))) active = det.analyze(f);
    expect(active.has(57)).toBe(true); // A3 = 220 Hz
  });
});
