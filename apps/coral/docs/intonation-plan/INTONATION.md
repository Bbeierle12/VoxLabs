# Coral — Intonation Visualization Plan

> **STATUS (2026-06): SHIPPED.** The intonation feature landed in June 2026 —
> QIFFT single-F0 + ET cents (`src/audio/qifft.ts`, `src/audio/intonation.ts`),
> live pitch trail + cents readout + summary on the waterfall. This document is
> the original build plan, kept for design rationale; where it disagrees with
> the shipped code, the code wins.

The v1 feature that turns Coral from "a choral spectrogram" into "a choral analysis tool a director would use." This document is the build plan: scope cap, design decisions, test plan, sequencing. It lives under the same planning guardrails as the rest of the project (1-page spec ceiling, 3-feature cap, tests first).

## Goal

After this lands, a user drops a recording of a single voice (or a mono mix of a choir) and sees not just *what's there* but *how in-tune it is*: a pitch line on top of the spectrogram, a frame-by-frame note name with cents deviation, and a summary of how the performance compared to a 12-TET reference.

That answers the central pedagogical question — "are we singing in tune?" — at the level a v1 tool can. Just intonation, section dispersion, drift telemetry, and live mic input all stay deferred.

## Three-feature MVP cap

1. **QIFFT F0 extraction.** Frame-by-frame fundamental-frequency estimate derived from the existing STFT, using parabolic (quadratic) interpolation around each frame's peak bin for sub-bin frequency accuracy. Pure function: `SpectrogramData → F0Track`. Voiced/unvoiced gate via a magnitude-ratio confidence threshold.

2. **Pitch trail overlay on the spectrogram.** An SVG line drawn on top of the canvas at the F0 trajectory's y-coordinates (same log-frequency mapping as the existing NoteGrid). Visually hidden where confidence is below threshold. Color-coded by cents-deviation from the nearest 12-TET note (configurable thresholds).

3. **Per-clip intonation summary.** A small readout panel below the spectrogram showing: detected pitch range as note names ("F2 — A4"), mean cents deviation from nearest ET note, RMS cents error, percentage of frames voiced, mean vibrato rate if detectable.

That's the cap. Three features. No more in this iteration.

## Explicit non-goals (FUTURE.md material)

- **Multi-voice / multi-F0**. Mono mix only for v1. Multi-track support comes with the section-tagger feature, separately.
- **Just intonation reference overlay.** ET-only readout for v1. JI overlay needs adaptive tonal-center inference (research-grade) or a manually-set key (UI complexity); both are v2.
- **Interactive scrub / hover**. Static summary only. Adding hover means adding a mouse-tracking layer over the canvas and a "current frame" state; that's the wrong place to spend time before we know the trail and summary are useful.
- **Live mic input**. Offline-file pipeline only.
- **Pitch drift visualization across long performances.** A drift graph (running mean F0 over time) is mechanically simple but UX-loaded; defer until the summary readout is proven.
- **CREPE / pYIN / ML-based F0**. QIFFT is the right MVP estimator because it builds on math we already trust. CREPE goes in if and only if QIFFT proves insufficient on real audio.
- **Multi-pitch / polyphonic F0 estimation**. Single-best F0 per frame for v1. Top-N candidates is a different algorithm.

## Algorithm decisions

**F0 estimator: QIFFT (quadratic-interpolated FFT).** Reasons:

- Zero new dependencies; consumes the existing `SpectrogramData` buffer.
- Mathematically well-characterized: Smith's *Spectral Audio Signal Processing* §5.7 derives the bias bounds (~1.5 cents max for Hann window, much less in the middle of the bin).
- At our fftSize = 4096 / 44.1 kHz, raw bin width is 21 cents at A4. QIFFT improves this to roughly 1-3 cents in the typical case — comfortably under the 10-cent "feels in tune" perceptual threshold from FOUNDATIONS.md.
- Where QIFFT struggles (very low frequencies, where bin width approaches a semitone): we accept the limitation for v1 and document it. The bass below ~80 Hz is the weak spot.

