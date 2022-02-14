use std::io::Read;
use std::path::PathBuf;
use anyhow::Context;
use crate::core::renderer::WgpuContext;
use egui::{ClippedMesh, Color32, RawInput, RichText, WidgetText, Window};
use crate::config::ConfigManager;
use crate::core::gui::theme::ThemeManager;

use crate::core::window::OverlayWindow;

pub mod window;
mod config;
mod theme;

pub struct Etx {
    pub ctx: egui::Context,
    pub enabled_windows: WindowEnabled,
    pub theme_manager: ThemeManager,
}

impl Etx {
    pub fn new(_ow: &OverlayWindow, theme_folder_path: PathBuf, default_theme_name: &str, fonts_dir: PathBuf) -> anyhow::Result<Self> {
        let ctx = egui::Context::default();
        let enabled_windows = WindowEnabled {
            config_window: false,
            theme_window: false,
            debug_window: false,
            marker_pack_window: false
        };
        let prop_fonts_dir = fonts_dir.join("proportional");
        let mono_fonts_dir = fonts_dir.join("monospace");
        std::fs::create_dir_all(&mono_fonts_dir).context("failed to create monospace fonts dir")?;
        std::fs::create_dir_all(&prop_fonts_dir).context("failed to create proportional fonts dir")?;
        let theme_manager = ThemeManager::new(theme_folder_path, default_theme_name).context("failed to create theme manager")?;
        let mut font_definitions = egui::FontDefinitions::default();
        for f in std::fs::read_dir(&prop_fonts_dir).context("failed to read prop fonts directory entries")? {
            let font_path = f.context("failed to get dir entry of prop fonts dir")?.path();
            let mut font_bytes = vec![];
            std::fs::File::open(&font_path).context("failed to open prop font file")?.read_to_end(&mut font_bytes).context("failed to read font file into Vec")?;
            let font_data = egui::FontData::from_owned(font_bytes);
            let name = font_path.file_stem().context("failed to get file stem of prop font file")?.to_string_lossy().to_string();
            font_definitions.font_data.entry(name.clone()).or_insert(font_data);
            font_definitions.families.entry(egui::FontFamily::Proportional).or_default().insert(0, name);
        }
        for f in std::fs::read_dir(&mono_fonts_dir).context("failed to read mono fonts directory entries")? {
            let font_path = f.context("failed to get dir entry of mono fonts dir")?.path();
            let mut font_bytes = vec![];
            std::fs::File::open(&font_path).context("failed to open mono font file")?.read_to_end(&mut font_bytes).context("failed to read font file into Vec")?;
            let font_data = egui::FontData::from_owned(font_bytes);
            let name = font_path.file_stem().context("failed to get file stem of mono font file")?.to_string_lossy().to_string();
            font_definitions.font_data.entry(name.clone()).or_insert(font_data);
            font_definitions.families.entry(egui::FontFamily::Monospace).or_default().insert(0, name);
        }
        ctx.set_fonts(font_definitions);
        ctx.set_style(theme_manager.list_of_themes.iter().find(|&t| t.name == default_theme_name).context("failed to find default theme")?.style.clone());
        Ok(Self { ctx, enabled_windows, theme_manager })
    }
    pub fn tick(
        &mut self,
        input: RawInput,
        ow: &mut OverlayWindow,
        wtx: &mut WgpuContext,
        cm: &mut ConfigManager,
        handle: tokio::runtime::Handle
    ) -> anyhow::Result<(egui::Output, Vec<ClippedMesh>)> {
         self.ctx.begin_frame(input);
         {
             let ctx = self.ctx.clone();
             egui::containers::Area::new("top menu container")
                 .show(&ctx, |ui| {
                     ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgba_unmultiplied(0, 0, 0, 100);
                    let joko_icon_title =  WidgetText::RichText(RichText::from("Joko\u{1F451}")).strong().text_style(egui::TextStyle::Heading);
                    ui.menu_button(joko_icon_title, |ui| {
                            ui.checkbox(&mut self.enabled_windows.config_window, "show config window");
                        ui.checkbox(&mut self.enabled_windows.debug_window, "show debug window");
                        ui.checkbox(&mut self.enabled_windows.theme_window, "show theme window");
                        ui.checkbox(&mut self.enabled_windows.marker_pack_window, "show marker pack manager");
                    });
                 });
                 Window::new("style editor")
                     .open(&mut self.enabled_windows.theme_window)
                     .show(&ctx, |ui| {

                         ctx.style_ui(ui);
                     });
            ow.gui(ctx.clone(), wtx).unwrap();
            cm.gui(ctx, handle).unwrap();
        }
        let (output, shapes) =  self.ctx.end_frame();
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
}