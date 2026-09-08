use crate::config::consts::{
    ADJACENT_PAIR, CENTS_PER_SEMITONE, DB_LOG_BASE, DB_PER_DECADE_AMPLITUDE, DB_PER_DECADE_POWER,
    HALF, HAMMING_A0, HAMMING_A1, HANN_A0, MIDI_A4, PARABOLIC_HEIGHT_DENOM, PARABOLIC_PEAK_DENOM,
    PERCENT, SEMITONES_PER_OCTAVE, SEMITONES_PER_OCTAVE_I32, SEMITONES_PER_OCTAVE_USIZE, TWO,
    TWO_USIZE,
};
use crate::config::voiceprint::N_SCALAR_FEATURES;
use crate::config::{
    CentroidConfig, CppConfig, FormantConfig, HarmonicsConfig, HnrConfig, LpcConfig,
    NoiseFloorConfig, PerturbationConfig, RoomCalibrationConfig, TimbreConfig, TuningConfig,
    VoiceClassConfig, VoiceprintConfig, VoicingConfig, YinConfig,
};
use crate::types::{Formant, MAX_PARTIALS, N_FORMANTS, VOICEPRINT_PROFILE_LEN, Voiceprint};
use aberth::AberthSolver;
use std::f32::consts::TAU;

// Stage configuration (Plan v3 §4): the defaults in `config`, mirrored in
// `pipeline.toml`. Phase 1 threads these through `Stage::init`.
const YIN: YinConfig = YinConfig::DEFAULT;
const VOICING: VoicingConfig = VoicingConfig::DEFAULT;
const NOISE: NoiseFloorConfig = NoiseFloorConfig::DEFAULT;
const CALIB: RoomCalibrationConfig = RoomCalibrationConfig::DEFAULT;
const LPC: LpcConfig = LpcConfig::DEFAULT;
const FORMANT: FormantConfig = FormantConfig::DEFAULT;
const HARMONICS: HarmonicsConfig = HarmonicsConfig::DEFAULT;
const HNR: HnrConfig = HnrConfig::DEFAULT;
const PERTURB: PerturbationConfig = PerturbationConfig::DEFAULT;
const CPP: CppConfig = CppConfig::DEFAULT;
const CENTROID: CentroidConfig = CentroidConfig::DEFAULT;
const TIMBRE: TimbreConfig = TimbreConfig::DEFAULT;
const CLASS: VoiceClassConfig = VoiceClassConfig::DEFAULT;
const TUNING: TuningConfig = TuningConfig::DEFAULT;
const VOICEPRINT: VoiceprintConfig = VoiceprintConfig::DEFAULT;

/// YIN analysis window length: the number of samples compared per lag in the
/// difference function. The analysis frame must be at least this plus the
/// maximum search lag so `x[j + tau]` never reads past the frame end.
pub const YIN_WINDOW: usize = YIN.window;

/// YIN search bounds (Hz) and the absolute threshold for the CMND dip.
const YIN_F0_MIN: f32 = YIN.f0_min_hz;
const YIN_F0_MAX: f32 = YIN.f0_max_hz;
const YIN_THRESHOLD: f32 = YIN.threshold;

thread_local! {
    // rustfft's planner caches plans by size; sharing one per thread means
    // cpp_db/spectral_centroid plan their FFT once instead of on every voiced
    // analysis frame (Spectrogram keeps its own plan the same way).
    static FFT_PLANNER: std::cell::RefCell<rustfft::FftPlanner<f32>> =
        std::cell::RefCell::new(rustfft::FftPlanner::new());
}

/// Forward FFT plan from the shared per-thread planner (also used by the
/// spatial module's two-channel STFT).
pub(crate) fn fft_forward(n: usize) -> std::sync::Arc<dyn rustfft::Fft<f32>> {
    FFT_PLANNER.with(|p| p.borrow_mut().plan_fft_forward(n))
}

/// Solves the Yule-Walker equations using Levinson-Durbin recursion.
/// Returns the prediction-error polynomial `A(z) = [1, a1, .., ap]`, i.e. the
/// residual is `e[n] = x[n] + Σ aⱼ x[n-j]`, so the roots of `A(z)` are the
/// LPC poles — the form `formants_from_lpc` consumes directly.
///
/// Requires autocorrelation lags `0..=order`; an undersized `autocorr` cannot
/// constrain the model and yields the identity polynomial `[1, 0, ..]` (an
/// all-pass "no prediction") rather than indexing out of bounds.
pub fn levinson_durbin(autocorr: &[f32], order: usize) -> Vec<f32> {
    let mut a = vec![0.0; order + 1];
    a[0] = 1.0;
    if autocorr.len() < order + 1 {
        return a;
    }
    let mut e = autocorr[0];

    for i in 1..=order {
        if e.abs() < LPC.levinson_error_floor {
            // Prediction error has collapsed (silent or degenerate frame).
            // Stop the recursion; remaining higher-order coeffs stay 0, which
            // is a valid lower-order polynomial rather than a NaN blow-up.
            break;
        }
        let mut acc = autocorr[i];
        for j in 1..i {
            acc += a[j] * autocorr[i - j];
        }
        let k = -acc / e;

        a[i] = k;
        let mut new_a = a.clone();
        for j in 1..i {
            new_a[j] = a[j] + k * a[i - j];
        }
        a = new_a;
        e *= 1.0 - k * k;
    }

    a
}

