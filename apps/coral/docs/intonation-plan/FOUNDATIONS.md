# Coral — Theoretical Foundations

A research synthesis covering the three foundations the project is built on: the mathematics of the spectrogram itself, the architecture by which we test code that implements that mathematics, and the centuries of musical and acoustical theory that determine what "right" looks like for a choral spectrogram in particular.

This document is research-grade and explicitly labeled as a learning artifact, not a build plan. Its sibling documents are `README.md` (what the project is), `TESTING.md` (how we validate), and — once they exist — `RESEARCH.md` (the sonic-fingerprint research roadmap) and `FUTURE.md` (deferred features). When the literature and this document disagree, the literature wins; flag the discrepancy and update.

---

## Part I — Mathematical Dependencies of the Spectrogram

The spectrogram is not a single invention; it is the composition of roughly eight ideas, each of which was a research program in its own time. To know what we are computing, and what could go wrong, we need to be fluent in the dependencies.

### I.1 Fourier's theorem (1822)

Joseph Fourier published *Théorie analytique de la chaleur* in 1822, in which he argued that any "reasonable" periodic function can be expressed as a sum of sines and cosines at integer multiples of a fundamental frequency. The technical conditions under which this is true (Dirichlet conditions, square-integrability, etc.) were filled in over the following century. For our purposes, the practical content is this: a steady-state musical tone — a sustained note from a singer, an instrument, a synthesizer — can be perfectly described by its fundamental frequency plus an amplitude and phase for each harmonic.

The transform that maps from time-domain signal to frequency-domain coefficients is the *Fourier transform*. Its discrete-time, finite-length counterpart, which is what computers actually compute, is the *discrete Fourier transform* (DFT). For a length-N real-valued signal x[n], the DFT is defined as:

$$X[k] = \sum_{n=0}^{N-1} x[n] \cdot e^{-i 2\pi k n / N}$$

This produces N complex coefficients, but by the conjugate-symmetry property of the DFT of a real-valued signal, X[N−k] = conj(X[k]), so only N/2 of them carry independent information. Those N/2 coefficients are our spectrogram's vertical axis.

### I.2 The Fast Fourier Transform (Cooley–Tukey, 1965)

A literal evaluation of the DFT costs O(N²) operations — for N = 4096 that is roughly sixteen million complex multiplies *per frame*. The Cooley–Tukey FFT algorithm, published in 1965 but partially anticipated by Gauss in 1805, reduces this to O(N log N) by recursive decomposition. For N = 4096 that is roughly 50,000 operations per frame. The factor of ~300 is the entire reason real-time spectrograms are computationally feasible.

`fft.js`, the library Coral uses, implements a radix-2 Cooley–Tukey decomposition with a real-input optimization (one length-N real FFT decomposes into a length-N/2 complex FFT plus a butterfly pass).

### I.3 The sampling theorem (Nyquist 1928, Shannon 1949)

The DFT operates on discrete samples of a continuous signal. The sampling theorem states that a continuous signal whose highest frequency component is at most B Hz can be perfectly reconstructed from samples taken at a rate of at least 2B Hz. The threshold 2B is the *Nyquist rate*. For audio, where the conventional upper bound of human hearing is taken at 20 kHz, the standard sample rate of 44.1 kHz (CD audio) and 48 kHz (digital video and broadcast) sit just above twice that bound.

The consequence for the spectrogram: at sample rate Fs, the highest representable frequency is Fs/2, called the *Nyquist frequency*. Energy at frequencies above Fs/2 in the original signal gets *aliased* — mirrored back down into the representable band — and contaminates the spectrogram. Anti-aliasing filters in the ADC suppress this before sampling; we cannot fix it after the fact.

Our spectrogram exposes Fs/2 as the top of its frequency axis. For 44.1 kHz audio that is 22.05 kHz; for 48 kHz it is 24 kHz. The choral analysis range we actually use ends at 8 kHz, far below either Nyquist.

### I.4 Time-frequency uncertainty (Heisenberg–Gabor)

Dennis Gabor's 1946 paper "Theory of Communication" proved an audio analogue of the Heisenberg uncertainty principle: the time resolution Δt and frequency resolution Δf of any joint time-frequency representation are bounded by Δt · Δf ≥ 1/(4π). You cannot get arbitrarily precise estimates of when something happened *and* what frequency it happened at; sharpening one blurs the other.

For an STFT with window length N samples at sample rate Fs:

- Time resolution is set by the window length: Δt ≈ N/Fs.
- Frequency resolution is set inversely: Δf ≈ Fs/N.

At our default of N = 4096, Fs = 44.1 kHz, we get Δt ≈ 93 ms, Δf ≈ 10.8 Hz. Sustained choral notes are typically 200 ms to several seconds long, so 93 ms of temporal blur is acceptable. Fast attacks (consonants, percussion) get smeared. We accept this trade because choral content is sustained-dominant.

The Constant-Q Transform (CQT) — discussed in Part VII below — works around this by varying window length with frequency, but it is not a free lunch; it pays in implementation complexity.

### I.5 Window functions

The DFT implicitly assumes the input is one period of an infinitely repeating signal. If your length-N input does not happen to contain an integer number of cycles of every component frequency, the implicit wraparound creates discontinuities at the frame boundary, and those discontinuities show up in the spectrum as *leakage* — energy that should belong to one bin spread across many.

A window function multiplies the input by a smooth taper that goes to zero at the endpoints, killing the discontinuity at the cost of slight smearing of the mainlobe. Harris's 1978 paper *On the use of windows for harmonic analysis with the discrete Fourier transform* is the canonical reference for choosing among windows; it tabulates leakage characteristics for dozens of options.

Coral uses the Hann window. It has:

- Mainlobe width ≈ 4 bins between first zeros — broader than rectangular (2 bins) but much narrower than Blackman-Harris (8 bins).
- Highest sidelobe ≈ −32 dB.
- Sidelobe rolloff of 18 dB/octave — modest, but adequate for music.
- Maximum scalloping loss ≈ −1.42 dB at the half-bin frequency.

Hann is the standard musical-analysis choice because the trade between mainlobe sharpness and sidelobe rejection is well-matched to music's typical dynamic range. For very-high-dynamic-range work (close tones at very different amplitudes), Kaiser or Blackman-Harris would be preferred; for very-high-resolution work, Gaussian. Hann is the right default and we own that choice.

### I.6 Short-Time Fourier Transform (STFT)

The STFT is what you get when you apply the DFT to successive overlapping windowed frames of a longer signal. Formally, for a signal x[n], a window w[n] of length N, and a hop size H:

$$X[m, k] = \sum_{n=0}^{N-1} x[mH + n] \cdot w[n] \cdot e^{-i 2\pi k n / N}$$

This produces a 2D complex-valued array indexed by frame number m and bin number k. The spectrogram is |X[m, k]|² (power) or |X[m, k]| (magnitude), optionally converted to decibels.

