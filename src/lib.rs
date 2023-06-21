pub mod jmf;
pub mod joko_script;
pub mod jokoapi;
pub mod jokolink;

use egui_backend::{
    egui::{self, Grid, Ui},
    raw_window_handle::HasRawWindowHandle,
    BackendConfig, GfxBackend, UserApp, WindowBackend,
};

use egui_window_glfw_passthrough::{GlfwBackend, GlfwConfig};

use jmf::manager::MarkerManager;
use joko_render::{JokoRenderer, MARKER_MAX_VISIBILITY_DISTANCE};
use jokolink::{MumbleManager, WindowDimensions};
use tracing::{info, warn};

pub struct Jokolay {
    pub last_check: f64,
    pub fps: u32,
    pub frame_count: u64,
    pub frame_reset_seconds_timestamp: u64,
    pub mumble: Option<MumbleManager>,
    pub marker_manager: Option<MarkerManager>,
    pub window_dimensions: WindowDimensions,
    pub joko_renderer: JokoRenderer,
    pub egui_context: egui::Context,
    pub window_backend: GlfwBackend,
}

impl Jokolay {
    fn new(mut window_backend: GlfwBackend, joko_renderer: JokoRenderer) -> Self {
        let mumble = MumbleManager::new("MumbleLink", window_backend.window.raw_window_handle())
            .map_err(|e| {
                warn!("error creating Mumble Manager: {}", e);
            })
            .map(|mut mumble| {
                match mumble.get_latest_window_dimensions() {
                    Ok(wd) => {
                        let (x, y) = window_backend.window.get_pos();
                        let (width, height) = window_backend.window.get_size();
                        if x != wd.x
                            || y != wd.y
                            || width as u32 != wd.width
                            || height as u32 != wd.height
                        {
                            window_backend.window.set_pos(wd.x, wd.y);
                            window_backend
                                .window
                                .set_size(wd.width as i32, wd.height as i32);
                        }
                    }
                    Err(e) => tracing::error!("failed ot get window dimensions: {e}"),
                }
                mumble
            })
            .ok();
        let marker_manager = MarkerManager::new(std::path::Path::new("./assets/packs"))
            .map_err(|e| {
                warn!("error creating Marker Manager: {}", e);
            })
            .ok();

        Self {
            mumble,
            marker_manager,
            last_check: 0.0,
            joko_renderer,
            egui_context: Default::default(),
            frame_count: 0,
            frame_reset_seconds_timestamp: 0,
            fps: 0,
            window_dimensions: WindowDimensions::default(),
            window_backend,
        }
    }
}
impl UserApp for Jokolay {
    fn gui_run(&mut self) {
        let Self {
            last_check,
            fps,
            frame_count,
            frame_reset_seconds_timestamp,
            mumble,
            marker_manager,
            window_dimensions,
            joko_renderer,
            egui_context,
            window_backend,
        } = self;
        // for ev in window_backend.frame_events.iter() {
        //     match ev {
        //         egui_window_glfw_passthrough::glfw::WindowEvent::Focus(_)
        //         | egui_window_glfw_passthrough::glfw::WindowEvent::Key(_, _, _, _) => {
        //             dbg!(ev);
        //         }
        //         _ => {}
        //     }
        // }
        let latest_time = window_backend.glfw.get_time();
        *frame_count += 1;
        if latest_time - *frame_reset_seconds_timestamp as f64 > 1.0 {
            *fps = *frame_count as u32;
            *frame_count = 0;
            *frame_reset_seconds_timestamp = latest_time as u64;
        }
        if let Some(mumble) = mumble {
            let _ = mumble.tick();
        }
        egui_context.request_repaint();
        let cursor_position = egui_context.pointer_latest_pos();
        egui::Window::new("egui window")
            .default_width(300.0)
            .show(egui_context, |ui| {
                ui.label(&format!("cursor position: {cursor_position:?}"));
                ui.label(&format!("fps: {}", *fps));
                let mut is_passthrough = window_backend.window.is_mouse_passthrough();
                ui.checkbox(&mut is_passthrough, "is window passthrough?");
                if let Some(mumble) = mumble {
                    if let Some(link) = mumble.get_mumble_link() {
                        mumble_ui(ui, link);
                    }
                }
                ui.label(format!(
                    "number of markers drawn: {}",
                    joko_renderer.markers.len()
                ));
            });
        if let Some(marker_manager) = marker_manager {
            marker_manager.tick(egui_context, latest_time);
        }
        if let Some(mumble) = mumble {
            if let Some(link) = mumble.get_mumble_link() {
                if let Some(marker_manager) = marker_manager {
                    marker_manager.render(link.context.map_id as u16, joko_renderer);
                }
                let ratio = window_backend.framebuffer_size_physical[0] as f32
                    / window_backend.framebuffer_size_physical[1] as f32;
                let center = link.f_camera_position + link.f_camera_front;
                let v = glam::Mat4::look_at_lh(
                    link.f_camera_position,
                    center,
                    glam::vec3(0.0, 1.0, 0.0),
                );
                let p = glam::Mat4::perspective_lh(
                    link.identity.fov,
                    ratio,
                    1.0,
                    MARKER_MAX_VISIBILITY_DISTANCE,
                );
                joko_renderer.set_mvp(p * v);
                joko_renderer.camera_position = link.f_camera_position;
                joko_renderer.mvp = p * v;
                joko_renderer.player_position = link.f_avatar_position;
            }
        }
        // if it doesn't require either keyboard or pointer, set passthrough to true
        window_backend.window.set_mouse_passthrough(
            !(egui_context.wants_keyboard_input() || egui_context.wants_pointer_input()),
        );
        if latest_time - *last_check > 10. {
            *last_check = latest_time;
            if let Some(mumble) = mumble {
                if let Ok(wd) = mumble.get_latest_window_dimensions() {
                    *window_dimensions = wd;
                    let (x, y) = window_backend.window.get_pos();
                    let (width, height) = window_backend.window.get_size();
                    if x != wd.x
                        || y != wd.y
                        || width as u32 != wd.width
                        || height as u32 != wd.height
                    {
                        info!("resizing/repositioning our window from {x},{y},{width},{height} to match gw2 window dimensions: {wd:?}");
                        window_backend.window.set_pos(wd.x, wd.y);
                        window_backend
                            .window
                            .set_size(wd.width as i32, wd.height as i32);
                    }
                }
            }
        }
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
            &mut self.window_backend,
            &mut self.joko_renderer,
            &self.egui_context,
        )
    }
}