/// Extract up to three formants (F1, F2, F3) from an LPC prediction polynomial.
///
/// `lpc` is `[a0, a1, ..., ap]` with `a0 == 1`, i.e. `A(z) = a0 + a1 z^-1 + ...
/// + ap z^-p`. The formant poles are the roots of `A(z)` **in z**.
///
/// Aberth's method (the `aberth` crate) finds roots of a polynomial written in
/// *ascending* powers of its variable: `c0 + c1 x + ... + cp x^p`. Handing it
/// `[a0..ap]` directly would solve in `x = z^-1` and return the *reciprocal*
/// poles — angles negated, magnitudes inverted — a silent mirror that puts
/// formants at the wrong frequencies with flipped bandwidths. We therefore
/// **reverse** the coefficients: since `z^p * A(z) = sum_k a_k z^(p-k)`, the
/// reversed array is a polynomial in `x = z`, so the returned roots are the
/// poles directly.
pub fn formants_from_lpc(lpc: &[f32], sample_rate: f32) -> [Formant; N_FORMANTS] {
    let mut result = [Formant {
        frequency: 0.0,
        bandwidth: 0.0,
    }; N_FORMANTS];

    if lpc.len() < FORMANT.min_coefficients {
        return result;
    }

    // Reverse (root in z, not z^-1) and promote to f64 for conditioning.
    let coeffs: Vec<f64> = lpc.iter().rev().map(|&c| c as f64).collect();

    let mut solver = AberthSolver::new();
    solver.max_iterations = FORMANT.aberth_max_iterations;
    solver.epsilon = FORMANT.aberth_epsilon;
    let roots = solver.find_roots(&coeffs);

    let fs = sample_rate as f64;
    let two_pi = std::f64::consts::TAU;
    let mut formants: Vec<Formant> = Vec::new();

    for z in roots.iter() {
        // Keep one member of each complex-conjugate pair (upper half-plane).
        if z.im <= 0.0 {
            continue;
        }
        let r = z.norm();
        // Minimum-phase guarantee: real formant poles satisfy 0 < |z| < 1.
        if r <= 0.0 || r >= 1.0 {
            continue;
        }
        let freq = fs * z.arg() / two_pi;
        let bandwidth = -fs * r.ln() / std::f64::consts::PI;

        // Speech formant band + sharpness gate (wide resonances aren't formants).
        if (FORMANT.band_lo_hz..=FORMANT.band_hi_hz).contains(&freq)
            && bandwidth < FORMANT.max_bandwidth_hz
        {
            formants.push(Formant {
                frequency: freq as f32,
                bandwidth: bandwidth as f32,
            });
        }
    }

    formants.sort_by(|a, b| {
        a.frequency
            .partial_cmp(&b.frequency)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (slot, f) in result.iter_mut().zip(formants) {
        *slot = f;
    }
    result
}

/// Computes a parabolic interpolation around the minimum lag `tau`.
/// Boundary lags (and any `tau` without two in-bounds neighbours, including on
/// an empty slice) are returned unrefined.
pub fn parabolic_interpolation(diff_fn: &[f32], tau: usize) -> f32 {
    if tau == 0 || tau + 1 >= diff_fn.len() {
        return tau as f32;
    }

    let s0 = diff_fn[tau - 1];
    let s1 = diff_fn[tau];
    let s2 = diff_fn[tau + 1];

    let denom = s0 - TWO * s1 + s2;
    if denom.abs() < YIN.parabolic_flat_eps {
        tau as f32
    } else {
        tau as f32 + (s0 - s2) / (TWO * denom)
    }
}

/// Result of a YIN pitch estimate.
pub struct PitchEstimate {
    /// Estimated fundamental frequency in Hz.
    pub f0: f32,
    /// Periodicity confidence in `[0, 1]` (`1 - d'(tau)` at the chosen lag).
    /// Near 1.0 for a clean periodic signal, near 0.0 for noise/silence.
    pub confidence: f32,
}

/// YIN difference function `d(tau)` for `tau in 0..=max_lag`:
/// `d(tau) = sum_{j=0}^{window-1} (x[j] - x[j+tau])^2`, with `d(0) = 0`.
///
/// The caller should guarantee `window + max_lag <= samples.len()` so every
/// `x[j+tau]` is in bounds — the same contract the GPU buffer sizing honors.
/// (The original 1024-sample buffer violated it, biasing the high lags.)
/// An undersized frame returns the all-zero difference, which downstream CMND
/// normalization turns into a zero-confidence (unvoiced) estimate.
pub fn yin_difference(samples: &[f32], window: usize, max_lag: usize) -> Vec<f32> {
    let mut diff = vec![0.0f32; max_lag + 1];
    if samples.len() < window + max_lag {
        return diff;
    }
    for tau in 1..=max_lag {
        let mut sum = 0.0f32;
        for j in 0..window {
            let delta = samples[j] - samples[j + tau];
            sum += delta * delta;
        }
        diff[tau] = sum;
    }
    diff
}

/// Resolve the YIN lag search bounds `(tau_min, tau_max)` for a diff/cumsum of
/// length `len`. Returns `None` if the range is too short to analyze.
fn yin_bounds(len: usize, sample_rate: f32) -> Option<(usize, usize)> {
    let tau_max = ((sample_rate / YIN_F0_MIN) as usize).min(len.saturating_sub(1));
    if tau_max < YIN.min_max_lag {
        return None;
    }
    let tau_min = ((sample_rate / YIN_F0_MAX) as usize).clamp(YIN.min_tau, tau_max - 1);
    Some((tau_min, tau_max))
}

/// YIN steps 3-4 on a precomputed CMND array: absolute-threshold lag search
/// (descend into the first sub-threshold dip, else fall back to the global
/// minimum) followed by parabolic interpolation. Shared by every path so they
/// all pick the same lag from the same CMND.
fn yin_pick(
    cmnd: &[f32],
    tau_min: usize,
    tau_max: usize,
    sample_rate: f32,
) -> Option<PitchEstimate> {
    // Step 3: absolute threshold. First lag whose CMND dips below the threshold,
    // then descend to the bottom of that dip (its local minimum).
    let mut tau_est = None;
    let mut tau = tau_min;
    while tau <= tau_max {
        if cmnd[tau] < YIN_THRESHOLD {
            while tau < tau_max && cmnd[tau + 1] < cmnd[tau] {
                tau += 1;
            }
            tau_est = Some(tau);
            break;
        }
        tau += 1;
    }

    // Fallback: nothing crossed the threshold (weakly periodic) — global minimum
    // of the CMND over the search range. Confidence will report this.
    let tau_est = tau_est.unwrap_or_else(|| {
        let mut best = tau_min;
        for t in (tau_min + 1)..=tau_max {
            if cmnd[t] < cmnd[best] {
                best = t;
            }
        }
        best
    });

    // Step 4: parabolic interpolation on the CMND for a sub-sample period.
    let refined_tau = parabolic_interpolation(cmnd, tau_est);
    if refined_tau <= 0.0 {
        return None;
    }

    Some(PitchEstimate {
        f0: sample_rate / refined_tau,
        confidence: (1.0 - cmnd[tau_est]).clamp(0.0, 1.0),
    })
}

/// YIN steps 2-4 with the CMND denominator (cumulative sum) computed on the CPU.
/// CPU reference path; also the GPU fallback.
pub fn yin_f0_from_diff(diff: &[f32], sample_rate: f32) -> Option<PitchEstimate> {
    let (tau_min, tau_max) = yin_bounds(diff.len(), sample_rate)?;

    // Step 2: cumulative mean normalized difference.
    //   d'(0) = 1,  d'(tau) = d(tau) / [ (1/tau) * sum_{j=1..tau} d(j) ].
    let mut cmnd = vec![1.0f32; tau_max + 1];
    let mut running = 0.0f32;
    for tau in 1..=tau_max {
        running += diff[tau];
        cmnd[tau] = if running > 0.0 {
            diff[tau] * tau as f32 / running
        } else {
            1.0
        };
    }
    yin_pick(&cmnd, tau_min, tau_max, sample_rate)
}

/// YIN steps 2-4 using a GPU-computed **inclusive prefix sum** of `diff` as the
/// CMND denominator. `cumsum[tau]` must equal `sum_{j=0..=tau} diff[j]`; since
/// `diff[0] == 0` that equals `sum_{j=1..tau} diff[j]`, exactly the YIN
/// denominator. This is the path where the cumulative sum runs as a parallel
/// prefix-sum on the GPU rather than a serial loop on the CPU.
pub fn yin_f0_from_diff_cumsum(
    diff: &[f32],
    cumsum: &[f32],
    sample_rate: f32,
) -> Option<PitchEstimate> {
    let len = diff.len().min(cumsum.len());
    let (tau_min, tau_max) = yin_bounds(len, sample_rate)?;

    let mut cmnd = vec![1.0f32; tau_max + 1];
    for tau in 1..=tau_max {
        let denom = cumsum[tau];
        cmnd[tau] = if denom > 0.0 {
            diff[tau] * tau as f32 / denom
        } else {
            1.0
        };
    }
    yin_pick(&cmnd, tau_min, tau_max, sample_rate)
}

/// YIN fundamental-frequency estimation (de Cheveigné & Kawahara, 2002),
/// CPU reference path. Computes the difference function on the CPU then shares
/// steps 2-4 with the GPU path via [`yin_f0_from_diff`]. Returns `None` only if
/// the frame is too short to analyze.
pub fn yin_pitch(samples: &[f32], sample_rate: f32) -> Option<PitchEstimate> {
    let n = samples.len();
    let w = YIN_WINDOW.min(n / TWO_USIZE);
    if w < YIN.min_window {
        return None;
    }
    // Largest lag we can evaluate without `x[j+tau]` leaving the frame, also
    // bounded by the lowest pitch we care about.
    let max_lag = ((sample_rate / YIN_F0_MIN) as usize).min(n - w);
    if max_lag < YIN.min_max_lag {
        return None;
    }
    let diff = yin_difference(samples, w, max_lag);
    yin_f0_from_diff(&diff, sample_rate)
}

/// Autocorrelation `r[0..=max_lag]` of `samples` (unnormalized).
pub fn autocorrelation(samples: &[f32], max_lag: usize) -> Vec<f32> {
    let n = samples.len();
    let mut r = vec![0.0f32; max_lag + 1];
    for (lag, r_lag) in r.iter_mut().enumerate() {
        let mut sum = 0.0f32;
        for i in lag..n {
            sum += samples[i] * samples[i - lag];
        }
        *r_lag = sum;
    }
    r
}

/// Windowed-sinc lowpass FIR taps (Hamming window), DC gain normalized to 1.
/// `cutoff_norm` is the cutoff in cycles/sample, in `(0, 0.5)`.
fn lowpass_fir(cutoff_norm: f32, num_taps: usize) -> Vec<f32> {
    let m = (num_taps - 1) as f32;
    let mut taps = vec![0.0f32; num_taps];
    let mut sum = 0.0f32;
    for (i, tap) in taps.iter_mut().enumerate() {
        let centered = i as f32 - m / TWO;
        let sinc = if centered.abs() < LPC.sinc_center_eps {
            TWO * cutoff_norm
        } else {
            (TAU * cutoff_norm * centered).sin() / (std::f32::consts::PI * centered)
        };
        let window = HAMMING_A0 - HAMMING_A1 * (TAU * i as f32 / m).cos();
        *tap = sinc * window;
        sum += *tap;
    }
    if sum.abs() > LPC.fir_gain_eps {
        for tap in taps.iter_mut() {
            *tap /= sum;
        }
    }
    taps
}

/// Anti-aliased integer decimation by `factor`. Applies a windowed-sinc lowpass
/// at ~90% of the new Nyquist before subsampling, so the formant band is kept
/// while energy above the new Nyquist (which would fold back) is suppressed.
pub fn decimate(samples: &[f32], factor: usize) -> Vec<f32> {
    if factor <= 1 {
        return samples.to_vec();
    }
    let cutoff = LPC.decimation_cutoff_of_nyquist / factor as f32;
    let taps = lowpass_fir(cutoff, LPC.decimation_fir_taps);
    let half = (taps.len() / TWO_USIZE) as isize;
    let n = samples.len() as isize;

    let mut out = Vec::with_capacity(samples.len() / factor + 1);
    let mut i = 0isize;
    while i < n {
        let mut acc = 0.0f32;
        for (k, &t) in taps.iter().enumerate() {
            let idx = i + k as isize - half;
            if idx >= 0 && idx < n {
                acc += t * samples[idx as usize];
            }
        }
        out.push(acc);
        i += factor as isize;
    }
    out
}

/// Full LPC analysis front-end: pre-emphasis → Hamming window → autocorrelation
/// → Levinson-Durbin. Returns the prediction-error polynomial `A(z)` of length
/// `order + 1` with `a[0] == 1.0`; its roots are the formant poles.
pub fn lpc_coefficients(samples: &[f32], order: usize, preemph: f32) -> Vec<f32> {
    let len = samples.len();
    if len < order + 1 {
        let mut a = vec![0.0f32; order + 1];
        a[0] = 1.0;
        return a;
    }

    // Pre-emphasis: y[n] = x[n] - preemph * x[n-1]. Flattens the glottal
    // -12 dB/oct tilt so the LPC fit spends its poles on formants, not slope.
    let mut x = vec![0.0f32; len];
    x[0] = samples[0];
    for n in 1..len {
        x[n] = samples[n] - preemph * samples[n - 1];
    }

    // Hamming window to tame autocorrelation edge effects.
    let m = (len - 1) as f32;
    for (n, v) in x.iter_mut().enumerate() {
        let w = HAMMING_A0 - HAMMING_A1 * (TAU * n as f32 / m).cos();
        *v *= w;
    }

    let r = autocorrelation(&x, order);
    if r[0] <= LPC.silence_energy {
        // Silent frame — return the trivial all-pass polynomial.
        let mut a = vec![0.0f32; order + 1];
        a[0] = 1.0;
        return a;
    }

    levinson_durbin(&r, order)
}

// ── Harmonic series ──────────────────────────────────────────────────────────

/// Amplitudes of the first [`MAX_PARTIALS`] harmonics of `f0`, measured by a
/// Hann-windowed Goertzel evaluation of the DFT at exactly k·f0. No FFT: f0 is
/// already known from YIN, so the analysis bins can sit on the harmonics
/// themselves instead of a fixed grid.
///
/// Returns linear peak amplitudes (a full-scale sine measures ≈ 1.0 at H1).
/// Harmonics at or above Nyquist — and everything when `f0` or `sample_rate`
/// is non-positive or non-finite, or the frame is degenerate — are 0.
pub fn harmonic_amplitudes(samples: &[f32], sample_rate: f32, f0: f32) -> [f32; MAX_PARTIALS] {
    let mut amps = [0.0f32; MAX_PARTIALS];
    let n = samples.len();
    // `!(x > 0.0)` (not `x <= 0.0`): NaN fails every comparison, so the
    // negated form routes NaN to the zero return instead of a NaN Goertzel.
    if n < HARMONICS.min_frame_samples
        || !(f0.is_finite() && f0 > 0.0)
        || !(sample_rate.is_finite() && sample_rate > 0.0)
    {
        return amps;
    }

    // Hann window; its coherent gain (Σw / 2) normalizes the DFT magnitude
    // back to sinusoid peak amplitude.
    let m = (n - 1) as f32;
    let mut windowed = vec![0.0f32; n];
    let mut wsum = 0.0f32;
    for (i, (w, &x)) in windowed.iter_mut().zip(samples).enumerate() {
        let win = HANN_A0 - HANN_A0 * (TAU * i as f32 / m).cos();
        *w = x * win;
        wsum += win;
    }
    let norm = TWO / wsum;

    let nyquist = sample_rate / TWO;
    for (k, amp) in amps.iter_mut().enumerate() {
        let freq = (k + 1) as f32 * f0;
        if freq >= nyquist {
            break;
        }
        // Goertzel recurrence at `freq`.
        let omega = TAU * freq / sample_rate;
        let coeff = TWO * omega.cos();
        let (mut s1, mut s2) = (0.0f32, 0.0f32);
        for &x in &windowed {
            let s0 = x + coeff * s1 - s2;
            s2 = s1;
            s1 = s0;
        }
        let power = (s1 * s1 + s2 * s2 - coeff * s1 * s2).max(0.0);
        *amp = power.sqrt() * norm;
    }
    amps
}

/// Harmonics-to-noise ratio in dB, Boersma/Praat style: the normalized
/// autocorrelation `r = ACF(T)/ACF(0)` at the pitch period `T = sr/f0`
/// (parabolic-refined around the nearest integer lag, per-count normalized so
/// the shrinking overlap doesn't bias `r` down), mapped through
/// `10·log10(r / (1 − r))` and clamped to ±40 dB.
///
/// This reads a couple of dB below Praat's windowed implementation — treat it
/// as a relative training signal, not a clinical instrument.
pub fn hnr_db(samples: &[f32], sample_rate: f32, f0: f32) -> Option<f32> {
    let n = samples.len();
    if f0 <= 0.0 || sample_rate <= 0.0 || n < HNR.min_frame_samples {
        return None;
    }
    let lag = sample_rate / f0;
    let lag_i = lag.round() as usize;
    if lag_i < HNR.min_period_samples || lag_i + HNR.period_margin_samples >= n {
        return None;
    }

    // Per-count-normalized autocorrelation at one lag.
    let r_at = |l: usize| -> f32 {
        let sum: f32 = samples[..n - l]
            .iter()
            .zip(&samples[l..])
            .map(|(a, b)| a * b)
            .sum();
        sum / (n - l) as f32
    };
    let r0 = r_at(0);
    if r0 <= HNR.energy_eps {
        return None;
    }

    // Parabolic vertex through the three lags around the period gives the
    // true (non-integer-lag) correlation peak height.
    let (y0, y1, y2) = (r_at(lag_i - 1), r_at(lag_i), r_at(lag_i + 1));
    let denom = y0 - TWO * y1 + y2;
    let peak = if denom.abs() < HNR.parabolic_flat_eps {
        y1
    } else {
        y1 - (y0 - y2) * (y0 - y2) / (PARABOLIC_PEAK_DENOM * denom)
    };

    let r = (peak / r0).clamp(-HNR.r_clamp, HNR.r_clamp);
    if r <= 0.0 {
        return Some(-HNR.db_limit);
    }
    Some((DB_PER_DECADE_POWER * (r / (1.0 - r)).log10()).clamp(-HNR.db_limit, HNR.db_limit))
}

/// H1–H2 in dB: the level difference between the first two harmonics, the
/// standard phonation-type measure (large = breathy/flowy, small or negative
/// = pressed). `None` when either harmonic sits below the −48 dB relative
/// floor (same floor as the ladder display).
pub fn h1_h2_db(amps: &[f32]) -> Option<f32> {
    let (a1, a2) = (*amps.first()?, *amps.get(1)?);
    let max = amps.iter().cloned().fold(0.0f32, f32::max);
    let floor = max * DB_LOG_BASE.powf(TIMBRE.relative_floor_db / DB_PER_DECADE_AMPLITUDE);
    (max > TIMBRE.amp_eps && a1 > floor && a2 > floor)
        .then(|| DB_PER_DECADE_AMPLITUDE * (a1 / a2).log10())
}

/// Cycle-to-cycle perturbation measures from one analysis frame.
#[derive(Debug, Clone, Copy)]
pub struct CyclePerturbation {
    /// Jitter (local), %: mean |T_i − T_{i+1}| / mean T · 100.
    pub jitter_pct: f32,
    /// Shimmer (local), dB: mean |20·log10(A_{i+1}/A_i)| over adjacent cycles.
    pub shimmer_db: f32,
}

/// Cycle marks by peak-picking: one positive peak per pitch period, each
/// refined parabolically for sub-sample precision (0.5 % jitter at a
/// 200-sample period is a single sample — integer marks would drown it).
///
/// Returns `None` unless the frame yields ≥ 5 peaks whose spacings all sit
/// within ±30 % of the YIN period — a tracking failure reads as "no
/// measurement", never as a wild number. The ≥ 5-cycle requirement puts a
/// floor on measurable pitch: f0 ≥ ~5·sr/frame (≈ 112 Hz @ 44.1 k / 2048).
pub fn cycle_perturbation(samples: &[f32], sample_rate: f32, f0: f32) -> Option<CyclePerturbation> {
    let n = samples.len();
    if f0 <= 0.0 || sample_rate <= 0.0 || n < PERTURB.min_frame_samples {
        return None;
    }
    let period = sample_rate / f0;
    if period < PERTURB.min_period_samples {
        return None;
    }

    // Refined (position, height) of the maximum in [lo, hi).
    let peak_in = |lo: usize, hi: usize| -> Option<(f32, f32)> {
        let hi = hi.min(n);
        if lo + 1 >= hi {
            return None;
        }
        let mut best = lo;
        for i in lo..hi {
            if samples[i] > samples[best] {
                best = i;
            }
        }
        if best == 0 || best + 1 >= n {
            return Some((best as f32, samples[best]));
        }
        let (y0, y1, y2) = (samples[best - 1], samples[best], samples[best + 1]);
        let denom = y0 - TWO * y1 + y2;
        if denom.abs() < PERTURB.parabolic_flat_eps {
            return Some((best as f32, y1));
        }
        let delta = ((y0 - y2) / (TWO * denom)).clamp(-HALF, HALF);
        let height = y1 - (y0 - y2) * delta / PARABOLIC_HEIGHT_DENOM;
        Some((best as f32 + delta, height))
    };

    // First mark: strongest sample in the first 1.5 periods; then march
    // forward one period at a time inside a ±30 % search window.
    let mut marks: Vec<(f32, f32)> = Vec::new();
    let first = peak_in(0, (PERTURB.first_search_periods * period) as usize)?;
    marks.push(first);
    loop {
        let prev = marks.last().unwrap().0;
        let lo = (prev + PERTURB.search_window_lo * period) as usize;
        let hi = (prev + PERTURB.search_window_hi * period).ceil() as usize;
        if hi > n {
            break;
        }
        match peak_in(lo, hi) {
            Some(p) => marks.push(p),
            None => break,
        }
    }
    if marks.len() < PERTURB.min_cycles {
        return None;
    }

    let periods: Vec<f32> = marks
        .windows(ADJACENT_PAIR)
        .map(|w| w[1].0 - w[0].0)
        .collect();
    // Sanity: every interval near the YIN period, else the marks double-fired
    // or skipped (strong formants can do this) and the numbers would be junk.
    if periods
        .iter()
        .any(|&t| t < PERTURB.search_window_lo * period || t > PERTURB.search_window_hi * period)
    {
        return None;
    }

    let mean_t = periods.iter().sum::<f32>() / periods.len() as f32;
    let jitter_pct = periods
        .windows(ADJACENT_PAIR)
        .map(|w| (w[0] - w[1]).abs())
        .sum::<f32>()
        / (periods.len() - 1) as f32
        / mean_t
        * PERCENT;
    // Plausibility ceiling: aperiodic input yields peaks scattered uniformly
    // inside the search windows (~20-30 % "jitter") — the ±30 % interval
    // check can't catch that by construction. Severe pathology is ~2-3 %;
    // beyond 5 % the marks aren't tracking real glottal cycles.
    if jitter_pct > PERTURB.max_jitter_pct {
        return None;
    }

    let amps: Vec<f32> = marks.iter().map(|&(_, a)| a).collect();
    if amps.iter().any(|&a| a <= PERTURB.amp_eps) {
        return None;
    }
    let shimmer_db = amps
        .windows(ADJACENT_PAIR)
        .map(|w| (DB_PER_DECADE_AMPLITUDE * (w[1] / w[0]).log10()).abs())
        .sum::<f32>()
        / (amps.len() - 1) as f32;

    Some(CyclePerturbation {
        jitter_pct,
        shimmer_db,
    })
}

/// Cepstral peak prominence in dB (Hillenbrand-style): Hann-windowed FFT →
/// dB power spectrum → FFT again → dB cepstrum; the peak in the 60–500 Hz
/// quefrency band is measured against a linear regression of the cepstrum
/// over the analysis range. High CPP = strong, clean periodicity; breathy or
/// dysphonic voices read low.
///
/// Absolute values depend on the recipe (window, normalization) — treat as an
/// internally-consistent relative measure, like our HNR.
pub fn cpp_db(samples: &[f32], sample_rate: f32) -> Option<f32> {
    use rustfft::num_complex::Complex;

    let n = samples.len();
    if n < CPP.min_frame_samples || sample_rate <= 0.0 {
        return None;
    }
    if samples.iter().map(|s| s * s).sum::<f32>() < CPP.silence_energy {
        return None; // silence
    }

    let fft = FFT_PLANNER.with(|p| p.borrow_mut().plan_fft_forward(n));

    // Hann-windowed spectrum.
    let m = (n - 1) as f32;
    let mut buf: Vec<Complex<f32>> = samples
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let w = HANN_A0 - HANN_A0 * (TAU * i as f32 / m).cos();
            Complex::new(x * w, 0.0)
        })
        .collect();
    fft.process(&mut buf);

    // dB power spectrum is real and even, so its FFT is the real cepstrum.
    let mut logspec: Vec<Complex<f32>> = buf
        .iter()
        .map(|c| {
            Complex::new(
                DB_PER_DECADE_POWER * (c.norm_sqr() + CPP.log_floor).log10(),
                0.0,
            )
        })
        .collect();
    fft.process(&mut logspec);
    let ceps_db: Vec<f32> = logspec
        .iter()
        .map(|c| {
            DB_PER_DECADE_POWER * (c.norm_sqr() / (n as f32) / (n as f32) + CPP.log_floor).log10()
        })
        .collect();

    // Voice pitch quefrency band: 60–500 Hz. If the frame is too short to
    // reach the 60 Hz end at this sample rate (e.g. 2048 samples at 96 kHz),
    // refuse rather than search a truncated band and report the prominence of
    // some spurious higher-pitch bin.
    let q_lo = (sample_rate / CPP.f0_band_hi_hz).ceil() as usize;
    let q_hi = (sample_rate / CPP.f0_band_lo_hz).floor() as usize;
    if q_hi > n / TWO_USIZE - 1 {
        return None;
    }
    if q_lo + CPP.min_band_bins >= q_hi {
        return None;
    }

    // Regression baseline over the full analyzed quefrency range.
    let range = q_lo..n / TWO_USIZE;
    let cnt = range.len() as f32;
    let mx = range.clone().map(|q| q as f32).sum::<f32>() / cnt;
    let my = range.clone().map(|q| ceps_db[q]).sum::<f32>() / cnt;
    let (mut num, mut den) = (0.0f32, 0.0f32);
    for q in range {
        let dx = q as f32 - mx;
        num += dx * (ceps_db[q] - my);
        den += dx * dx;
    }
    if den < CPP.regression_eps {
        return None;
    }
    let slope = num / den;

    let q_pk = (q_lo..=q_hi)
        .max_by(|&a, &b| {
            ceps_db[a]
                .partial_cmp(&ceps_db[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(q_lo);
    Some(ceps_db[q_pk] - (my + slope * (q_pk as f32 - mx)))
}

/// Spectral centroid in Hz: the magnitude-weighted mean frequency of the full
/// spectrum over 80 Hz–min(8000, Nyquist), not just the harmonic ladder — this
/// deliberately includes inter-harmonic noise energy, which is what makes it a
/// meaningful "brightness" measure distinct from the tilt of the harmonics
/// alone. Weighting is by |X(f)| (amplitude), not |X(f)|² (power): a power
/// weighting is an equally valid statistic but reads higher for bright voices,
/// and the `brightness_class` thresholds were tuned against this
/// magnitude-weighted form — keep them in sync if this ever changes.
pub fn spectral_centroid(samples: &[f32], sample_rate: f32) -> Option<f32> {
    use rustfft::num_complex::Complex;

    let n = samples.len();
    if n < CENTROID.min_frame_samples || sample_rate <= 0.0 {
        return None;
    }
    if samples.iter().map(|s| s * s).sum::<f32>() < CENTROID.silence_energy {
        return None; // silence
    }

    let fft = FFT_PLANNER.with(|p| p.borrow_mut().plan_fft_forward(n));

    let m = (n - 1) as f32;
    let mut buf: Vec<Complex<f32>> = samples
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let w = HANN_A0 - HANN_A0 * (TAU * i as f32 / m).cos();
            Complex::new(x * w, 0.0)
        })
        .collect();
    fft.process(&mut buf);

    let bin_hz = sample_rate / n as f32;
    let lo_bin = (CENTROID.band_lo_hz / bin_hz).ceil() as usize;
    let hi_bin = ((CENTROID.band_hi_hz.min(sample_rate / TWO)) / bin_hz).floor() as usize;
    let hi_bin = hi_bin.min(n / TWO_USIZE);
    if lo_bin >= hi_bin {
        return None;
    }

    let (mut num, mut den) = (0.0f32, 0.0f32);
    for (bin, c) in buf.iter().enumerate().take(hi_bin).skip(lo_bin) {
        let amp = c.norm();
        let freq = bin as f32 * bin_hz;
        num += freq * amp;
        den += amp;
    }
    (den > CENTROID.magnitude_eps).then(|| num / den)
}

// ── Musical mapping & timbre metrics ─────────────────────────────────────────

/// A 12-TET note name for a frequency: pitch class, octave, and signed cents
/// offset from the nearest equal-tempered pitch (A4 = 440 Hz).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Note {
    pub name: &'static str,
    pub octave: i32,
    pub cents: f32,
}

const NOTE_NAMES: [&str; SEMITONES_PER_OCTAVE_USIZE] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Nearest 12-TET note to `hz` (A4 = 440). `None` for non-positive/non-finite
/// input. Cents are in (-50, +50].
pub fn freq_to_note(hz: f32) -> Option<Note> {
    if hz <= 0.0 || !hz.is_finite() {
        return None;
    }
    let midi = MIDI_A4 + SEMITONES_PER_OCTAVE * (hz / TUNING.a4_hz).log2();
    let nearest = midi.round();
    let idx = (nearest as i32).rem_euclid(SEMITONES_PER_OCTAVE_I32) as usize;
    Some(Note {
        name: NOTE_NAMES[idx],
        octave: (nearest as i32).div_euclid(SEMITONES_PER_OCTAVE_I32) - 1,
        cents: (midi - nearest) * CENTS_PER_SEMITONE,
    })
}

/// Signed distance in semitones from `freq` to `reference` (positive = `freq`
/// above `reference`). `None` for non-positive/non-finite input.
///
/// This is the general form of Bozeman's acoustic-registration event: calling
/// `semitones_from(2.0 * f0, f1)` gives the "F1/H2 crossing" distance — the
/// second harmonic's position relative to the first formant. Negative while
/// H2 sits below F1 (open timbre, *voce aperta*), crossing zero at the
/// acoustic passaggio event ("turning over"), positive once H2 has cleared F1
/// (close timbre, *voce chiusa*). The same helper gives the treble-voice
/// analog via `semitones_from(f0, f1)` (F1/H1 tracking).
///
/// No longer drives the turnover gauge — that now reads the A2/A1 harmonic
/// ratio directly ([`a2_a1_db`]), since an F1 estimate is exactly what LPC
/// cannot provide in the passaggio. Kept as a general pitch-distance helper.
#[allow(dead_code)]
pub fn semitones_from(freq: f32, reference: f32) -> Option<f32> {
    if freq <= 0.0 || reference <= 0.0 || !freq.is_finite() || !reference.is_finite() {
        return None;
    }
    Some(SEMITONES_PER_OCTAVE * (freq / reference).log2())
}

/// Spectral tilt in dB/octave: least-squares slope of harmonic level (dB)
/// against log2(harmonic number). A sawtooth's 1/k rolloff measures ≈ −6
/// dB/oct; a brighter, more pressed voice is shallower (closer to 0).
pub fn spectral_tilt_db_per_octave(amps: &[f32]) -> Option<f32> {
    let max = amps.iter().cloned().fold(0.0f32, f32::max);
    if max <= TIMBRE.amp_eps {
        return None;
    }
    // Only fit partials within 48 dB of the strongest one: bins below that are
    // numerical noise whose ~−100 dB levels would drag the slope toward −∞.
    let floor = max * DB_LOG_BASE.powf(TIMBRE.relative_floor_db / DB_PER_DECADE_AMPLITUDE);
    let pts: Vec<(f32, f32)> = amps
        .iter()
        .enumerate()
        .filter(|&(_, &a)| a > floor)
        .map(|(k, &a)| (((k + 1) as f32).log2(), DB_PER_DECADE_AMPLITUDE * a.log10()))
        .collect();
    if pts.len() < TIMBRE.tilt_min_points {
        return None;
    }
    let n = pts.len() as f32;
    let (sx, sy) = pts
        .iter()
        .fold((0.0f32, 0.0f32), |(sx, sy), (x, y)| (sx + x, sy + y));
    let (mx, my) = (sx / n, sy / n);
    let (mut num, mut den) = (0.0f32, 0.0f32);
    for (x, y) in &pts {
        num += (x - mx) * (y - my);
        den += (x - mx) * (x - mx);
    }
    (den > TIMBRE.tilt_regression_eps).then(|| num / den)
}

/// Even/odd harmonic energy balance in dB over H2..=H16. H1 is excluded — it
/// has no parity partner and would swamp the comparison. Positive = even-heavy
/// (fuller, rounder); negative = odd-heavy (hollower, clarinet-like).
pub fn even_odd_balance_db(amps: &[f32]) -> Option<f32> {
    let (mut even, mut odd) = (0.0f32, 0.0f32);
    for (k, &a) in amps
        .iter()
        .enumerate()
        .take(TIMBRE.even_odd_partials)
        .skip(1)
    {
        let e = a * a;
        if (k + 1).is_multiple_of(TWO_USIZE) {
            even += e;
        } else {
            odd += e;
        }
    }
    (even > 0.0 && odd > 0.0).then(|| DB_PER_DECADE_POWER * (even / odd).log10())
}

/// Percentage of harmonic energy in the singer's-formant band (2.8–3.4 kHz) —
/// the resonance cluster that lets a trained voice project over an ensemble.
pub fn singers_formant_pct(amps: &[f32], f0: f32) -> Option<f32> {
    // Negated form so NaN/Inf f0 reads as unmeasurable (None), never Some(0.0).
    if !(f0.is_finite() && f0 > 0.0) {
        return None;
    }
    let (mut band, mut total) = (0.0f32, 0.0f32);
    for (k, &a) in amps.iter().enumerate() {
        let e = a * a;
        total += e;
        if (TIMBRE.singers_formant_lo_hz..=TIMBRE.singers_formant_hi_hz)
            .contains(&((k + 1) as f32 * f0))
        {
            band += e;
        }
    }
    (total > TIMBRE.energy_eps).then(|| band / total * PERCENT)
}

/// Rough vocal-range classification from mean fundamental frequency. Ported
/// from the Personal Harmonic Identifier prototype's thresholds; a coarse
/// label, not a diagnosis.
pub fn voice_class(mean_f0: f32) -> &'static str {
    if mean_f0 <= 0.0 || !mean_f0.is_finite() {
        return "—";
    }
    if mean_f0 < CLASS.bass_max_hz {
        "Bass"
    } else if mean_f0 < CLASS.baritone_max_hz {
        "Baritone"
    } else if mean_f0 < CLASS.tenor_max_hz {
        "Tenor"
    } else if mean_f0 < CLASS.alto_max_hz {
        "Alto"
    } else if mean_f0 < CLASS.mezzo_max_hz {
        "Mezzo-Soprano"
    } else {
        "Soprano"
    }
}

