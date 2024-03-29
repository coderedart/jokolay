use std::sync::Arc;

use cap_std::fs_utf8::Dir;
use egui_window_glfw_passthrough::{glfw::Context as _, GlfwBackend, GlfwConfig};
mod init;
mod wm;
use init::get_jokolay_dir;
use jmf::MarkerManager;
use joko_core::manager::{theme::ThemeManager, trace::JokolayTracingLayer};
use joko_render::JokoRenderer;
use jokolink::{MumbleChanges, MumbleManager};
use miette::{Context, Result};
use tracing::{error, info};
#[allow(unused)]
pub struct Jokolay {
    frame_stats: wm::WindowStatistics,
    jdir: Arc<Dir>,
    menu_panel: MenuPanel,
    mumble_manager: MumbleManager,
    marker_manager: MarkerManager,
    theme_manager: ThemeManager,
    joko_renderer: JokoRenderer,
    egui_context: egui::Context,
    glfw_backend: GlfwBackend,
}
impl Jokolay {
    pub fn new(jdir: Arc<Dir>) -> Result<Self> {
        let mumble =
            MumbleManager::new("MumbleLink", None).wrap_err("failed to create mumble manager")?;
        let marker_manager =
            MarkerManager::new(&jdir).wrap_err("failed to create marker manager")?;
        let mut theme_manager =
            ThemeManager::new(&jdir).wrap_err("failed to create theme manager")?;
        let egui_context = egui::Context::default();
        theme_manager.init_egui(&egui_context);
        let mut glfw_backend = GlfwBackend::new(GlfwConfig {
            glfw_callback: Box::new(|glfw_context| {
                glfw_context.window_hint(
                    egui_window_glfw_passthrough::glfw::WindowHint::SRgbCapable(true),
                );
                glfw_context.window_hint(egui_window_glfw_passthrough::glfw::WindowHint::Floating(
                    true,
                ));
                glfw_context.window_hint(
                    egui_window_glfw_passthrough::glfw::WindowHint::ContextVersion(4, 6),
                );
            }),
            opengl_window: Some(true),
            transparent_window: Some(true),
            window_title: "Jokolay".to_string(),
            ..Default::default()
        });
        glfw_backend.window.set_floating(true);
        glfw_backend.window.set_decorated(false);
        let joko_renderer = JokoRenderer::new(&mut glfw_backend, Default::default());
        Ok(Self {
            mumble_manager: mumble,
            marker_manager,
            frame_stats: wm::WindowStatistics::new(glfw_backend.glfw.get_time() as _),
            joko_renderer,
            glfw_backend,
            jdir,
            egui_context,
            theme_manager,
            menu_panel: MenuPanel::default(),
        })
    }
    pub fn enter_event_loop(mut self) {
        tracing::info!("entering glfw event loop");
        self.menu_panel.show_theme_window = true;
        self.menu_panel.show_marker_manager_window = true;
        loop {
            let Self {
                frame_stats,
                jdir: _,
                menu_panel,
                mumble_manager,
                marker_manager,
                theme_manager,
                joko_renderer,
                egui_context,
                glfw_backend,
            } = &mut self;
            let etx = egui_context.clone();

            // gather events
            glfw_backend.glfw.poll_events();
            glfw_backend.tick();

            if glfw_backend.window.should_close() {
                tracing::warn!("should close is true. So, exiting event loop");
                break;
            }

            if glfw_backend.resized_event_pending {
                let latest_size = glfw_backend.window.get_framebuffer_size();
                let latest_size = [latest_size.0 as _, latest_size.1 as _];

                glfw_backend.framebuffer_size_physical = latest_size;
                glfw_backend.window_size_logical = [
                    latest_size[0] as f32 / glfw_backend.scale,
                    latest_size[1] as f32 / glfw_backend.scale,
                ];
                joko_renderer.resize_framebuffer(latest_size);
                glfw_backend.resized_event_pending = false;
            }
            joko_renderer.prepare_frame(|| {
                let latest_size = glfw_backend.window.get_framebuffer_size();
                tracing::info!(
                    ?latest_size,
                    "failed to get surface texture, so calling latest framebuffer size"
                );
                let latest_size = [latest_size.0 as _, latest_size.1 as _];
                glfw_backend.framebuffer_size_physical = latest_size;
                glfw_backend.window_size_logical = [
                    latest_size[0] as f32 / glfw_backend.scale,
                    latest_size[1] as f32 / glfw_backend.scale,
                ];
                latest_size
            });

            let latest_time = glfw_backend.glfw.get_time();
            let mut input = glfw_backend.take_raw_input();
            input.time = Some(latest_time);

            etx.begin_frame(input);
            // do all the non-gui stuff first
            frame_stats.tick(latest_time);
            let link = match mumble_manager.tick() {
                Ok(ml) => ml,
                Err(e) => {
                    error!(?e, "mumble manager tick error");
                    None
                }
            };
            joko_renderer.tick(link.clone());
            marker_manager.tick(&etx, latest_time, joko_renderer, &link);
            menu_panel.tick(&etx, link.clone().as_ref().map(|m| m.as_ref()));

            // do the gui stuff now
            egui::Area::new("menu panel")
                .fixed_pos(menu_panel.pos)
                .interactable(true)
                .order(egui::Order::Foreground)
                .show(&etx, |ui| {
                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                        egui::Color32::TRANSPARENT;
                    ui.horizontal(|ui| {
                        ui.menu_button(
                            egui::RichText::new("JKL")
                                .size((MenuPanel::HEIGHT - 2.0) * menu_panel.ui_scaling_factor)
                                .background_color(egui::Color32::TRANSPARENT),
                            |ui| {
                                ui.checkbox(
                                    &mut menu_panel.show_window_manager,
                                    "Show Window Manager",
                                );
                                ui.checkbox(
                                    &mut menu_panel.show_marker_manager_window,
                                    "Show Marker Manager",
                                );
                                ui.checkbox(
                                    &mut menu_panel.show_mumble_manager_winodw,
                                    "Show Mumble Manager",
                                );
                                ui.checkbox(
                                    &mut menu_panel.show_theme_window,
                                    "Show Theme Manager",
                                );
                                ui.checkbox(&mut menu_panel.show_tracing_window, "Show Logs");
                                if ui.button("exit").clicked() {
                                    info!("exiting jokolay");
                                    glfw_backend.window.set_should_close(true);
                                }
                            },
                        );
                        marker_manager.menu_ui(ui);
                    });
                });
            marker_manager.gui(&etx, &mut menu_panel.show_marker_manager_window);
            mumble_manager.gui(&etx, &mut menu_panel.show_mumble_manager_winodw);
            JokolayTracingLayer::gui(&etx, &mut menu_panel.show_tracing_window);
            theme_manager.gui(&etx, &mut menu_panel.show_theme_window);
            frame_stats.gui(&etx, glfw_backend, &mut menu_panel.show_window_manager);
            // show notifications
            JokolayTracingLayer::show_notifications(&etx);

            // end gui stuff
            // check if we need to change window position or size.
            if let Some(link) = link.as_ref() {
                if link.changes.contains(MumbleChanges::WindowPosition)
                    || link.changes.contains(MumbleChanges::WindowSize)
                {
                    info!(
                        ?link.client_pos, ?link.client_size,
                        "resizing/repositioning to match gw2 window dimensions"
                    );

                    glfw_backend
                        .window
                        .set_pos(link.client_pos.x, link.client_pos.y);
                    // if gw2 is in windowed fullscreen mode, then the size is full resolution of the screen/monitor.
                    // But if we set that size, when you focus jokolay, the screen goes blank on win11 (some kind of fullscreen optimization maybe?)
                    // so we remove a pixel from right/bottom edges. mostly indistinguishable, but makes sure that transparency works even in windowed fullscrene mode of gw2
                    glfw_backend
                        .window
                        .set_size(link.client_size.x - 1, link.client_size.y - 1);
                }
            }
            etx.request_repaint();

            let egui::FullOutput {
                platform_output,
                textures_delta,
                shapes,
                ..
            } = etx.end_frame();

            if !platform_output.copied_text.is_empty() {
                glfw_backend
                    .window
                    .set_clipboard_string(&platform_output.copied_text);
            }

            // if it doesn't require either keyboard or pointer, set passthrough to true
            glfw_backend
                .window
                .set_mouse_passthrough(!(etx.wants_keyboard_input() || etx.wants_pointer_input()));
            joko_renderer.render_egui(
                etx.tessellate(shapes),
                textures_delta,
                glfw_backend.window_size_logical,
            );
            joko_renderer.present();
            glfw_backend.window.swap_buffers();
        }
    }
}

