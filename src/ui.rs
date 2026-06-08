use crate::concurrency::{EngineEvent, Telemetry};
use eframe::egui;
use rtrb::Producer;
use std::sync::Arc;

use triple_buffer::Output;
use crate::types::VocalProfile;

pub struct DashboardApp {
    event_tx: Producer<EngineEvent>,
    telemetry: Arc<Telemetry>,
    ui_profile_rx: Output<VocalProfile>,
    current_profile: VocalProfile,
    harmonic_count: usize,
    delta_f: f32,
}

impl DashboardApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        event_tx: Producer<EngineEvent>,
        telemetry: Arc<Telemetry>,
        ui_profile_rx: Output<VocalProfile>,
    ) -> Self {
        Self {
            event_tx,
            telemetry,
            ui_profile_rx,
            current_profile: VocalProfile::default(),
            harmonic_count: 5,
            delta_f: 6.0,
        }
    }
}

impl eframe::App for DashboardApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Continuous repaint for real-time telemetry
        ui.ctx().request_repaint();

        // Read latest profile non-blocking
        if self.ui_profile_rx.updated() {
            self.current_profile = *self.ui_profile_rx.read();
        }

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            ui.heading("Personal Voice-Harmonic Analysis & Synthesis Engine");
        });

        egui::Panel::left("left_panel").show_inside(ui, |ui| {
            ui.heading("Biometrics");
            ui.separator();

            ui.label(format!("f0: {:.1} Hz", self.current_profile.f0));
            ui.label(format!("F1: {:.1} Hz", self.current_profile.formants[0].frequency));
            ui.label(format!("F2: {:.1} Hz", self.current_profile.formants[1].frequency));
            ui.label(format!("F3: {:.1} Hz", self.current_profile.formants[2].frequency));

            if self.current_profile.valid {
                ui.label(egui::RichText::new("Status: LOCKED").color(egui::Color32::GREEN));
            } else {
                ui.label(egui::RichText::new("Status: SEARCHING").color(egui::Color32::YELLOW));
            }
        });

        egui::Panel::right("right_panel").show_inside(ui, |ui| {
            ui.heading("A/B Controls");
            ui.separator();

            ui.add(egui::Slider::new(&mut self.harmonic_count, 1..=32).text("Harmonics"));
            if ui.button("Apply Harmonic Count").clicked() {
                let _ = self
                    .event_tx
                    .push(EngineEvent::SetHarmonicCount(self.harmonic_count));
            }

            ui.add(egui::Slider::new(&mut self.delta_f, 0.5..=40.0).text("Delta f (Hz)"));
            if ui.button("Apply Delta f").clicked() {
                let _ = self.event_tx.push(EngineEvent::SetDeltaF(self.delta_f));
            }

            ui.separator();
            ui.label("Telemetry:");
            let consumed = self
                .telemetry
                .consumed_frames
                .load(std::sync::atomic::Ordering::Relaxed);
            let xruns = self
                .telemetry
                .xruns
                .load(std::sync::atomic::Ordering::Relaxed);
            ui.label(format!("Frames Consumed: {}", consumed));
            ui.label(format!("XRuns: {}", xruns));
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Spectrum Visualizer");
            
            let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
            let rect = response.rect;
            
            // Draw background
            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 20, 20));

            if self.current_profile.valid && self.current_profile.f0 > 0.0 {
                let max_freq = 4000.0; // View up to 4kHz
                let to_screen_x = |f: f32| -> f32 {
                    rect.left() + (f / max_freq) * rect.width()
                };

                // Helper to evaluate formant envelope (mirroring synthesis logic)
                let eval_amp = |freq: f32| -> f32 {
                    let mut gain = 0.0;
                    for formant in &self.current_profile.formants {
                        if formant.frequency > 0.0 && formant.bandwidth > 0.0 {
                            let q = formant.frequency / formant.bandwidth;
                            let omega = freq / formant.frequency;
                            let denom = ((1.0 - omega * omega).powi(2) + (omega / q).powi(2)).sqrt();
                            gain += 1.0 / (1.0 + denom * 10.0);
                        }
                    }
                    let rolloff = 1.0 / (1.0 + freq / 100.0);
                    (gain + 0.1) * rolloff
                };

                // 1. Draw Formant Envelope Curve
                let mut points = vec![];
                let steps = 200;
                for i in 0..=steps {
                    let freq = (i as f32 / steps as f32) * max_freq;
                    let amp = eval_amp(freq);
                    let display_amp = (amp * rect.height() * 3.0).clamp(0.0, rect.height());
                    points.push(egui::pos2(to_screen_x(freq), rect.bottom() - display_amp));
                }
                painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(100, 150, 250, 100))));

                // 2. Draw Active Harmonics
                for n in 1..=self.harmonic_count {
                    let freq = n as f32 * self.current_profile.f0;
                    if freq > max_freq { continue; }
                    
                    let amp = eval_amp(freq);
                    let display_amp = (amp * rect.height() * 3.0).clamp(0.0, rect.height());

                    // Left channel (carrier)
                    painter.line_segment(
                        [
                            egui::pos2(to_screen_x(freq), rect.bottom()),
                            egui::pos2(to_screen_x(freq), rect.bottom() - display_amp)
                        ],
                        egui::Stroke::new(3.0, egui::Color32::from_rgb(250, 150, 100))
                    );

                    // Right channel (shifted entrainment target)
                    let freq_r = freq + self.delta_f;
                    if freq_r <= max_freq {
                        painter.line_segment(
                            [
                                egui::pos2(to_screen_x(freq_r), rect.bottom()),
                                egui::pos2(to_screen_x(freq_r), rect.bottom() - display_amp * 0.8) // slightly shorter to visually distinguish
                            ],
                            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 250, 150))
                        );
                    }
                }
            } else {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Awaiting valid biometric lock...",
                    egui::FontId::proportional(20.0),
                    egui::Color32::GRAY
                );
            }
        });
    }
}