/// Brightness classification from spectral centroid. Same thresholds as the
/// Personal Harmonic Identifier prototype.
pub fn brightness_class(centroid_hz: f32) -> &'static str {
    if centroid_hz <= 0.0 || !centroid_hz.is_finite() {
        return "—";
    }
    if centroid_hz < CLASS.dark_max_hz {
        "Dark"
    } else if centroid_hz < CLASS.warm_max_hz {
        "Warm"
    } else if centroid_hz < CLASS.balanced_max_hz {
        "Balanced"
    } else if centroid_hz < CLASS.bright_max_hz {
        "Bright"
    } else {
        "Brilliant"
    }
}

/// Plain-language timbre description from the harmonic profile's shape.
///
/// Adapted (not identical) from the source prototype: that version compared
/// a *linear* odd/even amplitude-sum ratio against 1.9; ours reuses
/// [`even_odd_balance_db`]'s power-ratio dB value instead of recomputing a
/// separate linear ratio, with an independently chosen dB threshold for the
/// same "clearly hollow" judgment call. `rel_amps` are amplitudes relative to
/// the strongest partial (as displayed on the harmonic ladder); only
/// H2..H16 count toward the "strong harmonic" tally, matching the prototype.
pub fn timbre_description(rel_amps: &[f32], even_odd_db: Option<f32>) -> &'static str {
    if let Some(db) = even_odd_db
        && db < TIMBRE.hollow_even_odd_db
    {
        return "Hollow · odd-dominant";
    }
    let strong = rel_amps
        .iter()
        .skip(1)
        .filter(|&&a| a > TIMBRE.strong_partial_rel_amp)
        .count();
    if strong >= TIMBRE.rich_min_strong {
        "Rich · complex"
    } else if strong <= TIMBRE.pure_max_strong {
        "Pure · flute-like"
    } else {
        "Balanced"
    }
}

