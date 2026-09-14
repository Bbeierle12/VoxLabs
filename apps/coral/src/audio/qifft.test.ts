import { describe, it, expect } from 'vitest';
import { parabolicPeak, estimateF0 } from './qifft';
import { StreamingStft } from './streaming-stft';

const SR = 44100;
const FFT = 2048;

function sine(freq: number, n: number): Float32Array {
  const out = new Float32Array(n);
  const w = (2 * Math.PI * freq) / SR;
  for (let i = 0; i < n; i++) out[i] = Math.sin(w * i);
  return out;
}
function lastMags(samples: Float32Array): Float32Array {
  const stft = new StreamingStft(FFT, 512);
  let last: Float32Array = new Float32Array(0);
  for (const f of stft.pushSamples(samples)) last = f;
  return last;
}
const cents = (f: number, ref: number) => 1200 * Math.log2(f / ref);

describe('parabolicPeak', () => {
  it('is zero for a symmetric peak', () => {
    expect(parabolicPeak(0, 1, 0)).toBeCloseTo(0, 10);
  });
  it('leans toward the taller neighbour', () => {
    expect(parabolicPeak(0.5, 1, 0.9)).toBeGreaterThan(0);
    expect(parabolicPeak(0.9, 1, 0.5)).toBeLessThan(0);
  });
  it('clamps to ±0.5', () => {
    expect(parabolicPeak(1, 1, 0)).toBeGreaterThanOrEqual(-0.5);
    expect(parabolicPeak(0, 1, 1)).toBeLessThanOrEqual(0.5);
  });
});

describe('estimateF0', () => {
  it('recovers A4 (440 Hz) within a few cents', () => {
    const e = estimateF0(lastMags(sine(440, FFT * 4)), SR, FFT);
    expect(e).not.toBeNull();
    expect(Math.abs(cents(e!.frequencyHz, 440))).toBeLessThan(10);
  });

  it('recovers pitches across the vocal range within ±15 cents', () => {
    for (const f of [196, 261.63, 329.63, 440, 587.33, 880]) {
      const e = estimateF0(lastMags(sine(f, FFT * 4)), SR, FFT);
      expect(e, `${f} Hz`).not.toBeNull();
      expect(Math.abs(cents(e!.frequencyHz, f)), `${f} Hz`).toBeLessThan(15);
    }
  });

  it('returns null for silence', () => {
    expect(estimateF0(new Float32Array(FFT / 2), SR, FFT)).toBeNull();
  });

  it('does not crash on a degenerate (too-short) spectrum', () => {
    expect(estimateF0(new Float32Array(2), SR, FFT)).toBeNull();
  });

  it('avoids octave-flipping when H2 >= H1 (vocal open-vowel scenario)', () => {
    // Generate a spectrum where the fundamental (H1) is at 200 Hz, but H2 at 400 Hz is stronger.
    // At SR=44100, FFT=2048, bin frequency is ~21.53 Hz.
    // 200 Hz lands around bin 9. 400 Hz lands around bin 19.
    const mags = new Float32Array(FFT / 2);
    mags.fill(0.01); // noise floor
    
    // Set H1 at bin 9 (magnitude 0.8)
    mags[8] = 0.4;
    mags[9] = 0.8;
    mags[10] = 0.4;
    
    // Set H2 at bin 19 (magnitude 1.0, stronger than H1)
    mags[18] = 0.5;
    mags[19] = 1.0;
    mags[20] = 0.5;

    const e = estimateF0(mags, SR, FFT, { voicedRatio: 2 });
    expect(e).not.toBeNull();
    // It should estimate the fundamental around 200 Hz (bin 9), not the octave 400 Hz (bin 19)
    expect(e!.frequencyHz).toBeCloseTo(9 * (SR / FFT), 0);
  });
});
