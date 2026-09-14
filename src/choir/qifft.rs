//! Coral's single-F0 estimator (`qifft.ts`): the magnitude peak in the
//! vocal band, parabolic interpolation on log magnitudes, a voiced gate on
//! peak-to-mean, and the sub-octave check that stops H2 ≥ H1 from
//! flipping the octave.

use crate::config::QifftConfig;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct F0Estimate {
    pub frequency_hz: f32,
    /// 0..1 from the peak-to-mean ratio.
    pub confidence: f32,
}

/// Sub-bin peak offset in [−0.5, 0.5] from a parabola through three
/// samples (`y0` the peak).
pub fn parabolic_peak(ym1: f32, y0: f32, yp1: f32) -> f32 {
    let denom = ym1 - 2.0 * y0 + yp1;
    if denom == 0.0 {
        return 0.0;
    }
    ((0.5 * (ym1 - yp1)) / denom).clamp(-0.5, 0.5)
}

/// `mags` has Coral's `fftSize / 2` bins (or more; only `n` matter as
/// given); bin `b` sits at `b · sample_rate / fft_size` Hz.
pub fn estimate_f0(
    mags: &[f32],
    sample_rate: f32,
    fft_size: usize,
    cfg: &QifftConfig,
) -> Option<F0Estimate> {
    let n = mags.len();
    if n < 3 {
        return None;
    }
    let bin_hz = sample_rate / fft_size as f32;
    let lo_bin = ((cfg.min_f0_hz / bin_hz).floor() as usize).max(1);
    let hi_bin = ((cfg.max_f0_hz / bin_hz).ceil() as usize).min(n - 2);
    if hi_bin <= lo_bin {
        return None;
    }
    let mut peak_bin = lo_bin;
    let mut peak_mag = f32::NEG_INFINITY;
    for (b, &m) in mags.iter().enumerate().take(hi_bin + 1).skip(lo_bin) {
        if m > peak_mag {
            peak_mag = m;
            peak_bin = b;
        }
    }
    let mean = mags.iter().sum::<f32>() / n as f32;
    if mean.is_nan() || mean <= 0.0 || peak_mag / mean < cfg.voiced_ratio {
        return None;
    }
    // Sub-octave check.
    let mut final_bin = peak_bin;
    let sub = (peak_bin as f32 / 2.0).round() as usize;
    if sub >= lo_bin {
        let mut sub_bin = sub;
        let mut sub_mag = mags[sub];
        for (b, &m) in mags
            .iter()
            .enumerate()
            .take(sub + 2)
            .skip(sub.saturating_sub(1))
        {
            if b >= lo_bin && b <= hi_bin && m > sub_mag {
                sub_mag = m;
                sub_bin = b;
            }
        }
        if sub_mag > cfg.sub_octave_ratio * peak_mag
            && sub_mag >= mags[sub_bin - 1]
            && sub_mag >= mags[sub_bin + 1]
        {
            final_bin = sub_bin;
        }
    }
    let eps = cfg.log_epsilon;
    let a = (mags[final_bin - 1] + eps).ln();
    let b0 = (mags[final_bin] + eps).ln();
    let g = (mags[final_bin + 1] + eps).ln();
    let p = parabolic_peak(a, b0, g);
    let f0 = (final_bin as f32 + p) * bin_hz;
    if f0.is_nan() || f0 <= 0.0 {
        return None;
    }
    Some(F0Estimate {
        frequency_hz: f0,
        confidence: (peak_mag / mean / (cfg.confidence_divisor * cfg.voiced_ratio)).min(1.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::choir::stft::CoralStft;
    use std::f32::consts::TAU;

    const SR: f32 = 44_100.0;
    const FFT: usize = 2048;

    fn last_mags(f: f32) -> Vec<f32> {
        let n = FFT * 4;
        let sig: Vec<f32> = (0..n).map(|i| (TAU * f * i as f32 / SR).sin()).collect();
        let mut stft = CoralStft::new(FFT).unwrap();
        let mut out = vec![0.0f32; FFT / 2];
        // The last frame of Coral's streaming run: hop 512 from the first
        // full frame, i.e. the frame ending at the last hop boundary.
        let mut start = 0;
        let mut last = 0;
        while start + FFT <= n {
            last = start;
            start += 512;
        }
        stft.magnitudes(&sig[last..last + FFT], &mut out).unwrap();
        out
    }

    fn cents(f: f32, r: f32) -> f32 {
        1200.0 * (f / r).log2()
    }

    /// Coral's `qifft.test.ts`.
    #[test]
    fn parabolic_peak_is_zero_symmetric_leans_and_clamps() {
        assert!(parabolic_peak(0.0, 1.0, 0.0).abs() < 1e-10);
        assert!(parabolic_peak(0.5, 1.0, 0.9) > 0.0);
        assert!(parabolic_peak(0.9, 1.0, 0.5) < 0.0);
        assert!(parabolic_peak(1.0, 1.0, 0.0) >= -0.5);
        assert!(parabolic_peak(0.0, 1.0, 1.0) <= 0.5);
    }

    #[test]
    fn recovers_pitches_across_the_vocal_range() {
        let cfg = QifftConfig::DEFAULT;
        let e = estimate_f0(&last_mags(440.0), SR, FFT, &cfg).unwrap();
        assert!(cents(e.frequency_hz, 440.0).abs() < 10.0);
        for f in [196.0, 261.63, 329.63, 440.0, 587.33, 880.0] {
            let e = estimate_f0(&last_mags(f), SR, FFT, &cfg).expect("voiced");
            assert!(cents(e.frequency_hz, f).abs() < 15.0, "{f} Hz");
        }
        assert!(estimate_f0(&vec![0.0; FFT / 2], SR, FFT, &cfg).is_none());
        assert!(estimate_f0(&[0.0, 0.0], SR, FFT, &cfg).is_none());
    }

    #[test]
    fn avoids_octave_flipping_when_h2_exceeds_h1() {
        let mut mags = vec![0.01f32; FFT / 2];
        mags[8] = 0.4;
        mags[9] = 0.8;
        mags[10] = 0.4;
        mags[18] = 0.5;
        mags[19] = 1.0;
        mags[20] = 0.5;
        let cfg = QifftConfig {
            voiced_ratio: 2.0,
            ..QifftConfig::DEFAULT
        };
        let e = estimate_f0(&mags, SR, FFT, &cfg).unwrap();
        assert!(
            (e.frequency_hz - 9.0 * SR / FFT as f32).abs() < 0.5,
            "{e:?}"
        );
    }
}
