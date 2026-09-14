# Harmonic-analysis math review — 2026-06-10

**Scope:** `src/audio/note-detector.ts` (whitened salience + iterative
harmonic cancellation), `src/config/detector.ts`, the detection-STFT wiring
in `src/audio/dsp-core.ts`, `src/audio/qifft.ts`, `src/audio/intonation.ts`.
Excluded: `section-labeler.ts` (classification layer, not harmonic math) and
`choir-synth.ts` (ground-truth generator).

**Method:** line-by-line formula verification plus numerical probes run
through the production classes (probe file was throwaway, not committed —
results reproduced below). Review-only: no source changes were made.

**Verdict: the core polyphonic detector's math is sound.** No error found in
the detection path itself. The real defects are in satellite math: the QIFFT
pitch estimator octave-flips on a spectral shape common in real voices (its
docstring claims the opposite), and the vibrato detector's band edges are
inverted. Nothing here invalidates the shipped benchmark numbers (F1 0.78)
or the SATB battery.

---

## Verified correct (load-bearing math)

- **MIDI ↔ Hz** both directions, negative-modulo pitch-class wrap, octave
  naming (`floor(midi/12) − 1`) — exact (`intonation.ts:19–26`).
- **±quarter-tone bands**: `2^(1/24)` applied multiplicatively both sides —
  correct geometric ±50 cents (`note-detector.ts` band construction).
- **Whitening**: prefix sums in Float64 (correct precision choice for
  4096-bin sums), edge-clamped box average dividing by the *actual* window
  length (unbiased at edges), envelope floor, noise gate before division.
- **Flat-spectrum normalization is exact, not approximate**: salience
  Σ wₖ·peakₖ normalized by 1/Σ(wₖ over harmonics fitting below Nyquist)
  makes a flat whitened region score exactly 1.0 for every note, so
  `salienceThreshold: 3.0` genuinely reads "3× above flat."
- **Cancellation arithmetic**: each harmonic band scaled by (1−γ) once per
  accepted note. Within-note harmonic bands cannot overlap at K = 7 (would
  require K ≳ 13 at the bass given band width + bin expansion), so no bin is
  double-cancelled.
- **Octave-doubling recall ceiling is arithmetic**: cancelling the lower
  octave (K = 7, γ = 0.9) leaves the upper octave ≈ 0.36× its
  pre-cancellation salience → needs ~8× flat to clear threshold 3.0.
  Matches the documented bench R ≈ 0.33 for `octave-dbl`.
- **Greedy loop invariants**: strict `>` at acceptance and emission;
  accepted-note skip via `frameSalience > 0` is valid (accepted salience is
  always > threshold); residual re-scored every round; `maxPolyphony`
  bounds iterations.
- **Merge-to-centroid**: run grouping by gap ≤ radius and
  nearest-to-energy-weighted-centroid selection implemented correctly.
- **`parabolicPeak`**: standard Smith SASP form `½(α−γ)/(α−2β+γ)`; exact on
  a true parabola; sign convention verified. Measured end-to-end QIFFT bias
  **+2.5 cents at fft 2048** — inside the documented 1–3 cents.
- **Suspicion cleared by probe**: QIFFT at the smallest sidebar fftSize
  (512) measures only **−3.9 cents** on A3 — inside the 10-cent
  "feels in tune" threshold. Not a finding.

## Findings

### P1 — `estimateF0` octave-flips when H2 ≥ H1; docstring claims it can't

`qifft.ts` doc says the band restriction means "a loud upper harmonic can't
masquerade as the pitch." Only true for harmonics above `maxF0Hz` (1100).
For any sung fundamental below 550 Hz, H2 is inside the search band and the
estimator takes the global max-magnitude bin. Probe (200 Hz, three partials):

| H2/H1 | estimateF0 |
|---|---|
| 0.8 | 199.9 Hz ✓ |
| 1.0 | 400.1 Hz ✗ octave error |
| 1.2 | 400.1 Hz ✗ |
| 1.5 | 400.1 Hz ✗ |

H2 > H1 is the normal shape for open vowels with F1 near H2. Mitigating:
the flip is an exact octave, so the **cents value is unaffected** (in-tune
coloring stays honest); the note name and pitch range are an octave wrong
when it triggers. Candidate fix: after peak pick, check bins near peak/2
for a comparable partial and prefer the sub-octave; needs an H2>H1
regression test.

### P2 — vibrato detector band edges inverted; out-of-band rates reported

`detectVibrato` claims 3–8 Hz but uses `minLag = floor(frameRate/8)`,
`maxLag = ceil(frameRate/3)` — rounding widens the band to ~2.7–10.8 Hz at
the default frame rate. Probe (21.53 fps):

| true modulation | reported |
|---|---|
| 2.8 Hz | 2.69 Hz (below claimed band) |
| 9.0 Hz | **3.08 Hz** (aliased INTO the band — reads as plausible vibrato) |
| 10.5 Hz | 10.77 Hz (outside claimed band) |

The 9 Hz alias is the bad case: flutter masquerades as vibrato in the
summary. Fix: swap the rounding (`ceil(fr/8)` / `floor(fr/3)`); optionally
guard against period-multiple aliasing.

### P2 — CQT-mode whitening width conversion wrong-by-construction (latent)

`whiteningWindowHz → bins` uses linear-STFT spacing
`sampleRate/(2·numBins)`; with a CQT `binFreqs` front end the bins are
geometric, so the conversion is meaningless. Latent: CQT is unwired in
production and `cqt.test.ts` masks it by passing explicit
`whiteningWindowBins: 30`. Guard suggestion: throw if `binFreqs` is set
without explicit `whiteningWindowBins`.

### P3 — smaller items

- **`subOctaveRatio` is dead**: validated, stored, threaded from
  `DETECTOR_CONFIG` via `dsp-core.buildDetector`, never read by any
  analysis path. Config comment admits it's superseded. Delete or mark
  deprecated.
- **`analyze()` doesn't validate `magnitudes.length` vs `numBins`** — a
  short frame silently whitens to zeros (undefined fails the gate compare)
  instead of throwing. Production wiring is safe today; failure mode on
  contract violation is silent.
- **Acceptance-order-dependent confidences**: notes accepted on later
  cancellation iterations have salience measured on the already-cancelled
  residual, so `lastConfidences()` understates weaker/harmonic-sharing
  notes. Inherent to greedy cancellation; the section labeler consumes
  these for A/T tie-breaks — document there.
- **High-note lone-partial bias (calibrated behavior, not an error)**:
  normalization by Σw over *fitting* harmonics means a single strong
  partial scores peak/2.59 at C4 but peak/1.5 near the top of range —
  ~1.7× easier activation. Bench's weakest precision categories are `high`
  and `soprano` (both P = 0.667); plausibly the same phenomenon. Experiment
  to confirm/refute: normalize by full Σw always, re-run choir-bench.
- **Adjacent-semitone fundamental-band overlap is real** (probe at fft
  8192: A4 bins [79,85], A#4 [84,90]) — consequence of the deliberate
  floor/ceil over-coverage convention, pinned by test, absorbed by merge.
  Latent constraint worth a code comment: cancellation's
  no-within-note-overlap property breaks if `harmonicCount` is ever raised
  past ~13.

## Suggested follow-ups (not applied)

1. QIFFT sub-octave check + H2>H1 regression test (P1).
2. Vibrato lag rounding swap + band-edge tests (P2).
3. CQT whitening guard (P2, latent).
4. Delete dead `subOctaveRatio`; length guard in `analyze()` (P3).
5. Bench experiment: full-Σw normalization vs current, watch
   `high`/`soprano` precision (P3, research).
