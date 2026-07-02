//! VoxLab dashboard UI — native egui implementation of the claude.ai/design
//! prototype `VoxLab Prototype.dc.html` (project 4c72096c).
//!
//! Four screens — Overview, Capture, Sessions, Session detail — plus a
//! floating tab bar, in the light "sterile lab" glass style. egui has no
//! backdrop blur, so the glass is approximated with translucent white fills,
//! hairline strokes and soft shadows.
//!
//! Data policy (mirrors the prototype): the live acquisition readouts are
//! REAL — f0/formants from the analysis engine, input level from the audio
//! callback's RMS telemetry — while the session archive, HNR/jitter/shimmer
//! and match scoring are the prototype's mock layer (the engine does not
//! compute speaker embeddings yet).

use crate::concurrency::{EngineEvent, Telemetry};
use crate::types::{Formant, MAX_PARTIALS, Vibrato, VocalProfile};
use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, RichText, Sense, Shape, Stroke,
    StrokeKind, pos2, vec2,
};
use rtrb::Producer;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use triple_buffer::Output;

// ── design tokens ────────────────────────────────────────────────────────────

const INK: Color32 = Color32::from_rgb(11, 43, 49); // #0B2B31
const TEAL: Color32 = Color32::from_rgb(14, 148, 136); // #0E9488
const TEAL_DARK: Color32 = Color32::from_rgb(15, 118, 110); // #0F766E
const CYAN_DEEP: Color32 = Color32::from_rgb(8, 145, 178); // #0891B2
const AMBER_TEXT: Color32 = Color32::from_rgb(180, 83, 9); // #B45309
const AMBER: Color32 = Color32::from_rgb(217, 119, 6); // #D97706
const BG_BASE: Color32 = Color32::from_rgb(234, 243, 245); // between #E3EBEE / #F2F8FA

/// Sessions scoring at or above this are "Verified", below are "Flagged"
/// (the prototype's `matchThreshold` prop, default 85).
const MATCH_THRESHOLD: f32 = 85.0;

/// Content column width — the prototype is a 428 px phone layout.
const COL_WIDTH: f32 = 430.0;

/// Safe-area padding for Android's edge-to-edge rendering (status bar /
/// gesture bar). Zero elsewhere: desktop and web windows are not overlaid.
const TOP_INSET: f32 = if cfg!(target_os = "android") {
    40.0
} else {
    0.0
};
const BOTTOM_INSET: f32 = if cfg!(target_os = "android") {
    18.0
} else {
    0.0
};

fn ink(a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(11, 43, 49, a)
}
fn white(a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, a)
}
fn teal_a(a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(14, 148, 136, a)
}

/// Translucent white "glass" card frame (blur approximated by opacity).
fn glass(corner: f32) -> egui::Frame {
    egui::Frame::default()
        .fill(white(168))
        .stroke(Stroke::new(1.0, white(230)))
        .corner_radius(corner)
        .inner_margin(14.0)
}

// ── model ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Session {
    id: String,
    subj: String,
    date: String,
    f0: f32,
    match_pct: f32,
    /// Measured over the capture (means / stop-time snapshot). Seeded demo
    /// rows carry plausible static values; `None` = not measured.
    hnr_db: Option<f32>,
    h1_h2_db: Option<f32>,
    vibrato: Option<Vibrato>,
    steadiness_cents: Option<f32>,
    /// Real measured formants when the session came from a live capture;
    /// `None` for the seeded demo archive (detail view derives mock values).
    formants: Option<[Formant; 3]>,
}

struct CaptureResult {
    match_pct: f32,
    f0: f32,
    hnr_db: Option<f32>,
    h1_h2_db: Option<f32>,
    vibrato: Option<Vibrato>,
    steadiness_cents: Option<f32>,
    formants: Option<[Formant; 3]>,
}

#[derive(PartialEq, Clone, Copy)]
enum Screen {
    Overview,
    Capture,
    Sessions,
    Detail,
}

#[derive(PartialEq, Clone, Copy)]
enum Filter {
    All,
    Verified,
    Flagged,
}

enum RecState {
    Idle,
    Recording { start: f64 },
    Analyzing { start: f64, elapsed: f64 },
    Done { elapsed: f64 },
}

// ── app ──────────────────────────────────────────────────────────────────────

pub struct DashboardApp {
    /// Reserved: the VoxLab prototype has no engine-control surface yet, so no
    /// events are sent; kept so the entry points' wiring stays unchanged.
    #[allow(dead_code)]
    event_tx: Producer<EngineEvent>,
    telemetry: Arc<Telemetry>,
    ui_profile_rx: Output<VocalProfile>,
    current_profile: VocalProfile,

    screen: Screen,
    filter: Filter,
    rec: RecState,
    sessions: Vec<Session>,
    selected: Option<usize>,
    result: Option<CaptureResult>,
    export_queued: bool,

