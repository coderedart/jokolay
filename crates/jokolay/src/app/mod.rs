use cap_std::fs_utf8::Dir;
use egui::DragValue;
use egui_backend::{egui, BackendConfig, GfxBackend, UserApp, WindowBackend};
use egui_window_glfw_passthrough::{GlfwBackend, GlfwConfig};
mod frame;
mod init;
mod theme;
mod trace;
use self::theme::ThemeManager;
use init::get_jokolay_dir;
use jmf::manager::MarkerManager;
use joko_render::JokoRenderer;
use jokolink::{MumbleChanges, MumbleManager};
use miette::{Context, Result};
use trace::JokolayTracingLayer;
use tracing::{error, info};

#[allow(unused)]
pub struct Jokolay {
    show_window: ShowWindowStatus,
    frame_stats: frame::FrameStatistics,
    jdir: Dir,
    mumble_manager: MumbleManager,
    marker_manager: MarkerManager,
    theme_manager: ThemeManager,
    joko_renderer: JokoRenderer,
    egui_context: egui::Context,
    glfw_backend: GlfwBackend,
}

#[allow(unused)]
#[derive(Debug, Default)]
struct ShowWindowStatus {
    traces: bool,
}
impl Jokolay {
    fn new(jdir: Dir) -> Result<Self> {
        let mumble =
            MumbleManager::new("MumbleLink", None).wrap_err("failed to create mumble manager")?;
        let marker_manager =
            MarkerManager::new(&jdir).wrap_err("failed to create marker manager")?;
        let mut theme_manager =
            ThemeManager::new(&jdir).wrap_err("failed to create theme manager")?;
        let egui_context = egui::Context::default();
        theme_manager.init_egui(&egui_context);
        let mut glfw_backend = GlfwBackend::new(
            GlfwConfig {
                glfw_callback: Box::new(|glfw_context| {
                    glfw_context.window_hint(
                        egui_window_glfw_passthrough::glfw::WindowHint::SRgbCapable(true),
                    );
                    glfw_context.window_hint(
                        egui_window_glfw_passthrough::glfw::WindowHint::Floating(true),
                    );
                }),
                ..Default::default()
            },
            BackendConfig {
                transparent: Some(true),
                is_opengl: false,
                ..Default::default()
            },
        );

        let joko_renderer = JokoRenderer::new(&mut glfw_backend, Default::default());
        // remove decorations
        glfw_backend.window.set_decorated(false);
        Ok(Self {
            mumble_manager: mumble,
            marker_manager,
            frame_stats: frame::FrameStatistics::new(glfw_backend.glfw.get_time() as _),
            joko_renderer,
            glfw_backend,
            jdir,
            egui_context,
            show_window: Default::default(),
            theme_manager,
        })
    }
}
impl UserApp for Jokolay {
    fn gui_run(&mut self) {
        // most of the fn contents are in Self::run fn instead.
        // As we need some custom input filtering (to match scale of gw2 UI or custom scaling)
    }

    type UserGfxBackend = JokoRenderer;

    type UserWindowBackend = GlfwBackend;

    fn get_all(
        &mut self,
    ) -> (
        &mut Self::UserWindowBackend,
        &mut Self::UserGfxBackend,
        &egui::Context,
    ) {
        (
            &mut self.glfw_backend,
            &mut self.joko_renderer,
            &self.egui_context,
        )
    }

    fn run(
        &mut self,
        logical_size: [f32; 2],
    ) -> Option<(egui::PlatformOutput, std::time::Duration)> {
        let Self {
            mumble_manager,
            marker_manager,
            joko_renderer,
            egui_context,
            glfw_backend,
            frame_stats,
            ..
        } = self;
        let egui_context = egui_context.clone();
        // don't bother doing anything if there's no window
        if let Some(full_output) = if glfw_backend.get_window().is_some() {
            let input = glfw_backend.take_raw_input();
            joko_renderer.prepare_frame(glfw_backend);
            egui_context.begin_frame(input);
            let latest_time = glfw_backend.glfw.get_time();
            frame_stats.tick(latest_time);
            let link = match mumble_manager.tick(&egui_context) {
                Ok(ml) => ml,
                Err(e) => {
                    error!(?e, "mumble manager tick error");
                    None
                }
            };

            egui_context.request_repaint();
            let cursor_position = egui_context.pointer_latest_pos();

            egui::Window::new("egui window")
                .default_width(300.0)
                .show(&egui_context, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("cursor pos");
                        let mut cursor_position = cursor_position.unwrap_or_default();
                        ui.add(DragValue::new(&mut cursor_position.x).fixed_decimals(1));
                        ui.add(DragValue::new(&mut cursor_position.y).fixed_decimals(1));
                    });
                    let mut is_passthrough = glfw_backend.window.is_mouse_passthrough();
                    ui.checkbox(&mut is_passthrough, "passthrough");
                });
            marker_manager.tick(&egui_context, latest_time, joko_renderer, &link);

            // check if we need to change window position or size.
            if let Some(link) = link.as_ref() {
                joko_renderer.update_from_mumble_link(link);
                if link.changes.contains(MumbleChanges::WindowPosition)
                    || link.changes.contains(MumbleChanges::WindowSize)
                {
                    info!(
                        ?link.client_pos, ?link.client_size,
                        "resizing/repositioning to match gw2 window dimensions"
                    );
                    // to account for the invisible border shadows thingy. IDK if these pixel values are the same across all dpi/monitors
                    glfw_backend
                        .window
                        .set_pos(link.client_pos.x, link.client_pos.y);
                    glfw_backend
                        .window
                        .set_size(link.client_size.x, link.client_size.y);
                }
            }
            JokolayTracingLayer::show_notifications(&egui_context);
            // if it doesn't require either keyboard or pointer, set passthrough to true
            glfw_backend.window.set_mouse_passthrough(
                !(egui_context.wants_keyboard_input() || egui_context.wants_pointer_input()),
            );
            Some(egui_context.end_frame())
        } else {
            None
        } {
            let egui::FullOutput {
                platform_output,
                repaint_after,
                textures_delta,
                shapes,
            } = full_output;
            let (wb, gb, egui_context) = self.get_all();
            let egui_context = egui_context.clone();

            gb.render_egui(
                egui_context.tessellate(shapes),
                textures_delta,
                logical_size,
            );
            gb.present(wb);
            return Some((platform_output, repaint_after));
        }
        None
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

    match Jokolay::new(jdir) {
        Ok(jokolay) => {
            <Jokolay as UserApp>::UserWindowBackend::run_event_loop(jokolay);
        }
        Err(e) => {
            error!(?e, "failed to create Jokolay App");
        }
    };
    std::mem::drop(log_file_flush_guard);
}