Two STFT parameters not always discussed in introductory references:

**Constant Overlap-Add (COLA)** — the property that for some windows and hop sizes, the sum of overlapping windows is a constant (or near-constant) across time. COLA matters for perfect reconstruction in the inverse STFT, not so much for analysis-only work like ours. Hann with 50% overlap satisfies COLA; with 75% (our setting) it satisfies the looser NOLA (Nonzero Overlap-Add) condition. SciPy exposes a `check_NOLA` function for verification.

**Frame centering** — different libraries adopt different conventions for where the first frame is anchored: starting at sample 0, or centered at sample 0 (which requires padding the front of the signal). Librosa centers by default; SciPy does not by default. Coral does not center. This matters when comparing outputs across libraries.

### I.7 Magnitude, phase, power, and decibels

The complex output X[m, k] of the STFT carries both amplitude (|X|) and phase (arg X). The spectrogram conventionally discards phase, keeping only:

- **Magnitude**: |X[m, k]|, units proportional to amplitude.
- **Power**: |X[m, k]|², units proportional to energy.

For display, both are converted to decibels:

- **Amplitude → dB**: 20 · log₁₀(|X| / ref)
- **Power → dB**: 10 · log₁₀(|X|² / ref)

The two are equivalent because 20·log₁₀(|X|) = 10·log₁₀(|X|²). Confusing them produces a factor-of-2 dB error.

Phase is not useless. It carries onset information (the relative timing of components) and is essential to inverse STFT. The "phase vocoder" family of algorithms uses phase differences across frames to refine frequency estimates beyond the bin resolution; this is the foundation of the *phase-derived* pitch estimators used in some F0 trackers.

### I.8 Logarithmic frequency, logarithmic perception

Human pitch perception is approximately logarithmic. Doubling the frequency raises the perceived pitch by an octave, regardless of the starting frequency: A2 → A3 sounds the same "distance" as A3 → A4 even though the absolute Hz differences are 110 vs 220. This is built into the structure of every Western musical scale — twelve equal logarithmic divisions of the octave.

A linear-frequency spectrogram is therefore the wrong display for music. The bass region (50–500 Hz) — which contains every choral fundamental and most of the harmonically interesting content — occupies 2% of the linear range from 0 to 22 kHz. On a log axis, it occupies roughly a third of the height. Coral uses log-frequency on the display axis but a *linear* frequency layout for the underlying STFT bins; we sample the linear-bin data onto the log display.

This is good enough for visualization, but it does mean we lose musical resolution at low frequencies where linear-bin spacing maps to large cent intervals (at 65 Hz, one bin = 285 cents). The CQT addresses this at the analysis stage rather than the display stage.

### I.9 Decibel ranges and perceptual scaling

The dB clamp in our renderer maps a continuous range of magnitudes onto the colormap's discrete output. Wider clamp = more dynamic range visible, less contrast per dB; narrower clamp = punchier display, but quiet detail clips to the floor.

Our defaults of −100 to −20 dB are tuned for choral material. Tighter ranges are used for mastered audio, wider ranges for field recordings with high noise floors. The clamp interacts with perception: a 6 dB difference is "twice as loud" psychoacoustically, so the visual contrast between a −60 dB and a −66 dB region should arguably be perceptually equivalent to that between −20 dB and −26 dB. Our linear dB-to-colormap mapping does *not* enforce that; it is a uniform color-per-dB. Perceptually uniform colormaps (viridis, magma) compensate slightly, but the deeper fix would be a power-law or compression curve before the colormap.

---

## Part II — Implementation Dependencies

A spectrogram is a stack of software, not just math. Each layer has its own conventions and failure modes.

### II.1 The signal acquisition layer

Coral receives audio via the Web Audio API's `decodeAudioData`, which accepts compressed audio in any codec the host browser supports. The decoder normalizes:

- **Sample format** — from int16 / int24 / int32 / float32 source to float32 internal.
- **Sample rate** — preserved; we read `buffer.sampleRate` and respect whatever the file is.
- **Channel count** — preserved; we downmix to mono for v0 by averaging.

Failure modes: codec not supported (older browsers, exotic formats), DRM-locked content, partial-file decode on truncation. All surface as `decodeAudioData` rejecting, which we wrap into our own error.

The browser's audio resampling, if any happens, is implementation-defined. Chrome and Firefox use high-quality polyphase resamplers; results are typically indistinguishable from the original. Safari has historically been less consistent.

### II.2 The FFT layer

`fft.js` is the FFT library used. Alternatives we considered:

- **fftw.js** — JavaScript port of FFTW, the gold-standard C library. Larger bundle, no advantage at our FFT sizes.
- **kissfft (via WASM)** — small, clean, fast. Adds WASM toolchain complexity.
- **Hand-rolled radix-2** — educational but no benefit over a battle-tested library.
- **Web Audio's AnalyserNode** — built-in but only works for real-time playback, not offline analysis of a full file.

The choice criterion: `fft.js` is small (~7 KB), TypeScript-friendly with shipped types as of v4, zero dependencies, and well-tested. Performance is adequate for our use case (~30 µs per 4096-point real FFT on a modern laptop).

Numerical precision: `fft.js` operates in float32 (Float32Array). For our purposes this is fine — the dynamic range of a single FFT is roughly 24-bit, well within float32's mantissa. For very-high-precision research work, float64 is preferred but rarely necessary.

### II.3 Reference implementations as oracles

The single most useful test we will ever write is one that compares our output to that of a battle-tested reference. The references that matter for audio spectrograms:

- **librosa** (Python) — academic and educational standard, used by most MIR research. McFee et al. 2015.
- **scipy.signal** (Python) — general DSP reference, used as the substrate for many higher-level libraries.
- **Essentia** (C++ with Python bindings) — Universitat Pompeu Fabra, performance-oriented MIR.
- **Sonic Visualiser / Sonic Annotator** (UK, QMUL) — interactive and batch visualization, ground-truth annotation host.
- **MATLAB's Signal Processing Toolbox** — commercial gold standard, used in engineering.
- **MIR Toolbox / mir_eval** — evaluation metrics consolidation.

For Coral, librosa is the reference of choice. It is open-source, widely cited, and its STFT conventions are documented in detail. Conformance testing protocol: feed an identical input through librosa's `librosa.stft()` (with matched parameters — `n_fft=4096`, `hop_length=1024`, `window='hann'`, `center=False`) and compare against our output. Tolerable difference: under ~0.1 dB per bin, in the bulk; under 1 dB anywhere. We do not have this test wired in yet; it lives in the deferred Tier 4 of TESTING.md.

### II.4 The render layer

Canvas 2D is what we use; alternatives:

