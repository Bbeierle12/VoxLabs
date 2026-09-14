import { describe, it, expect } from 'vitest';
import { freqToNoteCents, summarize } from './intonation';

const SR = 44100;
const HOP = 512;
const FRAME_RATE = SR / HOP; // ≈86 fps

describe('freqToNoteCents', () => {
  it('maps 440 Hz to A4 at 0 cents', () => {
    const r = freqToNoteCents(440);
    expect(r.noteName).toBe('A4');
    expect(r.midi).toBe(69);
    expect(Math.abs(r.cents)).toBeLessThan(0.5);
  });
  it('maps 442 Hz to A4, +~8 cents (sharp)', () => {
    const r = freqToNoteCents(442);
    expect(r.noteName).toBe('A4');
    expect(r.cents).toBeGreaterThan(6);
    expect(r.cents).toBeLessThan(10);
  });
  it('maps 466.16 Hz to A#4 (sharps for accidentals)', () => {
    const r = freqToNoteCents(466.16);
    expect(r.noteName).toBe('A#4');
    expect(Math.abs(r.cents)).toBeLessThan(1);
  });
  it('maps 261.63 Hz to C4', () => {
    expect(freqToNoteCents(261.63).noteName).toBe('C4');
  });
});

describe('summarize', () => {
  it('reports nothing voiced for empty / all-unvoiced input', () => {
    expect(summarize([], HOP, SR).voicedFraction).toBe(0);
    expect(summarize([null, null, null], HOP, SR).pitchRange).toBeNull();
  });

  it('summarizes a steady 440 Hz tone as A4, ~0 cents', () => {
    const frames = new Array(100).fill(440);
    const s = summarize(frames, HOP, SR);
    expect(s.voicedFraction).toBe(1);
    expect(s.pitchRange).toEqual(['A4', 'A4']);
    expect(Math.abs(s.meanCents)).toBeLessThan(0.5);
    expect(s.rmsCents).toBeLessThan(0.5);
  });

  it('measures vibrato: 440 ±30 cents at 5 Hz → mean≈0, RMS≈21, rate≈5', () => {
    const frames: number[] = [];
    const n = Math.round(FRAME_RATE * 2); // 2 s
    for (let i = 0; i < n; i++) {
      const t = i / FRAME_RATE;
      const dCents = 30 * Math.sin(2 * Math.PI * 5 * t);
      frames.push(440 * Math.pow(2, dCents / 1200));
    }
    const s = summarize(frames, HOP, SR);
    expect(Math.abs(s.meanCents)).toBeLessThan(3);
    expect(s.rmsCents).toBeGreaterThan(17);
    expect(s.rmsCents).toBeLessThan(25);
    expect(s.vibratoRateHz).not.toBeNull();
    expect(s.vibratoRateHz!).toBeGreaterThan(4);
    expect(s.vibratoRateHz!).toBeLessThan(6.5);
  });

  it('counts the voiced fraction', () => {
    const s = summarize([440, null, 440, null, 440], HOP, SR);
    expect(s.voicedFraction).toBeCloseTo(0.6, 5);
  });

  it('does not detect vibrato rates outside the 3-8 Hz band', () => {
    const n = Math.round(FRAME_RATE * 2);
    
    // 9 Hz vibrato (out of band)
    const frames9: number[] = [];
    for (let i = 0; i < n; i++) {
      const t = i / FRAME_RATE;
      const dCents = 30 * Math.sin(2 * Math.PI * 9 * t);
      frames9.push(440 * Math.pow(2, dCents / 1200));
    }
    const s9 = summarize(frames9, HOP, SR);
    expect(s9.vibratoRateHz).toBeNull();

    // 2.5 Hz vibrato (out of band)
    const frames2_5: number[] = [];
    for (let i = 0; i < n; i++) {
      const t = i / FRAME_RATE;
      const dCents = 30 * Math.sin(2 * Math.PI * 2.5 * t);
      frames2_5.push(440 * Math.pow(2, dCents / 1200));
    }
    const s2_5 = summarize(frames2_5, HOP, SR);
    expect(s2_5.vibratoRateHz).toBeNull();
  });
});