    /// Waveform bar half-heights (px), scrolling left; length 110.
    wave: Vec<f32>,
    /// EMA-smoothed harmonic amplitudes for the ladder (profiles arrive every
    /// ~46 ms; raw bars at repaint rate read as flicker, not music).
    harm_ema: [f32; MAX_PARTIALS],
    /// Real f0 samples collected while recording (mean shown in the result).
    rec_f0_acc: Vec<f32>,
    /// Real per-frame metrics collected while recording (means stored).
    rec_hnr_acc: Vec<f32>,
    rec_h1h2_acc: Vec<f32>,
    /// Last valid formants seen while recording.
    rec_formants: Option<[Formant; 3]>,
    /// Display-smoothed live readouts (raw values update every ~46 ms and
    /// flicker as digits). `None` = unvoiced/unknown.
    hnr_disp: Option<f32>,
    h1h2_disp: Option<f32>,
    next_session_num: u32,
    rng: u64,
}

impl DashboardApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        event_tx: Producer<EngineEvent>,
        telemetry: Arc<Telemetry>,
        ui_profile_rx: Output<VocalProfile>,
    ) -> Self {
        let mut visuals = egui::Visuals::light();
        visuals.override_text_color = Some(INK);
        visuals.panel_fill = BG_BASE;
        visuals.window_fill = BG_BASE;
        cc.egui_ctx.set_visuals(visuals);

        // Seeded demo archive: plausible static values, marked apart from live
        // captures only by `formants: None`.
        #[allow(clippy::too_many_arguments)]
        let seed = |id: &str,
                    subj: &str,
                    date: &str,
                    f0: f32,
                    m: f32,
                    hnr: f32,
                    h1h2: f32,
                    vib: Option<(f32, f32)>,
                    steady: f32| Session {
            id: id.into(),
            subj: subj.into(),
            date: date.into(),
            f0,
            match_pct: m,
            hnr_db: Some(hnr),
            h1_h2_db: Some(h1h2),
            vibrato: vib.map(|(rate_hz, extent_cents)| Vibrato {
                rate_hz,
                extent_cents,
            }),
            steadiness_cents: Some(steady),
            formants: None,
        };

        Self {
            event_tx,
            telemetry,
            ui_profile_rx,
            current_profile: VocalProfile::default(),
            screen: Screen::Overview,
            filter: Filter::All,
            rec: RecState::Idle,
            sessions: vec![
                seed(
                    "VS-2481",
                    "S-0417",
                    "Jul 1 · 09:12",
                    118.4,
                    96.2,
                    21.7,
                    4.2,
                    Some((5.6, 42.0)),
                    9.0,
                ),
                seed(
                    "VS-2480",
                    "S-0392",
                    "Jun 30 · 16:40",
                    214.9,
                    91.8,
                    19.3,
                    6.8,
                    Some((5.1, 55.0)),
                    12.0,
                ),
                seed(
                    "VS-2479",
                    "S-0417",
                    "Jun 29 · 11:05",
                    121.2,
                    94.5,
                    20.4,
                    3.9,
                    Some((5.7, 38.0)),
                    8.0,
                ),
                seed(
                    "VS-2478",
                    "S-0105",
                    "Jun 28 · 14:22",
                    187.5,
                    62.3,
                    16.1,
                    12.5,
                    None,
                    28.0,
                ),
                seed(
                    "VS-2477",
                    "S-0233",
                    "Jun 27 · 10:48",
                    132.8,
                    88.1,
                    18.9,
                    7.4,
                    Some((4.8, 61.0)),
                    14.0,
                ),
            ],
            selected: None,
            result: None,
            export_queued: false,
            wave: vec![2.0; 110],
            harm_ema: [0.0; MAX_PARTIALS],
            rec_f0_acc: Vec::new(),
            rec_hnr_acc: Vec::new(),
            rec_h1h2_acc: Vec::new(),
            rec_formants: None,
            hnr_disp: None,
            h1h2_disp: None,
            next_session_num: 2482,
            rng: 0x9E37_79B9_7F4A_7C15,
        }
    }

    fn rand01(&mut self) -> f32 {
        // xorshift64 — no rng dependency, prototype-grade randomness.
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u64 << 24) as f32
    }

    fn verified(match_pct: f32) -> bool {
        match_pct >= MATCH_THRESHOLD
    }

    fn badge_style(match_pct: f32) -> (&'static str, Color32, Color32) {
        if Self::verified(match_pct) {
            ("Verified", teal_a(31), TEAL_DARK)
        } else {
            (
                "Flagged",
                Color32::from_rgba_unmultiplied(217, 119, 6, 36),
                AMBER_TEXT,
            )
        }
    }

    fn input_rms(&self) -> f32 {
        f32::from_bits(self.telemetry.input_rms.load(Ordering::Relaxed))
    }

    // ── state machine ────────────────────────────────────────────────────────

    fn advance(&mut self, now: f64) {
        if let RecState::Analyzing { start, elapsed } = self.rec
            && now - start >= 2.1
        {
            // Real f0 mean when we heard voiced frames; otherwise the
            // prototype's synthetic fallback. Match/HNR are mock — the
            // engine has no speaker-verification scoring yet.
            let f0 = if self.rec_f0_acc.is_empty() {
                114.0 + self.rand01() * 10.0
            } else {
                self.rec_f0_acc.iter().sum::<f32>() / self.rec_f0_acc.len() as f32
            };
            let match_pct = ((92.0 + self.rand01() * 6.0) * 10.0).round() / 10.0;
            let mean =
                |acc: &[f32]| (!acc.is_empty()).then(|| acc.iter().sum::<f32>() / acc.len() as f32);
            // Vibrato/steadiness are contour-level: snapshot the stop-time
            // values rather than averaging per-frame reports.
            let m = self.current_profile.metrics;
            self.result = Some(CaptureResult {
                match_pct,
                f0: (f0 * 10.0).round() / 10.0,
                hnr_db: mean(&self.rec_hnr_acc),
                h1_h2_db: mean(&self.rec_h1h2_acc),
                vibrato: m.vibrato,
                steadiness_cents: m.steadiness_cents,
                formants: self.rec_formants,
            });
            self.rec = RecState::Done { elapsed };
        }
    }

    fn start_rec(&mut self, now: f64) {
        self.rec = RecState::Recording { start: now };
        self.rec_f0_acc.clear();
        self.rec_hnr_acc.clear();
        self.rec_h1h2_acc.clear();
        self.rec_formants = None;
        self.result = None;
    }

    fn stop_rec(&mut self, now: f64) {
        if let RecState::Recording { start } = self.rec {
            self.rec = RecState::Analyzing {
                start: now,
                elapsed: now - start,
            };
        }
    }

    fn save_session(&mut self) {
        if let Some(res) = self.result.take() {
            let session = Session {
                id: format!("VS-{}", self.next_session_num),
                subj: "S-0417".into(),
                date: "Today".into(),
                f0: res.f0,
                match_pct: res.match_pct,
                hnr_db: res.hnr_db,
                h1_h2_db: res.h1_h2_db,
                vibrato: res.vibrato,
                steadiness_cents: res.steadiness_cents,
                formants: res.formants,
            };
            self.next_session_num += 1;
            self.sessions.insert(0, session);
            self.rec = RecState::Idle;
            self.filter = Filter::All;
            self.screen = Screen::Sessions;
        }
    }

    /// One waveform sample per frame: recording follows the live input RMS,
    /// idle decays flat (same smoothing constants as the prototype).
    fn push_wave_sample(&mut self) {
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

// ── top level ────────────────────────────────────────────────────────────────

impl eframe::App for DashboardApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Continuous repaint: canvases + live readouts animate every frame.
        ui.ctx().request_repaint();
        let now = ui.input(|i| i.time);

        self.advance(now);

        if self.ui_profile_rx.updated() {
            self.current_profile = *self.ui_profile_rx.read();
        }
        if matches!(self.rec, RecState::Recording { .. }) && self.current_profile.valid {
            self.rec_f0_acc.push(self.current_profile.f0);
            self.rec_formants = Some(self.current_profile.formants);
            if let Some(h) = self.current_profile.metrics.hnr_db {
                self.rec_hnr_acc.push(h);
            }
            if let Some(h) = self.current_profile.metrics.h1_h2_db {
                self.rec_h1h2_acc.push(h);
            }
        }
        for (ema, &a) in self
            .harm_ema
            .iter_mut()
            .zip(&self.current_profile.partial_amplitudes)
        {
            *ema += 0.3 * (a - *ema);
        }
        // Display smoothing for the digit readouts; reset when the value goes
        // away so stale numbers never linger.
        let smooth = |disp: &mut Option<f32>, v: Option<f32>| match (disp.as_mut(), v) {
            (Some(d), Some(v)) => *d += 0.2 * (v - *d),
            _ => *disp = v,
        };
        smooth(&mut self.hnr_disp, self.current_profile.metrics.hnr_db);
        smooth(&mut self.h1h2_disp, self.current_profile.metrics.h1_h2_db);
        self.push_wave_sample();

        let full = ui.max_rect();
        paint_background(ui.painter(), full);

        // Centered phone-width content column.
        let col_w = full.width().min(COL_WIDTH);
        let col_rect = Rect::from_min_max(
            pos2(full.center().x - col_w / 2.0, full.top()),
            pos2(full.center().x + col_w / 2.0, full.bottom()),
        );
        let mut col_ui = ui.new_child(egui::UiBuilder::new().max_rect(col_rect));
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(&mut col_ui, |ui| {
                egui::Frame::default().inner_margin(18.0).show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                    // Android renders edge-to-edge under the system bars and
                    // eframe exposes no safe-area insets, so pad past them.
                    ui.add_space(TOP_INSET);
                    match self.screen {
                        Screen::Overview => self.screen_overview(ui),
                        Screen::Capture => self.screen_capture(ui, now),
                        Screen::Sessions => self.screen_sessions(ui),
                        Screen::Detail => self.screen_detail(ui),
                    }
                    // Clearance for the floating tab bar.
                    ui.add_space(104.0 + BOTTOM_INSET);
                });
            });

        self.tab_bar(ui.ctx().clone(), now);
    }
}

