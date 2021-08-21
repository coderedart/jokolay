use crate::{JokolayApp, window::glfw_window::GlfwWindow};

pub mod glfw_window;

impl JokolayApp {
    pub fn attach_to_gw2window(&mut self) {
        let window_dimensions = self.mumble_manager.get_window_dimensions();
        self.overlay_window.set_inner_position(window_dimensions.x, window_dimensions.y);
        self.overlay_window.set_inner_size(window_dimensions.width, window_dimensions.height);
    }
}