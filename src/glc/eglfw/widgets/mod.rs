use egui::{CtxRef, Window};




pub struct MainWindow {
    pub name: String,
    pub mumble_window_show: bool,
    // pub timer_window_show: bool,
    // pub marker_window_show: bool,
}

impl Default for MainWindow {
    fn default() -> Self {
        MainWindow {
            name: "MainWindow".to_string(),
            mumble_window_show: false,
        }
    }
}

impl MainWindow {
    pub fn add_widgets_to_ui(&mut self, ctx: &CtxRef) {
        Window::new(&self.name).show(&ctx, |ui | {
            if ui.checkbox(&mut self.mumble_window_show, "show mumble Window").changed() {
                if self.mumble_window_show {
                    
                }
            };
        });
        
    }
}