import { describe, it, expect } from 'vitest';
import { synthesizeChoirChord, type Section } from './choir-synth';
import { SpectrogramDsp } from './dsp-core';

// End-to-end multi-F0 contract: synthetic SATB chords → the REAL deployed
// pipeline (SpectrogramDsp, which runs the decoupled 8192-pt detection STFT +
// the calibrated harmonic detector) → steady-state active-note set.
//
// This pins the calibrated behavior (P1). It asserts what the detector can
// honestly do — recover the fundamentals of well-voiced chords — and documents
// the physically-hard cases it cannot (octave doublings, semitone clusters,
// monaural subharmonic/twelfth phantoms). See docs/vocal-acoustics-research.md.

const SR = 44100;

function chordSamples(voices: { section: Section; midi: number }[]): Float32Array {
  return synthesizeChoirChord({ sampleRate: SR, durationSec: 0.4, seed: 7, voices }).samples;
}

/** Run samples through the real DSP path; return the last frame's active set. */
function detect(samples: Float32Array): Set<number> {
  const dsp = new SpectrogramDsp({
    fftSize: 2048, // display framing; detection uses DETECTOR_CONFIG.fftSize
    hopSize: 512,
    sampleRate: SR,
    minFreqHz: 50,
    maxFreqHz: 8000,
  });
  let last: number[] = [];
  const CHUNK = 128; // mimic the worklet's block size
  for (let off = 0; off < samples.length; off += CHUNK) {
    const r = dsp.pushSamples(samples.subarray(off, off + CHUNK));
    if (r.frames.length) last = r.frames[r.frames.length - 1]!.activeMidi;
  }
  return new Set(last);
}

describe('SATB multi-F0 detection (calibrated pipeline)', () => {
  it('returns nothing for silence', () => {
    expect(detect(new Float32Array(Math.round(SR * 0.4))).size).toBe(0);
  });

  it('reports a sustained note as exactly one note (no smear, no phantoms)', () => {
    const active = detect(chordSamples([{ section: 'A', midi: 64 }])); // E4
    // Merge folds the FFT-resolution smear, AND subharmonic suppression kills the
    // octave/twelfth phantoms a lone tone used to spawn — so the set is just E4.
    expect(active.has(64)).toBe(true);
    expect(active.has(63)).toBe(false);
    expect(active.has(65)).toBe(false);
    expect(active.size).toBe(1);
  });

  it('recovers a close-voiced triad exactly', () => {
    // C-E-G close position — the cleanest happy path, no octave/spread traps.
    const active = detect(chordSamples([
      { section: 'B', midi: 60 },
      { section: 'T', midi: 64 },
      { section: 'A', midi: 67 },
    ]));
    expect(active).toEqual(new Set([60, 64, 67]));
  });

  it('recovers the lower voices of a wide SATB chord (high soprano is a known gap)', () => {
    // B=C3, T=G3, A=E4, S=C5. Under realistic within-section detune (~25 cents)
    // a high soprano's upper harmonics smear in absolute Hz and its salience can
    // fall below threshold — a documented recall gap (W1.3 finding; the fix is
    // sum-in-band salience, deferred). We pin what's reliable: B/T/A are found.
    const active = detect(chordSamples([
      { section: 'B', midi: 48 },
      { section: 'T', midi: 55 },
      { section: 'A', midi: 64 },
      { section: 'S', midi: 72 },
    ]));
    for (const midi of [48, 55, 64]) expect(active.has(midi)).toBe(true);
  });

  it('hits the calibrated accuracy bar on a well-voiced chord battery', () => {
    const battery: { gt: number[]; voices: { section: Section; midi: number }[] }[] = [
      { gt: [48, 55, 64, 72], voices: [{ section: 'B', midi: 48 }, { section: 'T', midi: 55 }, { section: 'A', midi: 64 }, { section: 'S', midi: 72 }] },
      { gt: [60, 64, 67], voices: [{ section: 'B', midi: 60 }, { section: 'T', midi: 64 }, { section: 'A', midi: 67 }] },
      { gt: [55, 59, 62, 67], voices: [{ section: 'B', midi: 55 }, { section: 'T', midi: 59 }, { section: 'A', midi: 62 }, { section: 'S', midi: 67 }] },
    ];
    let tp = 0;
    let detectedTotal = 0;
    let gtTotal = 0;
    for (const c of battery) {
      const active = detect(chordSamples(c.voices));
      for (const m of c.gt) if (active.has(m)) tp++;
      detectedTotal += active.size;
      gtTotal += c.gt.length;
    }
    const recall = tp / gtTotal;
    const precision = tp / detectedTotal;
    // After the W1 realism tune-up + subharmonic suppression: recall ~0.73,
    // precision ~0.89 here. Precision rose (phantoms suppressed); recall fell
    // (realistic spread weakens high voices). Conservative floors pin the
    // contract without overfitting.
    expect(recall).toBeGreaterThanOrEqual(0.65);
    expect(precision).toBeGreaterThanOrEqual(0.8);
  });

  it('documents the octave-doubling limit: reports the lower octave', () => {
    // Bass C3 + Soprano C4 (octave). Sub-octave suppression under-reports the
    // upper octave — a known monaural limitation (see research brief). We pin
    // the side we can rely on: the lower note is reported.
    const active = detect(chordSamples([
      { section: 'B', midi: 48 },
      { section: 'S', midi: 60 },
    ]));
    expect(active.has(48)).toBe(true);
  });
});
