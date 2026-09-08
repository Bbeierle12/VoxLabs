//! Design tokens (colors, layout metrics) and the shared glass widgets every screen draws with.

use super::*;

// ── design tokens ────────────────────────────────────────────────────────────

pub(super) const INK: Color32 = Color32::from_rgb(11, 43, 49); // #0B2B31
pub(super) const TEAL: Color32 = Color32::from_rgb(14, 148, 136); // #0E9488
pub(super) const TEAL_DARK: Color32 = Color32::from_rgb(15, 118, 110); // #0F766E
pub(super) const CYAN_DEEP: Color32 = Color32::from_rgb(8, 145, 178); // #0891B2
pub(super) const AMBER_TEXT: Color32 = Color32::from_rgb(180, 83, 9); // #B45309
pub(super) const AMBER: Color32 = Color32::from_rgb(217, 119, 6); // #D97706
pub(super) const BG_BASE: Color32 = Color32::from_rgb(234, 243, 245); // between #E3EBEE / #F2F8FA

/// Content column width — the prototype is a 428 px phone layout.
pub(super) const COL_WIDTH: f32 = 430.0;

/// Content inset of a `glass` card: its 1 px stroke plus 14 px inner margin.
/// The Sessions header row is *not* inside a card, so it pads by this much on
/// each side to sit on the same grid as the rows below it.
pub(super) const GLASS_INSET: f32 = 15.0;

/// Safe-area padding for Android's edge-to-edge rendering (status bar /
/// gesture bar). Zero elsewhere: desktop and web windows are not overlaid.
pub(super) const TOP_INSET: f32 = if cfg!(target_os = "android") {
    40.0
} else {
    0.0
};
pub(super) const BOTTOM_INSET: f32 = if cfg!(target_os = "android") {
    18.0
} else {
    0.0
};

pub(super) fn ink(a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(11, 43, 49, a)
}
pub(super) fn white(a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, a)
}
pub(super) fn teal_a(a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(14, 148, 136, a)
}

/// Small teal chip button (the CALIBRATE pattern, as a helper); returns
/// whether it was clicked. Drawn by the device-probe card (Android) and
/// the capture-export toggle (everywhere).
pub(super) fn probe_chip(ui: &mut egui::Ui, label: &str) -> bool {
    let font = FontId::monospace(9.5);
    let galley = ui
        .painter()
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
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, label, font, TEAL_DARK);
    resp.clicked()
}

/// Translucent white "glass" card frame (blur approximated by opacity).
pub(super) fn glass(corner: f32) -> egui::Frame {
    egui::Frame::default()
        .fill(white(168))
        .stroke(Stroke::new(1.0, white(230)))
        .corner_radius(corner)
        .inner_margin(14.0)
}

/// One amber status line in a glass card, shown above whichever screen is up.
/// Returns true when a dismissible banner's ✕ was clicked this frame.
pub(super) fn notice_banner(ui: &mut egui::Ui, text: &str, dismissible: bool) -> bool {
    let mut dismissed = false;
    glass(16.0).show(ui, |ui| {
        ui.horizontal(|ui| {
            let close_w = if dismissible { 26.0 } else { 0.0 };
            ui.allocate_ui_with_layout(
                vec2(ui.available_width() - close_w, 0.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.add(
                        egui::Label::new(RichText::new(text).size(12.0).color(AMBER_TEXT)).wrap(),
                    );
                },
            );
            if dismissible {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let resp = ui
                        .add(
                            egui::Label::new(RichText::new("✕").size(13.0).color(ink(140)))
                                .sense(Sense::click()),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    dismissed = resp.clicked();
                });
            }
        });
    });
    dismissed
}

// ── background ───────────────────────────────────────────────────────────────

pub(super) fn paint_background(painter: &egui::Painter, rect: Rect) {
    painter.rect_filled(rect, 0.0, BG_BASE);
    // Soft radial washes (layered translucent circles stand in for gradients).
    let tr = pos2(rect.right() + 60.0, rect.top() - 60.0);
    let bl = pos2(rect.left() - 80.0, rect.bottom() - 120.0);
    for (i, r) in [150.0f32, 110.0, 70.0].iter().enumerate() {
        let a = 8 + i as u8 * 6;
        painter.circle_filled(
            tr,
            *r * 2.0,
            Color32::from_rgba_unmultiplied(34, 211, 238, a),
        );
        painter.circle_filled(
            bl,
            *r * 2.3,
            Color32::from_rgba_unmultiplied(14, 148, 136, a / 2),
        );
    }
}

pub(super) fn pill_badge(ui: &mut egui::Ui, text: &str, bg: Color32, fg: Color32) {
    let font = FontId::proportional(12.0);
    let galley = ui.painter().layout_no_wrap(text.into(), font.clone(), fg);
    let size = vec2(galley.size().x + 22.0, 24.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, 12.0, bg);
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, text, font, fg);
}

/// Rounded pill button; returns true on click.
pub(super) fn pill_button(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    text: &str,
    fill: Color32,
    fg: Color32,
    stroke: Stroke,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
    let fill = if resp.is_pointer_button_down_on() {
        fill.linear_multiply(0.9)
    } else {
        fill
    };
    ui.painter()
        .rect(rect, size.y / 2.0, fill, stroke, StrokeKind::Inside);
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        FontId::proportional(14.5),
        fg,
    );
    resp.clicked()
}

