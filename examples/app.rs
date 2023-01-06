use eframe::egui;
use std::*;

struct App {
    recognizer: graffiti::Recognizer,
    stroke: Vec<egui::Vec2>,
    letter: Option<char>,
}

impl App {
    fn new() -> Self {
        App {
            recognizer: graffiti::Recognizer::new(16.0),
            stroke: Vec::new(),
            letter: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            egui::Grid::new("grid").show(ui, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Recognized letter:")
                });
                ui.label(format!("{:?}", self.letter));
                ui.end_row();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Number mode:")
                });
                ui.label(format!("{:?}", self.recognizer.mode_number));
                ui.end_row();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Symbol next:")
                });
                ui.label(format!("{:?}", self.recognizer.next_symbol));
                ui.end_row();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Caps next:")
                });
                ui.label(format!("{:?}", self.recognizer.next_caps));
                ui.end_row();
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());
            let origin = response.rect.min;

            if let Some(pointer_pos) = response.interact_pointer_pos() {
                self.stroke.push(pointer_pos - origin);
            } else if !self.stroke.is_empty() {
                let stroke = self.stroke.iter().map(|v| [v.x, -v.y]).collect();
                let now = time::Instant::now();
                self.letter = self.recognizer.recognize(&stroke);
                println!("{:} ms", now.elapsed().as_millis());
                self.stroke.clear();
            }

            let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));
            for i in 0..cmp::max(self.stroke.len(), 1) - 1 {
                let x0 = self.stroke[i + 0];
                let x1 = self.stroke[i + 1];
                painter.line_segment([origin + x0, origin + x1], stroke);
            }
        });
    }
}

fn main() {
    eframe::run_native(
        "Graffiti",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(App::new())),
    );
}
