use crate::core::window::OverlayWindow;

impl OverlayWindow {
    pub fn gui(&mut self, ctx: egui::Context) -> anyhow::Result<()> {
        egui::Window::new("Window State").scroll2([true, true]).show(&ctx, |ui| {
            // ui.label("fps: ");
            // ui.add(egui::widgets::DragValue::new(&mut self.window_state.average_frame_rate));
            ui.label(&format!("{:#?}", &self.window_state));
        });
        Ok(())
    }
}