/// Circular progress ring ("match %" gauge). `pct` in 0..=100.
pub(super) fn ring_gauge(
    painter: &egui::Painter,
    rect: Rect,
    pct: f32,
    color: Color32,
    value: &str,
    sub: Option<&str>,
) {
    let c = rect.center();
    let r = rect.width().min(rect.height()) * 0.4375; // 42/96 of the viewbox
    let w = rect.width() / 12.0; // stroke 8/96
    painter.circle_stroke(c, r, Stroke::new(w, teal_a(31)));

    let frac = (pct / 100.0).clamp(0.0, 1.0);
    if frac > 0.005 {
        let start = -std::f32::consts::FRAC_PI_2;
        let sweep = frac * std::f32::consts::TAU;
        let n = 64;
        let points: Vec<Pos2> = (0..=n)
            .map(|i| {
                let a = start + sweep * i as f32 / n as f32;
                pos2(c.x + r * a.cos(), c.y + r * a.sin())
            })
            .collect();
        // Round caps faked with end dots.
        let (first, last) = (points[0], points[n]);
        painter.add(Shape::line(points, Stroke::new(w, color)));
        painter.circle_filled(first, w / 2.0, color);
        painter.circle_filled(last, w / 2.0, color);
    }

    if let Some(sub) = sub {
        painter.text(
            pos2(c.x, c.y - 3.0),
            Align2::CENTER_BOTTOM,
            value,
            FontId::monospace(rect.width() * 0.2),
            INK,
        );
        painter.text(
            pos2(c.x, c.y + 4.0),
            Align2::CENTER_TOP,
            sub,
            FontId::proportional(rect.width() * 0.1),
            ink(128),
        );
    } else {
        painter.text(
            c,
            Align2::CENTER_CENTER,
            value,
            FontId::monospace(rect.width() * 0.2),
            INK,
        );
    }
}

pub(super) fn sparkline(painter: &egui::Painter, rect: Rect, pts: &[(f32, f32)]) {
    let points: Vec<Pos2> = pts
        .iter()
        .map(|(x, y)| {
            pos2(
                rect.left() + x / 100.0 * rect.width(),
                rect.top() + y / 24.0 * rect.height(),
            )
        })
        .collect();
    painter.add(Shape::line(points, Stroke::new(1.8, teal_a(204))));
}

/// The prototype's capture-row glyph: a tiny symmetric waveform.
pub(super) fn waveform_glyph(painter: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 16.0;
    let o = rect.min;
    let seg = |x1: f32, y1: f32, x2: f32, y2: f32| {
        [
            pos2(o.x + x1 * s, o.y + y1 * s),
            pos2(o.x + x2 * s, o.y + y2 * s),
        ]
    };
    let stroke = Stroke::new(1.6, color);
    painter.line_segment(seg(2.0, 8.0, 3.5, 8.0), stroke);
    painter.line_segment(seg(5.0, 5.0, 5.0, 11.0), stroke);
    painter.line_segment(seg(8.0, 2.5, 8.0, 13.5), stroke);
    painter.line_segment(seg(11.0, 5.0, 11.0, 11.0), stroke);
    painter.line_segment(seg(14.0, 8.0, 12.5, 8.0), stroke);
}

/// Microphone glyph scaled into `rect` (design geometry is a 24 px viewbox).
pub(super) fn mic_glyph(painter: &egui::Painter, rect: Rect, color: Color32, filled_body: bool) {
    let s = rect.width() / 24.0;
    let o = rect.min;
    let p = |x: f32, y: f32| pos2(o.x + x * s, o.y + y * s);
    let body = Rect::from_min_max(p(9.0, 3.0), p(15.0, 14.0));
    if filled_body {
        painter.rect_filled(body, 3.0 * s, color);
    } else {
        painter.rect(
            body,
            3.0 * s,
            Color32::TRANSPARENT,
            Stroke::new(1.8 * s, color),
            StrokeKind::Middle,
        );
    }
    // Lower semicircle from (5.5,11) to (18.5,11) through (12,17.5).
    let c = p(12.0, 11.0);
    let r = 6.5 * s;
    let arc: Vec<Pos2> = (0..=24)
        .map(|i| {
            let a = std::f32::consts::PI * i as f32 / 24.0;
            pos2(c.x + r * a.cos(), c.y + r * a.sin())
        })
        .collect();
    painter.add(Shape::line(arc, Stroke::new(1.9 * s, color)));
    painter.line_segment([p(12.0, 17.5), p(12.0, 21.0)], Stroke::new(1.9 * s, color));
}

impl DashboardApp {
    pub(super) fn screen_kicker(&self, ui: &mut egui::Ui, kicker: &str, title: &str) {
        ui.label(
            RichText::new(kicker)
                .font(FontId::proportional(11.0))
                .color(TEAL)
                .strong(),
        );
        ui.add_space(3.0);
        ui.label(
            RichText::new(title)
                .font(FontId::proportional(30.0))
                .color(INK)
                .strong(),
        );
    }
}
