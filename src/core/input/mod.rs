use std::{rc::Rc, sync::mpsc::Receiver};

use glfw::{Glfw, WindowEvent};

use crate::core::{input::glfw_input::GlfwInput, window::glfw_window::OverlayWindow};

pub mod dq_input;
pub mod glfw_input;
pub mod rdev_input;

pub struct InputManager {
    pub glfw_input: GlfwInput,
}

impl InputManager {
    pub fn new(events: Receiver<(f64, WindowEvent)>, glfw: Glfw) -> Self {
        Self {
            glfw_input: GlfwInput::new(events, glfw),
        }
    }

    pub fn tick(&mut self, gl: Rc<glow::Context>, ow: &mut OverlayWindow) -> FrameEvents {
        let frame_events = self.glfw_input.get_events(gl, ow);
        frame_events
    }
}

#[derive(Debug, Clone)]
pub struct FrameEvents {
    pub all_events: Vec<WindowEvent>,
    pub time: f64,
    pub cursor_position: egui::Pos2,
}
