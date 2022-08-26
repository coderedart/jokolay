use egui_overlay::{
    egui::{self, CollapsingHeader},
    egui_backend::UserApp,
    egui_render_wgpu::WgpuBackend,
    egui_window_glfw_passthrough::GlfwWindow,
};
use jokolink::mumble_file::{MumbleFile, MumbleFileTrait};

pub struct Jokolay {
    pub mfile: MumbleFile,
    pub average_fps: u32,
    pub last_second: u32,
    pub current_frame_count: u32,
}

impl Default for Jokolay {
    fn default() -> Self {
        let mfile = MumbleFile::new("MumbleLink", 0.0).expect("failed to create mumble link");

        Self {
            mfile,
            average_fps: 0,
            last_second: 0,
            current_frame_count: 0,
        }
    }
}
impl UserApp<GlfwWindow, WgpuBackend> for Jokolay {
    fn run(
        &mut self,
        egui_context: &egui::Context,
        window_backend: &mut GlfwWindow,
        _gfx_backend: &mut WgpuBackend,
    ) {
        let latest_time = window_backend.glfw.get_time();
        self.current_frame_count += 1;
        if latest_time as u32 > self.last_second {
            self.last_second = latest_time as u32;
            self.average_fps = (self.average_fps + self.current_frame_count) / 2;
            self.current_frame_count = 0;
        }

        egui::Window::new("egui window")
            .default_width(300.0)
            .show(egui_context, |ui| {
                ui.label("hello");
                ui.label(format!("fps: {}", self.average_fps));
                CollapsingHeader::new("mumble link")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label(format!(
                            "{:#?}",
                            self.mfile
                                .get_link(latest_time)
                                .expect("failed to get mumble link")
                                .map(|m| { m.link.ui_tick })
                        ));
                    });
            });
        // if it doesn't require either keyboard or pointer, set passthrough to true
        window_backend.window.set_mouse_passthrough(
            !(egui_context.wants_keyboard_input() || egui_context.wants_pointer_input()),
        );
    }
}
