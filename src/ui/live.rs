//! Per-frame live state: engine-health watch, the recording/analyzing tick, and the waveform feed.

use super::*;

impl DashboardApp {
    // ── engine health ────────────────────────────────────────────────────────

    /// Polls the engine's telemetry once per frame and updates the two derived
    /// conditions the UI can't read directly from a flag: an analysis thread
    /// that is alive but no longer producing, and a cpal stream error recent
    /// enough to still be worth showing.
    pub(super) fn watch_engine(&mut self, now: f64) {
        let frames = self.telemetry.analysis_frames.load(Ordering::Relaxed);
        match self.last_analysis_change {
            // First observation: start the clock, never accuse on frame one.
            None => {
                self.last_analysis_frames = frames;
                self.last_analysis_change = Some(now);
                self.analysis_stalled = false;
            }
            Some(_) if frames != self.last_analysis_frames => {
                self.last_analysis_frames = frames;
                self.last_analysis_change = Some(now);
                self.analysis_stalled = false;
            }
            Some(since) => {
                // Only a live microphone makes silence suspicious: with no
                // input there is legitimately nothing to analyze.
                self.analysis_stalled = self.input_rms() > ANALYSIS_STALL_RMS_FLOOR
                    && now - since >= ANALYSIS_STALL_SECS;
            }
        }

        let input_errors = self.telemetry.input_stream_errors.load(Ordering::Relaxed);
        let output_errors = self.telemetry.output_stream_errors.load(Ordering::Relaxed);
        if input_errors != self.last_input_errors {
            self.last_input_errors = input_errors;
            self.audio_notice = Some((
                "Microphone stream error — input may be interrupted. Check the input device.",
                now,
            ));
        } else if output_errors != self.last_output_errors {
            self.last_output_errors = output_errors;
            self.audio_notice = Some((
                "Audio output stream error — monitoring may be interrupted.",
                now,
            ));
        }
        if let Some((_, seen)) = self.audio_notice
            && now - seen >= AUDIO_ERROR_NOTICE_SECS
        {
            self.audio_notice = None;
        }
    }

