use crate::types::{Formant, MAX_PARTIALS, Voiceprint};
use aberth::AberthSolver;
use std::f32::consts::PI;

/// YIN analysis window length: the number of samples compared per lag in the
/// difference function. The analysis frame must be at least this plus the
/// maximum search lag so `x[j + tau]` never reads past the frame end.
pub const YIN_WINDOW: usize = 1024;

/// YIN search bounds (Hz) and the absolute threshold for the CMND dip.
const YIN_F0_MIN: f32 = 50.0;
const YIN_F0_MAX: f32 = 1000.0;
const YIN_THRESHOLD: f32 = 0.12;

/// Solves the Yule-Walker equations using Levinson-Durbin recursion.
/// Returns the LPC reflection coefficients.
pub fn levinson_durbin(autocorr: &[f32], order: usize) -> Vec<f32> {
    let mut a = vec![0.0; order + 1];
    let mut e = autocorr[0];
    a[0] = 1.0;

    for i in 1..=order {
        if e.abs() < 1e-9 {
            // Prediction error has collapsed (silent or degenerate frame).
            // Stop the recursion; remaining higher-order coeffs stay 0, which
            // is a valid lower-order polynomial rather than a NaN blow-up.
            break;
        }
        let mut k = 0.0;
        for j in 1..i {
            k += a[j] * autocorr[i - j];
        }
        k = (autocorr[i] - k) / e;

        a[i] = k;
        let mut new_a = a.clone();
        for j in 1..i {
            new_a[j] = a[j] - k * a[i - j];
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
pub fn formants_from_lpc(lpc: &[f32], sample_rate: f32) -> [Formant; 3] {
    let mut result = [Formant {
        frequency: 0.0,
        bandwidth: 0.0,
    }; 3];

    if lpc.len() < 3 {
        return result;
    }

    // Reverse (root in z, not z^-1) and promote to f64 for conditioning.
    let coeffs: Vec<f64> = lpc.iter().rev().map(|&c| c as f64).collect();

    let mut solver = AberthSolver::new();
    solver.max_iterations = 50;
    solver.epsilon = 1e-9;
    let roots = solver.find_roots(&coeffs);

    let fs = sample_rate as f64;
    let two_pi = 2.0 * std::f64::consts::PI;
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
        if (90.0..=5000.0).contains(&freq) && bandwidth < 500.0 {
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
pub fn parabolic_interpolation(diff_fn: &[f32], tau: usize) -> f32 {
    if tau == 0 || tau >= diff_fn.len() - 1 {
        return tau as f32;
    }

    let s0 = diff_fn[tau - 1];
    let s1 = diff_fn[tau];
    let s2 = diff_fn[tau + 1];

    let denom = s0 - 2.0 * s1 + s2;
    if denom.abs() < 1e-6 {
        tau as f32
    } else {
        tau as f32 + (s0 - s2) / (2.0 * denom)
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
/// The caller must guarantee `window + max_lag <= samples.len()` so every
/// `x[j+tau]` is in bounds — the same contract the GPU buffer sizing honors.
/// (The original 1024-sample buffer violated it, biasing the high lags.)
pub fn yin_difference(samples: &[f32], window: usize, max_lag: usize) -> Vec<f32> {
    let mut diff = vec![0.0f32; max_lag + 1];
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
    if tau_max < 3 {
        return None;
    }
    let tau_min = ((sample_rate / YIN_F0_MAX) as usize).clamp(2, tau_max - 1);
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
    let w = YIN_WINDOW.min(n / 2);
    if w < 2 {
        return None;
    }
    // Largest lag we can evaluate without `x[j+tau]` leaving the frame, also
    // bounded by the lowest pitch we care about.
    let max_lag = ((sample_rate / YIN_F0_MIN) as usize).min(n - w);
    if max_lag < 3 {
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
        let centered = i as f32 - m / 2.0;
        let sinc = if centered.abs() < 1e-6 {
            2.0 * cutoff_norm
        } else {
            (2.0 * PI * cutoff_norm * centered).sin() / (PI * centered)
        };
        let window = 0.54 - 0.46 * (2.0 * PI * i as f32 / m).cos();
        *tap = sinc * window;
        sum += *tap;
    }
    if sum.abs() > 1e-9 {
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
    let cutoff = 0.45 / factor as f32;
    let taps = lowpass_fir(cutoff, 31);
    let half = (taps.len() / 2) as isize;
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
        let w = 0.54 - 0.46 * (2.0 * PI * n as f32 / m).cos();
        *v *= w;
    }

    let r = autocorrelation(&x, order);
    if r[0] <= 1e-9 {
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
/// Harmonics at or above Nyquist — and everything when `f0` is non-positive or
/// the frame is degenerate — are 0.
pub fn harmonic_amplitudes(samples: &[f32], sample_rate: f32, f0: f32) -> [f32; MAX_PARTIALS] {
    let mut amps = [0.0f32; MAX_PARTIALS];
    let n = samples.len();
    if n < 32 || f0 <= 0.0 || sample_rate <= 0.0 {
        return amps;
    }

    // Hann window; its coherent gain (Σw / 2) normalizes the DFT magnitude
    // back to sinusoid peak amplitude.
    let m = (n - 1) as f32;
    let mut windowed = vec![0.0f32; n];
    let mut wsum = 0.0f32;
    for (i, (w, &x)) in windowed.iter_mut().zip(samples).enumerate() {
        let win = 0.5 - 0.5 * (2.0 * PI * i as f32 / m).cos();
        *w = x * win;
        wsum += win;
    }
    let norm = 2.0 / wsum;

    let nyquist = sample_rate / 2.0;
    for (k, amp) in amps.iter_mut().enumerate() {
        let freq = (k + 1) as f32 * f0;
        if freq >= nyquist {
            break;
        }
        // Goertzel recurrence at `freq`.
        let omega = 2.0 * PI * freq / sample_rate;
        let coeff = 2.0 * omega.cos();
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
    if f0 <= 0.0 || sample_rate <= 0.0 || n < 64 {
        return None;
    }
    let lag = sample_rate / f0;
    let lag_i = lag.round() as usize;
    if lag_i < 2 || lag_i + 2 >= n {
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
    if r0 <= 1e-12 {
        return None;
    }

    // Parabolic vertex through the three lags around the period gives the
    // true (non-integer-lag) correlation peak height.
    let (y0, y1, y2) = (r_at(lag_i - 1), r_at(lag_i), r_at(lag_i + 1));
    let denom = y0 - 2.0 * y1 + y2;
    let peak = if denom.abs() < 1e-12 {
        y1
    } else {
        y1 - (y0 - y2) * (y0 - y2) / (8.0 * denom)
    };

    let r = (peak / r0).clamp(-0.9999, 0.9999);
    if r <= 0.0 {
        return Some(-40.0);
    }
    Some((10.0 * (r / (1.0 - r)).log10()).clamp(-40.0, 40.0))
}

/// H1–H2 in dB: the level difference between the first two harmonics, the
/// standard phonation-type measure (large = breathy/flowy, small or negative
/// = pressed). `None` when either harmonic sits below the −48 dB relative
/// floor (same floor as the ladder display).
pub fn h1_h2_db(amps: &[f32]) -> Option<f32> {
    let (a1, a2) = (*amps.first()?, *amps.get(1)?);
    let max = amps.iter().cloned().fold(0.0f32, f32::max);
    let floor = max * 10f32.powf(-48.0 / 20.0);
    (max > 1e-6 && a1 > floor && a2 > floor).then(|| 20.0 * (a1 / a2).log10())
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
    if f0 <= 0.0 || sample_rate <= 0.0 || n < 64 {
        return None;
    }
    let period = sample_rate / f0;
    if period < 8.0 {
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
        let denom = y0 - 2.0 * y1 + y2;
        if denom.abs() < 1e-12 {
            return Some((best as f32, y1));
        }
        let delta = ((y0 - y2) / (2.0 * denom)).clamp(-0.5, 0.5);
        let height = y1 - (y0 - y2) * delta / 4.0;
        Some((best as f32 + delta, height))
    };

    // First mark: strongest sample in the first 1.5 periods; then march
    // forward one period at a time inside a ±30 % search window.
    let mut marks: Vec<(f32, f32)> = Vec::new();
    let first = peak_in(0, (1.5 * period) as usize)?;
    marks.push(first);
    loop {
        let prev = marks.last().unwrap().0;
        let lo = (prev + 0.7 * period) as usize;
        let hi = (prev + 1.3 * period).ceil() as usize;
        if hi > n {
            break;
        }
        match peak_in(lo, hi) {
            Some(p) => marks.push(p),
            None => break,
        }
    }
    if marks.len() < 5 {
        return None;
    }

    let periods: Vec<f32> = marks.windows(2).map(|w| w[1].0 - w[0].0).collect();
    // Sanity: every interval near the YIN period, else the marks double-fired
    // or skipped (strong formants can do this) and the numbers would be junk.
    if periods
        .iter()
        .any(|&t| t < 0.7 * period || t > 1.3 * period)
    {
        return None;
    }

    let mean_t = periods.iter().sum::<f32>() / periods.len() as f32;
    let jitter_pct = periods.windows(2).map(|w| (w[0] - w[1]).abs()).sum::<f32>()
        / (periods.len() - 1) as f32
        / mean_t
        * 100.0;
    // Plausibility ceiling: aperiodic input yields peaks scattered uniformly
    // inside the search windows (~20-30 % "jitter") — the ±30 % interval
    // check can't catch that by construction. Severe pathology is ~2-3 %;
    // beyond 5 % the marks aren't tracking real glottal cycles.
    if jitter_pct > 5.0 {
        return None;
    }

    let amps: Vec<f32> = marks.iter().map(|&(_, a)| a).collect();
    if amps.iter().any(|&a| a <= 1e-6) {
        return None;
    }
    let shimmer_db = amps
        .windows(2)
        .map(|w| (20.0 * (w[1] / w[0]).log10()).abs())
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
    use rustfft::{FftPlanner, num_complex::Complex};

    let n = samples.len();
    if n < 256 || sample_rate <= 0.0 {
        return None;
    }
    if samples.iter().map(|s| s * s).sum::<f32>() < 1e-9 {
        return None; // silence
    }

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(n);

    // Hann-windowed spectrum.
    let m = (n - 1) as f32;
    let mut buf: Vec<Complex<f32>> = samples
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let w = 0.5 - 0.5 * (2.0 * PI * i as f32 / m).cos();
            Complex::new(x * w, 0.0)
        })
        .collect();
    fft.process(&mut buf);

    // dB power spectrum is real and even, so its FFT is the real cepstrum.
    let mut logspec: Vec<Complex<f32>> = buf
        .iter()
        .map(|c| Complex::new(10.0 * (c.norm_sqr() + 1e-12).log10(), 0.0))
        .collect();
    fft.process(&mut logspec);
    let ceps_db: Vec<f32> = logspec
        .iter()
        .map(|c| 10.0 * (c.norm_sqr() / (n as f32) / (n as f32) + 1e-12).log10())
        .collect();

    // Voice pitch quefrency band: 60–500 Hz.
    let q_lo = (sample_rate / 500.0).ceil() as usize;
    let q_hi = ((sample_rate / 60.0).floor() as usize).min(n / 2 - 1);
    if q_lo + 4 >= q_hi {
        return None;
    }

    // Regression baseline over the full analyzed quefrency range.
    let range = q_lo..n / 2;
    let cnt = range.len() as f32;
    let mx = range.clone().map(|q| q as f32).sum::<f32>() / cnt;
    let my = range.clone().map(|q| ceps_db[q]).sum::<f32>() / cnt;
    let (mut num, mut den) = (0.0f32, 0.0f32);
    for q in range {
        let dx = q as f32 - mx;
        num += dx * (ceps_db[q] - my);
        den += dx * dx;
    }
    if den < 1e-9 {
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

/// Spectral centroid in Hz: the power-weighted mean frequency of the full
/// magnitude spectrum over 80 Hz–min(8000, Nyquist), not just the harmonic
/// ladder — this deliberately includes inter-harmonic noise energy, which is
/// what makes it a meaningful "brightness" measure distinct from the tilt of
/// the harmonics alone.
pub fn spectral_centroid(samples: &[f32], sample_rate: f32) -> Option<f32> {
    use rustfft::{FftPlanner, num_complex::Complex};

    let n = samples.len();
    if n < 256 || sample_rate <= 0.0 {
        return None;
    }
    if samples.iter().map(|s| s * s).sum::<f32>() < 1e-9 {
        return None; // silence
    }

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(n);

    let m = (n - 1) as f32;
    let mut buf: Vec<Complex<f32>> = samples
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let w = 0.5 - 0.5 * (2.0 * PI * i as f32 / m).cos();
            Complex::new(x * w, 0.0)
        })
        .collect();
    fft.process(&mut buf);

    let bin_hz = sample_rate / n as f32;
    let lo_bin = (80.0 / bin_hz).ceil() as usize;
    let hi_bin = ((8000.0f32.min(sample_rate / 2.0)) / bin_hz).floor() as usize;
    let hi_bin = hi_bin.min(n / 2);
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
    (den > 1e-9).then(|| num / den)
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

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Nearest 12-TET note to `hz` (A4 = 440). `None` for non-positive/non-finite
/// input. Cents are in (-50, +50].
pub fn freq_to_note(hz: f32) -> Option<Note> {
    if hz <= 0.0 || !hz.is_finite() {
        return None;
    }
    let midi = 69.0 + 12.0 * (hz / 440.0).log2();
    let nearest = midi.round();
    let idx = (nearest as i32).rem_euclid(12) as usize;
    Some(Note {
        name: NOTE_NAMES[idx],
        octave: (nearest as i32).div_euclid(12) - 1,
        cents: (midi - nearest) * 100.0,
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
pub fn semitones_from(freq: f32, reference: f32) -> Option<f32> {
    if freq <= 0.0 || reference <= 0.0 || !freq.is_finite() || !reference.is_finite() {
        return None;
    }
    Some(12.0 * (freq / reference).log2())
}

/// Spectral tilt in dB/octave: least-squares slope of harmonic level (dB)
/// against log2(harmonic number). A sawtooth's 1/k rolloff measures ≈ −6
/// dB/oct; a brighter, more pressed voice is shallower (closer to 0).
pub fn spectral_tilt_db_per_octave(amps: &[f32]) -> Option<f32> {
    let max = amps.iter().cloned().fold(0.0f32, f32::max);
    if max <= 1e-6 {
        return None;
    }
    // Only fit partials within 48 dB of the strongest one: bins below that are
    // numerical noise whose ~−100 dB levels would drag the slope toward −∞.
    let floor = max * 10f32.powf(-48.0 / 20.0);
    let pts: Vec<(f32, f32)> = amps
        .iter()
        .enumerate()
        .filter(|&(_, &a)| a > floor)
        .map(|(k, &a)| (((k + 1) as f32).log2(), 20.0 * a.log10()))
        .collect();
    if pts.len() < 2 {
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
    (den > 1e-9).then(|| num / den)
}

/// Even/odd harmonic energy balance in dB over H2..=H16. H1 is excluded — it
/// has no parity partner and would swamp the comparison. Positive = even-heavy
/// (fuller, rounder); negative = odd-heavy (hollower, clarinet-like).
pub fn even_odd_balance_db(amps: &[f32]) -> Option<f32> {
    let (mut even, mut odd) = (0.0f32, 0.0f32);
    for (k, &a) in amps.iter().enumerate().take(16).skip(1) {
        let e = a * a;
        if (k + 1).is_multiple_of(2) {
            even += e;
        } else {
            odd += e;
        }
    }
    (even > 0.0 && odd > 0.0).then(|| 10.0 * (even / odd).log10())
}

/// Percentage of harmonic energy in the singer's-formant band (2.8–3.4 kHz) —
/// the resonance cluster that lets a trained voice project over an ensemble.
pub fn singers_formant_pct(amps: &[f32], f0: f32) -> Option<f32> {
    if f0 <= 0.0 {
        return None;
    }
    let (mut band, mut total) = (0.0f32, 0.0f32);
    for (k, &a) in amps.iter().enumerate() {
        let e = a * a;
        total += e;
        if (2_800.0..=3_400.0).contains(&((k + 1) as f32 * f0)) {
            band += e;
        }
    }
    (total > 1e-12).then(|| band / total * 100.0)
}

/// Rough vocal-range classification from mean fundamental frequency. Ported
/// from the Personal Harmonic Identifier prototype's thresholds; a coarse
/// label, not a diagnosis.
pub fn voice_class(mean_f0: f32) -> &'static str {
    if mean_f0 <= 0.0 || !mean_f0.is_finite() {
        return "—";
    }
    if mean_f0 < 130.0 {
        "Bass"
    } else if mean_f0 < 175.0 {
        "Baritone"
    } else if mean_f0 < 220.0 {
        "Tenor"
    } else if mean_f0 < 290.0 {
        "Alto"
    } else if mean_f0 < 370.0 {
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
    if centroid_hz < 1200.0 {
        "Dark"
    } else if centroid_hz < 2200.0 {
        "Warm"
    } else if centroid_hz < 3200.0 {
        "Balanced"
    } else if centroid_hz < 4400.0 {
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
        && db < -6.0
    {
        return "Hollow · odd-dominant";
    }
    let strong = rel_amps.iter().skip(1).filter(|&&a| a > 0.25).count();
    if strong >= 6 {
        "Rich · complex"
    } else if strong <= 2 {
        "Pure · flute-like"
    } else {
        "Balanced"
    }
}

// ── Classical voiceprint & similarity ────────────────────────────────────────

/// Rough adult-voice population `(mean, std)` for each scalar voiceprint
/// feature, used to z-score before comparison so heterogeneous units (Hz vs
/// dB/oct) contribute on a common scale. Approximate — good enough to weight
/// the features sensibly, not a calibrated model.
const VP_STATS: [(f32, f32); 5] = [
    (500.0, 150.0),  // F1
    (1500.0, 350.0), // F2
    (2600.0, 400.0), // F3
    (1800.0, 700.0), // centroid
    (-9.0, 4.0),     // tilt dB/oct
];

/// Build a pitch-invariant voiceprint from a capture's averaged features.
/// `formants` are F1/F2/F3 (Hz), `profile` the mean relative harmonic
/// amplitudes (H1..), `centroid_hz` the mean spectral centroid. Tilt is
/// derived from the profile. f0 is intentionally not an input.
pub fn build_voiceprint(formants: [Formant; 3], profile: &[f32], centroid_hz: f32) -> Voiceprint {
    let mut p = [0.0f32; 16];
    for (slot, &v) in p.iter_mut().zip(profile) {
        *slot = v;
    }
    Voiceprint {
        formants: [
            formants[0].frequency,
            formants[1].frequency,
            formants[2].frequency,
        ],
        centroid_hz,
        tilt_db_oct: spectral_tilt_db_per_octave(&p).unwrap_or(-9.0),
        profile: p,
    }
}

/// Similarity between two voiceprints in `0..=100`. Two parts:
///   * scalar resonance/brightness (F1/F2/F3, centroid, tilt) — z-scored, with
///     a Gaussian falloff on the RMS z-distance (identical → 1, ~1σ mean
///     difference → ~0.6, ~2σ → ~0.14); weight 0.6.
///   * timbre — cosine similarity of the harmonic profiles; weight 0.4.
///
/// Identical voiceprints score 100. A real, explainable voice-*similarity*
/// score (self-consistency), not forensic speaker recognition — classical
/// formant/timbre features conflate "same vowel/effort" with "same voice".
pub fn voiceprint_similarity(a: &Voiceprint, b: &Voiceprint) -> f32 {
    // Scalar part: RMS distance over z-scored features.
    let a_scalars = [
        a.formants[0],
        a.formants[1],
        a.formants[2],
        a.centroid_hz,
        a.tilt_db_oct,
    ];
    let b_scalars = [
        b.formants[0],
        b.formants[1],
        b.formants[2],
        b.centroid_hz,
        b.tilt_db_oct,
    ];
    let mut sumsq = 0.0f32;
    for i in 0..5 {
        let (_, std) = VP_STATS[i];
        let dz = (a_scalars[i] - b_scalars[i]) / std;
        sumsq += dz * dz;
    }
    let rms_z = (sumsq / 5.0).sqrt();
    let scalar_sim = (-0.5 * rms_z * rms_z).exp();

    // Timbre part: cosine of the (non-negative) harmonic profiles.
    let dot: f32 = a.profile.iter().zip(&b.profile).map(|(x, y)| x * y).sum();
    let na: f32 = a.profile.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.profile.iter().map(|x| x * x).sum::<f32>().sqrt();
    let timbre_sim = if na > 1e-9 && nb > 1e-9 {
        (dot / (na * nb)).clamp(0.0, 1.0)
    } else {
        0.0
    };

    ((0.6 * scalar_sim + 0.4 * timbre_sim) * 100.0).clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn voiceprint_identical_scores_100() {
        let profile: Vec<f32> = (1..=16).map(|k| 1.0 / k as f32).collect();
        let vp = build_voiceprint(fmt3(520.0, 1480.0, 2610.0), &profile, 1750.0);
        let s = voiceprint_similarity(&vp, &vp);
        assert!((s - 100.0).abs() < 0.01, "identical should be 100, got {s}");
    }

    #[test]
    fn voiceprint_different_voice_scores_lower() {
        let prof_a: Vec<f32> = (1..=16).map(|k| 1.0 / k as f32).collect();
        let a = build_voiceprint(fmt3(500.0, 1500.0, 2600.0), &prof_a, 1700.0);

        // A clearly different voice: formants shifted ~2σ, brighter centroid,
        // shallower tilt (via a flatter profile).
        let prof_b: Vec<f32> = (1..=16).map(|k| 1.0 / (k as f32).sqrt()).collect();
        let b = build_voiceprint(fmt3(760.0, 2150.0, 3300.0), &prof_b, 3200.0);

        let self_score = voiceprint_similarity(&a, &a);
        let cross = voiceprint_similarity(&a, &b);
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
        let ref_vp = build_voiceprint(fmt3(500.0, 1500.0, 2600.0), &prof, 1800.0);
        // Near neighbor (small formant drift) vs far (large drift).
        let near = build_voiceprint(fmt3(515.0, 1530.0, 2630.0), &prof, 1850.0);
        let far = build_voiceprint(fmt3(650.0, 1900.0, 3050.0), &prof, 2600.0);
        let s_near = voiceprint_similarity(&ref_vp, &near);
        let s_far = voiceprint_similarity(&ref_vp, &far);
        assert!(
            s_near > s_far,
            "nearer voice {s_near} should score above farther {s_far}"
        );
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
}
