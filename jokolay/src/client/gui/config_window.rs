use std::{path::PathBuf, sync::Arc};

use egui::{CollapsingHeader, CtxRef, DragValue, Pos2};
use parking_lot::RwLock;

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
    pub config: Arc<RwLock<JokoConfig>>,
    pub config_path: PathBuf,
    pub cursor_position: Pos2,
    pub average_fps: u16,
}

impl ConfigWindow {
    pub fn new(config: Arc<RwLock<JokoConfig>>, config_path: PathBuf) -> Self {
        Self {
            name: "Configuration Window",
            config,
            cursor_position: Pos2::default(),
            average_fps: 0,
            config_path,
        }
    }
    pub fn tick(&mut self, ctx: CtxRef, cfc: &mut CoreFrameCommands, new_frame_rate: u16) {
        self.average_fps = new_frame_rate;
        let mut joko_config = self.config.write();
        // let mut changed = false;
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
                         ui
                            .add(DragValue::new(&mut joko_config.overlay_window_config.vsync))
                            .on_hover_text("vsync frame interval. 0 = unlimited fps. 1 = match refresh rate of monitor (i.e.vsync). 2 = present frame every 2 refreshes...");
                    }
                    // transparent
                    ui.checkbox(&mut joko_config.overlay_window_config.transparency, "transparent")
                    .on_hover_text("only works when starting jokolay. can't change while running");
                    // decorations
                    ui.checkbox(&mut joko_config.overlay_window_config.decorated, "decorations");
                    // passthrough
                    ui.checkbox(&mut joko_config.overlay_window_config.passthrough, "passthrough");
                    // always on top
                    ui.checkbox(&mut joko_config.overlay_window_config.always_on_top, "On Top");
                    // window size
                    {
                        ui.label("window width");
                        ui.add(egui::widgets::DragValue::new(&mut joko_config.overlay_window_config.framebuffer_width));
                        ui.label("window height");
                        ui.add(egui::widgets::DragValue::new(&mut joko_config.overlay_window_config.framebuffer_height));
                    }
                    // window position
                    {
                        ui.label("window position x");
                        ui.add(egui::widgets::DragValue::new(&mut joko_config.overlay_window_config.window_pos_x));
                        ui.label("window pos y");
                        ui.add(egui::widgets::DragValue::new(&mut joko_config.overlay_window_config.window_pos_y));
                    }

                    if ui.button("apply config").clicked() {
                        cfc.window_commads
                            .push(WindowCommand::ApplyConfig);
                    }

                    if ui.button("reset window").clicked() {
                        joko_config.overlay_window_config = OverlayWindowConfig::default();
                        cfc.window_commads
                            .push(WindowCommand::ApplyConfig);
                    }
                });
                CollapsingHeader::new("Guild Wars 2 Settings").show(ui, |ui| {
                    // mumble link string
                    ui.text_edit_singleline(&mut joko_config.mumble_config.link_name);
                });
                CollapsingHeader::new("Input Settings").show(ui, |ui| {
                    {
                        ui.label("scroll power");
                        ui.add(DragValue::new(&mut joko_config.input_config.scroll_power).clamp_range(1.0..=50.0)).on_hover_text(
                            "how much to scroll when you turn the mouse scroll wheel."
                        );
                    }
                });

                egui::CollapsingHeader::new("style settings").show(ui, |ui| {
                    ctx.style_ui(ui);
                });
                if ui.button("save config to file").clicked() {
                    joko_config.style = (*ctx.style()).clone();
                    let jc = joko_config.clone();
                    let path = self.config_path.clone();
                    std::thread::spawn(move || {
                        jc.save_config(&path).unwrap();
                    });
                }
            });
    }
}
