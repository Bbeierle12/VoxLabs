import { describe, it, expect } from 'vitest';
import { labelSections } from './section-labeler';
import { synthesizeChoirChord, type Section } from './choir-synth';
import { StreamingStft } from './streaming-stft';
import { NoteDetector } from './note-detector';

const L = (notes: number[], opts?: Parameters<typeof labelSections>[1]) =>
  labelSections(notes, opts).map((x) => x.label);

describe('labelSections', () => {
  it('returns nothing for no notes', () => {
    expect(labelSections([])).toEqual([]);
  });

  it('labels a well-voiced SATB chord B/T/A/S by pitch order', () => {
    // B=C3, T=G3, A=E4, S=C5. The two mid voices sit in the A/T zone but are
    // pinned by a section below and above, so they keep their labels.
    expect(L([48, 55, 64, 72])).toEqual(['B', 'T', 'A', 'S']);
  });

  it('confidently labels notes outside the overlap', () => {
    expect(L([40])).toEqual(['B']); // E2 — bass only
    expect(L([82])).toEqual(['S']); // A#5 — soprano only
  });

  it('declares A/T? for a lone mid-register note (no order, no SPR)', () => {
    expect(L([62])).toEqual(['A/T?']); // D4 — genuinely ambiguous alone
  });

  it('breaks the A/T tie with the SPR lean when decisive', () => {
    expect(L([62], { spr: () => 0.8 })).toEqual(['A']); // bright → alto
    expect(L([62], { spr: () => -0.8 })).toEqual(['T']); // dark → tenor
    expect(L([62], { spr: () => 0.1 })).toEqual(['A/T?']); // weak → still ambiguous
  });

  it('flags the unpinned upper voice of a sparse chord', () => {
    // C3 + C4 octave: lower is bass; the upper is in the zone with nothing
    // above it to pin alto-vs-tenor → ambiguous.
    expect(L([48, 60])).toEqual(['B', 'A/T?']);
  });

  it('dedupes and degrades on a cluster without crashing', () => {
    const out = labelSections([64, 60, 62, 65, 60]); // unsorted + duplicate
    expect(out.map((x) => x.midi)).toEqual([60, 62, 64, 65]); // sorted, deduped
    for (const r of out) {
      expect(['S', 'A', 'T', 'B', 'A/T?']).toContain(r.label);
      expect(r.confidence).toBeGreaterThan(0);
      expect(r.confidence).toBeLessThanOrEqual(1);
    }
  });

  it('scales confidence by the detector confidence map', () => {
    const weak = labelSections([72], { confidence: new Map([[72, 1]]) })[0]!;
    const strong = labelSections([72], { confidence: new Map([[72, 4]]) })[0]!;
    expect(strong.confidence).toBeGreaterThan(weak.confidence);
  });

  it('gives A/T-zone notes lower confidence than clear-register notes', () => {
    const sop = labelSections([72])[0]!; // clear soprano
    const mid = labelSections([62])[0]!; // A/T?
    expect(mid.confidence).toBeLessThan(sop.confidence);
  });

  it('labels a real detected SATB chord end-to-end (detector → confidences → labeler)', () => {
    const SR = 44100;
    const voices: { section: Section; midi: number }[] = [
      { section: 'B', midi: 48 },
      { section: 'T', midi: 55 },
      { section: 'A', midi: 64 },
      { section: 'S', midi: 72 },
    ];
    const { samples } = synthesizeChoirChord({ sampleRate: SR, durationSec: 0.5, seed: 7, voices });
    const stft = new StreamingStft(8192, 2048);
    const det = new NoteDetector({ numBins: 4096, sampleRate: SR, minFreqHz: 50, maxFreqHz: 8000, harmonic: true });
    let active = new Set<number>();
    for (const f of stft.pushSamples(samples)) active = det.analyze(f);

    const labels = labelSections(active, { confidence: det.lastConfidences() });
    expect(labels.length).toBe(active.size);
    const byMidi = new Map(labels.map((l) => [l.midi, l.label]));
    // Clear-register voices, when detected, get their unambiguous section.
    if (active.has(48)) expect(byMidi.get(48)).toBe('B');
    if (active.has(72)) expect(byMidi.get(72)).toBe('S');
  });

  it('hits the P3 label-accuracy floor on labeled SATB chords (no SPR)', () => {
    // Pitch-range labeling (no SPR — SPR was measured to hurt, P3): when it
    // commits to a section it is almost always right, and it abstains (A/T?)
    // on the genuinely hard overlap. Measured ~70% correct / ~4% wrong.
    const SR = 44100;
    const battery: { section: Section; midi: number }[][] = [
      [{ section: 'B', midi: 48 }, { section: 'T', midi: 55 }, { section: 'A', midi: 64 }, { section: 'S', midi: 72 }],
      [{ section: 'B', midi: 55 }, { section: 'T', midi: 59 }, { section: 'A', midi: 62 }, { section: 'S', midi: 67 }],
      [{ section: 'B', midi: 50 }, { section: 'T', midi: 57 }, { section: 'A', midi: 62 }, { section: 'S', midi: 69 }],
      [{ section: 'B', midi: 52 }, { section: 'T', midi: 59 }, { section: 'A', midi: 67 }, { section: 'S', midi: 76 }],
    ];
    let correct = 0;
    let wrong = 0;
    let total = 0;
    for (const voices of battery) {
      const gt = new Map(voices.map((v) => [v.midi, v.section]));
      const { samples } = synthesizeChoirChord({ sampleRate: SR, durationSec: 0.5, seed: 7, voices });
      const stft = new StreamingStft(8192, 2048);
      const det = new NoteDetector({ numBins: 4096, sampleRate: SR, minFreqHz: 50, maxFreqHz: 8000, harmonic: true });
      let active = new Set<number>();
      for (const f of stft.pushSamples(samples)) active = det.analyze(f);
      for (const l of labelSections(active, { confidence: det.lastConfidences() })) {
        const truth = gt.get(l.midi);
        if (truth === undefined) continue; // phantom — not scored
        total++;
        if (l.label === truth) correct++;
        else if (l.label !== 'A/T?') wrong++; // A/T? is an honest abstention
      }
    }
    expect(total).toBeGreaterThan(8);
    expect(correct / total).toBeGreaterThanOrEqual(0.55);
    expect(wrong / total).toBeLessThanOrEqual(0.15);
  });
});
