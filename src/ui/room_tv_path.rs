//! The Room screen's TV-path (two-microphone spatial calibration) card.

use super::*;

impl DashboardApp {
    /// TV PATH card on the Room tab: learn the TV's spatial transfer path,
    /// then the user's own, and score frames as TV-like vs you-like on the
    /// bins where the two learned paths measurably differ. The first field
    /// run proved why the contrast matters: a single path scored 0.91 on a
    /// band coverage of 1% — confident-looking, weakly grounded. Copy keeps
    /// the two-microphone honesty boundary explicit throughout: this is
    /// evidence about *where sound comes from*, never proof of who made it.
    pub(super) fn tv_path_card(&mut self, ui: &mut egui::Ui) {
        use crate::spatial::{
            self, CONTRAST_TV, CONTRAST_USER, DISC_MIN, PathSummary, SCORE_NOT_TV, SCORE_TV,
            SINGLE_WEAK_COVERAGE, SpatialStatus,
        };
        let mono = |s: String| RichText::new(s).font(FontId::monospace(11.0));
        let path_line = |ui: &mut egui::Ui, name: &str, p: &PathSummary| {
            ui.label(
                mono(format!(
                    "{name} · quality {:.0}% · coverage {:.0}%",
                    p.quality * 100.0,
                    p.band_coverage * 100.0
                ))
                .color(INK),
            );
            if p.two_source_warning {
                ui.label(
                    RichText::new(format!(
                        "blurred: a second source or heavy reverberation during learning \
                         (rank ratio {:.2})",
                        p.rank_ratio
                    ))
                    .size(10.5)
                    .color(AMBER_TEXT),
                );
            }
        };
        glass(20.0).show(ui, |ui| {
            ui.label(
                RichText::new("TV PATH · SPATIAL")
                    .font(FontId::monospace(9.5))
                    .color(ink(115)),
            );
            ui.add_space(8.0);
            match spatial::status() {
                SpatialStatus::Off => {
                    #[cfg(target_os = "android")]
                    {
                        ui.label(
                            RichText::new(
                                "Learn where the TV's sound physically comes from — the \
                                 relative transfer path between the two microphones, which \
                                 survives any program. Phone in its usual spot, TV PLAYING at \
                                 normal volume, you silent (~11 s). Then teach it YOUR path \
                                 and it scores what separates you from the TV.",
                            )
                            .size(12.0)
                            .color(ink(140)),
                        );
                        ui.add_space(8.0);
                        if probe_chip(ui, "LEARN TV PATH") {
                            spatial::start_learning(spatial::PathTarget::Tv);
                        }
                    }
                    #[cfg(not(target_os = "android"))]
                    ui.label(
                        RichText::new(
                            "Android feature — it rides the two-channel capture the device \
                             probe validated. Learn the TV's spatial path in the phone build.",
                        )
                        .size(12.0)
                        .color(ink(140)),
                    );
                }
                SpatialStatus::Starting => {
                    ui.label(
                        RichText::new("starting two-channel capture…")
                            .size(12.0)
                            .color(TEAL_DARK),
                    );
                }
                SpatialStatus::Calibrating {
                    target,
                    seconds_left,
                    health,
                } => {
                    let line = match target {
                        spatial::PathTarget::Tv => format!(
                            "learning the TV path — TV on, stay silent · {seconds_left:.0} s left"
                        ),
                        spatial::PathTarget::User => format!(
                            "learning YOUR path — speak or sing from your spot, TV muted · \
                             {seconds_left:.0} s left (pauses don't count)"
                        ),
                    };
                    ui.label(RichText::new(line).size(12.0).color(TEAL_DARK));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!(
                            "2-ch {} · {} · {} Hz",
                            health.source, health.mask, health.sample_rate
                        ))
                        .size(10.0)
                        .color(ink(115)),
                    );
                }
                SpatialStatus::Ready {
                    tv,
                    user,
                    disc_coverage,
                    mean_sep,
                    live,
                    health,
                    last_error,
                } => {
                    path_line(ui, "TV path", &tv);
                    if let Some(u) = &user {
                        path_line(ui, "your path", u);
                    }
                    if let Some(e) = &last_error {
                        ui.label(
                            RichText::new(format!("last learn attempt failed: {e}"))
                                .size(10.5)
                                .color(AMBER_TEXT),
                        );
                    }

                    match (disc_coverage, mean_sep) {
                        (Some(d), Some(m)) => {
                            // ── Contrast mode: both paths learned ──
                            let geometry_ok = d >= DISC_MIN;
                            ui.add_space(4.0);
                            let sep_line = format!(
                                "separation: {:.0}% of band distinguishes TV from you · mean \
                                 sep {m:.2}",
                                d * 100.0
                            );
                            ui.label(mono(sep_line).color(if geometry_ok {
                                INK
                            } else {
                                AMBER_TEXT
                            }));
                            ui.add_space(8.0);

                            // Bar: −1 (YOU) .. +1 (TV), fill from center.
                            let (rect, _) = ui.allocate_exact_size(
                                vec2(ui.available_width(), 16.0),
                                Sense::hover(),
                            );
                            let track = Rect::from_min_size(
                                pos2(rect.left(), rect.center().y - 4.0),
                                vec2(rect.width(), 8.0),
                            );
                            ui.painter().rect_filled(track, 4.0, ink(10));
                            let to_x = |v: f32| {
                                track.left() + (v.clamp(-1.0, 1.0) + 1.0) * 0.5 * track.width()
                            };
                            if let Some(v) = live.contrast_ema {
                                let (a, b) = if v >= 0.0 {
                                    (to_x(0.0), to_x(v))
                                } else {
                                    (to_x(v), to_x(0.0))
                                };
                                let fill = if geometry_ok {
                                    if v >= 0.0 {
                                        Color32::from_rgba_unmultiplied(8, 145, 178, 110)
                                    } else {
                                        teal_a(110)
                                    }
                                } else {
                                    ink(50)
                                };
                                ui.painter().rect_filled(
                                    Rect::from_min_max(
                                        pos2(a, track.top()),
                                        pos2(b, track.bottom()),
                                    ),
                                    4.0,
                                    fill,
                                );
                            }
                            for t in [CONTRAST_USER, 0.0, CONTRAST_TV] {
                                let x = to_x(t);
                                ui.painter().line_segment(
                                    [pos2(x, track.top() - 3.0), pos2(x, track.bottom() + 3.0)],
                                    Stroke::new(1.5, ink(60)),
                                );
                            }
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("← YOU")
                                        .font(FontId::monospace(9.0))
                                        .color(ink(115)),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(
                                        RichText::new("TV →")
                                            .font(FontId::monospace(9.0))
                                            .color(ink(115)),
                                    );
                                });
                            });
                            ui.add_space(4.0);
                            let (line, color) = if !geometry_ok {
                                (
                                    format!(
                                        "the two paths barely differ at these mics ({:.0}% \
                                         separating) — the spatial score is not usable \
                                         evidence in this setup",
                                        d * 100.0
                                    ),
                                    AMBER_TEXT,
                                )
                            } else {
                                match live.contrast_ema {
                                    _ if live.quiet && live.contrast_ema.is_none() => {
                                        ("quiet — nothing to score".to_string(), ink(125))
                                    }
                                    Some(v) if v >= CONTRAST_TV => {
                                        (format!("contrast {v:+.2} — TV-like"), CYAN_DEEP)
                                    }
                                    Some(v) if v <= CONTRAST_USER => {
                                        (format!("contrast {v:+.2} — you-like"), TEAL_DARK)
                                    }
                                    Some(v) => (
                                        format!(
                                            "contrast {v:+.2} — indistinct (thresholds \
                                             ±{CONTRAST_TV})"
                                        ),
                                        ink(150),
                                    ),
                                    None => (
                                        "no contrast frames yet — make some sound".to_string(),
                                        ink(125),
                                    ),
                                }
                            };
                            ui.label(
                                RichText::new(line)
                                    .font(FontId::monospace(11.0))
                                    .color(color),
                            );
                        }
                        _ => {
                            // ── Single-path mode: TV path only ──
                            ui.add_space(8.0);
                            let (rect, _) = ui.allocate_exact_size(
                                vec2(ui.available_width(), 16.0),
                                Sense::hover(),
                            );
                            let track = Rect::from_min_size(
                                pos2(rect.left(), rect.center().y - 4.0),
                                vec2(rect.width(), 8.0),
                            );
                            ui.painter().rect_filled(track, 4.0, ink(10));
                            if let Some(ema) = live.single_ema {
                                let x = track.left() + ema.clamp(0.0, 1.0) * track.width();
                                ui.painter().rect_filled(
                                    Rect::from_min_max(track.min, pos2(x, track.bottom())),
                                    4.0,
                                    Color32::from_rgba_unmultiplied(8, 145, 178, 110),
                                );
                            }
                            for t in [SCORE_NOT_TV, SCORE_TV] {
                                let x = track.left() + t * track.width();
                                ui.painter().line_segment(
                                    [pos2(x, track.top() - 3.0), pos2(x, track.bottom() + 3.0)],
                                    Stroke::new(1.5, ink(60)),
                                );
                            }
                            ui.add_space(6.0);
                            let (line, color) = match live.single_ema {
                                _ if live.quiet && live.single_ema.is_none() => {
                                    ("quiet — nothing to score".to_string(), ink(125))
                                }
                                Some(v) if v >= SCORE_TV => (
                                    format!("consistency {v:.2} — energy arriving via the TV path"),
                                    CYAN_DEEP,
                                ),
                                Some(v) if v <= SCORE_NOT_TV => (
                                    format!(
                                        "consistency {v:.2} — NOT the TV path (position \
                                         unknown)"
                                    ),
                                    TEAL_DARK,
                                ),
                                Some(v) => (
                                    format!(
                                        "consistency {v:.2} — mixed or neither (thresholds \
                                         {SCORE_NOT_TV} / {SCORE_TV})"
                                    ),
                                    ink(150),
                                ),
                                None => (
                                    "no scored frames yet — make some sound".to_string(),
                                    ink(125),
                                ),
                            };
                            ui.label(
                                RichText::new(line)
                                    .font(FontId::monospace(11.0))
                                    .color(color),
                            );
                            if tv.band_coverage < SINGLE_WEAK_COVERAGE {
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(format!(
                                        "weak evidence: this score stands on {:.0}% of the \
                                         band, where most rooms look alike to a close mic \
                                         pair — LEARN YOUR PATH to measure what actually \
                                         separates you from the TV",
                                        tv.band_coverage * 100.0
                                    ))
                                    .size(10.5)
                                    .color(AMBER_TEXT),
                                );
                            }
                        }
                    }

                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(format!(
                            "2-ch {} · {} Hz · {} scored frames",
                            health.source, health.sample_rate, live.scored_frames
                        ))
                        .size(10.0)
                        .color(ink(115)),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(
                            "Two mics make this evidence, not proof: anything sounding from \
                             the TV's position matches its path, anyone at your spot matches \
                             yours, a stereo soundbar is only partly captured by one \
                             direction, and moving the phone invalidates everything. \
                             Per-session — relearn after any move.",
                        )
                        .size(10.5)
                        .color(ink(115)),
                    );
                    #[cfg(target_os = "android")]
                    {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if probe_chip(ui, "RE-LEARN TV") {
                                spatial::start_learning(spatial::PathTarget::Tv);
                            }
                            ui.add_space(8.0);
                            let user_label = if user.is_some() {
                                "RE-LEARN YOUR PATH"
                            } else {
                                "LEARN YOUR PATH"
                            };
                            if probe_chip(ui, user_label) {
                                spatial::start_learning(spatial::PathTarget::User);
                            }
                        });
                    }
                }
                SpatialStatus::Failed(e) => {
                    ui.label(
                        RichText::new(format!("failed: {e}"))
                            .size(11.5)
                            .color(AMBER_TEXT),
                    );
                    #[cfg(target_os = "android")]
                    {
                        ui.add_space(8.0);
                        if probe_chip(ui, "RETRY") {
                            spatial::start_learning(spatial::PathTarget::Tv);
                        }
                    }
                }
            }
        });
    }
}
