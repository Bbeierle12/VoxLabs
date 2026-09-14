/**
 * QIFFT — single-F0 estimation by quadratic (parabolic) interpolation of the
 * STFT magnitude peak. The fundamental of a voice rarely lands on a bin centre;
 * fitting a parabola to the peak bin and its two neighbours recovers the
 * sub-bin frequency to ~1–3 cents (Smith, *Spectral Audio Signal Processing*
 * §5.7), well under the ~10-cent "feels in tune" threshold even though a raw
 * bin is ~21 cents wide at A4 / fft 2048.
 *
 * This is the intonation feature's estimator: single best F0 per frame, NOT the
 * polyphonic chord detector. The peak search is restricted to the vocal
 * fundamental band so a loud upper harmonic can't masquerade as the pitch.
 */

export interface F0Estimate {
  frequencyHz: number;
  /** 0..1, from the peak-to-mean magnitude ratio. */
  confidence: number;
}

export interface QifftOptions {
  /** Lowest fundamental to search, Hz. Default 65 (~C2). */
  minF0Hz?: number;
  /** Highest fundamental to search, Hz. Default 1100 (~C#6). */
  maxF0Hz?: number;
  /** Voiced if peak/mean magnitude ≥ this. Default 5. */
  voicedRatio?: number;
}

const EPS = 1e-12;

/**
 * Sub-bin peak offset in [-0.5, 0.5] from a parabola through three samples
 * (use log magnitudes for the lowest bias). `y0` is the peak.
 */
export function parabolicPeak(ym1: number, y0: number, yp1: number): number {
  const denom = ym1 - 2 * y0 + yp1;
  if (denom === 0) return 0;
  const p = (0.5 * (ym1 - yp1)) / denom;
  return Math.max(-0.5, Math.min(0.5, p));
}

/**
 * Estimate the fundamental of one magnitude frame, or null if unvoiced.
 * `mags` has length fftSize/2; bin b sits at b·sampleRate/fftSize Hz.
 */
export function estimateF0(
  mags: Float32Array,
  sampleRate: number,
  fftSize: number,
  opts: QifftOptions = {}
): F0Estimate | null {
  const n = mags.length;
  if (n < 3) return null;
  const binFreqHz = sampleRate / fftSize;
  const minF0 = opts.minF0Hz ?? 65;
  const maxF0 = opts.maxF0Hz ?? 1100;
  const voicedRatio = opts.voicedRatio ?? 5;

  const loBin = Math.max(1, Math.floor(minF0 / binFreqHz));
  const hiBin = Math.min(n - 2, Math.ceil(maxF0 / binFreqHz));
  if (hiBin <= loBin) return null;

  let peakBin = loBin;
  let peakMag = -Infinity;
  for (let b = loBin; b <= hiBin; b++) {
    const m = mags[b] as number;
    if (m > peakMag) {
      peakMag = m;
      peakBin = b;
    }
  }

  let sum = 0;
  for (let b = 0; b < n; b++) sum += mags[b] as number;
  const mean = sum / n;
  if (!(mean > 0) || peakMag / mean < voicedRatio) return null;

  // Sub-octave check to prevent octave-flipping when H2 >= H1 (e.g. open vowels).
  let finalPeakBin = peakBin;
  const subBin = Math.round(peakBin / 2);
  if (subBin >= loBin) {
    let subPeakBin = subBin;
    let subPeakMag = mags[subBin] as number;
    for (let b = subBin - 1; b <= subBin + 1; b++) {
      if (b >= loBin && b <= hiBin) {
        const m = mags[b] as number;
        if (m > subPeakMag) {
          subPeakMag = m;
          subPeakBin = b;
        }
      }
    }
    // If the sub-octave has a peak at least 30% as large as the global peak
    // and is a local maximum (larger than or equal to its immediate neighbours)
    if (subPeakMag > 0.3 * peakMag) {
      const left = mags[subPeakBin - 1] as number;
      const right = mags[subPeakBin + 1] as number;
      if (subPeakMag >= left && subPeakMag >= right) {
        finalPeakBin = subPeakBin;
      }
    }
  }

  const a = Math.log((mags[finalPeakBin - 1] as number) + EPS);
  const b0 = Math.log((mags[finalPeakBin] as number) + EPS);
  const g = Math.log((mags[finalPeakBin + 1] as number) + EPS);
  const p = parabolicPeak(a, b0, g);
  const f0 = (finalPeakBin + p) * binFreqHz;
  if (!(f0 > 0)) return null;

  const confidence = Math.min(1, peakMag / mean / (2 * voicedRatio));
  return { frequencyHz: f0, confidence };
}
