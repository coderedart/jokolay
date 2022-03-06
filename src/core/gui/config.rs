use crate::config::{ConfigManager, VsyncMode};
use color_eyre::eyre::WrapErr;
use egui::{DragValue, Widget, Window};

impl ConfigManager {
    pub fn gui(&mut self, ctx: egui::Context, open: &mut bool) -> color_eyre::Result<()> {
        Window::new("Configuration")
            .open(open)
            .scroll2([true, true])
            .show(&ctx, |ui| {
                // Overlay Window Settings
                ui.label("default window size");
                DragValue::new(&mut self.config.overlay_window_config.size.x)
                    .clamp_range::<u32>(100..=4500)
                    .ui(ui);
                DragValue::new(&mut self.config.overlay_window_config.size.y)
                    .clamp_range::<u32>(100..=4500)
                    .ui(ui);
                ui.label("default window position");
                DragValue::new(&mut self.config.overlay_window_config.position.x)
                    .clamp_range::<i32>(0..=i32::MAX)
                    .ui(ui);
                DragValue::new(&mut self.config.overlay_window_config.position.y)
                    .clamp_range::<i32>(0..=i32::MAX)
                    .ui(ui);

                ui.radio_value(
                    &mut self.config.overlay_window_config.vsync,
                    VsyncMode::Immediate,
                    "unlimited fps",
                );
                ui.radio_value(
                    &mut self.config.overlay_window_config.vsync,
                    VsyncMode::Fifo,
                    "fps limited to vsync",
                );

                // Mumble Config
                ui.label("Mumble Link Name");
                ui.text_edit_singleline(&mut self.config.mumble_config.link_name);

                // Input Scroll Power
                ui.label("scroll power");
                DragValue::new(&mut self.config.input_config.scroll_power)
                    .clamp_range::<f32>(10.0..=200.0)
                    .ui(ui);
                // auto attach to gw2 window
                ui.label("auto attach to gw2 window");
                ui.checkbox(
                    &mut self.config.auto_attach_to_gw2,
                    "attach to gw2 when we find the window",
                );
                // log level
                ui.label("log level: trace, debug, info, warn, error.");
                ui.text_edit_singleline(&mut self.config.log_level);

                // theme name
                ui.label("default theme name");
                ui.text_edit_singleline(&mut self.config.theme_name);

                if ui.button("save configuration to file").clicked() {
                    self.needs_save = true;
                }
            });
        if self.needs_save {
            self.save_config().wrap_err("failed to save config file")?;
            self.needs_save = false;
        }
        Ok(())
    }
}
