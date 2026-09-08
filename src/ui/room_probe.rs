//! The Room screen's capture-stack capability probe card (Android device probe report).

use super::*;

impl DashboardApp {
    /// DEVICE card on the Room tab: the capture-stack capability probe.
    /// Every spatial method in the live-vs-loudspeaker roadmap is
    /// conditional on facts only measurable on the device in hand — real
    /// `UNPROCESSED` support, the microphone inventory, and whether two
    /// *independent* channels reach the app — so the probe's answers are
    /// shown here with their raw numbers, not rounded to a verdict alone.
    pub(super) fn device_probe_card(&mut self, ui: &mut egui::Ui) {
        use crate::device_probe as probe;
        let mono = |s: String| RichText::new(s).font(FontId::monospace(11.0));
        glass(20.0).show(ui, |ui| {
            ui.label(
                RichText::new("DEVICE · CAPTURE STACK")
                    .font(FontId::monospace(9.5))
                    .color(ink(115)),
            );
            ui.add_space(8.0);
            match probe::status() {
                probe::ProbeStatus::NotRun => {
                    #[cfg(target_os = "android")]
                    {
                        ui.label(
                            RichText::new(
                                "Three questions the spatial roadmap hangs on: is UNPROCESSED \
                                 capture real on this device, what microphones exist, and do \
                                 two independent channels reach the app. Run it with sound in \
                                 the room — TV playing, or speak while it runs; silence cannot \
                                 separate copies from coincidence.",
                            )
                            .size(12.0)
                            .color(ink(140)),
                        );
                        ui.add_space(8.0);
                        if probe_chip(ui, "RUN PROBE") {
                            probe::start();
                        }
                    }
                    #[cfg(not(target_os = "android"))]
                    ui.label(
                        RichText::new(
                            "Android diagnostic — run it in the phone build. It probes \
                             UNPROCESSED support, the microphone inventory, and whether two \
                             independent channels reach the app; desktop capture goes through \
                             a different stack and none of those answers transfer.",
                        )
                        .size(12.0)
                        .color(ink(140)),
                    );
                }
                probe::ProbeStatus::Running => {
                    ui.label(
                        RichText::new("probing — second recorder capturing ~1 s…")
                            .size(12.0)
                            .color(TEAL_DARK),
                    );
                }
                probe::ProbeStatus::Done(r) => {
                    // The raw-path answer is a device FACT, and the card
                    // must read that way. The first phrasing here ("not
                    // supported — capture is processed") looked like a
                    // malfunction to fix; it isn't one. No app, permission,
                    // or setting enables UNPROCESSED — the phone's firmware
                    // either offers the tuning or it doesn't.
                    match r.unprocessed {
                        Some(true) => {
                            ui.label(
                                mono("UNPROCESSED: supported — the raw path exists".to_string())
                                    .color(TEAL_DARK),
                            );
                        }
                        Some(false) => {
                            ui.label(
                                mono(
                                    "raw path (UNPROCESSED): not offered by this device"
                                        .to_string(),
                                )
                                .color(INK),
                            );
                            ui.label(
                                RichText::new(
                                    "Firmware fact, not a failure — nothing in this app or in \
                                     settings can enable it. Everything below runs on \
                                     VOICE_RECOGNITION instead: noise suppression and \
                                     auto-gain are off by platform default, but response \
                                     below ~100 Hz is unguaranteed, so the sub-bass and \
                                     envelope cues stay down-weighted, exactly as the \
                                     research plan assumed. The spatial and high-band cues \
                                     are unaffected.",
                                )
                                .size(10.5)
                                .color(ink(140)),
                            );
                        }
                        None => {
                            ui.label(mono("UNPROCESSED: query failed".to_string()).color(ink(150)));
                        }
                    }
                    if r.record_permission == Some(false) {
                        ui.label(
                            mono("microphone permission not granted".to_string())
                                .color(AMBER_TEXT),
                        );
                    }
                    ui.add_space(6.0);
                    ui.label(
                        mono(format!("microphones reported: {}", r.mics.len())).color(INK),
                    );
                    for m in &r.mics {
                        let pos = match m.position {
                            Some([x, y, z]) => format!(
                                " · ({:+.0}, {:+.0}, {:+.0}) mm",
                                x * 1e3,
                                y * 1e3,
                                z * 1e3
                            ),
                            None => String::new(),
                        };
                        ui.label(
                            RichText::new(format!("· {} — {}{}", m.description, m.location, pos))
                                .size(10.5)
                                .color(ink(140)),
                        );
                    }
                    ui.add_space(6.0);
                    match &r.capture {
                        probe::CaptureOutcome::Failed(why) => {
                            ui.label(
                                RichText::new(format!("2-ch capture failed: {why}"))
                                    .size(11.5)
                                    .color(AMBER_TEXT),
                            );
                        }
                        probe::CaptureOutcome::Ran(run) => {
                            ui.label(
                                mono(format!(
                                    "2-ch capture: {} · {} · {} Hz · {} frames",
                                    run.source, run.mask, run.sample_rate, run.frames
                                ))
                                .color(INK),
                            );
                            for a in &run.active_mics {
                                ui.label(
                                    RichText::new(format!("· {a}")).size(10.5).color(ink(140)),
                                );
                            }
                            let s = &run.stats;
                            ui.label(
                                mono(format!(
                                    "corr {:.4} @ {:+.2} ms · L−R {:.0} dB · L {:.0} / R {:.0} dBFS",
                                    s.corr_best, s.best_lag_ms, s.diff_db, s.rms_l_db, s.rms_r_db
                                ))
                                .color(INK),
                            );
                            ui.add_space(4.0);
                            let (verdict, vcolor) = match s.verdict {
                                probe::ChannelVerdict::Independent => (
                                    "TWO INDEPENDENT CHANNELS — the spatial lane is open",
                                    TEAL_DARK,
                                ),
                                probe::ChannelVerdict::Duplicated => (
                                    "CHANNELS ARE COPIES — one effective microphone reaches \
                                     the app",
                                    AMBER_TEXT,
                                ),
                                probe::ChannelVerdict::LowSignal => (
                                    "NO SIGNAL ON EITHER CHANNEL — mic busy or permission \
                                     missing; re-run",
                                    AMBER_TEXT,
                                ),
                                probe::ChannelVerdict::Inconclusive => (
                                    "INCONCLUSIVE — re-run with steady sound in the room",
                                    AMBER_TEXT,
                                ),
                            };
                            ui.label(
                                RichText::new(verdict)
                                    .font(FontId::monospace(10.5))
                                    .color(vcolor),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(
                                    "Limits, in the open: a duplicated pair can be HAL \
                                     beamforming rather than a copied wire, and one distant \
                                     source with no mic self-noise can also correlate near 1. \
                                     Sharpest run: TV playing plus your voice at arm's length.",
                                )
                                .size(10.5)
                                .color(ink(115)),
                            );
                        }
                    }
                    for n in &r.notes {
                        ui.label(RichText::new(format!("note: {n}")).size(10.0).color(ink(115)));
                    }
                    #[cfg(target_os = "android")]
                    {
                        ui.add_space(8.0);
                        if probe_chip(ui, "RE-RUN") {
                            probe::start();
                        }
                    }
                }
            }
        });
    }
}