// ── Ambient noise floor & SNR gating ─────────────────────────────────────────

/// Minimum frame-over-floor SNR (dB) for a YIN-voiced frame to count as
/// voice at all. Below this, whatever periodicity YIN found is riding on
/// ambience loud enough to corrupt every harmonic measure taken from the
/// frame, so the frame is demoted to unvoiced. Engineering floor; the
/// measurement-grade threshold below is the cited one.
pub const VOICED_MIN_SNR_DB: f32 = VOICING.voiced_min_snr_db;

/// Minimum SNR (dB) for a frame to contribute *identity* data (voiceprint
/// formants, VTL). The ASHA instrumental-assessment protocol (Patel et al.
/// 2018, AJSLP 27:887) specifies at least 30 dB signal-to-noise for
/// measurement-grade voice recording; frames below it may still display,
/// but must not become part of who the app thinks the singer is.
pub const IDENTITY_MIN_SNR_DB: f32 = VOICING.identity_min_snr_db;

/// Ring capacity for ambient-floor tracking, in unvoiced frames (~6 s of
/// non-phonation time at the ~21.5 Hz frame rate).
const NOISE_RING_LEN: usize = NOISE.ring_len;

/// Minimum unvoiced frames observed before the floor is trusted (~1.5 s).
/// Until then `snr_db` reports `None` and nothing is gated — a quiet room
/// must not lock the app out while the tracker warms up.
const NOISE_RING_MIN: usize = NOISE.ring_min;

/// Percentile of the ring taken as the floor. Low, so breaths, consonants
/// and other loud-but-unvoiced moments of real speech do not drag the
/// "ambient" estimate up toward the voice itself.
const NOISE_FLOOR_PERCENTILE: f32 = NOISE.floor_percentile;

/// Ambient-noise floor tracker.
///
/// Learns the room from frames the pitch detector calls *unvoiced* — never
/// from voiced frames, so a long sustained note cannot teach the tracker
/// that singing is "ambience" and then gate the singer off mid-phrase. The
/// floor is a low percentile of recent unvoiced frame RMS, which rides out
/// the loud unvoiced moments (breaths, fricatives) that are part of real
/// phonation.
///
/// Honest limitation, stated once here for every consumer: this separates
/// voice from *aperiodic* ambience (fans, HVAC, traffic). A loud periodic
/// source — music, a TV voice — passes YIN and IS the analyzed signal; no
/// floor tracker can tell the app which periodic source is the user. The
/// app analyzes the dominant periodic source at the microphone, and its UI
/// copy must never claim otherwise.
pub struct NoiseFloor {
    ring: [f32; NOISE_RING_LEN],
    len: usize,
    head: usize,
}

impl Default for NoiseFloor {
    fn default() -> Self {
        Self::new()
    }
}

impl NoiseFloor {
    pub fn new() -> Self {
        Self {
            ring: [0.0; NOISE_RING_LEN],
            len: 0,
            head: 0,
        }
    }

    /// Seed the tracker from a calibration pass: the whole ring takes the
    /// measured ambient level, so SNR gating is available immediately with
    /// no passive warmup, and later unvoiced frames adapt it as usual.
    pub fn seed(&mut self, ambient_rms: f32) {
        if !(ambient_rms.is_finite() && ambient_rms >= 0.0) {
            return;
        }
        self.ring = [ambient_rms; NOISE_RING_LEN];
        self.len = NOISE_RING_LEN;
        self.head = 0;
    }

    /// Feed one unvoiced frame's RMS. Voiced frames must NOT be pushed.
    pub fn push_unvoiced(&mut self, rms: f32) {
        if !(rms.is_finite() && rms >= 0.0) {
            return;
        }
        self.ring[self.head] = rms;
        self.head = (self.head + 1) % NOISE_RING_LEN;
        self.len = (self.len + 1).min(NOISE_RING_LEN);
    }

    /// Current floor estimate (linear RMS), once enough ambience is seen.
    pub fn floor(&self) -> Option<f32> {
        if self.len < NOISE_RING_MIN {
            return None;
        }
        let mut v: Vec<f32> = self.ring[..self.len].to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((self.len as f32 * NOISE_FLOOR_PERCENTILE) as usize).min(self.len - 1);
        Some(v[idx])
    }

    /// SNR of a frame against the learned floor, dB. `None` while warming
    /// up. A silent floor (true digital silence) reports a large finite SNR
    /// rather than infinity.
    pub fn snr_db(&self, frame_rms: f32) -> Option<f32> {
        let floor = self.floor()?;
        if !(frame_rms.is_finite() && frame_rms > 0.0) {
            return Some(0.0);
        }
        Some(DB_PER_DECADE_AMPLITUDE * (frame_rms / floor.max(NOISE.floor_min_rms)).log10())
    }
}

/// Frames in one room-calibration pass (~4.6 s at the ~21.5 Hz frame rate):
/// long enough for a stable spectrum-level and hum-pitch estimate, short
/// enough that "keep quiet for five seconds" is a reasonable ask. The
/// clinical practice this mirrors — record the room before the voice — is
/// part of the same ASHA protocol as the 30 dB SNR requirement.
pub const CALIB_FRAMES: u32 = CALIB.frames;

/// Fractional f0 tolerance for matching a live detection against the
/// calibrated interferer (±4%, roughly ±:two thirds of a semitone — wide
/// enough for hum drift, far narrower than typical vibrato excursions).
const INTERFERER_F0_TOL: f32 = CALIB.interferer_f0_tol;

/// A frame whose RMS exceeds the calibrated interferer's level by this many
/// dB is treated as the singer, even at the interferer's pitch: the gate
/// must never forbid singing a note the refrigerator also hums.
const INTERFERER_LEVEL_MARGIN_DB: f32 = CALIB.interferer_level_margin_db;

/// Voiced fraction of the calibration window above which an *unstable*
/// pitch means a voice (or TV) was talking during calibration — fail rather
/// than fingerprint speech as "the room".
const CALIB_VOICED_FAIL_FRACTION: f32 = CALIB.voiced_fail_fraction;
/// Voiced fraction above which a *stable* pitch is a real periodic
/// interferer worth fingerprinting (mains hum, fan blade-pass).
const CALIB_INTERFERER_MIN_FRACTION: f32 = CALIB.interferer_min_fraction;
/// Relative f0 standard deviation separating machine hum (very stable)
/// from anything vocal (wanders far more, even when trying not to).
const CALIB_STABLE_REL_STD: f32 = CALIB.stable_rel_std;

/// A stationary periodic interferer fingerprinted during calibration: the
/// pitch YIN keeps finding in the "silent" room, and how loud it is.
#[derive(Clone, Copy, Debug)]
pub struct Interferer {
    pub f0_hz: f32,
    pub rms: f32,
}

/// One room-calibration result: the ambient floor, and the periodic
/// interferer if the room has one.
#[derive(Clone, Copy, Debug)]
pub struct RoomCalibration {
    /// Median frame RMS over the calibration window — the room's level with
    /// everything in it (fan, hum, HVAC) running.
    pub ambient_rms: f32,
    pub interferer: Option<Interferer>,
}

/// Why a calibration pass was rejected.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CalibrationFailure {
    /// Periodicity with a wandering pitch — someone was talking or singing.
    /// Fingerprinting that as "the room" would teach the app to reject the
    /// user, so the pass is discarded instead.
    VoiceDetected,
    /// Not enough frames accumulated (engine restarted mid-pass).
    TooShort,
}

/// Accumulates one calibration pass frame by frame on the analysis thread.
///
/// What this can and cannot learn, stated once for every consumer: a
/// *stationary* source — fan, mains hum, HVAC — has a spectrum and pitch
/// now that predict its spectrum and pitch later, so fingerprinting works.
/// A *non-stationary* periodic source (TV, music) does not; calibrating
/// with one playing still raises the floor honestly, but no fingerprint
/// taken now can identify what it plays next. UI copy must not promise TV
/// rejection.
#[derive(Default)]
pub struct RoomCalibrator {
    rms: Vec<f32>,
    voiced_f0: Vec<f32>,
    frames: u32,
}

impl RoomCalibrator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed one frame: its RMS, and its YIN pitch if YIN called it periodic
    /// (pre-SNR-gate — calibration wants to see exactly what YIN sees).
    pub fn push(&mut self, rms: f32, yin_f0: Option<f32>) {
        if rms.is_finite() && rms >= 0.0 {
            self.rms.push(rms);
        }
        if let Some(f0) = yin_f0
            && f0.is_finite()
            && f0 > 0.0
        {
            self.voiced_f0.push(f0);
        }
        self.frames += 1;
    }

    /// Finalizes the pass. Stable periodicity becomes a fingerprinted
    /// interferer; unstable periodicity fails the pass as a voice.
    pub fn finish(&self) -> Result<RoomCalibration, CalibrationFailure> {
        if self.frames < CALIB_FRAMES / CALIB.too_short_divisor || self.rms.is_empty() {
            return Err(CalibrationFailure::TooShort);
        }
        let mut sorted = self.rms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let ambient_rms = sorted[sorted.len() / TWO_USIZE];

        let voiced_frac = self.voiced_f0.len() as f32 / self.frames as f32;
        let interferer = if self.voiced_f0.is_empty() {
            None
        } else {
            let n = self.voiced_f0.len() as f32;
            let mean = self.voiced_f0.iter().sum::<f32>() / n;
            let var = self
                .voiced_f0
                .iter()
                .map(|f| (f - mean) * (f - mean))
                .sum::<f32>()
                / n;
            let rel_std = var.sqrt() / mean.max(CALIB.rel_std_min_mean_hz);
            if rel_std < CALIB_STABLE_REL_STD {
                // Machine-stable pitch: a fingerprintable hum, if present
                // often enough to matter.
                (voiced_frac >= CALIB_INTERFERER_MIN_FRACTION).then_some(Interferer {
                    f0_hz: mean,
                    rms: ambient_rms,
                })
            } else if voiced_frac > CALIB_VOICED_FAIL_FRACTION {
                return Err(CalibrationFailure::VoiceDetected);
            } else {
                None
            }
        };

        Ok(RoomCalibration {
            ambient_rms,
            interferer,
        })
    }
}

/// Whether a live YIN detection matches the calibrated interferer: pitch
/// within tolerance of the fingerprinted hum, at a level the hum accounts
/// for. A frame much louder than the calibrated hum is the singer — even on
/// the hum's exact pitch.
pub fn interferer_match(f0_hz: f32, frame_rms: f32, interferer: &Interferer) -> bool {
    if !(f0_hz.is_finite() && f0_hz > 0.0) {
        return false;
    }
    let pitch_close = (f0_hz - interferer.f0_hz).abs() <= INTERFERER_F0_TOL * interferer.f0_hz;
    if !pitch_close {
        return false;
    }
    let margin = DB_LOG_BASE.powf(INTERFERER_LEVEL_MARGIN_DB / DB_PER_DECADE_AMPLITUDE);
    frame_rms <= interferer.rms.max(CALIB.interferer_min_rms) * margin
}

/// RMS of one analysis frame.
pub fn frame_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|x| x * x).sum::<f32>() / samples.len() as f32).sqrt()
}

// ── Formant reliability gating ───────────────────────────────────────────────

/// Above this f0, formant estimates are excluded from identity features (the
/// voiceprint). Chen, Whalen & Shadle (2019, JASA-EL 145:EL360) synthesized
/// 470,000 vowels with known formants: above ~200 Hz the LPC-measured F1's
/// variability equals that of the nearest harmonic *regardless of the true
/// F1* — the estimate re-encodes f0, and averaging over a capture does not
/// remove the bias ("even a thousand tokens were not sufficient").
pub const FORMANT_F0_IDENTITY_MAX_HZ: f32 = FORMANT.identity_f0_max_hz;

/// Above this f0, formant estimates are not stored or displayed at all.
/// Monsen & Engebretson (1983, JSHR 26:89): "the accuracy of both methods
/// decreases greatly when fundamental frequency is 350 Hz or greater" — the
/// threshold the singing-voice literature (Joliveau, Smith & Wolfe 2004,
/// JASA 116:2434) adopts, en route to "essentially impossible" above 500 Hz.
pub const FORMANT_F0_DISPLAY_MAX_HZ: f32 = FORMANT.display_f0_max_hz;

/// Half-width of the harmonic-proximity suspect band, as a fraction of the
/// harmonic's frequency. Boë, Sawallis, Badin & Schwartz (2023, Int. J.
/// Primatology 44:1046) flag F1 within ±15% of f0 (and F2 within ±15% of
/// 2·f0) as likely harmonic/formant confusions; Grawunder et al. (2023,
/// Phil. Trans. R. Soc. B 378:20230319) adopted the same exclusion in a
/// formal correction.
pub const FORMANT_HARMONIC_SUSPECT_FRAC: f32 = FORMANT.harmonic_suspect_frac;

/// How far a frame's formant estimates can be trusted, judged from the f0 at
/// which they were measured. LPC fits an envelope to a line spectrum: as f0
/// rises the harmonics thin out and the fit slides onto individual harmonics
/// instead of the envelope between them ("harmonic attraction" — the errors
/// are systematic, toward the strongest nearby harmonic, not noise).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormantGrade {
    /// Measured at f0 ≤ [`FORMANT_F0_IDENTITY_MAX_HZ`], away from harmonics:
    /// usable everywhere, including as voiceprint identity features.
    Identity,
    /// Measured at f0 ≤ [`FORMANT_F0_DISPLAY_MAX_HZ`]: displayable (expected
    /// uncertainty ≈ ±f0/4 — Kent & Vorperian 2018), but excluded from the
    /// voiceprint, where the harmonic-attraction bias would make the score
    /// partly a pitch comparison.
    DisplayOnly,
    /// Measured at higher f0, at an unusable f0, or with F1/F2 sitting on a
    /// harmonic (the Boë suspect band): not reported at all.
    Reject,
}