pub fn start_jokolay() {
    let mut glfw_backend = GlfwBackend::new(
        GlfwConfig {
            glfw_callback: Box::new(|glfw_context| {
                glfw_context.window_hint(
                    egui_window_glfw_passthrough::glfw::WindowHint::SRgbCapable(true),
                );
                glfw_context.window_hint(egui_window_glfw_passthrough::glfw::WindowHint::Floating(
                    true,
                ));
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
    let jokolay = Jokolay::new(glfw_backend, joko_renderer);
    <Jokolay as UserApp>::UserWindowBackend::run_event_loop(jokolay);
}

fn mumble_ui(ui: &mut Ui, link: &jokolink::MumbleLink) {
    Grid::new("link grid").num_columns(2).show(ui, |ui| {
        ui.label("ui tick: ");
        ui.label(format!("{}", link.ui_tick));
        ui.end_row();
        ui.label("character: ");
        ui.label(&link.identity.name);
        ui.end_row();
        ui.label("map: ");
        ui.label(format!("{}", link.context.map_id));
        ui.end_row();
        ui.label(format!("pid: {}", link.context.process_id));
        // ui.label("player position");
        // ui.label(format!(
        //     "{:.2} {:.2} {:.2}",
        //     link.f_avatar_position.x, link.f_avatar_position.y, link.f_avatar_position.z
        // ));
        ui.end_row();
    });
}
