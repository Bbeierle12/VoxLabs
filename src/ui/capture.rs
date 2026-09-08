//! The Capture screen: live readouts, record controls, and the result card.

use super::*;

impl DashboardApp {
    pub(super) fn screen_capture(&mut self, ui: &mut egui::Ui, now: f64) {
        let chip = self
            .enrolled_id
            .clone()
            .unwrap_or_else(|| "Not enrolled".into());
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.screen_kicker(ui, "LIVE ACQUISITION", "Capture");
            });
            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                let font = FontId::monospace(12.5);
                let galley = ui.painter().layout_no_wrap(chip.clone(), font.clone(), INK);
                let size = vec2(galley.size().x + 38.0, 30.0);
                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                ui.painter().rect(
                    rect,
                    15.0,
                    white(179),
                    Stroke::new(1.0, white(230)),
                    StrokeKind::Inside,
                );
                ui.painter()
                    .circle_filled(pos2(rect.left() + 15.0, rect.center().y), 3.5, TEAL);
                ui.painter().text(
                    pos2(rect.left() + 26.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    &chip,
                    font,
                    INK,
                );
            });
        });
        ui.add_space(16.0);

        // Room status, read-only here — the interactive calibration card
        // lives on the Room tab.
        self.room_status_line(ui);
        ui.add_space(10.0);

        let recording = matches!(self.rec, RecState::Recording { .. });

        // Vocal tract card — THE articulatory model, live. The mesh is the
        // measured Story area function: the 44 ring diameters are
        // diameters(basis, q1, q2) from the current formant inversion,
        // VTL-scaled, and they move only when the frame passed every gate
        // (the same LIVE/NOISY/HELD/CALIBRATING states as everywhere else).
        // Before the first measurement the model rests at its neutral
        // posture — a labeled model state, never fabricated motion.
        glass(24.0).show(ui, |ui| {
            let basis = crate::tract::basis_for_vtl(self.vtl_est_cm);
            let (state, accent, live) = self.tract_state();
            let (q1, q2) = self.tract_q.unwrap_or((0.0, 0.0));
            let d = crate::tract::diameters(basis, q1, q2);

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Vocal tract · articulatory model")
                        .size(13.0)
                        .color(INK)
                        .strong(),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Two distinct facts, two chips. LIVE (cyan, blinking)
                    // means a recording is running — the acquisition is
                    // live, exactly as the card always showed. The model
                    // chip beside it says what the mesh is doing with that
                    // audio: TRACKING on gated frames, else HELD / NOISY /
                    // CALIBRATING / —. Dropping the LIVE indicator in favor
                    // of the model state alone read as "live capture is
                    // gone" — it isn't, and the card must not imply it.
                    if recording {
                        let blink = if now.fract() < 0.5 { 255 } else { 64 };
                        // A file import drives the same card from a file,
                        // not the microphone: say so instead of LIVE.
                        let source = if self.import_job.is_some() {
                            "FILE"
                        } else {
                            "LIVE"
                        };
                        ui.label(
                            RichText::new(source)
                                .font(FontId::monospace(10.0))
                                .color(CYAN_DEEP),
                        );
                        let (dot, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                        ui.painter().circle_filled(
                            dot.center(),
                            3.0,
                            Color32::from_rgba_unmultiplied(6, 182, 212, blink),
                        );
                        ui.add_space(10.0);
                    }
                    ui.label(
                        RichText::new(state)
                            .font(FontId::monospace(10.0))
                            .color(accent)
                            .strong(),
                    );
                });
            });
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 236.0), Sense::hover());
            paint_tract(ui.painter(), rect, (now * 1000.0) as f32, live, &d);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("SAGITTAL MESH · 44 SECTIONS")
                        .font(FontId::monospace(9.5))
                        .color(ink(107)),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let right = if self.tract_q.is_some() {
                        "AREA FN · STORY FIT"
                    } else {
                        "AREA FN · NEUTRAL"
                    };
                    ui.label(
                        RichText::new(right)
                            .font(FontId::monospace(9.5))
                            .color(ink(107)),
                    );
                });
            });

            // The data-log half: vowel-space trail beside the caption. Same
            // card, not another mode or tab — the model IS the instrument.
            ui.add_space(8.0);
            let sep = ui.available_rect_before_wrap();
            ui.painter().line_segment(
                [pos2(sep.left(), sep.top()), pos2(sep.right(), sep.top())],
                Stroke::new(1.0, ink(18)),
            );
            ui.add_space(8.0);
            let map_side = 96.0f32;
            let (row, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), map_side), Sense::hover());
            let map = Rect::from_min_size(row.min, vec2(map_side, map_side));
            self.vowel_map(ui.painter(), map);
            let text_left = map.right() + 12.0;
            ui.painter().text(
                pos2(text_left, row.top() + 2.0),
                Align2::LEFT_TOP,
                "VOWEL SPACE · SESSION LOG",
                FontId::monospace(9.5),
                ink(115),
            );
            // Caption wrapped into the remaining width.
            let caption = self.tract_caption(basis);
            let galley = ui.painter().layout(
                caption,
                FontId::monospace(9.0),
                ink(125),
                (row.right() - text_left).max(60.0),
            );
            ui.painter()
                .galley(pos2(text_left, row.top() + 20.0), galley, ink(125));
        });
        ui.add_space(12.0);

        // Waveform + live readouts.
        glass(24.0).show(ui, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::hover());
            paint_wave(ui.painter(), rect, &self.wave);
            ui.add_space(11.0);
            let sep = ui.available_rect_before_wrap();
            ui.painter().line_segment(
                [pos2(sep.left(), sep.top()), pos2(sep.right(), sep.top())],
                Stroke::new(1.0, ink(18)),
            );
            ui.add_space(11.0);

            let f0 = if recording && self.current_profile.valid {
                format!("{:.1} Hz", self.current_profile.f0)
            } else {
                "—".into()
            };
            let level = if recording {
                let rms = self.input_rms().max(1e-6);
                format!("{:.1} dB", (20.0 * rms.log10()).clamp(-90.0, 0.0))
            } else {
                "—".into()
            };
            let elapsed = match self.rec {
                RecState::Recording { start } => now - start,
                RecState::Analyzing { elapsed, .. } | RecState::Done { elapsed } => elapsed,
                RecState::Idle => 0.0,
            };
            let timer = format!("{}:{:04.1}", (elapsed / 60.0) as u32, elapsed % 60.0);
            ui.columns(3, |cols| {
                for (col, (label, value)) in
                    cols.iter_mut()
                        .zip([("F0", f0), ("LEVEL", level), ("ELAPSED", timer)])
                {
                    col.label(
                        RichText::new(label)
                            .font(FontId::monospace(9.5))
                            .color(ink(115)),
                    );
                    col.add_space(3.0);
                    col.label(
                        RichText::new(value)
                            .font(FontId::monospace(16.0))
                            .color(INK)
                            .strong(),
                    );
                }
            });
        });
        ui.add_space(14.0);

        // Harmonic series (live, musician-facing).
        self.harmonics_card(ui);
        ui.add_space(14.0);

        // Analyzing card.
        if let RecState::Analyzing { start, .. } = self.rec {
            glass(22.0).show(ui, |ui| {
                ui.label(
                    RichText::new("Analyzing capture")
                        .size(14.0)
                        .color(INK)
                        .strong(),
                );
                ui.add_space(10.0);
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::hover());
                ui.painter().rect_filled(rect, 3.0, teal_a(31));
                let frac = (((now - start) / 2.0).clamp(0.04, 1.0)) as f32;
                let bar = Rect::from_min_size(rect.min, vec2(rect.width() * frac, 6.0));
                ui.painter().rect_filled(bar, 3.0, TEAL);
                ui.add_space(10.0);
                let analyzing_line = match &self.enrolled_id {
                    Some(id) => format!("EXTRACTING VOICEPRINT · SCORING VS {id}"),
                    None => "EXTRACTING VOICEPRINT · NO REFERENCE YET".to_string(),
                };
                ui.label(
                    RichText::new(analyzing_line)
                        .font(FontId::monospace(10.5))
                        .color(ink(128)),
                );
            });
            ui.add_space(14.0);
        }

        // Result card.
        if matches!(self.rec, RecState::Done { .. })
            && let Some(res) = &self.result
        {
            let is_ref = res.is_reference;
            let (badge, badge_bg, badge_fg) = if is_ref {
                ("Reference", teal_a(31), TEAL_DARK)
            } else {
                Self::badge_style(res.match_pct)
            };
            let match_str = Self::match_text(res.match_pct);
            let heading = match (&self.enrolled_id, is_ref) {
                (_, true) => "First capture — enrolls your reference voiceprint".to_string(),
                (Some(id), false) => format!("Similarity vs voiceprint {id}"),
                (None, false) => "No reference enrolled yet".to_string(),
            };
            let blocked = res.blocked.clone();
            let mut save = false;
            let mut discard = false;
            glass(24.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(heading).size(13.0).color(ink(140)));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        pill_badge(ui, badge, badge_bg, badge_fg);
                    });
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(match_str)
                            .font(FontId::monospace(44.0))
                            .color(INK)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("%")
                            .font(FontId::monospace(18.0))
                            .color(ink(128)),
                    );
                });
                ui.add_space(6.0);
                self.capture_export_line(ui);
                ui.add_space(10.0);
                if let Some(reason) = &blocked {
                    // Save is withheld (G2): say why inline; only Discard.
                    ui.label(RichText::new(reason.as_str()).size(12.0).color(AMBER_TEXT));
                    ui.add_space(10.0);
                    discard = pill_button(
                        ui,
                        vec2(ui.available_width(), 46.0),
                        "Discard",
                        white(179),
                        INK,
                        Stroke::new(1.0, ink(31)),
                    );
                } else {
                    ui.horizontal(|ui| {
                        let w = (ui.available_width() - 10.0) / 2.0;
                        save = pill_button(
                            ui,
                            vec2(w, 46.0),
                            "Save session",
                            Color32::from_rgb(17, 166, 166),
                            Color32::WHITE,
                            Stroke::NONE,
                        );
                        ui.add_space(10.0);
                        discard = pill_button(
                            ui,
                            vec2(w, 46.0),
                            "Discard",
                            white(179),
                            INK,
                            Stroke::new(1.0, ink(31)),
                        );
                    });
                }
            });
            if save {
                self.save_session();
            }
            if discard {
                self.rec = RecState::Idle;
                self.result = None;
            }
        }

        // Record button.
        if matches!(self.rec, RecState::Idle | RecState::Recording { .. }) {
            ui.add_space(6.0);
            ui.vertical_centered(|ui| {
                let (rect, resp) = ui.allocate_exact_size(vec2(100.0, 100.0), Sense::click());
                let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                let c = rect.center();

                if recording {
                    // Two expanding pulse rings, 1.8 s period, 0.9 s apart.
                    for phase in [0.0, 0.5] {
                        let t = ((now / 1.8 + phase).fract()) as f32;
                        let r = 46.0 * (0.9 + 0.65 * t);
                        let a = (0.7 * (1.0 - t) * 255.0) as u8;
                        ui.painter().circle_stroke(
                            c,
                            r,
                            Stroke::new(2.0, Color32::from_rgba_unmultiplied(6, 182, 212, a / 2)),
                        );
                    }
                }

                // Button disc (flat stand-in for the teal→cyan gradient).
                ui.painter()
                    .circle_filled(c, 42.0, Color32::from_rgb(18, 168, 178));
                ui.painter()
                    .circle_stroke(c, 42.0, Stroke::new(1.0, white(120)));
                if recording {
                    let sq = Rect::from_center_size(c, vec2(26.0, 26.0));
                    ui.painter().rect_filled(sq, 7.0, Color32::WHITE);
                } else {
                    mic_glyph(
                        ui.painter(),
                        Rect::from_center_size(c, vec2(30.0, 30.0)),
                        Color32::WHITE,
                        true,
                    );
                }

                if resp.clicked() {
                    if self.import_job.is_some() {
                        self.cancel_import();
                    } else if recording {
                        self.stop_rec(now);
                    } else {
                        self.start_rec(now);
                    }
                }

                ui.add_space(12.0);
                let hint = if let Some(job) = &self.import_job {
                    format!(
                        "Analyzing {} · {:.0}% — tap to cancel",
                        job.name,
                        100.0 * job.progress()
                    )
                } else if let RecState::Recording { start } = self.rec {
                    format!(
                        "Recording {:.0} s{} — tap to stop (auto-stops at {MAX_REC_SECS:.0} s)",
                        now - start,
                        if capture_export::is_armed() {
                            " · saving audio"
                        } else {
                            ""
                        }
                    )
                } else {
                    "Tap to begin capture · sustained /a/ · 8 s min".to_string()
                };
                ui.label(
                    RichText::new(hint)
                        .font(FontId::monospace(11.0))
                        .color(ink(128)),
                );
            });
        }
    }
}
