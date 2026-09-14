//! Coral's multi-F0 note detector (`note-detector.ts`), both paths:
//! the legacy ±quarter-tone band detector and the production harmonic
//! path — whiten by a box-average envelope, then iterative harmonic
//! cancellation (pick the strongest normalized harmonic-sum salience,
//! accept, subtract its spectral-smoothness model in the power domain),
//! the fundamental-support gate, the harmonic-family test, decay-then-max
//! smoothing, and the run merge to the energy-weighted centroid. One
//! instance is bound to (num_bins, sample_rate, range); no resize path.
//! Buffers are allocated once (Coral's Guardrail 7).

use crate::config::ChoirDetectorConfig;

#[derive(Clone, Debug)]
pub struct DetectorOptions {
    /// Coral's `numBins` (`fftSize / 2`).
    pub num_bins: usize,
    pub sample_rate: f32,
    pub min_freq_hz: f32,
    pub max_freq_hz: f32,
    /// The harmonic-aware path (Coral's production mode) or the legacy band detector.
    pub harmonic: bool,
    pub cfg: ChoirDetectorConfig,
}

pub(super) struct Accepted {
    pub(super) i: usize,
    /// Raw-spectrum partial levels, dB, k = 1..=Kc (None where absent).
    pub(super) parts: Vec<Option<f32>>,
    /// Cached envelope fits per excluded harmonic number k0: (intercept, slope).
    pub(super) fits: Vec<Option<(f32, f32)>>,
}

pub struct NoteDetector {
    pub midi_min: i32,
    pub midi_max: i32,
    pub note_count: usize,
    pub harmonic: bool,
    pub(super) cfg: ChoirDetectorConfig,
    num_bins: usize,
    threshold_linear: f32,
    whitening_window_bins: usize,
    pub(super) bin_stride: usize,
    /// (lo, hi) per note; hi < lo = skip.
    bin_ranges: Vec<(i64, i64)>,
    energy: Vec<f32>,
    pub(super) harmonic_bins: Vec<(i64, i64)>,
    harmonic_weights: Vec<f32>,
    note_weight_norm: Vec<f32>,
    whitened: Vec<f32>,
    prefix: Vec<f64>,
    residual: Vec<f32>,
    frame_salience: Vec<f32>,
    cancel_amps: Vec<f32>,
    pub(super) accepted: Vec<Accepted>,
    explained: Vec<bool>,
    active: Vec<i32>,
    merged: Vec<i32>,
    last_conf: Vec<(i32, f32)>,
}

fn midi_freq(midi: i32) -> f32 {
    440.0 * 2f32.powf((midi - 69) as f32 / 12.0)
}

