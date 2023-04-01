// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod chatbox;
mod egui_texture;
mod model;
mod openvr;
#[cfg(not(windows))]
mod osdep;
#[cfg(windows)]
mod osdep_win;
mod ui;
mod vr_input;
use eframe::{egui, glow};
#[cfg(windows)]
use osdep_win as osdep;
use std::*;

struct App {
    interval: time::Duration,
    time: time::Instant,
    model: model::Model,
    openvr: openvr::OpenVr,
    vr_input: vr_input::VrInput,
    ui: ui::Ui,
    overlay_texture: egui_texture::EguiTexture,
    overlay_handle: u64,
    chatbox: Option<chatbox::ChatBox>,
}

impl App {
    fn new(cc: &eframe::CreationContext, name: &[u8]) -> Self {
        let openvr = openvr::OpenVr::new(openvr::ApplicationType::Overlay).unwrap();

        let overlay_texture =
            egui_texture::EguiTexture::new(cc.gl.as_ref().unwrap().clone(), &[512, 128]);

        let overlay_handle = openvr.create_overlay(name, name);
        openvr.set_overlay_flag(overlay_handle, openvr::OVERLAY_FLAGS_IS_PREMULTIPLIED, true);
        openvr.set_overlay_width_in_meters(overlay_handle, 1.0);
        let m = openvr::HmdMatrix34::from_nalgebra(&nalgebra::Matrix3x4::new(
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, -2.0, //
        ));
        openvr.set_overlay_transform_tracked_device_relative(overlay_handle, 0, &m);

        App {
            interval: time::Duration::from_secs(1) / 90,
            time: time::Instant::now(),
            model: model::Model::new(),
            openvr: openvr,
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
        osdep::sleep(self.interval.saturating_sub(self.time.elapsed()));
        self.time = time::Instant::now();

        self.vr_input.update(&self.openvr, &mut self.model);

        if self.model.is_active {
            self.overlay_texture
                .run(|ctx| self.ui.overlay(ctx, &mut self.model));
            self.openvr.set_overlay_texture(
                self.overlay_handle,
                &openvr::Texture {
                    handle: self.overlay_texture.texture().0.get() as usize,
                    type_: openvr::TextureType::OpenGL,
                    color_space: openvr::ColorSpace::Auto,
                },
            );
            self.openvr.show_overlay(self.overlay_handle);
        } else {
            self.openvr.hide_overlay(self.overlay_handle);
        }

        self.ui.main(ctx, &mut self.model);

        if self.model.is_active {
            if self.model.use_key_emulation {
                if let Some(c) = self.model.new_char {
                    osdep::emulate_key(c);
                }
            }
            if self.model.use_chatbox {
                if let Some(ref mut chatbox) = self.chatbox {
                    chatbox.input(format!("{}{}", self.model.text_l(), self.model.text_r()));
                    chatbox.typing(self.model.current_strokes.iter().any(|s| s.len() > 0));
                    chatbox.update();
                }
            }
        }

        self.model.new_char = None;
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _: Option<&glow::Context>) {
        self.openvr.destroy_overlay(self.overlay_handle);
        self.overlay_texture.destroy();
        // XXX: self.openvr
    }
}

fn main() -> eframe::Result<()> {
    let mut opt = eframe::NativeOptions::default();
    opt.vsync = false;
    let result = eframe::run_native(
        "mimizu",
        opt,
        Box::new(move |cc| Box::new(App::new(cc, b"mimizu\0"))),
    );

    result
}
