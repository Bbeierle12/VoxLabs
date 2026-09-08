//! The hero vocal-tract card: model update/smoothing, the vowel map, captions, the lazily built inversion grid, and the tract/waveform painters.

use super::*;

/// Lazily built inversion grids, one per basis, built off the UI thread on
/// native (the UI shows CALIBRATING for the first moments of the first
/// session). On wasm — which has no analysis thread and thus never measures
/// formants — the build would run inline if ever requested.
///
/// The grids are the pipeline's shared ones (`pipeline::stages::inverse`):
/// when the runner is up, its `init` has already built them and this
/// returns immediately; otherwise the card starts the build here, once.
pub(super) fn tract_grid_for(
    basis: &'static crate::tract::TractBasis,
) -> Option<&'static crate::tract::TractGrid> {
    use crate::pipeline::stages::inverse::{shared_grid, shared_grid_if_built};
    use std::sync::atomic::{AtomicBool, Ordering};
    static BUILDING_M: AtomicBool = AtomicBool::new(false);
    static BUILDING_F: AtomicBool = AtomicBool::new(false);

    if let Some(g) = shared_grid_if_built(basis) {
        return Some(g);
    }
    let building = if std::ptr::eq(basis, &crate::tract::ADULT_FEMALE) {
        &BUILDING_F
    } else {
        &BUILDING_M
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        if !building.swap(true, Ordering::Relaxed) {
            std::thread::spawn(move || {
                shared_grid(basis, crate::tract::GRID_N);
            });
        }
        None
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = building;
        Some(shared_grid(basis, crate::tract::GRID_N))
    }
}

// ── canvases (ports of the prototype's <canvas> drawing) ────────────────────

/// 3D wireframe of the vocal tract: 44 cross-section rings swept along a
/// bent centerline (pharynx up, bend at the velum, oral cavity forward),
/// rotated around Y and projected with a simple perspective divide. While
/// recording, the area function ripples. Direct port of the JS `drawTract`.
/// The hero mesh: the measured Story area function as a rotating sagittal
/// wireframe. `d` is the 44-section diameter profile (cm) from the live
/// (q1, q2) inversion — the rings ARE the data; the only animation is the
/// slow view rotation. `live` picks the ink: teal when the current frame
/// passed every gate, grey when the shape is held.
pub(super) fn paint_tract(
    painter: &egui::Painter,
    rect: Rect,
    time_ms: f32,
    live: bool,
    d: &[f32],
) {
    const N: usize = crate::tract::N_SECTIONS;
    const M: usize = 18;
    /// Ring radius scale: Story diameters run ~0.3–4.5 cm; 8.5 px/cm keeps
    /// the widest section near the footprint the card was designed around.
    const PX_PER_CM: f32 = 8.5;
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

        // Measured area-function radius: section i's diameter, in pixels.
        // No synthetic ripple — when the model is LIVE the motion comes
        // from the data itself (the q EMA refreshes every gated frame).
        let r = (d[i.min(d.len().saturating_sub(1))] * 0.5 * PX_PER_CM).max(2.0);

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

    // State ink: teal/cyan when the shape is measured-live, neutral grey
    // when held — the same convention as every other gated readout.
    let (long_rgb, ring_rgb, emph_rgb) = if live {
        (
            (8u8, 145u8, 178u8),
            (8u8, 145u8, 178u8),
            (13u8, 148u8, 136u8),
        )
    } else {
        ((11u8, 43u8, 49u8), (11u8, 43u8, 49u8), (11u8, 43u8, 49u8))
    };

    // Longitudinal lines (every 3rd meridian).
    for j in (0..M).step_by(3) {
        let points: Vec<Pos2> = rings.iter().map(|r| pos2(r.pts[j].0, r.pts[j].1)).collect();
        painter.add(Shape::line(
            points,
            Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(long_rgb.0, long_rgb.1, long_rgb.2, 46),
            ),
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
                    emph_rgb.0,
                    emph_rgb.1,
                    emph_rgb.2,
                    (((alpha + 0.22).min(1.0)) * 255.0) as u8,
                ),
                1.5,
            )
        } else {
            (
                Color32::from_rgba_unmultiplied(
                    ring_rgb.0,
                    ring_rgb.1,
                    ring_rgb.2,
                    (alpha * 255.0) as u8,
                ),
                1.0,
            )
        };
        painter.add(Shape::closed_line(points, Stroke::new(width, color)));
    }
}

