import { describe, it, expect } from 'vitest';
import { computeSpectrogram } from './spectrogram';
import { StreamingStft } from './streaming-stft';
import { FFT_CONFIG } from '../config/fft';

/**
 * Batch ↔ streaming calibration parity.
 *
 * The two STFT paths (batch `computeSpectrogram`, live `StreamingStft`)
 * promise identical calibration: same window, same scaling, same framing
 * convention (frame f covers samples [f*hop, f*hop + fftSize)). This test
 * makes that promise enforceable — every magnitude must be BIT-identical,
 * not merely close. Both paths do the same float32 arithmetic in the same
 * order, so any divergence at all means one path's math changed.
 */

const SAMPLE_RATE = 44100;

/** Deterministic pseudo-noise (LCG) — broadband content with no RNG seed drift. */
function makeTestSignal(durationSec: number): Float32Array {
  const n = Math.floor(SAMPLE_RATE * durationSec);
  const buf = new Float32Array(n);
  let state = 0x12345678;
  for (let i = 0; i < n; i++) {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
    const noise = (state / 0xffffffff - 0.5) * 0.1;
    // Two non-bin-aligned tones + noise: exercises leakage across all bins.
    buf[i] =
      0.5 * Math.sin((2 * Math.PI * 440.7 * i) / SAMPLE_RATE) +
      0.3 * Math.sin((2 * Math.PI * 1337.2 * i) / SAMPLE_RATE) +
      noise;
  }
  return buf;
}

describe('batch ↔ streaming STFT parity', () => {
  it('produces bit-identical magnitudes for the same signal at equal framing', () => {
    const { fftSize, hopSize } = FFT_CONFIG;
    const samples = makeTestSignal(1.0);

    const batch = computeSpectrogram(samples, SAMPLE_RATE);

    const stft = new StreamingStft(fftSize, hopSize);
    const streamed = stft.pushSamples(samples);

    expect(streamed.length).toBe(batch.numFrames);

    for (let f = 0; f < batch.numFrames; f++) {
      const frame = streamed[f]!;
      const offset = f * batch.numBins;
      for (let b = 0; b < batch.numBins; b++) {
        // toBe (Object.is) — bit-identical, no tolerance.
        if (frame[b] !== batch.magnitudes[offset + b]) {
          // Fail with context rather than 1M anonymous assertions.
          expect.fail(
            `frame ${f} bin ${b}: streaming=${frame[b]} batch=${batch.magnitudes[offset + b]}`
          );
        }
      }
    }
  });

  it('parity holds when the streaming path receives worklet-sized chunks', () => {
    const { fftSize, hopSize } = FFT_CONFIG;
    const samples = makeTestSignal(0.5);

    const batch = computeSpectrogram(samples, SAMPLE_RATE);

    const stft = new StreamingStft(fftSize, hopSize);
    const streamed: Float32Array[] = [];
    const CHUNK = 128; // AudioWorklet block size
    for (let off = 0; off < samples.length; off += CHUNK) {
      streamed.push(...stft.pushSamples(samples.subarray(off, Math.min(off + CHUNK, samples.length))));
    }

    expect(streamed.length).toBe(batch.numFrames);
    for (let f = 0; f < batch.numFrames; f++) {
      const frame = streamed[f]!;
      const offset = f * batch.numBins;
      for (let b = 0; b < batch.numBins; b++) {
        if (frame[b] !== batch.magnitudes[offset + b]) {
          expect.fail(
            `frame ${f} bin ${b}: streaming=${frame[b]} batch=${batch.magnitudes[offset + b]}`
          );
        }
      }
    }
  });
});
