//! Recording lifecycle: start/stop/reset, profile ingestion into the capture accumulators, saving a session, and the raw-audio export toggle.

use super::*;

impl DashboardApp {
    /// Result-card line for the raw-audio export: what was saved (name,
    /// length, peak, clipping), where, and the on/off chip. Nothing is
    /// drawn on platforms with no export directory.
    pub(super) fn capture_export_line(&mut self, ui: &mut egui::Ui) {
        if self.capture_dir.is_none() {
            return;
        }
        let mono = |s: String, a: u8| RichText::new(s).font(FontId::monospace(10.0)).color(ink(a));
        let imported = self
            .last_capture
            .as_ref()
            .is_some_and(|c| self.is_imported(&c.path));
        ui.horizontal(|ui| {
            if imported {
                ui.label(
                    RichText::new("FILE")
                        .font(FontId::monospace(9.5))
                        .color(TEAL_DARK),
                );
            } else {
                let label = if self.export_audio {
                    "AUDIO EXPORT: ON"
                } else {
                    "AUDIO EXPORT: OFF"
                };
                if probe_chip(ui, label) {
                    self.export_audio = !self.export_audio;
                }
            }
            ui.add_space(8.0);
            match (&self.last_capture, &self.capture_error) {
                (Some(cap), _) => {
                    let clipped = if cap.clipped > 0 {
                        format!(" · {} CLIPPED", cap.clipped)
                    } else {
                        String::new()
                    };
                    ui.label(mono(
                        format!(
                            "{} · {:.1} s · peak {:.1} dBFS{clipped}",
                            cap.file_name(),
                            cap.seconds,
                            cap.peak_dbfs
                        ),
                        150,
                    ));
                }
                (None, Some(err)) => {
                    ui.label(RichText::new(err.as_str()).size(11.0).color(AMBER_TEXT));
                }
                (None, None) => {
                    ui.label(mono("no audio saved for this capture".into(), 110));
                }
            }
        });
        if let Some(cap) = &self.last_capture
            && let Some(dir) = cap.path.parent()
        {
            ui.add_space(3.0);
            ui.label(mono(format!("in {}", dir.display()), 100));
        }
    }

    /// Begin a live capture: reset the accumulators, arm the raw-audio
    /// export.
    pub(super) fn start_rec(&mut self, now: f64) {
        self.reset_rec(now);
        if self.export_audio
            && let Some(dir) = &self.capture_dir
        {
            match capture_export::arm(dir, self.sample_rate.round() as u32) {
                Ok(path) => log::info!("capture export → {}", path.display()),
                Err(e) => {
                    log::warn!("capture export could not start: {e}");
                    self.capture_error = Some(format!("Audio export off: {e}"));
                }
            }
        }
    }

    /// Enter Recording with every per-capture accumulator cleared. Shared
    /// by the live capture and the file import.
    pub(super) fn reset_rec(&mut self, now: f64) {
        self.rec = RecState::Recording { start: now };
        self.last_capture = None;
        self.capture_error = None;
        self.rec_f0_acc.clear();
        self.rec_frames_total = 0;
        self.rec_hnr_acc.clear();
        self.rec_h1h2_acc.clear();
        self.rec_jitter_acc.clear();
        self.rec_shimmer_acc.clear();
        self.rec_cpp_acc.clear();
        self.rec_centroid_acc.clear();
        self.rec_profile_sum = [0.0; 16];
        self.rec_profile_n = 0;
        self.rec_formants = None;
        self.rec_id_f_acc = [Vec::new(), Vec::new(), Vec::new()];
        self.rec_vtl_acc.clear();
        self.rec_cov_voiced = 0;
        self.rec_cov_identity = 0;
        self.result = None;
    }

    pub(super) fn stop_rec(&mut self, now: f64) {
        if let RecState::Recording { start } = self.rec {
            self.rec = RecState::Analyzing {
                start: now,
                elapsed: now - start,
            };
            self.last_capture = capture_export::disarm();
        }
    }

