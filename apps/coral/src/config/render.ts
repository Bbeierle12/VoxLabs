/**
 * Spectrogram render parameters.
 *
 * dB scale is calibrated: a 1.0-amplitude sine peaks at 0 dB
 * (window-gain corrected in computeSpectrogram). Defaults give a
 * 100 dB dynamic range running from the noise floor up to 0 dBFS.
 * Quieter material → lower minDb (e.g. -120). Brick-walled masters →
 * raise minDb to e.g. -60 to spread color across the visible range.
 *
 * Frequency range is musical: 50 Hz captures down to ~G1 (lowest bass
 * fundamental), 8 kHz captures up through the singer's-formant region
 * (2.5–3.5 kHz) and harmonics of high sopranos. Linear above the
 * singer's formant adds little for choral work.
 */
export const RENDER_CONFIG = {
  /** Canvas width in pixels. */
  width: 1400,
  /** Canvas height in pixels (includes metadata footer band). */
  height: 1400,
  /** Bottom band reserved for the analysis-metadata footer, in pixels. */
  metadataHeight: 24,
  /** Left band reserved for the frequency-axis labels and tick marks, in pixels. */
  axisWidth: 72,
  /** Magnitudes below this dB level render as the colormap's "floor". */
  minDb: -100,
  /** Magnitudes at or above this dB level render as the colormap's "ceiling". */
  maxDb: 0,
  /** Lowest frequency shown (bottom of spectrogram), Hz. */
  minFreqHz: 50,
  /** Highest frequency shown (top of spectrogram), Hz. */
  maxFreqHz: 8000,
  /** Epsilon added before log to avoid log(0). */
  magnitudeEpsilon: 1e-10,
} as const;

/**
 * Live waterfall geometry + buffering.
 *
 * Orientation: X = log frequency (low at left), Y = time with the NEWEST
 * row at the TOP (it scrolls downward). The bottom carries a horizontal
 * piano-keyboard / Hz axis and a metadata footer.
 *
 * The time history lives in a small DATA-space offscreen buffer
 * (`cols × histRows`) that the GPU scales up to the plot in a single
 * drawImage — not a full display-resolution ring. Scroll cadence is
 * throttled and decoupled from the ~90 fps DSP producer: at most one row
 * advances per `scrollIntervalMs`, and frames arriving between advances
 * are max-pooled per bin so transient peaks survive the downsampling.
 * (Render techniques borrowed from the Resonator waterfall; Coral stays a
 * TypeScript/Vite app — no native backend.)
 *
 * Separate from RENDER_CONFIG so the batch SpectrogramView export keeps
 * its own square geometry untouched.
 */
export const WATERFALL_CONFIG = {
  /** Live canvas width in px (X = frequency). */
  width: 1400,
  /** Live canvas height in px (Y = time + bottom axis bands). */
  height: 760,
  /** Bottom band for the horizontal piano-keyboard / Hz axis, px. */
  keyboardHeight: 64,
  /** Bottom band for the metadata footer, px. */
  footerHeight: 24,
  /** Data-space frequency columns in the history buffer. */
  cols: 1024,
  /** Data-space time rows kept in the history buffer (~17 s at 30 rows/s). */
  histRows: 512,
  /**
   * Minimum ms between row advances. Caps scroll speed and decouples it
   * from the audio hop rate; 33 ms ≈ 30 rows/s.
   */
  scrollIntervalMs: 33,
} as const;
