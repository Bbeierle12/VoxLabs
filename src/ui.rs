use crate::concurrency::{EngineEvent, Telemetry};
use eframe::egui;
use rtrb::Producer;
use std::sync::Arc;

use crate::types::VocalProfile;
use triple_buffer::Output;

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
        cc: &eframe::CreationContext<'_>,
        event_tx: Producer<EngineEvent>,
        telemetry: Arc<Telemetry>,
        ui_profile_rx: Output<VocalProfile>,
    ) -> Self {
        // Setup laboratory-esque dark/neon theme
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = egui::Color32::from_rgb(10, 12, 16);
        visuals.panel_fill = egui::Color32::from_rgb(12, 15, 20);
        visuals.override_text_color = Some(egui::Color32::from_rgb(220, 225, 230));
        
        // Neon accents for widgets
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 25, 30);
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 40, 50));
        
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(25, 30, 35);
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 50, 60));
        
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(35, 45, 55);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 255)); // Neon Cyan hover
        
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(45, 55, 65);
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(57, 255, 20)); // Neon Green active
        
        visuals.selection.bg_fill = egui::Color32::from_rgba_unmultiplied(0, 255, 255, 60);
        visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 255, 255));

        cc.egui_ctx.set_visuals(visuals);

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

// Helper function to draw glowing lines
fn draw_glow_line(painter: &egui::Painter, points: Vec<egui::Pos2>, color: egui::Color32, base_thickness: f32) {
    // Outer glow
    painter.add(egui::Shape::line(
        points.clone(),
        egui::Stroke::new(base_thickness * 4.0, color.linear_multiply(0.1)),
    ));
    // Mid glow
    painter.add(egui::Shape::line(
        points.clone(),
        egui::Stroke::new(base_thickness * 2.0, color.linear_multiply(0.3)),
    ));
    // Core line
    painter.add(egui::Shape::line(
        points,
        egui::Stroke::new(base_thickness, color),
    ));
}

fn draw_glow_segment(painter: &egui::Painter, segment: [egui::Pos2; 2], color: egui::Color32, base_thickness: f32) {
    // Outer glow
    painter.line_segment(segment, egui::Stroke::new(base_thickness * 4.0, color.linear_multiply(0.1)));
    // Mid glow
    painter.line_segment(segment, egui::Stroke::new(base_thickness * 2.0, color.linear_multiply(0.3)));
    // Core line
    painter.line_segment(segment, egui::Stroke::new(base_thickness, color));
}