- **WebGL2** — necessary once we draw multi-channel data at scrub-rate; overkill for v0.
- **WebGPU** — successor to WebGL, more parallelizable. Bleeding-edge browser support.
- **SVG** — too slow for 1200 × 480 pixel data.
- **Off-the-shelf**: Plotly, D3 heatmap, Vega — all add bundle weight without buying anything we need for a custom musical display.

The render layer is also where colormaps live. The viridis family (viridis, plasma, magma, inferno) is the modern standard for scientific data: perceptually uniform, colorblind-friendly, looks reasonable on both light and dark backgrounds. Jet and rainbow colormaps, common in older spectrogram software, are known to introduce artificial perceptual structure where no data structure exists; we explicitly do not use them.

---

## Part III — Test Architecture for DSP Code

DSP code has a peculiar property: it can produce *plausible-looking* output that is mathematically wrong. A spectrogram with a sign error in the imaginary component still renders as a colorful heatmap. A pitch tracker with a bug that always returns f0/2 still tracks pitch *somewhere* — just an octave low. Traditional unit testing finds bugs that crash or return obviously wrong values; DSP bugs are subtler. The test architecture has to compensate.

### III.1 The test pyramid is the wrong shape for DSP

The classical software test pyramid — many unit tests, fewer integration tests, even fewer end-to-end — is built around the assumption that unit tests are cheap and meaningful for individual functions. For DSP code, the unit "is this function correct in isolation" is often impossible to answer in isolation; correctness is defined by mathematical relationships that span multiple functions.

For Coral, the right shape is a *layered correctness pyramid*:

1. **Math identities** (Tier 1 in TESTING.md) — properties that must hold by theorem
2. **Reference signal characterization** (Tier 2) — known inputs producing known outputs end-to-end
3. **Quantitative metric** (Tier 3, deferred) — domain-specific accuracy bars (cents error)
4. **Real-data validation** (Tier 4, deferred) — published datasets with annotated ground truth
5. **Visual / human-in-the-loop** (the debug panel, plus eyeball protocol) — what the unit tests can't catch

Each tier has different cost, different velocity, and finds different bugs. The first two run on every commit; the last three are progressively more expensive and run on milestones rather than per-commit.

### III.2 Property-based testing

Property-based testing (Hypothesis in Python, fast-check in TypeScript, QuickCheck in Haskell) generates random inputs and checks that the function satisfies stated invariants for all of them. For DSP this is enormously powerful because most DSP correctness can be stated as invariants:

- **Linearity**: FFT(a·x + b·y) ≈ a·FFT(x) + b·FFT(y) for any a, b, x, y.
- **Conjugate symmetry**: For real-valued input, X[N−k] = conj(X[k]) for all k.
- **Parseval's theorem**: ∑|x[n]|² = (1/N) · ∑|X[k]|², after window normalization.
- **Linearity of magnitude under unitary transforms**: scaling input by α scales output magnitude by |α|.
- **Time-shift property**: shifting x[n] by m samples multiplies X[k] by e^(−i2πkm/N) — magnitude is unchanged.
- **Time-reversal property**: x[−n] in time domain gives conj(X) in frequency domain.
- **Frame-count formula**: ⌊(L − N) / H⌋ + 1 frames for any signal length L ≥ N.

A property-based test generator can feed in randomized lengths, randomized amplitudes, randomized seeds, and verify each invariant holds across thousands of cases. The framework also *shrinks* failing inputs: when it finds a failure, it automatically searches for the smallest input that still triggers it, producing a minimal counterexample.

Recent academic work (Maaz et al. 2025) has applied LLM-driven agents to property-based testing across the Python ecosystem; Hypothesis remains the dominant Python framework. For our TypeScript stack, `fast-check` is the standard equivalent.

Coral does not currently use property-based testing — our Tier 1 tests are example-based, with specific bin numbers and specific assertion targets. That is fine for now but the upgrade path is clear: each property in our current `describe` block can become a `fc.assert(fc.property(...))` test with randomized input. Estimated effort to retrofit: half a day. Estimated bug-finding value: high; the documented industry experience is that property-based tests routinely find edge cases that hand-written tests miss.

### III.3 Oracle testing

When you have a reference implementation, you don't need to derive the expected output by hand — you ask the reference. This is *oracle testing* and it is the right way to validate any non-trivial DSP function.

The protocol:

1. Pick a reference (librosa for us).
2. Generate a corpus of test inputs covering the parameter space: short signals, long signals, multi-channel, edge cases like silence and full-scale.
3. Run both implementations on each input.
4. Compare outputs with a stated numerical tolerance.

The right tolerance is implementation-defined. For STFT output in float32, atol of ~1e-5 in magnitude or ~0.1 dB in log-magnitude is reasonable. NVIDIA's DALI documentation, validating their own STFT against librosa, uses `np.allclose(spectrogram_dali_db, spectrogram_librosa_db, atol=2)` — 2 dB tolerance — which is generous but defensible for production audio work.

Coral's Tier 4 should include librosa-conformance tests. We do not have them yet; setting one up requires a Python sidecar for generating reference outputs, then storing them as test fixtures.

### III.4 Differential testing

A specialization of oracle testing where the "oracle" is a different implementation of the same algorithm. Run input through both, compare. Useful for catching bugs that aren't obvious from spec compliance — both implementations may agree with the spec while disagreeing with each other on edge cases the spec doesn't pin down.

For STFT, differential testing across librosa, scipy.signal.stft, MATLAB, and our implementation will surface any framing/centering/padding inconsistencies. Each library's conventions differ subtly.

### III.5 Golden master / regression testing

When a function produces output that is hard to validate analytically but easy to validate visually, store a known-good output and assert that future runs match it. For spectrograms this is straightforward:

1. Run a reference signal through the pipeline.
2. Hash or pixel-compare the output image.
3. Store it as a fixture.
4. On every CI run, regenerate and compare.

If the comparison fails, *either* the code changed (the test caught a regression) *or* the expected output is wrong (you need to update the golden master after manual review). The golden-master pattern is best for catching unintentional changes; it is bad at catching deliberate-but-incorrect changes that you reviewed and approved without noticing the bug.

Coral does not have golden masters yet. The candidate fixtures are the seven reference signals from the debug panel — A440 harmonics, chirp, impulse, white noise, vibrato, SATB chord, silence — each rendered to a known canvas image.

### III.6 Visual regression testing

The render layer specifically benefits from screenshot diffing. Tools like `playwright`'s `toMatchSnapshot()`, Percy, Chromatic, and Loki capture rendered pixels and diff them across runs. For Coral this would catch the entire class of bugs where the math is right but the colormap, axis labels, or layout regressed.

Cost: setting it up requires a headless browser harness in CI. Benefit: catches every visual bug, deterministically.

### III.7 Numerical precision and tolerance

Every DSP test needs a tolerance. The wrong choice in either direction is harmful: too tight and tests fail intermittently from float roundoff; too loose and real bugs slip through.

