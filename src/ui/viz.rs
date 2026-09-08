//! Analyzer visualizations: the spectrogram waterfall (log-frequency row map, viridis LUT), the oscilloscope, the harmonic ladder, and the radial view.

use super::*;

// ── spectrogram waterfall ─────────────────────────────────────────────────────

/// Which visualization fills the switchable region *below* the always-present
/// harmonic ladder.
#[derive(PartialEq, Clone, Copy)]
pub(super) enum VizMode {
    Radial,
    Spectrogram,
    Scope,
}

/// Waterfall grid: time on X (newest at right), log-frequency on Y.
pub(super) const SPEC_TIME_COLS: usize = 320;
pub(super) const SPEC_FREQ_ROWS: usize = 200;
pub(super) const SPEC_FMIN_HZ: f32 = 50.0; // matches YIN_F0_MIN
pub(super) const SPEC_FMAX_HZ: f32 = 5000.0; // matches the formant search band ceiling
pub(super) const SPEC_DB_FLOOR: f32 = -90.0;
pub(super) const SPEC_DB_CEIL: f32 = -10.0;

/// How one waterfall display-row samples the FFT magnitude bins. When the row
/// spans ≥ 1 bin, take the max (peaks survive); when narrower than a bin,
/// linearly interpolate. Ported from Resonator's `ColMap`.
#[derive(Clone, Copy)]
pub(super) struct BinSpan {
    lo: usize,
    hi: usize,
    frac: f32,
}

impl BinSpan {
    pub(super) fn sample(&self, bins: &[f32]) -> f32 {
        if self.hi > self.lo {
            bins[self.lo..=self.hi]
                .iter()
                .cloned()
                .fold(f32::NEG_INFINITY, f32::max)
        } else {
            let a = bins[self.lo];
            let b = bins[(self.lo + 1).min(bins.len().saturating_sub(1))];
            a + (b - a) * self.frac
        }
    }
}

pub(super) fn spec_db_to_byte(db: f32) -> u8 {
    let v = ((db - SPEC_DB_FLOOR) / (SPEC_DB_CEIL - SPEC_DB_FLOOR)).clamp(0.0, 1.0);
    (v * 255.0) as u8
}

/// Viridis 6th-degree polynomial colormap fit (Matt Zucker, shadertoy WlfXRN)
/// — a dependency-free perceptual ramp, ported from Resonator. Reads well on
/// the light glass background (dark-purple low end contrasts cleanly).
#[rustfmt::skip]
pub(super) const VIRIDIS: [[f32; 3]; 7] = [
    [0.27772733, 0.0054073445, 0.334_099_8],
    [0.10509304, 1.4046135, 1.3845902],
    [-0.33086183, 0.21484756, 0.095095163],
    [-4.6342305, -5.799_101, -19.332441],
    [6.228_27, 14.179933, 56.690_55],
    [4.776_385, -13.745145, -65.353033],
    [-5.435_456, 4.6458526, 26.312435],
];

pub(super) fn build_heat_lut() -> [Color32; 256] {
    let mut lut = [Color32::BLACK; 256];
    for (v, slot) in lut.iter_mut().enumerate() {
        let t = v as f32 / 255.0;
        let mut c = VIRIDIS[6];
        for coef in VIRIDIS.iter().take(6).rev() {
            c = [coef[0] + t * c[0], coef[1] + t * c[1], coef[2] + t * c[2]];
        }
        *slot = Color32::from_rgb(
            (c[0].clamp(0.0, 1.0) * 255.0) as u8,
            (c[1].clamp(0.0, 1.0) * 255.0) as u8,
            (c[2].clamp(0.0, 1.0) * 255.0) as u8,
        );
    }
    lut
}