impl NoteDetector {
    pub fn new(o: DetectorOptions) -> Result<Self, String> {
        let c = o.cfg;
        if o.num_bins == 0 {
            return Err(format!(
                "NoteDetector: numBins must be positive, got {}",
                o.num_bins
            ));
        }
        if !(o.sample_rate.is_finite() && o.sample_rate > 0.0) {
            return Err(format!(
                "NoteDetector: sampleRate must be positive, got {}",
                o.sample_rate
            ));
        }
        if !(o.min_freq_hz > 0.0 && o.max_freq_hz > o.min_freq_hz) {
            return Err(format!(
                "NoteDetector: require 0 < minFreqHz < maxFreqHz, got min={} max={}",
                o.min_freq_hz, o.max_freq_hz
            ));
        }
        if !(c.decay_per_frame > 0.0 && c.decay_per_frame < 1.0) {
            return Err(format!(
                "NoteDetector: decayPerFrame must be in (0, 1), got {}",
                c.decay_per_frame
            ));
        }
        let bin_hz = o.sample_rate / (2.0 * o.num_bins as f32);
        let whitening_window_bins = ((c.whitening_window_hz / bin_hz).round() as usize).max(1);
        if o.harmonic {
            if c.harmonic_count < 1 {
                return Err("NoteDetector: harmonicCount must be positive".into());
            }
            if c.salience_threshold.is_nan() || c.salience_threshold <= 0.0 {
                return Err("NoteDetector: salienceThreshold must be positive".into());
            }
            if c.max_polyphony < 1 {
                return Err("NoteDetector: maxPolyphony must be positive".into());
            }
            if !(c.cancel_factor > 0.0 && c.cancel_factor <= 1.0) {
                return Err("NoteDetector: cancelFactor must be in (0, 1]".into());
            }
            if c.cancel_harmonic_count < 1 {
                return Err("NoteDetector: cancelHarmonicCount must be positive".into());
            }
            if c.cancel_headroom.is_nan() || c.cancel_headroom < 1.0 {
                return Err("NoteDetector: cancelHeadroom must be >= 1".into());
            }
        }
        let eff = o.num_bins as i64;
        let band_for = |flo: f32, fhi: f32| -> (i64, i64) {
            let lo = (flo / bin_hz).floor() as i64;
            let hi = (fhi / bin_hz).ceil() as i64;
            if lo > eff - 1 || hi < 0 {
                (0, -1)
            } else {
                (lo.max(0), hi.min(eff - 1))
            }
        };
        let midi_min = ((12.0 * (o.min_freq_hz / 440.0).log2() + 69.0).floor() as i32).max(0);
        let midi_max = ((12.0 * (o.max_freq_hz / 440.0).log2() + 69.0).ceil() as i32).min(127);
        let note_count = (midi_max - midi_min + 1).max(0) as usize;
        let half_step = 2f32.powf(1.0 / 24.0);
        let bin_ranges: Vec<(i64, i64)> = (0..note_count)
            .map(|i| {
                let f = midi_freq(midi_min + i as i32);
                band_for(f / half_step, f * half_step)
            })
            .collect();
        let bin_stride = if o.harmonic {
            c.harmonic_count.max(c.cancel_harmonic_count)
        } else {
            c.harmonic_count
        };
        let (harmonic_bins, harmonic_weights, note_weight_norm) = if o.harmonic {
            let k = c.harmonic_count;
            let weights: Vec<f32> = (0..k).map(|i| 1.0 / (i + 1) as f32).collect();
            let mut hb = vec![(0i64, -1i64); note_count * bin_stride];
            for i in 0..note_count {
                let f0 = midi_freq(midi_min + i as i32);
                for kk in 1..=bin_stride {
                    let fk = kk as f32 * f0;
                    hb[i * bin_stride + (kk - 1)] = band_for(fk / half_step, fk * half_step);
                }
            }
            let norm: Vec<f32> = (0..note_count)
                .map(|i| {
                    let sum_w: f32 = (0..k)
                        .filter(|&kk| {
                            let (lo, hi) = hb[i * bin_stride + kk];
                            hi >= lo
                        })
                        .map(|kk| weights[kk])
                        .sum();
                    if sum_w > 0.0 { 1.0 / sum_w } else { 0.0 }
                })
                .collect();
            (hb, weights, norm)
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };
        let nb = if o.harmonic { o.num_bins } else { 0 };
        let accepted = (0..c.max_polyphony)
            .map(|_| Accepted {
                i: 0,
                parts: Vec::with_capacity(c.cancel_harmonic_count),
                fits: vec![None; c.family_k_max + 1],
            })
            .collect();
        Ok(Self {
            midi_min,
            midi_max,
            note_count,
            harmonic: o.harmonic,
            cfg: c,
            num_bins: o.num_bins,
            threshold_linear: 10f32.powf(c.threshold_db / 20.0),
            whitening_window_bins,
            bin_stride,
            bin_ranges,
            energy: vec![0.0; note_count],
            harmonic_bins,
            harmonic_weights,
            note_weight_norm,
            whitened: vec![0.0; nb],
            prefix: vec![0.0; nb + 1],
            residual: vec![0.0; nb],
            frame_salience: vec![0.0; if o.harmonic { note_count } else { 0 }],
            cancel_amps: vec![0.0; bin_stride],
            accepted,
            explained: vec![false; note_count],
            active: Vec::with_capacity(note_count),
            merged: Vec::with_capacity(note_count),
            last_conf: Vec::with_capacity(note_count),
        })
    }

    /// Per-MIDI fundamental bin range.
    pub fn bin_range_for_midi(&self, midi: i32) -> Option<(i64, i64)> {
        if midi < self.midi_min || midi > self.midi_max {
            return None;
        }
        Some(self.bin_ranges[(midi - self.midi_min) as usize])
    }

    /// Per-MIDI harmonic bin ranges (fundamental first), harmonic mode only.
    pub fn harmonic_bins_for_midi(&self, midi: i32) -> Option<Vec<(i64, i64)>> {
        if !self.harmonic || midi < self.midi_min || midi > self.midi_max {
            return None;
        }
        let i = (midi - self.midi_min) as usize;
        Some(
            (0..self.cfg.harmonic_count)
                .map(|k| self.harmonic_bins[i * self.bin_stride + k])
                .collect(),
        )
    }

