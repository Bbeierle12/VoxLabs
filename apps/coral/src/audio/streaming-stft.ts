import FFT from 'fft.js';
import { createHannWindow } from './window';

/**
 * Streaming STFT. Stateful sibling to the batch `computeSpectrogram`.
 *
 * The batch version is monolithic: take a complete sample array, emit a
 * complete spectrogram. That contract doesn't fit live capture, where
 * samples arrive 128 at a time and we need a new magnitude frame every
 * `hopSize` samples. This class keeps a sliding buffer of the most
 * recent `fftSize` samples and emits a frame whenever enough new
 * samples have accumulated since the last emission.
 *
 * Magnitude scaling matches the batch path: divide by sum(window)/2 so
 * a 1.0-amplitude sine at a bin center peaks at 0 dB. Keeping the two
 * paths calibrated identically means dB readings are comparable across
 * live vs file analysis.
 */
export class StreamingStft {
  readonly numBins: number;

  private readonly buffer: Float32Array;
  private readonly window: Float32Array;
  private readonly windowed: Float32Array;
  private readonly fft: FFT;
  private readonly spectrum: number[];
  private readonly invScale: number;

  /** How many samples currently live in [0, bufferFill). */
  private bufferFill = 0;
  /** Sample countdown until the next frame should emit. */
  private samplesUntilNextFrame: number;

  constructor(
    readonly fftSize: number,
    readonly hopSize: number
  ) {
    if (fftSize <= 0 || (fftSize & (fftSize - 1)) !== 0) {
      throw new Error(`StreamingStft: fftSize must be a power of 2, got ${fftSize}`);
    }
    if (hopSize <= 0 || hopSize > fftSize) {
      throw new Error(
        `StreamingStft: hopSize must be in (0, fftSize], got ${hopSize} (fftSize=${fftSize})`
      );
    }

    // Buffer is 2x fftSize so we can absorb a full fftSize-worth of new
    // samples before sliding — eliminates per-sample copyWithin in the
    // common case.
    this.buffer = new Float32Array(fftSize * 2);
    this.window = createHannWindow(fftSize);
    this.windowed = new Float32Array(fftSize);
    this.fft = new FFT(fftSize);
    this.spectrum = this.fft.createComplexArray();
    this.numBins = fftSize / 2;

    // First emission needs fftSize samples; subsequent emissions need hopSize.
    this.samplesUntilNextFrame = fftSize;

    let windowSum = 0;
    for (let i = 0; i < fftSize; i++) windowSum += this.window[i] as number;
    this.invScale = 2 / windowSum;
  }

  /**
   * Push a chunk of audio samples. Returns zero or more new magnitude
   * frames, each a fresh `Float32Array(numBins)`. Callers may stash the
   * returned frames — they are not reused.
   */
  pushSamples(chunk: Float32Array): Float32Array[] {
    const frames: Float32Array[] = [];
    let chunkOffset = 0;

    while (chunkOffset < chunk.length) {
      // Slide if we'd overflow on the next append. After slide the most
      // recent `fftSize` samples sit at [0, fftSize); we can then absorb
      // another fftSize before sliding again.
      if (this.bufferFill >= this.buffer.length) {
        this.buffer.copyWithin(0, this.bufferFill - this.fftSize, this.bufferFill);
        this.bufferFill = this.fftSize;
      }

      const space = this.buffer.length - this.bufferFill;
      const remaining = chunk.length - chunkOffset;
      const need = this.samplesUntilNextFrame;
      const consume = Math.min(space, remaining, need);

      this.buffer.set(chunk.subarray(chunkOffset, chunkOffset + consume), this.bufferFill);
      this.bufferFill += consume;
      chunkOffset += consume;
      this.samplesUntilNextFrame -= consume;

      if (this.samplesUntilNextFrame === 0 && this.bufferFill >= this.fftSize) {
        frames.push(this.emitFrame());
        this.samplesUntilNextFrame = this.hopSize;
      }
    }

    return frames;
  }

  /** Reset internal state — call before reusing the instance for a new session. */
  reset(): void {
    this.bufferFill = 0;
    this.samplesUntilNextFrame = this.fftSize;
  }

  private emitFrame(): Float32Array {
    const start = this.bufferFill - this.fftSize;
    for (let i = 0; i < this.fftSize; i++) {
      this.windowed[i] = (this.buffer[start + i] as number) * (this.window[i] as number);
    }
    this.fft.realTransform(this.spectrum, this.windowed);
    this.fft.completeSpectrum(this.spectrum);

    const magnitudes = new Float32Array(this.numBins);
    for (let b = 0; b < this.numBins; b++) {
      const re = this.spectrum[2 * b] as number;
      const im = this.spectrum[2 * b + 1] as number;
      magnitudes[b] = Math.sqrt(re * re + im * im) * this.invScale;
    }
    return magnitudes;
  }
}
