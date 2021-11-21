use std::rc::Rc;
use std::sync::mpsc::Receiver;

use egui::{Key, PointerButton};

use glfw::{Action, Glfw, WindowEvent};

use crate::core::input::FrameEvents;
use crate::core::window::OverlayWindow;

#[derive(Debug)]
pub struct GlfwInput {
    pub events: Receiver<(f64, WindowEvent)>,
    pub glfw: Glfw,
    pub frame_number: usize,
    pub last_reset: f64,
    pub average_fps: usize,
}

impl GlfwInput {
    pub fn new(events: Receiver<(f64, WindowEvent)>, glfw: Glfw) -> Self {
        let last_reset = glfw.get_time();
        Self {
            events,
            glfw,
            frame_number: 0,
            last_reset,
            average_fps: 0,
        }
    }
    pub fn get_events(&mut self, gl: Rc<glow::Context>, ow: &mut OverlayWindow) -> FrameEvents {
        self.glfw.poll_events();
        let (x, y) = ow.window.get_cursor_pos();
        let time = self.glfw.get_time();
        let delta = time - self.last_reset;
        self.frame_number += 1;
        if delta > 1.0 {
            self.average_fps = (self.frame_number as f64 / delta) as usize;
            self.last_reset = time;
            self.frame_number = 0;
        }
        let mut frame_event = FrameEvents {
            average_frame_rate: self.average_fps,
            all_events: vec![],
            time,
            cursor_position: [x as f32, y as f32].into(),
        };

        for (_, event) in glfw::flush_messages(&self.events) {
            frame_event.all_events.push(event.clone());
            if let glfw::WindowEvent::FramebufferSize(width, height) = event {
                // make sure the viewport matches the new window dimensions; note that width and
                // height will be significantly larger than specified on retina displays.
                unsafe {
                    use glow::HasContext;
                    gl.viewport(0, 0, width, height);
                }
            }
        }
        frame_event
    }
}
/// a function to get the matching egui key event for a given glfw key. egui does not support all the keys provided here.
pub fn glfw_to_egui_key(key: glfw::Key) -> Option<Key> {
    match key {
        glfw::Key::Space => Some(Key::Space),
        glfw::Key::Num0 => Some(Key::Num0),
        glfw::Key::Num1 => Some(Key::Num1),
        glfw::Key::Num2 => Some(Key::Num2),
        glfw::Key::Num3 => Some(Key::Num3),
        glfw::Key::Num4 => Some(Key::Num4),
        glfw::Key::Num5 => Some(Key::Num5),
        glfw::Key::Num6 => Some(Key::Num6),
        glfw::Key::Num7 => Some(Key::Num7),
        glfw::Key::Num8 => Some(Key::Num8),
        glfw::Key::Num9 => Some(Key::Num9),
        glfw::Key::A => Some(Key::A),
        glfw::Key::B => Some(Key::B),
        glfw::Key::C => Some(Key::C),
        glfw::Key::D => Some(Key::D),
        glfw::Key::E => Some(Key::E),
        glfw::Key::F => Some(Key::F),
        glfw::Key::G => Some(Key::G),
        glfw::Key::H => Some(Key::H),
        glfw::Key::I => Some(Key::I),
        glfw::Key::J => Some(Key::J),
        glfw::Key::K => Some(Key::K),
        glfw::Key::L => Some(Key::L),
        glfw::Key::M => Some(Key::M),
        glfw::Key::N => Some(Key::N),
        glfw::Key::O => Some(Key::O),
        glfw::Key::P => Some(Key::P),
        glfw::Key::Q => Some(Key::Q),
        glfw::Key::R => Some(Key::R),
        glfw::Key::S => Some(Key::S),
        glfw::Key::T => Some(Key::T),
        glfw::Key::U => Some(Key::U),
        glfw::Key::V => Some(Key::V),
        glfw::Key::W => Some(Key::W),
        glfw::Key::X => Some(Key::X),
        glfw::Key::Y => Some(Key::Y),
        glfw::Key::Z => Some(Key::Z),
        glfw::Key::Escape => Some(Key::Escape),
        glfw::Key::Enter => Some(Key::Enter),
        glfw::Key::Tab => Some(Key::Tab),
        glfw::Key::Backspace => Some(Key::Backspace),
        glfw::Key::Insert => Some(Key::Insert),
        glfw::Key::Delete => Some(Key::Delete),
        glfw::Key::Right => Some(Key::ArrowRight),
        glfw::Key::Left => Some(Key::ArrowLeft),
        glfw::Key::Down => Some(Key::ArrowDown),
        glfw::Key::Up => Some(Key::ArrowUp),
        glfw::Key::PageUp => Some(Key::PageUp),
        glfw::Key::PageDown => Some(Key::PageDown),
        glfw::Key::Home => Some(Key::Home),
        glfw::Key::End => Some(Key::End),
        _ => None,
    }
}

pub fn glfw_to_egui_modifers(modifiers: glfw::Modifiers) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.contains(glfw::Modifiers::Alt),
        ctrl: modifiers.contains(glfw::Modifiers::Control),
        shift: modifiers.contains(glfw::Modifiers::Shift),
        mac_cmd: false,
        command: modifiers.contains(glfw::Modifiers::Control),
    }
}

pub fn glfw_to_egui_pointer_button(mb: glfw::MouseButton) -> PointerButton {
    match mb {
        glfw::MouseButton::Button1 => PointerButton::Primary,
        glfw::MouseButton::Button2 => PointerButton::Secondary,
        glfw::MouseButton::Button3 => PointerButton::Middle,
        _ => PointerButton::Secondary,
    }
}

pub fn glfw_to_egui_action(a: glfw::Action) -> bool {
    match a {
        Action::Release => false,
        Action::Press => true,
        Action::Repeat => true,
    }
}