/// Grades a frame's formant estimates by the f0 at which they were measured.
///
/// `measured_f0` must be the f0 of the frame the LPC ran on — not the current
/// frame's f0, which may differ when formants are held across unvoiced gaps.
pub fn formant_grade(formants: &[Formant; N_FORMANTS], measured_f0: f32) -> FormantGrade {
    if !(measured_f0.is_finite() && measured_f0 > 0.0) {
        return FormantGrade::Reject;
    }
    if measured_f0 > FORMANT_F0_DISPLAY_MAX_HZ {
        return FormantGrade::Reject;
    }
    // Boë suspect band: F1 within ±15% of f0, or F2 within ±15% of 2·f0.
    // These are the two published confusion signatures, not a generalization.
    let near = |f: f32, harmonic: f32| -> bool {
        f > 0.0 && (f - harmonic).abs() <= FORMANT_HARMONIC_SUSPECT_FRAC * harmonic
    };
    if near(formants[0].frequency, measured_f0)
        || near(
            formants[1].frequency,
            FORMANT.f2_suspect_harmonic * measured_f0,
        )
    {
        return FormantGrade::Reject;
    }
    if measured_f0 <= FORMANT_F0_IDENTITY_MAX_HZ {
        FormantGrade::Identity
    } else {
        FormantGrade::DisplayOnly
    }
}

/// A2/A1 — the second harmonic's level over the first, in dB, from the same
/// measured harmonic amplitudes as the ladder.
///
/// This is Bozeman's own observable for the acoustic passaggio: "When H2
/// passes through F1 and begins to weaken, H1 will increase in power as it
/// approaches the first formant peak" (Journal of Singing 66:291, 2010). The
/// dominant-harmonic switch — A2/A1 crossing 0 dB — is what practitioners
/// watch on a real-time spectrogram, and unlike an F1 estimate it exists at
/// every pitch the app accepts.
///
/// Floor handling differs from [`h1_h2_db`] on purpose: H2 sinking below the
/// −48 dB relative floor is not "unmeasurable" here — it is the deeply
/// turned-over state itself. A1 above the floor with A2 below it returns the
/// floor-limited value (a lower bound on how far H2 has fallen, clamped, not
/// a precise level). `None` only when the fundamental itself is missing.
pub fn a2_a1_db(amps: &[f32]) -> Option<f32> {
    let (a1, a2) = (*amps.first()?, *amps.get(1)?);
    let max = amps.iter().cloned().fold(0.0f32, f32::max);
    let floor = max * DB_LOG_BASE.powf(TIMBRE.relative_floor_db / DB_PER_DECADE_AMPLITUDE);
    (max > TIMBRE.amp_eps && a1 > floor)
        .then(|| DB_PER_DECADE_AMPLITUDE * (a2.max(floor) / a1).log10())
}

// ── Classical voiceprint & similarity ────────────────────────────────────────

/// Rough adult-voice population `(mean, std)` for each scalar voiceprint
/// feature, used to z-score before comparison so heterogeneous units (Hz vs
/// dB/oct) contribute on a common scale. Approximate — good enough to weight
/// the features sensibly, not a calibrated model.
///
/// Order: F1, F2, F3, centroid, tilt dB/oct, VTL cm. The VTL population stats
/// come from Story et al. 2018's adult cohort: male mean 17.6 (sd 0.89),
/// female 15.6 (sd 1.14) — pooled across the sexes the spread is dominated by
/// the between-sex gap, hence ~1.4. Values live in `VoiceprintConfig`.
const VP_STATS: [(f32, f32); N_SCALAR_FEATURES] = {
    let mut out = [(0.0f32, 0.0f32); N_SCALAR_FEATURES];
    let mut i = 0;
    while i < N_SCALAR_FEATURES {
        out[i] = (VOICEPRINT.scalar_means[i], VOICEPRINT.scalar_stds[i]);
        i += 1;
    }
    out
};

/// Build a pitch-invariant voiceprint from a capture's averaged features.
/// `formants` are F1/F2/F3 (Hz), `profile` the mean relative harmonic
/// amplitudes (H1..), `centroid_hz` the mean spectral centroid. Tilt is
/// derived from the profile. f0 is intentionally not an input.
///
/// Unmeasured inputs (`None`) are stored with the struct's 0.0/`None`
/// "unmeasured" encodings — never substituted with plausible-looking values —
/// and `voiceprint_similarity` skips them rather than scoring them.
pub fn build_voiceprint(
    formants: Option<[Formant; N_FORMANTS]>,
    profile: &[f32],
    centroid_hz: Option<f32>,
    vtl_cm: Option<f32>,
) -> Voiceprint {
    let mut p = [0.0f32; VOICEPRINT_PROFILE_LEN];
    for (slot, &v) in p.iter_mut().zip(profile) {
        *slot = v;
    }
    let f = formants.unwrap_or(
        [Formant {
            frequency: 0.0,
            bandwidth: 0.0,
        }; N_FORMANTS],
    );
    Voiceprint {
        formants: f.map(|formant| formant.frequency),
        centroid_hz: centroid_hz.unwrap_or(0.0),
        tilt_db_oct: spectral_tilt_db_per_octave(&p),
        profile: p,
        vtl_cm: vtl_cm.unwrap_or(0.0),
    }
}