    /// The engine's worst current problem, as one line for the status banner,
    /// or `None` when everything is running. Ordered by how much it costs the
    /// user: no analysis at all, then stopped, then stalled, then audio I/O.
    pub(super) fn engine_notice(&self) -> Option<&'static str> {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(notice) = self.pipeline_notice() {
            return Some(notice);
        }
        // No microphone outranks everything else: with no input stream the
        // analysis path is idle by definition, so any staleness or GPU notice
        // below would only describe a consequence of this.
        if self.telemetry.audio_unavailable() {
            return Some(
                "Microphone unavailable — no input stream is open. If the permission \
                 dialog did not appear or was dismissed: Room → DIAGNOSTICS → Open app \
                 settings → Permissions → Microphone → Allow. Audio starts as soon as \
                 it is granted.",
            );
        }
        match self.telemetry.analysis_state() {
            AnalysisState::Unavailable => Some(
                "Analysis unavailable — no compatible GPU adapter was found. \
                 Live readouts and capture are off.",
            ),
            AnalysisState::Stopped => Some(
                "Analysis stopped — the analysis thread exited. \
                 Restart the app to resume live readouts.",
            ),
            // A slow start is not a stall: `Starting` means the engine has not
            // claimed to be producing yet, so the absence of frames is expected.
            AnalysisState::Starting => self.audio_notice.map(|(msg, _)| msg),
            AnalysisState::Running => {
                if self.analysis_stalled {
                    Some(
                        "Analysis stalled — the microphone is live but no frames have \
                         arrived recently. Readouts below may be out of date.",
                    )
                } else {
                    self.audio_notice.map(|(msg, _)| msg)
                }
            }
        }
    }

    // ── state machine ────────────────────────────────────────────────────────

    pub(super) fn advance(&mut self, now: f64) {
        // A capture left running (e.g. off another screen) auto-stops so the
        // accumulators can't grow unbounded; the tab bar shows a live dot
        // meanwhile.
        if let RecState::Recording { start } = self.rec
            && self.import_job.is_none()
            && now - start >= MAX_REC_SECS
        {
            self.stop_rec(now);
        }
        if let RecState::Analyzing { start, elapsed } = self.rec
            && now - start >= 2.1
        {
            // Real f0 mean over voiced frames; a silent capture stays honest
            // (None readout) and is blocked from saving below.
            let f0 = (!self.rec_f0_acc.is_empty())
                .then(|| self.rec_f0_acc.iter().sum::<f32>() / self.rec_f0_acc.len() as f32)
                .map(|f| (f * 10.0).round() / 10.0);
            let mean =
                |acc: &[f32]| (!acc.is_empty()).then(|| acc.iter().sum::<f32>() / acc.len() as f32);
            // Median: robust central tendency for the identity features —
            // one harmonic-attracted stray fit shifts a mean, not a median.
            let median = |acc: &[f32]| -> Option<f32> {
                if acc.is_empty() {
                    return None;
                }
                let mut v: Vec<f32> = acc.iter().copied().filter(|x| x.is_finite()).collect();
                if v.is_empty() {
                    return None;
                }
                v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mid = v.len() / 2;
                Some(if v.len().is_multiple_of(2) {
                    0.5 * (v[mid - 1] + v[mid])
                } else {
                    v[mid]
                })
            };
            // Vibrato/steadiness are contour-level: snapshot the stop-time
            // values rather than averaging per-frame reports.
            let m = self.current_profile.metrics;
            let profile = if self.rec_profile_n > 0 {
                let n = self.rec_profile_n as f32;
                std::array::from_fn(|i| self.rec_profile_sum[i] / n)
            } else {
                [0.0; 16]
            };

            // Build this capture's classical voiceprint and score it against
            // the enrolled reference.
            // Identity-grade formants only: a capture sung above ~200 Hz f0
            // gets a formant-less voiceprint and is scored on its remaining
            // features (or reads "Unscored") — phase C's skip-and-renormalize
            // machinery — instead of on harmonic-attraction artifacts. Each
            // slot is the capture's *median* over identity-grade frames;
            // slots with no accepted frames stay at the 0.0 unmeasured
            // encoding and are skipped by the similarity scorer.
            let id_formants = {
                let med: Vec<Option<f32>> = self.rec_id_f_acc.iter().map(|v| median(v)).collect();
                med.iter().any(Option::is_some).then(|| {
                    std::array::from_fn(|i| Formant {
                        frequency: med[i].unwrap_or(0.0),
                        bandwidth: 0.0,
                    })
                })
            };
            let voiceprint = crate::math::build_voiceprint(
                id_formants,
                &profile,
                mean(&self.rec_centroid_acc),
                median(&self.rec_vtl_acc),
            );

            // Enrollment gate: only a long-enough, mostly-voiced capture may
            // become the reference (a bad reference poisons every later match).
            let voiced_frac = if self.rec_frames_total > 0 {
                self.rec_f0_acc.len() as f32 / self.rec_frames_total as f32
            } else {
                0.0
            };
            let enroll_eligible = f0.is_some()
                && elapsed >= MIN_ENROLL_SECS
                && voiced_frac >= MIN_ENROLL_VOICED_FRACTION;
            let is_reference = self.enrolled.is_none() && enroll_eligible;
            let match_pct = match &self.enrolled {
                Some(reference) => crate::math::voiceprint_similarity(&voiceprint, reference)
                    .map(|s| (s * 10.0).round() / 10.0),
                // An eligible first capture *becomes* the reference (self-match
                // 100); an ineligible one has nothing to score against.
                None => is_reference.then_some(100.0),
            };
            let blocked = if f0.is_none() {
                Some("No voiced signal captured — nothing to save".to_string())
            } else if self.enrolled.is_none() && !enroll_eligible {
                Some(format!(
                    "The reference capture needs ≥{MIN_ENROLL_SECS:.0} s of sustained voice — \
                     this one ran {elapsed:.0} s at {:.0}% voiced",
                    voiced_frac * 100.0
                ))
            } else {
                None
            };

            let coverage_pct = (self.rec_cov_voiced > 0)
                .then(|| 100.0 * self.rec_cov_identity as f32 / self.rec_cov_voiced as f32);
            self.result = Some(CaptureResult {
                match_pct,
                f0,
                hnr_db: mean(&self.rec_hnr_acc),
                h1_h2_db: mean(&self.rec_h1h2_acc),
                vibrato: m.vibrato,
                steadiness_cents: m.steadiness_cents,
                jitter_pct: mean(&self.rec_jitter_acc),
                shimmer_db: mean(&self.rec_shimmer_acc),
                cpp_db: mean(&self.rec_cpp_acc),
                centroid_hz: mean(&self.rec_centroid_acc),
                profile,
                formants: self.rec_formants,
                voiceprint,
                coverage_pct,
                is_reference,
                blocked,
            });
            self.rec = RecState::Done { elapsed };
        }
    }

    /// One waveform sample per frame: recording follows the live input RMS,
    /// idle decays flat (same smoothing constants as the prototype).
    pub(super) fn push_wave_sample(&mut self) {
        const H: f32 = 52.0;
        let recording = matches!(self.rec, RecState::Recording { .. });
        let last = *self.wave.last().unwrap_or(&2.0);
        let next = if recording {
            let jitter = 0.7 + 0.6 * self.rand01();
            let target = (3.0 + self.input_rms() * 260.0 * jitter).min(H * 0.42);
            last + (target - last) * 0.55
        } else {
            last + (2.0 - last) * 0.12
        };
        self.wave.push(next.max(1.5));
        self.wave.remove(0);
    }
}
