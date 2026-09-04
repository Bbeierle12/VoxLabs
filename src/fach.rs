//! Voice-part (Fach) measurements from the "Fach, Measured" research brief.
//!
//! Every function here is a small, testable definition of a quantity the
//! literature ranked as carrying voice-classification information — none of
//! them is a classifier. They run on the same frames the phone analyzes
//! (see `frame::FrameAnalyzer`) and feed the study harness first, the app's
//! evidence panel later.
//!
//!   * [`band_half_energy`] — "frequency of half energy" (FHE) and centroid
//!     of the singer's-formant band (Müller et al. 2022: 2.0–3.6 kHz for
//!     male voices, 2.3–4.5 kHz for sopranos; bass 2384 · baritone 2454 ·
//!     tenor 2705 · soprano 3092 Hz). Designed to be independent of pitch,
//!     vowel and loudness because it is a *position within* the band, not
//!     a level.
//!   * [`Ltas`] — long-term average spectrum with ISO third-octave band
//!     levels 100 Hz–8 kHz, mean-normalized (channel-robust-ish).
//!   * [`cluster_stats`] — singer's-formant cluster presence: peak,
//!     prominence over the valley below it, −3 dB width (tenors < 1 kHz,
//!     sopranos ≥ 2 kHz or no cluster at all — Weiss 2001, Sundberg 2001).
//!   * [`tessitura`] — duration-weighted f0 percentiles; the strongest
//!     classifier in the literature is *performed* range (η ≈ 0.98).
//!   * [`TurnoverTracker`] — the pitch at which A2/A1 crosses 0 dB on a
//!     glide (the objective acoustic passaggio, Neumann 2005 / Bozeman).
//!   * [`dominant_harmonic`] — which of H1/H2/H3 wins: R1:f0 tuning (H1)
//!     for sopranos/altos vs R1:2f0 / 3f0 (H2/H3) for tenors/baritones in
//!     the overlap zone (Henrich, Smith & Wolfe 2011).
//!   * [`RegisterDetector`] — coincident f0 jump + H1–H2 / CPP / jitter
//!     change on a glide; experimental, flagged as such.
//!
//! Honesty boundary: adjacent parts overlap by design of the categories
//! (baritone–bass d ≈ 0.4 on the best measure; anatomy itself predicts
//! soprano vs alto at 68–74 %). These numbers are evidence to be shown with
//! their overlap, never a label to be printed.

use crate::math;
use rustfft::num_complex::Complex;
use std::f32::consts::PI;

/// Singer's-formant analysis bands (Hz), male and female (Müller 2022).
pub const FHE_BAND_MALE: (f32, f32) = (2000.0, 3600.0);
pub const FHE_BAND_FEMALE: (f32, f32) = (2300.0, 4500.0);
/// Cluster search window (Hz) and the floor below which the valley is sought.
pub const CLUSTER_LO_HZ: f32 = 2200.0;
pub const CLUSTER_HI_HZ: f32 = 3400.0;
pub const CLUSTER_VALLEY_FROM_HZ: f32 = 1500.0;
pub const CLUSTER_WIDTH_CEIL_HZ: f32 = 5000.0;
/// ISO third-octave band centres, 100 Hz – 8 kHz.
pub const THIRD_OCTAVE_CENTERS: [f32; 20] = [
    100.0, 125.0, 160.0, 200.0, 250.0, 315.0, 400.0, 500.0, 630.0, 800.0, 1000.0, 1250.0, 1600.0,
    2000.0, 2500.0, 3150.0, 4000.0, 5000.0, 6300.0, 8000.0,
];
/// Turnover hysteresis: A2/A1 must sit at or beyond ±this to count as
/// "below" / "above", so a value wobbling around 0 dB fires once.
pub const TURNOVER_HYSTERESIS_DB: f32 = 3.0;
/// Register-event heuristics: minimum f0 jump between consecutive voiced
/// frames, and the accompanying source-change thresholds.
pub const REGISTER_MIN_JUMP_ST: f32 = 2.0;
pub const REGISTER_H1H2_DELTA_DB: f32 = 4.0;
pub const REGISTER_CPP_DROP_DB: f32 = 2.0;
pub const REGISTER_JITTER_PCT: f32 = 1.5;

