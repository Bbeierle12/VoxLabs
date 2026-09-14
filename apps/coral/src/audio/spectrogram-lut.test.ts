import { describe, it, expect } from 'vitest';
import { buildSpectrogramLut } from './spectrogram-lut';
import { quantize } from './dsp-core';
import { viridis } from '../utils/colormap';
import {
  SPEC_SRC_FLOOR_DB,
  SPEC_SRC_CEIL_DB,
} from '../config/spectrogram-encoding';

const rgb = (lut: Uint8Array, v: number): [number, number, number] => [
  lut[v * 3] as number,
  lut[v * 3 + 1] as number,
  lut[v * 3 + 2] as number,
];

describe('buildSpectrogramLut', () => {
  it('produces a 256-entry RGB table', () => {
    expect(buildSpectrogramLut(-100, 0)).toHaveLength(256 * 3);
  });

  it('maps the top byte (wire ceiling, above dispCeil) to the colormap ceiling', () => {
    // Byte 255 decodes to SPEC_SRC_CEIL_DB (+12), above dispCeil 0 here,
    // so t clamps to 1 → top of viridis.
    const lut = buildSpectrogramLut(-100, 0);
    const [r, g, b] = viridis(1);
    expect(rgb(lut, 255)).toEqual([r, g, b]);
  });

  it('clamps bytes below the display floor to the colormap floor', () => {
    // dispFloor -100 dB ≈ byte 67/255 of the [-140, +12] wire range.
    // Bytes at or below the floor → identical floor color.
    const lut = buildSpectrogramLut(-100, 0);
    const [r, g, b] = viridis(0);
    expect(rgb(lut, 0)).toEqual([r, g, b]);
    expect(rgb(lut, 10)).toEqual([r, g, b]);
  });

  it('moves color through the table between floor and ceiling', () => {
    const lut = buildSpectrogramLut(-100, 0);
    // A mid byte should differ from both endpoints.
    expect(rgb(lut, 180)).not.toEqual(rgb(lut, 0));
    expect(rgb(lut, 180)).not.toEqual(rgb(lut, 255));
  });

  it('lowering the display floor brightens a fixed mid byte', () => {
    // Byte 150 decodes to ≈ -50.6 dB on the [-140, +12] wire range. With
    // the ceiling fixed at 0, a lower floor maps that same dB to a higher
    // t → brighter (viridis green rises with t through the mid range).
    const higherFloor = buildSpectrogramLut(-100, 0);
    const lowerFloor = buildSpectrogramLut(-120, 0);
    expect(lowerFloor[150 * 3 + 1]).toBeGreaterThanOrEqual(higherFloor[150 * 3 + 1] as number);
  });
});

describe('quantize ↔ LUT wire-range round trip', () => {
  // The encoder (dsp-core quantize) and decoder (LUT byte→dB mapping)
  // must agree on [SPEC_SRC_FLOOR_DB, SPEC_SRC_CEIL_DB]. This round-trip
  // pins them together: if either side drifts from the shared constants,
  // the reconstruction error blows past one quantization step.
  const SPAN = SPEC_SRC_CEIL_DB - SPEC_SRC_FLOOR_DB;
  const STEP = SPAN / 255;

  const decode = (byte: number): number =>
    SPEC_SRC_FLOOR_DB + (byte / 255) * SPAN;

  it('reconstructs dB within half a quantization step across the wire range', () => {
    // Sweep the open interval; exact endpoints clamp by design.
    for (let db = SPEC_SRC_FLOOR_DB + 0.5; db < SPEC_SRC_CEIL_DB; db += 0.5) {
      const mag = Math.pow(10, db / 20);
      const byte = quantize(new Float32Array([mag]))[0] as number;
      const decoded = decode(byte);
      // Half a step of quantization error, plus a small allowance for the
      // log-epsilon the encoder adds (only visible near the floor).
      expect(Math.abs(decoded - db)).toBeLessThanOrEqual(STEP / 2 + 0.02);
    }
  });

  it('clamps magnitudes outside the wire range to the endpoint bytes', () => {
    const aboveCeil = Math.pow(10, (SPEC_SRC_CEIL_DB + 10) / 20);
    const belowFloor = Math.pow(10, (SPEC_SRC_FLOOR_DB - 10) / 20);
    const bytes = quantize(new Float32Array([aboveCeil, belowFloor]));
    expect(bytes[0]).toBe(255);
    expect(bytes[1]).toBe(0);
  });

  it('keeps coherent-summation headroom: +6 dBFS does not clip to the ceiling byte', () => {
    // Two in-phase unison voices sum to amplitude 2.0 (+6 dBFS). The wire
    // ceiling exists so this lands BELOW byte 255 with structure intact.
    const mag = 2.0;
    const byte = quantize(new Float32Array([mag]))[0] as number;
    expect(byte).toBeLessThan(255);
    expect(Math.abs(decode(byte) - 20 * Math.log10(mag))).toBeLessThanOrEqual(STEP / 2 + 0.02);
  });
});
