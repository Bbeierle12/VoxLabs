//! The Overview screen: enrolled reference card and the compact recent-sessions list.

use super::*;

impl DashboardApp {
    pub(super) fn screen_overview(&mut self, ui: &mut egui::Ui) {
        // Header row.
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.screen_kicker(ui, "VOXLAB · ACOUSTIC BIOMETRICS", "Overview");
            });
            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::hover());
                ui.painter().circle_filled(rect.center(), 20.0, white(179));
                ui.painter()
                    .circle_stroke(rect.center(), 20.0, Stroke::new(1.0, white(230)));
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "AK",
                    FontId::proportional(13.0),
                    TEAL_DARK,
                );
            });
        });
        ui.add_space(18.0);

        // Reference-voiceprint hero card. Reflects real enrollment state: the
        // ring shows the latest capture's real match; the label is the enrolled
        // reference ID and true session count. Before enrollment it invites a
        // first capture rather than showing a fake subject.
        let latest = self.sessions.first().cloned();
        let count = self.sessions.len();
        let id_label = self.enrolled_id.clone();
        let reenroll_armed = self.reenroll_armed;
        let mut reenroll_clicked = false;
        glass(24.0).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(96.0, 96.0), Sense::hover());
                let ring_pct = latest.as_ref().and_then(|s| s.match_pct).unwrap_or(0.0);
                let ring_val = latest
                    .as_ref()
                    .map(|s| Self::match_text(s.match_pct))
                    .unwrap_or_else(|| "—".into());
                ring_gauge(
                    ui.painter(),
                    rect,
                    ring_pct,
                    TEAL,
                    &ring_val,
                    Some("MATCH %"),
                );
                ui.add_space(14.0);
                ui.vertical(|ui| match &id_label {
                    Some(id) => {
                        ui.label(
                            RichText::new("Reference voiceprint")
                                .size(12.0)
                                .color(ink(140)),
                        );
                        ui.add_space(3.0);
                        ui.label(
                            RichText::new(id)
                                .font(FontId::monospace(20.0))
                                .color(INK)
                                .strong(),
                        );
                        ui.add_space(6.0);
                        let plural = if count == 1 { "" } else { "s" };
                        ui.label(
                            RichText::new(format!("Enrolled · {count} session{plural}"))
                                .size(12.0)
                                .color(ink(140)),
                        );
                        ui.add_space(9.0);
                        if let Some(s) = &latest {
                            let (badge, bg, fg) = Self::badge_style(s.match_pct);
                            pill_badge(ui, badge, bg, fg);
                        }
                        ui.add_space(8.0);
                        // Two-tap re-enroll: a bad reference is recoverable
                        // without hand-deleting archive.json. Past sessions
                        // keep their historical scores.
                        let (relabel, recolor) = if reenroll_armed {
                            ("Tap again to clear the reference", AMBER_TEXT)
                        } else {
                            ("Re-enroll…", ink(128))
                        };
                        let resp = ui
                            .add(
                                egui::Label::new(RichText::new(relabel).size(11.0).color(recolor))
                                    .sense(Sense::click()),
                            )
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        if resp.clicked() {
                            reenroll_clicked = true;
                        }
                    }
                    None => {
                        ui.label(
                            RichText::new("No voiceprint enrolled")
                                .size(12.0)
                                .color(ink(140)),
                        );
                        ui.add_space(3.0);
                        ui.label(
                            RichText::new("—")
                                .font(FontId::monospace(20.0))
                                .color(INK)
                                .strong(),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Capture and save to enroll your reference")
                                .size(12.0)
                                .color(ink(140)),
                        );
                    }
                });
            });
        });
        if reenroll_clicked {
            if self.reenroll_armed {
                // Confirmed: clear the reference. The next capture that passes
                // the enrollment gate becomes the new one on save.
                self.enrolled = None;
                self.enrolled_id = None;
                self.reenroll_armed = false;
                self.persist_state();
            } else {
                self.reenroll_armed = true;
            }
        }
        ui.add_space(12.0);

        // Metric tiles (2 × 2) — real aggregates across the saved archive.
        // Sessions are stored newest-first; `.rev()` gives oldest→newest for
        // the trend sparkline.
        let vals_f0: Vec<f32> = self.sessions.iter().rev().map(|s| s.f0).collect();
        let vals_jit: Vec<f32> = self
            .sessions
            .iter()
            .rev()
            .filter_map(|s| s.jitter_pct)
            .collect();
        let vals_shim: Vec<f32> = self
            .sessions
            .iter()
            .rev()
            .filter_map(|s| s.shimmer_db)
            .collect();
        let vals_hnr: Vec<f32> = self
            .sessions
            .iter()
            .rev()
            .filter_map(|s| s.hnr_db)
            .collect();

        let mean = |v: &[f32]| (!v.is_empty()).then(|| v.iter().sum::<f32>() / v.len() as f32);
        // Last up-to-8 values → sparkline points (higher value = toward top).
        let spark = |v: &[f32]| -> Vec<(f32, f32)> {
            let w = &v[v.len().saturating_sub(8)..];
            if w.len() < 2 {
                return Vec::new();
            }
            let (mn, mx) = w
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
            let range = (mx - mn).max(1e-6);
            w.iter()
                .enumerate()
                .map(|(i, &x)| {
                    let px = i as f32 / (w.len() - 1) as f32 * 100.0;
                    let py = 22.0 - (x - mn) / range * 20.0;
                    (px, py)
                })
                .collect()
        };
        let fmt = |v: Option<f32>, d: usize| match v {
            Some(x) => format!("{x:.*}", d),
            None => "—".into(),
        };
        let tiles = [
            (
                "F0 MEAN",
                fmt(mean(&vals_f0), 1),
                "Hz",
                "across saved captures",
                spark(&vals_f0),
            ),
            (
                "JITTER",
                fmt(mean(&vals_jit), 2),
                "%",
                "ref < 1.04 %",
                spark(&vals_jit),
            ),
            (
                "SHIMMER",
                fmt(mean(&vals_shim), 2),
                "dB",
                "ref < 0.35 dB",
                spark(&vals_shim),
            ),
            (
                "HNR",
                fmt(mean(&vals_hnr), 1),
                "dB",
                "ref > 17 dB",
                spark(&vals_hnr),
            ),
        ];
        for row in tiles.chunks(2) {
            ui.columns(2, |cols| {
                for (col, (label, value, unit, reference, pts)) in cols.iter_mut().zip(row) {
                    glass(20.0).show(col, |ui| {
                        ui.label(
                            RichText::new(*label)
                                .font(FontId::monospace(10.0))
                                .color(ink(128)),
                        );
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(value)
                                    .font(FontId::monospace(22.0))
                                    .color(INK)
                                    .strong(),
                            );
                            ui.label(RichText::new(*unit).size(12.0).color(ink(128)));
                        });
                        ui.add_space(7.0);
                        let (rect, _) = ui
                            .allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
                        sparkline(ui.painter(), rect, pts);
                        ui.add_space(6.0);
                        ui.label(RichText::new(*reference).size(10.5).color(ink(115)));
                    });
                }
            });
            ui.add_space(12.0);
        }
        ui.add_space(8.0);

        // Recent captures.
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Recent captures")
                    .size(14.0)
                    .color(INK)
                    .strong(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let resp = ui
                    .label(RichText::new("View all").size(12.5).color(TEAL).strong())
                    .interact(Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if resp.clicked() {
                    self.screen = Screen::Sessions;
                }
            });
        });
        ui.add_space(10.0);

        if self.sessions.is_empty() {
            ui.label(RichText::new("No captures yet.").size(12.5).color(ink(120)));
        }
        let recent: Vec<(usize, Session)> =
            self.sessions.iter().take(3).cloned().enumerate().collect();
        for (i, s) in recent {
            if self.session_row_compact(ui, &s, i) {
                self.selected = Some(i);
                self.export_queued = false;
                self.screen = Screen::Detail;
            }
            ui.add_space(9.0);
        }
    }

    /// Overview "recent capture" row; returns true when clicked.
    pub(super) fn session_row_compact(&self, ui: &mut egui::Ui, s: &Session, i: usize) -> bool {
        let (_, badge_bg, badge_fg) = Self::badge_style(s.match_pct);
        let ir = glass(18.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(36.0, 36.0), Sense::hover());
                ui.painter().rect_filled(rect, 12.0, teal_a(26));
                waveform_glyph(
                    ui.painter(),
                    Rect::from_center_size(rect.center(), vec2(16.0, 16.0)),
                    TEAL,
                );
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&s.id)
                            .font(FontId::monospace(13.5))
                            .color(INK)
                            .strong(),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(format!("{} · {}", s.subj, s.date))
                            .size(11.5)
                            .color(ink(128)),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let text = Self::match_text(s.match_pct);
                    let font = FontId::monospace(12.0);
                    let galley = ui
                        .painter()
                        .layout_no_wrap(text.clone(), font.clone(), badge_fg);
                    let size = vec2(galley.size().x + 20.0, 24.0);
                    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                    ui.painter().rect_filled(rect, 12.0, badge_bg);
                    ui.painter()
                        .text(rect.center(), Align2::CENTER_CENTER, text, font, badge_fg);
                });
            });
        });
        ui.interact(
            ir.response.rect,
            ui.id().with("recent").with(i),
            Sense::click(),
        )
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    }
}
