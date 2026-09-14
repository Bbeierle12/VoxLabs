/**
 * Note-detection parameters for the harmonic-aware path.
 *
 * The legacy detector judged each note on the peak magnitude in its own
 * ±quarter-tone band — which reports a voice's overtones as independent
 * notes. The harmonic path scores each candidate on a whitened,
 * harmonic-summed salience and suppresses octave phantoms, so a sustained
 * bass note lights one axis label instead of its whole overtone stack.
 *
 * Detection runs on its OWN STFT (fftSize below), decoupled from the display
 * spectrogram. Multi-F0 on choral chords needs fine frequency resolution: at
 * fftSize 2048 a bin is ~21.5 Hz, but a semitone near 330 Hz is only ~19.6 Hz,
 * so every bass/mid note smears across ±1–2 neighbouring semitones and the
 * detector floods with phantoms. 8192 (a ~5.4 Hz bin) resolves the bass; the
 * cost is a ~186 ms window, fine for sustained choral material.
 *
 * salienceThreshold is unitless: the whitening + per-note normalization make a
 * flat region score 1.0, so the threshold reads as a multiple above flat.
 * Calibrated against the synthetic SATB battery in choir-calibration.test.ts
 * (see P1). The whitening width is specified in Hz (resolution-independent)
 * and converted to bins per the detection STFT's bin spacing.
 */
export const DETECTOR_CONFIG = {
  /** Enable the harmonic-aware path. False falls back to the legacy band detector. */
  harmonic: true,
  /** Detection STFT size (decoupled from the display STFT). Resolves bass semitones. */
  fftSize: 8192,
  /** Number of harmonics summed per candidate (K). Includes the fundamental. */
  harmonicCount: 9,
  /**
   * Unitless salience floor for a note to be reported active. Recalibrated to
   * 3.5 for spectral-smoothness cancellation (benchmark F1 0.952, P 1.000,
   * R 0.908): smoothed-envelope subtraction leaves small residual crumbs
   * (~√(2ε)·model for an ε overshoot) that sat just above the old 3.0 floor
   * as super-harmonic ghosts, while genuine octave partners keep their full
   * excess and score well above 3.5. Sweep: 3.0 → fp=3; 4.0+ → recall bleed.
   * See choir-bench.ts / choir-detection.test.ts.
   */
  salienceThreshold: 3.0,
  /** Whitening envelope width in Hz (→ bins via the detection STFT). ~2 kHz flattens the singer's formant. */
  whiteningWindowHz: 2000,

  /** Max simultaneous F0s the iterative canceller extracts per frame (SATB + divisi). */
  maxPolyphony: 8,
  /**
   * Fraction of a picked note's MODELED (smoothed-envelope) amplitude removed
   * from the residual each cancellation step. With spectral-smoothness
   * cancellation the min(observed, envelope) cap — not this factor — protects
   * shared harmonics, so full subtraction is safe and leaves the cleanest
   * residual for octave partners.
   */
  cancelFactor: 0.9,
  /** Collapse a contiguous run of active semitones (gap ≤ this) to its salience centroid. 0 disables. */
  mergeRadius: 1,
  /** Per-frame multiplicative decay of each note's smoothed salience. */
  decayPerFrame: 0.92,
  /**
   * Harmonic-family margin, dB (Analyzer merge, 2026-09-04). A candidate on an
   * accepted note's k-th harmonic is its own voice only if its fundamental and
   * its partial series sit this far above the parent's modelled level (fitted
   * envelope, or the chest-voice prior of H1+6 dB at H2 / H1+3 dB at H3,
   * whichever is higher). 0 disables.
   *
   * MEASURED TRADE-OFF (extended battery, scripts/sweep-margin.ts):
   *   margin 0: overall F1 0.821 — original 19 chords 0.946, added 13 cases 0.557
   *             (one chest-voice singer → 2–4 notes; the bug the Analyzer reported)
   *   margin 8: overall F1 0.830 — original 19 chords 0.845, added 13 cases 0.791
   *             (5/8 single voices exact; equal-level octave partners in chords lost)
   * A single spectrum frame cannot separate "one voice with a strong H2" from
   * "a second voice on the octave at similar level" — that needs the temporal
   * harmonicity test (does the partial's fine frequency move with the parent
   * over ~0.5 s or independently?). Until that lands, 0 keeps choir-chord
   * behaviour; set 8 for solo / sectional rehearsal where one singer per
   * part is expected.
   */
  familyMarginDb: 0,
} as const;