// ─── Power spectrum ─────────────────────────────────────────────────────────

/// One-sided power spectrum of a Hann-windowed frame (linear power per bin).
#[derive(Clone, Debug)]
pub struct PowerSpectrum {
    pub bin_hz: f32,
    pub power: Vec<f32>,
}

impl PowerSpectrum {
    pub fn bin_of(&self, hz: f32) -> usize {
        ((hz / self.bin_hz).round() as usize).min(self.power.len().saturating_sub(1))
    }
}

/// Hann-windowed one-sided power spectrum. `None` for tiny or silent frames.
pub fn power_spectrum(frame: &[f32], sample_rate: f32) -> Option<PowerSpectrum> {
    let n = frame.len();
    if n < 256 || sample_rate <= 0.0 {
        return None;
    }
    if frame.iter().map(|s| s * s).sum::<f32>() < 1e-12 {
        return None;
    }
    let fft = math::fft_forward(n);
    let m = (n - 1) as f32;
    let mut buf: Vec<Complex<f32>> = frame
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let w = 0.5 - 0.5 * (2.0 * PI * i as f32 / m).cos();
            Complex::new(x * w, 0.0)
        })
        .collect();
    fft.process(&mut buf);
    let power: Vec<f32> = buf[..=n / 2].iter().map(|c| c.norm_sqr()).collect();
    Some(PowerSpectrum {
        bin_hz: sample_rate / n as f32,
        power,
    })
}

// ─── FHE / band centroid ────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BandStats {
    /// Frequency at which half the band's energy has accumulated.
    pub fhe_hz: f32,
    /// Power-weighted mean frequency of the band.
    pub centroid_hz: f32,
    /// Total band power (linear) — for gating, not for comparison.
    pub energy: f32,
}

/// FHE + centroid of `[lo_hz, hi_hz]`. `None` when the band holds no energy.
pub fn band_half_energy(ps: &PowerSpectrum, lo_hz: f32, hi_hz: f32) -> Option<BandStats> {
    let lo = ps.bin_of(lo_hz);
    let hi = ps.bin_of(hi_hz);
    if hi <= lo {
        return None;
    }
    let total: f64 = ps.power[lo..=hi].iter().map(|&p| p as f64).sum();
    if total <= 1e-18 {
        return None;
    }
    let mut acc = 0.0f64;
    let mut fhe = hi_hz;
    for (i, &p) in ps.power[lo..=hi].iter().enumerate() {
        let next = acc + p as f64;
        if next >= total * 0.5 {
            // Interpolate inside the crossing bin.
            let frac = if p > 0.0 {
                ((total * 0.5 - acc) / p as f64) as f32
            } else {
                0.5
            };
            fhe = ((lo + i) as f32 + frac - 0.5) * ps.bin_hz;
            break;
        }
        acc = next;
    }
    let centroid = ps.power[lo..=hi]
        .iter()
        .enumerate()
        .map(|(i, &p)| (lo + i) as f64 * ps.bin_hz as f64 * p as f64)
        .sum::<f64>()
        / total;
    Some(BandStats {
        fhe_hz: fhe,
        centroid_hz: centroid as f32,
        energy: total as f32,
    })
}

// ─── LTAS ───────────────────────────────────────────────────────────────────

/// Long-term average power spectrum accumulator.
#[derive(Clone, Debug)]
pub struct Ltas {
    bin_hz: f32,
    sum: Vec<f64>,
    frames: usize,
}

impl Ltas {
    pub fn new(n_bins: usize, bin_hz: f32) -> Self {
        Self {
            bin_hz,
            sum: vec![0.0; n_bins],
            frames: 0,
        }
    }

    pub fn push(&mut self, ps: &PowerSpectrum) {
        if ps.power.len() != self.sum.len() {
            return; // mismatched frame size — never silently mix
        }
        for (s, &p) in self.sum.iter_mut().zip(&ps.power) {
            *s += p as f64;
        }
        self.frames += 1;
    }

