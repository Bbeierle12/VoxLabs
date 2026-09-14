/**
 * Intonation visualization tunables: the cents-deviation color bands for the
 * pitch trail (defaults from FOUNDATIONS.md's perception thresholds — ~10 cents
 * is the "feels in tune" boundary) and the F0 voicing parameters.
 */
export const INTONATION_CONFIG = {
  /** Pitch-trail color by |cents from nearest ET note|, first match wins. */
  bands: [
    { maxCents: 5, color: '#fde725' }, // viridis yellow — tuner-grade
    { maxCents: 15, color: '#5ec962' }, // green — musically in tune
    { maxCents: 30, color: '#2a9d8f' }, // cyan/teal — drifting
    { maxCents: Infinity, color: '#b07fd6' }, // purple — out of tune
  ],
  /** F0 search band (vocal fundamentals) + voiced gate, for qifft.estimateF0. */
  minF0Hz: 65,
  maxF0Hz: 1100,
  voicedRatio: 5,
} as const;

/** Color for a signed/unsigned cents deviation, by the configured bands. */
export function centsColor(cents: number): string {
  const a = Math.abs(cents);
  for (const band of INTONATION_CONFIG.bands) {
    if (a <= band.maxCents) return band.color;
  }
  return INTONATION_CONFIG.bands[INTONATION_CONFIG.bands.length - 1]!.color;
}
