pub struct FrameStatistics {
    pub fps_last_reset: f64,
    pub frame_count: u32,
    pub total_frame_count: u32,
    pub average_fps: u32,
}

impl FrameStatistics {
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

    pub fn gui(&mut self, ui: &mut egui::Ui) {
        ui.label(&format!("fps: {}", self.average_fps));
    }
}
