mod openvr;
mod three;
use eframe::egui;
use std::*;

type Vector2 = nalgebra::Vector2<f32>;
type Vector3 = nalgebra::Vector3<f32>;
type Vector4 = nalgebra::Vector4<f32>;

#[derive(Default)]
struct Model {
    is_finished: bool,
    error: Option<String>,
    current_stroke: [Vec<Vector3>; 2],
    current_direction: [Vector3; 2],
    strokes: collections::VecDeque<(Vector3, Vec<Vector3>)>,
}

struct App {
    model: sync::Arc<sync::Mutex<Model>>,
    recognizer: graffiti::GraffitiRecognizer,
    text: Vec<char>,
    cursor: usize,
}

fn v2_invert_y(v: Vector2) -> Vector2 {
    Vector2::new(v[0], -v[1])
}

impl App {
    fn new(model: sync::Arc<sync::Mutex<Model>>) -> Self {
        App {
            model: model,
            recognizer: graffiti::GraffitiRecognizer::new(0.02),
            text: Vec::new(),
            cursor: 0,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        // XXX
        let error;
        let current_stroke;
        let current_direction;
        let stroke;
        {
            let mut model = self.model.lock().unwrap();
            error = model.error.clone();
            current_stroke = model.current_stroke.clone();
            current_direction = model.current_direction.clone();
            stroke = model.strokes.pop_front();
        }

        let up = Vector3::new(0.0, 1.0, 0.0);

        if let Some((dir, stroke3)) = stroke {
            let stroke2 = three::project_to_plane(&stroke3, up, dir);
            let stroke2: Vec<_> = stroke2.iter().map(|v| [v[0], v[1]]).collect();
            match self.recognizer.recognize(&stroke2) {
                Some('\x08') => {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                        self.text.remove(self.cursor);
                    }
                }
                Some('←') => {
                    self.cursor = cmp::max(self.cursor - 1, 0);
                }
                Some('→') => {
                    self.cursor = cmp::min(self.cursor + 1, self.text.len());
                }
                Some(c) => {
                    self.text.insert(self.cursor, c);
                    self.cursor += 1;
                }
                None => (),
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(err) = error {
                ui.label(err);
            }

            let indicator = match self.recognizer.modifier() {
                graffiti::GraffitiModifier::Symbol => ".",
                graffiti::GraffitiModifier::Caps => "^",
                graffiti::GraffitiModifier::None => match self.recognizer.mode() {
                    graffiti::GraffitiMode::Number => "#",
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

            for i in 0..2 {
                if current_stroke[i].len() < 2 {
                    continue;
                }
                let stroke2 = three::project_to_plane(&current_stroke[i], up, current_direction[i]);

                let mut s_min = Vector2::repeat(f32::INFINITY);
                let mut s_max = Vector2::repeat(-f32::INFINITY);
                for v in stroke2.iter() {
                    let v = v2_invert_y(*v);
                    s_min = s_min.inf(&v);
                    s_max = s_max.sup(&v);
                }
                let scale = (r_max - r_min).component_div(&(s_max - s_min)).min();
                let offset = 0.5 * ((r_max + r_min) - scale * (s_max + s_min));

                let stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));
                for i in 0..stroke2.len() - 1 {
                    let v0 = scale * v2_invert_y(stroke2[i + 0]) + offset;
                    let v1 = scale * v2_invert_y(stroke2[i + 1]) + offset;
                    painter.line_segment(
                        [egui::Pos2::new(v0[0], v0[1]), egui::Pos2::new(v1[0], v1[1])],
                        stroke,
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
                    let m = poses[i + 1].device_to_absolute_tracking.to_nalgebra();
                    let pos = m * Vector4::new(0.0, 0.0, 0.0, 1.0);
                    let dir = m * Vector4::new(0.0, 0.0, 1.0, 0.0);
                    model.current_direction[i] += dir;
                    model.current_stroke[i].push(pos);
                }
                if (prev_buttons[i], next_button) == (true, false) {
                    let dir = mem::replace(&mut model.current_direction[i], Vector3::zeros());
                    let stroke = mem::replace(&mut model.current_stroke[i], Vec::new());
                    model.strokes.push_back((dir, stroke));
                }
                prev_buttons[i] = next_button;
            }
        }

        ctx.request_repaint();
        thread::sleep(time::Duration::from_secs(1) / 90);
    }
}

fn main() {
    let model = sync::Arc::new(sync::Mutex::new(Default::default()));
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
