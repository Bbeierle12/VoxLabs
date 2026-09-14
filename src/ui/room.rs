//! The Room screen: ambient floor, calibration, coverage, and the room status rows.

use super::*;

impl DashboardApp {
    /// The Room screen: everything DETERMINISTIC about the acoustic space.
    /// Ambient floor, one-tap calibration, the fingerprinted persistent
    /// tones (fans, HVAC, mains hum, the TV's *hardware* noise — measured
    /// with the TV on and muted). Program audio is out of scope on purpose:
    /// a show's soundtrack is non-stationary, so no profile taken now
    /// predicts it later. That boundary is a finding, not a limitation to
    /// apologize for.
    pub(super) fn screen_room(&mut self, ui: &mut egui::Ui) {
        use crate::concurrency::CalibState;
        self.screen_kicker(ui, "ACOUSTIC ENVIRONMENT", "Room");
        ui.add_space(16.0);

        // What the app knows about its own microphone and analysis state,
        // and the recent log — the bug report for a phone without adb.
        self.diagnostics_card(ui);
        ui.add_space(12.0);

        // The analysis pipeline's taps and timing (Plan v3 Phase 1 gate).
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.pipeline_card(ui);
            ui.add_space(12.0);
        }

        // ── Levels: live input against the learned floor ──
        glass(20.0).show(ui, |ui| {
            ui.label(
                RichText::new("LEVELS")
                    .font(FontId::monospace(9.5))
                    .color(ink(115)),
            );
            ui.add_space(8.0);

            let live_rms = self.input_rms();
            let live_db = 20.0 * live_rms.max(1e-6).log10();
            let (ambient, _) = self.telemetry.calibration_summary();
            let floor_db = (self.telemetry.calib_state() == CalibState::Done)
                .then(|| 20.0 * ambient.max(1e-6).log10());
            let snr = self.current_profile.metrics.snr_db;

            // Bar from -70 dBFS to 0.
            const LO: f32 = -70.0;
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 16.0), Sense::hover());
            let track = Rect::from_min_size(
                pos2(rect.left(), rect.center().y - 4.0),
                vec2(rect.width(), 8.0),
            );
            let to_x = |db: f32| -> f32 {
                let t = ((db - LO) / -LO).clamp(0.0, 1.0);
                track.left() + t * track.width()
            };
            ui.painter().rect_filled(track, 4.0, ink(10));
            let live_x = to_x(live_db);
            ui.painter().rect_filled(
                Rect::from_min_max(track.min, pos2(live_x, track.bottom())),
                4.0,
                teal_a(90),
            );
            if let Some(f) = floor_db {
                let fx = to_x(f);
                ui.painter().line_segment(
                    [pos2(fx, track.top() - 3.0), pos2(fx, track.bottom() + 3.0)],
                    Stroke::new(2.0, AMBER),
                );
            }
            ui.add_space(6.0);
            let mut line = format!("input {live_db:.0} dBFS");
            match floor_db {
                Some(f) => line.push_str(&format!(" · floor {f:.0} dBFS")),
                None => line.push_str(" · floor: learning passively"),
            }
            if let Some(v) = snr {
                line.push_str(&format!(" · SNR {v:.0} dB"));
            }
            ui.label(
                RichText::new(line)
                    .font(FontId::monospace(10.5))
                    .color(ink(150)),
            );
        });
        ui.add_space(12.0);

        // ── Calibration (the interactive card lives here, not on Capture) ──
        self.room_row(ui);
        ui.add_space(12.0);

        // ── Persistent tones ──
        glass(20.0).show(ui, |ui| {
            ui.label(
                RichText::new("PERSISTENT TONES")
                    .font(FontId::monospace(9.5))
                    .color(ink(115)),
            );
            ui.add_space(8.0);
            let (_, hum) = self.telemetry.calibration_summary();
            match hum {
                Some((f0, rms)) if self.telemetry.calib_state() == CalibState::Done => {
                    let db = 20.0 * rms.max(1e-6).log10();
                    ui.label(
                        RichText::new(format!(
                            "{f0:.0} Hz · {db:.0} dBFS — gated (a louder voice on this pitch \
                             still passes)"
                        ))
                        .size(12.0)
                        .color(INK),
                    );
                }
                _ => {
                    ui.label(
                        RichText::new(
                            "None fingerprinted. Calibrate with the steady sources running — \
                             fans, HVAC, and the TV powered on but MUTED (that captures its \
                             hardware hum, the part that persists).",
                        )
                        .size(12.0)
                        .color(ink(140)),
                    );
                }
            }
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "Out of scope by design: TV shows, music, speech from speakers. Program \
                     audio is non-stationary — no profile taken now predicts what plays \
                     next. For singing over media, use headphones.",
                )
                .size(10.5)
                .color(ink(115)),
            );
        });
        ui.add_space(12.0);

        // ── Coverage: what the gating costs, measured ──
        glass(20.0).show(ui, |ui| {
            ui.label(
                RichText::new("COVERAGE · THIS SESSION")
                    .font(FontId::monospace(9.5))
                    .color(ink(115)),
            );
            ui.add_space(8.0);
            if self.cov_periodic == 0 {
                ui.label(
                    RichText::new("No periodic frames yet — sing or speak to accumulate.")
                        .size(12.0)
                        .color(ink(140)),
                );
            } else {
                let pct = |n: u32, d: u32| -> f32 {
                    if d == 0 {
                        0.0
                    } else {
                        100.0 * n as f32 / d as f32
                    }
                };
                ui.label(
                    RichText::new(format!(
                        "voiced (SNR + hum gates): {:.0}% of {} periodic frames",
                        pct(self.cov_voiced, self.cov_periodic),
                        self.cov_periodic
                    ))
                    .font(FontId::monospace(11.0))
                    .color(INK),
                );
                ui.add_space(3.0);
                ui.label(
                    RichText::new(format!(
                        "identity-grade (f0 ≤ 200 Hz · SNR ≥ 30 dB): {:.0}% of voiced",
                        pct(self.cov_identity, self.cov_voiced)
                    ))
                    .font(FontId::monospace(11.0))
                    .color(INK),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(
                        "A measurement kept at 90% coverage and one kept at 10% are \
                         different claims — this is the price of every gate, in the open.",
                    )
                    .size(10.5)
                    .color(ink(115)),
                );
            }
        });
        ui.add_space(12.0);
        self.device_probe_card(ui);
        ui.add_space(12.0);
        self.tv_path_card(ui);
    }

    /// One-line, read-only room status for the Capture screen; the
    /// interactive calibration card lives on the Room tab.
    pub(super) fn room_status_line(&self, ui: &mut egui::Ui) {
        use crate::concurrency::CalibState;
        let text = match self.telemetry.calib_state() {
            CalibState::Done => {
                let (ambient, hum) = self.telemetry.calibration_summary();
                let mut t = format!("ROOM · floor {:.0} dBFS", 20.0 * ambient.max(1e-6).log10());
                if let Some((f0, _)) = hum {
                    t.push_str(&format!(" · hum {f0:.0} Hz gated"));
                }
                if let Some(snr) = self.current_profile.metrics.snr_db {
                    t.push_str(&format!(" · SNR {snr:.0} dB"));
                }
                t
            }
            CalibState::Running => "ROOM · calibrating — keep silent".to_string(),
            _ => "ROOM · uncalibrated — see the Room tab".to_string(),
        };
        ui.label(
            RichText::new(text)
                .font(FontId::monospace(10.0))
                .color(ink(125)),
        );
    }

    /// Room-status row on the Capture screen. Shows what the noise gating
    /// knows (ambient floor, fingerprinted hum, live SNR) and offers the
    /// calibration pass. Copy is deliberate about scope: a *steady* hum can
    /// be fingerprinted; a TV or music cannot (non-stationary), and the row
    /// never claims otherwise.
    pub(super) fn room_row(&mut self, ui: &mut egui::Ui) {
        use crate::concurrency::CalibState;
        let state = self.telemetry.calib_state();
        let live_snr = self.current_profile.metrics.snr_db;

        glass(16.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("ROOM")
                        .font(FontId::monospace(9.5))
                        .color(ink(115)),
                );
                ui.add_space(8.0);

                let status = match state {
                    CalibState::Idle => "uncalibrated · floor learns passively".to_string(),
                    CalibState::Running => {
                        let secs =
                            self.telemetry.calib_frames_left() as f32 * ANALYSIS_FRAME_SECS_APPROX;
                        format!("listening — keep silent · {secs:.1} s")
                    }
                    CalibState::Done => {
                        let (ambient, hum) = self.telemetry.calibration_summary();
                        let floor_db = 20.0 * ambient.max(1e-7).log10();
                        let mut t = format!("floor {floor_db:.0} dBFS");
                        if let Some((f0, _)) = hum {
                            t.push_str(&format!(" · hum @ {f0:.0} Hz gated"));
                        }
                        if let Some(snr) = live_snr {
                            t.push_str(&format!(" · SNR {snr:.0} dB"));
                        }
                        t
                    }
                    CalibState::FailedVoice => "heard a voice — retry in silence".to_string(),
                };
                let status_color = match state {
                    CalibState::FailedVoice => AMBER_TEXT,
                    CalibState::Running => TEAL_DARK,
                    _ => ink(150),
                };
                ui.label(RichText::new(status).size(11.5).color(status_color));

                if state != CalibState::Running {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let label = if state == CalibState::Done {
                            "RECALIBRATE"
                        } else {
                            "CALIBRATE"
                        };
                        let font = FontId::monospace(9.5);
                        let galley =
                            ui.painter()
                                .layout_no_wrap(label.into(), font.clone(), TEAL_DARK);
                        let size = vec2(galley.size().x + 20.0, 24.0);
                        let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
                        let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                        ui.painter().rect(
                            rect,
                            12.0,
                            teal_a(26),
                            Stroke::new(1.0, teal_a(64)),
                            StrokeKind::Inside,
                        );
                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            label,
                            font,
                            TEAL_DARK,
                        );
                        if resp.clicked() {
                            self.telemetry.start_calibration(crate::math::CALIB_FRAMES);
                        }
                    });
                }
            });
        });
    }
}
