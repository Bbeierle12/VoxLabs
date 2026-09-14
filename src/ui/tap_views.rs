//! Tap renderers for the modes beyond the Live Model (Plan v3 Phase 4/5a):
//! Coral's rehearsal view from the `multi_f0` + `satb` taps through the
//! harmony consumer, and the atlas mesh from the `TractGeometry` tap.
//! Both read taps the pipeline panel already keeps; nothing here computes.

use super::*;
use crate::choir::cards::HarmonyResult;
use crate::choir::harmony::{Band, HarmonyConfig};
use crate::choir::labeler::Section;
use crate::config::ChoirHarmonyConfig;
use crate::pipeline::types::TractGeometry;

/// Coral's four fixed S/A/T/B cards, the chord line and the pair table.
pub(super) fn rehearsal_card(ui: &mut egui::Ui, r: &HarmonyResult, cfg: &HarmonyConfig) {
    let hc = ChoirHarmonyConfig::DEFAULT;
    ui.label(
        RichText::new("REHEARSAL (Coral)")
            .font(FontId::monospace(9.5))
            .color(ink(115)),
    );
    let chord = match r.id {
        Some(id) if id.cluster => "Cluster / Polyphony".to_string(),
        Some(id) => format!(
            "{} {} ({})",
            crate::choir::harmony::NOTE_NAMES[id.root],
            id.chord.name,
            id.chord.ratio_text()
        ),
        None => "— no chord —".into(),
    };
    ui.label(
        RichText::new(format!(
            "{chord} · consonance {} · drift {} · {} · {}",
            r.cons
                .map(|c| format!("{c}%"))
                .unwrap_or_else(|| "—".into()),
            r.drift
                .map(|d| format!("{d:+.1}c"))
                .unwrap_or_else(|| "—".into()),
            if r.settled {
                "settled".to_string()
            } else {
                format!("settling ({:.0} ms)", r.chord_held_ms)
            },
            cfg.profile.name()
        ))
        .size(12.0)
        .color(INK),
    );
    ui.add_space(4.0);
    for s in Section::ASC {
        let Some(i) = r.voices.iter().position(|v| v.section == s) else {
            ui.label(
                RichText::new(format!("{:<8} —", s.name()))
                    .font(FontId::monospace(10.0))
                    .color(ink(120)),
            );
            continue;
        };
        let v = &r.voices[i];
        let (target, color) = match r.tg.get(i).and_then(|t| t.as_ref()) {
            Some(t) if t.anchor => (format!("ANCHOR ({})", t.role), TEAL_DARK),
            Some(t) => {
                let band = r.band_of(i, cfg, &hc).unwrap_or(Band::Out);
                let color = match band {
                    Band::In => TEAL_DARK,
                    Band::Marginal => AMBER_TEXT,
                    Band::Out => AMBER_TEXT,
                };
                (
                    format!(
                        "{} {} → {:.1} Hz · sing {:.0}c {}",
                        t.role,
                        t.ratio_txt,
                        t.tgt,
                        t.err.abs(),
                        if t.err > 0.0 { "lower" } else { "higher" }
                    ),
                    color,
                )
            }
            None => ("no target".into(), ink(140)),
        };
        ui.label(
            RichText::new(format!(
                "{:<8} {}{} {:.1} Hz  ET {:+.1}c  {}{}",
                s.name(),
                v.info.name,
                v.info.oct,
                v.f,
                v.info.cents,
                target,
                v.scatter
                    .map(|c| format!("  scatter ±{c:.0}c"))
                    .unwrap_or_default()
            ))
            .font(FontId::monospace(10.0))
            .color(color),
        );
    }
    if !r.pairs.is_empty() {
        ui.add_space(4.0);
        for p in r.pairs.iter().take(6) {
            let (a, b) = (&r.voices[p.a], &r.voices[p.b]);
            ui.label(
                RichText::new(format!(
                    "{}{}–{}{} {} {}: {:.1}c (err {:+.1}c) beats {:.2} Hz",
                    a.info.name,
                    a.info.oct,
                    b.info.name,
                    b.info.oct,
                    p.name,
                    p.ratio,
                    p.cents,
                    p.err,
                    p.beat
                ))
                .font(FontId::monospace(9.5))
                .color(ink(150)),
            );
        }
    }
}

/// The lumen mesh from the `mesh` tap: centerline and every ring,
/// projected orthographically onto the mesh's own x–y plane (the
/// renderer's mm frame; the transform to any anatomical frame is
/// unknown and is not invented), the uncertainty envelope behind it.
pub(super) fn mesh_card(ui: &mut egui::Ui, g: &TractGeometry) {
    ui.label(
        RichText::new("AIRWAY MESH (atlas)")
            .font(FontId::monospace(9.5))
            .color(ink(115)),
    );
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 220.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 12.0, ink(6));
    let n = g.vertex_count.min(g.vertices_mm.len() / 3);
    if n == 0 {
        return;
    }
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for v in g.uncertainty_vertices_mm.chunks_exact(3).take(n) {
        min_x = min_x.min(v[0]);
        max_x = max_x.max(v[0]);
        min_y = min_y.min(v[1]);
        max_y = max_y.max(v[1]);
    }
    let span = (max_x - min_x).max(max_y - min_y).max(1.0);
    let scale = (rect.width().min(rect.height()) - 24.0) / span;
    let cx = (min_x + max_x) * 0.5;
    let cy = (min_y + max_y) * 0.5;
    let project = |x: f32, y: f32| -> Pos2 {
        pos2(
            rect.center().x + (x - cx) * scale,
            rect.center().y - (y - cy) * scale,
        )
    };
    let ring = |verts: &[f32], s: usize| -> Vec<Pos2> {
        (0..g.angular)
            .map(|i| {
                let idx = (s * g.angular + i) * 3;
                project(verts[idx], verts[idx + 1])
            })
            .collect()
    };
    let stroke_u = Stroke::new(0.6, teal_a(40));
    let stroke_m = Stroke::new(0.8, if g.live { TEAL_DARK } else { ink(90) });
    for s in 0..g.sections {
        let mut pts = ring(&g.uncertainty_vertices_mm, s);
        pts.push(pts[0]);
        painter.add(Shape::line(pts, stroke_u));
        let mut pts = ring(&g.vertices_mm, s);
        pts.push(pts[0]);
        painter.add(Shape::line(pts, stroke_m));
    }
    let center: Vec<Pos2> = g
        .centerline_mm
        .chunks_exact(3)
        .map(|c| project(c[0], c[1]))
        .collect();
    painter.add(Shape::line(center, Stroke::new(1.2, AMBER_TEXT)));
    ui.label(
        RichText::new(format!(
            "{} · {} sections × {} · rel σ {:.2} · {} · {}",
            g.model_id,
            g.sections,
            g.angular,
            g.relative_area_std,
            if g.live { "LIVE" } else { "HELD" },
            "x–y of the renderer's mm frame; anatomical orientation unknown"
        ))
        .font(FontId::monospace(9.5))
        .color(ink(140)),
    );
}
