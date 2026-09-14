import { describe, it, expect } from 'vitest';
import { StreamingStft } from './streaming-stft';

const SAMPLE_RATE = 44100;
const FFT_SIZE = 2048;
const HOP_SIZE = 512;

function makeSine(freq: number, durationSec: number): Float32Array {
  const n = Math.floor(SAMPLE_RATE * durationSec);
  const buf = new Float32Array(n);
  const omega = 2 * Math.PI * freq;
  for (let i = 0; i < n; i++) buf[i] = Math.sin((omega * i) / SAMPLE_RATE);
  return buf;
}

function argmax(arr: Float32Array): number {
  let maxIdx = 0;
  let maxVal = arr[0] ?? -Infinity;
  for (let i = 1; i < arr.length; i++) {
    const v = arr[i];
    if (v !== undefined && v > maxVal) {
      maxVal = v;
      maxIdx = i;
    }
  }
  return maxIdx;
}

describe('StreamingStft', () => {
  it('emits exactly one frame after fftSize samples, then one per hopSize', () => {
    const stft = new StreamingStft(FFT_SIZE, HOP_SIZE);

    // Push the first fftSize samples in one shot.
    const first = stft.pushSamples(new Float32Array(FFT_SIZE));
    expect(first).toHaveLength(1);

    // Push hopSize more — one additional frame.
    const second = stft.pushSamples(new Float32Array(HOP_SIZE));
    expect(second).toHaveLength(1);

    // Push 3 × hopSize — three additional frames.
    const batch = stft.pushSamples(new Float32Array(HOP_SIZE * 3));
    expect(batch).toHaveLength(3);
  });

  it('emits the same total number of frames regardless of chunk size', () => {
    // 4 seconds of zeros — enough to exercise plenty of sliding.
    const total = SAMPLE_RATE * 4;
    const samples = new Float32Array(total);

    const stftAll = new StreamingStft(FFT_SIZE, HOP_SIZE);
    const allAtOnce = stftAll.pushSamples(samples).length;

    const stftSmall = new StreamingStft(FFT_SIZE, HOP_SIZE);
    let smallChunks = 0;
    const CHUNK = 128; // matches the AudioWorklet block size
    for (let off = 0; off < total; off += CHUNK) {
      const end = Math.min(off + CHUNK, total);
      smallChunks += stftSmall.pushSamples(samples.subarray(off, end)).length;
    }

    expect(smallChunks).toBe(allAtOnce);

    // Sanity check the formula: floor((N - fftSize) / hopSize) + 1.
    const expected = Math.floor((total - FFT_SIZE) / HOP_SIZE) + 1;
    expect(allAtOnce).toBe(expected);
  });

  it('places the spectral peak at the expected bin for a pure sine wave', () => {
    const freq = 1000;
    const samples = makeSine(freq, 0.5);
    const stft = new StreamingStft(FFT_SIZE, HOP_SIZE);
    const frames = stft.pushSamples(samples);

    expect(frames.length).toBeGreaterThan(5);

    // Use a middle frame to avoid windowing edge artifacts.
    const mid = frames[Math.floor(frames.length / 2)];
    expect(mid).toBeDefined();
    const peakBin = argmax(mid!);

    const binWidthHz = SAMPLE_RATE / FFT_SIZE;
    const expectedBin = Math.round(freq / binWidthHz);
    expect(Math.abs(peakBin - expectedBin)).toBeLessThanOrEqual(1);
  });

  it('produces a calibrated 0-dB peak for a bin-aligned unit-amplitude sine', () => {
    // Pick a frequency that lands exactly on a bin center, so we're
    // testing the scaling math without scalloping loss as a confound.
    // bin * Fs / fftSize = 100 * 44100 / 2048 ≈ 2153.3 Hz.
    const bin = 100;
    const freq = (bin * SAMPLE_RATE) / FFT_SIZE;
    const samples = makeSine(freq, 0.5);
    const stft = new StreamingStft(FFT_SIZE, HOP_SIZE);
    const frames = stft.pushSamples(samples);
    const mid = frames[Math.floor(frames.length / 2)]!;
    const peakBin = argmax(mid);
    expect(peakBin).toBe(bin);
    const peakMag = mid[peakBin] as number;
    const peakDb = 20 * Math.log10(peakMag);
    // Window-gain-corrected: a bin-aligned unit sine peaks within
    // ±0.3 dB of 0. Any larger drift means the scaling math is wrong.
    expect(peakDb).toBeGreaterThan(-0.3);
    expect(peakDb).toBeLessThan(0.3);
  });

  it('stays calibrated at 48 kHz (the common macOS/Windows device rate)', () => {
    // Same 0-dB canary as above, but at 48 kHz — everything else in the
    // suite pins 44.1 kHz, and sample rate enters the math only through
    // the test signal, so this guards against any future code path that
    // bakes 44100 into scaling or bin mapping.
    const sampleRate = 48000;
    const bin = 100;
    const freq = (bin * sampleRate) / FFT_SIZE;
    const n = Math.floor(sampleRate * 0.5);
    const samples = new Float32Array(n);
    const omega = 2 * Math.PI * freq;
    for (let i = 0; i < n; i++) samples[i] = Math.sin((omega * i) / sampleRate);

    const stft = new StreamingStft(FFT_SIZE, HOP_SIZE);
    const frames = stft.pushSamples(samples);
    const mid = frames[Math.floor(frames.length / 2)]!;
    const peakBin = argmax(mid);
    expect(peakBin).toBe(bin);
    const peakDb = 20 * Math.log10(mid[peakBin] as number);
    expect(peakDb).toBeGreaterThan(-0.3);
    expect(peakDb).toBeLessThan(0.3);
  });

  it('rejects non-power-of-2 fftSize and invalid hopSize', () => {
    expect(() => new StreamingStft(1000, 256)).toThrow(/power of 2/i);
    expect(() => new StreamingStft(2048, 0)).toThrow();
    expect(() => new StreamingStft(2048, 4096)).toThrow();
  });
});
