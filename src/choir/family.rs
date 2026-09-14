//! The harmonic-family test of Coral's note detector (`note-detector.ts`,
//! `explainedByParent` / `fitEnvelopeExcluding` / `partialLevels`): is a
//! candidate at an integer multiple of an accepted note its own voice, or
//! a partial of that note? Evaluated on the raw spectrum; the envelope is
//! a robust line fit in dB against log2 k with the chest-voice prior.

use super::note_detector::{NoteDetector, peak_in};

impl NoteDetector {
    pub(super) fn record_accepted(&mut self, slot: usize, i: usize, mags: &[f32]) {
        let k_max = self.cfg.cancel_harmonic_count.min(self.bin_stride);
        let mut parts = std::mem::take(&mut self.accepted[slot].parts);
        parts.clear();
        for k in 1..=k_max {
            let d = self.partial_db(i, k, mags);
            parts.push(d.filter(|&d| d > self.cfg.partial_floor_db));
        }
        let a = &mut self.accepted[slot];
        a.i = i;
        a.parts = parts;
        a.fits.iter_mut().for_each(|f| *f = None);
    }

    /// Peak level (dB) of note i's k-th harmonic band on the raw spectrum.
    fn partial_db(&self, i: usize, k: usize, mags: &[f32]) -> Option<f32> {
        if k < 1 || k > self.bin_stride {
            return None;
        }
        let (lo, hi) = self.harmonic_bins[i * self.bin_stride + (k - 1)];
        if hi < lo {
            return None;
        }
        let peak = peak_in(mags, lo, hi);
        if peak <= 0.0 {
            None
        } else {
            Some(20.0 * peak.log10())
        }
    }

    fn fit_envelope_excluding(&self, parts: &[Option<f32>], k0: usize) -> (f32, f32) {
        let c = &self.cfg;
        let mut pts: Vec<(f32, f32)> = parts
            .iter()
            .enumerate()
            .filter_map(|(idx, d)| {
                let k = idx + 1;
                match d {
                    Some(d) if k % k0 != 0 => Some(((k as f32).log2(), *d)),
                    _ => None,
                }
            })
            .collect();
        let fit = |p: &[(f32, f32)]| -> (f32, f32) {
            let m = p.len();
            if m == 0 {
                return (c.fit_default_intercept, c.fit_default_slope);
            }
            if m == 1 {
                return (p[0].1, c.fit_default_slope);
            }
            let (mut sx, mut sy, mut sxx, mut sxy) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
            for &(x, y) in p {
                sx += x;
                sy += y;
                sxx += x * x;
                sxy += x * y;
            }
            let den = m as f32 * sxx - sx * sx;
            let slope = if den != 0.0 {
                (m as f32 * sxy - sx * sy) / den
            } else {
                c.fit_default_slope
            };
            let sl = slope.min(0.0);
            ((sy - sl * sx) / m as f32, sl)
        };
        let mut f = fit(&pts);
        for _ in 0..2 {
            if pts.len() <= 2 {
                break;
            }
            let keep: Vec<(f32, f32)> = pts
                .iter()
                .copied()
                .filter(|&(x, y)| y - (f.0 + f.1 * x) <= c.fit_outlier_db)
                .collect();
            if keep.len() == pts.len() || keep.len() < 2 {
                break;
            }
            pts = keep;
            f = fit(&pts);
        }
        f
    }

    fn prior_db(&self, k: usize) -> f32 {
        match k {
            2 => self.cfg.prior_h2_db,
            3 => self.cfg.prior_h3_db,
            _ => f32::NEG_INFINITY,
        }
    }

    pub(super) fn explained_by_parent(
        &mut self,
        i: usize,
        n_accepted: usize,
        margin: f32,
        mags: &[f32],
    ) -> bool {
        let slots: Vec<usize> = (0..n_accepted).collect();
        self.explained_by_parent_among(i, &slots, margin, mags)
    }

    pub(super) fn explained_by_parent_among(
        &mut self,
        i: usize,
        slots: &[usize],
        margin: f32,
        mags: &[f32],
    ) -> bool {
        let midi = self.midi_min + i as i32;
        for &slot in slots {
            let pm = self.midi_min + self.accepted[slot].i as i32;
            let semis = midi - pm;
            if semis <= 0 {
                continue;
            }
            let ratio = 2f32.powf(semis as f32 / 12.0);
            let k = ratio.round() as usize;
            if k < self.cfg.family_k_min || k > self.cfg.family_k_max {
                continue;
            }
            if (1200.0 * (ratio / k as f32).log2()).abs() > self.cfg.family_cents {
                continue;
            }
            let Some(h1) = self.accepted[slot].parts.first().copied().flatten() else {
                continue;
            };
            let fit = match self.accepted[slot].fits[k] {
                Some(f) => f,
                None => {
                    let f = self.fit_envelope_excluding(&self.accepted[slot].parts, k);
                    self.accepted[slot].fits[k] = Some(f);
                    f
                }
            };
            let env = |kk: usize| -> f32 {
                (fit.0 + fit.1 * (kk as f32).log2()).max(h1 + self.prior_db(kk))
            };
            let mut excess: Vec<f32> = Vec::with_capacity(3);
            for m in 1..=3 {
                if let Some(own) = self.partial_db(i, m, mags) {
                    excess.push(own - env(k * m));
                }
            }
            if excess.is_empty() {
                continue;
            }
            let mean = excess.iter().sum::<f32>() / excess.len() as f32;
            let independent = excess[0] >= margin && mean >= margin;
            if !independent {
                return true;
            }
        }
        false
    }
}