    pub(super) fn save_session(&mut self) {
        // Degenerate captures may not be saved (G2): the Save button is
        // withheld for them, and this guard keeps the archive honest even if
        // this is reached some other way.
        if self.result.as_ref().is_none_or(|r| r.blocked.is_some()) {
            return;
        }
        if let Some(res) = self.result.take() {
            // First *eligible* saved capture enrolls the reference voiceprint;
            // its match reads 100% against itself.
            if self.enrolled.is_none() && res.is_reference {
                self.enrolled = Some(res.voiceprint);
                self.enrolled_id = Some(self.derive_voice_id(&res.voiceprint));
            }
            let subj = self
                .enrolled_id
                .clone()
                .unwrap_or_else(|| "V-????".to_string());
            let session = Session {
                id: format!("VS-{:03}", self.next_session_num),
                subj,
                date: today_string(),
                // Gated above: blocked.is_none() implies a measured f0.
                f0: res.f0.unwrap_or(0.0),
                match_pct: res.match_pct,
                hnr_db: res.hnr_db,
                h1_h2_db: res.h1_h2_db,
                vibrato: res.vibrato,
                steadiness_cents: res.steadiness_cents,
                jitter_pct: res.jitter_pct,
                shimmer_db: res.shimmer_db,
                cpp_db: res.cpp_db,
                centroid_hz: res.centroid_hz,
                profile: res.profile,
                formants: res.formants,
                coverage_pct: res.coverage_pct,
                capture_file: self.last_capture.as_ref().map(|c| {
                    if self.is_imported(&c.path) {
                        import::session_file_name(&c.path)
                    } else {
                        c.file_name()
                    }
                }),
            };
            self.next_session_num += 1;
            // Sidecar for the study harness: the session's own measurements
            // next to its WAV (`capture-….json`), so the phone's numbers and
            // the harness's numbers join on the file name with a plain
            // `adb pull` — the archive itself stays app-private.
            if let Some(cap) = &self.last_capture {
                let sidecar = cap.path.with_extension("json");
                match serde_json::to_vec_pretty(&session) {
                    Ok(bytes) => {
                        if let Err(e) = std::fs::write(&sidecar, bytes) {
                            log::warn!("capture sidecar {} not written: {e}", sidecar.display());
                        }
                    }
                    Err(e) => log::warn!("capture sidecar could not be encoded: {e}"),
                }
            }
            self.sessions.insert(0, session);
            // Enrollment and the new capture just changed the archive — write it
            // through so it survives a restart.
            self.persist_state();
            self.rec = RecState::Idle;
            self.filter = Filter::All;
            self.screen = Screen::Sessions;
        }
    }

