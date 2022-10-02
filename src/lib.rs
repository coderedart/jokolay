pub mod jmf;
pub mod joko_render;
pub mod jokoapi;
pub mod jokolink;
use egui_overlay::{
    egui::{self, Grid, Ui},
    egui_backend::{raw_window_handle::HasRawWindowHandle, GfxBackend, UserApp, WindowBackend},
    egui_window_glfw_passthrough::GlfwWindow,
};
use glam::{vec3, Mat4};
use jmf::manager::MarkerManager;
use joko_render::JokoRenderer;
use jokolink::MumbleManager;

pub struct Jokolay {
    pub last_check: f64,
    pub mumble: MumbleManager,
    pub marker_manager: MarkerManager,
}

impl Jokolay {
    fn new(window_backend: &mut GlfwWindow) -> Self {
        let window_id: u32 = match window_backend.window.raw_window_handle() {
            egui_overlay::egui_backend::raw_window_handle::RawWindowHandle::Xlib(id) => {
                id.window.try_into().unwrap()
            }
            egui_overlay::egui_backend::raw_window_handle::RawWindowHandle::Xcb(id) => id.window,
            _ => 0,
        };
        let mut mumble =
            MumbleManager::new("MumbleLink", window_id).expect("failed to create mumble manager");
        let marker_manager = MarkerManager::new(std::path::Path::new("./assets/packs"))
            .expect("failed to create marker manager");
        if let Ok(wd) = mumble.get_latest_window_dimensions() {
            let (x, y) = window_backend.window.get_pos();
            let (width, height) = window_backend.window.get_size();
            if x != wd.x || y != wd.y || width as u32 != wd.width || height as u32 != wd.height {
                window_backend.window.set_pos(wd.x, wd.y);
                window_backend
                    .window
                    .set_size(wd.width as i32, wd.height as i32);
            }
        }
        Self {
            mumble,
            marker_manager,
            last_check: 0.0,
        }
    }
}
impl UserApp<GlfwWindow, JokoRenderer> for Jokolay {
    fn run(
        &mut self,
        egui_context: &egui::Context,
        window_backend: &mut GlfwWindow,
        renderer: &mut JokoRenderer,
    ) {
        let latest_time = window_backend.glfw.get_time();
        let _ = self.mumble.tick();

        egui::Window::new("egui window")
            .default_width(300.0)
            .show(egui_context, |ui| {
                if let Some(link) = self.mumble.get_mumble_link() {
                    mumble_ui(ui, link);
                }
            });
        self.marker_manager.tick(egui_context, latest_time);
        if let Some(link) = self.mumble.get_mumble_link() {
            self.marker_manager
                .render(link.context.map_id as u16, renderer);
            let ratio = window_backend.size_physical_pixels[0] as f32
                / window_backend.size_physical_pixels[1] as f32;
            let center = link.f_camera_position + link.f_camera_front;
            let v = Mat4::look_at_lh(link.f_camera_position, center, vec3(0.0, 1.0, 0.0));
            let p = Mat4::perspective_lh(link.identity.fov, ratio, 1.0, 10000.0);
            renderer.set_mvp(p * v);
        }
        // if it doesn't require either keyboard or pointer, set passthrough to true
        window_backend.window.set_mouse_passthrough(
            !(egui_context.wants_keyboard_input() || egui_context.wants_pointer_input()),
        );
        if latest_time - self.last_check > 0. {
            self.last_check = latest_time;
            // self.window_dimensions = self.mumble.get_window_dimensions();
            // if let Ok(wd) = self.window_dimensions {
            //     let (x, y) = window_backend.window.get_pos();
            //     let (width, height) = window_backend.window.get_size();
            //     if x != wd.x || y != wd.y || width as u32 != wd.width || height as u32 != wd.height
            //     {
            //         window_backend.window.set_pos(wd.x, wd.y);
            //         window_backend
            //             .window
            //             .set_size(wd.width as i32, wd.height as i32);
            //     }
            // }
        }
    }
}

pub fn start_jokolay() {
    let mut glfw_backend = GlfwWindow::new(Default::default(), Default::default());
    let wgpu_backend = JokoRenderer::new(&mut glfw_backend, Default::default());
    // remove decorations
    glfw_backend.window.set_decorated(false);
    let jokolay = Jokolay::new(&mut glfw_backend);
    glfw_backend.run_event_loop(wgpu_backend, jokolay);
}

fn mumble_ui(ui: &mut Ui, link: &jokolink::MumbleLink) {
    Grid::new("link grid").num_columns(2).show(ui, |ui| {
        ui.label("ui tick: ");
        ui.label(format!("{}", link.ui_tick));
        ui.end_row();
        ui.label("character: ");
        ui.label(&link.identity.name);
        ui.end_row();
        ui.label("ui state");
        ui.label(format!("{:?}", unsafe {
            jokolink::UIState::from_bits_unchecked(link.context.ui_state)
        }));
    });
}
