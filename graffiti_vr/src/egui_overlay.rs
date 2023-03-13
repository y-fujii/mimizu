use crate::*;
use eframe::egui;
use eframe::egui_glow;
use eframe::glow;
use eframe::glow::HasContext;
use std::*;

pub struct EguiOverlay {
    pub size: [u32; 2],
    pub context: egui::Context,
    pub painter: egui_glow::Painter,
    pub texture: glow::Texture,
    pub framebuffer: glow::Framebuffer,
    pub overlay: openvr::Overlay,
    pub handle: usize,
}

impl Drop for EguiOverlay {
    fn drop(&mut self) {
        // XXX
        self.overlay.destroy(self.handle);
        unsafe {
            let gl = self.painter.gl();
            gl.delete_framebuffer(self.framebuffer);
            gl.delete_texture(self.texture);
        }
    }
}

impl EguiOverlay {
    pub fn new(gl: sync::Arc<glow::Context>, size: &[u32; 2], name: &[u8]) -> Self {
        let tex;
        unsafe {
            tex = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::SRGB8_ALPHA8 as i32,
                size[0] as i32,
                size[1] as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }

        let fb;
        unsafe {
            fb = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(tex),
                0,
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }

        let overlay = openvr::Overlay::new();
        let handle = overlay.create(name, name);
        overlay.set_flag(handle, openvr::OVERLAY_FLAGS_PREMULTIPLIED, true);

        // XXX
        overlay.set_width_in_meters(handle, 1.0);
        let m = openvr::HmdMatrix34::from_nalgebra(&nalgebra::Matrix3x4::new(
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, -2.0, //
        ));
        overlay.set_transform_tracked_device_relative(handle, 0, &m);
        overlay.show(handle);

        EguiOverlay {
            size: *size,
            context: egui::Context::default(),
            painter: egui_glow::Painter::new(gl, "", None).unwrap(),
            texture: tex,
            framebuffer: fb,
            overlay: overlay,
            handle: handle,
        }
    }

    pub fn run(&mut self, run_ui: impl FnOnce(&egui::Context)) {
        let ppp = self.context.pixels_per_point();
        let mut input = egui::RawInput::default();
        input.screen_rect = Some(egui::Rect::from_min_size(
            Default::default(),
            egui::Vec2::new(self.size[0] as f32, self.size[1] as f32) / ppp,
        ));
        let out = self.context.run(input, run_ui);
        let prims = self.context.tessellate(out.shapes);

        unsafe {
            let gl = self.painter.gl();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        self.painter
            .paint_and_update_textures(self.size, ppp, &prims, &out.textures_delta);
        unsafe { self.painter.gl().bind_framebuffer(glow::FRAMEBUFFER, None) };

        self.overlay
            .set_texture(self.handle, self.texture.0.get() as usize);
    }
}
