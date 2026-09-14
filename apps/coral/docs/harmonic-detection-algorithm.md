# Coral — Harmonic-Salience Multi-F0 Detector

*Algorithm outline for external review. Last updated 2026-06-03.*

## 0. Naming — read this first

The project vision calls the long-term goal **"harmonic fingerprinting"** (telling
individual singers apart). **That is not what this algorithm does, and it is not
yet implemented.** What exists today is the substrate for it: a **whitened
harmonic-summation multi-F0 pitch detector** (a "harmonic sieve") that decides
*which pitches are sounding* in a single ambient-mic signal. It does not attribute
energy to individual singers, and (today) not to SATB sections either — section
labeling is designed but unbuilt (see §8).

Reviewers should evaluate it as a **multi-F0 / pitch-salience detector**, in the
family of subharmonic-summation (Hermes 1988) and harmonic-salience multi-F0
methods (Klapuri 2006/2008) — not as a source-separation or speaker-ID system.

## 1. Where it lives

| Concern | File / symbol |
|---|---|
| Core algorithm | `src/audio/note-detector.ts` → `NoteDetector.analyzeHarmonic`, `whiten`, `mergeAdjacent` |
| STFT (analysis front end) | `src/audio/streaming-stft.ts` → `StreamingStft` |
| Pipeline wiring (decoupled detection STFT) | `src/audio/dsp-core.ts` → `SpectrogramDsp.pushSamples` |
| Tunable parameters | `src/config/detector.ts` → `DETECTOR_CONFIG` |
| Ground-truth generator (eval) | `src/audio/choir-synth.ts` |
| Validation tests | `src/audio/choir-synth.test.ts`, `src/audio/choir-detection.test.ts` |

## 2. Signal chain

```
mic / file → AudioWorklet (128-sample blocks) → DSP worker (off-thread)
  → StreamingStft  (fftSize 8192, hop 2048, Hann)        ── DETECTION path
  → magnitude frame  X[b], b = 0 … N−1   (N = fftSize/2 = 4096)
  → NoteDetector.analyzeHarmonic(X)  → set of active MIDI notes
```

The detection STFT is **deliberately decoupled** from the display spectrogram's
STFT (2048-pt). Multi-F0 needs fine frequency resolution: at 2048 a bin is
~21.5 Hz while a semitone near 330 Hz is only ~19.6 Hz, so notes smear across
neighbouring semitones. At 8192 a bin is ~5.4 Hz, which resolves the bass; the
cost is a ~186 ms analysis window, acceptable for sustained choral material.
Detection updates at sampleRate/hop ≈ **21.5 frames/s**.

Magnitudes are amplitude-calibrated: a unit-amplitude, bin-centred sinusoid
peaks at magnitude ≈ 1.0 (`invScale = 2/Σ window`).

## 3. Pre-computed state (per configuration)

Bound once to `(numBins, sampleRate, [minFreqHz, maxFreqHz])`:

- **Candidate notes** i = 0 … M−1, MIDI = `midiMin + i`, where `midiMin/midiMax`
  come from the frequency range (50–8000 Hz → MIDI ≈ 31–120, so M ≈ 90).
  Fundamental of note i: `f0(i) = 440 · 2^((MIDIᵢ − 69)/12)`.
- **Harmonic bands.** For each note i and harmonic h = 1 … K (K = 7), a
  ±quarter-tone band in bins around `h·f0(i)`:
  `[ floor( h·f0 · 2^(−1/24) / binHz ), ceil( h·f0 · 2^(+1/24) / binHz ) ]`,
  clamped to `[0, N−1]`. Harmonics at/above Nyquist are flagged "skip".
  (`binHz = sampleRate / fftSize`.) Bands reach the full spectrum up to Nyquist,
  not just the display ceiling.
- **Harmonic weights** `w(h) = 1/h` (fundamental 1, then 1/2, 1/3, … 1/7).
- **Per-note normalisation** `norm(i) = 1 / Σ_{h present} w(h)` so that a *flat*
  whitened spectrum scores exactly **1.0** for every note, regardless of how many
  harmonics fit under Nyquist. This makes the salience threshold read as a
  multiple above the flat baseline.
- **Whitening window** `W = round(whiteningWindowHz / binHz)` bins (≈ 372 bins
  ≈ 2 kHz at fft 8192). Specified in Hz so it stays constant across fftSizes.

## 4. Per-frame algorithm

Input: one magnitude frame `X[0…N−1]`.

**Stage 1 — Spectral whitening (formant flattening + noise gate).**
Compute a local box-average envelope via prefix sums (O(N)):
`env[b] = mean( X[b−W/2 … b+W/2] )`. Then
```
Xw[b] = X[b] / max(env[b], ε)      if X[b] ≥ gate
        0                           otherwise
```
`gate = 10^(−50/20) ≈ 0.00316` (−50 dBFS). Whitening divides out broad spectral
shape — most importantly the singer's-formant bump (2.4–3.1 kHz) — so a candidate
is scored on harmonic *structure*, not raw loudness. The gate stops silence/noise
between harmonics from normalising up to ~1.

**Stage 2 — Harmonic summation (salience).**
For each candidate note i:
```
salience(i) = norm(i) · Σ_{h=1..K, present}  w(h) · max_{b ∈ band(i,h)} Xw[b]
```
A note with a full overtone stack accrues weighted energy across all its
harmonic bands; a lone partial (e.g. one isolated overtone) gets support at only
one band and stays near the flat baseline (~1.0).

**Stage 3 — Temporal smoothing (decay-then-max envelope).**
```
energy(i) ← max( salience(i), decay · energy(i) )      decay = 0.92
```
Fast attack (a louder frame overrides history immediately), exponential release.
Prevents flicker when a note grazes the threshold. State is per note across
frames; `reset()` clears it.

