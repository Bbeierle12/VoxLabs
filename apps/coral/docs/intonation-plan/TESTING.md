# Coral — Testing & Validation Plan

This document lays out how we keep the spectrogram mathematically and perceptually honest, from "the FFT obeys its theorems" to "real choral audio comes out the way a trained musician would expect." Every guardrail below maps to a documented failure mode in the DSP or MIR literature; we're not inventing tests, we're encoding battle-tested ones.

## Why this matters

Errors compound through the pipeline. A 1% magnitude error in the STFT becomes a ±5 cent pitch error after F0 extraction, becomes a chord-quality misclassification, becomes a wrong section-fingerprint vector, becomes a research result that can't be reproduced. The math has to be right at the bottom or every layer above it lies. The MIR community settled on a **50-cent threshold for Raw Pitch Accuracy** because that's a quarter-tone — the smallest perceptible interval to untrained listeners. Professional tuners target **±10 cents**. We aim for the latter on synthetic test signals.

## The four-tier validation pyramid

We test in tiers, from cheapest and most certain (math identities you can prove on paper) to most expensive and most situated (real choral recordings). Lower tiers must pass before we trust the next tier up.

### Tier 1 — Math correctness

**What it proves.** The DSP code obeys the underlying theorems. If a Tier 1 test fails, the implementation is wrong, full stop.

**Status: required, runs on every commit, blocks merge.**

| Test | What's checked | Theoretical basis |
| --- | --- | --- |
| Parseval's energy conservation | Sum of squared time samples (windowed) ≈ sum of squared magnitudes / N | Parseval's theorem |
| DC bin | Constant input → all energy in bin 0 | DFT definition at k=0 |
| Nyquist bin | Alternating ±1 input → energy concentrates at bin N/2 | DFT at Nyquist |
| Linearity | FFT(a·x + b·y) = a·FFT(x) + b·FFT(y) (within float epsilon) | Linearity of the DFT |
| Conjugate symmetry | For real input, X[N−k] = conj(X[k]); we only keep N/2 bins | Real-input symmetry |
| Rayleigh resolution | Two sinusoids separated by ≥ 2·(Fs/N) resolve as two peaks; closer fails | Rayleigh criterion for windowed DFT |
| Hann window shape | w[0] = w[N−1] = 0, w[N/2] ≈ 1, w symmetric | Hann definition |
| Frame count | numFrames = ⌊(N_samples − fftSize) / hopSize⌋ + 1 | STFT framing |
| Peak bin localization | Sine at exact bin frequency → max bin = expected (±0) | DFT at integer frequencies |

### Tier 2 — Reference-signal characterization

**What it proves.** Known inputs produce known outputs *through the entire pipeline*, including windowing, framing, and dB conversion. Catches integration bugs that pure FFT tests miss (wrong hop accounting, wrong window normalization, wrong dB clamp).

**Status: required, runs on every commit.**