The formula, fixed in `qifft.ts`:

```
p = 0.5 * (α − γ) / (α − 2β + γ)
f = (k* + p) * sampleRate / fftSize
```

where α, β, γ are the magnitudes (or log magnitudes — both work; log is more accurate, see Smith) at bins k*−1, k*, k*+1, and k* is the peak bin.

**Voiced/unvoiced gate: peak-to-mean magnitude ratio.** When the strongest peak in a frame is less than a configurable multiple of the frame's mean magnitude, the frame is "unvoiced" and F0 = null. Simple, robust on synthetic test signals; we'll see how it holds up on real audio. Spectral-flatness gate is the v2 upgrade if needed.

**Tuning reference: equal temperament only.** The cents readout shows distance from the nearest 12-TET note. The reasoning is in FOUNDATIONS.md Part VI: choirs trained against piano accompaniment use ET as a melodic baseline, then shift to JI on sustained chords. ET is the right "neutral" reference; JI overlay is a v2 contextual addition.

**Color coding (pitch trail): cents-deviation bands.**

| |cents deviation| | color | meaning |
|---|---|---|
| ≤ 5 | viridis-yellow | tuner-grade in-tune |
| ≤ 15 | viridis-green | musically in-tune |
| ≤ 30 | viridis-cyan | drifting |
| > 30 | viridis-purple | out-of-tune |

Bands are config-driven and easy to retune. Default values from FOUNDATIONS.md's perception thresholds.

## Architecture

New files:

```
src/
  audio/
    qifft.ts                # parabolic interpolation, peak refinement
    qifft.test.ts
    f0-track.ts             # SpectrogramData → F0Track with confidence
    f0-track.test.ts
    intonation.ts           # F0Track → ET-relative cents, note names, summary
    intonation.test.ts
  components/
    PitchTrail.tsx          # SVG overlay rendering the F0 line
    IntonationSummary.tsx   # the readout panel
  config/
    f0.ts                   # confidence thresholds, voicing parameters
    intonation.ts           # cents bands, color mapping
```

Modified files:

- `SpectrogramView.tsx` — adds a second SVG overlay (PitchTrail) above the NoteGrid layer
- `App.tsx` — runs the F0 + intonation pipeline after computeSpectrogram, threads results into view
- `TESTING.md` — Tier 3 section, currently deferred, gets implemented here

Data types:

```typescript
interface F0Frame {
  frequencyHz: number | null;   // null = unvoiced
  confidence: number;            // 0..1, peak-to-mean ratio
}

interface F0Track {
  frames: F0Frame[];             // one per spectrogram frame
  hopSize: number;
  sampleRate: number;
}

interface IntonationFrame {
  noteName: string | null;       // "A4", null when unvoiced
  centsFromNearest: number;      // signed; positive = sharp
  midiNumber: number | null;
}

interface IntonationSummary {
  voicedFraction: number;        // 0..1
  pitchRangeNotes: [string, string] | null;  // ["F2", "A4"]
  meanCentsDeviation: number;    // signed mean over voiced frames
  rmsCentsDeviation: number;
  vibratoRateHz: number | null;  // detected via autocorrelation of F0 deviation
}
```

## Test plan

Maps to TESTING.md Tier 3, defined for the first time:

**Tier 3.1 — QIFFT correctness (math)**

- Pure sine at exact bin center: QIFFT returns the bin frequency, p = 0
- Pure sine at half-bin: QIFFT returns within ±5 cents of true (Hann bias bound)
- Pure sine sweep across A2–C6 in 5-cent increments: RMS error < 5 cents, max < 15 cents
- Edge case: peak at bin 0 (DC) returns the bin frequency without crashing
- Edge case: peak at last bin returns sensibly (no out-of-bounds read)

**Tier 3.2 — F0 track behavior**

