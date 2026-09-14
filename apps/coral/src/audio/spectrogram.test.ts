import { describe, it, expect } from 'vitest';
import { computeSpectrogram } from './spectrogram';
import { FFT_CONFIG } from '../config/fft';

const SAMPLE_RATE = 44100;

/** Generate a pure sine wave at `freq` Hz for `durationSec` seconds. */
function makeSine(freq: number, durationSec: number): Float32Array {
  const n = Math.floor(SAMPLE_RATE * durationSec);
  const buf = new Float32Array(n);
  const omega = 2 * Math.PI * freq;
  for (let i = 0; i < n; i++) {
    buf[i] = Math.sin((omega * i) / SAMPLE_RATE);
  }
  return buf;
}

/** Find the index of the max value in a slice of a Float32Array. */
function argmax(arr: Float32Array, start: number, length: number): number {
  let maxIdx = start;
  let maxVal = arr[start] ?? -Infinity;
  for (let i = start + 1; i < start + length; i++) {
    const v = arr[i];
    if (v !== undefined && v > maxVal) {
      maxVal = v;
      maxIdx = i;
    }
  }
  return maxIdx - start;
}

describe('computeSpectrogram', () => {
  it('places the spectral peak at the expected bin for a pure sine wave', () => {
    const freq = 1000; // 1 kHz, well inside any reasonable range
    const samples = makeSine(freq, 0.5);
    const data = computeSpectrogram(samples, SAMPLE_RATE);

    const binWidthHz = SAMPLE_RATE / FFT_CONFIG.fftSize;
    const expectedBin = Math.round(freq / binWidthHz);

    // Check the middle frame (avoid window edge artifacts in frame 0).
    const midFrameIdx = Math.floor(data.numFrames / 2);
    const peakBin = argmax(
      data.magnitudes,
      midFrameIdx * data.numBins,
      data.numBins
    );

    // Allow ±1 bin of slack for windowing leakage.
    expect(Math.abs(peakBin - expectedBin)).toBeLessThanOrEqual(1);
  });

  it('produces the expected number of frames for known input length', () => {
    const samples = new Float32Array(FFT_CONFIG.fftSize * 4);
    const data = computeSpectrogram(samples, SAMPLE_RATE);
    const expected =
      Math.floor((samples.length - FFT_CONFIG.fftSize) / FFT_CONFIG.hopSize) + 1;
    expect(data.numFrames).toBe(expected);
  });

  it('throws on empty input', () => {
    expect(() => computeSpectrogram(new Float32Array(0), SAMPLE_RATE)).toThrow();
  });

  it('throws on invalid sample rate', () => {
    const samples = new Float32Array(FFT_CONFIG.fftSize);
    expect(() => computeSpectrogram(samples, 0)).toThrow();
    expect(() => computeSpectrogram(samples, -1)).toThrow();
    expect(() => computeSpectrogram(samples, NaN)).toThrow();
  });

  it('throws when samples are shorter than one FFT window', () => {
    const samples = new Float32Array(FFT_CONFIG.fftSize - 1);
    expect(() => computeSpectrogram(samples, SAMPLE_RATE)).toThrow(/too short/i);
  });
});