pub fn start_jokolay() {
    let jdir = match get_jokolay_dir() {
        Ok(jdir) => jdir,
        Err(e) => {
            eprintln!("failed to create jokolay dir: {e:#?}");
            panic!("failed to create jokolay_dir: {e:#?}");
        }
    };
    let log_file_flush_guard = match JokolayTracingLayer::install_tracing(&jdir) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("failed to install tracing: {e:#?}");
            panic!("failed to install tracing: {e:#?}");
        }
    };

    if let Err(e) = rayon::ThreadPoolBuilder::default()
        .panic_handler(|panic_info| {
            error!(?panic_info, "rayon thread paniced.");
        })
        .build_global()
    {
        error!(
            ?e,
            "failed to set panic handler and build global threadpool for rayon"
        );
    }

    match Jokolay::new(jdir.into()) {
        Ok(jokolay) => {
            jokolay.enter_event_loop();
        }
        Err(e) => {
            error!(?e, "failed to create Jokolay App");
        }
    };
    std::mem::drop(log_file_flush_guard);
}
/// Guild Wars 2 has an array of menu icons on top left corner of the game.
/// Its size is affected by four different factors
/// 1. UISZ:
///     This is a setting in graphics options of gw2 and it comes in 4 variants
///     small, normal, large and larger.
///     This is something we can get from mumblelink's context.
/// 2. DPI scaling
///     This is a setting in graphics options too. When scaling is enabled, sizes of menu become bigger according to the dpi of gw2 window
///     This is something we get from gw2's config file in AppData/Roaming and store in mumble link as dpi scaling
///     We also get dpi of gw2 window and store it in mumble link.
/// 3. Dimensions of the gw2 window
///     This is something we get from mumble link and win32 api. We store this as client pos/size in mumble link
///     It is not just the width or height, but their ratio to the 1024x768 resolution

