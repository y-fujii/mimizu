// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::model;
use eframe::egui;
use std::*;

type Vector2 = nalgebra::Vector2<f32>;

pub struct Ui {}

fn v2_invert_y(v: Vector2) -> Vector2 {
    Vector2::new(v[0], -v[1])
}

impl Ui {
    pub fn new(ctx_main: &egui::Context, ctx_overlay: &egui::Context) -> Self {
        Self::add_font_ja(ctx_main);
        Self::add_font_ja(ctx_overlay);

        Ui {}
    }

    pub fn main(&self, ctx: &egui::Context, model: &mut model::Model) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.controls(ui, model);
            if model.is_active {
                self.text(ui, model);
                self.plot(ui, model);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        "Press the grips and triggers of both hands simultaneously to activate.",
                    )
                });
            }
        });
    }

    pub fn overlay(&self, ctx: &egui::Context, model: &mut model::Model) {
        let frame = egui::Frame::none();
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            self.controls(ui, model);
            self.text(ui, model);
            //self.plot(ui, model);
        });
    }

    fn controls(&self, ui: &mut egui::Ui, model: &mut model::Model) {
        ui.horizontal(|ui| {
            ui.checkbox(&mut model.is_active, "Active");
            ui.checkbox(&mut model.use_chatbox, "OSC Chatbox");
            ui.checkbox(&mut model.use_key_emulation, "Keyboard emulation");
            let labels = ["Latin", "ひらがな"];
            egui::ComboBox::from_id_salt(egui::Id::new("CharClass"))
                .selected_text(labels[model.char_class as usize])
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut model.char_class, model::CharClass::Latin, labels[0]);
                    ui.selectable_value(
                        &mut model.char_class,
                        model::CharClass::Hiragana,
                        labels[1],
                    );
                });
        });
    }

    fn text(&self, ui: &mut egui::Ui, model: &model::Model) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            ui.label(
                egui::RichText::new(model.text_l())
                    .size(24.0)
                    .color(ui.visuals().strong_text_color()),
            );
            ui.label(
                egui::RichText::new(self.indicator(model))
                    .size(24.0)
                    .color(ui.visuals().strong_text_color())
                    .background_color(ui.visuals().selection.bg_fill),
            );
            ui.label(
                egui::RichText::new(model.text_r())
                    .size(24.0)
                    .color(ui.visuals().strong_text_color()),
            );
        });
    }

    fn plot(&self, ui: &mut egui::Ui, model: &model::Model) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());
        let r_min = Vector2::new(response.rect.min.x, response.rect.min.y);
        let r_max = Vector2::new(response.rect.max.x, response.rect.max.y);

        for stroke in model.current_strokes.iter() {
            if stroke.len() < 2 {
                continue;
            }
            let mut s_min = Vector2::repeat(f32::INFINITY);
            let mut s_max = Vector2::repeat(-f32::INFINITY);
            for v in stroke.iter() {
                let v = v2_invert_y(*v);
                s_min = s_min.inf(&v);
                s_max = s_max.sup(&v);
            }
            let scale = (r_max - r_min).component_div(&(s_max - s_min)).min();
            let offset = 0.5 * ((r_max + r_min) - scale * (s_max + s_min));

            let egui_stroke = egui::Stroke::new(2.0, ui.visuals().text_color());
            for i in 0..stroke.len() - 1 {
                let v0 = scale * v2_invert_y(stroke[i + 0]) + offset;
                let v1 = scale * v2_invert_y(stroke[i + 1]) + offset;
                painter.line_segment(
                    [egui::Pos2::new(v0[0], v0[1]), egui::Pos2::new(v1[0], v1[1])],
                    egui_stroke,
                );
            }
        }
    }

    fn indicator(&self, model: &model::Model) -> char {
        match model.recognizer.modifier() {
            mimizu::GraffitiModifier::Symbol => '.',
            mimizu::GraffitiModifier::Caps => '^',
            mimizu::GraffitiModifier::None => match model.recognizer.mode() {
                mimizu::GraffitiMode::Number => '#',
                _ => ' ',
            },
        }
    }

    fn add_font_ja(ctx: &egui::Context) {
        let mut font = egui::FontDefinitions::default();
        font.font_data.insert(
            "mplus".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/mplus-1c-regular-sub.ttf")),
        );
        font.families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .push("mplus".to_owned());
        font.families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push("mplus".to_owned());
        ctx.set_fonts(font);
    }
}
