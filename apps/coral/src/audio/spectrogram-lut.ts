import { viridis } from '../utils/colormap';
import { SPEC_SRC_FLOOR_DB, SPEC_SRC_CEIL_DB } from '../config/spectrogram-encoding';

/**
 * Precompute a 256-entry RGB lookup table mapping a quantized magnitude
 * byte (encoded over [SPEC_SRC_FLOOR_DB, SPEC_SRC_CEIL_DB] by the DSP
 * worker) to a display color.
 *
 * Each byte is decoded back to dB, remapped through the user's display
 * window [dispFloor, dispCeil] into t∈[0,1], then run through the
 * colormap. Building this once per parameter change turns the per-pixel
 * render path into three array reads — no log, no pow, no allocation —
 * which is the point of moving the dB math off the hot loop. Mirrors
 * Resonator's web waterfall LUT.
 *
 * Layout: `lut[v * 3 + {0,1,2}]` is the R/G/B for source byte `v`.
 */
export function buildSpectrogramLut(
  dispFloor: number,
  dispCeil: number,
  gamma = 1,
  srcFloor: number = SPEC_SRC_FLOOR_DB,
  srcCeil: number = SPEC_SRC_CEIL_DB
): Uint8Array {
  const lut = new Uint8Array(256 * 3);
  const srcSpan = srcCeil - srcFloor;
  const dispSpan = dispCeil - dispFloor || 1;
  for (let v = 0; v < 256; v++) {
    const db = srcFloor + (v / 255) * srcSpan;
    let t = (db - dispFloor) / dispSpan;
    t = t < 0 ? 0 : t > 1 ? 1 : t;
    if (gamma !== 1) t = Math.pow(t, gamma);
    const [r, g, b] = viridis(t);
    lut[v * 3] = r;
    lut[v * 3 + 1] = g;
    lut[v * 3 + 2] = b;
  }
  return lut;
}
