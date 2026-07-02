//! Voice-quality metrics over the f0 contour: vibrato and sustain steadiness.
//!
//! One contour sample arrives per analysis frame (2048 audio samples), so the
//! contour rate is `sample_rate / 2048` (~21.5 Hz @ 44.1 k) and is uniform in
//! AUDIO time regardless of thread scheduling. Nyquist ~10 Hz comfortably
//! covers the 3–9 Hz vibrato band; faster tremor is out of scope.

use crate::types::Vibrato;
use std::f32::consts::TAU;

/// Vibrato search band (Hz), sweep step, and reporting gates.
const VIB_MIN_HZ: f32 = 3.0;
const VIB_MAX_HZ: f32 = 9.0;
const VIB_STEP_HZ: f32 = 0.25;
/// Rates this close to the band edges are likelier drift/noise than vibrato.
const VIB_EDGE_HZ: f32 = 0.25;
/// Minimum fraction of contour variance the fitted sinusoid must explain.
const VIB_MIN_EXPLAINED: f32 = 0.4;
/// Minimum extent (± cents) worth calling vibrato rather than pitch jitter.
const VIB_MIN_EXTENT: f32 = 8.0;

/// Rolling window of recent voiced f0 estimates (~2 s), with vibrato and
/// steadiness estimation. Owned by the analysis loop; one `push` per frame.
pub struct F0Contour {
    /// Voiced f0 values, oldest first; uniform in audio time.
    buf: Vec<f32>,
    cap: usize,
    /// Minimum samples (~1 s of continuous voicing) before reporting anything.
    min_len: usize,
    contour_hz: f32,
    unvoiced_run: usize,
    /// Unvoiced frames tolerated (~250 ms) before the window resets.
    max_gap: usize,
}

impl F0Contour {
    /// `contour_hz` = analysis frames per second of audio (`sample_rate / 2048`).
    pub fn new(contour_hz: f32) -> Self {
        Self {
            buf: Vec::new(),
            cap: ((contour_hz * 2.0).round() as usize).max(8),
            min_len: ((contour_hz).round() as usize).max(8),
            contour_hz,
            unvoiced_run: 0,
            max_gap: ((contour_hz * 0.25).round() as usize).max(1),
        }
    }

    pub fn push(&mut self, f0: f32, voiced: bool) {
        if voiced && f0 > 0.0 {
            self.unvoiced_run = 0;
            if self.buf.len() == self.cap {
                self.buf.remove(0);
            }
            self.buf.push(f0);
        } else {
            self.unvoiced_run += 1;
            if self.unvoiced_run > self.max_gap {
                self.buf.clear();
            }
        }
    }

    /// `(vibrato, steadiness RMS cents)`. Both `None` until ≥ ~1 s of voicing.
    pub fn analyze(&self) -> (Option<Vibrato>, Option<f32>) {
        let n = self.buf.len();
        if n < self.min_len {
            return (None, None);
        }

        // Cents relative to the window's log-domain mean pitch, drift removed
        // so a slow slide doesn't read as oscillation or unsteadiness.
        let mean_log = self.buf.iter().map(|f| f.ln()).sum::<f32>() / n as f32;
        let cents: Vec<f32> = self
            .buf
            .iter()
            .map(|f| 1200.0 * (f.ln() - mean_log) / std::f32::consts::LN_2)
            .collect();
        let d = detrend(&cents);

        let var = d.iter().map(|x| x * x).sum::<f32>() / n as f32;
        if var < 1e-4 {
            // Dead-flat sustain: no vibrato, essentially perfect steadiness.
            return (None, Some(var.sqrt()));
        }

        // Stage 1: Hann-windowed DFT sweep locates the dominant rate.
        // Stage 2: exact least-squares sinusoid fit at that rate gives
        // amplitude, phase, and the residual for the explained-variance gate.
        let rate = dominant_rate(&d, self.contour_hz);
        let (extent, residual) = sinusoid_fit(&d, rate / self.contour_hz);
        let res_var = residual.iter().map(|x| x * x).sum::<f32>() / n as f32;
        let explained = 1.0 - res_var / var;

        let is_vibrato = explained >= VIB_MIN_EXPLAINED
            && extent >= VIB_MIN_EXTENT
            && rate >= VIB_MIN_HZ + VIB_EDGE_HZ
            && rate <= VIB_MAX_HZ - VIB_EDGE_HZ;

        if is_vibrato {
            (
                Some(Vibrato {
                    rate_hz: rate,
                    extent_cents: extent,
                }),
                Some(res_var.sqrt()),
            )
        } else {
            (None, Some(var.sqrt()))
        }
    }
}

