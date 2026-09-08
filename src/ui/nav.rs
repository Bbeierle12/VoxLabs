//! The floating tab bar and its glyphs.

use super::*;

pub(super) fn grid_glyph(painter: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 20.0;
    let o = rect.min;
    let stroke = Stroke::new(1.7 * s, color);
    for (x, y) in [(2.5, 2.5), (11.5, 2.5), (2.5, 11.5), (11.5, 11.5)] {
        let r = Rect::from_min_size(pos2(o.x + x * s, o.y + y * s), vec2(6.0 * s, 6.0 * s));
        painter.rect(r, 2.0 * s, Color32::TRANSPARENT, stroke, StrokeKind::Middle);
    }
}

/// Room tab glyph: a small level-meter — three bars of rising height.
pub(super) fn room_glyph(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() / 5.0;
    for (i, h_frac) in [0.45f32, 0.75, 1.0].iter().enumerate() {
        let x = rect.left() + w * (0.5 + i as f32 * 1.6);
        let h = rect.height() * h_frac;
        painter.rect_filled(
            Rect::from_min_size(pos2(x, rect.bottom() - h), vec2(w, h)),
            1.5,
            color,
        );
    }
}

pub(super) fn list_glyph(painter: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 20.0;
    let o = rect.min;
    let stroke = Stroke::new(1.7 * s, color);
    let seg = |x1: f32, y1: f32, x2: f32| {
        [
            pos2(o.x + x1 * s, o.y + y1 * s),
            pos2(o.x + x2 * s, o.y + y1 * s),
        ]
    };
    painter.line_segment(seg(3.0, 5.0, 17.0), stroke);
    painter.line_segment(seg(3.0, 10.0, 17.0), stroke);
    painter.line_segment(seg(3.0, 15.0, 12.0), stroke);
}

impl DashboardApp {
    // ── floating tab bar ─────────────────────────────────────────────────────

    pub(super) fn tab_bar(&mut self, ctx: egui::Context, now: f64) {
        egui::Area::new(egui::Id::new("voxlab_tab_bar"))
            .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -16.0 - BOTTOM_INSET))
            .order(egui::Order::Foreground)
            .show(&ctx, |ui| {
                egui::Frame::default()
                    .fill(white(200))
                    .stroke(Stroke::new(1.0, white(235)))
                    .corner_radius(28.0)
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            for (screen, label) in [
                                (Screen::Overview, "Overview"),
                                (Screen::Capture, "Capture"),
                                (Screen::Room, "Room"),
                                (Screen::Sessions, "Sessions"),
                            ] {
                                let active = self.screen == screen
                                    || (screen == Screen::Sessions
                                        && self.screen == Screen::Detail);
                                let fg = if active { TEAL } else { ink(107) };
                                let (rect, resp) =
                                    ui.allocate_exact_size(vec2(86.0, 44.0), Sense::click());
                                let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                                if active {
                                    ui.painter().rect_filled(rect, 22.0, teal_a(26));
                                }
                                let icon = Rect::from_center_size(
                                    pos2(rect.center().x, rect.top() + 15.0),
                                    vec2(20.0, 20.0),
                                );
                                match screen {
                                    Screen::Overview => grid_glyph(ui.painter(), icon, fg),
                                    Screen::Capture => mic_glyph(ui.painter(), icon, fg, false),
                                    Screen::Room => room_glyph(ui.painter(), icon, fg),
                                    _ => list_glyph(ui.painter(), icon, fg),
                                }
                                // Live-capture dot: recording must stay visible
                                // from every screen, not just Capture.
                                if screen == Screen::Capture
                                    && matches!(self.rec, RecState::Recording { .. })
                                {
                                    let pulse = (0.55 + 0.45 * (now * 4.0).sin().abs()) as f32;
                                    ui.painter().circle_filled(
                                        pos2(icon.right() + 3.0, icon.top() + 1.0),
                                        3.5,
                                        Color32::from_rgba_unmultiplied(
                                            239,
                                            68,
                                            68,
                                            (pulse * 255.0) as u8,
                                        ),
                                    );
                                }
                                ui.painter().text(
                                    pos2(rect.center().x, rect.bottom() - 9.0),
                                    Align2::CENTER_CENTER,
                                    label,
                                    FontId::proportional(10.5),
                                    fg,
                                );
                                if resp.clicked() {
                                    self.screen = screen;
                                    // Leaving Overview disarms a half-confirmed
                                    // re-enroll.
                                    self.reenroll_armed = false;
                                }
                            }
                        });
                    });
            });
    }
}