// ── background ───────────────────────────────────────────────────────────────

fn paint_background(painter: &egui::Painter, rect: Rect) {
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

// ── shared widgets ───────────────────────────────────────────────────────────

impl DashboardApp {
    fn screen_kicker(&self, ui: &mut egui::Ui, kicker: &str, title: &str) {
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

fn pill_badge(ui: &mut egui::Ui, text: &str, bg: Color32, fg: Color32) {
    let font = FontId::proportional(12.0);
    let galley = ui.painter().layout_no_wrap(text.into(), font.clone(), fg);
    let size = vec2(galley.size().x + 22.0, 24.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, 12.0, bg);
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, text, font, fg);
}

/// Rounded pill button; returns true on click.
fn pill_button(
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
fn ring_gauge(
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

fn sparkline(painter: &egui::Painter, rect: Rect, pts: &[(f32, f32)]) {
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
fn waveform_glyph(painter: &egui::Painter, rect: Rect, color: Color32) {
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
fn mic_glyph(painter: &egui::Painter, rect: Rect, color: Color32, filled_body: bool) {
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

// ── screens ──────────────────────────────────────────────────────────────────

impl DashboardApp {
    fn screen_overview(&mut self, ui: &mut egui::Ui) {
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

        // Active-subject hero card.
        let hero = self
            .sessions
            .iter()
            .find(|s| s.subj == "S-0417")
            .unwrap_or(&self.sessions[0])
            .clone();
        let (badge, badge_bg, badge_fg) = Self::badge_style(hero.match_pct);
        glass(24.0).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(96.0, 96.0), Sense::hover());
                ring_gauge(
                    ui.painter(),
                    rect,
                    hero.match_pct,
                    TEAL,
                    &format!("{:.1}", hero.match_pct),
                    Some("MATCH %"),
                );
                ui.add_space(14.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("Active subject").size(12.0).color(ink(140)));
                    ui.add_space(3.0);
                    ui.label(
                        RichText::new("S-0417")
                            .font(FontId::monospace(20.0))
                            .color(INK)
                            .strong(),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("Enrolled · 12 sessions · M, 34")
                            .size(12.0)
                            .color(ink(140)),
                    );
                    ui.add_space(9.0);
                    pill_badge(ui, badge, badge_bg, badge_fg);
                });
            });
        });
        ui.add_space(12.0);

        // Metric tiles (2 × 2) — prototype archive metrics.
        struct Tile {
            label: &'static str,
            value: &'static str,
            unit: &'static str,
            reference: &'static str,
            points: [(f32, f32); 8],
        }
        const TILES: [Tile; 4] = [
            Tile {
                label: "F0 MEAN",
                value: "118.4",
                unit: "Hz",
                reference: "+1.2 vs enrollment",
                points: [
                    (0.0, 16.0),
                    (14.0, 13.0),
                    (28.0, 15.0),
                    (42.0, 10.0),
                    (56.0, 12.0),
                    (70.0, 7.0),
                    (84.0, 9.0),
                    (100.0, 6.0),
                ],
            },
            Tile {
                label: "JITTER",
                value: "0.42",
                unit: "%",
                reference: "ref < 1.04 %",
                points: [
                    (0.0, 8.0),
                    (14.0, 11.0),
                    (28.0, 9.0),
                    (42.0, 13.0),
                    (56.0, 12.0),
                    (70.0, 15.0),
                    (84.0, 14.0),
                    (100.0, 16.0),
                ],
            },
            Tile {
                label: "SHIMMER",
                value: "0.28",
                unit: "dB",
                reference: "ref < 0.35 dB",
                points: [
                    (0.0, 12.0),
                    (14.0, 10.0),
                    (28.0, 13.0),
                    (42.0, 11.0),
                    (56.0, 14.0),
                    (70.0, 12.0),
                    (84.0, 15.0),
                    (100.0, 13.0),
                ],
            },
            Tile {
                label: "HNR",
                value: "21.7",
                unit: "dB",
                reference: "ref > 17 dB",
                points: [
                    (0.0, 18.0),
                    (14.0, 15.0),
                    (28.0, 16.0),
                    (42.0, 12.0),
                    (56.0, 13.0),
                    (70.0, 9.0),
                    (84.0, 10.0),
                    (100.0, 6.0),
                ],
            },
        ];
        for row in TILES.chunks(2) {
            ui.columns(2, |cols| {
                for (col, tile) in cols.iter_mut().zip(row) {
                    let (label, value, unit, reference, pts) = (
                        &tile.label,
                        &tile.value,
                        &tile.unit,
                        &tile.reference,
                        &tile.points,
                    );
                    glass(20.0).show(col, |ui| {
                        ui.label(
                            RichText::new(*label)
                                .font(FontId::monospace(10.0))
                                .color(ink(128)),
                        );
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(*value)
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
    fn session_row_compact(&self, ui: &mut egui::Ui, s: &Session, i: usize) -> bool {
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
                    let text = format!("{:.1}", s.match_pct);
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

    fn screen_capture(&mut self, ui: &mut egui::Ui, now: f64) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.screen_kicker(ui, "LIVE ACQUISITION", "Capture");
            });
            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                let font = FontId::monospace(12.5);
                let galley = ui
                    .painter()
                    .layout_no_wrap("S-0417".into(), font.clone(), INK);
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
                    "S-0417",
                    font,
                    INK,
                );
            });
        });
        ui.add_space(16.0);

        let recording = matches!(self.rec, RecState::Recording { .. });

        // Vocal tract card.
        glass(24.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Vocal tract · articulatory model")
                        .size(13.0)
                        .color(INK)
                        .strong(),
                );
                if recording {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let blink = if (now * 1.0).fract() < 0.5 { 255 } else { 64 };
                        ui.label(
                            RichText::new("LIVE")
                                .font(FontId::monospace(10.0))
                                .color(CYAN_DEEP),
                        );
                        let (dot, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                        ui.painter().circle_filled(
                            dot.center(),
                            3.0,
                            Color32::from_rgba_unmultiplied(6, 182, 212, blink),
                        );
                    });
                }
            });
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 236.0), Sense::hover());
            paint_tract(ui.painter(), rect, (now * 1000.0) as f32, recording);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("SAGITTAL MESH · 44 SECTIONS")
                        .font(FontId::monospace(9.5))
                        .color(ink(107)),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("AREA FN · MRI FIT")
                            .font(FontId::monospace(9.5))
                            .color(ink(107)),
                    );
                });
            });
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
                ui.label(
                    RichText::new("EXTRACTING EMBEDDING · SCORING VS ENROLLMENT S-0417")
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
            let (badge, badge_bg, badge_fg) = Self::badge_style(res.match_pct);
            let match_str = format!("{:.1}", res.match_pct);
            let mut save = false;
            let mut discard = false;
            glass(24.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Similarity vs voiceprint S-0417")
                            .size(13.0)
                            .color(ink(140)),
                    );
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
                ui.add_space(14.0);
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
                    if recording {
                        self.stop_rec(now);
                    } else {
                        self.start_rec(now);
                    }
                }

                ui.add_space(12.0);
                let hint = if recording {
                    "Recording — tap to stop"
                } else {
                    "Tap to begin capture · sustained /a/ · 8 s min"
                };
                ui.label(
                    RichText::new(hint)
                        .font(FontId::monospace(11.0))
                        .color(ink(128)),
                );
            });
        }
    }

    /// Live harmonic-series card: per-partial ladder (dB bars, note names,
    /// cents) plus the timbre metrics strip (tilt, even/odd, singer's formant).
    fn harmonics_card(&self, ui: &mut egui::Ui) {
        const ROWS: usize = 16;
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

            // Ladder: dB relative to the strongest partial, floored at −48 dB.
            let max_amp = self
                .harm_ema
                .iter()
                .take(ROWS)
                .cloned()
                .fold(0.0f32, f32::max);
            let audible = max_amp > 1e-5;
            let label_font = FontId::monospace(10.0);
            for k in 0..ROWS {
                let db = if audible && self.harm_ema[k] > 1e-6 {
                    (20.0 * (self.harm_ema[k] / max_amp).log10()).max(-48.0)
                } else {
                    -48.0
                };
                let frac = 1.0 + db / 48.0;

                ui.horizontal(|ui| {
                    let (hr, _) = ui.allocate_exact_size(vec2(28.0, 16.0), Sense::hover());
                    ui.painter().text(
                        hr.left_center(),
                        Align2::LEFT_CENTER,
                        format!("H{}", k + 1),
                        label_font.clone(),
                        ink(115),
                    );

                    let note = if valid {
                        crate::math::freq_to_note((k + 1) as f32 * f0)
                            .map(|n| format!("{}{} {:+.0}¢", n.name, n.octave, n.cents))
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    let (nr, _) = ui.allocate_exact_size(vec2(76.0, 16.0), Sense::hover());
                    ui.painter().text(
                        nr.left_center(),
                        Align2::LEFT_CENTER,
                        note,
                        label_font.clone(),
                        ink(166),
                    );

                    let db_w = 46.0;
                    let bar_w = (ui.available_width() - db_w).max(20.0);
                    let (br, _) = ui.allocate_exact_size(vec2(bar_w, 16.0), Sense::hover());
                    let track =
                        Rect::from_min_size(pos2(br.left(), br.center().y - 4.0), vec2(bar_w, 8.0));
                    ui.painter().rect_filled(track, 4.0, ink(10));
                    if frac > 0.02 {
                        // Same teal→cyan ramp as the waveform, low → high partials.
                        let t = k as f32 / (ROWS - 1) as f32;
                        let color = Color32::from_rgb(
                            (14.0 + t * 20.0) as u8,
                            (148.0 + t * 63.0) as u8,
                            (136.0 + t * 102.0) as u8,
                        );
                        let fill = Rect::from_min_size(
                            track.min,
                            vec2(track.width() * frac.clamp(0.0, 1.0), 8.0),
                        );
                        ui.painter().rect_filled(fill, 4.0, color);
                    }

                    let (dr, _) = ui.allocate_exact_size(vec2(db_w, 16.0), Sense::hover());
                    let db_str = if audible && db > -48.0 {
                        format!("{db:+.0} dB")
                    } else {
                        "—".into()
                    };
                    ui.painter().text(
                        dr.right_center(),
                        Align2::RIGHT_CENTER,
                        db_str,
                        FontId::monospace(9.5),
                        ink(128),
                    );
                });
                ui.add_space(2.0);
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

    fn screen_sessions(&mut self, ui: &mut egui::Ui) {
        self.screen_kicker(ui, "CAPTURE ARCHIVE", "Sessions");
        ui.add_space(16.0);

        // Filter chips.
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            for (key, label) in [
                (Filter::All, "All"),
                (Filter::Verified, "Verified"),
                (Filter::Flagged, "Flagged"),
            ] {
                let active = self.filter == key;
                let (fill, fg, stroke) = if active {
                    (TEAL, Color32::WHITE, Stroke::NONE)
                } else {
                    (white(158), ink(158), Stroke::new(1.0, white(230)))
                };
                let font = FontId::proportional(13.0);
                let galley = ui.painter().layout_no_wrap(label.into(), font.clone(), fg);
                let size = vec2(galley.size().x + 32.0, 33.0);
                let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
                let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                ui.painter()
                    .rect(rect, size.y / 2.0, fill, stroke, StrokeKind::Inside);
                ui.painter()
                    .text(rect.center(), Align2::CENTER_CENTER, label, font, fg);
                if resp.clicked() {
                    self.filter = key;
                }
            }
        });
        ui.add_space(16.0);

        // Column headers.
        let header_font = FontId::monospace(9.5);
        ui.horizontal(|ui| {
            ui.add_space(15.0);
            for (w, label) in [(82.0, "ID"), (0.0, "SUBJECT"), (62.0, "F0 HZ")] {
                let text = RichText::new(label)
                    .font(header_font.clone())
                    .color(ink(115));
                let w = if w > 0.0 {
                    w
                } else {
                    ui.available_width() - 62.0 - 68.0 - 31.0
                };
                ui.allocate_ui_with_layout(
                    vec2(w, 14.0),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.set_width(w);
                        ui.label(text);
                    },
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(15.0);
                ui.label(
                    RichText::new("MATCH")
                        .font(header_font.clone())
                        .color(ink(115)),
                );
            });
        });
        ui.add_space(8.0);

        // Rows.
        let rows: Vec<(usize, Session)> = self
            .sessions
            .iter()
            .enumerate()
            .filter(|(_, s)| match self.filter {
                Filter::All => true,
                Filter::Verified => Self::verified(s.match_pct),
                Filter::Flagged => !Self::verified(s.match_pct),
            })
            .map(|(i, s)| (i, s.clone()))
            .collect();

        for (i, s) in rows {
            let (_, badge_bg, badge_fg) = Self::badge_style(s.match_pct);
            let ir = glass(18.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        vec2(78.0, 34.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_width(78.0);
                            ui.label(
                                RichText::new(&s.id)
                                    .font(FontId::monospace(12.5))
                                    .color(INK)
                                    .strong(),
                            );
                        },
                    );
                    let subj_w = ui.available_width() - 62.0 - 68.0 - 16.0;
                    ui.allocate_ui_with_layout(
                        vec2(subj_w, 34.0),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.set_width(subj_w);
                            ui.label(RichText::new(&s.subj).size(13.0).color(INK).strong());
                            ui.label(RichText::new(&s.date).size(10.5).color(ink(115)));
                        },
                    );
                    ui.allocate_ui_with_layout(
                        vec2(58.0, 34.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_width(58.0);
                            ui.label(
                                RichText::new(format!("{:.0}", s.f0))
                                    .font(FontId::monospace(12.5))
                                    .color(ink(179)),
                            );
                        },
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let text = format!("{:.1}", s.match_pct);
                        let font = FontId::monospace(11.5);
                        let galley =
                            ui.painter()
                                .layout_no_wrap(text.clone(), font.clone(), badge_fg);
                        let size = vec2(galley.size().x + 18.0, 23.0);
                        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                        ui.painter().rect_filled(rect, 11.5, badge_bg);
                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            text,
                            font,
                            badge_fg,
                        );
                    });
                });
            });
            let clicked = ui
                .interact(
                    ir.response.rect,
                    ui.id().with("session").with(i),
                    Sense::click(),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked();
            if clicked {
                self.selected = Some(i);
                self.export_queued = false;
                self.screen = Screen::Detail;
            }
            ui.add_space(8.0);
        }
    }

    fn screen_detail(&mut self, ui: &mut egui::Ui) {
        let sel = self
            .selected
            .and_then(|i| self.sessions.get(i))
            .unwrap_or(&self.sessions[0])
            .clone();
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
                    sel.match_pct,
                    ring_color,
                    &format!("{:.1}", sel.match_pct),
                    None,
                );
                ui.add_space(14.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Verification score")
                            .size(12.0)
                            .color(ink(140)),
                    );
                    ui.add_space(3.0);
                    ui.label(
                        RichText::new("vs enrolled voiceprint")
                            .size(15.0)
                            .color(INK)
                            .strong(),
                    );
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
        let params: [(String, String, &str, Color32); 5] = [
            (
                "F0 mean".into(),
                format!("{:.1} Hz", sel.f0),
                "85–255 Hz",
                dot((85.0..=255.0).contains(&sel.f0)),
            ),
            (
                "HNR".into(),
                opt_val(sel.hnr_db, "dB", 1),
                "> 17 dB",
                opt_dot(sel.hnr_db, |v| v > 17.0),
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

        // Formant chips: real measurements when the session captured them,
        // otherwise the prototype's f0-derived mock values. F4 is always
        // derived (the engine extracts three formants).
        let derived = [
            420.0 + sel.f0 * 0.8,
            1390.0 + sel.f0 * 0.8,
            2450.0 + sel.f0 * 0.9,
        ];
        let mut chips: Vec<(String, f32)> = match sel.formants {
            Some(f) => f
                .iter()
                .enumerate()
                .map(|(i, f)| (format!("F{}", i + 1), f.frequency))
                .collect(),
            None => derived
                .iter()
                .enumerate()
                .map(|(i, f)| (format!("F{}", i + 1), *f))
                .collect(),
        };
        chips.push(("F4".into(), 3500.0 + sel.f0 * 0.95));

        glass(24.0).show(ui, |ui| {
            ui.label(
                RichText::new("Acoustic parameters")
                    .size(14.0)
                    .color(INK)
                    .strong(),
            );
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
                        format!("{:.0}", freq),
                        FontId::monospace(13.0),
                        TEAL_DARK,
                    );
                    col.allocate_exact_size(vec2(rect.width(), 52.0), Sense::hover());
                }
            });
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

    // ── floating tab bar ─────────────────────────────────────────────────────

    fn tab_bar(&mut self, ctx: egui::Context, _now: f64) {
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
                                    _ => list_glyph(ui.painter(), icon, fg),
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
                                }
                            }
                        });
                    });
            });
    }
}

