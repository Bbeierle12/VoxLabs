//! Definitional constants: numbers that are mathematics or a standard's
//! definition, not settings. Named here so the DSP modules contain no bare
//! numerals, and so each one says what it is. None of these is a tunable and
//! none appears in `pipeline.toml` — changing one would not retune the
//! analysis, it would make the formula wrong.
//!
//! Angles use `std::f32::consts::TAU` (2π) directly.

/// The two of halving and doubling: the Nyquist bin `n / 2`, the midpoint
/// `(a + b) / 2`, the parabola vertex `(s0 − s2) / (2·(s0 − 2·s1 + s2))`, and
/// `2^x` in octave arithmetic. Range: exactly 2.
pub const TWO: f32 = 2.0;
/// [`TWO`] for `f64` arithmetic. Range: exactly 2.
pub const TWO_F64: f64 = 2.0;
/// [`TWO`] as an index/length divisor (Nyquist bin, integer midpoint).
/// Range: exactly 2.
pub const TWO_USIZE: usize = 2;
/// One half: bin centering, half-energy, and the Hann window's DC term.
/// Range: exactly 0.5.
pub const HALF: f32 = 0.5;
/// [`HALF`] for `f64` arithmetic. Range: exactly 0.5.
pub const HALF_F64: f64 = 0.5;
/// Exponent of a square (`x.powi(SQUARED)`). Range: exactly 2.
pub const SQUARED: i32 = 2;
/// Width of a sliding window over adjacent pairs (`slice.windows(2)`):
/// cycle-to-cycle differences. Range: exactly 2.
pub const ADJACENT_PAIR: usize = 2;
/// Milliseconds in a second. Range: exactly 1000.
pub const MILLIS_PER_SECOND: f32 = 1000.0;
/// Microseconds in a second (`f64`, for timing arithmetic). Range: exactly 1e6.
pub const MICROS_PER_SECOND_F64: f64 = 1_000_000.0;
/// Closed-form eigenvalues of a 2×2 Hermitian matrix `[[a, c], [c*, b]]`:
/// `λ = (a + b ± sqrt((a − b)² + 4|c|²)) / 2`; this is the 4. Range: exactly 4.
pub const EIGEN2_DISCRIMINANT_FACTOR: f64 = 4.0;

/// Decibels per decade of a *power* ratio: `10·log10(P/P0)`. Range: exactly 10.
pub const DB_PER_DECADE_POWER: f32 = 10.0;
/// Decibels per decade of an *amplitude* ratio: `20·log10(A/A0)`. Range:
/// exactly 20.
pub const DB_PER_DECADE_AMPLITUDE: f32 = 20.0;
/// Logarithm base of the decibel: `10^(dB/20)` converts back to a ratio.
/// Range: exactly 10.
pub const DB_LOG_BASE: f32 = 10.0;

/// Hamming window: `w[n] = A0 − A1·cos(2πn/(N−1))`, A0. Range: exactly 0.54.
pub const HAMMING_A0: f32 = 0.54;
/// Hamming window A1 (see [`HAMMING_A0`]). Range: exactly 0.46.
pub const HAMMING_A1: f32 = 0.46;
/// Hann window: `w[n] = A0 − A0·cos(2πn/(N−1))`, A0. Range: exactly 0.5.
pub const HANN_A0: f32 = 0.5;

/// Semitones in an octave (12-tone equal temperament). Range: exactly 12.
pub const SEMITONES_PER_OCTAVE: f32 = 12.0;
/// [`SEMITONES_PER_OCTAVE`] as an integer for pitch-class arithmetic.
/// Range: exactly 12.
pub const SEMITONES_PER_OCTAVE_I32: i32 = 12;
/// [`SEMITONES_PER_OCTAVE`] as a table length (pitch-class names).
/// Range: exactly 12.
pub const SEMITONES_PER_OCTAVE_USIZE: usize = 12;
/// Cents in an octave (100 per semitone). Range: exactly 1200.
pub const CENTS_PER_OCTAVE: f32 = 1200.0;
/// Cents in a semitone. Range: exactly 100.
pub const CENTS_PER_SEMITONE: f32 = 100.0;
/// MIDI note number of A4 — the anchor of the MIDI/12-TET mapping. Range:
/// exactly 69.
pub const MIDI_A4: f32 = 69.0;
/// Fraction → percent. Range: exactly 100.
pub const PERCENT: f32 = 100.0;
/// The percentile that is the median. Range: exactly 50.
pub const MEDIAN_PERCENTILE: f32 = 50.0;

/// Denominator of the three-point parabolic *peak height*:
/// `y1 − (y0 − y2)² / (8·(y0 − 2·y1 + y2))`. Range: exactly 8.
pub const PARABOLIC_PEAK_DENOM: f32 = 8.0;
/// Denominator of the parabolic peak height written in terms of the vertex
/// offset δ: `y1 − (y0 − y2)·δ / 4`. Range: exactly 4.
pub const PARABOLIC_HEIGHT_DENOM: f32 = 4.0;

/// Closed–open tube: the n-th resonance sits at `(2n−1)·c / (4·L)`; this is
/// the 4. Range: exactly 4.
pub const QUARTER_WAVE_DENOM: f32 = 4.0;
/// The `(2n−1)` numerators of the first three closed–open tube modes.
/// Range: exactly [1, 3, 5].
pub const QUARTER_WAVE_ODD_MULTIPLES: [f32; 3] = [1.0, 3.0, 5.0];

/// A third-octave band spans `fc / 2^(1/6)` to `fc · 2^(1/6)`; this is the
/// 1/6. Range: exactly 1/6.
pub const THIRD_OCTAVE_HALF_BAND_EXP: f32 = 1.0 / 6.0;