    /// A short, stable ID for a voiceprint (e.g. "V-3F9A"), hashed from its
    /// formants + centroid so the same reference reads consistently.
    pub(super) fn derive_voice_id(&self, vp: &Voiceprint) -> String {
        let mut h: u32 = 0x811c_9dc5;
        for v in [
            vp.formants[0],
            vp.formants[1],
            vp.formants[2],
            vp.centroid_hz,
        ] {
            h ^= (v as i32) as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
        format!("V-{:04X}", (h >> 16) as u16)
    }

    /// Feed one profile into the live readouts and, while a capture is
    /// running, its accumulators. `fresh` marks a new analysis frame (the
    /// live path repaints ~3× per frame and calls this every repaint with
    /// `fresh` only on the first; a file import calls it once per frame).
    /// The microphone loop and the file-import worker both end here, which
    /// is what makes an imported file a capture like any other.
    pub(super) fn ingest_profile(&mut self, p: VocalProfile, fresh: bool) {
        if fresh {
            self.current_profile = p;

            // Coverage accounting, once per analysis frame (repaints arrive
            // ~3x more often than profiles; counting those would inflate
            // denominators with duplicates).
            let p = &self.current_profile;
            let identity_ok = p.valid
                && crate::math::formant_grade(&p.formants, p.formants_f0)
                    == crate::math::FormantGrade::Identity
                && p.metrics
                    .snr_db
                    .is_none_or(|snr| snr >= crate::math::IDENTITY_MIN_SNR_DB);
            if p.valid || p.metrics.voiced_but_noisy {
                self.cov_periodic += 1;
            }
            if p.valid {
                self.cov_voiced += 1;
                if identity_ok {
                    self.cov_identity += 1;
                }
                if matches!(self.rec, RecState::Recording { .. }) {
                    self.rec_cov_voiced += 1;
                    if identity_ok {
                        self.rec_cov_identity += 1;
                    }
                }
            }
        }
        if matches!(self.rec, RecState::Recording { .. }) {
            // Count every recording tick (voiced or not) so the enrollment
            // gate can compute the capture's voiced fraction.
            self.rec_frames_total += 1;
        }
        if matches!(self.rec, RecState::Recording { .. }) && self.current_profile.valid {
            self.rec_f0_acc.push(self.current_profile.f0);
            // Formants are latched by the reliability of their *measurement*
            // frame (`formants_f0`, not the current f0): LPC's harmonic-
            // attraction bias belongs to the frame the fit ran on. Identity
            // grade additionally feeds the voiceprint; rejects feed nothing.
            match crate::math::formant_grade(
                &self.current_profile.formants,
                self.current_profile.formants_f0,
            ) {
                crate::math::FormantGrade::Identity => {
                    let f = self.current_profile.formants;
                    self.rec_formants = Some(f);
                    // Identity data additionally demands measurement-grade
                    // SNR (ASHA's ≥30 dB; math::IDENTITY_MIN_SNR_DB): a
                    // frame can be clean enough to show and still too
                    // noise-contaminated to become part of who the app
                    // thinks this singer is.
                    let id_snr_ok = self
                        .current_profile
                        .metrics
                        .snr_db
                        .is_none_or(|snr| snr >= crate::math::IDENTITY_MIN_SNR_DB);
                    if id_snr_ok {
                        for (acc, fm) in self.rec_id_f_acc.iter_mut().zip(&f) {
                            if fm.frequency > 0.0 && fm.frequency.is_finite() {
                                acc.push(fm.frequency);
                            }
                        }
                        if let Some(l) =
                            crate::tract::vtl_from_formants(f[1].frequency, f[2].frequency)
                        {
                            self.rec_vtl_acc.push(l);
                        }
                    }
                }
                crate::math::FormantGrade::DisplayOnly => {
                    self.rec_formants = Some(self.current_profile.formants);
                }
                crate::math::FormantGrade::Reject => {}
            }
            if let Some(h) = self.current_profile.metrics.hnr_db {
                self.rec_hnr_acc.push(h);
            }
            if let Some(h) = self.current_profile.metrics.h1_h2_db {
                self.rec_h1h2_acc.push(h);
            }
            if let Some(j) = self.current_profile.metrics.jitter_pct {
                self.rec_jitter_acc.push(j);
            }
            if let Some(s) = self.current_profile.metrics.shimmer_db {
                self.rec_shimmer_acc.push(s);
            }
            if let Some(c) = self.current_profile.metrics.cpp_db {
                self.rec_cpp_acc.push(c);
            }
            if let Some(c) = self.current_profile.metrics.centroid_hz {
                self.rec_centroid_acc.push(c);
            }
        }
        for (ema, &a) in self
            .harm_ema
            .iter_mut()
            .zip(&self.current_profile.partial_amplitudes)
        {
            *ema += 0.3 * (a - *ema);
        }
        if matches!(self.rec, RecState::Recording { .. }) && self.current_profile.valid {
            // Snapshot the just-updated harm_ema (normalized to its own max)
            // for the capture-mean profile driving the Timbre classification.
            let max_amp = self
                .harm_ema
                .iter()
                .take(16)
                .cloned()
                .fold(0.0f32, f32::max);
            if max_amp > 1e-6 {
                for (sum, &a) in self.rec_profile_sum.iter_mut().zip(&self.harm_ema[..16]) {
                    *sum += a / max_amp;
                }
                self.rec_profile_n += 1;
            }
        }
    }
}