    pub fn frames(&self) -> usize {
        self.frames
    }

    pub fn bin_hz(&self) -> f32 {
        self.bin_hz
    }

    /// Mean linear power per bin. `None` before any frame.
    pub fn mean_power(&self) -> Option<Vec<f32>> {
        (self.frames > 0).then(|| {
            self.sum
                .iter()
                .map(|&s| (s / self.frames as f64) as f32)
                .collect()
        })
    }

    /// ISO third-octave band levels in dB, mean-normalized across bands (so
    /// a flat gain change cancels). `None` before any frame.
    pub fn third_octave_db(&self) -> Option<[f32; 20]> {
        let mean = self.mean_power()?;
        let mut out = [0.0f32; 20];
        for (k, &fc) in THIRD_OCTAVE_CENTERS.iter().enumerate() {
            let lo = fc / 2f32.powf(1.0 / 6.0);
            let hi = fc * 2f32.powf(1.0 / 6.0);
            let b0 = ((lo / self.bin_hz).ceil() as usize).min(mean.len().saturating_sub(1));
            let b1 = ((hi / self.bin_hz).floor() as usize).min(mean.len().saturating_sub(1));
            let p: f64 = if b1 >= b0 {
                mean[b0..=b1].iter().map(|&v| v as f64).sum()
            } else {
                mean[b0] as f64
            };
            out[k] = 10.0 * (p.max(1e-18)).log10() as f32;
        }
        let avg = out.iter().sum::<f32>() / out.len() as f32;
        for v in out.iter_mut() {
            *v -= avg;
        }
        Some(out)
    }
}

// ─── Singer's-formant cluster ───────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClusterStats {
    pub peak_hz: f32,
    /// Peak level over the lowest level between [`CLUSTER_VALLEY_FROM_HZ`]
    /// and the peak, dB.
    pub prominence_db: f32,
    /// Contiguous −3 dB span around the peak, Hz.
    pub width_hz: f32,
}

/// Triangular smoothing of a power spectrum over ±`half_width_hz`, in
/// linear power. A sustained vowel's mean spectrum is a line spectrum at
/// k·f0, and [`cluster_stats`] on it would just pick the loudest harmonic
/// near 2.8 kHz and report a one-bin width. Smoothing with a kernel one
/// harmonic spacing wide (half-width ≈ f0) turns the lines into the
/// envelope the cluster statistics are meant to describe, with almost no
/// residual ripple (a box that wide always straddles the same number of
/// lines; the second pass removes what is left).
pub fn smooth_power(mean_power: &[f32], bin_hz: f32, half_width_hz: f32) -> Vec<f32> {
    let n = mean_power.len();
    if n == 0 || bin_hz <= 0.0 {
        return Vec::new();
    }
    let r = ((0.5 * half_width_hz / bin_hz).round() as usize).min(n);
    if r == 0 {
        return mean_power.to_vec();
    }
    let pass = |x: &[f32]| -> Vec<f32> {
        // Prefix sums for an O(n) box filter.
        let mut acc = Vec::with_capacity(n + 1);
        acc.push(0.0f64);
        for &p in x {
            let last = *acc.last().unwrap();
            acc.push(last + p as f64);
        }
        (0..n)
            .map(|i| {
                let lo = i.saturating_sub(r);
                let hi = (i + r + 1).min(n);
                ((acc[hi] - acc[lo]) / (hi - lo) as f64) as f32
            })
            .collect()
    };
    pass(&pass(mean_power))
}

