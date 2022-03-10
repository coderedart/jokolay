use crate::core::gui::Etx;
use crate::core::marker::MarkerManager;
use egui::Window;
use jokolink::MumbleCtx;

impl Etx {
    pub fn marker_gui(
        &mut self,
        _mm: &mut MarkerManager,
        _mctx: &MumbleCtx,
    ) -> color_eyre::Result<()> {
        Window::new("Marker Manager").show(&self.ctx, |ui| {
            ui.label("something");
        });
        Ok(())
    }
}
