import { describe, it, expect } from 'vitest';
import { runBench, formatReport, BENCH_CASES, type BenchCase } from './choir-bench';
import { SpectrogramDsp } from './dsp-core';

const SR = 44100;

/** Run samples through the real DSP path; return the last frame's active set. */
function detectViaPipeline(samples: Float32Array): Set<number> {
  const dsp = new SpectrogramDsp({ fftSize: 2048, hopSize: 512, sampleRate: SR, minFreqHz: 50, maxFreqHz: 8000 });
  let last: number[] = [];
  const CHUNK = 128;
  for (let off = 0; off < samples.length; off += CHUNK) {
    const r = dsp.pushSamples(samples.subarray(off, off + CHUNK));
    if (r.frames.length) last = r.frames[r.frames.length - 1]!.activeMidi;
  }
  return new Set(last);
}

// On-demand: prints the full benchmark report (~19s). Un-skip between tiers to
// compare. Progress (2026-06-04):
//   post-W1.2: P=0.776 R=0.585 F1=0.667. Weak: bass/low R=0.25,
//     wide/high/soprano R=0.50, octave-dbl R=0.50, unison P=0.00.
//   T1 (centroid merge; sum-in-band salience rejected): P=0.796 R=0.600 F1=0.684.
//   T2 (iterative harmonic cancellation, threshold 3.0): P=0.842 R=0.738 F1=0.787.
//     unison P=0.00 -> 1.00; recall +0.14. octave-dbl still ~0.33 (physics).
//   T3 (CQT front end): REJECTED by gate — CQT F1~0.64 vs STFT ~0.92 (subset,
//     1.6s). Side-finding: STFT F1 0.79->0.92 from 0.4s->1.6s integration.
describe.skip('detector benchmark (full report, on demand)', () => {
  it('runs the full battery and logs the report', () => {
    const rep = runBench(BENCH_CASES, detectViaPipeline);
    console.log('\n' + formatReport(rep));
    expect(rep.cases.length).toBe(BENCH_CASES.length);
  });
});

// Permanent baseline guard: a small well-voiced subset, asserting the post-W1.2
// floor so later tiers must not regress it.
describe('detector benchmark baseline', () => {
  it('holds the recorded floor on well-voiced chords', () => {
    // Well-voiced subset (post-W1.2: P≈0.83, R≈0.67). Floors leave headroom so
    // the guard catches regressions; the on-demand full report tracks progress.
    const subset: BenchCase[] = BENCH_CASES.filter((c) =>
      ['Cmaj wide', 'Fmaj close', 'Gmaj wide', 'CEG triad'].includes(c.name)
    );
    const rep = runBench(subset, detectViaPipeline);
    expect(rep.overall.precision).toBeGreaterThanOrEqual(0.8);
    expect(rep.overall.recall).toBeGreaterThanOrEqual(0.6);
  });
});
