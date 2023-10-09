use egui_window_glfw_passthrough::GlfwBackend;

pub struct WindowStatistics {
    pub fps_last_reset: f64,
    pub frame_count: u32,
    pub total_frame_count: u32,
    pub average_fps: u32,
}

impl WindowStatistics {
    pub fn new(current_time: f64) -> Self {
        Self {
            fps_last_reset: current_time,
            frame_count: 0,
            total_frame_count: 0,
            average_fps: 0,
        }
    }

    pub fn tick(&mut self, current_time: f64) {
        self.total_frame_count += 1;
        self.frame_count += 1;
        if current_time - self.fps_last_reset > 1.0 {
            self.average_fps = self.frame_count;
            self.frame_count = 0;
            self.fps_last_reset = current_time;
        }
    }

    pub fn gui(&mut self, etx: &egui::Context, wb: &mut GlfwBackend, open: &mut bool) {
        egui::Window::new("Window Manager")
            .open(open)
            .show(etx, |ui| {
                egui::Grid::new("frame details")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("fps");
                        ui.label(&format!("{}", self.average_fps));
                        ui.end_row();
                        ui.label("frame count");
                        ui.label(&format!("{}", self.total_frame_count));
                        ui.end_row();
                        ui.label("jokolay pos");
                        ui.label(&format!(
                            "x: {}; y: {}",
                            wb.window_position[0], wb.window_position[1]
                        ));
                        ui.end_row();
                        ui.label("jokolay size");
                        ui.label(&format!(
                            "width: {}, height: {}",
                            wb.framebuffer_size_physical[0], wb.framebuffer_size_physical[1]
                        ));
                        ui.end_row();
                        ui.label("decorations (borders)");
                        let is_decorated = wb.window.is_decorated();
                        let mut result = is_decorated;
                        if ui
                            .checkbox(
                                &mut result,
                                if is_decorated {
                                    "borders visible"
                                } else {
                                    "borders hidden"
                                },
                            )
                            .changed()
                        {
                            wb.window.set_decorated(result);
                        }
                        ui.end_row();
                    });
            });
    }
}
