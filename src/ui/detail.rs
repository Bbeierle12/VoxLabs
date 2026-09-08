//! The Session detail screen.

use super::*;

impl DashboardApp {
    pub(super) fn screen_detail(&mut self, ui: &mut egui::Ui) {
        // Resolve the selected session; fall back to Sessions if the archive is
        // empty or the selection is stale (no unwrap on an empty vec).
        let sel = match self.selected.and_then(|i| self.sessions.get(i)) {
            Some(s) => s.clone(),
            None => {
                self.screen = Screen::Sessions;
                return;
            }
        };
        let (badge, badge_bg, badge_fg) = Self::badge_style(sel.match_pct);
        let ring_color = if Self::verified(sel.match_pct) {
            TEAL
        } else {
            AMBER
        };

        // Back + title.
        ui.horizontal(|ui| {
            let (rect, resp) = ui.allocate_exact_size(vec2(38.0, 38.0), Sense::click());
            let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            ui.painter().circle_filled(rect.center(), 19.0, white(179));
            ui.painter()
                .circle_stroke(rect.center(), 19.0, Stroke::new(1.0, white(230)));
            let c = rect.center();
            let stroke = Stroke::new(2.4, INK);
            ui.painter()
                .line_segment([pos2(c.x + 3.0, c.y - 7.0), pos2(c.x - 3.0, c.y)], stroke);
            ui.painter()
                .line_segment([pos2(c.x - 3.0, c.y), pos2(c.x + 3.0, c.y + 7.0)], stroke);
            if resp.clicked() {
                self.screen = Screen::Sessions;
            }
            ui.add_space(12.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(&sel.id)
                        .font(FontId::monospace(20.0))
                        .color(INK)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{} · {}", sel.subj, sel.date))
                        .size(12.0)
                        .color(ink(128)),
                );
            });
        });
        ui.add_space(16.0);

        // Verification score card.
        glass(24.0).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(86.0, 86.0), Sense::hover());
                ring_gauge(
                    ui.painter(),
                    rect,
                    sel.match_pct.unwrap_or(0.0),
                    ring_color,
                    &Self::match_text(sel.match_pct),
                    None,
                );
                ui.add_space(14.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("Match score").size(12.0).color(ink(140)));
                    ui.add_space(3.0);
                    let vp_line = self
                        .enrolled_id
                        .as_ref()
                        .map(|id| format!("vs voiceprint {id}"))
                        .unwrap_or_else(|| "vs reference voiceprint".into());
                    ui.label(RichText::new(vp_line).size(15.0).color(INK).strong());
                    ui.add_space(8.0);
                    pill_badge(ui, badge, badge_bg, badge_fg);
                });
            });
        });
        ui.add_space(14.0);

        // Acoustic parameters — measured values only (means / stop-time
        // snapshot from the capture; static demo values for seeded rows).
        // Reference ranges are musician guidance, not diagnosis; the neutral
        // dot means "not measured".
        let dot = |ok: bool| if ok { TEAL } else { AMBER };
        let opt_dot = |v: Option<f32>, ok: fn(f32) -> bool| match v {
            Some(v) => dot(ok(v)),
            None => ink(64),
        };
        let opt_val = |v: Option<f32>, unit: &str, digits: usize| match v {
            Some(v) => format!("{v:.digits$} {unit}"),
            None => "—".into(),
        };
        let params: [(String, String, &str, Color32); 8] = [
            (
                "F0 mean".into(),
                format!("{:.1} Hz", sel.f0),
                "85–255 Hz",
                dot((85.0..=255.0).contains(&sel.f0)),
            ),
            (
                "Jitter (local)".into(),
                opt_val(sel.jitter_pct, "%", 2),
                "< 1.04 %",
                opt_dot(sel.jitter_pct, |v| v < 1.04),
            ),
            (
                "Shimmer".into(),
                opt_val(sel.shimmer_db, "dB", 2),
                "< 0.35 dB",
                opt_dot(sel.shimmer_db, |v| v < 0.35),
            ),
            (
                "HNR".into(),
                opt_val(sel.hnr_db, "dB", 1),
                "> 17 dB",
                opt_dot(sel.hnr_db, |v| v > 17.0),
            ),
            (
                "CPP".into(),
                opt_val(sel.cpp_db, "dB", 1),
                "> 11 dB",
                opt_dot(sel.cpp_db, |v| v > 11.0),
            ),
            (
                "H1–H2".into(),
                opt_val(sel.h1_h2_db, "dB", 1),
                "0–10 dB",
                opt_dot(sel.h1_h2_db, |v| (0.0..=10.0).contains(&v)),
            ),
            (
                "Vibrato".into(),
                sel.vibrato
                    .map(|v| format!("{:.1} Hz ±{:.0}¢", v.rate_hz, v.extent_cents))
                    .unwrap_or_else(|| "—".into()),
                "4.5–6.5 Hz",
                match sel.vibrato {
                    Some(v) => dot((4.5..=6.5).contains(&v.rate_hz)),
                    None => ink(64),
                },
            ),
            (
                "Steadiness".into(),
                opt_val(sel.steadiness_cents, "¢", 0),
                "< 15 ¢",
                opt_dot(sel.steadiness_cents, |v| v < 15.0),
            ),
        ];

        // Formant chips: measured values only — an unresolved formant reads
        // "—", and there is no F4 chip (the engine extracts three formants;
        // deriving a fourth from f0 was prototype residue, not a measurement).
        let chips: Vec<(String, Option<f32>)> = match sel.formants {
            Some(f) => f
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    (
                        format!("F{}", i + 1),
                        (f.frequency > 0.0).then_some(f.frequency),
                    )
                })
                .collect(),
            None => (1..=3).map(|i| (format!("F{i}"), None)).collect(),
        };

        glass(24.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Acoustic parameters")
                        .size(14.0)
                        .color(INK)
                        .strong(),
                );
                // Identity coverage: how much of this capture's voiced signal
                // met every identity gate. "—" on pre-statistic sessions.
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let text = match sel.coverage_pct {
                        Some(c) => format!("id coverage {c:.0}%"),
                        None => "id coverage —".to_string(),
                    };
                    ui.label(
                        RichText::new(text)
                            .font(FontId::monospace(9.5))
                            .color(ink(115)),
                    );
                });
            });
            ui.add_space(4.0);
            for (name, value, reference, dot_color) in &params {
                ui.add_space(11.0);
                ui.horizontal(|ui| {
                    // Value column sized for the widest case ("5.1 Hz ±55¢").
                    let name_w = ui.available_width() - 104.0 - 84.0 - 12.0 - 24.0;
                    ui.allocate_ui_with_layout(
                        vec2(name_w, 18.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_width(name_w);
                            ui.label(RichText::new(name).size(13.0).color(ink(191)));
                        },
                    );
                    ui.allocate_ui_with_layout(
                        vec2(104.0, 18.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_width(104.0);
                            ui.label(
                                RichText::new(value)
                                    .font(FontId::monospace(13.0))
                                    .color(INK)
                                    .strong(),
                            );
                        },
                    );
                    ui.allocate_ui_with_layout(
                        vec2(84.0, 18.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_width(84.0);
                            ui.label(
                                RichText::new(*reference)
                                    .font(FontId::monospace(10.5))
                                    .color(ink(107)),
                            );
                        },
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (rect, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                        ui.painter().circle_filled(rect.center(), 4.0, *dot_color);
                    });
                });
                ui.add_space(11.0);
                let sep = ui.available_rect_before_wrap();
                ui.painter().line_segment(
                    [pos2(sep.left(), sep.top()), pos2(sep.right(), sep.top())],
                    Stroke::new(1.0, ink(15)),
                );
            }
            ui.add_space(14.0);
            let n = chips.len();
            ui.columns(n, |cols| {
                for (col, (label, freq)) in cols.iter_mut().zip(&chips) {
                    let rect = col.available_rect_before_wrap();
                    let chip = Rect::from_min_size(rect.min, vec2(rect.width(), 52.0));
                    col.painter().rect(
                        chip,
                        14.0,
                        teal_a(18),
                        Stroke::new(1.0, teal_a(31)),
                        StrokeKind::Inside,
                    );
                    col.painter().text(
                        pos2(chip.center().x, chip.top() + 12.0),
                        Align2::CENTER_CENTER,
                        label,
                        FontId::monospace(9.5),
                        ink(128),
                    );
                    col.painter().text(
                        pos2(chip.center().x, chip.top() + 32.0),
                        Align2::CENTER_CENTER,
                        match freq {
                            Some(v) => format!("{v:.0}"),
                            None => "—".to_string(),
                        },
                        FontId::monospace(13.0),
                        TEAL_DARK,
                    );
                    col.allocate_exact_size(vec2(rect.width(), 52.0), Sense::hover());
                }
            });

            // Personal Harmonic Identifier port: plain-language classifiers
            // derived at render time from the session's stored profile, not
            // pre-rendered strings.
            ui.add_space(10.0);
            let even_odd = crate::math::even_odd_balance_db(&sel.profile);
            let timbre = crate::math::timbre_description(&sel.profile, even_odd);
            let voice_class = crate::math::voice_class(sel.f0);
            let brightness = sel
                .centroid_hz
                .map(crate::math::brightness_class)
                .unwrap_or("—");
            ui.label(
                RichText::new(format!(
                    "Voice Class: {voice_class}  ·  Brightness: {brightness}  ·  Timbre: {timbre}"
                ))
                .size(11.5)
                .color(ink(140)),
            );
        });
        ui.add_space(14.0);

        let label = if self.export_queued {
            "Report queued"
        } else {
            "Export report (PDF)"
        };
        if pill_button(
            ui,
            vec2(ui.available_width(), 48.0),
            label,
            white(179),
            TEAL_DARK,
            Stroke::new(1.0, teal_a(64)),
        ) {
            self.export_queued = true;
        }
    }
}