/// Scrolling bar waveform, teal fading up to cyan at the live (right) edge.
pub(super) fn paint_wave(painter: &egui::Painter, rect: Rect, samples: &[f32]) {
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

impl DashboardApp {
    /// Per-frame tract-model update: refresh the VTL estimate from
    /// identity-grade frames, then invert the current display-grade formants
    /// to (q1, q2). Anything that fails the reliability gate leaves the
    /// shape HELD — visibly grey, never animating on unreliable data.
    pub(super) fn update_tract_model(&mut self, now: f64) {
        self.tract_live = false;
        let p = &self.current_profile;
        let grade = crate::math::formant_grade(&p.formants, p.formants_f0);
        if !p.valid || grade == crate::math::FormantGrade::Reject {
            return;
        }

        // VTL: anatomy accumulates slowly, and only from identity-grade
        // frames — the same rule as the voiceprint, for the same reason.
        if grade == crate::math::FormantGrade::Identity
            && let Some(l) =
                crate::tract::vtl_from_formants(p.formants[1].frequency, p.formants[2].frequency)
        {
            self.vtl_est_cm = Some(match self.vtl_est_cm {
                Some(prev) => prev + VTL_EMA_ALPHA * (l - prev),
                None => l,
            });
        }

        let basis = crate::tract::basis_for_vtl(self.vtl_est_cm);
        let Some(grid) = tract_grid_for(basis) else {
            return; // still building, first frames only
        };
        // Formants scale ~1/L: map the measured pair into the basis's length
        // reference before lookup (uniform-scaling approximation; the
        // oral/pharyngeal cavities actually scale differently — Fant 1966 —
        // which is part of why this is a model fit, not a measurement).
        let scale = self.vtl_est_cm.map_or(1.0, |l| l / basis.vtl_cm);
        let (f1, f2) = (
            p.formants[0].frequency * scale,
            p.formants[1].frequency * scale,
        );
        if let Some((q1, q2)) = grid.invert(f1, f2) {
            let (dq1, dq2) = match self.tract_q {
                Some((a, b)) => (
                    a + TRACT_Q_EMA_ALPHA * (q1 - a),
                    b + TRACT_Q_EMA_ALPHA * (q2 - b),
                ),
                None => (q1, q2),
            };
            self.tract_q = Some((dq1, dq2));
            self.tract_live = true;

            // The articulatory log: one sample whenever a frame clears every
            // gate (voiced, SNR over floor, formant grade, in vowel space),
            // decimated so a session-length log stays small. Nothing that
            // failed a gate is ever logged — the log's value IS the gating.
            if now - self.tract_log_last_t >= TRACT_LOG_MIN_DT {
                self.tract_log_last_t = now;
                if self.tract_log.len() == TRACT_LOG_CAP {
                    self.tract_log.pop_front();
                }
                self.tract_log.push_back((now, dq1, dq2));
            }
        }
    }

    /// 2.5D pseudo-midsagittal tract profile: the model diameter function
    /// D(i) drawn as a ribbon along a quarter-turn centerline (pharynx
    /// vertical, glottis at bottom; oral cavity horizontal, lips at right) —
    /// the same projection Story uses. Teal while LIVE, grey while HELD or
    /// idle; labeled a model throughout, because that is what it is.
    /// The tract model's honest display state: what the hero card may
    /// animate on and what it must admit to. Returns (label, accent, live).
    pub(super) fn tract_state(&self) -> (&'static str, Color32, bool) {
        let basis = crate::tract::basis_for_vtl(self.vtl_est_cm);
        if tract_grid_for(basis).is_none() {
            ("CALIBRATING", ink(115), false)
        } else if self.tract_live {
            ("TRACKING", TEAL, true)
        } else if self.current_profile.metrics.voiced_but_noisy {
            // Periodicity present but the room is too loud to measure it
            // honestly: say so, instead of silently holding.
            ("NOISY", AMBER_TEXT, false)
        } else if self.tract_q.is_some() {
            ("HELD", AMBER_TEXT, false)
        } else {
            ("—", ink(115), false)
        }
    }

    /// Vowel-space log map: the session's articulatory history. Axes are
    /// the model's mode coefficients (q1 →, q2 ↑). Every dot is one
    /// fully-gated moment of the singer's session; the trail fades with
    /// age. Anchors are the published Table II vowels — the model's
    /// landmarks, not measurements of this singer.
    pub(super) fn vowel_map(&self, painter: &egui::Painter, map: Rect) {
        painter.rect(
            map,
            6.0,
            white(120),
            Stroke::new(1.0, ink(28)),
            StrokeKind::Inside,
        );
        let to_map = |q1: f32, q2: f32| -> Pos2 {
            let tx = (q1 - crate::tract::Q1_MIN) / (crate::tract::Q1_MAX - crate::tract::Q1_MIN);
            let ty = (q2 - crate::tract::Q2_MIN) / (crate::tract::Q2_MAX - crate::tract::Q2_MIN);
            pos2(
                map.left() + tx.clamp(0.0, 1.0) * map.width(),
                map.bottom() - ty.clamp(0.0, 1.0) * map.height(),
            )
        };
        // Zero-axes, faint.
        let z = to_map(0.0, 0.0);
        painter.line_segment(
            [pos2(map.left(), z.y), pos2(map.right(), z.y)],
            Stroke::new(0.5, ink(18)),
        );
        painter.line_segment(
            [pos2(z.x, map.top()), pos2(z.x, map.bottom())],
            Stroke::new(0.5, ink(18)),
        );
        for (name, vq1, vq2) in crate::tract::VOWEL_ANCHORS {
            painter.text(
                to_map(vq1, vq2),
                Align2::CENTER_CENTER,
                name,
                FontId::monospace(9.0),
                ink(96),
            );
        }
        let trail_n = self.tract_log.len().min(TRACT_MAP_TRAIL);
        for (age, &(_, lq1, lq2)) in self.tract_log.iter().rev().take(trail_n).enumerate() {
            // Newest opaque, oldest nearly gone.
            let a = (200.0 * (1.0 - age as f32 / trail_n as f32)).max(14.0) as u8;
            painter.circle_filled(to_map(lq1, lq2), 1.6, teal_a(a));
        }
        if self.tract_live
            && let Some((cq1, cq2)) = self.tract_q
        {
            let c = to_map(cq1, cq2);
            painter.circle_filled(c, 3.0, TEAL);
            painter.circle_stroke(c, 4.5, Stroke::new(1.0, teal_a(120)));
        }
    }

    /// Caption for the hero card: what the shape is, what it is scaled to,
    /// and how much history the log holds. Never "your vocal tract" — see
    /// tract_data's honesty boundary.
    pub(super) fn tract_caption(&self, basis: &'static crate::tract::TractBasis) -> String {
        let log_txt = if self.tract_log.is_empty() {
            String::new()
        } else {
            let span_s = self.tract_log.back().map(|b| b.0).unwrap_or(0.0)
                - self.tract_log.front().map(|f| f.0).unwrap_or(0.0);
            format!(
                " · log {} pts / {:.0} min",
                self.tract_log.len(),
                (span_s / 60.0).max(0.0)
            )
        };
        match self.vtl_est_cm {
            Some(l) => format!(
                "model tract fitted to your formants — not an image of your anatomy · \
                 est. length ≈ {l:.1} cm{log_txt}"
            ),
            None => format!(
                "model tract at neutral posture · assumed {:.1} cm{log_txt}",
                basis.vtl_cm
            ),
        }
    }
}