    /// Clear decay state.
    pub fn reset(&mut self) {
        self.energy.fill(0.0);
    }

    /// Per-note confidence (smoothed salience / threshold) of the notes the
    /// last `analyze` emitted, in ascending MIDI.
    pub fn last_confidences(&self) -> &[(i32, f32)] {
        &self.last_conf
    }

    pub fn confidence_of(&self, midi: i32) -> Option<f32> {
        self.last_conf
            .iter()
            .find(|(m, _)| *m == midi)
            .map(|(_, c)| *c)
    }

    /// Active MIDI numbers for one magnitude frame (ascending). Mutates the
    /// decay state — not idempotent.
    pub fn analyze(&mut self, magnitudes: &[f32]) -> Result<&[i32], String> {
        if magnitudes.len() < self.num_bins {
            return Err(format!(
                "NoteDetector.analyze: magnitude frame length ({}) is smaller than expected numBins ({})",
                magnitudes.len(),
                self.num_bins
            ));
        }
        if self.harmonic {
            return Ok(self.analyze_harmonic(magnitudes));
        }
        self.active.clear();
        self.last_conf.clear();
        for i in 0..self.note_count {
            let (lo, hi) = self.bin_ranges[i];
            let peak = peak_in(magnitudes, lo, hi);
            let decayed = self.energy[i] * self.cfg.decay_per_frame;
            let next = if peak > decayed { peak } else { decayed };
            self.energy[i] = next;
            if next > self.threshold_linear {
                self.active.push(self.midi_min + i as i32);
            }
        }
        Ok(&self.active)
    }

    fn whiten(&mut self, mags: &[f32]) {
        let n = self.num_bins;
        let half = (self.whitening_window_bins >> 1) as i64;
        self.prefix[0] = 0.0;
        let mut total = 0.0f64;
        for (b, &m) in mags.iter().enumerate().take(n) {
            total += m as f64;
            self.prefix[b + 1] = total;
        }
        for (b, &m) in mags.iter().enumerate().take(n) {
            let lo = (b as i64 - half).max(0) as usize;
            let hi = ((b as i64 + half) as usize).min(n - 1);
            let sum = self.prefix[hi + 1] - self.prefix[lo];
            let env = (sum / (hi - lo + 1) as f64) as f32;
            self.whitened[b] = if m >= self.threshold_linear {
                m / env.max(self.cfg.envelope_floor)
            } else {
                0.0
            };
        }
    }

    fn analyze_harmonic(&mut self, mags: &[f32]) -> &[i32] {
        self.whiten(mags);
        let n = self.note_count;
        let threshold = self.cfg.salience_threshold;
        self.residual.copy_from_slice(&self.whitened);
        self.frame_salience.fill(0.0);
        self.explained.fill(false);
        let gamma = self.cfg.cancel_factor;
        let f_floor = self.cfg.fundamental_floor;
        let family_margin = self.cfg.family_margin_db;
        let mut n_accepted = 0usize;
        for _ in 0..self.cfg.max_polyphony {
            let mut best_i: Option<usize> = None;
            let mut best_sal = threshold;
            for i in 0..n {
                if self.frame_salience[i] > 0.0 || self.explained[i] {
                    continue;
                }
                if f_floor > 0.0 && !self.has_fundamental_support(i, f_floor) {
                    continue;
                }
                let sal = self.salience_of(i, &self.residual);
                if sal > best_sal {
                    if family_margin > 0.0
                        && self.explained_by_parent(i, n_accepted, family_margin, mags)
                    {
                        self.explained[i] = true;
                        self.cancel(i, gamma);
                        continue;
                    }
                    best_sal = sal;
                    best_i = Some(i);
                }
            }
            let Some(bi) = best_i else { break };
            self.frame_salience[bi] = best_sal;
            self.record_accepted(n_accepted, bi, mags);
            n_accepted += 1;
            self.cancel(bi, gamma);
        }
        // Family resolution pass, ascending.
        if family_margin > 0.0 && n_accepted > 1 {
            let mut order: Vec<usize> = (0..n_accepted).collect();
            order.sort_by_key(|&a| self.accepted[a].i);
            let mut kept: Vec<usize> = Vec::with_capacity(n_accepted);
            for a in order {
                let i = self.accepted[a].i;
                if !kept.is_empty() && self.explained_by_parent_among(i, &kept, family_margin, mags)
                {
                    self.frame_salience[i] = 0.0;
                    continue;
                }
                kept.push(a);
            }
        }
        // Temporal smoothing, emit, merge.
        self.active.clear();
        let decay = self.cfg.decay_per_frame;
        for i in 0..n {
            let s = self.frame_salience[i];
            let decayed = self.energy[i] * decay;
            let e = if s > decayed { s } else { decayed };
            self.energy[i] = e;
            if e > threshold {
                self.active.push(self.midi_min + i as i32);
            }
        }
        if self.cfg.merge_radius > 0 {
            self.merge_adjacent();
        } else {
            self.merged.clear();
            self.merged.extend_from_slice(&self.active);
        }
        self.last_conf.clear();
        for &midi in &self.merged {
            let e = self.energy[(midi - self.midi_min) as usize];
            self.last_conf.push((midi, e / threshold));
        }
        &self.merged
    }

