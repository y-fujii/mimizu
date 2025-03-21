// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use eframe::{egui, egui_glow, glow, glow::HasContext};
use std::*;

pub struct EguiTexture {
    size: [u32; 2],
    context: egui::Context,
    painter: egui_glow::Painter,
    texture: glow::Texture,
    framebuffer: glow::Framebuffer,
}

impl EguiTexture {
    pub fn new(gl: sync::Arc<glow::Context>, size: &[u32; 2]) -> Self {
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
                glow::PixelUnpackData::Slice(None),
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

        EguiTexture {
            size: *size,
            context: egui::Context::default(),
            painter: egui_glow::Painter::new(gl, "", None, false).unwrap(),
            texture: tex,
            framebuffer: fb,
        }
    }

    pub fn run(&mut self, run_ui: impl FnMut(&egui::Context)) {
        let ppp = self.context.pixels_per_point();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Default::default(),
                egui::Vec2::new(self.size[0] as f32, self.size[1] as f32) / ppp,
            )),
            ..Default::default()
        };
        let out = self.context.run(input, run_ui);
        let prims = self.context.tessellate(out.shapes, ppp);

        unsafe {
            let gl = self.painter.gl();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        self.painter
            .paint_and_update_textures(self.size, ppp, &prims, &out.textures_delta);
        unsafe { self.painter.gl().bind_framebuffer(glow::FRAMEBUFFER, None) };
    }

    pub fn context(&self) -> &egui::Context {
        &self.context
    }

    pub fn texture(&self) -> glow::Texture {
        self.texture
    }

    pub fn destroy(&mut self) {
        self.painter.destroy();
        unsafe {
            let gl = self.painter.gl();
            gl.delete_framebuffer(self.framebuffer);
            gl.delete_texture(self.texture);
        }
    }
}
