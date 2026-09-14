//! The pipeline's shell side: tap polling and the Room-screen debug panel
//! (Plan v3 §5 Phase 1 gate: every tap shows a live value, timing and
//! latency are shown against the thresholds in config, and the `tract` tap
//! drives a tube view).

use super::*;
use crate::pipeline::runner::{RunnerState, ShellHandle};
use crate::pipeline::tap::TapMsg;
use crate::pipeline::types::{TractModelId, Wire};
use std::sync::atomic::Ordering as AtomicOrdering;

/// What the app holds once a runner is attached.
pub(super) struct PipelineShell {
    pub(super) handle: ShellHandle,
    /// Latest tap per stage index.
    pub(super) latest: Vec<Option<TapMsg>>,
    /// Taps received since the panel was last painted.
    pub(super) received: u64,
}

impl DashboardApp {
    /// Hands the app the runner's taps and stats (Android only today; the
    /// desktop keeps its GPU engine until Phase 5c).
    pub fn attach_pipeline(&mut self, handle: ShellHandle) {
        let n = handle.stages.len();
        self.pipeline = Some(PipelineShell {
            handle,
            latest: vec![None; n],
            received: 0,
        });
    }

    /// Per repaint: drain the taps, keep the newest per stage, and record
    /// the render leg of the mic-to-render latency for the tube's tap.
    pub(super) fn poll_pipeline(&mut self) {
        let Some(shell) = self.pipeline.as_mut() else {
            return;
        };
        for msg in shell.handle.taps.drain() {
            if matches!(msg.value, Wire::AreaFunction(_)) {
                shell.handle.stats.note_render(msg.captured_at);
            }
            if let Some(slot) = shell.latest.get_mut(msg.stage) {
                *slot = Some(msg);
            }
            shell.received += 1;
        }
    }

