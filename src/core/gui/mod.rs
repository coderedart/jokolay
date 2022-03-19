use crate::config::ConfigManager;
use crate::core::gui::theme::ThemeManager;
use crate::core::marker::MarkerManager;
use crate::core::renderer::WgpuContext;
use crate::core::window::OverlayWindow;
use cached::TimedCache;
use color_eyre::eyre::WrapErr;
use egui::{ClippedPrimitive, Color32, RawInput, RichText, TextureHandle, WidgetText, Window};
use jokolink::MumbleCtx;
use std::path::PathBuf;

mod config;
pub mod marker;
mod theme;
pub mod window;

pub struct Etx {
    pub ctx: egui::Context,
    pub enabled_windows: WindowEnabled,
    pub theme_manager: ThemeManager,
    pub hash_textures: TimedCache<u64, TextureHandle>,
}

impl Etx {
    pub fn new(
        _ow: &OverlayWindow,
        theme_folder_path: PathBuf,
        default_theme_name: &str,
        fonts_dir: PathBuf,
    ) -> color_eyre::Result<Self> {
        let ctx = egui::Context::default();
        let enabled_windows = WindowEnabled::default();
        let theme_manager = ThemeManager::new(theme_folder_path, fonts_dir, default_theme_name)
            .wrap_err("failed to create theme manager")?;

        ctx.set_fonts(theme_manager.font_definitions.clone());
        ctx.set_style(theme_manager.get_current_theme()?.style.clone());
        let hash_textures = TimedCache::with_lifespan_and_refresh(300, true);
        Ok(Self {
            ctx,
            enabled_windows,
            theme_manager,
            hash_textures,
        })
    }
    pub async fn tick(
        &mut self,
        input: RawInput,
        ow: &mut OverlayWindow,
        wtx: &mut WgpuContext,
        cm: &mut ConfigManager,
        mctx: &mut MumbleCtx,
        mm: &mut MarkerManager,
        // handle: tokio::runtime::Handle,
    ) -> color_eyre::Result<(
        egui::PlatformOutput,
        egui::TexturesDelta,
        Vec<ClippedPrimitive>,
    )> {
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
                    ui.checkbox(
                        &mut self.enabled_windows.overlay_controls,
                        "show overlay controls",
                    );
                    ui.checkbox(&mut self.enabled_windows.debug_window, "show debug window");
                    ui.checkbox(
                        &mut self.enabled_windows.marker_pack_window,
                        "show marker pack manager",
                    );
                    ui.checkbox(
                        &mut self.enabled_windows.mumble_window,
                        "show mumble window",
                    );
                });
            });

            self.theme_manager
                .gui(ctx.clone(), &mut self.enabled_windows.theme_window)?;
            ow.gui(ctx.clone(), &mut self.enabled_windows.overlay_controls, wtx)?;
            cm.gui(ctx.clone(), &mut self.enabled_windows.config_window)?;
            self.marker_gui(mm, mctx).await?;
            Window::new("Mumble Window")
                .open(&mut self.enabled_windows.mumble_window)
                .scroll2([true, true])
                .show(&ctx, |ui| {
                    ui.set_width(300.0);

                    ui.horizontal(|ui| {
                        ui.label("mumble link name: ");
                        ui.label(&mctx.config.link_name);
                    });
                    ui.label("time since the last change");
                    ui.label(&format!(
                        "uitick change: {:.1}",
                        ow.window_state.glfw_time - mctx.src.last_uitick_update
                    ));
                    ui.label(&format!(
                        "dimensions change: {:.1}",
                        ow.window_state.glfw_time - mctx.src.last_pos_size_update
                    ));
                    ui.label(&format!("gw2 pid: {}", mctx.src.gw2_pid));
                    ui.label(&format!("gw2 xid: {}", mctx.src.gw2_window_handle));
                    ui.label(&format!("gw2 position: {:#?}", mctx.src.gw2_pos));
                    ui.label(&format!("gw2 size: {:#?}", mctx.src.gw2_size));
                    ui.collapsing("mumble link data", |ui| {
                        ui.label(&format!("{:#?}", mctx.src.get_link()));
                    });
                });
        }
        let egui::FullOutput {
            platform_output,
            needs_repaint: _,
            textures_delta,
            shapes,
        } = self.ctx.end_frame();
        let shapes = self.ctx.tessellate(shapes);
        Ok((platform_output, textures_delta, shapes))
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct WindowEnabled {
    pub config_window: bool,
    pub theme_window: bool,
    pub debug_window: bool,
    pub marker_pack_window: bool,
    pub overlay_controls: bool,
    pub mumble_window: bool,
}
