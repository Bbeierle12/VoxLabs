import FFT from 'fft.js';
import { createHannWindow } from './window';

/**
 * Streaming Constant-Q Transform (kernel method, Schörkhuber/Klapuri-style).
 *
 * STATUS: evaluated as a detection front end (workstream T3) and NOT adopted —
 * on the synthetic benchmark it scored F1 ≈ 0.64 vs the 8192-pt STFT's ≈ 0.92.
 * The STFT's fine linear resolution + iterative cancellation already wins, and
 * recall is better recovered by longer temporal integration (T4) than by the
 * representation. Retained as a correct, tested CQT for possible future use
 * (it drives NoteDetector via the `binFreqs` option).
 *
 * Unlike the STFT's uniform linear bins, the CQT's bins are geometrically
 * spaced (constant cents per bin) and constant-Q, so a semitone has the same
 * resolution at every register and a note's harmonics sit at FIXED bin offsets
 * (an octave = +binsPerOctave bins). That's the property the detector exploits.
 *
 * Implementation: precompute, for each CQT bin, a windowed complex-exponential
 * "temporal atom" of length Q·sr/f_k, FFT it once into a SPARSE spectral
 * kernel, and conjugate it. Per frame: take one big FFT of the latest
 * `fftSize` samples and dot it against each sparse kernel — O(Σ nonzeros), cheap
 * after the shared FFT. The lowest bins need ~1 s of history, so `fftSize` is
 * large (the inherent time–frequency cost of fine bass resolution).
 */
export interface CqtConfig {
  sampleRate: number;
  /** Lowest CQT bin centre frequency, Hz. */
  minFreqHz: number;
  /** Highest CQT bin centre frequency, Hz (reach ~Nyquist to capture harmonics). */
  maxFreqHz: number;
  /** Bins per octave (e.g. 36 = 3 bins/semitone). */
  binsPerOctave: number;
  /** Samples between emitted frames. */
  hopSize: number;
}

/** Relative magnitude below which a spectral-kernel entry is dropped (sparsity). */
const KERNEL_THRESH = 0.0054;

export class StreamingCqt {
  readonly numBins: number;
  readonly binsPerOctave: number;
  /** Centre frequency of each CQT bin, ascending. */
  readonly binFreqs: Float32Array;
  readonly fftSize: number;
  readonly hopSize: number;

  private readonly fft: FFT;
  private readonly spectrum: number[];
  // Flattened sparse spectral kernels (conjugated). Bin k occupies
  // [kStart[k], kStart[k]+kLen[k]) in kBin/kRe/kIm.
  private readonly kStart: Int32Array;
  private readonly kLen: Int32Array;
  private readonly kBin: Int32Array;
  private readonly kRe: Float32Array;
  private readonly kIm: Float32Array;

  private readonly buffer: Float32Array; // 2×fftSize sliding history
  private bufferFill = 0;
  private samplesUntilNextFrame: number;

