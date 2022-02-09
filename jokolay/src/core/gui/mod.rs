use crate::core::renderer::WgpuContext;
use egui::{ClippedMesh, RawInput};

use crate::core::window::OverlayWindow;

pub mod window;
pub struct Etx {
    pub ctx: egui::Context,
}

impl Etx {
    pub fn new(_ow: &OverlayWindow) -> anyhow::Result<Self> {
        let ctx = egui::Context::default();

        Ok(Self { ctx })
    }
    pub fn tick(
        &mut self,
        input: RawInput,
        ow: &mut OverlayWindow,
        wtx: &mut WgpuContext,
    ) -> anyhow::Result<(egui::Output, Vec<ClippedMesh>)> {
        let (output, shapes) = self.ctx.run(input, |ctx| {
            // Window::new("hello").show(ctx, |ui| {
            //     ui.label("hello");
            // });
            ow.gui(ctx.clone(), wtx).unwrap();
            // egui::Window::new("settings")
            //     .scroll2([true, true])
            //     .show(ctx, |ui| {
            //         ctx.inspection_ui(ui);
            //         ctx.settings_ui(ui);
            //     });
            // self.app.ui(&self.ctx);
        });
        let shapes = self.ctx.tessellate(shapes);
        Ok((output, shapes))
    }
}
