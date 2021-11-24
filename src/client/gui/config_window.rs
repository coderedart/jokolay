use egui::{CollapsingHeader, CtxRef, Pos2};

use crate::{
    config::JokoConfig,
    core::{
        window::{OverlayWindowConfig, WindowCommand},
        CoreFrameCommands,
    },
};

#[derive(Debug)]
pub struct ConfigWindow {
    pub name: &'static str,
    pub config: JokoConfig,
    pub cursor_position: Pos2,
    pub average_fps: u16,
}

impl ConfigWindow {
    pub fn new(config: JokoConfig) -> Self {
        Self {
            name: "Configuration Window",
            config,
            cursor_position: Pos2::default(),
            average_fps: 0,
        }
    }
    pub fn tick(&mut self, ctx: CtxRef, cfc: &mut CoreFrameCommands, new_frame_rate: u16) {
        let mut needs_repaint = false;
        if new_frame_rate != self.average_fps {
            self.average_fps = new_frame_rate;
            needs_repaint = true;
        }
        egui::Window::new(self.name)
            .scroll2([true, true])
            .show(&ctx, |ui| {
                egui::CollapsingHeader::new("Overlay Window Settings").show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("average fps: ");
                        ui.label(self.average_fps);
                    });
                    // Vsync
                    {
                        let mut vsync_enabled = self.config.overlay_window_config.vsync.is_some();
                        if ui
                            .checkbox(&mut vsync_enabled, "vsync to limit fps")
                            .on_hover_text("if enabled, will limit fps to the refresh rate. if disabled (e.g. to check performance), removes fps limit")
                            .changed()
                        {
                            self.config.overlay_window_config.vsync =
                                if vsync_enabled { Some(1) } else { None };
                                needs_repaint = true;
                        }
                    }
                    // transparent
                    ui.checkbox(&mut self.config.overlay_window_config.transparency, "transparent").on_hover_text("only works when starting jokolay. can't change while running");
                    // decorations
                    ui.checkbox(&mut self.config.overlay_window_config.decorated, "decorations");
                    // passthrough
                    ui.checkbox(&mut self.config.overlay_window_config.passthrough, "passthrough");
                    // always on top
                    ui.checkbox(&mut self.config.overlay_window_config.always_on_top, "On Top");
                    // window size
                    {
                        ui.add(egui::widgets::DragValue::new(&mut self.config.overlay_window_config.framebuffer_width));
                        ui.add(egui::widgets::DragValue::new(&mut self.config.overlay_window_config.framebuffer_height));
                    }
                    // window position
                    {
                        ui.add(egui::widgets::DragValue::new(&mut self.config.overlay_window_config.window_pos_x));
                        ui.add(egui::widgets::DragValue::new(&mut self.config.overlay_window_config.window_pos_y));
                    }

                    if ui.button("apply config").clicked() {
                        cfc.window_commads
                            .push(WindowCommand::ConfigSync(self.config.clone()));
                    }

                    if ui.button("reset").clicked() {
                        self.config.overlay_window_config = OverlayWindowConfig::default();
                    }
                });
                CollapsingHeader::new("Guild Wars 2 Settings").show(ui, |ui| {
                    // mumble link string
                    ui.text_edit_singleline(&mut self.config.mumble_config.link_name);
                });
            });
        if needs_repaint {
            ctx.request_repaint();
        }
    }
}
