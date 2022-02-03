use egui::{ClippedMesh, RawInput};

use crate::core::window::OverlayWindow;

pub mod window;
pub struct Etx {
    pub ctx: egui::Context,
    pub app: egui_demo_lib::DemoWindows,
}

impl Etx {
    pub fn new(ow: &OverlayWindow) -> anyhow::Result<Self> {
        let ctx = egui::Context::default();
        ctx.input_mut().screen_rect = egui::Rect::from_two_pos(
            [0.0, 0.0].into(),
            [
                ow.window_state.framebuffer_size.x as f32,
                ow.window_state.framebuffer_size.y as f32,
            ]
            .into(),
        );
        ctx.input_mut().pixels_per_point = 2.0;
        let app = egui_demo_lib::DemoWindows::default();

        Ok(Self { ctx, app })
    }
    pub fn tick(
        &mut self,
        input: RawInput,
        ow: &mut OverlayWindow,
    ) -> anyhow::Result<(egui::Output, Vec<ClippedMesh>)> {
        let (output, shapes) = self.ctx.run(input, |ctx| {
            ow.gui(ctx.clone()).unwrap();
            egui::Window::new("settings").scroll2([true, true]).show(ctx, |ui| {
                ctx.inspection_ui(ui);
                ctx.settings_ui(ui);
            });
            // self.app.ui(&self.ctx);

        });
        let shapes = self.ctx.tessellate(shapes);
        Ok((output, shapes))
    }
}
