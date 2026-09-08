//! File import: the import-folder job lifecycle and the Files card that lists the folder.

use super::*;

impl DashboardApp {
    /// Drain the import worker: up to [`IMPORT_FRAMES_PER_TICK`] frames per
    /// repaint into the capture accumulators, then the hand-off to the
    /// Analyzing state when the file ends.
    pub(super) fn poll_import(&mut self, now: f64) {
        let mut n = 0;
        while n < IMPORT_FRAMES_PER_TICK {
            let Some(job) = self.import_job.as_mut() else {
                return;
            };
            match job.try_recv() {
                None => return,
                Some(import::Msg::Meta { .. }) => {}
                Some(import::Msg::Profile(p)) => {
                    self.ingest_profile(*p, true);
                    n += 1;
                }
                Some(import::Msg::Done) => {
                    self.finish_import(now);
                    return;
                }
                Some(import::Msg::Error(e)) => {
                    let name = job.name.clone();
                    self.import_job = None;
                    self.rec = RecState::Idle;
                    self.persist_notice = Some(format!("Could not analyze {name} — {e}"));
                    return;
                }
            }
        }
    }

    /// Analyze `path` as a capture: the accumulators reset, the worker
    /// starts, and the Capture screen shows the progress. Refused while a
    /// live capture or another import is running.
    pub(super) fn start_import(&mut self, path: PathBuf, now: f64) {
        if self.import_job.is_some()
            || matches!(
                self.rec,
                RecState::Recording { .. } | RecState::Analyzing { .. }
            )
        {
            return;
        }
        self.reset_rec(now);
        self.import_job = Some(import::Job::start(path, self.sample_rate));
        self.screen = Screen::Capture;
    }

    /// The file ended: what the export line and the sidecar need comes from
    /// the job, and the state machine proceeds exactly as after a live stop.
    pub(super) fn finish_import(&mut self, now: f64) {
        let Some(job) = self.import_job.take() else {
            return;
        };
        self.last_capture = Some(CaptureFile {
            path: job.path,
            seconds: job.seconds,
            peak_dbfs: job.peak_dbfs,
            clipped: job.clipped,
        });
        self.rec = RecState::Analyzing {
            start: now,
            elapsed: job.seconds as f64,
        };
    }

    pub(super) fn cancel_import(&mut self) {
        // Dropping the job disconnects the worker, which then returns.
        self.import_job = None;
        self.rec = RecState::Idle;
        self.result = None;
    }

    /// Whether `path` is one of the import folder's files (a session saved
    /// from it records `import/<name>` rather than a capture export name).
    pub(super) fn is_imported(&self, path: &std::path::Path) -> bool {
        self.import_dir
            .as_ref()
            .is_some_and(|d| path.starts_with(d))
    }

    pub(super) fn refresh_import_list(&mut self, now: f64) {
        if now - self.import_list_at < IMPORT_LIST_REFRESH_SECS {
            return;
        }
        self.import_list_at = now;
        self.import_list = self
            .import_dir
            .as_deref()
            .map(import::list)
            .unwrap_or_default();
    }

    /// The import folder, as a card: each audio file with an ANALYZE chip
    /// that runs it through the capture pipeline. Nothing is drawn on
    /// platforms without an import folder.
    pub(super) fn files_card(&mut self, ui: &mut egui::Ui) {
        let Some(dir) = self.import_dir.clone() else {
            return;
        };
        let now = ui.input(|i| i.time);
        self.refresh_import_list(now);
        let busy = self.import_job.is_some()
            || matches!(
                self.rec,
                RecState::Recording { .. } | RecState::Analyzing { .. }
            );
        let mut start: Option<PathBuf> = None;
        glass(20.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Files").size(14.0).color(INK).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new("IMPORT FOLDER")
                            .font(FontId::monospace(9.5))
                            .color(ink(115)),
                    );
                });
            });
            ui.add_space(8.0);
            if self.import_list.is_empty() {
                ui.label(
                    RichText::new(
                        "No audio files yet. Share a recording to VoxLabs from any app, \
                         or drop WAV / FLAC / MP3 / M4A / OGG files in the folder below.",
                    )
                    .size(12.0)
                    .color(ink(140)),
                );
            }
            for entry in self.import_list.iter().take(IMPORT_LIST_MAX) {
                ui.horizontal(|ui| {
                    let name = if entry.name.chars().count() > 30 {
                        let head: String = entry.name.chars().take(27).collect();
                        format!("{head}…")
                    } else {
                        entry.name.clone()
                    };
                    ui.label(RichText::new(name).size(12.5).color(INK));
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(import::size_label(entry.bytes))
                            .font(FontId::monospace(9.5))
                            .color(ink(110)),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if busy {
                            ui.label(
                                RichText::new("BUSY")
                                    .font(FontId::monospace(9.5))
                                    .color(ink(90)),
                            );
                        } else if probe_chip(ui, "ANALYZE") {
                            start = Some(entry.path.clone());
                        }
                    });
                });
                ui.add_space(4.0);
            }
            if self.import_list.len() > IMPORT_LIST_MAX {
                ui.label(
                    RichText::new(format!(
                        "+{} more (newest {IMPORT_LIST_MAX} shown)",
                        self.import_list.len() - IMPORT_LIST_MAX
                    ))
                    .font(FontId::monospace(9.5))
                    .color(ink(110)),
                );
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new(dir.display().to_string())
                    .font(FontId::monospace(9.0))
                    .color(ink(100)),
            );
        });
        if let Some(path) = start {
            self.start_import(path, now);
        }
        ui.add_space(14.0);
    }
}
