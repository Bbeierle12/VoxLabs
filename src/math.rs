use crate::types::{Formant, MAX_PARTIALS};
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