impl eframe::App for DashboardApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Continuous repaint for real-time telemetry
        ui.ctx().request_repaint();

        // Read latest profile non-blocking
        if self.ui_profile_rx.updated() {
            self.current_profile = *self.ui_profile_rx.read();
        }

        let neon_cyan = egui::Color32::from_rgb(0, 255, 255);
        let neon_green = egui::Color32::from_rgb(57, 255, 20);
        let neon_orange = egui::Color32::from_rgb(255, 165, 0);

        egui::Panel::top("top_panel")
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(8, 10, 12)).inner_margin(10.0))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("⚡ LABOSYNTH").font(egui::FontId::proportional(20.0)).color(neon_cyan).strong());
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("QUANTUM RESEARCH PLATFORM v3.1").font(egui::FontId::proportional(14.0)).color(egui::Color32::GRAY));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("CORE SYSTEM: ACTIVE").font(egui::FontId::proportional(12.0)).color(neon_green));
                    });
                });
            });

        let side_panel_frame = egui::Frame::default()
            .fill(egui::Color32::from_rgb(14, 18, 24))
            .inner_margin(12.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 40, 50)));

        egui::Panel::left("left_panel")
            .frame(side_panel_frame.clone())
            .min_size(200.0)
            .show_inside(ui, |ui| {
                ui.label(egui::RichText::new("BIOMETRICS").font(egui::FontId::proportional(16.0)).color(neon_cyan).strong());
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                let data_font = egui::FontId::monospace(14.0);

                egui::Grid::new("biometrics_grid").num_columns(2).spacing([20.0, 10.0]).show(ui, |ui| {
                    ui.label(egui::RichText::new("f0").color(egui::Color32::GRAY));
                    ui.label(egui::RichText::new(format!("{:.1} Hz", self.current_profile.f0)).font(data_font.clone()).color(neon_cyan));
                    ui.end_row();

                    ui.label(egui::RichText::new("F1").color(egui::Color32::GRAY));
                    ui.label(egui::RichText::new(format!("{:.1} Hz", self.current_profile.formants[0].frequency)).font(data_font.clone()).color(neon_cyan));
                    ui.end_row();

                    ui.label(egui::RichText::new("F2").color(egui::Color32::GRAY));
                    ui.label(egui::RichText::new(format!("{:.1} Hz", self.current_profile.formants[1].frequency)).font(data_font.clone()).color(neon_cyan));
                    ui.end_row();

                    ui.label(egui::RichText::new("F3").color(egui::Color32::GRAY));
                    ui.label(egui::RichText::new(format!("{:.1} Hz", self.current_profile.formants[2].frequency)).font(data_font.clone()).color(neon_cyan));
                    ui.end_row();
                });

                ui.add_space(20.0);
                
                let status_bg = if self.current_profile.valid {
                    egui::Color32::from_rgb(10, 40, 10)
                } else {
                    egui::Color32::from_rgb(40, 40, 10)
                };
                let status_color = if self.current_profile.valid { neon_green } else { neon_orange };
                let status_text = if self.current_profile.valid { "STATUS: LOCKED" } else { "STATUS: SEARCHING" };

                egui::Frame::default()
                    .fill(status_bg)
                    .stroke(egui::Stroke::new(1.0, status_color))
                    .inner_margin(8.0)
                    .corner_radius(4.0)
                    .show(ui, |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new(status_text).font(egui::FontId::monospace(14.0)).color(status_color).strong());
                        });
                    });
            });

        egui::Panel::right("right_panel")
            .frame(side_panel_frame)
            .min_size(220.0)
            .show_inside(ui, |ui| {
                ui.label(egui::RichText::new("A/B CONTROLS").font(egui::FontId::proportional(16.0)).color(neon_cyan).strong());
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(egui::RichText::new("Harmonics").color(egui::Color32::GRAY));
                ui.add(egui::Slider::new(&mut self.harmonic_count, 1..=32));
                if ui.button("APPLY HARMONICS").clicked() {
                    let _ = self
                        .event_tx
                        .push(EngineEvent::SetHarmonicCount(self.harmonic_count));
                }

                ui.add_space(15.0);

                ui.label(egui::RichText::new("Delta f (Hz)").color(egui::Color32::GRAY));
                ui.add(egui::Slider::new(&mut self.delta_f, 0.5..=40.0));
                if ui.button("APPLY DELTA F").clicked() {
                    let _ = self.event_tx.push(EngineEvent::SetDeltaF(self.delta_f));
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(egui::RichText::new("TELEMETRY").font(egui::FontId::proportional(14.0)).color(neon_cyan).strong());
                ui.add_space(8.0);

                let consumed = self
                    .telemetry
                    .consumed_frames
                    .load(std::sync::atomic::Ordering::Relaxed);
                let xruns = self
                    .telemetry
                    .xruns
                    .load(std::sync::atomic::Ordering::Relaxed);
                
                let data_font = egui::FontId::monospace(12.0);
                egui::Grid::new("telemetry_grid").num_columns(2).spacing([20.0, 8.0]).show(ui, |ui| {
                    ui.label(egui::RichText::new("Frames Consumed:").color(egui::Color32::GRAY).font(data_font.clone()));
                    ui.label(egui::RichText::new(format!("{}", consumed)).font(data_font.clone()).color(neon_cyan));
                    ui.end_row();

                    ui.label(egui::RichText::new("XRuns:").color(egui::Color32::GRAY).font(data_font.clone()));
                    ui.label(egui::RichText::new(format!("{}", xruns)).font(data_font.clone()).color(neon_orange));
                    ui.end_row();
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(10, 12, 16)).inner_margin(16.0))
            .show_inside(ui, |ui| {
                
                ui.label(egui::RichText::new("REAL-TIME SPECTRUM VISUALIZER").font(egui::FontId::proportional(14.0)).color(egui::Color32::GRAY).strong());
                ui.add_space(8.0);

                let (response, painter) =
                    ui.allocate_painter(ui.available_size(), egui::Sense::hover());
                let rect = response.rect;

                // Draw background screen
                painter.rect(
                    rect,
                    4.0,
                    egui::Color32::from_rgb(5, 7, 10),
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 40, 50)),
                    egui::StrokeKind::Inside,
                );

                // Draw Oscilloscope Grid
                let grid_color = egui::Color32::from_rgba_unmultiplied(30, 50, 70, 100);
                let grid_spacing = 40.0;
                
                let mut x = rect.left();
                while x < rect.right() {
                    painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], egui::Stroke::new(1.0, grid_color));
                    x += grid_spacing;
                }
                
                let mut y = rect.bottom();
                while y > rect.top() {
                    painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], egui::Stroke::new(1.0, grid_color));
                    y -= grid_spacing;
                }

                if self.current_profile.valid && self.current_profile.f0 > 0.0 {
                    let max_freq = 4000.0; // View up to 4kHz
                    let to_screen_x = |f: f32| -> f32 { rect.left() + (f / max_freq) * rect.width() };

                    // Helper to evaluate formant envelope (mirroring synthesis logic)
                    let eval_amp = |freq: f32| -> f32 {
                        let mut gain = 0.0;
                        for formant in &self.current_profile.formants {
                            if formant.frequency > 0.0 && formant.bandwidth > 0.0 {
                                let q = formant.frequency / formant.bandwidth;
                                let omega = freq / formant.frequency;
                                let denom =
                                    ((1.0 - omega * omega).powi(2) + (omega / q).powi(2)).sqrt();
                                gain += 1.0 / (1.0 + denom * 10.0);
                            }
                        }
                        let rolloff = 1.0 / (1.0 + freq / 100.0);
                        (gain + 0.1) * rolloff
                    };

                    // 1. Draw Formant Envelope Curve
                    let mut points = vec![];
                    let steps = 300;
                    for i in 0..=steps {
                        let freq = (i as f32 / steps as f32) * max_freq;
                        let amp = eval_amp(freq);
                        let display_amp = (amp * rect.height() * 3.0).clamp(0.0, rect.height());
                        points.push(egui::pos2(to_screen_x(freq), rect.bottom() - display_amp));
                    }
                    
                    draw_glow_line(&painter, points, egui::Color32::from_rgb(0, 150, 255), 1.5);

                    // 2. Draw Active Harmonics
                    for n in 1..=self.harmonic_count {
                        let freq = n as f32 * self.current_profile.f0;
                        if freq > max_freq {
                            continue;
                        }

                        let amp = eval_amp(freq);
                        let display_amp = (amp * rect.height() * 3.0).clamp(0.0, rect.height());

                        // Left channel (carrier) - Cyan
                        draw_glow_segment(
                            &painter,
                            [
                                egui::pos2(to_screen_x(freq), rect.bottom()),
                                egui::pos2(to_screen_x(freq), rect.bottom() - display_amp),
                            ],
                            neon_cyan,
                            1.5
                        );

                        // Right channel (shifted entrainment target) - Green
                        let freq_r = freq + self.delta_f;
                        if freq_r <= max_freq {
                            draw_glow_segment(
                                &painter,
                                [
                                    egui::pos2(to_screen_x(freq_r), rect.bottom()),
                                    egui::pos2(to_screen_x(freq_r), rect.bottom() - display_amp * 0.8), // slightly shorter to visually distinguish
                                ],
                                neon_green,
                                1.5
                            );
                        }
                    }
                } else {
                    // Scanning animation simulation or awaiting text
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "AWAITING BIOMETRIC LOCK...",
                        egui::FontId::monospace(20.0),
                        egui::Color32::from_rgba_unmultiplied(0, 255, 255, 100),
                    );
                }
            });
    }
}