| Test | Reference signal | Expected outcome |
| --- | --- | --- |
| Mid-bin scalloping | Sine at (k + 0.5)·(Fs/N) | Energy splits ~equally between bins k and k+1; peak amplitude is ~1.42 dB below true (Hann scalloping loss) |
| Linear chirp track | 100→4000 Hz sweep over 1s | Peak-bin trajectory is monotonically increasing, slope matches sweep rate within 1 bin |
| Harmonic series | 220 Hz + 6 harmonics, equal amplitude | Peaks at 220, 440, 660 … 1540 Hz, each within ±1 bin |
| Impulse response | Single non-zero sample | Spectrum is nearly flat (within window's main-lobe shape) |
| White noise flatness | Gaussian white noise, σ=0.1 | Spectral flatness > 0.5 across the analysis band |
| Silence floor | All-zero buffer | All magnitudes = 0 exactly, dB floor renders correctly |

### Tier 3 — Frequency accuracy (deferred until F0 tracker lands)

**What it proves.** Pitch values derived from the spectrogram match ground truth to professional-tuner tolerance. Distinct from Tier 1/2 because it requires the QIFFT (quadratic interpolated FFT) or external pitch detector on top of the STFT.

**Status: defined now, implemented when `f0-tracker.ts` exists.**

| Test | Method | Acceptance |
| --- | --- | --- |
| QIFFT bias sweep | Parabolic peak interp on Hann-windowed sines from A0 to C8 in 1-cent increments | Mean error < 5 cents, max error < 15 cents (Hann bias is bounded ~1.5 cents in dB, but mid-bin signals approach the bound) |
| Cents accuracy | RMS cents error vs ground-truth frequency over the test sweep | RMS < 5 cents |
| Octave robustness | Same sweep, count gross errors (>1100 cents off) | 0 gross errors on synthetic signals |
| Vibrato fidelity | 5 Hz sinusoidal F0 modulation, depth ±50 cents | Detected modulation rate within ±0.5 Hz, depth within ±10 cents |

References for these metrics: Smith, *Spectral Audio Signal Processing* §5.7 (Quadratically Interpolated FFT); the MIR pitch-evaluation standard formalized in CREPE/SPICE benchmarks (Raw Pitch Accuracy at 50¢, Cents Accuracy with exponential penalty, Octave Accuracy). Parabolic interpolation is exact on a Gaussian window; on Hann it's biased but bounded — usable as long as we know the bound.

### Tier 4 — Real-data validation (research phase)

**What it proves.** The spectrogram behaves as expected on actual choral recordings against published ground-truth annotations.

**Status: planned, executed during the research phase outlined in `RESEARCH.md`.**

Public datasets to validate against, in priority order:

1. **Dagstuhl ChoirSet (DCS)** — Rosenzweig et al. 2020. 13 singers, SATB, multi-mic close + room, F0 + beat + score annotations, ~55 min. Closest fit to our multi-track v1 design.
2. **Choral Singing Dataset (CSD)** — Cuesta et al. 2018. 16 singers, three SATB pieces (Locus Iste, Niño Dios, El Rossinyol), MIDI + per-section F0 + note annotations. Sections recorded separately, so good for per-channel tests.
3. **ESMUC Choir Dataset (ECD)** — manually corrected F0 contours and notes. Smaller, useful as a held-out test set.
4. **ChoralSynth** — synthesized SATB, perfect ground truth, no acoustic realism. Use for upper-bound sanity checks.
5. **Annotated-VocalSet** — solo, useful for individual-voice pretraining and per-singer pipeline tests.

Acceptance bar at Tier 4:

- Per-channel F0 RPA ≥ 90% on DCS close-mic tracks (matches off-the-shelf CREPE / pYIN on similar material)
- Spectrogram visual review by two trained musicians (you + one other) on at least 10 minutes of held-out audio with no obvious artifacts (inverted axes, wrong dB clamp, time-reversal, aliasing)

## Visual debug protocol

Unit tests catch mathematical bugs; the eye catches *category* bugs that pass numerical tests but render wrong. Run this every time the render pipeline changes.

The seven canonical signals to render and eyeball:

1. **A 440 Hz sine + 5 harmonics, 2 seconds.** Should look like 6 horizontal lines, equally spaced, lowest at ~A4 (mid-height on the log axis), brightest at the fundamental.
2. **A 100→4000 Hz linear chirp, 4 seconds.** Should look like a single diagonal line sweeping bottom-left to top-right. On a *log-frequency* axis, the line should be **concave** (slope decreases as freq increases), not straight — that's correct, not a bug.
3. **A Dirac impulse at t = 1 s in 2 seconds of silence.** Should look like a single bright vertical line at the impulse time, flat across all frequencies.
4. **White noise for 2 seconds.** Should look like a uniform gray/green wash with no banding or visible structure.
5. **Vibrato: a 440 Hz sine modulated by 6 Hz with ±30 cents depth.** Should look like a clean horizontal line with a visible sinusoidal wobble.
6. **SATB C-major chord (C3 / G3 / C4 / E4) with 5 harmonics per voice, 2 seconds.** The choral eyeball. You should be able to count four distinct horizontal lines around the lower-middle of the canvas (the fundamentals, each landing right on a NoteGrid marker), with harmonic stacks of fainter lines marching upward above each one. C3 and G3 share no obvious harmonics, but C3's 2nd harmonic and C4 fall on the same line (262 Hz) — that's musical reality, not a bug.
7. **Silence.** Solid dark/colormap-floor wash. Tests the dB clamp at the noise threshold.

If any of these renders looks wrong, suspect (in order):

1. **Time-reversal** — frame 0 is rendered on the right instead of the left
2. **Axis inversion** — high frequencies at the bottom instead of the top
3. **Log/linear mismatch** — chirp looks straight on log axis or curved on linear
4. **dB clamp wrong** — image is fully saturated white or fully black
5. **Colormap inverted** — silence is bright, peaks are dark
6. **Window leakage too strong** — single sine smears across many bins (suggests wrong window or no window)
7. **Off-by-one in framing** — chirp endpoint doesn't reach the expected frequency

## What to do when a test fails

A red Tier 1 test is a math bug. The order of suspicion:

1. **Input validation** — is the test feeding what we think it is? Print `samples.length`, `sampleRate`, `samples[0..3]`.
2. **Window correctness** — is the Hann window symmetric and zero at endpoints?
3. **FFT library** — does the library use a different normalization convention than we assume? `fft.js` does *not* normalize on the forward transform; we account for that in Parseval's tests.
4. **Bin indexing** — are we reading complex output as `[2k, 2k+1]` (interleaved) or `[k, N+k]` (split)? `fft.js` is interleaved.

A red Tier 2 test usually means an STFT integration bug:

1. **Hop accounting** — is the frame index correctly multiplied by `hopSize`, not `fftSize`?
2. **Frame boundary** — does the last frame include or exclude a partial window?
3. **dB conversion** — `20·log10(mag + ε)` vs `10·log10(mag² + ε)` (the latter is power, the former is amplitude; using one where you meant the other gives a 2× dB error)

A red Tier 3 test means the math is right but the *estimator* is biased. Either pick a better estimator (QIFFT instead of nearest bin, CREPE instead of YIN) or widen the FFT window (more bins, less time resolution).

A red Tier 4 test means real-world acoustics are exposing assumptions we baked in. That's not a bug — that's research.

## Metrics, defined

We use the MIR-community-standard pitch evaluation metrics. They become live once the F0 tracker exists; until then we report only the bin-localization error from Tier 1.

**Raw Pitch Accuracy (RPA).** Fraction of voiced frames whose detected F0 lies within 50 cents (a quarter-tone) of ground truth. The standard "is the pitch right" headline number.

**Cents Accuracy (CA).** A continuous metric: `exp(-|cents_error| / 500)` averaged over voiced frames. Penalizes large errors exponentially, doesn't have RPA's hard threshold.

**Octave Accuracy (OA).** Fraction of voiced frames where the detected F0 is within 50 cents *modulo octave*. Distinguishes "right pitch, wrong octave" from "wrong pitch entirely."

**Gross Error Rate.** Fraction of voiced frames with |cents_error| > 1200. Catches catastrophic detector failures.

**Voicing Precision / Recall.** Detector said voiced when it was / detector caught all voiced frames. Distinguishes detection of a pitch from estimation of its value.

**Harmonic Mean of the above.** Single number for benchmark tables. Forces the detector to be balanced — failing any one component drags the HM down.

These come from the CREPE evaluation methodology (Kim et al. 2018) and the SwiftF0 / pitch-benchmark consolidation (2025).

## Spectral feature sanity descriptors

Optional Tier 2 additions, useful for "is the signal what I think it is" checks rather than correctness:

- **Spectral centroid** — frequency-weighted mean. White noise sits near Fs/4; a pure sine sits at its frequency.
- **Spectral flatness** — geometric mean over arithmetic mean. White noise ≈ 1, pure tone ≈ 0. Excellent voiced/unvoiced gate.
- **Spectral flux** — frame-to-frame magnitude change. Onsets spike it.

We aren't testing these yet — they're descriptors of the audio, not validators of our pipeline — but they go in the feature-extraction module when Tier 3 lands.

## Choral-specific parameter rationale

The default analysis parameters in `src/config/fft.ts` and `src/config/render.ts` are tuned for choral content. If you ever change them, the test thresholds in this plan may need re-derivation.

| Parameter | Value | Reason |
| --- | --- | --- |
| `fftSize` | 4096 | At 44.1 kHz this gives 10.8 Hz bin width — 21 cents/bin at A4, 42 cents/bin at A3. Generic-audio tools default to 1024–2048 and become useless for low-bass intonation work. Trade: 93 ms window smears fast attacks (fine for sustained choral notes, bad for percussion). |
| `hopSize` | 1024 | 75% overlap. Standard musical-analysis default; 23 ms per frame for time visualization. |
| `minFreqHz` | 65 | C2, below the lowest reasonable bass fundamental. Drops mains hum (50/60 Hz), HVAC rumble, mic handling noise. |
| `maxFreqHz` | 8000 | Covers singer's-formant cluster (2.5–3.5 kHz), high-vowel second formants, and most sibilant energy. |
| `minDb` | −100 | Below this is preamp noise floor, not music. |
| `maxDb` | −20 | Tracks loud choral peaks; lower for field recordings, raise for mastered material. |
| Pitch markers | C octaves + A4 | Every octave anchor plus the tuning reference. More markers becomes visual noise; this is the Sonic Visualiser / Praat default. |

If you switch to a domain other than choral (instrumental, speech, bioacoustic, percussion), the parameters above are wrong by design. Re-tune deliberately and re-derive the Tier 2 thresholds.

## Implementation status

What ships in the same commit as this document:

- **`src/audio/test-signals.ts`** — synthesis of sine, harmonic series, chirp, impulse, white noise, silence, and vibrato signals at controllable parameters.
- **Tier 1 unit tests** for Parseval, DC bin, Nyquist bin, linearity, real-input symmetry, Rayleigh resolution, frame count, peak bin.
- **Tier 1 window tests** for Hann symmetry, endpoint zeros, and midpoint value.
- **Tier 2 unit tests** for harmonic series, chirp tracking, scalloping at half-bin, impulse flatness, and silence floor.
- **A debug switch in the App UI** that renders any reference signal through the same spectrogram pipeline, so Tier 1/2 results are also eyeball-verifiable.

What's deferred:

- Tier 3 (parabolic interpolation, cents accuracy) — lands with `f0-tracker.ts`.
- Tier 4 (Dagstuhl, CSD, ECD validation) — research phase, after multi-channel support.
- Two-musician visual review — not blocking v0; informal eyeballing for now.

## Acceptance bar for the walking skeleton

Before this gets a tag and a Vercel deploy: **all Tier 1 and Tier 2 tests pass, all five visual debug signals render correctly to your eye, and `npm test` is green in CI.** That's the bar.

## References

- Smith, J. O. *Spectral Audio Signal Processing.* W3K Publishing / CCRMA. Online: `ccrma.stanford.edu/~jos/sasp/`. Authoritative on QIFFT and window analysis.
- Harris, F. J. (1978). "On the use of windows for harmonic analysis with the discrete Fourier transform." *Proc. IEEE*, 66(1), 51–83. The window function-of-merit reference.
- Rosenzweig, S., Cuesta, H., Weiß, C., Scherbaum, F., Gómez, E., & Müller, M. (2020). "Dagstuhl ChoirSet: A multitrack dataset for MIR research on choral singing." *TISMIR*, 3(1), 98–110.
- Cuesta, H., Gómez, E., Martorell, A., & Loáiciga, F. (2018). "Analysis of intonation in unison choir singing." *ICMPC*.
- Kim, J. W., Salamon, J., Li, P., & Bello, J. P. (2018). "CREPE: A convolutional representation for pitch estimation." *ICASSP*.
- Nieradzik, L. (2025). "SwiftF0: Fast and accurate monophonic pitch detection." arXiv:2508.18440. Source of the unified harmonic-mean pitch metric.