    /// A notice when the runner has died — the engine-health banner reads it.
    pub(super) fn pipeline_notice(&self) -> Option<&'static str> {
        let shell = self.pipeline.as_ref()?;
        match shell.handle.stats.state() {
            RunnerState::Failed => Some(
                "Analysis pipeline stopped — it failed to build or a stage panicked (see log).",
            ),
            _ => None,
        }
    }

    /// The Room screen's pipeline card.
    pub(super) fn pipeline_card(&mut self, ui: &mut egui::Ui) {
        let time_ms = ui.input(|i| i.time) as f32 * crate::config::consts::MILLIS_PER_SECOND;
        let Some(shell) = self.pipeline.as_ref() else {
            glass(20.0).show(ui, |ui| {
                ui.label(
                    RichText::new("PIPELINE")
                        .font(FontId::monospace(9.5))
                        .color(ink(115)),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(
                        "Runner not attached on this target: the desktop keeps its GPU engine \
                         until Phase 5c. The Android build runs live_model.toml.",
                    )
                    .size(11.5)
                    .color(ink(160)),
                );
            });
            return;
        };
        let stats = &shell.handle.stats;
        let cfg = shell.handle.config;
        let load = |a: &std::sync::atomic::AtomicU64| a.load(AtomicOrdering::Relaxed);
        let budget_us = load(&stats.hop_budget_us).max(1);
        let hops = load(&stats.hops);
        let worst = load(&stats.worst_hop_us);
        let worst_frac = worst as f32 / budget_us as f32;
        let gate_ok = worst_frac < cfg.hop_budget_fraction_max;
        let render_max_ms =
            load(&stats.render_latency_max_us) as f32 / crate::config::consts::MILLIS_PER_SECOND;
        let render_ok = render_max_ms < cfg.mic_to_render_max_ms;

        glass(20.0).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("PIPELINE").font(FontId::monospace(9.5)).color(ink(115)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let (label, color) = match stats.state() {
                        RunnerState::Building => ("BUILDING", AMBER_TEXT),
                        RunnerState::Running => ("RUNNING", TEAL_DARK),
                        RunnerState::Failed => ("FAILED", AMBER_TEXT),
                        RunnerState::Stopped => ("STOPPED", ink(140)),
                    };
                    pill_badge(ui, label, white(200), color);
                });
            });
            ui.label(
                RichText::new(format!(
                    "{} · hop {} @ {:.0} Hz · Phase 1 walking skeleton",
                    shell.handle.pipeline_name,
                    shell.handle.format.hop,
                    shell.handle.format.sample_rate_hz
                ))
                .size(11.0)
                .color(ink(160)),
            );
            ui.add_space(8.0);

            // ── Taps: one row per stage ──
            for (i, (name, backend)) in shell.handle.stages.iter().enumerate() {
                let st = &stats.stages[i];
                let mean_us = load(&st.total_us) / hops.max(1);
                let value = shell
                    .latest
                    .get(i)
                    .and_then(|m| m.as_ref())
                    .map(|m| describe_tap(&m.value))
                    .unwrap_or_else(|| "— no tap yet —".into());
                ui.label(
                    RichText::new(format!("{name} · {backend}"))
                        .font(FontId::monospace(10.0))
                        .color(TEAL_DARK),
                );
                ui.label(RichText::new(value).size(12.0).color(INK));
                ui.label(
                    RichText::new(format!(
                        "{mean_us} µs mean · {} µs max",
                        load(&st.max_us)
                    ))
                    .font(FontId::monospace(9.5))
                    .color(ink(140)),
                );
                ui.add_space(5.0);
            }

            // ── Tube view from the tract tap ──
            if let Some(Wire::AreaFunction(af)) = shell
                .latest
                .last()
                .and_then(|m| m.as_ref())
                .map(|m| &m.value)
            {
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 150.0), Sense::hover());
                paint_tract(ui.painter(), rect, time_ms, af.live, &af.diameters_cm);
                ui.label(
                    RichText::new(format!(
                        "tube from tap: {} basis · {} · model length {:.1} cm",
                        af.basis.name(),
                        if af.live { "LIVE" } else { "HELD" },
                        af.vtl_cm
                    ))
                    .font(FontId::monospace(9.5))
                    .color(ink(140)),
                );
                ui.add_space(6.0);
            }

            // ── Timing against the thresholds in config ──
            let line = |ui: &mut egui::Ui, text: String, ok: bool| {
                ui.label(
                    RichText::new(text)
                        .font(FontId::monospace(9.5))
                        .color(if ok { ink(160) } else { AMBER_TEXT }),
                );
            };
            line(
                ui,
                format!(
                    "hop: last {} µs · worst {} µs = {:.0}% of {} µs budget (gate < {:.0}%)",
                    load(&stats.last_hop_us),
                    worst,
                    worst_frac * crate::config::consts::PERCENT,
                    budget_us,
                    cfg.hop_budget_fraction_max * crate::config::consts::PERCENT
                ),
                gate_ok,
            );
            line(
                ui,
                format!(
                    "deadline misses {} / {} hops · over-gate hops {}",
                    load(&stats.misses),
                    hops,
                    load(&stats.over_fraction)
                ),
                load(&stats.misses) == 0,
            );
            line(
                ui,
                format!(
                    "latency: analysis last {} µs max {} µs · mic→render last {:.1} ms max {:.1} ms (gate < {:.0} ms)",
                    load(&stats.analysis_latency_last_us),
                    load(&stats.analysis_latency_max_us),
                    load(&stats.render_latency_last_us) as f32
                        / crate::config::consts::MILLIS_PER_SECOND,
                    render_max_ms,
                    cfg.mic_to_render_max_ms
                ),
                render_ok,
            );
            line(
                ui,
                format!(
                    "backlog max {} samples · tap drops {} · stage errors {} · taps received {}",
                    load(&stats.backlog_max_samples),
                    load(&stats.tap_drops),
                    load(&stats.stage_errors),
                    shell.received
                ),
                load(&stats.stage_errors) == 0,
            );
            ui.label(
                RichText::new(
                    "mic→render is measured from ring-buffer arrival to the paint that consumed \
                     the tract tap; the device's own capture latency is not included.",
                )
                .size(10.0)
                .color(ink(120)),
            );
        });
    }
}

