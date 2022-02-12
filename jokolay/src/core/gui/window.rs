use crate::core::renderer::WgpuContext;
use crate::core::window::OverlayWindow;
use egui::{CollapsingHeader, DragValue, Widget};

impl OverlayWindow {
    pub fn gui(&mut self, ctx: egui::Context, wtx: &mut WgpuContext) -> anyhow::Result<()> {
        egui::Window::new("Window Controls")
            .scroll2([true, true])
            .show(&ctx, |ui| {
                ui.set_width(300.0);
                ui.horizontal(|ui| {
                    ui.label("fps: ");
                    let mut fps = self.window_state.average_frame_rate;
                    DragValue::new(&mut fps).ui(ui);
                });
                ui.label(&format!(
                    "cursor position: x: {} , y: {}",
                    self.window_state.cursor_position.x, self.window_state.cursor_position.y
                ));
                ui.label(&format!(
                    "scale level: x: {} y: {}",
                    self.window_state.scale.x, self.window_state.scale.y
                ));

                if ui.button("toggle decorations").clicked() {
                    self.window.set_decorated(!self.window.is_decorated());
                }
                let mut size = self.window_state.size;

                // minimum 100 so that users don't accidentally go to zero size
                // maximum to a reasonable 4k ish size
                if DragValue::new(&mut size.x)
                    .clamp_range::<u32>(100..=4500)
                    .ui(ui)
                    .changed()
                    || DragValue::new(&mut size.y)
                        .clamp_range::<u32>(100..=4500)
                        .ui(ui)
                        .changed()
                {
                    self.window.set_size(size.x as i32, size.y as i32)
                }

                let mut position = self.window_state.position;
                // minimum 0 to avoid window being pushed out of screen and maximum can't be restricted due to
                // multi monitor setups having large widths/heights
                if DragValue::new(&mut position.x)
                    .clamp_range::<i32>(0..=i32::MAX)
                    .ui(ui)
                    .changed()
                    || DragValue::new(&mut position.y)
                        .clamp_range::<i32>(0..=i32::MAX)
                        .ui(ui)
                        .changed()
                {
                    self.window.set_pos(position.x, position.y)
                }
                if ui
                    .radio_value(
                        &mut wtx.config.present_mode,
                        wgpu::PresentMode::Immediate,
                        "unlimited fps",
                    )
                    .changed()
                    || ui
                        .radio_value(
                            &mut wtx.config.present_mode,
                            wgpu::PresentMode::Fifo,
                            "fps limited to vsync",
                        )
                        .changed()
                {
                    wtx.surface.configure(&wtx.device, &wtx.config);
                }
                CollapsingHeader::new("latest local events").show(ui, |ui| {
                    for event in self.window_state.latest_local_events.asc_iter().rev() {
                        ui.label(&format!("{event:?}"));
                    }
                });
                ui.label(&format!("uptime: {:.1}", self.window_state.glfw_time));
            });
        Ok(())
    }
}