/// Build the row→bin map for the log-frequency Y axis: row 0 is the top
/// (SPEC_FMAX_HZ), the last row is the bottom (SPEC_FMIN_HZ).
pub(super) fn build_freq_row_map(sample_rate: f32) -> Vec<BinSpan> {
    let n = crate::spectrogram::N_BINS;
    let nyquist = sample_rate * 0.5;
    let to_bin = |hz: f32| hz * (n - 1) as f32 / nyquist.max(1.0);
    let ratio = SPEC_FMAX_HZ / SPEC_FMIN_HZ;
    let rows = SPEC_FREQ_ROWS;
    let half_step = ratio.powf(0.5 / (rows - 1).max(1) as f32);
    (0..rows)
        .map(|r| {
            let t = r as f32 / (rows - 1).max(1) as f32; // 0 top .. 1 bottom
            let fc = SPEC_FMIN_HZ * ratio.powf(1.0 - t); // high at top, low at bottom
            let bc = to_bin(fc);
            let blo = to_bin(fc / half_step);
            let bhi = to_bin(fc * half_step);
            if bhi - blo >= 1.0 {
                let lo = (blo.floor().max(0.0) as usize).min(n - 1);
                let hi = (bhi.ceil() as usize).min(n - 1);
                BinSpan { lo, hi, frac: 0.0 }
            } else {
                let lo = (bc.floor().max(0.0) as usize).min(n.saturating_sub(2));
                BinSpan {
                    lo,
                    hi: lo,
                    frac: (bc - lo as f32).clamp(0.0, 1.0),
                }
            }
        })
        .collect()
}