/// Cluster statistics on a mean power spectrum. `None` if the window is
/// empty or the spectrum is silent. Callers measuring a sustained vowel
/// should pass the spectrum through [`smooth_power`] first.
pub fn cluster_stats(mean_power: &[f32], bin_hz: f32) -> Option<ClusterStats> {
    if mean_power.is_empty() || bin_hz <= 0.0 {
        return None;
    }
    let db: Vec<f32> = mean_power
        .iter()
        .map(|&p| 10.0 * p.max(1e-18).log10())
        .collect();
    let bin = |hz: f32| ((hz / bin_hz).round() as usize).min(db.len() - 1);
    let (lo, hi) = (bin(CLUSTER_LO_HZ), bin(CLUSTER_HI_HZ));
    if hi <= lo {
        return None;
    }
    let (peak_i, peak_db) =
        db[lo..=hi]
            .iter()
            .enumerate()
            .fold(
                (lo, f32::MIN),
                |(bi, bv), (i, &v)| {
                    if v > bv { (lo + i, v) } else { (bi, bv) }
                },
            );
    if peak_db <= -170.0 {
        return None;
    }
    let valley_from = bin(CLUSTER_VALLEY_FROM_HZ);
    let valley_db = db[valley_from..=peak_i]
        .iter()
        .cloned()
        .fold(f32::MAX, f32::min);
    // −3 dB width: walk out from the peak while level ≥ peak − 3.
    let ceil = bin(CLUSTER_WIDTH_CEIL_HZ);
    let thr = peak_db - 3.0;
    let mut left = peak_i;
    while left > valley_from && db[left - 1] >= thr {
        left -= 1;
    }
    let mut right = peak_i;
    while right < ceil && db[right + 1] >= thr {
        right += 1;
    }
    Some(ClusterStats {
        peak_hz: peak_i as f32 * bin_hz,
        prominence_db: peak_db - valley_db,
        width_hz: (right - left + 1) as f32 * bin_hz,
    })
}

// ─── Tessitura ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tessitura {
    pub p10: f32,
    pub p25: f32,
    pub p50: f32,
    pub p75: f32,
    pub p90: f32,
    /// Robust extremes (2nd / 98th percentile), Hz.
    pub lo: f32,
    pub hi: f32,
    pub n: usize,
}

/// Percentile (0–100) of a sorted slice, linearly interpolated.
pub fn percentile(sorted: &[f32], pct: f32) -> Option<f32> {
    if sorted.is_empty() {
        return None;
    }
    let pos = (pct / 100.0).clamp(0.0, 1.0) * (sorted.len() - 1) as f32;
    let i = pos.floor() as usize;
    let frac = pos - i as f32;
    let a = sorted[i];
    let b = sorted[(i + 1).min(sorted.len() - 1)];
    Some(a + (b - a) * frac)
}

/// Median of an unsorted slice.
pub fn median(v: &[f32]) -> Option<f32> {
    let mut s: Vec<f32> = v.iter().cloned().filter(|x| x.is_finite()).collect();
    if s.is_empty() {
        return None;
    }
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    percentile(&s, 50.0)
}

/// Duration-weighted (one value per voiced frame) f0 percentiles.
pub fn tessitura(f0s: &[f32]) -> Option<Tessitura> {
    let mut s: Vec<f32> = f0s
        .iter()
        .cloned()
        .filter(|&f| f > 0.0 && f.is_finite())
        .collect();
    if s.len() < 10 {
        return None;
    }
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(Tessitura {
        p10: percentile(&s, 10.0)?,
        p25: percentile(&s, 25.0)?,
        p50: percentile(&s, 50.0)?,
        p75: percentile(&s, 75.0)?,
        p90: percentile(&s, 90.0)?,
        lo: percentile(&s, 2.0)?,
        hi: percentile(&s, 98.0)?,
        n: s.len(),
    })
}

/// MIDI-style semitone number (A4 = 69).
pub fn hz_to_semitone(hz: f32) -> f32 {
    69.0 + 12.0 * (hz / 440.0).log2()
}

/// Scientific pitch name of the nearest semitone, e.g. "C4", "F#4".
pub fn note_name(hz: f32) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    if hz <= 0.0 || !hz.is_finite() {
        return "—".into();
    }
    let st = hz_to_semitone(hz).round() as i32;
    let octave = st.div_euclid(12) - 1;
    format!("{}{}", NAMES[st.rem_euclid(12) as usize], octave)
}