  constructor(cfg: CqtConfig) {
    const { sampleRate, minFreqHz, maxFreqHz, binsPerOctave, hopSize } = cfg;
    if (!(minFreqHz > 0) || !(maxFreqHz > minFreqHz)) {
      throw new Error(`StreamingCqt: require 0 < minFreqHz < maxFreqHz`);
    }
    this.binsPerOctave = binsPerOctave;
    const Q = 1 / (Math.pow(2, 1 / binsPerOctave) - 1);
    this.numBins = Math.floor(binsPerOctave * Math.log2(maxFreqHz / minFreqHz)) + 1;
    this.binFreqs = new Float32Array(this.numBins);
    for (let k = 0; k < this.numBins; k++) {
      this.binFreqs[k] = minFreqHz * Math.pow(2, k / binsPerOctave);
    }

    // FFT length: at least the longest atom (the lowest bin's window).
    const maxLen = Math.ceil((Q * sampleRate) / minFreqHz);
    let nfft = 1;
    while (nfft < maxLen) nfft <<= 1;
    this.fftSize = nfft;
    this.hopSize = hopSize;

    this.fft = new FFT(nfft);
    this.spectrum = this.fft.createComplexArray();

    // Build sparse spectral kernels.
    const atom = this.fft.createComplexArray();
    const kfft = this.fft.createComplexArray();
    const idx: number[] = [];
    const re: number[] = [];
    const im: number[] = [];
    this.kStart = new Int32Array(this.numBins);
    this.kLen = new Int32Array(this.numBins);

    for (let k = 0; k < this.numBins; k++) {
      const fk = this.binFreqs[k] as number;
      const Lk = Math.max(1, Math.min(nfft, Math.ceil((Q * sampleRate) / fk)));
      const start = (nfft - Lk) >> 1;
      atom.fill(0);
      const win = createHannWindow(Lk);
      const norm = 1 / Lk;
      for (let n = 0; n < Lk; n++) {
        const phase = (2 * Math.PI * fk * n) / sampleRate;
        const w = (win[n] as number) * norm;
        atom[2 * (start + n)] = w * Math.cos(phase);
        atom[2 * (start + n) + 1] = w * Math.sin(phase);
      }
      this.fft.transform(kfft, atom);

      let maxMag = 0;
      for (let j = 0; j < nfft; j++) {
        const r = kfft[2 * j] as number;
        const i = kfft[2 * j + 1] as number;
        const m = Math.sqrt(r * r + i * i);
        if (m > maxMag) maxMag = m;
      }
      const cut = maxMag * KERNEL_THRESH;
      this.kStart[k] = idx.length;
      for (let j = 0; j < nfft; j++) {
        const r = kfft[2 * j] as number;
        const i = kfft[2 * j + 1] as number;
        if (Math.sqrt(r * r + i * i) >= cut) {
          idx.push(j);
          re.push(r);
          im.push(-i); // store the conjugate
        }
      }
      this.kLen[k] = idx.length - (this.kStart[k] as number);
    }
    this.kBin = Int32Array.from(idx);
    this.kRe = Float32Array.from(re);
    this.kIm = Float32Array.from(im);

    this.buffer = new Float32Array(nfft * 2);
    this.samplesUntilNextFrame = nfft;
  }

  /** Push samples; return zero or more CQT magnitude frames (length numBins). */
  pushSamples(chunk: Float32Array): Float32Array[] {
    const frames: Float32Array[] = [];
    let off = 0;
    while (off < chunk.length) {
      if (this.bufferFill >= this.buffer.length) {
        this.buffer.copyWithin(0, this.bufferFill - this.fftSize, this.bufferFill);
        this.bufferFill = this.fftSize;
      }
      const space = this.buffer.length - this.bufferFill;
      const remaining = chunk.length - off;
      const consume = Math.min(space, remaining, this.samplesUntilNextFrame);
      this.buffer.set(chunk.subarray(off, off + consume), this.bufferFill);
      this.bufferFill += consume;
      off += consume;
      this.samplesUntilNextFrame -= consume;
      if (this.samplesUntilNextFrame === 0 && this.bufferFill >= this.fftSize) {
        frames.push(this.computeFrame());
        this.samplesUntilNextFrame = this.hopSize;
      }
    }
    return frames;
  }

  reset(): void {
    this.bufferFill = 0;
    this.samplesUntilNextFrame = this.fftSize;
  }

  private computeFrame(): Float32Array {
    const start = this.bufferFill - this.fftSize;
    // realTransform wants a 0-based real array of length fftSize.
    const frame = this.buffer.subarray(start, start + this.fftSize);
    this.fft.realTransform(this.spectrum, frame);
    this.fft.completeSpectrum(this.spectrum);
    const sp = this.spectrum;

    const out = new Float32Array(this.numBins);
    for (let k = 0; k < this.numBins; k++) {
      const s = this.kStart[k] as number;
      const len = this.kLen[k] as number;
      let re = 0;
      let im = 0;
      for (let t = 0; t < len; t++) {
        const j = this.kBin[s + t] as number;
        const sr = sp[2 * j] as number;
        const si = sp[2 * j + 1] as number;
        const kr = this.kRe[s + t] as number;
        const ki = this.kIm[s + t] as number;
        re += sr * kr - si * ki;
        im += sr * ki + si * kr;
      }
      out[k] = Math.sqrt(re * re + im * im);
    }
    return out;
  }
}