/// Least-squares removal of mean + linear trend.
fn detrend(x: &[f32]) -> Vec<f32> {
    let n = x.len() as f32;
    let mx = (n - 1.0) / 2.0;
    let my = x.iter().sum::<f32>() / n;
    let (mut num, mut den) = (0.0f32, 0.0f32);
    for (i, &y) in x.iter().enumerate() {
        let dx = i as f32 - mx;
        num += dx * (y - my);
        den += dx * dx;
    }
    let slope = if den > 1e-9 { num / den } else { 0.0 };
    x.iter()
        .enumerate()
        .map(|(i, &y)| y - (my + slope * (i as f32 - mx)))
        .collect()
}

/// Peak of a Hann-windowed DFT magnitude sweep over the vibrato band,
/// parabolic-refined between the 0.25 Hz bins.
fn dominant_rate(d: &[f32], fs: f32) -> f32 {
    let n = d.len();
    let m = (n - 1) as f32;
    let win: Vec<f32> = (0..n)
        .map(|i| 0.5 - 0.5 * (TAU * i as f32 / m).cos())
        .collect();

    let mut mags: Vec<(f32, f32)> = Vec::new();
    let mut f = VIB_MIN_HZ;
    while f <= VIB_MAX_HZ + 1e-6 {
        let (mut re, mut im) = (0.0f32, 0.0f32);
        for (i, &x) in d.iter().enumerate() {
            let th = TAU * f * i as f32 / fs;
            let w = win[i] * x;
            re += w * th.cos();
            im -= w * th.sin();
        }
        mags.push((f, re * re + im * im));
        f += VIB_STEP_HZ;
    }

    let best = mags
        .iter()
        .enumerate()
        .max_by(|a, b| {
            a.1.1
                .partial_cmp(&b.1.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0);

    // Parabolic refinement between bins where neighbors exist.
    if best > 0 && best + 1 < mags.len() {
        let (y0, y1, y2) = (mags[best - 1].1, mags[best].1, mags[best + 1].1);
        let denom = y0 - 2.0 * y1 + y2;
        if denom.abs() > 1e-12 {
            let delta = ((y0 - y2) / (2.0 * denom)).clamp(-0.5, 0.5);
            return mags[best].0 + delta * VIB_STEP_HZ;
        }
    }
    mags[best].0
}

/// Exact least-squares fit of `a·cos(ωi) + b·sin(ωi)` at normalized frequency
/// `f_norm` (cycles per sample). Returns `(amplitude, residual)`.
fn sinusoid_fit(d: &[f32], f_norm: f32) -> (f32, Vec<f32>) {
    let omega = TAU * f_norm;
    let (mut scc, mut sss, mut scs) = (0.0f32, 0.0f32, 0.0f32);
    let (mut sc, mut ss) = (0.0f32, 0.0f32);
    for (i, &x) in d.iter().enumerate() {
        let th = omega * i as f32;
        let (c, s) = (th.cos(), th.sin());
        scc += c * c;
        sss += s * s;
        scs += c * s;
        sc += x * c;
        ss += x * s;
    }
    let det = scc * sss - scs * scs;
    if det.abs() < 1e-9 {
        return (0.0, d.to_vec());
    }
    let a = (sc * sss - ss * scs) / det;
    let b = (ss * scc - sc * scs) / det;
    let residual: Vec<f32> = d
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let th = omega * i as f32;
            x - a * th.cos() - b * th.sin()
        })
        .collect();
    ((a * a + b * b).sqrt(), residual)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FS: f32 = 21.5; // contour rate @ 44.1 kHz / 2048

    /// Push `secs` of voiced contour: f0 with optional vibrato in cents.
    fn feed(c: &mut F0Contour, f0: f32, secs: f32, vib_hz: f32, vib_cents: f32) {
        let n = (secs * FS) as usize;
        for i in 0..n {
            let t = i as f32 / FS;
            let cents = vib_cents * (TAU * vib_hz * t).sin();
            c.push(f0 * 2f32.powf(cents / 1200.0), true);
        }
    }

    #[test]
    fn detects_known_vibrato() {
        let mut c = F0Contour::new(FS);
        feed(&mut c, 220.0, 2.5, 5.5, 50.0);
        let (vib, steady) = c.analyze();
        let v = vib.expect("vibrato should be detected");
        assert!((v.rate_hz - 5.5).abs() < 0.3, "rate {}", v.rate_hz);
        assert!(
            (v.extent_cents - 50.0).abs() < 7.5,
            "extent {}",
            v.extent_cents
        );
        // Residual after removing a clean sinusoid should be small.
        assert!(steady.unwrap() < 10.0, "steadiness {:?}", steady);
    }

    #[test]
    fn flat_sustain_is_steady_with_no_vibrato() {
        let mut c = F0Contour::new(FS);
        feed(&mut c, 220.0, 2.0, 0.0, 0.0);
        let (vib, steady) = c.analyze();
        assert!(vib.is_none());
        assert!(steady.unwrap() < 1.0, "steadiness {:?}", steady);
    }

    #[test]
    fn linear_drift_is_not_vibrato() {
        let mut c = F0Contour::new(FS);
        // Slide up 100 cents/s for 2 s — drift, not oscillation.
        for i in 0..(2.0 * FS) as usize {
            let t = i as f32 / FS;
            c.push(220.0 * 2f32.powf(100.0 * t / 1200.0), true);
        }
        let (vib, steady) = c.analyze();
        assert!(vib.is_none(), "drift misread as vibrato: {vib:?}");
        assert!(steady.unwrap() < 5.0, "detrended drift {:?}", steady);
    }

    #[test]
    fn aperiodic_wobble_reports_honest_rms() {
        let mut c = F0Contour::new(FS);
        for i in 0..(2.0 * FS) as usize {
            let x = i as f32;
            // Deterministic pseudo-noise, ~±30 cents, no dominant rate.
            let cents = (((x * 12.9898).sin() * 43758.5).fract() - 0.5) * 60.0;
            c.push(220.0 * 2f32.powf(cents / 1200.0), true);
        }
        let (vib, steady) = c.analyze();
        assert!(vib.is_none(), "noise misread as vibrato: {vib:?}");
        assert!(steady.unwrap() > 5.0, "noise RMS {:?}", steady);
    }

    #[test]
    fn too_little_voicing_reports_nothing() {
        let mut c = F0Contour::new(FS);
        feed(&mut c, 220.0, 0.5, 5.5, 50.0);
        assert_eq!(c.analyze().0, None);
        assert!(c.analyze().1.is_none());
    }

    #[test]
    fn long_unvoiced_gap_resets_the_window() {
        let mut c = F0Contour::new(FS);
        feed(&mut c, 220.0, 1.5, 5.5, 50.0);
        assert!(c.analyze().0.is_some());
        for _ in 0..(0.4 * FS) as usize {
            c.push(0.0, false); // ~400 ms silence > 250 ms tolerance
        }
        let (vib, steady) = c.analyze();
        assert!(vib.is_none());
        assert!(steady.is_none());
    }
}
