#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod chatbox;
mod openvr;
use eframe::egui;
use std::*;

type Vector2 = nalgebra::Vector2<f32>;

struct Model {
    is_finished: bool,
    error: Option<String>,
    projector: [graffiti_3d::StrokeProjector; 2],
    strokes: collections::VecDeque<Vec<Vector2>>,
}

struct App {
    model: sync::Arc<sync::Mutex<Model>>,
    recognizer: graffiti_3d::GraffitiRecognizer,
    text: Vec<char>,
    cursor: usize,
    chatbox: Option<chatbox::ChatBox>,
}

fn v2_invert_y(v: Vector2) -> Vector2 {
    Vector2::new(v[0], -v[1])
}

impl App {
    fn new(model: sync::Arc<sync::Mutex<Model>>) -> Self {
        App {
            model: model,
            recognizer: graffiti_3d::GraffitiRecognizer::new(0.02),
            text: Vec::new(),
            cursor: 0,
            chatbox: chatbox::ChatBox::new().ok(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let error;
        let current_strokes;
        let stroke;
        {
            let mut model = self.model.lock().unwrap();
            error = model.error.clone();
            current_strokes = [model.projector[0].stroke(), model.projector[1].stroke()];
            stroke = model.strokes.pop_front();
        }

        if let Some(stroke) = stroke {
            match self.recognizer.recognize(&stroke) {
                Some('\x08') => {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                        self.text.remove(self.cursor);
                    }
                }
                Some('←') => {
                    self.cursor = cmp::max(self.cursor, 1) - 1;
                }
                Some('→') => {
                    self.cursor = cmp::min(self.cursor + 1, self.text.len());
                }
                Some('\n') => {
                    self.text.clear();
                    self.cursor = 0;
                }
                Some(c) => {
                    self.text.insert(self.cursor, c);
                    self.cursor += 1;
                }
                None => (),
            }
        }
        if let Some(ref mut chatbox) = self.chatbox {
            chatbox.input(self.text.iter().collect());
            chatbox.typing(current_strokes.iter().any(|s| s.len() > 0));
            chatbox.update();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(err) = error {
                ui.label(err);
            }

            if let None = self.chatbox {
                ui.label("Failed to initialize Chatbox client.");
            }

            let indicator = match self.recognizer.modifier() {
                graffiti_3d::GraffitiModifier::Symbol => ".",
                graffiti_3d::GraffitiModifier::Caps => "^",
                graffiti_3d::GraffitiModifier::None => match self.recognizer.mode() {
                    graffiti_3d::GraffitiMode::Number => "#",
                    _ => " ",
                },
            };
            let lhs: String = self.text[..self.cursor].iter().collect();
            let rhs: String = self.text[self.cursor..].iter().collect();
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 1.0;
                ui.label(
                    egui::RichText::new(lhs)
                        .size(24.0)
                        .color(egui::Color32::from_rgb(255, 255, 255)),
                );
                ui.label(
                    egui::RichText::new(indicator)
                        .size(24.0)
                        .color(egui::Color32::from_rgb(0, 0, 0))
                        .background_color(egui::Color32::from_rgb(128, 192, 255)),
                );
                ui.label(
                    egui::RichText::new(rhs)
                        .size(24.0)
                        .color(egui::Color32::from_rgb(255, 255, 255)),
                );
            });

            let (response, painter) =
                ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());
            let r_min = Vector2::new(response.rect.min.x, response.rect.min.y);
            let r_max = Vector2::new(response.rect.max.x, response.rect.max.y);

            for stroke in current_strokes.iter() {
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

                let egui_stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 255));
                for i in 0..stroke.len() - 1 {
                    let v0 = scale * v2_invert_y(stroke[i + 0]) + offset;
                    let v1 = scale * v2_invert_y(stroke[i + 1]) + offset;
                    painter.line_segment(
                        [egui::Pos2::new(v0[0], v0[1]), egui::Pos2::new(v1[0], v1[1])],
                        egui_stroke,
                    );
                }
            }
        });
    }
}

fn vr_thread_proc(model: sync::Arc<sync::Mutex<Model>>, ctx: egui::Context) {
    let Ok(vr) = openvr::OpenVr::new() else {
        model.lock().unwrap().error = Some("Failed to initialize OpenVR.".to_string());
        return;
    };
    let mut prev_buttons = [false; 2];
    //GetTrackedDeviceIndexForControllerRole
    let mut time = time::Instant::now();
    loop {
        let mut poses: [_; 3] = Default::default();
        vr.get_device_to_absolute_tracking_pose(&mut poses);
        let controller_states = [vr.get_controller_state(1), vr.get_controller_state(2)];

        {
            let mut model = model.lock().unwrap();
            if model.is_finished {
                break;
            }
            for i in 0..2 {
                let next_button = controller_states[i].button_pressed
                    & (openvr::BUTTON_MASK_GRIP | openvr::BUTTON_MASK_TRIGGER)
                    != 0;
                if next_button {
                    let hand = poses[i + 1].device_to_absolute_tracking.to_nalgebra();
                    let head = poses[0].device_to_absolute_tracking.to_nalgebra();
                    model.projector[i].feed(&hand, &head);
                }
                if (prev_buttons[i], next_button) == (true, false) {
                    let stroke = model.projector[i].stroke();
                    model.strokes.push_back(stroke);
                    model.projector[i].clear();
                }
                prev_buttons[i] = next_button;
            }
        }

        ctx.request_repaint();

        let t0 = mem::replace(&mut time, time::Instant::now());
        thread::sleep(t0 + time::Duration::from_secs(1) / 90 - time);
    }
}

fn main() {
    let model = sync::Arc::new(sync::Mutex::new(Model {
        is_finished: false,
        error: None,
        projector: [
            graffiti_3d::StrokeProjector::new(),
            graffiti_3d::StrokeProjector::new(),
        ],
        strokes: collections::VecDeque::new(),
    }));
    let vr_thread = rc::Rc::new(cell::Cell::new(None));
    eframe::run_native(
        "GraffitiVR",
        eframe::NativeOptions::default(),
        Box::new({
            let model = model.clone();
            let vr_thread = vr_thread.clone();
            move |cc| {
                vr_thread.set(Some(thread::spawn({
                    let model = model.clone();
                    let ctx = cc.egui_ctx.clone();
                    move || vr_thread_proc(model, ctx)
                })));
                Box::new(App::new(model))
            }
        }),
    );
    model.lock().unwrap().is_finished = true;
    vr_thread.take().unwrap().join().unwrap();
}
