# Numerical tolerances

What "matches" means for Coral's DSP, and why. Decided 2026-06-09 alongside
the periodic-Hann switch; read this before generating any oracle fixtures or
loosening a test tolerance.

## Precision floor: float32

The hot path is float32 end to end — samples, Hann window, windowed buffer,
and output magnitudes are all `Float32Array`; only fft.js's internal complex
array is float64. The pipeline's effective precision is therefore
**float32 (≈1.2e-7 relative per operation)**, with accumulated windowing and
FFT rounding error growing with `fftSize` (worst case here: the detection
STFT at N = 8192).

We deliberately do **not** promote the buffers to float64. The cost would be
memory and copy bandwidth in the per-frame hot loop, and nothing downstream
(display quantized to 8-bit dB steps; detection thresholds calibrated in
whole-dB units) can observe the extra precision. Revisit only if a future
oracle comparison demonstrably fails because of the f32 floor, not before.

## Tiers

| Tier | Tolerance | Used for |
|---|---|---|
| **Bit-identical** | `Object.is`, no tolerance | Batch ↔ streaming parity (`stft-parity.test.ts`). Both paths run the same f32 arithmetic in the same order; any divergence is a code change, not noise. |
| **Calibration** | ±0.3 dB | 0-dB canaries for bin-aligned unit sines (44.1 kHz and 48 kHz). Headroom covers scalloping residue and frame placement, not scaling errors. |
| **Cross-implementation** | rtol 1e-4 + atol 1e-7 on linear magnitudes | librosa oracle fixtures (`oracle-fixtures.test.ts`). The atol term floors near-zero bins where relative error is meaningless; rtol governs bins above atol/rtol = 1e-3. |

### Measured (2026-06-10, librosa 0.11.0, 6 fixtures)

Worst case across all fixtures (sines, two-tone+noise, chirp; 44.1/48 kHz;
2048/512 and 4096/1024 framing): **max rel error 1.6e-6** on
rtol-governed bins, **max abs error 3.9e-9** on atol-governed bins —
~60× and ~25× inside the tier. The f32 pipeline lands at ~2e-6 because
fft.js computes internally in float64; only sample storage and
windowing are float32, so the predicted 1e-5–1e-4 accumulation budget
was conservative. The tier deliberately stays at 1e-4 anyway: it is the
*contract*, with headroom for future framings and signal content.
Tighten to rtol 1e-5 only as a deliberate decision recorded here — and
if the measured number ever drifts within 10× of the tier, treat that
as a regression to root-cause, not a tolerance to loosen.

### Regenerating fixtures

```bash
python3 -m venv scripts/.venv
scripts/.venv/bin/pip install -r scripts/requirements-fixtures.txt
scripts/.venv/bin/python scripts/generate_fixtures.py
```

Fixtures live in `src/audio/__fixtures__/oracle/` and are committed —
CI never needs Python. Each stores the float32 input samples alongside
librosa's expected magnitudes, so both implementations always see
bit-identical input; regeneration is only needed if cases are added or
the window/framing convention changes (in which case re-read this whole
document first).

## Window convention (prerequisite for the cross-impl tier)

`createHannWindow` is the **periodic** Hann (`denom = N`), matching
`librosa.stft(..., window='hann')` and `scipy.signal.get_window(..., fftbins=True)`.
Fixtures generated against the symmetric window (`denom = N - 1`) differ by
O(1/N) per coefficient — ~2.4e-4 at N = 4096, i.e. *above* the cross-impl
tier — so the convention is load-bearing, not cosmetic. Changed from
symmetric on 2026-06-09; any fixture generated before that date is invalid.

## Calibration reference

A unit-amplitude sine centered on bin k peaks at 0 dB in both STFT paths via
the single-sided correction `2 / sum(window)`. DC (bin 0) gets the 2× factor
it shouldn't (+6 dB) — documented in `spectrogram.ts`, excluded by the
default 50 Hz display floor.
