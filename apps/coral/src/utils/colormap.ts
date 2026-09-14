/**
 * Viridis colormap. Perceptually uniform, colorblind-friendly,
 * looks good on both dark and light backgrounds. Industry-standard
 * default for scientific intensity maps.
 *
 * 16-stop lookup table; values interpolated linearly. Compact enough
 * to inline, smooth enough that nobody will notice the discretization
 * once it's rendered as pixels.
 */

const VIRIDIS_LUT: ReadonlyArray<readonly [number, number, number]> = [
  [68, 1, 84],
  [72, 26, 108],
  [71, 47, 125],
  [65, 68, 135],
  [57, 86, 140],
  [49, 104, 142],
  [42, 120, 142],
  [35, 136, 142],
  [31, 152, 139],
  [34, 168, 132],
  [53, 183, 121],
  [84, 197, 104],
  [122, 209, 81],
  [165, 219, 54],
  [210, 226, 27],
  [253, 231, 37],
] as const;

const LAST_INDEX = VIRIDIS_LUT.length - 1;

/**
 * Map t∈[0,1] to an RGB triple [0..255]. Values outside the range are
 * clamped silently — this is fine for a colormap; values are always
 * derived from a clamped dB→[0,1] mapping upstream.
 */
export function viridis(t: number): readonly [number, number, number] {
  if (t <= 0) return VIRIDIS_LUT[0] as [number, number, number];
  if (t >= 1) return VIRIDIS_LUT[LAST_INDEX] as [number, number, number];

  const scaled = t * LAST_INDEX;
  const lo = Math.floor(scaled);
  const hi = Math.min(LAST_INDEX, lo + 1);
  const frac = scaled - lo;

  const a = VIRIDIS_LUT[lo] as [number, number, number];
  const b = VIRIDIS_LUT[hi] as [number, number, number];

  // Round to integers so Uint8Array stores see the same value tests
  // compare against (plain assignment would truncate instead).
  return [
    Math.round(a[0] + (b[0] - a[0]) * frac),
    Math.round(a[1] + (b[1] - a[1]) * frac),
    Math.round(a[2] + (b[2] - a[2]) * frac),
  ];
}
