use crate::config::ConfigManager;
use crate::core::gui::theme::ThemeManager;
use crate::core::renderer::WgpuContext;
use anyhow::Context;
use egui::{ClippedMesh, Color32, RawInput, RichText, WidgetText};
use std::path::PathBuf;

use crate::core::window::OverlayWindow;

mod config;
mod theme;
pub mod window;

pub struct Etx {
    pub ctx: egui::Context,
    pub enabled_windows: WindowEnabled,
    pub theme_manager: ThemeManager,
}

impl Etx {
    pub fn new(
        _ow: &OverlayWindow,
        theme_folder_path: PathBuf,
        default_theme_name: &str,
        fonts_dir: PathBuf,
    ) -> anyhow::Result<Self> {
        let ctx = egui::Context::default();
        let enabled_windows = WindowEnabled::default();
        let theme_manager = ThemeManager::new(theme_folder_path, fonts_dir, default_theme_name)
            .context("failed to create theme manager")?;

        ctx.set_fonts(theme_manager.font_definitions.clone());
        ctx.set_style(theme_manager.get_current_theme()?.style.clone());
        Ok(Self {
            ctx,
            enabled_windows,
            theme_manager,
        })
    }
    pub fn tick(
        &mut self,
        input: RawInput,
        ow: &mut OverlayWindow,
        wtx: &mut WgpuContext,
        cm: &mut ConfigManager,
        handle: tokio::runtime::Handle,
    ) -> anyhow::Result<(egui::Output, Vec<ClippedMesh>)> {
        self.ctx.begin_frame(input);
        {
            let ctx = self.ctx.clone();
            egui::containers::Area::new("top menu container").show(&ctx, |ui| {
                ui.style_mut().visuals.widgets.inactive.bg_fill =
                    Color32::from_rgba_unmultiplied(0, 0, 0, 100);
                let joko_icon_title = WidgetText::RichText(RichText::from("Joko\u{1F451}"))
                    .strong()
                    .text_style(egui::TextStyle::Heading);
                ui.menu_button(joko_icon_title, |ui| {
                    ui.checkbox(
                        &mut self.enabled_windows.config_window,
                        "show config window",
                    );
                    ui.checkbox(&mut self.enabled_windows.theme_window, "show theme window");
                    ui.checkbox(&mut self.enabled_windows.overlay_controls, "show overlay controls");
                    ui.checkbox(&mut self.enabled_windows.debug_window, "show debug window");
                    ui.checkbox(
                        &mut self.enabled_windows.marker_pack_window,
                        "show marker pack manager",
                    );
                });
            });
            if self.enabled_windows.theme_window {
                self.theme_manager.gui(ctx.clone())?;
            }
            if self.enabled_windows.overlay_controls {
                ow.gui(ctx.clone(), wtx)?;
            }
            if self.enabled_windows.config_window {
                cm.gui(ctx, handle)?;
            }
        }
        let (output, shapes) = self.ctx.end_frame();
        let shapes = self.ctx.tessellate(shapes);
        Ok((output, shapes))
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct WindowEnabled {
    pub config_window: bool,
    pub theme_window: bool,
    pub debug_window: bool,
    pub marker_pack_window: bool,
    pub overlay_controls: bool,
}
