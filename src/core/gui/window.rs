use crate::core::window::OverlayWindow;
use crate::WgpuContext;
use egui::{CollapsingHeader, DragValue, Widget};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

impl OverlayWindow {
    pub fn gui(
        &mut self,
        ctx: egui::Context,
        open: &mut bool,
        wtx: WgpuContext,
    ) -> color_eyre::Result<()> {
        let window_state = self.window_state.read();
        egui::Window::new("Window Controls")
            .open(open)
            .scroll2([true, true])
            .show(&ctx, |ui| {
                ui.set_width(300.0);
                ui.horizontal(|ui| {
                    ui.label("fps: ");
                    let mut fps = window_state.average_frame_rate;
                    DragValue::new(&mut fps).ui(ui);
                });
                ui.label(&format!(
                    "cursor position: x: {} , y: {}",
                    window_state.cursor_position.x, window_state.cursor_position.y
                ));
                ui.label(&format!(
                    "scale level: x: {} y: {}",
                    window_state.scale.x, window_state.scale.y
                ));

                if ui.button("toggle decorations").clicked() {
                    self.window.set_decorated(!self.window.is_decorated());
                }
                let mut size = window_state.size;

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

                let mut position = window_state.position;
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
                // let mut wtx = wtx.write();
                // if ui
                //     .radio_value(
                //         &mut wtx.config.present_mode,
                //         wgpu::PresentMode::Immediate,
                //         "unlimited fps",
                //     )
                //     .changed()
                //     || ui
                //         .radio_value(
                //             &mut wtx.config.present_mode,
                //             wgpu::PresentMode::Fifo,
                //             "fps limited to vsync",
                //         )
                //         .changed()
                // {
                //     wtx.surface.configure(&wtx.device, &wtx.config);
                // }
                CollapsingHeader::new("latest local events").show(ui, |ui| {
                    for event in window_state.latest_local_events.asc_iter().rev() {
                        ui.label(&format!("{event:?}"));
                    }
                });
                ui.label(&format!("uptime: {:.1}", window_state.glfw_time));
            });
        Ok(())
    }
}