/// Harmonic-amplitude ladder: H1..H16 as horizontal bars in dB relative to the
/// strongest partial (floored at −48 dB), with note + cents per harmonic. This
/// is the default (`VizMode::Ladder`) view; the code is unchanged from when it
/// lived inline in `harmonics_card`.
pub(super) fn harmonic_ladder(ui: &mut egui::Ui, harm_ema: &[f32], valid: bool, f0: f32) {
    const ROWS: usize = 16;
    let max_amp = harm_ema.iter().take(ROWS).cloned().fold(0.0f32, f32::max);
    let audible = max_amp > 1e-5;
    let label_font = FontId::monospace(10.0);
    for k in 0..ROWS {
        let amp = harm_ema.get(k).copied().unwrap_or(0.0);
        let db = if audible && amp > 1e-6 {
            (20.0 * (amp / max_amp).log10()).max(-48.0)
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
            let track = Rect::from_min_size(pos2(br.left(), br.center().y - 4.0), vec2(bar_w, 8.0));
            ui.painter().rect_filled(track, 4.0, ink(10));
            if frac > 0.02 {
                let t = k as f32 / (ROWS - 1) as f32;
                let color = Color32::from_rgb(
                    (14.0 + t * 20.0) as u8,
                    (148.0 + t * 63.0) as u8,
                    (136.0 + t * 102.0) as u8,
                );
                let fill =
                    Rect::from_min_size(track.min, vec2(track.width() * frac.clamp(0.0, 1.0), 8.0));
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
}

/// Radial harmonic signature: 16 spokes at `2π·i/16 − π/2` (12 o'clock start),
/// radius = amplitude relative to the strongest partial. A closed teal polygon
/// (triangle-fan filled so a concave signature renders correctly) with labeled
/// nodes; idle "—" when unvoiced.
pub(super) fn radial_view(ui: &mut egui::Ui, harm_ema: &[f32], valid: bool) {
    use std::f32::consts::{FRAC_PI_2, TAU};
    const N: usize = 16;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 240.0), Sense::hover());
    let c = rect.center();
    let radius = rect.height() * 0.40;

    for r in 1..=4 {
        ui.painter()
            .circle_stroke(c, radius * r as f32 / 4.0, Stroke::new(1.0, ink(12)));
    }
    let spoke = |i: usize| i as f32 / N as f32 * TAU - FRAC_PI_2;
    for i in 0..N {
        let a = spoke(i);
        ui.painter().line_segment(
            [c, pos2(c.x + a.cos() * radius, c.y + a.sin() * radius)],
            Stroke::new(1.0, ink(10)),
        );
    }

    let max_amp = harm_ema.iter().take(N).cloned().fold(0.0f32, f32::max);
    if !valid || max_amp <= 1e-5 {
        ui.painter().text(
            c,
            Align2::CENTER_CENTER,
            "—",
            FontId::monospace(20.0),
            ink(90),
        );
        return;
    }

    let pts: Vec<Pos2> = (0..N)
        .map(|i| {
            let a = spoke(i);
            let rel = (harm_ema[i] / max_amp).clamp(0.02, 1.0);
            pos2(c.x + a.cos() * radius * rel, c.y + a.sin() * radius * rel)
        })
        .collect();
    for i in 0..N {
        let j = (i + 1) % N;
        ui.painter().add(Shape::convex_polygon(
            vec![c, pts[i], pts[j]],
            teal_a(28),
            Stroke::NONE,
        ));
    }
    ui.painter()
        .add(Shape::closed_line(pts.clone(), Stroke::new(1.8, TEAL)));
    for (i, p) in pts.iter().enumerate() {
        let (col, r) = if i == 0 { (AMBER, 3.5) } else { (TEAL, 2.2) };
        ui.painter().circle_filled(*p, r, col);
    }
    for i in 0..N {
        let a = spoke(i);
        let lr = radius + 12.0;
        ui.painter().text(
            pos2(c.x + a.cos() * lr, c.y + a.sin() * lr),
            Align2::CENTER_CENTER,
            format!("H{}", i + 1),
            FontId::monospace(8.5),
            ink(120),
        );
    }
}

impl DashboardApp {
    /// Segmented control selecting which visualization fills the switchable
    /// region: `Ladder | Radial | Spectro | Scope`.
    pub(super) fn viz_mode_switch(&mut self, ui: &mut egui::Ui) {
        let modes = [
            (VizMode::Radial, "Radial"),
            (VizMode::Spectrogram, "Spectro"),
            (VizMode::Scope, "Scope"),
        ];
        let gap = 4.0;
        let seg_w = (ui.available_width() - gap * (modes.len() - 1) as f32) / modes.len() as f32;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (mode, label) in modes {
                let active = self.viz_mode == mode;
                let (rect, resp) = ui.allocate_exact_size(vec2(seg_w, 26.0), Sense::click());
                let resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                let (fill, fg, stroke) = if active {
                    (teal_a(31), TEAL_DARK, Stroke::new(1.0, teal_a(64)))
                } else {
                    (white(120), ink(120), Stroke::new(1.0, ink(20)))
                };
                ui.painter()
                    .rect(rect, 7.0, fill, stroke, StrokeKind::Inside);
                ui.painter().text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    label,
                    FontId::monospace(10.5),
                    fg,
                );
                if resp.clicked() {
                    self.viz_mode = mode;
                }
            }
        });
    }

    /// Scrolling spectrogram waterfall: time on X (newest at right),
    /// log-frequency (50–5000 Hz) on Y, Viridis intensity. Ingests one new
    /// column per repaint from the latest published magnitude frame, then
    /// composes and uploads the RGBA texture. Renderer ported from Resonator's
    /// `draw_waterfall`.
    pub(super) fn spectrogram_view(&mut self, ui: &mut egui::Ui) {
        // Pull the latest magnitude frame into the reused scratch buffer.
        {
            let mags = self.spectrum_rx.read();
            self.wf_scratch.clear();
            self.wf_scratch.extend_from_slice(mags);
        }
        // Advance the ring head and rewrite that column along the log-freq axis.
        self.wf_head = (self.wf_head + 1) % SPEC_TIME_COLS;
        {
            let Self {
                wf_cols,
                wf_freq_map,
                wf_scratch,
                wf_head,
                ..
            } = self;
            let col = &mut wf_cols[*wf_head];
            for (cell, span) in col.iter_mut().zip(wf_freq_map.iter()) {
                *cell = spec_db_to_byte(span.sample(wf_scratch));
            }
        }
        // Compose RGBA: display x = time (oldest left, newest right), y = freq.
        let (cols, rows) = (SPEC_TIME_COLS, SPEC_FREQ_ROWS);
        {
            let Self {
                wf_pixels,
                wf_cols,
                wf_lut,
                wf_head,
                ..
            } = self;
            for x in 0..cols {
                let ring_idx = (*wf_head + 1 + x) % cols; // x = cols-1 → newest
                let column = &wf_cols[ring_idx];
                for (y, &byte) in column.iter().enumerate() {
                    let c = wf_lut[byte as usize];
                    let p = (y * cols + x) * 4;
                    wf_pixels[p] = c.r();
                    wf_pixels[p + 1] = c.g();
                    wf_pixels[p + 2] = c.b();
                    wf_pixels[p + 3] = 255;
                }
            }
        }
        let image = ColorImage::from_rgba_unmultiplied([cols, rows], &self.wf_pixels);
        let tex = self.wf_tex.get_or_insert_with(|| {
            ui.ctx()
                .load_texture("voxlab_waterfall", image.clone(), TextureOptions::LINEAR)
        });
        tex.set(image, TextureOptions::LINEAR);

        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 210.0), Sense::hover());
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        ui.painter().rect(
            rect,
            6.0,
            Color32::TRANSPARENT,
            Stroke::new(1.0, ink(30)),
            StrokeKind::Inside,
        );
        // Log-frequency axis labels (white reads over the dark low-magnitude field).
        for (hz, lbl) in [
            (100.0, "100"),
            (300.0, "300"),
            (1000.0, "1k"),
            (3000.0, "3k"),
        ] {
            let t = (SPEC_FMAX_HZ / hz).ln() / (SPEC_FMAX_HZ / SPEC_FMIN_HZ).ln();
            let y = rect.top() + t * rect.height();
            ui.painter().text(
                pos2(rect.left() + 4.0, y),
                Align2::LEFT_CENTER,
                lbl,
                FontId::monospace(8.5),
                white(205),
            );
        }
    }

    /// Oscilloscope: zero-crossing-triggered time-domain trace of the most
    /// recent raw frame, ~3 periods wide when voiced.
    pub(super) fn scope_view(&mut self, ui: &mut egui::Ui, valid: bool, f0: f32) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 130.0), Sense::hover());
        ui.painter().rect(
            rect,
            6.0,
            ink(6),
            Stroke::new(1.0, ink(20)),
            StrokeKind::Inside,
        );
        ui.painter().line_segment(
            [
                pos2(rect.left(), rect.center().y),
                pos2(rect.right(), rect.center().y),
            ],
            Stroke::new(1.0, ink(18)),
        );

        let sr = self.sample_rate;
        let buf = self.scope_rx.read();
        let n = buf.len();
        if n < 8 {
            return;
        }
        // Trigger on the first rising zero-crossing in the first half.
        let mut start = 0;
        for i in 1..n / 2 {
            if buf[i - 1] <= 0.0 && buf[i] > 0.0 {
                start = i;
                break;
            }
        }
        // ~3.2 periods when voiced, else a fixed window.
        let span = if valid && f0 > 0.0 {
            ((sr / f0 * 3.2) as usize).clamp(64, n - start)
        } else {
            1024.min(n - start)
        };
        if span < 2 {
            return;
        }
        let pts: Vec<Pos2> = (0..span)
            .map(|i| {
                let x = rect.left() + (i as f32 / span as f32) * rect.width();
                let y = rect.center().y - buf[start + i] * rect.height() * 0.46;
                pos2(x, y)
            })
            .collect();
        ui.painter().add(Shape::line(pts, Stroke::new(1.5, TEAL)));
    }
}
