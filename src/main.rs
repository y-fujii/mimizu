mod graffiti;
use eframe::egui;
use std::*;

struct App {
    engine: graffiti::Engine,
    stroke: Vec<egui::Vec2>,
    letter: Option<char>,
}

impl App {
    fn new() -> Self {
        App {
            engine: graffiti::Engine::new(),
            stroke: Vec::new(),
            letter: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label(format!("{:?}", self.letter));
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());
            let origin = response.rect.min;
            let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));

            if let Some(pointer_pos) = response.interact_pointer_pos() {
                self.stroke.push(pointer_pos - origin);
            } else {
                self.letter = self.engine.classify_2d(&self.stroke);
                self.stroke.clear();
            }

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