/// One line of human-readable value per wire type.
fn describe_tap(w: &Wire) -> String {
    match w {
        Wire::AudioFrame(f) => format!("{} samples · frame {}", f.samples.len(), f.frame_index),
        Wire::F0Track(t) => format!(
            "{:.1} Hz · confidence {:.2} · {}{}",
            t.hz,
            t.confidence,
            if t.voiced {
                "VOICED"
            } else if t.rejected {
                "REJECTED (noisy)"
            } else {
                "unvoiced"
            },
            t.snr_db
                .map(|s| format!(" · SNR {s:.0} dB"))
                .unwrap_or_default()
        ),
        Wire::Spectrum(s) => {
            let peak = s
                .magnitude
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map(|(i, _)| i as f32 * s.bin_hz)
                .unwrap_or(0.0);
            format!(
                "{} bins · {:.1} Hz/bin · peak {peak:.0} Hz",
                s.magnitude.len(),
                s.bin_hz
            )
        }
        Wire::HarmonicSeries(h) => format!(
            "H1 {:.3} · H2 {:.3} · H3 {:.3} · {}",
            h.amplitudes[0],
            h.amplitudes[1],
            h.amplitudes[2],
            if h.voiced { "VOICED" } else { "silent" }
        ),
        Wire::VoiceMetrics(m) => {
            let f = |v: Option<f32>| v.map(|x| format!("{x:.1}")).unwrap_or_else(|| "—".into());
            format!(
                "HNR {} · H1-H2 {} · CPP {} · jitter {} % · shimmer {} · centroid {}",
                f(m.hnr_db),
                f(m.h1_h2_db),
                f(m.cpp_db),
                f(m.jitter_pct),
                f(m.shimmer_db),
                f(m.centroid_hz)
            )
        }
        Wire::FormantTrack(t) => format!(
            "F1 {:.0} · F2 {:.0} · F3 {:.0} Hz · measured at {:.0} Hz · {}",
            t.formants[0].frequency,
            t.formants[1].frequency,
            t.formants[2].frequency,
            t.measured_f0,
            if t.fresh { "fresh" } else { "held" }
        ),
        Wire::TractParams(p) => match p.model {
            TractModelId::StoryTwoMode => format!(
                "q1 {:+.2} · q2 {:+.2} · {} basis · VTL {} · {}",
                p.q1,
                p.q2,
                p.basis.name(),
                p.vtl_est_cm
                    .map(|l| format!("{l:.1} cm"))
                    .unwrap_or_else(|| "—".into()),
                if p.valid { "VALID" } else { "held" }
            ),
            TractModelId::MriPca4 => format!(
                "c {:+.2} {:+.2} {:+.2} {:+.2} SD · conf {:.2} · rel σ {:.2} · {}",
                p.modes[0],
                p.modes[1],
                p.modes[2],
                p.modes[3],
                p.confidence.unwrap_or(0.0),
                p.uncertainty.unwrap_or(0.0),
                if p.abstained {
                    format!("ABSTAINED ({})", p.reason.as_str())
                } else {
                    "audio evidence".into()
                }
            ),
        },
        Wire::AreaFunction(a) => format!(
            "{} sections · {:.2} cm each · {} basis · {}",
            a.sections,
            a.section_len_cm,
            a.basis.name(),
            if a.live { "LIVE" } else { "HELD" }
        ),
        Wire::NoteSet(n) => format!(
            "{} active · {} voices · {}",
            n.active_midi.len(),
            n.voices.len(),
            n.voices
                .iter()
                .map(|v| format!("{} {:.1} Hz", v.midi, v.f0_hz))
                .collect::<Vec<_>>()
                .join(" · ")
        ),
        Wire::SectionLabels(l) => l
            .labels
            .iter()
            .map(|x| format!("{} {} ({:.2})", x.midi, x.label.text(), x.confidence))
            .collect::<Vec<_>>()
            .join(" · "),
        Wire::TractGeometry(g) => format!(
            "{} vertices · {} triangles · rel σ {:.2} · {} · {}",
            g.vertex_count,
            g.triangles.len() / 3,
            g.relative_area_std,
            g.model_id,
            if g.live { "LIVE" } else { "HELD" }
        ),
    }
}
