use std::sync::mpsc::Receiver;

use glc::renderer::vertex_array::VertexArrayObject;
use glfw::{Action, Key};
use glow::HasContext;

use crate::glc::renderer::vertex_buffer::{VertexBuffer, VertexBufferLayout};

pub mod glc;
pub mod gw;

pub fn process_events(
    window: &mut glfw::Window,
    events: &Receiver<(f64, glfw::WindowEvent)>,
    gl: &glow::Context,
) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            glfw::WindowEvent::FramebufferSize(width, height) => {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe {
                    gl.viewport(0, 0, width, height);
                }
                eprintln!("resizing viewport");
            }
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            _ => {}
        }
    }
}