fn grid_glyph(painter: &egui::Painter, rect: Rect, color: Color32) {
    let s = rect.width() / 20.0;
    let o = rect.min;
    let stroke = Stroke::new(1.7 * s, color);
    for (x, y) in [(2.5, 2.5), (11.5, 2.5), (2.5, 11.5), (11.5, 11.5)] {
        let r = Rect::from_min_size(pos2(o.x + x * s, o.y + y * s), vec2(6.0 * s, 6.0 * s));
        painter.rect(r, 2.0 * s, Color32::TRANSPARENT, stroke, StrokeKind::Middle);
    }
}

fn list_glyph(painter: &egui::Painter, rect: Rect, color: Color32) {
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

// ── canvases (ports of the prototype's <canvas> drawing) ────────────────────

/// 3D wireframe of the vocal tract: 44 cross-section rings swept along a
/// bent centerline (pharynx up, bend at the velum, oral cavity forward),
/// rotated around Y and projected with a simple perspective divide. While
/// recording, the area function ripples. Direct port of the JS `drawTract`.
fn paint_tract(painter: &egui::Painter, rect: Rect, time_ms: f32, rec: bool) {
    const N: usize = 44;
    const M: usize = 18;
    let ang = 0.55 + time_ms * 0.000_32;
    let (cos_a, sin_a) = (ang.cos(), ang.sin());
    let (w, h) = (rect.width(), rect.height());
    let cx = rect.left() + w / 2.0 - 6.0;
    let cy = rect.top() + h / 2.0;
    let (f, s_scale) = (300.0f32, 1.32f32);

    struct Ring {
        pts: Vec<(f32, f32, f32)>, // x, y, depth
        z_avg: f32,
        s: f32,
    }
    let mut rings: Vec<Ring> = Vec::with_capacity(N);

    for i in 0..N {
        let s = i as f32 / (N - 1) as f32;
        let (px, py, tx, ty);
        if s < 0.45 {
            let u = s / 0.45;
            px = 0.0;
            py = u * 90.0;
            tx = 0.0;
            ty = 1.0;
        } else if s < 0.62 {
            let u = (s - 0.45) / 0.17;
            let a = std::f32::consts::PI - u * std::f32::consts::FRAC_PI_2;
            px = 38.0 + 38.0 * a.cos();
            py = 52.0 + 38.0 * a.sin();
            tx = a.sin();
            ty = -a.cos();
        } else {
            let u = (s - 0.62) / 0.38;
            px = 38.0 + u * 74.0;
            py = 90.0;
            tx = 1.0;
            ty = 0.0;
        }

        // Area-function radius, rippling while recording.
        let mut r = 10.5 + 7.0 * (s * std::f32::consts::PI).sin() + 3.0 * (s * 6.3 + 1.7).sin();
        if s > 0.9 {
            r *= 1.0 - (s - 0.9) * 3.2;
        }
        if s < 0.06 {
            r *= 0.55 + s * 7.0;
        }
        if rec {
            r += 2.2 * (time_ms * 0.012 + s * 9.0).sin() + 1.2 * (time_ms * 0.027 + s * 17.0).sin();
        }
        r = r.max(4.0);

        let (nx, ny) = (-ty, tx);
        let (mx, my) = (px - 40.0, py - 46.0);
        let mut pts = Vec::with_capacity(M);
        let mut z_sum = 0.0;
        for j in 0..M {
            let th = j as f32 / M as f32 * std::f32::consts::TAU;
            let x3 = mx + r * th.cos() * nx;
            let y3 = my + r * th.cos() * ny;
            let z3 = r * th.sin();
            let xr = x3 * cos_a + z3 * sin_a;
            let zr = -x3 * sin_a + z3 * cos_a;
            let k = f / (f + zr);
            pts.push((cx + xr * k * s_scale, cy - y3 * k * s_scale, zr));
            z_sum += zr;
        }
        rings.push(Ring {
            pts,
            z_avg: z_sum / M as f32,
            s,
        });
    }

    // Longitudinal lines (every 3rd meridian).
    for j in (0..M).step_by(3) {
        let points: Vec<Pos2> = rings.iter().map(|r| pos2(r.pts[j].0, r.pts[j].1)).collect();
        painter.add(Shape::line(
            points,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(8, 145, 178, 46)),
        ));
    }

    // Rings painted far → near, alpha by depth; every 6th emphasized.
    let mut order: Vec<usize> = (0..N).collect();
    order.sort_by(|&a, &b| {
        rings[b]
            .z_avg
            .partial_cmp(&rings[a].z_avg)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for idx in order {
        let ring = &rings[idx];
        let depth = ((ring.z_avg + 30.0) / 60.0).clamp(0.0, 1.0);
        let alpha = 0.10 + depth * 0.42;
        let emph = ((ring.s * (N - 1) as f32).round() as usize).is_multiple_of(6);
        let points: Vec<Pos2> = ring.pts.iter().map(|p| pos2(p.0, p.1)).collect();
        let (color, width) = if emph {
            (
                Color32::from_rgba_unmultiplied(
                    13,
                    148,
                    136,
                    (((alpha + 0.22).min(1.0)) * 255.0) as u8,
                ),
                1.5,
            )
        } else {
            (
                Color32::from_rgba_unmultiplied(8, 145, 178, (alpha * 255.0) as u8),
                1.0,
            )
        };
        painter.add(Shape::closed_line(points, Stroke::new(width, color)));
    }
}

/// Scrolling bar waveform, teal fading up to cyan at the live (right) edge.
fn paint_wave(painter: &egui::Painter, rect: Rect, samples: &[f32]) {
    let n = samples.len().max(1);
    let step = rect.width() / n as f32;
    let mid = rect.center().y;
    for (i, &a) in samples.iter().enumerate() {
        let x = rect.left() + i as f32 * step;
        let t = i as f32 / n as f32;
        // teal 25% → teal 90% → cyan, matching the canvas gradient stops.
        let color = if t < 0.75 {
            let a8 = (64.0 + (t / 0.75) * 166.0) as u8;
            Color32::from_rgba_unmultiplied(14, 148, 136, a8)
        } else {
            let u = (t - 0.75) / 0.25;
            Color32::from_rgb(
                (14.0 + u * 20.0) as u8,
                (148.0 + u * 63.0) as u8,
                (136.0 + u * 102.0) as u8,
            )
        };
        let bar = Rect::from_min_max(
            pos2(x + step * 0.18, mid - a),
            pos2(x + step * 0.82, mid + a),
        );
        painter.rect_filled(bar, 2.0, color);
    }
}
