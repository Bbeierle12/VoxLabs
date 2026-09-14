import FFT from 'fft.js';
import { FFT_CONFIG } from '../config/fft';
import { createHannWindow, applyWindow } from './window';

/**
 * Result of an STFT magnitude computation.
 *
 * `magnitudes` is a flat row-major buffer: frame `f`, bin `b` lives at
 * index `f * numBins + b`. Flat layout keeps cache locality good for
 * sequential render and avoids array-of-arrays allocation overhead.
 */
export interface SpectrogramData {
  magnitudes: Float32Array;
  numFrames: number;
  numBins: number;
  hopSize: number;
  sampleRate: number;
}

/**
 * Compute the magnitude spectrogram of `samples` at `sampleRate`.
 *
 * Throws on empty input, invalid sample rate, or samples shorter than
 * one FFT window. Pre-allocates all buffers; no allocations inside the
 * frame loop.
 */
export function computeSpectrogram(
  samples: Float32Array,
  sampleRate: number
): SpectrogramData {
  if (samples.length === 0) {
    throw new Error('computeSpectrogram: samples is empty');
  }
  if (!Number.isFinite(sampleRate) || sampleRate <= 0) {
    throw new Error(
      `computeSpectrogram: sampleRate must be a positive finite number, got ${sampleRate}`
    );
  }

  const { fftSize, hopSize } = FFT_CONFIG;
  if (samples.length < fftSize) {
    throw new Error(
      `computeSpectrogram: samples too short (${samples.length} < fftSize ${fftSize})`
    );
  }

  const numFrames = Math.floor((samples.length - fftSize) / hopSize) + 1;
  const numBins = fftSize / 2;

  // All allocations live OUTSIDE the loop (Guardrail 7).
  const fft = new FFT(fftSize);
  const window = createHannWindow(fftSize);
  const windowed = new Float32Array(fftSize);
  const spectrum = fft.createComplexArray();
  const magnitudes = new Float32Array(numFrames * numBins);

  // Single-sided amplitude correction: a pure sine of amplitude A at a
  // bin center produces FFT magnitude A * sum(window) / 2. Dividing by
  // sum(window) / 2 restores amplitude A, so a 1.0-amplitude sine peaks
  // at 0 dB and dB readings are reproducible across runs and window
  // choices. DC and Nyquist would need a separate factor; both are
  // excluded from the log-frequency render range. (Edge case: at
  // fftSize=512 the bin width is ~86 Hz, so a 20 Hz display floor maps
  // its lowest columns onto bin 0 and shows its spurious +6 dB —
  // cosmetic only, and absent at the default 50 Hz floor.)
  let windowSum = 0;
  for (let i = 0; i < fftSize; i++) windowSum += window[i] as number;
  const invScale = 2 / windowSum;

  for (let f = 0; f < numFrames; f++) {
    const start = f * hopSize;
    applyWindow(samples, start, window, windowed);
    fft.realTransform(spectrum, windowed);
    fft.completeSpectrum(spectrum);

    const offset = f * numBins;
    for (let b = 0; b < numBins; b++) {
      const re = spectrum[2 * b] as number;
      const im = spectrum[2 * b + 1] as number;
      magnitudes[offset + b] = Math.sqrt(re * re + im * im) * invScale;
    }
  }

  return { magnitudes, numFrames, numBins, hopSize, sampleRate };
}
