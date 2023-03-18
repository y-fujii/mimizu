#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod chatbox;
mod egui_texture;
mod model;
mod openvr;
mod ui;
mod vr_input;
use eframe::{egui, glow};
use std::*;

struct App {
    interval: time::Duration,
    time: time::Instant,
    model: model::Model,
    system: openvr::System,
    overlay: openvr::Overlay,
    vr_input: vr_input::VrInput,
    ui: ui::Ui,
    overlay_texture: egui_texture::EguiTexture,
    overlay_handle: usize,
    chatbox: Option<chatbox::ChatBox>,
}

fn sleep_high_res(d: time::Duration) {
    #[cfg(target_os = "windows")]
    {
        extern "C" {
            fn sleep_100ns(_: i64) -> bool;
        }
        let t = (d.as_nanos() / 100).try_into().unwrap();
        let r = unsafe { sleep_100ns(t) };
        assert!(r);
    }
    #[cfg(not(target_os = "windows"))]
    thread::sleep(d);
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        let system = openvr::System::new();
        let overlay = openvr::Overlay::new();

        let overlay_texture =
            egui_texture::EguiTexture::new(cc.gl.as_ref().unwrap().clone(), &[512, 512]);

        let overlay_handle = overlay.create(b"TegakiVR\0", b"TegakiVR\0");
        overlay.set_flag(overlay_handle, openvr::OVERLAY_FLAGS_PREMULTIPLIED, true);
        overlay.set_width_in_meters(overlay_handle, 1.0);
        let m = openvr::HmdMatrix34::from_nalgebra(&nalgebra::Matrix3x4::new(
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, -2.0, //
        ));
        overlay.set_transform_tracked_device_relative(overlay_handle, 0, &m);
        overlay.show(overlay_handle);

        App {
            interval: time::Duration::from_secs(1) / 90,
            time: time::Instant::now(),
            model: model::Model::new(),
            system: system,
            overlay: overlay,
            vr_input: vr_input::VrInput::new(),
            ui: ui::Ui::new(&cc.egui_ctx, overlay_texture.context()),
            overlay_texture: overlay_texture,
            overlay_handle: overlay_handle,
            chatbox: chatbox::ChatBox::new().ok(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        sleep_high_res(self.interval.saturating_sub(self.time.elapsed()));
        self.time = time::Instant::now();

        self.vr_input.update(&self.system);
        if self.model.is_active {
            self.model.current_strokes = self.vr_input.current_strokes();
            if let Some(stroke) = self.vr_input.pop_stroke() {
                self.model.feed_stroke(&stroke);
            }
        } else {
            self.model.current_strokes = [Vec::new(), Vec::new()];
        }

        self.overlay_texture
            .run(|ctx| self.ui.overlay(ctx, &mut self.model));
        self.overlay.set_texture(
            self.overlay_handle,
            self.overlay_texture.texture().0.get() as usize,
        );
        self.ui.main(ctx, &mut self.model);

        if self.model.use_chatbox {
            if let Some(ref mut chatbox) = self.chatbox {
                chatbox.input(format!("{}{}", self.model.text_l(), self.model.text_r()));
                chatbox.typing(self.model.current_strokes.iter().any(|s| s.len() > 0));
                chatbox.update();
            }
        }

        ctx.request_repaint();
    }

    fn on_exit(&mut self, _: Option<&glow::Context>) {
        self.overlay_texture.destroy();
        self.overlay.destroy(self.overlay_handle);
    }
}

fn main() -> eframe::Result<()> {
    assert!(openvr::init());

    let mut opt = eframe::NativeOptions::default();
    opt.vsync = false;
    let result = eframe::run_native(
        "TegakiVR",
        opt,
        Box::new(move |cc| Box::new(App::new(cc))),
    );

    openvr::shutdown();
    result
}
