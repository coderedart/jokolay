use std::path::PathBuf;

use egui_backend::{
    egui::{self, Grid, Ui},
    BackendConfig, GfxBackend, UserApp, WindowBackend,
};
use egui_window_glfw_passthrough::{GlfwBackend, GlfwConfig};
use joko_core::{init::get_jokolay_dir, prelude::*, trace::install_tracing};

use jmf::manager::MarkerManager;
use joko_render::JokoRenderer;
use jokolink::MumbleManager;

pub struct Jokolay {
    pub last_check: f64,
    pub show_tracing_window: bool,
    pub fps: u32,
    pub frame_count: u64,
    pub frame_reset_seconds_timestamp: u64,
    pub jdir: Dir,
    pub jpath: PathBuf,
    pub mumble_manager: Result<MumbleManager>,
    pub marker_manager: Result<MarkerManager>,
    pub joko_renderer: JokoRenderer,
    pub egui_context: egui::Context,
    pub window_backend: GlfwBackend,
}

impl Jokolay {
    fn new(
        window_backend: GlfwBackend,
        joko_renderer: JokoRenderer,
        jpath: PathBuf,
        jdir: Dir,
    ) -> Result<Self> {
        let mumble = MumbleManager::new("MumbleLink", None);
        let marker_manager = MarkerManager::new(&jdir);
        let egui_context = egui::Context::default();
        // use roboto for ui fonts
        {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "roboto".to_owned(),
                egui::FontData::from_static(include_bytes!("roboto.ttf")),
            );
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "roboto".to_owned());
            egui_context.set_fonts(fonts);
        }

        Ok(Self {
            mumble_manager: mumble,
            marker_manager,

            last_check: 0.0,
            joko_renderer,
            frame_count: 0,
            frame_reset_seconds_timestamp: 0,
            fps: 0,
            window_backend,
            jdir,
            jpath,
            egui_context,
            show_tracing_window: true,
        })
    }
}
impl UserApp for Jokolay {
    fn gui_run(&mut self) {
        let Self {
            last_check,
            fps,
            frame_count,
            frame_reset_seconds_timestamp,
            mumble_manager,
            marker_manager,
            joko_renderer: _,
            egui_context,
            window_backend,
            jdir: _,
            jpath: _,
            show_tracing_window,
        } = self;

        let latest_time = window_backend.glfw.get_time();
        *frame_count += 1;
        if latest_time - *frame_reset_seconds_timestamp as f64 > 1.0 {
            *fps = *frame_count as u32;
            *frame_count = 0;
            *frame_reset_seconds_timestamp = latest_time as u64;
        }
        if let Ok(mm) = mumble_manager.as_mut() {
            let _ = mm.tick();
        };
        egui_context.request_repaint();
        let cursor_position = egui_context.pointer_latest_pos();
        egui::Window::new("Tracing Window")
            .open(show_tracing_window)
            .show(egui_context, |ui| {
                joko_core::trace::show_tracing_events(ui);
            });
        egui::Window::new("egui window")
            .default_width(300.0)
            .show(egui_context, |ui| {
                ui.label(&format!("cursor position: {cursor_position:?}"));
                ui.label(&format!("fps: {}", *fps));
                let mut is_passthrough = window_backend.window.is_mouse_passthrough();
                ui.checkbox(&mut is_passthrough, "is window passthrough?");
                match mumble_manager {
                    Ok(mm) => {
                        let [x, y, w, h] = mm.get_pos_size();
                        ui.label(format!("pos: {x}, {y}"));
                        ui.label(format!("size: {w}, {h}"));
                        match mm.get_mumble_link() {
                            Ok(link) => {
                                mumble_ui(ui, link);
                            }
                            Err(e) => {
                                ui.label(format!("mumble link error: {e:#?}"));
                            }
                        }
                    }
                    Err(e) => {
                        ui.label(format!("mumble manager error: {e:#?}"));
                    }
                }
            });
        if let Ok(marker_manager) = marker_manager {
            marker_manager.tick(egui_context, latest_time);
        }
        // if let Some(_link) = link.as_ref() {
        //     if let Ok(_marker_manager) = marker_manager {
        //         // marker_manager.render(link.context.map_id as u16, joko_renderer);
        //     }
        // }

        // if it doesn't require either keyboard or pointer, set passthrough to true
        window_backend.window.set_mouse_passthrough(
            !(egui_context.wants_keyboard_input() || egui_context.wants_pointer_input()),
        );
        if latest_time - *last_check > 10. {
            *last_check = latest_time;

            if let Ok(mumble) = mumble_manager {
                let [x, y, w, h] = mumble.get_pos_size();
                if w != 0 && h != 0 {
                    let (wx, wy) = window_backend.window.get_pos();
                    let (ww, wh) = window_backend.window.get_size();
                    if x != wx || y != wy || w != ww || h != wh {
                        info!("resizing/repositioning our window from {wx},{wy},{ww},{wh} to match gw2 window dimensions: {x} {y} {w} {h}");
                        window_backend.window.set_pos(x, y);
                        window_backend.window.set_size(w, h);
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
    let (jokolay_dir_path, jdir) = get_jokolay_dir().unwrap();
    let _log_file_flush_guard = install_tracing(&jdir).unwrap();
    info!("using {jokolay_dir_path:?} as the jokolay data directory");

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
    let jokolay = Jokolay::new(glfw_backend, joko_renderer, jokolay_dir_path, jdir);
    <Jokolay as UserApp>::UserWindowBackend::run_event_loop(
        jokolay.expect("failed to create jokolay app"),
    );
}

fn mumble_ui(ui: &mut Ui, link: &jokolink::MumbleLink) {
    Grid::new("link grid").num_columns(2).show(ui, |ui| {
        ui.label("ui tick: ");
        ui.label(format!("{}", link.ui_tick));
        ui.end_row();
        ui.label("character: ");
        ui.label(&link.name);
        ui.end_row();
        ui.label("map: ");
        ui.label(format!("{}", link.map_id));
        ui.end_row();
    });
}