    fn has_fundamental_support(&self, i: usize, floor: f32) -> bool {
        let (lo, hi) = self.harmonic_bins[i * self.bin_stride];
        (lo..=hi).any(|b| self.whitened[b as usize] > floor)
    }

    fn salience_of(&self, i: usize, spec: &[f32]) -> f32 {
        let base = i * self.bin_stride;
        let mut salience = 0.0f32;
        for (k, w) in self.harmonic_weights.iter().enumerate() {
            let (lo, hi) = self.harmonic_bins[base + k];
            salience += w * peak_in(spec, lo, hi);
        }
        salience * self.note_weight_norm[i]
    }

    /// Spectral-smoothness cancellation of note i from the residual.
    fn cancel(&mut self, i: usize, gamma: f32) {
        let kc = self.cfg.cancel_harmonic_count;
        let headroom = self.cfg.cancel_headroom;
        let base = i * self.bin_stride;
        for k in 0..kc {
            let (lo, hi) = self.harmonic_bins[base + k];
            self.cancel_amps[k] = if hi < lo {
                -1.0
            } else {
                peak_in(&self.residual, lo, hi)
            };
        }
        for k in 0..kc {
            if self.cancel_amps[k] < 0.0 {
                continue;
            }
            let mut sum = 0.0f32;
            let mut cnt = 0usize;
            for j in k.saturating_sub(1)..=k + 1 {
                if j >= kc {
                    continue;
                }
                let a = self.cancel_amps[j];
                if a < 0.0 {
                    continue;
                }
                sum += a;
                cnt += 1;
            }
            let smoothed = (if cnt > 0 {
                sum / cnt as f32
            } else {
                self.cancel_amps[k]
            }) * headroom;
            let own = self.cancel_amps[k];
            let model = gamma * if own < smoothed { own } else { smoothed };
            if model <= 0.0 {
                continue;
            }
            let m2 = model * model;
            let (lo, hi) = self.harmonic_bins[base + k];
            for b in lo..=hi {
                let v = self.residual[b as usize];
                let p = v * v - m2;
                self.residual[b as usize] = if p > 0.0 { p.sqrt() } else { 0.0 };
            }
        }
    }

    fn merge_adjacent(&mut self) {
        self.merged.clear();
        let sorted = &self.active;
        if sorted.len() <= 1 {
            self.merged.extend_from_slice(sorted);
            return;
        }
        let radius = self.cfg.merge_radius as i32;
        let mut run_start = 0usize;
        for idx in 0..sorted.len() {
            let is_last = idx == sorted.len() - 1;
            let gap_over = is_last || sorted[idx + 1] - sorted[idx] > radius;
            if gap_over {
                let mut wsum = 0.0f32;
                let mut esum = 0.0f32;
                for &m in &sorted[run_start..=idx] {
                    let e = self.energy[(m - self.midi_min) as usize];
                    wsum += m as f32 * e;
                    esum += e;
                }
                let centroid = if esum > 0.0 {
                    wsum / esum
                } else {
                    sorted[run_start] as f32
                };
                let mut best = sorted[run_start];
                let mut best_dist = (best as f32 - centroid).abs();
                for &m in &sorted[run_start + 1..=idx] {
                    let d = (m as f32 - centroid).abs();
                    if d < best_dist {
                        best_dist = d;
                        best = m;
                    }
                }
                self.merged.push(best);
                run_start = idx + 1;
            }
        }
    }
}

pub(super) fn peak_in(spec: &[f32], lo: i64, hi: i64) -> f32 {
    let mut peak = 0.0f32;
    let mut b = lo;
    while b <= hi {
        let v = spec[b as usize];
        if v > peak {
            peak = v;
        }
        b += 1;
    }
    peak
}

#[cfg(test)]
#[path = "note_detector_tests.rs"]
mod tests;
