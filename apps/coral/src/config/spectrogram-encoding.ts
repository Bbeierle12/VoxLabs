/**
 * Wire encoding for spectrogram magnitude frames sent from the DSP
 * worker to the renderer.
 *
 * The worker quantizes each bin's magnitude to a single u8 over a fixed,
 * wide dBFS source range. The renderer then maps that byte back to dB
 * and through the user's display window [minDb, maxDb] via a 256-entry
 * colormap LUT (see spectrogram-lut.ts) — the same two-stage scheme
 * Resonator uses. New rows pick up a rebuilt LUT without re-quantizing
 * anything, and the per-pixel render path stays branch-free. (History
 * rows have their RGB baked at write time, so a clamp change recolors
 * from the change forward — full retroactive recolor would require
 * keeping the byte history and applying the LUT at paint, which is the
 * planned WebGL2 shape.)
 *
 * The range is deliberately wider than any usable display window so the
 * quantization never clips the window. The floor covers quiet material
 * (defaults run -100..0; minDb drops toward -120). The ceiling carries
 * +12 dB of headroom above full scale: a single FFT bin legitimately
 * exceeds 0 dBFS under this calibration when coherent sources share the
 * bin — two in-phase unison voices sum amplitudes (+6 dB), and dense
 * sections stack further. 256 steps over 152 dB ≈ 0.59 dB per step —
 * still finer than the colormap can resolve once rendered.
 *
 * These constants are isolated in their own module so the renderer can
 * import them without pulling the worker's FFT dependency into the main
 * bundle.
 */
export const SPEC_SRC_FLOOR_DB = -140;
export const SPEC_SRC_CEIL_DB = 12;

/** Epsilon added before log to avoid log(0); matches the legacy path. */
export const SPEC_MAGNITUDE_EPSILON = 1e-10;
