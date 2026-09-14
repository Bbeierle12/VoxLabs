//! Room → DIAGNOSTICS: what the app knows about its own microphone and
//! analysis state, and the last log lines, on screen — for a phone with no
//! `adb`. A screenshot of this card is the bug report.

use super::*;
use crate::concurrency::{AnalysisState, MicPermission, MicRequest};

/// Log lines shown; the ring holds more (`diag::CAPACITY`).
const LOG_LINES: usize = 24;

impl DashboardApp {
    pub(super) fn diagnostics_card(&mut self, ui: &mut egui::Ui) {
        glass(20.0).show(ui, |ui| {
            ui.label(
                RichText::new("DIAGNOSTICS")
                    .font(FontId::monospace(9.5))
                    .color(ink(115)),
            );
            ui.add_space(6.0);

            let row = |ui: &mut egui::Ui, key: &str, value: String, ok: bool| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(key)
                            .font(FontId::monospace(10.0))
                            .color(ink(140)),
                    );
                    ui.label(
                        RichText::new(value)
                            .font(FontId::monospace(10.0))
                            .color(if ok { TEAL_DARK } else { AMBER_TEXT }),
                    );
                });
            };

            let permission = self.telemetry.mic_permission();
            row(
                ui,
                "mic permission",
                match permission {
                    MicPermission::Unknown => "not checked on this target".into(),
                    MicPermission::NotGranted => "NOT GRANTED".into(),
                    MicPermission::Granted => "granted".into(),
                },
                permission != MicPermission::NotGranted,
            );
            let request = self.telemetry.mic_request();
            row(
                ui,
                "permission dialog",
                match request {
                    MicRequest::NotNeeded => "not needed".into(),
                    MicRequest::Raised => "raised (tap Allow)".into(),
                    MicRequest::Failed => "COULD NOT BE SHOWN".into(),
                },
                request != MicRequest::Failed,
            );
            let audio_ok = !self.telemetry.audio_unavailable();
            row(
                ui,
                "audio engine",
                if audio_ok {
                    format!("running · input level {:.4}", self.input_rms())
                } else {
                    "NOT RUNNING".into()
                },
                audio_ok,
            );
            let analysis = self.telemetry.analysis_state();
            row(
                ui,
                "analysis",
                format!("{analysis:?}"),
                matches!(analysis, AnalysisState::Running),
            );

            // Grant by hand: the system Settings page for this app.
            #[cfg(target_os = "android")]
            if permission == MicPermission::NotGranted {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(
                        "Grant it by hand: Open app settings → Permissions → Microphone → \
                         Allow. Audio starts here as soon as it is granted; no relaunch.",
                    )
                    .size(11.0)
                    .color(ink(160)),
                );
                ui.add_space(4.0);
                if probe_chip(ui, "OPEN APP SETTINGS") && !crate::permission::open_app_settings() {
                    self.persist_notice = Some(
                        "Could not open Settings from here; open it from the launcher.".into(),
                    );
                }
            }

            ui.add_space(8.0);
            ui.label(
                RichText::new("recent log")
                    .font(FontId::monospace(9.5))
                    .color(ink(115)),
            );
            let lines = crate::diag::tail(LOG_LINES);
            if lines.is_empty() {
                ui.label(
                    RichText::new("(nothing logged on this target)")
                        .font(FontId::monospace(9.0))
                        .color(ink(120)),
                );
            }
            for line in lines {
                let warn = line.starts_with("WARN") || line.starts_with("ERROR");
                ui.label(
                    RichText::new(line)
                        .font(FontId::monospace(9.0))
                        .color(if warn { AMBER_TEXT } else { ink(150) }),
                );
            }
        });
    }
}