// ─── Turnover (acoustic passaggio) ──────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TurnoverEvent {
    /// Interpolated f0 at the A2/A1 = 0 dB crossing.
    pub f0_hz: f32,
    /// Pitch direction at the crossing: true on an ascending glide. Real
    /// voices turn over at a slightly different pitch going up than coming
    /// down, so the two are reported separately.
    pub rising: bool,
    /// Which harmonic took over: true when H1 became dominant (A2/A1 fell
    /// through 0 dB — the physical ascending turnover, H2 leaving F1),
    /// false when H2 took over (A2/A1 rose through 0 dB).
    pub to_h1: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Region {
    Unknown,
    Below,
    Above,
}

/// Tracks the A2/A1 sign with hysteresis and records each crossing with
/// the pitch it happened at. The crossing pitch is interpolated at the
/// actual 0 dB sign change; the hysteresis only decides whether that
/// crossing is *confirmed* (the ratio went on to ±3 dB) or was a wobble.
#[derive(Clone, Debug, Default)]
pub struct TurnoverTracker {
    region: Option<Region>,
    /// Previous frame's (f0, a2a1).
    prev: Option<(f32, f32)>,
    /// The most recent unconfirmed 0 dB crossing.
    pending: Option<TurnoverEvent>,
    events: Vec<TurnoverEvent>,
}

impl TurnoverTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed one frame. Unvoiced frames (`None` f0) reset the tracker so a
    /// crossing is never inferred across a gap.
    pub fn push(&mut self, f0: Option<f32>, a2_a1_db: Option<f32>) {
        let (Some(f0), Some(r)) = (f0, a2_a1_db) else {
            self.region = Some(Region::Unknown);
            self.prev = None;
            self.pending = None;
            return;
        };
        // Exact sign change between consecutive frames: remember where.
        if let Some((pf0, pr)) = self.prev
            && (pr < 0.0) != (r < 0.0)
        {
            let t = if (r - pr).abs() > 1e-6 {
                ((0.0 - pr) / (r - pr)).clamp(0.0, 1.0)
            } else {
                0.5
            };
            self.pending = Some(TurnoverEvent {
                f0_hz: pf0 + t * (f0 - pf0),
                rising: f0 >= pf0,
                to_h1: r < pr,
            });
        }
        let now = if r <= -TURNOVER_HYSTERESIS_DB {
            Region::Below
        } else if r >= TURNOVER_HYSTERESIS_DB {
            Region::Above
        } else {
            self.region.unwrap_or(Region::Unknown)
        };
        let prev_region = self.region.unwrap_or(Region::Unknown);
        if prev_region != Region::Unknown && now != Region::Unknown && now != prev_region {
            // Confirmed: the ratio moved a full hysteresis band across zero.
            let ev = self.pending.take().unwrap_or(TurnoverEvent {
                f0_hz: f0,
                rising: self.prev.is_none_or(|(pf0, _)| f0 >= pf0),
                to_h1: now == Region::Below,
            });
            self.events.push(ev);
        }
        self.region = Some(now);
        self.prev = Some((f0, r));
    }

    pub fn events(&self) -> &[TurnoverEvent] {
        &self.events
    }
}

// ─── Dominant harmonic ──────────────────────────────────────────────────────

/// Which of H1, H2, H3 is strongest (1-based). `None` when all are zero.
pub fn dominant_harmonic(amps: &[f32]) -> Option<u8> {
    let top: Vec<f32> = amps.iter().take(3).cloned().collect();
    if top.iter().all(|&a| a <= 0.0) {
        return None;
    }
    let (i, _) =
        top.iter().enumerate().fold(
            (0usize, f32::MIN),
            |(bi, bv), (i, &v)| if v > bv { (i, v) } else { (bi, bv) },
        );
    Some(i as u8 + 1)
}

// ─── Register events (experimental) ─────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegisterEvent {
    pub f0_from: f32,
    pub f0_to: f32,
    pub jump_semitones: f32,
    pub h1h2_delta_db: Option<f32>,
    pub cpp_delta_db: Option<f32>,
    pub jitter_pct: Option<f32>,
}

#[derive(Clone, Copy, Debug)]
struct RegFrame {
    f0: f32,
    h1h2: Option<f32>,
    cpp: Option<f32>,
}