Guidelines for choosing tolerances:

- **Float32 multiply/add**: ~1e-7 relative error per operation. After N operations, expect ~N · 1e-7. For FFT, log₂(N) operations per output bin, so ~1e-6 for N=2048 and ~1e-5 cumulative is realistic.
- **Float64**: ~1e-16 per operation. For most musical work, 1e-10 absolute tolerance is conservative.
- **dB-domain comparisons**: 0.01 dB is high-precision, 0.1 dB is typical, 1 dB is loose, 6 dB is "in the same neighborhood".
- **Cents-domain comparisons**: 1 cent is the just-noticeable-difference for trained musicians under ideal conditions; 5 cents is professional-tuner-grade; 10 cents is "feels in tune to most listeners"; 50 cents is the RPA threshold (a quarter-tone, halfway between adjacent semitones).

Tighter tolerances are not always more rigorous. A tolerance tighter than the float roundoff floor produces tests that fail randomly based on operation ordering, which trains the team to ignore failures.

### III.8 The Coral tier mapping

Coral's current test architecture, mapped to the categories above:

| Layer | Strategy | Status |
|---|---|---|
| FFT math | Example-based unit tests; **upgrade target: property-based** | shipped (Tier 1) |
| STFT integration | Example-based reference signal tests | shipped (Tier 2) |
| Notes / pitch utilities | Example-based unit tests | shipped |
| Render pipeline | Manual visual debug; **upgrade target: golden master + screenshot diff** | partial (debug panel) |
| Library conformance | **Upgrade target: librosa oracle tests via Python sidecar** | not started |
| F0 accuracy | **Upgrade target: cents-error tests once F0 tracker lands** | deferred (Tier 3) |
| Real-data validation | **Upgrade target: Dagstuhl ChoirSet / CSD evaluation** | deferred (Tier 4) |

The deferred items are not deferred because they're unimportant; they're deferred because they require infrastructure (Python sidecar, dataset download, F0 tracker implementation) that has not been built. Each is a concrete, scoped follow-up.

---

## Part IV — Five Centuries of Musical Mathematics

The mathematics of music predates the mathematics of digital signal processing by roughly two and a half millennia. Pythagoras was solving Coral's problems in 530 BCE; the only difference is the form of the answer.

### IV.1 Pythagoras and the harmonic series (~530 BCE)

The Pythagorean school discovered that strings whose lengths are in simple integer ratios produce consonant intervals: 2:1 (octave), 3:2 (perfect fifth), 4:3 (perfect fourth), 5:4 (major third in some readings, though this is debated for the Pythagorean tradition specifically). The same ratios appear in the *harmonic series* — the sequence of integer multiples f, 2f, 3f, 4f, 5f, ... that arise naturally when a string or air column vibrates: the 2nd harmonic is one octave above the fundamental, the 3rd is an octave plus a fifth, the 4th is two octaves, and so on.

This is not just historical trivia. The harmonic series is *what we see on a spectrogram of a singing voice*. The fundamental frequency is the lowest peak; equally spaced peaks above it (on a linear axis) or logarithmically-decreasing-spacing peaks (on a log axis) are the harmonics. The Coral SATB-chord debug signal is constructed exactly this way: four fundamentals at SATB pitches, each with five integer-multiple harmonics, summed.

### IV.2 Pythagorean tuning

Building a scale from stacked perfect fifths (ratio 3:2) generates *Pythagorean tuning*. Twelve fifths take you up roughly seven octaves: (3/2)¹² ≈ 129.7. Seven octaves is exactly 2⁷ = 128. The difference, (3/2)¹² / 2⁷ ≈ 1.0136, is the *Pythagorean comma* — roughly 23.5 cents, almost a quarter-tone.

The comma is not a rounding error; it is a structural feature of arithmetic. There is no way to subdivide the octave into twelve equal steps while also having every fifth be a true 3:2. You either have eleven pure fifths and one mistuned "wolf" fifth, or you compromise on all of them. This is the central tension of all Western tuning theory.

Pythagorean tuning was widespread in medieval European music, especially before about 1300. Its melodic character is bright and bracing; its harmonic character — the major third comes out as the "ditone" 81:64, about 22 cents sharper than a pure 5:4 major third — was acceptable when thirds were treated as dissonances. When thirds began to be sung as consonances in the Renaissance, the Pythagorean third became intolerable.

### IV.3 Just intonation (Ptolemy ~150 AD, widespread ~1300–1550)

Ptolemy described what we now call *just intonation* in *Harmonics* (~150 AD): a scale built on simple integer ratios for *all* the consonant intervals, including thirds. The just major third is 5:4 (~386 cents), the just minor third is 6:5 (~316 cents), the just fifth is 3:2 (~702 cents).

Just intonation produces beautifully consonant chords — a major triad in just intonation has no audible beating between its harmonics. But it doesn't survive transposition. The ratios that produce pure consonance in C major do not produce pure consonance in F# major; you would need a different physical tuning of every instrument for every key. For unaccompanied singers, this is no problem — they retune on the fly. For fixed-pitch instruments (keyboards, fretted strings), it is a deal-breaker.

The *syntonic comma* — about 21.5 cents — quantifies the irreconcilability: it is the difference between a Pythagorean major third (81:64) and a just major third (5:4 = 80:64). Twelve just fifths exceed seven octaves by both the Pythagorean comma *and* the syntonic comma stacked.

### IV.4 Meantone temperament (~1550)

The Renaissance solution: temper the fifth slightly so that four stacked fifths give a pure major third. Specifically, quarter-comma meantone narrows each fifth by 1/4 of a syntonic comma, making the major third 5:4 exactly while the fifth becomes about 696.6 cents — perceptibly flat from a pure 3:2 (702 cents) but acceptable.