- Silence → all frames unvoiced
- White noise → < 10% voiced (high false-alarm threshold)
- Pure sine + appropriate noise → ≥ 95% voiced
- Linear chirp → monotonically increasing F0
- Vibrato signal → F0 oscillates at the vibrato rate within ±20 cents of nominal

**Tier 3.3 — Intonation summary**

- A 440 Hz sine → noteName "A4", cents = 0
- A 442 Hz sine → noteName "A4", cents = +8
- A 415 Hz sine → noteName "G#4" or "A4" (boundary case, document expected behavior)
- Vibrato 440 ±30 cents → mean cents ≈ 0, RMS cents ≈ 21 (RMS of a 30-cent-amplitude sine)
- Linear chirp 200→4000 → pitch range from low note to high note; mean deviation depends on which notes the sweep crosses

Tier 4 (real-data validation against Dagstuhl ChoirSet F0 annotations) remains deferred — that's a separate workstream involving dataset infrastructure.

## Acceptance bar

Before this is mergeable to main:

1. All Tier 3 tests above are green
2. The existing 66 tests still pass
3. The pitch trail visibly tracks the F0 of a vibrato test signal (eyeball verification in the debug panel)
4. The intonation summary correctly identifies A4 = "A4, +0¢" and A4 + 50¢ ≈ "A4, +50¢"
5. README.md and TESTING.md updated to reflect the new feature surface
6. No file over 500 lines
7. Type-check clean, production build clean

## Sequencing

Estimated 1–2 working sessions. In strict order:

**Session 1**

1. Write Tier 3.1 tests for QIFFT (TDD per Guardrail 3)
2. Implement `qifft.ts` until tests pass
3. Write Tier 3.2 tests for F0 track
4. Implement `f0-track.ts` until tests pass
5. Validate visually: load a vibrato signal, log F0 to console, confirm it tracks

**Session 2**

6. Write Tier 3.3 tests for intonation summary
7. Implement `intonation.ts` until tests pass
8. Build `PitchTrail.tsx` SVG overlay
9. Build `IntonationSummary.tsx` panel
10. Wire into `App.tsx` and `SpectrogramView.tsx`
11. Visual eyeball pass: every reference signal in the debug panel renders the trail correctly
12. Update README.md and TESTING.md

If anything in Session 1 takes longer than expected, Session 2's work shifts. Do NOT begin Session 2 before Session 1 is green; the visualization is dead weight without correct F0 underneath.

## Open questions

These deserve explicit answers before I start writing code, but they are not blocking — the listed default works.

1. **Where does the "best peak" search happen — over the full bin range, or restricted to a vocal-fundamental band (65 Hz – 1100 Hz)?**
   *Default:* restricted to the vocal range. A loud harmonic at 2 kHz can outpeak the fundamental at 440 Hz in some choral material, biasing detection. Restricting to the vocal-fundamental band per `config/notes.ts`'s `VOICE_RANGES` is the standard MIR move and matches what pYIN does internally.

2. **Smoothing of the F0 track?**
   *Default:* none in v1. Raw QIFFT output frame-by-frame. Median filtering, HMM smoothing, etc. are well-studied (pYIN uses HMM) but each adds a tunable knob. Ship raw, then smooth if needed.

3. **Vibrato detection — implement now or defer?**
   *Default:* implement a simple version (autocorrelation of the cents-deviation signal, peak in 3–8 Hz band). Marked as part of summary feature #3. If it turns out flaky on real material, demote to "deferred" and remove from the summary panel.

4. **Note-name preference for sharp-vs-flat ambiguity (e.g., G#4 vs Ab4)?**
   *Default:* sharps. Matches what `midiToNoteName` already does. Add a config flag if anyone complains.

## What this earns

If all three features land cleanly, Coral becomes the first tool in the public landscape (per FOUNDATIONS.md Part VII.7) that puts a choral-tuned spectrogram and a real intonation readout in the same view for free, in a browser, with no install. It's still v1 — single voice, ET only, offline only — but it's a genuinely useful artifact for choral pedagogy and a credible base for the multi-channel / JI / live-mic upgrades that follow.