**Stage 4 — Threshold + sub-octave (octave-phantom) suppression.**
Note i is a candidate-active iff `energy(i) > τ` (τ = salienceThreshold = 4.0).
Then suppress octave phantoms: a candidate sitting on a lower note's 2nd harmonic
accrues real salience from shared even harmonics, but the true fundamental an
octave below scores higher off its denser harmonic set. So:
```
drop note i  if  energy(i − 12 semitones) > ρ · energy(i)      ρ = 0.7
```

**Stage 5 — Resolution-smear merge.**
Collapse each contiguous run of active MIDI numbers (gap ≤ `mergeRadius` = 1) to
its single highest-`energy` member. This folds the residual ±1-semitone smear of
a chorus-spread note back into one note. (Tradeoff: a *genuine* minor-second
cluster also collapses — the FFT can't reliably resolve those anyway.)

**Output:** the set of MIDI note numbers reported active this frame.

## 5. Parameters (calibrated — `DETECTOR_CONFIG`)

| Parameter | Value | Role |
|---|---|---|
| `fftSize` | 8192 | Detection STFT (decoupled from 2048 display) |
| `harmonicCount` K | 7 | Harmonics summed per candidate |
| `salienceThreshold` τ | 4.0 | Activation floor (multiple above flat baseline) |
| `whiteningWindowHz` | 2000 | Whitening envelope width (→ bins per resolution) |
| `subOctaveRatio` ρ | 0.7 | Octave-phantom suppression strength |
| `mergeRadius` | 1 | Semitone-smear merge radius |
| `decayPerFrame` | 0.92 | Temporal release |

`τ = 4.0` is the **calibrated** value (see §7); it was the F1 peak. The whitening
gate uses the legacy `thresholdDb = −50`.

## 6. Complexity & cadence

Per frame: whitening O(N) (prefix sums); summation O(M · K · B̄) where B̄ is the
mean band width in bins; suppression O(M); merge O(A log A) on the active set A.
With N = 4096, M ≈ 90, K = 7 this is a few ×10⁴ ops, run ~21.5×/s — trivial cost;
it runs comfortably in the DSP Web Worker.

## 7. Validation methodology

There is **no labelled real-choir corpus**, so the detector is measured against a
**synthetic ground-truth generator** (`choir-synth.ts`): additive synthesis of an
SATB chord as *audio*, with per-section chorus spread (N detuned singers),
shared-vibrato harmonics (FM, common-fate), a singer's-formant resonance, and a
−6 dB/oct-ish tilt. Crucially the synthetic **audio is run through the same real
`StreamingStft`** the live worker uses — so the detector is tested on the deployed
front end, not on hand-built spectra. Ground truth is the known MIDI set.

Metric: note-**set** precision / recall / F1 vs ground truth over a chord battery.
`salienceThreshold` was swept; F1 peaks at **τ = 4.0**:

| | F1 | precision | recall |
|---|---|---|---|
| pre-calibration (fft 2048, τ 1.6) | 0.18 | 0.11 | 0.65 |
| **calibrated (fft 8192 + merge, τ 4.0)** | **0.76** | **0.74** | **0.77** |

The canonical wide SATB triad (C3-G3-E4-C5) is detected **exactly**. Pinned in
`choir-detection.test.ts`, which runs the full deployed path (`SpectrogramDsp`).

## 8. Known limitations (physics, not bugs)

Corroborated by an independent vocal-acoustics literature review
(`docs/vocal-acoustics-research.md`):

- **Octave doublings under-reported** — Stage-4 sub-octave suppression cannot tell
  a genuine octave-doubled voice from an octave phantom (it has only one channel).
- **Semitone clusters collapse** — Stage-5 merge + the resolution limit.
- **Alto/Tenor overlap (~G3–A4)** — A/T confusion is near-chance even for
  state-of-the-art (≈56% RPA in the literature); a structural monaural limit.
- **Non-octave harmonic/subharmonic phantoms** — only the octave (N−12) is
  suppressed. A fifth/twelfth above a strong bass note (its 3rd harmonic), or a
  subharmonic below it, can still fire. **This is the largest remaining error
  source.** The principled fix is *iterative harmonic cancellation* (subtract the
  explained harmonics of the strongest F0, then re-estimate) — not yet built.
- **High soprano (> ~G5)** — fewer harmonics under Nyquist + nonlinear
  source-filter coupling weaken salience and any formant-based reasoning.

## 9. What is explicitly NOT in this algorithm

- **SATB section labeling** — designed (pitch-range priors + an explicit "A/T?"
  ambiguity class + a per-note singer's-formant tiebreaker) but **not implemented**.
- **Individual-singer fingerprinting** — the north-star. Infeasible from one mic
  for a unison section (information-theoretically); a constrained,
  non-unison/temporal (vibrato common-fate) approach is the only plausible path
  and is not started.
- **Any ML / learned model** — the pipeline is fully analytic DSP.

## 10. Lineage / related work (for reviewers to place it)

In the family of **harmonic / subharmonic summation pitch salience**:
- Hermes, *Measurement of pitch by subharmonic summation*, JASA 1988.
- Klapuri, multi-F0 estimation via spectral whitening + harmonic summation
  (e.g. ISMIR 2006; IEEE TASLP 2008).
- Related ideas: harmonic-sieve (Duifhuis et al.), two-way mismatch
  (Maher & Beauchamp).

The spectral whitening (Stage 1) and harmonic-sum salience (Stage 2) are standard;
the **per-note flat-baseline normalisation**, **sub-octave suppression**, and
**resolution-smear merge** are pragmatic engineering additions specific to this
detector. Reviewers familiar with the above literature should be able to assess
those three additions quickly.
