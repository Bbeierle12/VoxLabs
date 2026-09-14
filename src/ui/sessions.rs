//! The Sessions screen: the archive table with its shared column grid, plus the import-folder listing.

use super::*;

/// Sessions-table column grid. The header and the data rows are laid out by
/// separate code — the header on the panel, each row inside a `glass` card —
/// so the widths live here instead of as literals in both places, where they
/// had drifted 4 px apart and left the SUBJECT column visibly misaligned with
/// its own header.
pub(super) const SESSIONS_ID_W: f32 = 78.0;
pub(super) const SESSIONS_F0_W: f32 = 58.0;
/// Space reserved at the right for the match badge, which sizes itself to its
/// text; this is the room the flexible SUBJECT column must leave it.
pub(super) const SESSIONS_MATCH_W: f32 = 68.0;

/// Width of the flexible SUBJECT column from the width available at that point
/// in the layout. `inset_right` is the card inset the header must give back and
/// the rows must not — the card already supplies it. Both call sites use this
/// so the grid has exactly one definition.
pub(super) fn sessions_subject_w(available: f32, inset_right: f32) -> f32 {
    available - SESSIONS_F0_W - SESSIONS_MATCH_W - inset_right
}

impl DashboardApp {
    pub(super) fn screen_sessions(&mut self, ui: &mut egui::Ui) {
        self.screen_kicker(ui, "CAPTURE ARCHIVE", "Sessions");
        ui.add_space(16.0);

        self.files_card(ui);

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
            ui.add_space(GLASS_INSET);
            for (w, label) in [
                (SESSIONS_ID_W, "ID"),
                (0.0, "SUBJECT"),
                (SESSIONS_F0_W, "F0 HZ"),
            ] {
                let text = RichText::new(label)
                    .font(header_font.clone())
                    .color(ink(115));
                let w = if w > 0.0 {
                    w
                } else {
                    // The rows are inset by the card on BOTH sides; the header
                    // only pads its left above, so it gives back the right-hand
                    // inset here to land on the same grid.
                    sessions_subject_w(ui.available_width(), GLASS_INSET)
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
                ui.add_space(GLASS_INSET);
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

        if rows.is_empty() {
            ui.add_space(24.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(if self.sessions.is_empty() {
                        "No captures yet — record on the Capture tab.\nYour first save enrolls your reference voiceprint."
                    } else {
                        "No sessions match this filter."
                    })
                    .size(12.5)
                    .color(ink(120)),
                );
            });
        }

        for (i, s) in rows {
            let (_, badge_bg, badge_fg) = Self::badge_style(s.match_pct);
            let ir = glass(18.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        vec2(SESSIONS_ID_W, 34.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_width(SESSIONS_ID_W);
                            ui.label(
                                RichText::new(&s.id)
                                    .font(FontId::monospace(12.5))
                                    .color(INK)
                                    .strong(),
                            );
                        },
                    );
                    let subj_w = sessions_subject_w(ui.available_width(), 0.0);
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
                        vec2(SESSIONS_F0_W, 34.0),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.set_width(SESSIONS_F0_W);
                            ui.label(
                                RichText::new(format!("{:.0}", s.f0))
                                    .font(FontId::monospace(12.5))
                                    .color(ink(179)),
                            );
                        },
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let text = Self::match_text(s.match_pct);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pixel tolerance for a column edge: the header and the row reach the same
    /// x by different arithmetic, so the last float bit may differ.
    const ALIGN_EPS: f32 = 0.01;

    fn assert_aligned(label: &str, header: f32, row: f32, col: f32) {
        assert!(
            (header - row).abs() < ALIGN_EPS,
            "{label} misaligned at column width {col}: header {header}, row {row}"
        );
    }

    /// The Sessions header sits on the panel; each row sits inside a `glass`
    /// card that insets it on both sides. Two different expressions therefore
    /// compute what has to be one grid — this pins the invariant they exist to
    /// satisfy. Before the shared constants the SUBJECT column was 4 px out,
    /// and nothing said so.
    #[test]
    fn sessions_header_and_rows_share_one_column_grid() {
        for col in [320.0f32, COL_WIDTH, 900.0] {
            // Header: spans the full content column, padding itself by the
            // card inset at each end.
            let head_id_x = GLASS_INSET;
            let head_subj_x = head_id_x + SESSIONS_ID_W;
            let head_subj_w = sessions_subject_w(col - head_subj_x, GLASS_INSET);
            let head_f0_x = head_subj_x + head_subj_w;
            let head_right = col - GLASS_INSET;

            // Row: the card supplies the inset, so its content box is narrower
            // and its subject column subtracts one fewer term.
            let row_w = col - 2.0 * GLASS_INSET;
            let row_id_x = GLASS_INSET;
            let row_subj_x = row_id_x + SESSIONS_ID_W;
            let row_subj_w = sessions_subject_w(row_w - SESSIONS_ID_W, 0.0);
            let row_f0_x = row_subj_x + row_subj_w;
            let row_right = GLASS_INSET + row_w;

            assert_aligned("ID left edge", head_id_x, row_id_x, col);
            assert_aligned("SUBJECT left edge", head_subj_x, row_subj_x, col);
            assert_aligned("SUBJECT width", head_subj_w, row_subj_w, col);
            assert_aligned("F0 left edge", head_f0_x, row_f0_x, col);
            assert_aligned("MATCH right edge", head_right, row_right, col);
        }
    }

    /// The flexible column has to survive the narrowest layout the app can be
    /// shown at; a negative width would invert the allocation.
    #[test]
    fn subject_column_stays_positive_at_the_narrowest_layout() {
        let row_w = 320.0f32 - 2.0 * GLASS_INSET;
        let subj_w = sessions_subject_w(row_w - SESSIONS_ID_W, 0.0);
        assert!(subj_w > 0.0, "SUBJECT column collapsed to {subj_w}");
    }
}
