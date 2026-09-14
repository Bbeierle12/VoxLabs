import { describe, it, expect } from 'vitest';
import { SpectrogramDsp, quantize } from './dsp-core';
import { synthesizeChoirChord } from './choir-synth';
import {
  SPEC_SRC_FLOOR_DB,
  SPEC_SRC_CEIL_DB,
} from '../config/spectrogram-encoding';

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

function argmax(arr: Uint8Array): number {
  let maxIdx = 0;
  let maxVal = -Infinity;
  for (let i = 0; i < arr.length; i++) {
    const v = arr[i] as number;
    if (v > maxVal) {
      maxVal = v;
      maxIdx = i;
    }
  }
  return maxIdx;
}

const OPTS = {
  fftSize: FFT_SIZE,
  hopSize: HOP_SIZE,
  sampleRate: SAMPLE_RATE,
  minFreqHz: 50,
  maxFreqHz: 8000,
};

/** Byte the wire encoding assigns to a given dB value. */
const byteForDb = (db: number): number =>
  Math.round(((db - SPEC_SRC_FLOOR_DB) / (SPEC_SRC_CEIL_DB - SPEC_SRC_FLOOR_DB)) * 255);

describe('quantize', () => {
  it('maps a 0 dBFS (unit) magnitude to the 0 dB byte (below 255 — headroom)', () => {
    // The wire ceiling sits ABOVE 0 dBFS so coherent multi-source bins
    // (e.g. in-phase unisons at +6 dB) don't clip. 0 dBFS must land at
    // the constant-derived byte, not the top of the range.
    const bytes = quantize(Float32Array.of(1.0));
    expect(bytes[0]).toBe(byteForDb(0));
    expect(bytes[0]).toBeLessThan(255);
  });

  it('maps silence to byte 0 (well below the source floor)', () => {
    const bytes = quantize(Float32Array.of(0));
    expect(bytes[0]).toBe(0);
  });

  it('places the byte at the floor for a magnitude at SPEC_SRC_FLOOR_DB', () => {
    const mag = Math.pow(10, SPEC_SRC_FLOOR_DB / 20); // dB → linear
    const bytes = quantize(Float32Array.of(mag));
    expect(bytes[0]).toBeLessThanOrEqual(1);
  });
});

describe('SpectrogramDsp', () => {
  it('emits the same frame count as the underlying STFT formula', () => {
    const dsp = new SpectrogramDsp(OPTS);
    const total = SAMPLE_RATE * 2;
    let count = 0;
    const CHUNK = 128;
    const samples = new Float32Array(total);
    for (let off = 0; off < total; off += CHUNK) {
      count += dsp.pushSamples(samples.subarray(off, off + CHUNK)).frames.length;
    }
    const expected = Math.floor((total - FFT_SIZE) / HOP_SIZE) + 1;
    expect(count).toBe(expected);
  });

  it('quantizes a unit sine so its peak bin hits the 0 dB byte', () => {
    // Bin-aligned frequency so the peak lands cleanly on one bin. A
    // calibrated unit sine peaks at 0 dB, which maps to the constant-
    // derived byte — NOT 255, since the wire range carries headroom
    // above full scale for coherent summation.
    const bin = 100;
    const freq = (bin * SAMPLE_RATE) / FFT_SIZE;
    const dsp = new SpectrogramDsp(OPTS);
    const frames = dsp.pushSamples(makeSine(freq, 0.5)).frames;
    expect(frames.length).toBeGreaterThan(5);
    const mid = frames[Math.floor(frames.length / 2)]!;
    expect(mid.bytes).toHaveLength(FFT_SIZE / 2);
    expect(argmax(mid.bytes)).toBe(bin);
    // ±1 byte: the sine peaks within ±0.3 dB of 0 (see streaming-stft
    // tests); one quantization step is ≈0.6 dB.
    expect(Math.abs((mid.bytes[bin] as number) - byteForDb(0))).toBeLessThanOrEqual(1);
    expect(Array.isArray(mid.activeMidi)).toBe(true);
  });

  it('reports a level reading once a ~50 ms window has filled', () => {
    const dsp = new SpectrogramDsp(OPTS);
    // 32 ms of audio — under one level window — yields no reading yet.
    const short = dsp.pushSamples(makeSine(440, 0.032));
    expect(short.level).toBeNull();
    // Topping past 50 ms total emits one.
    const more = dsp.pushSamples(makeSine(440, 0.03));
    expect(more.level).not.toBeNull();
    expect(more.level!.peakDb).toBeGreaterThan(-10); // unit sine ≈ 0 dBFS peak
  });

  it('reset() clears streaming state so frame emission restarts', () => {
    const dsp = new SpectrogramDsp(OPTS);
    dsp.pushSamples(new Float32Array(FFT_SIZE)); // primes one frame
    dsp.reset();
    // After reset the first emission again needs a full fftSize.
    expect(dsp.pushSamples(new Float32Array(HOP_SIZE)).frames).toHaveLength(0);
    expect(dsp.pushSamples(new Float32Array(FFT_SIZE - HOP_SIZE)).frames).toHaveLength(1);
  });
});

describe('setFraming (reframe semantics)', () => {
  it('drops partial display state; the next frame needs a full new fftSize', () => {
    const dsp = new SpectrogramDsp(OPTS);
    expect(dsp.pushSamples(new Float32Array(FFT_SIZE)).frames).toHaveLength(1);
    // Leave a partial hop in flight, then reframe.
    dsp.pushSamples(new Float32Array(HOP_SIZE - 1));
    dsp.setFraming(1024, 256);
    expect(dsp.numBins).toBe(512);
    // The old partial-frame samples are dropped: nothing emits until a
    // full NEW fftSize accumulates (the documented contract in
    // MicPipeline.setFraming), then one frame per new hop.
    expect(dsp.pushSamples(new Float32Array(1023)).frames).toHaveLength(0);
    expect(dsp.pushSamples(new Float32Array(1)).frames).toHaveLength(1);
    expect(dsp.pushSamples(new Float32Array(256)).frames).toHaveLength(1);
  });

  it('note detection survives a display reframe', () => {
    // Sustained E4 through the real pipeline (same recipe as
    // choir-detection.test.ts). Detection runs on its own fixed-framing
    // STFT, so swapping the display framing must not disturb it.
    const { samples } = synthesizeChoirChord({
      sampleRate: SAMPLE_RATE,
      durationSec: 0.6,
      seed: 7,
      voices: [{ section: 'A', midi: 64 }],
    });
    const dsp = new SpectrogramDsp(OPTS);
    const split = Math.floor(samples.length * 0.7);
    let last: number[] = [];
    const r1 = dsp.pushSamples(samples.subarray(0, split));
    last = r1.frames[r1.frames.length - 1]!.activeMidi;
    expect(last).toContain(64); // locked before the reframe

    dsp.setFraming(1024, 256);
    const r2 = dsp.pushSamples(samples.subarray(split));
    expect(r2.frames.length).toBeGreaterThan(0);
    const after = r2.frames[r2.frames.length - 1]!.activeMidi;
    expect(after).toContain(64); // still locked after it
  });
});