/// Flags a consecutive-frame pitch jump that coincides with a source change.
/// Experimental: it will also fire on deliberate leaps; the study reads it
/// on glides only, where a jump can only be a register event.
#[derive(Clone, Debug, Default)]
pub struct RegisterDetector {
    prev: Option<RegFrame>,
}

impl RegisterDetector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(
        &mut self,
        f0: Option<f32>,
        h1h2_db: Option<f32>,
        cpp_db: Option<f32>,
        jitter_pct: Option<f32>,
    ) -> Option<RegisterEvent> {
        let Some(f0) = f0 else {
            self.prev = None;
            return None;
        };
        let cur = RegFrame {
            f0,
            h1h2: h1h2_db,
            cpp: cpp_db,
        };
        let out = self.prev.and_then(|p| {
            let jump = hz_to_semitone(f0) - hz_to_semitone(p.f0);
            if jump.abs() < REGISTER_MIN_JUMP_ST {
                return None;
            }
            let dh = match (cur.h1h2, p.h1h2) {
                (Some(a), Some(b)) => Some(a - b),
                _ => None,
            };
            let dc = match (cur.cpp, p.cpp) {
                (Some(a), Some(b)) => Some(a - b),
                _ => None,
            };
            let source_change = dh.is_some_and(|d| d.abs() >= REGISTER_H1H2_DELTA_DB)
                || dc.is_some_and(|d| d <= -REGISTER_CPP_DROP_DB)
                || jitter_pct.is_some_and(|j| j >= REGISTER_JITTER_PCT);
            source_change.then_some(RegisterEvent {
                f0_from: p.f0,
                f0_to: f0,
                jump_semitones: jump,
                h1h2_delta_db: dh,
                cpp_delta_db: dc,
                jitter_pct,
            })
        });
        self.prev = Some(cur);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: f32 = 48_000.0;
    const N: usize = 2048;

    /// Harmonic series at `f0` with per-harmonic amplitude from `amp(k)`.
    fn synth(f0: f32, n: usize, amp: impl Fn(usize) -> f32) -> Vec<f32> {
        (0..n)
            .map(|i| {
                let t = i as f32 / SR;
                (1..=40)
                    .map(|k| {
                        let f = f0 * k as f32;
                        if f >= SR / 2.0 {
                            0.0
                        } else {
                            amp(k) * (2.0 * PI * f * t).sin()
                        }
                    })
                    .sum::<f32>()
                    * 0.05
            })
            .collect()
    }

    #[test]
    fn fhe_lands_on_a_synthetic_cluster() {
        // f0 = 200 Hz; harmonics 13–15 (2600–3000 Hz) at equal amplitude,
        // ~28 dB over the rest. Half the band's energy accumulates inside
        // the middle harmonic, i.e. ~2800 Hz. (A cluster that FALLS with the
        // series puts the median on its first harmonic — that is correct
        // behaviour, and why FHE is a position statistic, not a peak.)
        let sig = synth(
            200.0,
            N,
            |k| if (13..=15).contains(&k) { 0.5 } else { 0.02 },
        );
        let ps = power_spectrum(&sig, SR).unwrap();
        let b = band_half_energy(&ps, FHE_BAND_MALE.0, FHE_BAND_MALE.1).unwrap();
        assert!((b.fhe_hz - 2800.0).abs() < 60.0, "fhe {}", b.fhe_hz);
        assert!(
            (b.centroid_hz - 2800.0).abs() < 80.0,
            "centroid {}",
            b.centroid_hz
        );
    }

    #[test]
    fn fhe_is_loudness_invariant() {
        let sig = synth(220.0, N, |k| (0.85f32).powi(k as i32));
        let quiet: Vec<f32> = sig.iter().map(|s| s * 0.1).collect();
        let a = band_half_energy(&power_spectrum(&sig, SR).unwrap(), 2000.0, 3600.0).unwrap();
        let b = band_half_energy(&power_spectrum(&quiet, SR).unwrap(), 2000.0, 3600.0).unwrap();
        assert!((a.fhe_hz - b.fhe_hz).abs() < 1.0);
    }

    #[test]
    fn cluster_stats_find_a_gaussian_bump() {
        let bin_hz = SR / N as f32;
        let n_bins = N / 2 + 1;
        // Flat floor at −40 dB with a +20 dB Gaussian bump at 2.8 kHz, σ = 150 Hz.
        let mean: Vec<f32> = (0..n_bins)
            .map(|i| {
                let f = i as f32 * bin_hz;
                let bump = 20.0 * (-(f - 2800.0).powi(2) / (2.0 * 150f32.powi(2))).exp();
                10f32.powf((-40.0 + bump) / 10.0)
            })
            .collect();
        let c = cluster_stats(&mean, bin_hz).unwrap();
        assert!(
            (c.peak_hz - 2800.0).abs() <= bin_hz * 1.5,
            "peak {}",
            c.peak_hz
        );
        assert!(
            c.prominence_db > 18.0 && c.prominence_db < 22.0,
            "prom {}",
            c.prominence_db
        );
        // The bump is Gaussian in dB: level drops 3 dB where
        // 20·exp(−x²/2σ²) = 17 → x = σ·√(2 ln(20/17)) ≈ 0.57σ ≈ 85 Hz each
        // side → ~171 Hz full width (bin-quantized to 23.4 Hz).
        assert!((c.width_hz - 171.0).abs() < 50.0, "width {}", c.width_hz);
    }

    #[test]
    fn cluster_stats_on_a_flat_spectrum_has_no_prominence() {
        let bin_hz = SR / N as f32;
        let mean = vec![1e-4f32; N / 2 + 1];
        let c = cluster_stats(&mean, bin_hz).unwrap();
        assert!(c.prominence_db.abs() < 0.01);
    }

    #[test]
    fn third_octave_marks_the_loud_band() {
        let bin_hz = SR / N as f32;
        let mut l = Ltas::new(N / 2 + 1, bin_hz);
        let sig = synth(250.0, N, |k| if k == 10 { 5.0 } else { 0.05 }); // 2.5 kHz loud
        l.push(&power_spectrum(&sig, SR).unwrap());
        let bands = l.third_octave_db().unwrap();
        let (best, _) =
            bands.iter().enumerate().fold(
                (0, f32::MIN),
                |(bi, bv), (i, &v)| if v > bv { (i, v) } else { (bi, bv) },
            );
        assert_eq!(THIRD_OCTAVE_CENTERS[best], 2500.0);
        assert!(bands.iter().sum::<f32>().abs() < 1e-3, "mean-normalized");
    }

    #[test]
    fn smoothing_turns_a_line_spectrum_into_its_envelope() {
        // Harmonics of 200 Hz under a Gaussian envelope centred on 2800 Hz.
        let bin_hz = 23.4375; // 48 kHz / 2048
        let n = 1025;
        let mut lines = vec![0.0f32; n];
        for k in 1..60 {
            let f = 200.0 * k as f32;
            let i = (f / bin_hz).round() as usize;
            if i < n {
                lines[i] = (-((f - 2800.0) / 400.0).powi(2)).exp();
            }
        }
        // Unsmoothed: the loudest harmonic (2800 exactly) with a one-bin width.
        let raw = cluster_stats(&lines, bin_hz).unwrap();
        assert!(raw.width_hz < 2.0 * bin_hz, "raw width {}", raw.width_hz);
        // Smoothed over ±f0: a contiguous envelope several hundred Hz wide
        // (the Gaussian's own −3 dB width is 666 Hz), still peaking near 2800.
        let sm = smooth_power(&lines, bin_hz, 200.0);
        let env = cluster_stats(&sm, bin_hz).unwrap();
        assert!((env.peak_hz - 2800.0).abs() < 150.0, "peak {}", env.peak_hz);
        assert!(env.width_hz > 500.0, "smoothed width {}", env.width_hz);
        assert!(env.prominence_db > 10.0);
        assert!(smooth_power(&[], bin_hz, 100.0).is_empty());
        assert_eq!(smooth_power(&lines, bin_hz, 0.0), lines);
    }

    #[test]
    fn tessitura_percentiles() {
        let f0s: Vec<f32> = (0..101).map(|i| 100.0 + i as f32).collect(); // 100..200
        let t = tessitura(&f0s).unwrap();
        assert!((t.p50 - 150.0).abs() < 0.01);
        assert!((t.p10 - 110.0).abs() < 0.01);
        assert!((t.p90 - 190.0).abs() < 0.01);
        assert!((t.lo - 102.0).abs() < 0.01 && (t.hi - 198.0).abs() < 0.01);
        assert_eq!(t.n, 101);
        assert!(tessitura(&[0.0; 20]).is_none());
    }

    #[test]
    fn note_names() {
        assert_eq!(note_name(440.0), "A4");
        assert_eq!(note_name(261.63), "C4");
        assert_eq!(note_name(82.41), "E2");
        assert_eq!(note_name(1046.5), "C6");
        assert_eq!(note_name(0.0), "—");
    }

    #[test]
    fn turnover_fires_once_at_the_crossing_on_a_glide() {
        // f0 200 → 400 over 100 frames; A2/A1 = (300 − f0)/10 dB crosses 0
        // at 300, falling as pitch rises (H2 leaves F1, H1 takes over).
        let mut t = TurnoverTracker::new();
        for i in 0..=100 {
            let f0 = 200.0 + 2.0 * i as f32;
            t.push(Some(f0), Some((300.0 - f0) / 10.0));
        }
        let ev = t.events();
        assert_eq!(ev.len(), 1, "events {ev:?}");
        assert!(
            (ev[0].f0_hz - 300.0).abs() < 4.0,
            "crossing {}",
            ev[0].f0_hz
        );
        assert!(ev[0].rising && ev[0].to_h1);
        // The same glide back down: descending crossing, H2 takes over.
        for i in (0..=100).rev() {
            let f0 = 200.0 + 2.0 * i as f32;
            t.push(Some(f0), Some((300.0 - f0) / 10.0));
        }
        let ev = t.events();
        assert_eq!(ev.len(), 2, "events {ev:?}");
        assert!(!ev[1].rising && !ev[1].to_h1);
        assert!((ev[1].f0_hz - 300.0).abs() < 4.0);
    }

    #[test]
    fn turnover_ignores_wobble_and_gaps() {
        let mut t = TurnoverTracker::new();
        // Wobble inside the hysteresis band: no event.
        for i in 0..50 {
            t.push(Some(300.0), Some(if i % 2 == 0 { 1.0 } else { -1.0 }));
        }
        assert!(t.events().is_empty());
        // Below, then a gap, then above: no event across the gap.
        for _ in 0..5 {
            t.push(Some(250.0), Some(-10.0));
        }
        t.push(None, None);
        for _ in 0..5 {
            t.push(Some(350.0), Some(10.0));
        }
        assert!(t.events().is_empty());
    }

    #[test]
    fn dominant_harmonic_picks_the_strongest_of_three() {
        assert_eq!(dominant_harmonic(&[1.0, 0.5, 0.2, 5.0]), Some(1));
        assert_eq!(dominant_harmonic(&[0.4, 1.0, 0.2]), Some(2));
        assert_eq!(dominant_harmonic(&[0.1, 0.2, 0.9]), Some(3));
        assert_eq!(dominant_harmonic(&[0.0, 0.0, 0.0]), None);
    }

    #[test]
    fn register_event_needs_a_jump_and_a_source_change() {
        let mut d = RegisterDetector::new();
        assert!(
            d.push(Some(250.0), Some(2.0), Some(15.0), Some(0.3))
                .is_none()
        );
        // Small drift, no event.
        assert!(
            d.push(Some(255.0), Some(2.5), Some(15.0), Some(0.3))
                .is_none()
        );
        // 4.3-semitone jump with a 6 dB H1–H2 rise: event.
        let ev = d
            .push(Some(327.0), Some(8.5), Some(14.0), Some(0.4))
            .unwrap();
        assert!(ev.jump_semitones > 4.0);
        assert_eq!(ev.h1h2_delta_db.map(|v| v.round()), Some(6.0));
        // A jump alone, with no source change, is a leap, not a register event.
        assert!(
            d.push(Some(420.0), Some(8.5), Some(14.0), Some(0.4))
                .is_none()
        );
    }
}