///
/// 1. By default, with dpi 96 (scale 1.0), at resolution 1024x768 these are the sizes of menu at different uisz settings
///     UISZ   -> WIDTH   HEIGHT
///     small  -> 288     27
///     normal -> 319     31
///     large  -> 355     34
///     larger -> 391     38
///     all units are in raw pixels.
///     
///     If we think of small uisz as the default. Then, we can express the rest of the sizes as ratio to small.
///     small = 1.0
///     normal = 1.1
///     large = 1.23
///     larger = 1.35
///     
///     So, just multiply small (288) with these ratios to get the actual pixels of each uisz.
/// 2. When dpi doubles, so do the sizes. 288 -> 576, 319 -> 638 etc.. So, when dpi scaling is enabled, we must multiply the above uisz ratio with dpi scale ratio to get the combined scaling ratio.
/// 3. The dimensions thing is a little complicated. So, i will just list the actual steps here.
///     1. take gw2's actual width in raw pixels. lets call this gw2_width.
///     2. take 1024 as reference minimum width. If dpi scaling is enabled, multiply 1024 * dpi scaling ratio. lets call this reference_width.
///     3. Now, get the smaller value out of the two. lets call this minimum_width.
///     4. finally, do (minimum_width / reference_width) to get "width scaling ratio".
///     5. repeat steps 1 - 4, but for height. use 768 as the reference width (with approapriate dpi scaling).
///     6. now just take the minimum of "width scaling ratio" and "height scaling ratio". lets call this "aspect ratio scaling".
///
/// Finally, just multiply the width 288 or height 27 with these three values.
/// eg: menu width = 288 * uisz_ratio * dpi_scaling_ratio * aspect_ratio_scaling;
/// do the same with 288 replaced by 27 for height.
#[derive(Debug, Default)]
pub struct MenuPanel {
    pub pos: egui::Pos2,
    pub ui_scaling_factor: f32,
    show_tracing_window: bool,
    show_theme_window: bool,
    // show_settings_window: bool,
    show_marker_manager_window: bool,
    show_mumble_manager_winodw: bool,
    show_window_manager: bool,
}

