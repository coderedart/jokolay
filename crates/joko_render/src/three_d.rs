use egui_backend::GfxBackend;
use egui_render_three_d::*;
// use three_d::*;

pub struct JokoRenderer {
    three_d_backend: ThreeDBackend,
}

impl GfxBackend for JokoRenderer {
    type Configuration = <ThreeDBackend as GfxBackend>::Configuration;

    fn new(
        window_backend: &mut impl egui_backend::WindowBackend,
        config: Self::Configuration,
    ) -> Self {
        Self {
            three_d_backend: ThreeDBackend::new(window_backend, config),
        }
    }

    fn prepare_frame(&mut self, window_backend: &mut impl egui_backend::WindowBackend) {
        self.three_d_backend.prepare_frame(window_backend);
    }

    fn render_egui(
        &mut self,
        meshes: Vec<egui_backend::egui::ClippedPrimitive>,
        textures_delta: egui_backend::egui::TexturesDelta,
        logical_screen_size: [f32; 2],
    ) {
        self.three_d_backend
            .render_egui(meshes, textures_delta, logical_screen_size);
    }

    fn present(&mut self, window_backend: &mut impl egui_backend::WindowBackend) {
        self.three_d_backend.present(window_backend);
    }

    fn resize_framebuffer(&mut self, window_backend: &mut impl egui_backend::WindowBackend) {
        self.three_d_backend.resize_framebuffer(window_backend);
    }
}
