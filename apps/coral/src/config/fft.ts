/**
 * FFT and STFT analysis parameters.
 *
 * Frequency resolution = sampleRate / fftSize
 *   At 44.1 kHz, fftSize=2048 → ~21.5 Hz bin width
 *   That's roughly a quarter-step at A4 (440 Hz) — fine enough for
 *   musical pitch tracking, coarse enough to keep render cost manageable.
 *
 * Time resolution = hopSize / sampleRate
 *   hopSize=512 → 11.6 ms at 44.1 kHz, 75% overlap with fftSize=2048.
 *
 * These are tuned for choral analysis (50 Hz–8 kHz, slow-evolving
 * harmonic content). Live mic and percussion-heavy material may
 * benefit from smaller fftSize / hopSize.
 */
export const FFT_CONFIG = {
  /** FFT window length in samples. Must be power of 2. */
  fftSize: 2048,
  /** Samples between consecutive STFT frames. fftSize / 4 = 75% overlap. */
  hopSize: 512,
} as const;