impl MenuPanel {
    pub const WIDTH: f32 = 288.0;
    pub const HEIGHT: f32 = 27.0;
    pub fn tick(&mut self, etx: &egui::Context, link: Option<&jokolink::MumbleLink>) {
        let mut ui_scaling_factor = 1.0;
        if let Some(link) = link.as_ref() {
            let gw2_scale: f32 = if link.dpi_scaling == 1 || link.dpi_scaling == -1 {
                (if link.dpi == 0 { 96.0 } else { link.dpi as f32 }) / 96.0
            } else {
                1.0
            };

            ui_scaling_factor *= gw2_scale;
            let uisz_scale = convert_uisz_to_scale(link.uisz);
            ui_scaling_factor *= uisz_scale;

            let min_width = 1024.0 * gw2_scale;
            let min_height = 768.0 * gw2_scale;
            let gw2_width = link.client_size.x as f32;
            let gw2_height = link.client_size.y as f32;
            let min_width_ratio = min_width.min(gw2_width) / min_width;
            let min_height_ratio = min_height.min(gw2_height) / min_height;

            let min_ratio = min_height_ratio.min(min_width_ratio);
            ui_scaling_factor *= min_ratio;

            let egui_scale = etx.pixels_per_point();
            ui_scaling_factor /= egui_scale;
        }

        self.pos.x = ui_scaling_factor * (Self::WIDTH + 8.0); // add 8 pixels padding just for some space
        self.ui_scaling_factor = ui_scaling_factor;
    }
}

fn convert_uisz_to_scale(uisize: jokolink::UISize) -> f32 {
    const SMALL: f32 = 288.0;
    const NORMAL: f32 = 319.0;
    const LARGE: f32 = 355.0;
    const LARGER: f32 = 391.0;
    const SMALL_SCALING_RATIO: f32 = 1.0;
    const NORMAL_SCALING_RATIO: f32 = NORMAL / SMALL;
    const LARGE_SCALING_RATIO: f32 = LARGE / SMALL;
    const LARGER_SCALING_RATIO: f32 = LARGER / SMALL;
    match uisize {
        jokolink::UISize::Small => SMALL_SCALING_RATIO,
        jokolink::UISize::Normal => NORMAL_SCALING_RATIO,
        jokolink::UISize::Large => LARGE_SCALING_RATIO,
        jokolink::UISize::Larger => LARGER_SCALING_RATIO,
    }
}
/*
Just some random measurements to verify in the future (or write tests for :))
with dpi enabled, there's some math involved it seems.
Linux ->
width 1920 pixels. height 2113 pixels. ratio 0.91. fov 1.01. scaling 2.0. dpi enabled
small  -> 540     53
normal -> 599     59
large  -> 667     65
larger -> 734     72


Windows ->
width 1920 pixels. height 2113 pixels. ratio 0.91. fov 1.01. scaling 2.0. dpi enabled.
small  -> 540     53
normal -> 599     59
large  -> 667     65
larger -> 734     72

width 1914 pixels. height 2072 pixels. ratio 0.92. fov 1.01. scaling 3.0. dpi enabled. dpi 288
small  -> 538     52
normal -> 598     58
large  -> 665     65
larger -> 731     72

width 3840. height 2160. ratio 1.78. scaling 3. dpi true. dpi 288 (windowed fullscreen)
small  -> 810     80
normal -> 900     89
large  -> 1000    99
larger -> 1100    109

width 1916 pixels. height 2113 pixels. ratio 0.91. fov 1.01. scaling 1.5. dpi enabled. dpi 144
small  -> 432     42
normal -> 480     47
large  -> 533     52
larger -> 586     57

width 1000 pixels. height 1000 pixels. ratio 1. fov 1.01. scaling 2.0. dpi enabled.
small  -> 281     26
normal -> 312     29
large  -> 347     33
larger -> 382     36

width 2000 pixels. height 1000 pixels. ratio 2. fov 1.01. scaling 2.0. dpi enabled.
small  -> 375     36
normal -> 416     40
large  -> 463     45
larger -> 509     49

width 2000 pixels. height 2000 pixels. ratio 1. fov 1.01. scaling 2.0. dpi enabled.
small  -> 562     55
normal -> 624     61
large  -> 694     68
larger -> 764     75


*/
