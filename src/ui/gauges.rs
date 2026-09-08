//! Capture-screen gauges: the cents needle, the turnover (A2/A1) gauge, and the harmonics card.

use super::*;

impl DashboardApp {
    /// Live harmonic-series card: per-partial ladder (dB bars, note names,
    /// cents) plus the timbre metrics strip (tilt, even/odd, singer's formant).
    /// Bozeman's "turning over" gauge, driven by the A2/A1 harmonic ratio —
    /// his own observable for the event (Journal of Singing, 2010): H2
    /// dominant (open timbre, *voce aperta*) on one side, the dominant-
    /// harmonic switch at 0 dB, H1 dominant (turned over, *voce chiusa*) on
    /// the other. The gauge shows sign and crossing only — no target value:
    /// whether singers deliberately tune the crossing is disputed (Sundberg,
    /// Lã & Gill 2013 found no systematic tuning in male opera singers), so
    /// nothing here is presented as a number to hold.
    /// Cents tuning needle: ±50¢ around the current note, glowing teal within
    /// ±5¢ ("in tune"), amber otherwise. Ported from the Personal Harmonic
    /// Identifier prototype's needle meter — previously cents were text-only.
    pub(super) fn cents_needle_gauge(&self, ui: &mut egui::Ui) {
        const RANGE: f32 = 50.0;
        let valid = self.current_profile.valid && self.current_profile.f0 > 0.0;
        let cents = if valid {
            crate::math::freq_to_note(self.current_profile.f0).map(|n| n.cents)
        } else {
            None
        };
        let in_tune = cents.is_some_and(|c| c.abs() <= 5.0);
        let color = if in_tune { TEAL } else { AMBER };

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("TUNING")
                    .font(FontId::monospace(9.5))
                    .color(ink(115)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let text = match cents {
                    Some(c) if in_tune => format!("IN TUNE  {c:+.0}¢"),
                    Some(c) => format!("{c:+.0}¢"),
                    None => "—".into(),
                };
                ui.label(
                    RichText::new(text)
                        .font(FontId::monospace(10.5))
                        .color(color)
                        .strong(),
                );
            });
        });
        ui.add_space(5.0);

        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 14.0), Sense::hover());
        let track = Rect::from_min_size(
            pos2(rect.left(), rect.center().y - 3.0),
            vec2(rect.width(), 6.0),
        );
        let to_x = |c: f32| -> f32 {
            let t = ((c + RANGE) / (2.0 * RANGE)).clamp(0.0, 1.0);
            track.left() + t * track.width()
        };
        ui.painter().rect_filled(track, 3.0, ink(10));
        let cx = to_x(0.0);
        ui.painter().line_segment(
            [pos2(cx, track.top() - 3.0), pos2(cx, track.bottom() + 3.0)],
            Stroke::new(1.5, ink(90)),
        );
        if let Some(c) = cents {
            let mx = to_x(c.clamp(-RANGE, RANGE));
            let my = track.center().y;
            ui.painter().circle_filled(pos2(mx, my), 5.0, color);
            ui.painter()
                .circle_stroke(pos2(mx, my), 5.0, Stroke::new(1.0, white(230)));
        }
    }

    pub(super) fn turning_over_gauge(&self, ui: &mut egui::Ui) {
        const RANGE: f32 = 12.0; // displayed span, ± dB of A2/A1
        // Half-width of the amber "turning" band. Display convention, not a
        // measurement claim: ±3 dB (half power) keeps the needle from
        // flickering between states around the dominant-harmonic switch.
        const TURN_ZONE: f32 = 3.0;

        // Positive A2/A1 = H2 dominant = open timbre. The axis keeps the
        // established left-open / right-closed reading, so the needle plots
        // the *negated* value.
        let a2a1 = self.turnover_disp;
        let (state, color) = match a2a1 {
            Some(v) if v > TURN_ZONE => ("OPEN", TEAL),
            Some(v) if v < -TURN_ZONE => ("CLOSED", CYAN_DEEP),
            Some(_) => ("TURNING", AMBER),
            None => ("—", ink(115)),
        };

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("A2 / A1 · PASSAGGIO")
                    .font(FontId::monospace(9.5))
                    .color(ink(115)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let numeric = a2a1
                    .map(|v| format!("{v:+.1} dB"))
                    .unwrap_or_else(|| "—".into());
                ui.label(
                    RichText::new(format!("{state}  {numeric}"))
                        .font(FontId::monospace(10.5))
                        .color(color)
                        .strong(),
                );
            });
        });
        ui.add_space(5.0);

        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 18.0), Sense::hover());
        let track = Rect::from_min_size(
            pos2(rect.left(), rect.center().y - 4.0),
            vec2(rect.width(), 8.0),
        );
        let to_x = |st: f32| -> f32 {
            let t = ((st + RANGE) / (2.0 * RANGE)).clamp(0.0, 1.0);
            track.left() + t * track.width()
        };

        ui.painter().rect_filled(track, 4.0, ink(10));
        // Amber "turning" zone band, centered on the crossing.
        let zone = Rect::from_min_max(
            pos2(to_x(-TURN_ZONE), track.top()),
            pos2(to_x(TURN_ZONE), track.bottom()),
        );
        ui.painter()
            .rect_filled(zone, 4.0, Color32::from_rgba_unmultiplied(217, 119, 6, 46));
        // Crossing tick at 0 dB — the dominant-harmonic switch.
        let cx = to_x(0.0);
        ui.painter().line_segment(
            [pos2(cx, track.top() - 3.0), pos2(cx, track.bottom() + 3.0)],
            Stroke::new(1.5, ink(90)),
        );

        if let Some(v) = a2a1 {
            let mx = to_x((-v).clamp(-RANGE, RANGE));
            let my = track.center().y;
            ui.painter().circle_filled(pos2(mx, my), 6.0, color);
            ui.painter()
                .circle_stroke(pos2(mx, my), 6.0, Stroke::new(1.0, white(230)));
        }
    }

    pub(super) fn harmonics_card(&mut self, ui: &mut egui::Ui) {
        let valid = self.current_profile.valid && self.current_profile.f0 > 0.0;
        let f0 = self.current_profile.f0;

        glass(24.0).show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            // Header: title + live fundamental note readout.
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Harmonic series")
                        .size(13.0)
                        .color(INK)
                        .strong(),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut readout = if valid {
                        crate::math::freq_to_note(f0)
                            .map(|n| format!("{}{} · {:+.0}¢", n.name, n.octave, n.cents))
                            .unwrap_or_else(|| "—".into())
                    } else {
                        "—".into()
                    };
                    if let Some(v) = self.current_profile.metrics.vibrato {
                        readout += &format!(" · vib {:.1} Hz ±{:.0}¢", v.rate_hz, v.extent_cents);
                    }
                    ui.label(
                        RichText::new(readout)
                            .font(FontId::monospace(13.0))
                            .color(TEAL_DARK)
                            .strong(),
                    );
                });
            });
            ui.add_space(10.0);

            self.cents_needle_gauge(ui);
            ui.add_space(12.0);

            self.turning_over_gauge(ui);
            ui.add_space(12.0);

            // The harmonic ladder is always shown.
            harmonic_ladder(ui, &self.harm_ema, valid, f0);
            ui.add_space(12.0);

            // Secondary switchable visualization below the ladder: radial
            // signature, spectrogram waterfall, or oscilloscope.
            self.viz_mode_switch(ui);
            ui.add_space(10.0);
            match self.viz_mode {
                VizMode::Radial => radial_view(ui, &self.harm_ema, valid),
                VizMode::Spectrogram => self.spectrogram_view(ui),
                VizMode::Scope => self.scope_view(ui, valid, f0),
            }

            // Metrics strip, same pattern as the F0/LEVEL/ELAPSED readouts.
            ui.add_space(9.0);
            let sep = ui.available_rect_before_wrap();
            ui.painter().line_segment(
                [pos2(sep.left(), sep.top()), pos2(sep.right(), sep.top())],
                Stroke::new(1.0, ink(18)),
            );
            ui.add_space(11.0);

            let (tilt, balance, formant_pct) = if valid {
                (
                    crate::math::spectral_tilt_db_per_octave(&self.harm_ema),
                    crate::math::even_odd_balance_db(&self.harm_ema),
                    crate::math::singers_formant_pct(&self.harm_ema, f0),
                )
            } else {
                (None, None, None)
            };
            let fmt = |v: Option<f32>, unit: &str| {
                v.map(|v| format!("{v:+.1} {unit}"))
                    .unwrap_or_else(|| "—".into())
            };
            let steadiness = self.current_profile.metrics.steadiness_cents;
            let rows = [
                [
                    ("TILT", fmt(tilt, "dB/oct")),
                    ("EVEN/ODD", fmt(balance, "dB")),
                    (
                        "SINGER'S FMT",
                        formant_pct
                            .map(|p| format!("{p:.0} %"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                ],
                [
                    ("H1–H2", fmt(self.h1h2_disp, "dB")),
                    (
                        "HNR",
                        self.hnr_disp
                            .map(|v| format!("{v:.0} dB"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                    (
                        "STEADINESS",
                        steadiness
                            .map(|s| format!("{s:.0} ¢"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                ],
                [
                    (
                        "JITTER",
                        self.jitter_disp
                            .map(|v| format!("{v:.2} %"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                    (
                        "SHIMMER",
                        self.shimmer_disp
                            .map(|v| format!("{v:.2} dB"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                    (
                        "CPP",
                        self.cpp_disp
                            .map(|v| format!("{v:.0} dB"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                ],
                [
                    (
                        "CENTROID",
                        self.centroid_disp
                            .map(|v| format!("{v:.0} Hz"))
                            .unwrap_or_else(|| "—".into()),
                    ),
                    (
                        "VOICE CLASS",
                        if valid {
                            crate::math::voice_class(f0).to_string()
                        } else {
                            "—".into()
                        },
                    ),
                    (
                        "BRIGHTNESS",
                        self.centroid_disp
                            .map(crate::math::brightness_class)
                            .unwrap_or("—")
                            .to_string(),
                    ),
                ],
            ];
            for (i, row) in rows.into_iter().enumerate() {
                if i > 0 {
                    ui.add_space(10.0);
                }
                ui.columns(3, |cols| {
                    for (col, (label, value)) in cols.iter_mut().zip(row) {
                        col.label(
                            RichText::new(label)
                                .font(FontId::monospace(9.5))
                                .color(ink(115)),
                        );
                        col.add_space(3.0);
                        col.label(
                            RichText::new(value)
                                .font(FontId::monospace(13.0))
                                .color(INK)
                                .strong(),
                        );
                    }
                });
            }
        });
    }
}