/// Similarity between two voiceprints in `0..=100`. Two parts:
///   * scalar resonance/brightness (F1/F2/F3, centroid, tilt) — z-scored, with
///     a Gaussian falloff on the RMS z-distance (identical → 1, ~1σ mean
///     difference → ~0.6, ~2σ → ~0.14); weight 0.6.
///   * timbre — cosine similarity of the harmonic profiles; weight 0.4.
///
/// A feature that is unmeasured on either side (0.0/non-finite formant or
/// centroid, `None`/non-finite tilt, degenerate profile) is *skipped* and the
/// remaining features renormalized — never z-scored as if 0 Hz were data, so
/// two failed captures cannot score as a perfect match. `None` means the two
/// prints share no comparable feature at all ("unscorable"); the result is
/// otherwise always finite.
///
/// Identical fully-measured voiceprints score 100. A real, explainable
/// voice-*similarity* score (self-consistency), not forensic speaker
/// recognition — classical formant/timbre features conflate "same
/// vowel/effort" with "same voice".
pub fn voiceprint_similarity(a: &Voiceprint, b: &Voiceprint) -> Option<f32> {
    // A measurable scalar feature: finite and physically possible (> 0 Hz for
    // formants/centroid). Tilt is measurable whenever it is finite.
    fn hz(v: f32) -> Option<f32> {
        (v.is_finite() && v > 0.0).then_some(v)
    }
    let [a_f1, a_f2, a_f3] = a.formants;
    let [b_f1, b_f2, b_f3] = b.formants;
    let a_scalars = [
        hz(a_f1),
        hz(a_f2),
        hz(a_f3),
        hz(a.centroid_hz),
        a.tilt_db_oct.filter(|t| t.is_finite()),
        hz(a.vtl_cm),
    ];
    let b_scalars = [
        hz(b_f1),
        hz(b_f2),
        hz(b_f3),
        hz(b.centroid_hz),
        b.tilt_db_oct.filter(|t| t.is_finite()),
        hz(b.vtl_cm),
    ];

    // Scalar part: RMS distance over the z-scored features measured on BOTH
    // sides, renormalized to however many that is.
    let mut sumsq = 0.0f32;
    let mut compared = 0usize;
    for i in 0..VP_STATS.len() {
        if let (Some(x), Some(y)) = (a_scalars[i], b_scalars[i]) {
            let (_, std) = VP_STATS[i];
            let dz = (x - y) / std;
            sumsq += dz * dz;
            compared += 1;
        }
    }
    let scalar_sim = (compared > 0).then(|| {
        let rms_z = (sumsq / compared as f32).sqrt();
        (-HALF * rms_z * rms_z).exp()
    });

    // Timbre part: cosine of the (non-negative) harmonic profiles; degenerate
    // (silent/NaN) profiles are unmeasured, not "zero similarity".
    let dot: f32 = a.profile.iter().zip(&b.profile).map(|(x, y)| x * y).sum();
    let na: f32 = a.profile.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.profile.iter().map(|x| x * x).sum::<f32>().sqrt();
    let timbre_sim =
        (na > VOICEPRINT.profile_norm_eps && nb > VOICEPRINT.profile_norm_eps && dot.is_finite())
            .then(|| (dot / (na * nb)).clamp(0.0, 1.0));

    // Weighted blend of whichever parts exist (0.6/0.4 when both do).
    let sim = match (scalar_sim, timbre_sim) {
        (Some(s), Some(t)) => VOICEPRINT.scalar_weight * s + VOICEPRINT.timbre_weight * t,
        (Some(s), None) => s,
        (None, Some(t)) => t,
        (None, None) => return None,
    };
    Some((sim * PERCENT).clamp(0.0, PERCENT))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn sine(freq: f32, sample_rate: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| (2.0 * PI * freq * i as f32 / sample_rate).sin())
            .collect()
    }

    #[test]
    fn yin_detects_pure_tones() {
        let sr = 44_100.0;
        for &f in &[110.0f32, 220.0, 440.0, 880.0] {
            let buf = sine(f, sr, 2048);
            let est = yin_pitch(&buf, sr).expect("estimate");
            assert!(
                est.confidence > 0.8,
                "low confidence for {f} Hz: {}",
                est.confidence
            );
            let err = (est.f0 - f).abs();
            assert!(err < f * 0.02, "f0 {} too far from {f} Hz", est.f0);
        }
    }

    #[test]
    fn yin_reports_low_confidence_on_noise() {
        // Deterministic pseudo-noise (no rng dependency): a non-periodic mix.
        let sr = 44_100.0;
        let buf: Vec<f32> = (0..2048)
            .map(|i| {
                let x = i as f32;
                (x * 0.91).sin() * 0.5
                    + (x * 0.37).sin() * 0.3
                    + ((x * 12.9898).sin() * 43758.5).fract()
            })
            .collect();
        let est = yin_pitch(&buf, sr).expect("estimate");
        // Aperiodic input should not look strongly voiced.
        assert!(
            est.confidence < 0.8,
            "noise looked too periodic: {}",
            est.confidence
        );
    }

    #[test]
    fn autocorrelation_peaks_at_zero_lag() {
        let buf = sine(200.0, 44_100.0, 1024);
        let r = autocorrelation(&buf, 64);
        assert!(r[0] > 0.0);
        for lag in 1..r.len() {
            assert!(
                r[0] >= r[lag].abs() - 1e-3,
                "r[0] should dominate, but r[{lag}]={}",
                r[lag]
            );
        }
    }

    #[test]
    fn lpc_coefficients_well_formed() {
        let buf = sine(300.0, 44_100.0, 1024);
        let a = lpc_coefficients(&buf, 12, 0.97);
        assert_eq!(a.len(), 13);
        assert!((a[0] - 1.0).abs() < 1e-6, "a[0] must be 1.0, got {}", a[0]);
        assert!(
            a.iter().all(|c| c.is_finite()),
            "coeffs must be finite: {a:?}"
        );
        // The model should have captured *something* (not the trivial all-pass).
        assert!(a[1..].iter().any(|&c| c.abs() > 1e-3), "LPC fit is trivial");
    }

    #[test]
    fn levinson_durbin_recovers_known_ar_polynomial() {
        // Ground truth: two resonators, the same shape the formant pipeline
        // models. Drive 1/A(z) with deterministic pseudo-noise and check the
        // recursion recovers A(z) itself — sign convention included. (The
        // original implementation returned the predictor coefficients, every
        // sign after a[0] flipped, which this test would fail at ~2.5 vs -2.5.)
        let fs = 10_000.0;
        let a_true = conv(&pole_pair(0.95, 700.0, fs), &pole_pair(0.93, 1800.0, fs));

        let n = 8192;
        let mut x = vec![0.0f32; n];
        let mut seed: u32 = 0x1234_5678;
        for i in 0..n {
            // Zero-mean deterministic white noise via an LCG (no rng dependency).
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let e = (seed >> 8) as f32 / (1u32 << 24) as f32 - 0.5;
            let mut acc = e;
            for k in 1..a_true.len() {
                if i >= k {
                    acc -= a_true[k] * x[i - k];
                }
            }
            x[i] = acc;
        }

        let order = a_true.len() - 1;
        let r = autocorrelation(&x, order);
        let a = levinson_durbin(&r, order);

        for (k, (&got, &want)) in a.iter().zip(&a_true).enumerate() {
            assert!((got - want).abs() < 0.05, "a[{k}] = {got}, want {want}");
        }
    }

    #[test]
    fn levinson_durbin_undersized_autocorr_is_identity() {
        for (r, order) in [(&[][..], 4usize), (&[1.0f32, 0.5][..], 4)] {
            let a = levinson_durbin(r, order);
            assert_eq!(a.len(), order + 1);
            assert_eq!(a[0], 1.0);
            assert!(a[1..].iter().all(|&c| c == 0.0), "{a:?}");
        }
    }

    #[test]
    fn yin_difference_undersized_input_returns_zeros() {
        // 100 samples cannot honor window=1024 + max_lag=50: all-zero diff,
        // no out-of-bounds read.
        let d = yin_difference(&vec![0.5; 100], 1024, 50);
        assert_eq!(d.len(), 51);
        assert!(d.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn parabolic_interpolation_boundary_inputs() {
        // Empty slice and edge taus come back unrefined, never panic.
        assert_eq!(parabolic_interpolation(&[], 0), 0.0);
        assert_eq!(parabolic_interpolation(&[], 1), 1.0);
        assert_eq!(parabolic_interpolation(&[1.0, 0.5, 1.0], 0), 0.0);
        assert_eq!(parabolic_interpolation(&[1.0, 0.5, 1.0], 2), 2.0);
        // Interior tau still interpolates (symmetric dip refines to itself).
        let t = parabolic_interpolation(&[1.0, 0.0, 1.0], 1);
        assert!((t - 1.0).abs() < 1e-6);
    }

    #[test]
    fn harmonic_amplitudes_rejects_non_finite_inputs() {
        let buf = sine(220.0, 44_100.0, 2048);
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -5.0] {
            let amps = harmonic_amplitudes(&buf, 44_100.0, bad);
            assert!(amps.iter().all(|&a| a == 0.0), "f0={bad} leaked: {amps:?}");
        }
        let amps = harmonic_amplitudes(&buf, f32::NAN, 220.0);
        assert!(amps.iter().all(|&a| a == 0.0), "NaN sample_rate leaked");
    }

    #[test]
    fn singers_formant_pct_rejects_non_finite_f0() {
        let amps = [1.0f32; 16];
        for bad in [f32::NAN, f32::INFINITY, 0.0, -5.0] {
            assert!(
                singers_formant_pct(&amps, bad).is_none(),
                "f0={bad} should be unmeasurable"
            );
        }
    }

    #[test]
    fn cpp_db_declines_truncated_quefrency_band() {
        // 2048 samples at 96 kHz reach only ~94 Hz, not the 60 Hz band end:
        // refuse rather than report a spurious in-band prominence.
        let hi = sine(220.0, 96_000.0, 2048);
        assert!(cpp_db(&hi, 96_000.0).is_none());
        // The same frame length at 44.1 kHz has the full band and measures.
        let ok = sine(220.0, 44_100.0, 2048);
        assert!(cpp_db(&ok, 44_100.0).is_some());
    }

    #[test]
    fn decimate_preserves_low_tone() {
        let sr = 44_100.0;
        let buf = sine(200.0, sr, 2048);
        let factor = 4;
        let dec = decimate(&buf, factor);
        // Length drops ~factor x.
        assert!(
            (dec.len() as i32 - 512).abs() <= 2,
            "len {} ~ 512",
            dec.len()
        );
        // The 200 Hz tone survives at the decimated rate.
        let est = yin_pitch(&dec, sr / factor as f32).expect("estimate");
        assert!(
            (est.f0 - 200.0).abs() < 6.0,
            "decimated f0 {} ~ 200 Hz",
            est.f0
        );
    }

    /// Denominator of a single 2-pole resonator at radius `r`, center `f` Hz:
    /// A(z) = 1 - 2r·cos(theta)·z^-1 + r^2·z^-2.
    fn pole_pair(r: f64, f: f64, fs: f64) -> [f32; 3] {
        let theta = 2.0 * std::f64::consts::PI * f / fs;
        [1.0, (-2.0 * r * theta.cos()) as f32, (r * r) as f32]
    }

    fn conv(a: &[f32], b: &[f32]) -> Vec<f32> {
        let mut out = vec![0.0f32; a.len() + b.len() - 1];
        for (i, &av) in a.iter().enumerate() {
            for (j, &bv) in b.iter().enumerate() {
                out[i + j] += av * bv;
            }
        }
        out
    }

    #[test]
    fn formant_orientation_and_bandwidth() {
        // Known single resonance at 1500 Hz, r = 0.95, fs = 10 kHz.
        let fs = 10_000.0;
        let (r, f) = (0.95, 1500.0);
        let a = pole_pair(r, f, fs);
        let formants = formants_from_lpc(&a, fs as f32);

        // If coefficient order were reversed (the mirror bug), the frequency
        // would land negative/culled and bandwidth would flip sign.
        assert!(
            (formants[0].frequency - 1500.0).abs() < 25.0,
            "F1 {} should be ~1500 Hz (orientation check)",
            formants[0].frequency
        );
        let expected_bw = -fs / std::f64::consts::PI * r.ln();
        assert!(
            (formants[0].bandwidth as f64 - expected_bw).abs() < 30.0,
            "BW {} should be ~{:.0} Hz and positive",
            formants[0].bandwidth,
            expected_bw
        );
    }

    /// Bandlimited sawtooth: Σ sin(2π k f t)/k for k·f below Nyquist.
    fn sawtooth(f: f32, sr: f32, n: usize, max_k: usize) -> Vec<f32> {
        (0..n)
            .map(|i| {
                let t = i as f32 / sr;
                (1..=max_k)
                    .filter(|&k| (k as f32) * f < sr / 2.0)
                    .map(|k| (2.0 * PI * k as f32 * f * t).sin() / k as f32)
                    .sum()
            })
            .collect()
    }

    #[test]
    fn harmonics_of_sawtooth_follow_one_over_k() {
        let (sr, f0) = (44_100.0, 110.0);
        let buf = sawtooth(f0, sr, 2048, 16);
        let amps = harmonic_amplitudes(&buf, sr, f0);
        assert!(amps[0] > 0.5, "H1 amp {}", amps[0]);
        for k in 1..8 {
            let expected = amps[0] / (k + 1) as f32;
            let got = amps[k];
            assert!(
                (got - expected).abs() < expected * 0.25,
                "H{} = {got}, expected ~{expected}",
                k + 1
            );
        }
    }

    #[test]
    fn harmonics_of_sine_isolate_h1() {
        let (sr, f0) = (44_100.0, 220.0);
        let buf = sine(f0, sr, 2048);
        let amps = harmonic_amplitudes(&buf, sr, f0);
        assert!(
            (amps[0] - 1.0).abs() < 0.05,
            "H1 {} should be ~1.0",
            amps[0]
        );
        for (k, &a) in amps.iter().enumerate().skip(1) {
            assert!(a < 0.01, "H{} should be ~0, got {a}", k + 1);
        }
    }

    #[test]
    fn harmonics_truncate_at_nyquist() {
        let (sr, f0) = (22_050.0, 5_000.0);
        let buf = sine(f0, sr, 2048);
        let amps = harmonic_amplitudes(&buf, sr, f0);
        assert!(amps[0] > 0.9, "H1 {} audible", amps[0]);
        // 3 · 5000 = 15000 ≥ Nyquist (11025) — zero from H3 up.
        assert_eq!(amps[2], 0.0);
        assert_eq!(amps[31], 0.0);
    }

    #[test]
    fn harmonics_degenerate_inputs_are_silent() {
        assert_eq!(harmonic_amplitudes(&[0.0; 16], 44_100.0, 110.0), [0.0; 32]);
        let buf = sine(220.0, 44_100.0, 2048);
        assert_eq!(harmonic_amplitudes(&buf, 44_100.0, 0.0), [0.0; 32]);
        assert_eq!(harmonic_amplitudes(&buf, 44_100.0, -5.0), [0.0; 32]);
    }

    /// Phase-accumulator synthesis with per-cycle frequency deviation and
    /// gain: cycle boundaries land at exact fractional sample positions, so
    /// the imposed perturbations are the ground truth (no rounding jitter).
    fn perturbed_tone(
        sr: f32,
        f_base: f32,
        n: usize,
        dev: impl Fn(usize) -> f32,
        gain: impl Fn(usize) -> f32,
    ) -> Vec<f32> {
        let mut phase = 0.0f64; // in cycles
        (0..n)
            .map(|_| {
                let cyc = phase.floor() as usize;
                let f = f_base * (1.0 + dev(cyc));
                let x: f32 = (1..=6)
                    .map(|k| {
                        ((2.0 * std::f64::consts::PI * k as f64 * phase).sin() / k as f64) as f32
                    })
                    .sum();
                phase += (f / sr) as f64;
                gain(cyc) * x
            })
            .collect()
    }

    #[test]
    fn perturbation_floor_on_clean_tone() {
        let sr = 44_100.0;
        let buf = perturbed_tone(sr, 220.0, 2048, |_| 0.0, |_| 1.0);
        let p = cycle_perturbation(&buf, sr, 220.0).expect("clean tone measures");
        assert!(p.jitter_pct < 0.2, "jitter floor {}", p.jitter_pct);
        assert!(p.shimmer_db < 0.1, "shimmer floor {}", p.shimmer_db);
    }

    #[test]
    fn jitter_tracks_imposed_period_perturbation() {
        // Alternating ±1 % frequency → adjacent periods differ by ~2 %.
        let sr = 44_100.0;
        let dev = |c: usize| if c.is_multiple_of(2) { 0.01 } else { -0.01 };
        let buf = perturbed_tone(sr, 220.0, 2048, dev, |_| 1.0);
        let p = cycle_perturbation(&buf, sr, 220.0).expect("jittered tone measures");
        assert!(
            (p.jitter_pct - 2.0).abs() < 0.5,
            "jitter {} should be ~2 %",
            p.jitter_pct
        );
    }

    #[test]
    fn shimmer_tracks_imposed_amplitude_perturbation() {
        // Alternating ±0.25 dB gain → adjacent cycles differ by 0.5 dB.
        let sr = 44_100.0;
        let g = 10f32.powf(0.25 / 20.0);
        let gain = move |c: usize| if c.is_multiple_of(2) { g } else { 1.0 / g };
        let buf = perturbed_tone(sr, 220.0, 2048, |_| 0.0, gain);
        let p = cycle_perturbation(&buf, sr, 220.0).expect("shimmered tone measures");
        assert!(
            (p.shimmer_db - 0.5).abs() < 0.15,
            "shimmer {} should be ~0.5 dB",
            p.shimmer_db
        );
    }

    #[test]
    fn perturbation_gates_reject_bad_frames() {
        let sr = 44_100.0;
        // Too few cycles: 80 Hz gives ~3.7 periods in 2048 samples.
        let low = perturbed_tone(sr, 80.0, 2048, |_| 0.0, |_| 1.0);
        assert!(cycle_perturbation(&low, sr, 80.0).is_none());
        // Aperiodic noise: interval sanity gate must fire rather than
        // returning junk numbers.
        let noise: Vec<f32> = (0..2048)
            .map(|i| ((i as f32 * 12.9898).sin() * 43758.5).fract() - 0.5)
            .collect();
        assert!(cycle_perturbation(&noise, sr, 220.0).is_none());
        assert!(cycle_perturbation(&[0.0; 2048], sr, 220.0).is_none());
    }

    #[test]
    fn cpp_separates_periodic_from_noise() {
        let sr = 44_100.0;
        let voiced = sawtooth(220.0, sr, 2048, 16);
        let noise: Vec<f32> = (0..2048)
            .map(|i| ((i as f32 * 12.9898).sin() * 43758.5).fract() - 0.5)
            .collect();

        let cpp_voiced = cpp_db(&voiced, sr).expect("voiced CPP");
        let cpp_noise = cpp_db(&noise, sr).expect("noise CPP");
        assert!(
            cpp_voiced > cpp_noise + 5.0,
            "voiced {cpp_voiced} should clearly exceed noise {cpp_noise}"
        );
        assert!(cpp_voiced.is_finite() && cpp_noise.is_finite());

        assert!(cpp_db(&[0.0; 2048], sr).is_none());
        assert!(cpp_db(&voiced[..64], sr).is_none());
    }

    #[test]
    fn centroid_of_pure_tone_is_that_tone() {
        let sr = 44_100.0;
        let buf = sine(1000.0, sr, 2048);
        let c = spectral_centroid(&buf, sr).expect("centroid");
        assert!((c - 1000.0).abs() < 30.0, "centroid {c} should be ~1000 Hz");
    }

    #[test]
    fn centroid_of_two_tones_is_between_them_weighted() {
        let sr = 44_100.0;
        // Equal-amplitude 500 Hz + 1500 Hz should centroid near the midpoint;
        // boosting the higher tone should pull it up.
        let equal: Vec<f32> = sine(500.0, sr, 2048)
            .iter()
            .zip(sine(1500.0, sr, 2048))
            .map(|(a, b)| a + b)
            .collect();
        let c_equal = spectral_centroid(&equal, sr).expect("equal centroid");
        assert!(
            (c_equal - 1000.0).abs() < 60.0,
            "equal-weight centroid {c_equal} should be ~1000 Hz"
        );

        let boosted: Vec<f32> = sine(500.0, sr, 2048)
            .iter()
            .zip(sine(1500.0, sr, 2048))
            .map(|(a, b)| a + 4.0 * b)
            .collect();
        let c_boosted = spectral_centroid(&boosted, sr).expect("boosted centroid");
        assert!(
            c_boosted > c_equal,
            "boosting the higher tone should raise centroid: {c_boosted} vs {c_equal}"
        );
    }

    #[test]
    fn centroid_degenerate_inputs_are_none() {
        assert!(spectral_centroid(&[0.0; 2048], 44_100.0).is_none());
        assert!(spectral_centroid(&sine(1000.0, 44_100.0, 2048), 0.0).is_none());
        assert!(spectral_centroid(&sine(1000.0, 44_100.0, 64), 44_100.0).is_none());
    }

    #[test]
    fn hnr_of_pure_sine_hits_ceiling() {
        let buf = sine(220.0, 44_100.0, 2048);
        let hnr = hnr_db(&buf, 44_100.0, 220.0).unwrap();
        assert!(hnr > 25.0, "pure tone HNR {hnr} should be very high");
    }

    #[test]
    fn hnr_tracks_known_snr() {
        // Sine (power 0.5) + deterministic pseudo-noise at a known level.
        let sr = 44_100.0;
        let tone = sine(220.0, sr, 2048);
        let noise: Vec<f32> = (0..2048)
            .map(|i| {
                let x = i as f32;
                (((x * 12.9898).sin() * 43758.5).fract() - 0.5) * 0.2
            })
            .collect();
        let sig_pow: f32 = tone.iter().map(|s| s * s).sum::<f32>() / 2048.0;
        let noise_pow: f32 = noise.iter().map(|s| s * s).sum::<f32>() / 2048.0;
        let expected = 10.0 * (sig_pow / noise_pow).log10();

        let mixed: Vec<f32> = tone.iter().zip(&noise).map(|(a, b)| a + b).collect();
        let hnr = hnr_db(&mixed, sr, 220.0).unwrap();
        assert!(
            (hnr - expected).abs() < 3.0,
            "HNR {hnr} should be near true SNR {expected}"
        );
    }

    #[test]
    fn hnr_degenerate_inputs_are_none() {
        let buf = sine(220.0, 44_100.0, 2048);
        assert!(hnr_db(&buf, 44_100.0, 0.0).is_none());
        assert!(hnr_db(&buf, 44_100.0, 30_000.0).is_none()); // lag < 2
        assert!(hnr_db(&[0.0; 2048], 44_100.0, 220.0).is_none()); // silence
        assert!(hnr_db(&buf[..32], 44_100.0, 220.0).is_none()); // too short
    }

    #[test]
    fn h1_h2_of_sawtooth_is_six_db() {
        let amps: Vec<f32> = (1..=8).map(|k| 1.0 / k as f32).collect();
        let d = h1_h2_db(&amps).unwrap();
        assert!((d - 6.02).abs() < 0.05, "H1-H2 {d}");

        // Missing H2 → None, not infinity.
        let mut no_h2 = amps.clone();
        no_h2[1] = 0.0;
        assert!(h1_h2_db(&no_h2).is_none());
        assert!(h1_h2_db(&[0.0; 8]).is_none());

        // Pressed voice: H2 louder than H1 → negative.
        let pressed = [0.5f32, 1.0, 0.3, 0.2];
        assert!(h1_h2_db(&pressed).unwrap() < 0.0);
    }

    #[test]
    fn note_naming_hits_known_pitches() {
        let a4 = freq_to_note(440.0).unwrap();
        assert_eq!((a4.name, a4.octave), ("A", 4));
        assert!(a4.cents.abs() < 0.01, "A440 cents {}", a4.cents);

        let a2 = freq_to_note(110.0).unwrap();
        assert_eq!((a2.name, a2.octave), ("A", 2));

        let c4 = freq_to_note(261.626).unwrap();
        assert_eq!((c4.name, c4.octave), ("C", 4));
        assert!(c4.cents.abs() < 0.5);

        let bb4 = freq_to_note(466.164).unwrap();
        assert_eq!((bb4.name, bb4.octave), ("A#", 4));

        // 445 Hz is ~19.6 cents sharp of A4.
        let sharp = freq_to_note(445.0).unwrap();
        assert_eq!(sharp.name, "A");
        assert!((sharp.cents - 19.56).abs() < 0.5, "cents {}", sharp.cents);

        assert!(freq_to_note(0.0).is_none());
        assert!(freq_to_note(-100.0).is_none());
        assert!(freq_to_note(f32::NAN).is_none());
    }

    #[test]
    fn semitones_from_basic_intervals() {
        assert!((semitones_from(440.0, 440.0).unwrap()).abs() < 0.01);
        assert!((semitones_from(880.0, 440.0).unwrap() - 12.0).abs() < 0.01);
        assert!((semitones_from(220.0, 440.0).unwrap() + 12.0).abs() < 0.01);
        // A perfect fifth up is 7 semitones (well-known 3:2 ratio approx).
        assert!((semitones_from(660.0, 440.0).unwrap() - 7.02).abs() < 0.05);
    }

    #[test]
    fn semitones_from_degenerate_inputs_are_none() {
        assert!(semitones_from(0.0, 440.0).is_none());
        assert!(semitones_from(440.0, 0.0).is_none());
        assert!(semitones_from(-1.0, 440.0).is_none());
        assert!(semitones_from(f32::NAN, 440.0).is_none());
        assert!(semitones_from(440.0, f32::INFINITY).is_none());
    }

    #[test]
    fn turning_over_crossing_reads_zero_at_h2_equals_f1() {
        // Bozeman's F1/H2 crossing: f0 at exactly F1/2 puts H2 on top of F1 —
        // the acoustic passaggio event — which this helper must read as 0.
        let f1 = 700.0;
        let f0_at_crossing = f1 / 2.0;
        let semis = semitones_from(2.0 * f0_at_crossing, f1).unwrap();
        assert!(semis.abs() < 0.01, "crossing should read ~0, got {semis}");

        // Below the crossing pitch, H2 sits below F1 (open timbre): negative.
        let below = semitones_from(2.0 * (f0_at_crossing - 50.0), f1).unwrap();
        assert!(
            below < 0.0,
            "below crossing should be negative, got {below}"
        );

        // Above it, H2 has cleared F1 (turned over): positive.
        let above = semitones_from(2.0 * (f0_at_crossing + 50.0), f1).unwrap();
        assert!(
            above > 0.0,
            "above crossing should be positive, got {above}"
        );
    }

    #[test]
    fn seventh_harmonic_is_31_cents_flat() {
        // The musician's landmark: H7 of any fundamental sits ~31.2 cents
        // below the 12-TET minor seventh (2 octaves + m7 above f0).
        let h7 = freq_to_note(7.0 * 110.0).unwrap();
        assert_eq!((h7.name, h7.octave), ("G", 5));
        assert!((h7.cents + 31.2).abs() < 0.5, "H7 cents {}", h7.cents);
    }

    #[test]
    fn tilt_of_sawtooth_spectrum_is_minus_six() {
        let amps: Vec<f32> = (1..=16).map(|k| 1.0 / k as f32).collect();
        let tilt = spectral_tilt_db_per_octave(&amps).unwrap();
        assert!((tilt + 6.02).abs() < 0.1, "tilt {tilt}");

        let flat = [1.0f32; 8];
        assert!(spectral_tilt_db_per_octave(&flat).unwrap().abs() < 1e-3);

        assert!(spectral_tilt_db_per_octave(&[1.0]).is_none());
        assert!(spectral_tilt_db_per_octave(&[0.0; 8]).is_none());
    }

    #[test]
    fn tilt_ignores_partials_below_noise_floor() {
        // A bandlimited source measures near-zero (but not exactly zero) in
        // the bins above its last harmonic; those must not skew the fit.
        let mut amps: Vec<f32> = (1..=8).map(|k| 1.0 / k as f32).collect();
        amps.extend([1e-5f32; 8]); // ~−100 dB numerical residue
        let tilt = spectral_tilt_db_per_octave(&amps).unwrap();
        assert!((tilt + 6.02).abs() < 0.1, "tilt {tilt} should stay ~ −6");
    }

    #[test]
    fn even_odd_balance_signs_and_edges() {
        // H2 twice the amplitude of H3 → 10·log10(4) ≈ +6.02 dB (even-heavy).
        let mut amps = [0.0f32; 16];
        amps[1] = 1.0; // H2
        amps[2] = 0.5; // H3
        let b = even_odd_balance_db(&amps).unwrap();
        assert!((b - 6.02).abs() < 0.05, "balance {b}");

        // Odd-only spectrum (square-wave-like) has no even energy → None.
        let mut odd_only = [0.0f32; 16];
        odd_only[2] = 1.0;
        assert!(even_odd_balance_db(&odd_only).is_none());

        // H1 alone must not count toward either side.
        let mut h1_only = [0.0f32; 16];
        h1_only[0] = 1.0;
        assert!(even_odd_balance_db(&h1_only).is_none());
    }

    #[test]
    fn singers_formant_counts_only_band_partials() {
        // f0 = 300 Hz: harmonics at 2800..=3400 are H10 (3000) and H11 (3300).
        let mut amps = [0.0f32; 32];
        for a in amps.iter_mut().take(11) {
            *a = 1.0;
        }
        let pct = singers_formant_pct(&amps, 300.0).unwrap();
        assert!((pct - 2.0 / 11.0 * 100.0).abs() < 0.1, "pct {pct}");

        assert!(singers_formant_pct(&amps, 0.0).is_none());
        assert!(singers_formant_pct(&[0.0; 32], 300.0).is_none());
    }

    #[test]
    fn voice_class_thresholds() {
        assert_eq!(voice_class(100.0), "Bass");
        assert_eq!(voice_class(150.0), "Baritone");
        assert_eq!(voice_class(200.0), "Tenor");
        assert_eq!(voice_class(250.0), "Alto");
        assert_eq!(voice_class(320.0), "Mezzo-Soprano");
        assert_eq!(voice_class(400.0), "Soprano");
        assert_eq!(voice_class(0.0), "—");
        assert_eq!(voice_class(-10.0), "—");
        assert_eq!(voice_class(f32::NAN), "—");
    }

    #[test]
    fn brightness_class_thresholds() {
        assert_eq!(brightness_class(800.0), "Dark");
        assert_eq!(brightness_class(1800.0), "Warm");
        assert_eq!(brightness_class(2800.0), "Balanced");
        assert_eq!(brightness_class(4000.0), "Bright");
        assert_eq!(brightness_class(5000.0), "Brilliant");
        assert_eq!(brightness_class(0.0), "—");
        assert_eq!(brightness_class(f32::NAN), "—");
    }

    #[test]
    fn timbre_description_branches() {
        // Hollow: even/odd dB clearly negative (odd-dominant) wins regardless
        // of harmonic count.
        assert_eq!(
            timbre_description(&[1.0; 16], Some(-10.0)),
            "Hollow · odd-dominant"
        );

        // Rich: >=6 of H2..H16 above the 0.25 relative-amplitude threshold.
        let mut rich = [0.0f32; 16];
        for a in rich.iter_mut() {
            *a = 0.4;
        }
        assert_eq!(timbre_description(&rich, Some(0.0)), "Rich · complex");

        // Pure: only H1 strong, everything else below threshold.
        let mut pure = [0.0f32; 16];
        pure[0] = 1.0;
        assert_eq!(timbre_description(&pure, Some(0.0)), "Pure · flute-like");
        assert_eq!(timbre_description(&pure, None), "Pure · flute-like");

        // Balanced: a middling number of strong harmonics, even/odd neutral.
        let mut balanced = [0.0f32; 16];
        balanced[0] = 1.0;
        balanced[1] = 0.4;
        balanced[2] = 0.4;
        balanced[3] = 0.4;
        assert_eq!(timbre_description(&balanced, Some(0.0)), "Balanced");
    }

    fn fmt3(a: f32, b: f32, c: f32) -> [Formant; 3] {
        [
            Formant {
                frequency: a,
                bandwidth: 80.0,
            },
            Formant {
                frequency: b,
                bandwidth: 120.0,
            },
            Formant {
                frequency: c,
                bandwidth: 160.0,
            },
        ]
    }

    #[test]
    fn lpc_pipeline_recovers_synthesized_vowel_formants() {
        // End-to-end regression for the levinson_durbin sign-convention bug:
        // run the exact analysis.rs recipe (decimate → pre-emphasized LPC →
        // Aberth roots) on a synthetic two-resonator "vowel" and require real
        // formants out. Under the flipped-sign bug this finds NO formants, so
        // the assertions below fail loudly rather than drifting.
        let sr = 44_100.0;
        // Noise-excited resonators at 700/1800 Hz (~120 Hz bandwidths).
        let bw = |b: f32| (-std::f32::consts::PI * b / sr).exp();
        let a_true = conv(
            &pole_pair(bw(120.0) as f64, 700.0, sr as f64),
            &pole_pair(bw(140.0) as f64, 1800.0, sr as f64),
        );
        let n = 4096; // warmup + the 2048-sample analysis frame
        let mut x = vec![0.0f32; n];
        let mut seed: u32 = 0xDEAD_BEEF;
        for i in 0..n {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let mut acc = (seed >> 8) as f32 / (1u32 << 24) as f32 - 0.5;
            for k in 1..a_true.len() {
                if i >= k {
                    acc -= a_true[k] * x[i - k];
                }
            }
            x[i] = acc;
        }
        let frame = &x[n - 2048..];

        // Mirror analysis.rs: decimate to ~11 kHz, order from the decimated rate.
        let m = ((sr / 11_025.0).round() as usize).max(1);
        let fs_dec = sr / m as f32;
        let order = (2 + (fs_dec / 1000.0) as usize).clamp(8, 20);
        let decimated = decimate(frame, m);
        let lpc = lpc_coefficients(&decimated, order, 0.97);
        let formants = formants_from_lpc(&lpc, fs_dec);

        // LPC may resolve an extra spurious resonance between the true ones,
        // shifting slot positions — require both true resonances to appear
        // *somewhere* in the resolved set (under the sign-flip bug the set is
        // empty, which is what this guards against).
        let resolved: Vec<f32> = formants
            .iter()
            .map(|f| f.frequency)
            .filter(|&f| f > 0.0)
            .collect();
        assert!(
            !resolved.is_empty(),
            "pipeline resolved no formants: {formants:?}"
        );
        for (target, tol) in [(700.0f32, 100.0f32), (1800.0, 150.0)] {
            assert!(
                resolved.iter().any(|f| (f - target).abs() < tol),
                "no resolved formant near {target} Hz: {resolved:?}"
            );
        }
    }

    #[test]
    fn voiceprint_identical_scores_100() {
        let profile: Vec<f32> = (1..=16).map(|k| 1.0 / k as f32).collect();
        let vp = build_voiceprint(
            Some(fmt3(520.0, 1480.0, 2610.0)),
            &profile,
            Some(1750.0),
            None,
        );
        let s = voiceprint_similarity(&vp, &vp).expect("fully measured prints are scorable");
        assert!((s - 100.0).abs() < 0.01, "identical should be 100, got {s}");
    }

    #[test]
    fn voiceprint_different_voice_scores_lower() {
        let prof_a: Vec<f32> = (1..=16).map(|k| 1.0 / k as f32).collect();
        let a = build_voiceprint(
            Some(fmt3(500.0, 1500.0, 2600.0)),
            &prof_a,
            Some(1700.0),
            None,
        );

        // A clearly different voice: formants shifted ~2σ, brighter centroid,
        // shallower tilt (via a flatter profile).
        let prof_b: Vec<f32> = (1..=16).map(|k| 1.0 / (k as f32).sqrt()).collect();
        let b = build_voiceprint(
            Some(fmt3(760.0, 2150.0, 3300.0)),
            &prof_b,
            Some(3200.0),
            None,
        );

        let self_score = voiceprint_similarity(&a, &a).expect("scorable");
        let cross = voiceprint_similarity(&a, &b).expect("scorable");
        assert!(
            cross < self_score - 25.0,
            "different voice {cross} should be well below self {self_score}"
        );
        assert!(
            (0.0..=100.0).contains(&cross),
            "score out of range: {cross}"
        );
    }

    #[test]
    fn voiceprint_closer_voice_scores_higher() {
        let prof: Vec<f32> = (1..=16).map(|k| 1.0 / k as f32).collect();
        let ref_vp = build_voiceprint(Some(fmt3(500.0, 1500.0, 2600.0)), &prof, Some(1800.0), None);
        // Near neighbor (small formant drift) vs far (large drift).
        let near = build_voiceprint(Some(fmt3(515.0, 1530.0, 2630.0)), &prof, Some(1850.0), None);
        let far = build_voiceprint(Some(fmt3(650.0, 1900.0, 3050.0)), &prof, Some(2600.0), None);
        let s_near = voiceprint_similarity(&ref_vp, &near).expect("scorable");
        let s_far = voiceprint_similarity(&ref_vp, &far).expect("scorable");
        assert!(
            s_near > s_far,
            "nearer voice {s_near} should score above farther {s_far}"
        );
    }

    #[test]
    fn voiceprint_unmeasured_features_skip_not_match() {
        let prof: Vec<f32> = (1..=16).map(|k| 1.0 / k as f32).collect();
        // Two captures whose formant extraction failed entirely: previously the
        // 0.0 sentinels z-scored as identical (dz = 0) and inflated the match.
        // Now they must score on the remaining features only.
        let no_formants_a = build_voiceprint(None, &prof, Some(1800.0), None);
        let no_formants_b = build_voiceprint(None, &prof, Some(2600.0), None);
        let with = voiceprint_similarity(&no_formants_a, &no_formants_b).expect("scorable");
        // The same centroid gap on otherwise-identical fully-measured prints:
        // adding three *matching* formants can only raise the score, so the
        // formantless pair must not out-score it (no free perfect features).
        let full_a = build_voiceprint(Some(fmt3(500.0, 1500.0, 2600.0)), &prof, Some(1800.0), None);
        let full_b = build_voiceprint(Some(fmt3(500.0, 1500.0, 2600.0)), &prof, Some(2600.0), None);
        let full = voiceprint_similarity(&full_a, &full_b).expect("scorable");
        assert!(
            with <= full + 0.01,
            "skipped formants ({with}) must not score above matching formants ({full})"
        );

        // A capture missing everything scalar still scores via timbre alone.
        let timbre_only = build_voiceprint(None, &prof, None, None);
        let s = voiceprint_similarity(&timbre_only, &full_a).expect("timbre is comparable");
        assert!((0.0..=100.0).contains(&s));
    }

    #[test]
    fn voiceprint_unscorable_and_nan_safety() {
        // Nothing measured on one side at all: unscorable, not NaN, not 100.
        let empty = build_voiceprint(None, &[0.0; 16], None, None);
        let real_prof: Vec<f32> = (1..=16).map(|k| 1.0 / k as f32).collect();
        let real = build_voiceprint(
            Some(fmt3(500.0, 1500.0, 2600.0)),
            &real_prof,
            Some(1800.0),
            None,
        );
        assert_eq!(voiceprint_similarity(&empty, &empty), None);
        assert_eq!(voiceprint_similarity(&empty, &real), None);

        // NaN smuggled into a stored print (e.g. a hand-edited archive) is
        // treated as unmeasured — the result stays finite, never NaN.
        let mut poisoned = real;
        poisoned.centroid_hz = f32::NAN;
        poisoned.tilt_db_oct = Some(f32::NAN);
        let s = voiceprint_similarity(&poisoned, &real).expect("still scorable");
        assert!(s.is_finite(), "NaN leaked into the match score: {s}");
    }

    #[test]
    fn formants_two_resonances_sorted() {
        let fs = 10_000.0;
        let a = conv(&pole_pair(0.95, 700.0, fs), &pole_pair(0.93, 1800.0, fs));
        let formants = formants_from_lpc(&a, fs as f32);

        assert!(
            (formants[0].frequency - 700.0).abs() < 30.0,
            "F1 {} ~ 700 Hz",
            formants[0].frequency
        );
        assert!(
            (formants[1].frequency - 1800.0).abs() < 30.0,
            "F2 {} ~ 1800 Hz",
            formants[1].frequency
        );
    }

    // ── room calibration ────────────────────────────────────────────────────

    #[test]
    fn calibration_of_a_quiet_room_finds_floor_and_no_interferer() {
        let mut c = RoomCalibrator::new();
        for _ in 0..CALIB_FRAMES {
            c.push(0.001, None);
        }
        let cal = c.finish().expect("clean pass");
        assert!((cal.ambient_rms - 0.001).abs() < 1e-6);
        assert!(cal.interferer.is_none());
    }

    #[test]
    fn calibration_fingerprints_a_stable_hum() {
        // A fan/mains hum: YIN finds ~120 Hz on most frames, rock steady.
        let mut c = RoomCalibrator::new();
        for i in 0..CALIB_FRAMES {
            let f0 = if i % 3 == 0 {
                None
            } else {
                Some(120.0 + (i % 5) as f32 * 0.2)
            };
            c.push(0.004, f0);
        }
        let cal = c.finish().expect("hum is a valid room");
        let hum = cal.interferer.expect("hum fingerprinted");
        assert!((hum.f0_hz - 120.4).abs() < 1.0, "got {}", hum.f0_hz);
    }

    #[test]
    fn calibration_rejects_speech_instead_of_fingerprinting_it() {
        // Wandering pitch on many frames = someone talked during the pass.
        let mut c = RoomCalibrator::new();
        for i in 0..CALIB_FRAMES {
            let f0 = (i % 2 == 0).then(|| 150.0 + 40.0 * ((i as f32) * 0.7).sin());
            c.push(0.02, f0);
        }
        assert_eq!(c.finish().unwrap_err(), CalibrationFailure::VoiceDetected);
    }

    #[test]
    fn calibration_too_short_is_rejected() {
        let mut c = RoomCalibrator::new();
        for _ in 0..10 {
            c.push(0.001, None);
        }
        assert_eq!(c.finish().unwrap_err(), CalibrationFailure::TooShort);
    }

    #[test]
    fn interferer_gate_blocks_the_hum_but_not_the_louder_singer() {
        let hum = Interferer {
            f0_hz: 120.0,
            rms: 0.004,
        };
        // The hum itself, at its own level: blocked.
        assert!(interferer_match(121.0, 0.004, &hum));
        // The singer on the same pitch but 20 dB louder: allowed.
        assert!(!interferer_match(121.0, 0.04, &hum));
        // A different pitch at hum level: allowed (it is not the hum).
        assert!(!interferer_match(150.0, 0.004, &hum));
        assert!(!interferer_match(f32::NAN, 0.004, &hum));
    }

    #[test]
    fn seeded_noise_floor_gates_immediately() {
        let mut nf = NoiseFloor::new();
        assert!(nf.snr_db(0.1).is_none(), "unseeded floor must warm up");
        nf.seed(0.001);
        let snr = nf.snr_db(0.1).expect("seeded floor reports at once");
        assert!((snr - 40.0).abs() < 1.0, "got {snr}");
    }

    // ── noise floor & SNR gating ────────────────────────────────────────────

    #[test]
    fn noise_floor_warms_up_before_reporting() {
        let mut nf = NoiseFloor::new();
        assert!(nf.snr_db(0.1).is_none(), "must not gate before warmup");
        for _ in 0..31 {
            nf.push_unvoiced(0.001);
        }
        assert!(nf.snr_db(0.1).is_none());
        nf.push_unvoiced(0.001);
        let snr = nf.snr_db(0.1).expect("warmed up");
        assert!(
            (snr - 40.0).abs() < 1.0,
            "0.1 over 0.001 floor = 40 dB, got {snr}"
        );
    }

    #[test]
    fn noise_floor_ignores_loud_unvoiced_outliers() {
        // Real speech's breaths and fricatives are loud but unvoiced; the
        // low percentile must keep them out of the ambient estimate.
        let mut nf = NoiseFloor::new();
        for i in 0..128 {
            // 1 in 4 frames is a "breath" 20x the ambience.
            nf.push_unvoiced(if i % 4 == 0 { 0.02 } else { 0.001 });
        }
        let floor = nf.floor().unwrap();
        assert!(floor < 0.002, "floor {floor} dragged up by outliers");
    }

    #[test]
    fn noise_floor_tracks_a_step_up_in_ambience() {
        let mut nf = NoiseFloor::new();
        for _ in 0..128 {
            nf.push_unvoiced(0.001);
        }
        // Air conditioner switches on: ambience jumps 10x.
        for _ in 0..128 {
            nf.push_unvoiced(0.01);
        }
        let floor = nf.floor().unwrap();
        assert!(floor > 0.008, "floor {floor} failed to track the new room");
        // A frame that cleared the old room by 40 dB now clears by ~20.
        let snr = nf.snr_db(0.1).unwrap();
        assert!((snr - 20.0).abs() < 1.5, "got {snr}");
    }

    #[test]
    fn frame_rms_basics() {
        assert_eq!(frame_rms(&[]), 0.0);
        let s: Vec<f32> = (0..1000)
            .map(|i| (2.0 * PI * 100.0 * i as f32 / 8000.0).sin())
            .collect();
        let r = frame_rms(&s);
        assert!(
            (r - std::f32::consts::FRAC_1_SQRT_2).abs() < 0.01,
            "sine RMS {r}"
        );
    }

    // ── VTL as an identity feature ──────────────────────────────────────────

    #[test]
    fn voiceprint_vtl_difference_lowers_the_score() {
        let prof: [f32; 16] = std::array::from_fn(|i| 1.0 / (i + 1) as f32);
        let base = build_voiceprint(
            Some(fmt3(500.0, 1500.0, 2600.0)),
            &prof,
            Some(1800.0),
            Some(17.5),
        );
        let same_vtl = build_voiceprint(
            Some(fmt3(500.0, 1500.0, 2600.0)),
            &prof,
            Some(1800.0),
            Some(17.5),
        );
        let short_vtl = build_voiceprint(
            Some(fmt3(500.0, 1500.0, 2600.0)),
            &prof,
            Some(1800.0),
            Some(14.5),
        );
        let s_same = voiceprint_similarity(&base, &same_vtl).unwrap();
        let s_diff = voiceprint_similarity(&base, &short_vtl).unwrap();
        assert!((s_same - 100.0).abs() < 0.01, "identical prints: {s_same}");
        assert!(
            s_diff < s_same - 2.0,
            "a 3 cm anatomy difference must cost score: {s_diff} vs {s_same}"
        );
    }

    #[test]
    fn voiceprint_unmeasured_vtl_is_skipped_not_scored() {
        let prof: [f32; 16] = std::array::from_fn(|i| 1.0 / (i + 1) as f32);
        let with = build_voiceprint(
            Some(fmt3(500.0, 1500.0, 2600.0)),
            &prof,
            Some(1800.0),
            Some(17.5),
        );
        let without =
            build_voiceprint(Some(fmt3(500.0, 1500.0, 2600.0)), &prof, Some(1800.0), None);
        // One side unmeasured: the feature is skipped, so two otherwise
        // identical prints still score 100 — never penalized, never
        // rewarded, for a value that does not exist.
        let s = voiceprint_similarity(&with, &without).unwrap();
        assert!((s - 100.0).abs() < 0.01, "got {s}");
    }

    // ── formant gating & A2/A1 ──────────────────────────────────────────────

    fn f3(a: f32, b: f32, c: f32) -> [Formant; 3] {
        [
            Formant {
                frequency: a,
                bandwidth: 80.0,
            },
            Formant {
                frequency: b,
                bandwidth: 120.0,
            },
            Formant {
                frequency: c,
                bandwidth: 160.0,
            },
        ]
    }

    #[test]
    fn formant_grade_thresholds() {
        // A clean male-vowel frame: f0 well below both thresholds, formants
        // far from any harmonic of it.
        let fmts = f3(700.0, 1500.0, 2500.0);
        assert_eq!(formant_grade(&fmts, 110.0), FormantGrade::Identity);
        assert_eq!(
            formant_grade(&fmts, FORMANT_F0_IDENTITY_MAX_HZ),
            FormantGrade::Identity
        );
        // Between the identity and display ceilings: display only. 260 Hz
        // keeps 2·f0 = 520 comfortably outside F1's ±15% band.
        assert_eq!(
            formant_grade(&f3(700.0, 1600.0, 2500.0), 260.0),
            FormantGrade::DisplayOnly
        );
        // Above the display ceiling: rejected outright.
        assert_eq!(formant_grade(&fmts, 351.0), FormantGrade::Reject);
        assert_eq!(formant_grade(&fmts, 600.0), FormantGrade::Reject);
    }

    #[test]
    fn formant_grade_suspect_bands() {
        // F1 within ±15% of f0 → harmonic/formant confusion (Boë et al.).
        assert_eq!(
            formant_grade(&f3(200.0, 1500.0, 2500.0), 190.0),
            FormantGrade::Reject
        );
        // F2 within ±15% of 2·f0 — F1 clean.
        assert_eq!(
            formant_grade(&f3(700.0, 390.0 * 2.0 * 0.5, 2500.0), 195.0),
            FormantGrade::Reject
        );
        // Just outside the band on both counts: accepted.
        assert_eq!(
            formant_grade(&f3(240.0, 1500.0, 2500.0), 190.0),
            FormantGrade::Identity
        );
        // Unresolved slots (0.0) never trigger the proximity check.
        assert_eq!(
            formant_grade(&f3(0.0, 0.0, 0.0), 190.0),
            FormantGrade::Identity
        );
    }

    #[test]
    fn formant_grade_rejects_unusable_f0() {
        let fmts = f3(700.0, 1500.0, 2500.0);
        assert_eq!(formant_grade(&fmts, 0.0), FormantGrade::Reject);
        assert_eq!(formant_grade(&fmts, -1.0), FormantGrade::Reject);
        assert_eq!(formant_grade(&fmts, f32::NAN), FormantGrade::Reject);
        assert_eq!(formant_grade(&fmts, f32::INFINITY), FormantGrade::Reject);
    }

    #[test]
    fn a2_a1_of_sawtooth_is_minus_six_db() {
        // Sawtooth partials fall as 1/k: A2/A1 = 1/2 → −6.02 dB.
        let mut amps = [0.0f32; 16];
        for (k, a) in amps.iter_mut().enumerate() {
            *a = 1.0 / (k + 1) as f32;
        }
        let db = a2_a1_db(&amps).expect("measurable");
        assert!((db + 6.02).abs() < 0.1, "got {db}");
    }

    #[test]
    fn a2_a1_h2_below_floor_reads_deeply_negative_not_none() {
        // H2 gone entirely is the deeply turned-over state, not "unknown":
        // the reading clamps at the −48 dB relative floor instead of
        // disappearing exactly when the gauge is most informative.
        let mut amps = [0.0f32; 16];
        amps[0] = 1.0;
        let db = a2_a1_db(&amps).expect("A1 alone is measurable");
        assert!((db + 48.0).abs() < 0.5, "got {db}");
    }

    #[test]
    fn a2_a1_dominant_h2_is_positive_and_silence_is_none() {
        let mut amps = [0.0f32; 16];
        amps[0] = 0.5;
        amps[1] = 1.0; // open-timbre signature: H2 riding the F1 peak
        let db = a2_a1_db(&amps).expect("measurable");
        assert!((db - 6.02).abs() < 0.1, "got {db}");

        assert!(a2_a1_db(&[0.0; 16]).is_none());
        assert!(a2_a1_db(&[]).is_none());
    }
}
