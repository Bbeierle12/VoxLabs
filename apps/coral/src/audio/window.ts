/**
 * Hann window. Standard choice for STFT — smooth taper, low spectral
 * leakage, no parameters to tune.
 *
 * Pre-allocate once with createHannWindow, then call applyWindow in the
 * hot loop with a re-used output buffer. NEVER allocate inside the frame
 * loop (Guardrail 7).
 */

const TWO_PI = 2 * Math.PI;

/** Build a Hann window of the given size. Call ONCE, outside any loop. */
export function createHannWindow(size: number): Float32Array {
  if (size <= 0 || !Number.isInteger(size)) {
    throw new Error(
      `createHannWindow: size must be a positive integer, got ${size}`
    );
  }
  const w = new Float32Array(size);
  // Periodic Hann (denom = N), the STFT-analysis convention. Matches
  // librosa/scipy `fftbins=True` — required for oracle-fixture parity and
  // for constant-overlap-add at standard hops. NOT the symmetric
  // filter-design window (denom = N - 1).
  const denom = size;
  for (let i = 0; i < size; i++) {
    w[i] = 0.5 * (1 - Math.cos((TWO_PI * i) / denom));
  }
  return w;
}

/**
 * Apply the window to `samples[start .. start+size]`, writing into `out`.
 * `out` must already have length === window.length.
 */
export function applyWindow(
  samples: Float32Array,
  start: number,
  window: Float32Array,
  out: Float32Array
): void {
  const size = window.length;
  if (out.length !== size) {
    throw new Error(
      `applyWindow: out length ${out.length} does not match window length ${size}`
    );
  }
  if (start < 0 || start + size > samples.length) {
    throw new Error(
      `applyWindow: window range [${start}, ${start + size}) out of bounds for samples length ${samples.length}`
    );
  }
  for (let i = 0; i < size; i++) {
    out[i] = (samples[start + i] as number) * (window[i] as number);
  }
}