The cost: meantone has a wolf fifth (G#–Eb in its standard layout) and modulations to distant keys break down. It works beautifully in keys near the home key, terribly far from it.

### IV.5 Well temperament (Werckmeister 1691, Kirnberger 1779, others)

A family of irregular temperaments designed to make *all* keys playable, with each key having its own slightly different character. Werckmeister III, published in 1691 by Andreas Werckmeister, was particularly influential. The Bach *Well-Tempered Clavier* (1722, 1742) is named for this family — though which specific well-tempered tuning Bach intended is still debated by historical performance scholars.

Well temperaments preserve the *idea* of each key having a distinct affect (the bright, sharp keys; the dark, flat keys) while making all of them functional. Equal temperament destroys this; every key sounds identical except for absolute pitch.

### IV.6 Equal temperament (widely adopted late 18th and 19th centuries)

The mathematical solution: divide the octave into twelve geometrically equal steps. Each semitone is the 12th root of 2: 2^(1/12) ≈ 1.0594631. This is *equal temperament*, also called *12-TET*.

In equal temperament:

- Every fifth is 700 cents (2 cents narrow of pure 3:2)
- Every major third is 400 cents (14 cents wide of pure 5:4)
- Every minor third is 300 cents (16 cents narrow of pure 6:5)
- Every interval is the same regardless of key — every key sounds identical structurally
- The wolf is dispersed equally among all twelve intervals — no key sounds worse than any other, but none sound as pure as a just-intoned major triad either

Equal temperament was adopted in France and Germany by the late 18th century, in England by the 19th. It is now overwhelmingly the dominant tuning system for fixed-pitch instruments — pianos, guitars, synthesizers, electronic music. It is *not* the dominant system for unaccompanied vocal music. We will come back to this distinction in Part VI; it matters enormously for choral spectrograms.

### IV.7 Cents (Ellis, 1880)

Alexander Ellis introduced the cent as a unit of measurement around 1880, defining 1 cent = 1/1200 of an octave on a logarithmic scale. In equation form:

$$\text{cents} = 1200 \cdot \log_2(f_1 / f_2)$$

Cents are now the universal currency of intonation discussion. A few useful benchmarks:

- **1 cent**: just-noticeable-difference for trained musicians under ideal conditions
- **5 cents**: professional tuning grade
- **10 cents**: "feels in tune" for most listeners
- **14 cents**: equal-tempered minus pure major third (the syntonic-comma-divided-by-four error of equal temperament's third)
- **22 cents**: syntonic comma; difference between Pythagorean and just major third
- **23.5 cents**: Pythagorean comma
- **50 cents**: a quarter-tone; the standard RPA threshold in MIR pitch evaluation
- **100 cents**: one equal-tempered semitone
- **1200 cents**: one octave

### IV.8 Modern extensions

Microtonal music explores divisions of the octave finer than 12 — 19-TET, 22-TET, 31-TET, 53-TET (which closely approximates Pythagorean and just intonation simultaneously), 72-TET — each with its own intonation properties. Composers like Harry Partch, Lou Harrison, and James Tenney explored these in the 20th century. Modern computer-based music makes any tuning system equally easy to implement.

For choral analysis, the relevant modern systems are *just intonation* (which is what well-trained a cappella choirs naturally use for sustained chords) and *equal temperament* (which is what choirs use when accompanied by piano, organ, or any fixed-pitch instrument). The interaction between the two is the central drama of choral intonation, covered in Part VI.

---

## Part V — Choral Acoustics

What we know about voices, formants, and the acoustic difference between solo and choral singing — drawn mostly from the Stockholm research tradition (Sundberg, Ternström, Rossing) and its descendants.

### V.1 Voice production: the source-filter model

The standard model of voice acoustics has two components:

- **Source** — vibrating vocal folds produce a roughly sawtooth-shaped pressure wave at the fundamental frequency F0. The source spectrum has energy at F0 and all integer multiples (harmonics).
- **Filter** — the vocal tract above the larynx (pharynx, oral cavity, nasal cavity, lips) acts as a complex acoustic resonator. Its resonances (formants) selectively amplify certain frequency regions.

The source determines the *pitch*; the filter determines the *vowel*. Different vowels correspond to different formant patterns. The first two formants (F1, F2) are the main vowel-distinguishing resonances. Roughly: /i/ has low F1 and high F2; /a/ has high F1 and middle F2; /u/ has low F1 and low F2.

A spectrogram visualizes the source-filter product. The horizontal lines are the harmonics of F0 (source); their brightness pattern (the spectral envelope) is the filter's frequency response (vowel and formants).

### V.2 Formant frequencies — rough ranges

For an adult, the first three formants typically sit in these ranges:

- F1: 250–900 Hz
- F2: 600–2900 Hz
- F3: 1700–3500 Hz

Higher formants (F4, F5) sit roughly in the 3000–5000 Hz region for adults. Vocal tract length is the primary determinant: longer tract → lower formants. Adult male F1 is roughly 17% lower than adult female F1 on average; children's formants are higher still.

### V.3 The singer's formant (Sundberg, 1974)

Johan Sundberg's 1974 paper *The Acoustics of the Singing Voice* established that classically-trained male opera singers produce an extra spectral peak around 2.5–3.5 kHz, distinct from the speech-formant pattern. This *singer's formant* is created by a clustering of the third, fourth, and fifth formants (F3, F4, F5) — pushed close together by the singer's positioning of the larynx and pharynx (narrowed larynx tube, widened pharynx, ratio of cross-sections approximately 1:6).

The functional significance: the singer's formant region is where orchestral energy is *lowest* (orchestras tend to roll off above 2 kHz). An operatic singer with a strong singer's formant can be heard over a full orchestra unamplified — the singer "punches through" in the frequency band where the orchestra has left them room.

The singer's formant is most prominent in trained male classical singers. There is limited evidence for an analogous effect in trained female sopranos (Rossing, Sundberg, Ternström 1985), though the acoustic mechanism is somewhat different. Pop, choral, and folk traditions do not consistently produce the effect.

### V.4 Solo vs choral phonation differences

The Stockholm group (Rossing, Sundberg, Ternström, mid-1980s) conducted a series of comparison studies on singers performing the same passages in solo versus choral mode. The headline findings:

- **Singer's formant intensity is typically lower in choral mode** for liturgical and small-ensemble singers. The 2.5–3.5 kHz cluster is less prominent; energy redistributes lower.
- **Fundamental frequency amplitude is typically higher in choral mode.** Choral singers favor the fundamental over the upper partials.
- **First formant frequencies shift slightly between modes** — articulation changes a little, consistent with the goal of *blend* rather than *projection*.
- **More recent work (Hunter et al. 2006) on professional opera choristers found the opposite pattern** — their choral-mode singing retained or even strengthened singer's-formant energy, presumably because they are trained as soloists and habitually produce that resonance.

The takeaway is not "choral singers have weaker singer's formants" but rather "the acoustic profile of choral singing depends heavily on the training tradition and the context." For our spectrogram, this is information about what we will *see* in real choral material vs. solo singer recordings.

### V.5 Vibrato

Vibrato is a roughly sinusoidal modulation of F0 at a rate of 4–7 Hz with a depth of roughly ±30–100 cents (a quarter-tone to a semitone). Trained classical singers produce vibrato as a relaxation-driven side effect of supported phonation; folk and pop styles often suppress or modify it.

Choir vibrato has measurable peculiarities. Cuesta et al.'s analysis of the Choral Singing Dataset found a mean vibrato rate around 5.13 Hz with a standard deviation of 0.26 Hz across choristers — very tight clustering. Jers and Ternström, analyzing a 16-channel choral recording in 2005, found an unexpected *lining up* of vibrato across singers in a section — singers in unison have a tendency to phase-lock their vibrato, producing the perceptual "chorus effect" of a section sounding bigger and brighter than the sum of its individuals.

Vibrato on a spectrogram looks like a horizontal line with a sinusoidal wobble. Coral's debug-signal vibrato (A4, 6 Hz, ±30 cents) renders exactly this; it is the canonical eyeball test for vibrato visualization.

### V.6 Long-Term Average Spectrum (LTAS) as identity

The LTAS is the average spectral envelope across a sustained passage — minutes of singing, averaged. The fast spectral features (the comings and goings of individual phonemes, vibrato cycles) average out, leaving the singer's habitual spectral envelope: their typical formant positions, their typical use of singer's formant, their typical balance between fundamental and upper partials.

LTAS has been used as a chorister "fingerprint" since at least the 1980s. It is one of the dimensions in the sonic-fingerprint hierarchy outlined in Coral's research roadmap (RESEARCH.md). The LTAS of an entire section averages across singers, producing a section-level fingerprint; the LTAS of the full ensemble averages across sections, producing an ensemble-level fingerprint. The relationship between these three levels — whether ensemble LTAS is reducible to the sum of section LTASes, whether section LTAS reduces to individuals — is exactly the research question Coral's sonic-fingerprint work is set up to address.

---

## Part VI — Choral Intonation Mathematics

The interaction of just intonation and equal temperament in real performance is the most quantitatively interesting thing about choral acoustics. It is also where Coral can do something no generic spectrogram can.

### VI.1 The tuning dichotomy

When a cappella SATB singers perform sustained chords, they tend toward *just intonation* for the vertical (harmonic) intervals — the beat-free 3:2 fifths, 5:4 major thirds, 6:5 minor thirds. This is what makes a good choir sound "ringing" or "locked in" on chords. The locked-in sensation is literal: at just intonation, the harmonics of adjacent notes coincide exactly, and the beating that produces a sense of harmonic tension disappears.

When the *same* singers perform melodic lines, they tend toward Pythagorean or even slightly sharper than equal-tempered intonation for leading tones and upward steps — what makes a melody sound "directional" or "going somewhere."

So the same chord might be sung in two different intonations depending on whether the singer is thinking of it as a melodic event or a harmonic event. David Howard's research at the University of York measured this directly in real choirs.

### VI.2 Measured intervals in real choirs

Lottermoser and Meyer (1960), cited in Ternström, measured the major and minor thirds of three commercially-recorded choirs:

- **Major thirds**: averaged 416 cents (vs 400 equal-tempered, 386 just)
- **Minor thirds**: averaged 276 cents (vs 300 equal-tempered, 316 just)

The pattern is striking: choirs are producing thirds that are *wider* than equal temperament for major thirds and *narrower* than equal temperament for minor thirds — *moving away from* just intonation, not toward it. The interpretation in the literature is that singers are exaggerating the major/minor difference for expressive emphasis. Octaves and fifths in the same recordings were measured close to just (3:2 fifths, 2:1 octaves).

This is exactly the kind of measurement Coral's spectrogram could enable on amateur recordings. A choir-director-facing tool that displays "your tenors are singing the major third 12 cents wide of just, 4 cents flat of equal" is a real pedagogical product that does not yet exist in user-friendly form.

### VI.3 Pitch drift

The deepest consequence of just intonation in a cappella performance is *pitch drift*. If the music modulates from one key to another and singers maintain just intervals in both keys, the absolute pitch of the home key shifts. Whether it shifts up or down depends on the specific chord progression.

David Howard's 2007 paper on intonation drift in SATB quartets measured drifts of approximately 18 cents per chord transition in specific cadential progressions designed to maximize the effect. Hiroko Terasawa's CCRMA study constructed a sequence designed to drift by a syntonic comma per repetition, accumulating to nearly 9% (about 150 cents — a minor second!) over twelve repetitions when sung in pure just intonation.

For Coral, pitch drift is an *observable*. A spectrogram with a clearly-displayed reference pitch (A4 = 440 Hz, currently the brightest line in our note grid) and accurate enough rendering will visibly show drift across a long performance. The bass line that started on E2 (82 Hz) and ends 30 cents below E2 has drifted — and the choir doesn't necessarily know it happened.

### VI.4 Vowel-dependent pitch

A subtler effect: the perceived pitch of a sung note depends in part on the vowel. Acoustically identical fundamental frequencies on /a/ and /i/ are perceived as slightly different pitches; trained singers compensate by adjusting F0 slightly across vowels to keep the perceived pitch constant. Daniel Daffern and others have measured this; it is small (a few cents) but real and consistent.

For our purposes this is a reminder that "the pitch I see on the spectrogram" and "the pitch the singer is producing" and "the pitch the listener perceives" are three slightly different things. The first is the most measurable; the last is what the performance is actually about.

### VI.5 Unison singing — the hardest problem

Unison in a choir section is acoustically *not* unison. Multiple singers attempting to sing the same pitch produce a distribution of F0 values around a mean, with standard deviation typically 5–14 cents (Cuesta et al.'s measurements on the Choral Singing Dataset). Below 5 cents the section sounds tight; above 14 cents it starts to sound dissonant rather than blended.

This *F0 dispersion* — Sacerdote's term from 1957 — is what makes a choir section sound like a section rather than a soloist. The dispersion is also what makes multi-F0 estimation on choir unison so hard: there is no single F0 to find; there is a distribution. Cuesta et al. 2019, *A Framework for Multi-F0 Modeling in SATB Choir Recordings*, proposes modeling the *distribution* of F0 within each section rather than trying to estimate individual F0s — explicitly noting that the latter is information-theoretically very hard from a mixed recording.

For Coral, this means that even with a perfect multi-F0 algorithm, "Each section sings exactly one pitch" is the wrong mental model. The right mental model is "Each section produces a pitch distribution with a center and a width, and the perception of the section is shaped by both."

---

## Part VII — Choral-Specific Spectrogram Considerations

Bringing the math, the test architecture, and the choral acoustics together, here is what makes a *choir-specific* spectrogram different from a generic one — beyond the parameter tuning already documented in `README.md`.

### VII.1 The Constant-Q Transform as the natural musical representation

Judith Brown's 1991 *Calculation of a constant-Q spectral transform* introduced what is now the standard musical alternative to the STFT. Where the STFT has uniform frequency resolution across all bins (e.g., 10.8 Hz/bin at our settings), the CQT has uniform *log-frequency* resolution — every bin spans the same number of cents regardless of frequency.

In typical musical CQT settings, the configuration is 12, 24, 36, or 96 bins per octave. At 24 bins/octave, every bin spans 50 cents (half a semitone), at all frequencies from sub-bass to high treble. This matches the structure of musical pitch perception exactly: the same number of bins separates A2 from A3 as separates A4 from A5.

Costs of CQT vs STFT:

- Computational: roughly equivalent at high frequencies, more expensive at low frequencies where window length grows
- Invertibility: STFT can be perfectly inverted; CQT can be inverted to ~55 dB SNR (good enough for most uses, not perfect)
- Implementation complexity: significantly higher than STFT; multiple competing algorithms (direct, sub-sampled per octave, hybrid)
- Cuesta et al.'s SATB multi-F0 work uses the *harmonic CQT* (HCQT), stacking copies of the CQT at multiples of each harmonic — explicitly because the constant log-frequency structure makes harmonic templates fixed-shape across pitches

For Coral, the path is: STFT now (shipping), CQT in v2 once we have real intonation features to express. The "constant cents per bin" property would let us draw a note-grid overlay with much higher fidelity than our current STFT bin sampling allows.

### VII.2 The harmonic salience problem

A choral spectrogram of a four-voice chord shows many more than four prominent peaks. Each voice's fundamental is the *most* visible, but the second, third, fourth, fifth harmonics of each voice are typically also clearly visible. Many of those harmonics coincide with other voices' fundamentals or harmonics — by design, that is what *consonance* is. A major triad's notes share many harmonics; that is why it sounds consonant.

For F0 estimation, this means a peak-picking algorithm that grabs the N strongest peaks does not give you the N fundamentals — it gives you a mix of fundamentals and salient harmonics. Algorithms like Salience-based methods (Salamon and Gómez, *Melody extraction from polyphonic music signals using pitch contour characteristics*, 2012), CREPE (Kim et al. 2018), Cuesta's HCQT-CNN, and the Schramm/Benetos PLCA approach all attack this differently, but the underlying problem is the same: distinguishing "the F0 you should call out" from "the harmonic of someone else's F0."

For Coral's current scope, we don't have to solve this — the spectrogram displays everything and lets the user (or a future F0 tracker) interpret. But it is on the roadmap.

### VII.3 The blend-vs-individual tension

The acoustic goal of a good choir section is *blend* — multiple individual voices perceived as a single unified section voice. The acoustic goal of Coral's per-singer identification research is the *opposite* — extracting the individual signature from a blended mix.

Coral is, in some sense, working against the artistic intent. This is worth being honest about. The pedagogical use case (showing each section their part) and the analytical/musicological use case (forensically dissecting performances for study) are well-served by individual identification; the artistic use case (a choir director listening to their own choir to refine the blend) might be better served by tooling that explicitly characterizes *blend quality* — F0 dispersion, vibrato lock-in coherence, vowel uniformity — rather than singer separation.

A choir-director-facing v2 of Coral could expose both modes: "blend audit" mode (statistics on dispersion and coherence) versus "individual identification" mode (per-singer F0 tracks). The first is more useful pedagogically; the second is more interesting research.

### VII.4 Just-intonation visualization

If we display measured F0 values against a note grid drawn in equal temperament, well-tuned just intonation will appear to be *out of tune*. The major third on a just-intoned C-major chord will sit visibly below the equal-tempered E line; the minor third will sit visibly above the equal-tempered E♭ line.

This is correct, and it is in fact the desired behavior — but only if we tell the user. A naive interpretation ("the choir is flat on E!") is wrong; the correct interpretation ("the choir is singing just intonation, which is 14 cents below equal-tempered E") is right.

Two options for Coral v2:

1. **Annotate the reference grid with both ET and JI** — show two lines for the major third, one at 400 cents (ET), one at 386 cents (JI). Let the user see which the choir actually hit.
2. **Adaptive temperament inference** — detect the local tonal center and draw the JI grid relative to it. Much harder, requires chord tracking.

Option 1 is the right v2 move. Option 2 is a research project.

### VII.5 Reassigned spectrograms and synchrosqueezing

A class of post-processing techniques that refine the spectrogram's time-frequency localization beyond the underlying STFT's resolution, by using the instantaneous frequency and group-delay information present in the FFT's phase output. The *reassigned spectrogram* (Auger and Flandrin, 1995) reassigns each STFT bin's energy to its "true" instantaneous frequency-time location, producing a much sharper image without changing the underlying analysis.

Reassignment costs perfect invertibility, but for analysis-only work (no resynthesis required) it can dramatically improve readability. Librosa exposes `librosa.reassigned_spectrogram`. For our choral context, reassignment would sharpen vibrato visualization in particular — instead of a wobbly fuzzy line, you would see a precise wobble.

This is a v2 conversation, not v0. But it is worth knowing about: the STFT is not the only game in town, and the post-processing toolkit is rich.

### VII.6 Datasets in the choral MIR ecosystem

For real-data validation (Tier 4 of the testing plan), the relevant publicly available choral datasets:

| Dataset | Size | Annotations | Notes |
|---|---|---|---|
| **Choral Singing Dataset (CSD)** | 16 singers, 3 SATB pieces, ~7 min | MIDI, per-section F0, note annotations | Cuesta et al. 2018; the most-cited choral MIR dataset |
| **Dagstuhl ChoirSet (DCS)** | 13 singers, ~55 min, mixed mic setups (lavalier, dynamic, headset) | F0, beats, score | Rosenzweig et al. 2020; closest match to our multi-track v1 design |
| **ESMUC Choir Dataset (ECD)** | 3 SATB pieces, full choir | Manually-corrected F0 contours and notes | Cuesta PhD 2022 |
| **Cantoría / Bach Chorales** | 26 Bach chorale recordings | Per-singer audio, MIDI, score | Schramm and Benetos |
| **ChoralSynth** | Variable; synthetic | Perfect ground truth | Synthesizer-rendered; use for upper-bound checks |

The Cuesta-Cuesta-Cuesta sequence (CSD then HCQT-CNN then unison dispersion work) is the most coherent body of recent choral MIR. Their methodology — multi-track close-mic recording with simultaneously-recorded room reference — is what Coral's multi-track v1 design is patterned after.

### VII.7 Where Coral fits in the choral MIR landscape

Existing tools:

- **Praat** (Boersma, Amsterdam) — the universal acoustic-phonetics workhorse. Free, scriptable, but UI dated and not music-specialized.
- **Sonic Visualiser** (QMUL, UK) — well-designed musical analysis UI, plugin ecosystem (Vamp plugins). Free, mature, but more focused on instrumental MIR.
- **Tony** (QMUL) — pitch annotation tool, derivative of Sonic Visualiser, specialized for vocal monophonic pitch correction.
- **VoceVista** — commercial vocal-pedagogy spectrogram tool; closest existing product to a "choir spectrogram." Single-voice oriented.
- **Auto-Tune Choir / Antares** — production tool for *synthesizing* choirs from single voices, the inverse of what Coral does.
- **Melodyne** — commercial polyphonic pitch correction; can do multi-F0 on simple polyphony but not choral unison.

Coral's place: a *purpose-built choral analysis tool*. Praat is general phonetics; Sonic Visualiser is general music; Tony is solo vocal; VoceVista is solo pedagogy. None of them is specifically built for the dynamics of a section-organized SATB ensemble with the intonation, blend, and unison dispersion phenomena unique to choirs.

That niche is real, and it is small but defensible. The hierarchical sonic fingerprint research (RESEARCH.md) is what makes the niche valuable as a contribution rather than just a product — *purpose-built tooling for a problem that the research community has identified but not yet solved end-to-end*.

---

## Part VIII — Open Research Questions Coral Could Address

Items where the literature has acknowledged a gap and Coral's data infrastructure would be well-placed to fill it.

1. **The blend / dispersion / coherence triangulation.** F0 dispersion has been measured (Cuesta), vibrato coherence has been measured (Jers & Ternström), spectral envelope blend has been measured (Daffern). No single tool synthesizes all three into a "blend score" for a choir director. Coral's multi-channel design would support this.

2. **Real-time pitch-drift telemetry.** Howard's work measures drift offline on prepared recordings. A live-mic version that warns a choir as they begin to drift — and shows which singer or section is leading the drift — has been proposed but not built in production form.

3. **Just-intonation grid visualization.** Standard tuning meters and spectrograms show equal-tempered pitch. A grid that adapts to the local tonal center and shows the JI target intervals dynamically would be a genuine pedagogical innovation. Implementation requires real-time chord tracking, which is well-studied for instrumental music but less so for choral material.

4. **Section-specific singer-formant tracking.** The Stockholm group's work characterized solo vs choral phonation in aggregate. Per-section, per-singer formant tracking in real recordings would let us study how individual choristers shift their phonation when joining or leaving a section — a question of vocal pedagogy that is acoustically tractable but underdocumented.

5. **The "vocal fingerprint half-life" question.** How stable is an individual singer's sonic fingerprint across days, illnesses, fatigue states, vocal ages? Speaker-recognition literature acknowledges this question; the choral-singer-specific version is essentially open.

---

## References

Auger, F., & Flandrin, P. (1995). Improving the readability of time-frequency and time-scale representations by the reassignment method. *IEEE Transactions on Signal Processing*, 43(5), 1068–1089.

Brown, J. C. (1991). Calculation of a constant Q spectral transform. *Journal of the Acoustical Society of America*, 89(1), 425–434.

Cooley, J. W., & Tukey, J. W. (1965). An algorithm for the machine calculation of complex Fourier series. *Mathematics of Computation*, 19(90), 297–301.

Cuesta, H., Gómez, E., Martorell, A., & Loáiciga, F. (2018). Analysis of intonation in unison choir singing. *Proceedings of ICMPC*.

Cuesta, H., McFee, B., & Gómez, E. (2020). Multiple F0 estimation in vocal ensembles using convolutional neural networks. *Proceedings of ISMIR*.

Cuesta, H. (2022). Data-driven pitch content description of choral singing recordings. PhD thesis, Universitat Pompeu Fabra.

Daffern, H. (2017). Blend in singing ensemble performance: Vibrato production in a vocal quartet. *Journal of Voice*, 31(3), 385.e23–385.e29.

Ellis, A. J. (1880, in supplementary work to). On the musical scales of various nations. *Journal of the Society of Arts*.

Fourier, J. (1822). *Théorie analytique de la chaleur*. Paris: Firmin Didot.

Gabor, D. (1946). Theory of communication. *Journal of the Institution of Electrical Engineers*, 93(III), 429–457.

Harris, F. J. (1978). On the use of windows for harmonic analysis with the discrete Fourier transform. *Proceedings of the IEEE*, 66(1), 51–83.

Howard, D. M. (2007). Intonation drift in a cappella SATB quartet singing with key modulation. *Journal of Voice*, 21(3), 300–315.

Hunter, E. J., Titze, I. R., & Alipour, F. (2006). The acoustic characteristics of professional opera singers performing in chorus versus solo mode. *Journal of Voice*, 20(3), 478–487.

Jers, H., & Ternström, S. (2005). Intonation analysis of a multi-channel choir recording. *KTH TMH Quarterly Progress and Status Report*.

Kim, J. W., Salamon, J., Li, P., & Bello, J. P. (2018). CREPE: A convolutional representation for pitch estimation. *Proceedings of ICASSP*.

Maaz, M., DeVoe, L., Hatfield-Dodds, Z., & Carlini, N. (2025). Agentic property-based testing: Finding bugs across the Python ecosystem. arXiv:2510.09907.

Mauch, M., & Dixon, S. (2014). pYIN: A fundamental frequency estimator using probabilistic threshold distributions. *Proceedings of ICASSP*.

McFee, B., Raffel, C., Liang, D., Ellis, D. P. W., McVicar, M., Battenberg, E., & Nieto, O. (2015). librosa: Audio and music signal analysis in Python. *Proceedings of the 14th Python in Science Conference*, 18–25.

Nyquist, H. (1928). Certain topics in telegraph transmission theory. *Transactions of the AIEE*, 47(2), 617–644.

Raffel, C., McFee, B., Humphrey, E. J., Salamon, J., Nieto, O., Liang, D., & Ellis, D. P. W. (2014). mir_eval: A transparent implementation of common MIR metrics. *Proceedings of ISMIR*.

Rosenzweig, S., Cuesta, H., Weiß, C., Scherbaum, F., Gómez, E., & Müller, M. (2020). Dagstuhl ChoirSet: A multitrack dataset for MIR research on choral singing. *Transactions of the International Society for Music Information Retrieval*, 3(1), 98–110.

Rossing, T. D., Sundberg, J., & Ternström, S. (1985). Acoustic comparison of soprano solo and choir singing. *KTH STL-QPSR*, 26(4), 43–58.

Rossing, T. D., Sundberg, J., & Ternström, S. (1986). Acoustic comparison of voice use in solo and choir singing. *Journal of the Acoustical Society of America*, 79(6), 1975–1981.

Salamon, J., & Gómez, E. (2012). Melody extraction from polyphonic music signals using pitch contour characteristics. *IEEE Transactions on Audio, Speech, and Language Processing*, 20(6), 1759–1770.

Schörkhuber, C., & Klapuri, A. (2010). Constant-Q transform toolbox for music processing. *Proceedings of the 7th Sound and Music Computing Conference*.

Schramm, R., & Benetos, E. (2017). Automatic transcription of a cappella recordings from multiple singers. *Proceedings of the AES International Conference on Semantic Audio*.

Shannon, C. E. (1949). Communication in the presence of noise. *Proceedings of the IRE*, 37(1), 10–21.

Sundberg, J. (1974). Articulatory interpretation of the "singing formant". *Journal of the Acoustical Society of America*, 55(4), 838–844.

Sundberg, J. (1987). *The science of the singing voice*. DeKalb, IL: Northern Illinois University Press.

Ternström, S. (2003). Choir acoustics — an overview of scientific research. *KTH TMH-QPSR*.

Werckmeister, A. (1691). *Musicalische Temperatur*. Frankfurt and Leipzig.
