use crate::core::{
    mlink::MumbleSource,
    window::glfw_window::{OverlayWindow, WindowDimensions},
};

pub struct WindowsPlatformData {
    pub gw2_window_handle: u32,
    pub gw2_pid: u32,
}

impl WindowsPlatformData {
    pub fn new(window: &glfw::Window, mumble_src: &mut MumbleSource) -> Self {
        todo!()
    }
    pub fn is_gw2_alive(&self) -> bool {
        todo!()
    }
    pub fn get_gw2_windim(&self) -> WindowDimensions {
        todo!()
    }
}

impl OverlayWindow {
    pub fn is_gw2_alive(&self) -> bool {
        self.platform_data.is_gw2_alive()
    }
    pub fn get_gw2_windim(&self) -> WindowDimensions {
        self.platform_data.get_gw2_windim()
    }
}
