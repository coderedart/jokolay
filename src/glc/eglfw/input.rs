use crate::JokolayApp;
use copypasta::{ClipboardContext, ClipboardProvider};
use device_query::{DeviceState, Keycode};
use egui::{Event, Key, Modifiers, Pos2, RawInput, Rect};
use nalgebra_glm::{make_vec2, I32Vec2};
use std::collections::BTreeSet;

pub struct GlobalInputState {
    pub mouse_position: I32Vec2,
    pub mouse_buttons: [bool; 3],
    pub keys_pressed: BTreeSet<Key>,
    pub clipboard: copypasta::ClipboardContext,
    pub dq: DeviceState,
    pub egui_input: RawInput,
    pub dimensions: (f32, f32),

}
impl GlobalInputState {
    pub fn new() -> Self {
        let clipboard = ClipboardContext::new().expect("couldn't get clipboard");
        let mut egui_input = RawInput::default();
        egui_input.screen_rect = Some(Rect::from_two_pos(Pos2::new(0.0, 0.0), Pos2::new(800.0, 600.0)));
        egui_input.predicted_dt = 1.0 / 75.0;
        egui_input.pixels_per_point = Some(1.0);
        GlobalInputState {
            mouse_position: Default::default(),
            mouse_buttons: [false, false, false],
            keys_pressed: BTreeSet::new(),
            dq: DeviceState::new(),
            clipboard,
            egui_input,
            dimensions: (800.0, 600.0)
        }
    }
}

impl JokolayApp {
    pub fn query_input_events(&self) {
        let mut input_state = self.input_state.borrow_mut();
        let mut events = Vec::new();

        let keys = input_state.dq.query_keymap();
        let mouse = input_state.dq.query_pointer();
        let mut modifiers = Modifiers::default();

        if keys.contains(&Keycode::LControl) | keys.contains(&Keycode::RControl) {
            modifiers.ctrl = true;
            modifiers.command = true;
            // check for copy
            if keys.contains(&Keycode::C) {
                events.push(Event::Copy);
            }
            // cut
            if keys.contains(&Keycode::X) {
                events.push(Event::Cut);
            }
            // paste
            if keys.contains(&Keycode::V) {
                events.push(Event::Text(
                    input_state.clipboard
                        .get_contents()
                        .unwrap_or_default(),
                ));
            }
        }
        if keys.contains(&Keycode::LShift) | keys.contains(&Keycode::RShift) {
            modifiers.shift = true;
        }
        if keys.contains(&Keycode::LAlt) | keys.contains(&Keycode::RAlt) {
            modifiers.alt = true;
        }

        let mouse_position = Pos2::new(mouse.coords.0 as f32, mouse.coords.1 as f32);
        if input_state.mouse_position[0] != mouse.coords.0
            || input_state.mouse_position[1] != mouse.coords.1
        {
            events.push(Event::PointerMoved(mouse_position));
            input_state.mouse_position = make_vec2(&[mouse.coords.0, mouse.coords.1]);
        }
        //mouse buttons start at 1 and can go upto 5 buttons in query. so, we compare index zero in our array to index 1 in query.
        if input_state.mouse_buttons[0] != mouse.button_pressed[1] {
            input_state.mouse_buttons[0] = !input_state.mouse_buttons[0];

            events.push(Event::PointerButton {
                pos: mouse_position,
                button: egui::PointerButton::Primary,
                pressed: input_state.mouse_buttons[0],
                modifiers,
            });
        }
        if input_state.mouse_buttons[1] != mouse.button_pressed[2] {
            input_state.mouse_buttons[1] = !input_state.mouse_buttons[1];

            events.push(Event::PointerButton {
                pos: mouse_position,
                button: egui::PointerButton::Secondary,
                pressed: input_state.mouse_buttons[1],
                modifiers,
            });
        }
        if input_state.mouse_buttons[2] != mouse.button_pressed[3] {
            input_state.mouse_buttons[2] = !input_state.mouse_buttons[2];

            events.push(Event::PointerButton {
                pos: mouse_position,
                button: egui::PointerButton::Middle,
                pressed: input_state.mouse_buttons[2],
                modifiers,
            });
        }
        if !keys.is_empty() {
            dbg!(&keys);
        }
        let keys: Vec<Key> = keys
            .into_iter()
            .filter_map(|k| dq_key_to_egui_key(k))
            .collect();
        input_state.keys_pressed.retain(|&k| {
            if !keys.contains(&k) {
                events.push(Event::Key {
                    key: k,
                    pressed: false,
                    modifiers,
                });
                false
            } else {
                true
            }
        });
        for k in keys {
            let new_press = input_state.keys_pressed.insert(k);
            if modifiers.ctrl {
                match k {
                    Key::C | Key::V | Key::X => continue,
                    _ => (),
                }
            }
            if new_press {
                events.push(Event::Key {
                    key: k,
                    pressed: true,
                    modifiers,
                });
            }
        }
        input_state.egui_input.events = events;

    }
}

/// converts device_query key code into egui key. none if egui doesn't have that key
fn dq_key_to_egui_key(key: Keycode) -> Option<Key> {
    match key {
        Keycode::Key0 | Keycode::Numpad0 => Some(Key::Num0),
        Keycode::Key1 | Keycode::Numpad1 => Some(Key::Num1),
        Keycode::Key2 | Keycode::Numpad2 => Some(Key::Num2),
        Keycode::Key3 | Keycode::Numpad3 => Some(Key::Num3),
        Keycode::Key4 | Keycode::Numpad4 => Some(Key::Num4),
        Keycode::Key5 | Keycode::Numpad5 => Some(Key::Num5),
        Keycode::Key6 | Keycode::Numpad6 => Some(Key::Num6),
        Keycode::Key7 | Keycode::Numpad7 => Some(Key::Num7),
        Keycode::Key8 | Keycode::Numpad8 => Some(Key::Num8),
        Keycode::Key9 | Keycode::Numpad9 => Some(Key::Num9),
        Keycode::A => Some(Key::A),
        Keycode::B => Some(Key::B),
        Keycode::C => Some(Key::C),
        Keycode::D => Some(Key::D),
        Keycode::E => Some(Key::E),
        Keycode::F => Some(Key::F),
        Keycode::G => Some(Key::G),
        Keycode::H => Some(Key::H),
        Keycode::I => Some(Key::I),
        Keycode::J => Some(Key::J),
        Keycode::K => Some(Key::K),
        Keycode::L => Some(Key::L),
        Keycode::M => Some(Key::M),
        Keycode::N => Some(Key::N),
        Keycode::O => Some(Key::O),
        Keycode::P => Some(Key::P),
        Keycode::Q => Some(Key::Q),
        Keycode::R => Some(Key::R),
        Keycode::S => Some(Key::S),
        Keycode::T => Some(Key::T),
        Keycode::U => Some(Key::U),
        Keycode::V => Some(Key::V),
        Keycode::W => Some(Key::W),
        Keycode::X => Some(Key::X),
        Keycode::Y => Some(Key::Y),
        Keycode::Z => Some(Key::Z),
        Keycode::Enter => Some(Key::Enter),
        Keycode::Up => Some(Key::ArrowUp),
        Keycode::Down => Some(Key::ArrowDown),
        Keycode::Left => Some(Key::ArrowLeft),
        Keycode::Right => Some(Key::ArrowRight),
        Keycode::Backspace => Some(Key::Backspace),
        Keycode::Tab => Some(Key::Tab),
        Keycode::Home => Some(Key::Home),
        Keycode::End => Some(Key::End),
        Keycode::PageUp => Some(Key::PageUp),
        Keycode::PageDown => Some(Key::PageDown),
        Keycode::Insert => Some(Key::Insert),
        Keycode::Delete => Some(Key::Delete),
        _ => None,
    }
}

// pub fn glfw_to_egui_key(key: glfw::Key) -> Option<Key> {
//     match key {
//         glfw::Key::Space => todo!(),
//         glfw::Key::Num0 => todo!(),
//         glfw::Key::Num1 => todo!(),
//         glfw::Key::Num2 => todo!(),
//         glfw::Key::Num3 => todo!(),
//         glfw::Key::Num4 => todo!(),
//         glfw::Key::Num5 => todo!(),
//         glfw::Key::Num6 => todo!(),
//         glfw::Key::Num7 => todo!(),
//         glfw::Key::Num8 => todo!(),
//         glfw::Key::Num9 => todo!(),
//         glfw::Key::A => todo!(),
//         glfw::Key::B => todo!(),
//         glfw::Key::C => todo!(),
//         glfw::Key::D => todo!(),
//         glfw::Key::E => todo!(),
//         glfw::Key::F => todo!(),
//         glfw::Key::G => todo!(),
//         glfw::Key::H => todo!(),
//         glfw::Key::I => todo!(),
//         glfw::Key::J => todo!(),
//         glfw::Key::K => todo!(),
//         glfw::Key::L => todo!(),
//         glfw::Key::M => todo!(),
//         glfw::Key::N => todo!(),
//         glfw::Key::O => todo!(),
//         glfw::Key::P => todo!(),
//         glfw::Key::Q => todo!(),
//         glfw::Key::R => todo!(),
//         glfw::Key::S => todo!(),
//         glfw::Key::T => todo!(),
//         glfw::Key::U => todo!(),
//         glfw::Key::V => todo!(),
//         glfw::Key::W => todo!(),
//         glfw::Key::X => todo!(),
//         glfw::Key::Y => todo!(),
//         glfw::Key::Z => todo!(),
//         glfw::Key::Escape => todo!(),
//         glfw::Key::Enter => todo!(),
//         glfw::Key::Tab => todo!(),
//         glfw::Key::Backspace => todo!(),
//         glfw::Key::Insert => todo!(),
//         glfw::Key::Delete => todo!(),
//         glfw::Key::Right => todo!(),
//         glfw::Key::Left => todo!(),
//         glfw::Key::Down => todo!(),
//         glfw::Key::Up => todo!(),
//         glfw::Key::PageUp => todo!(),
//         glfw::Key::PageDown => todo!(),
//         glfw::Key::Home => todo!(),
//         glfw::Key::End => todo!(),
//         glfw::Key::CapsLock => todo!(),
//         glfw::Key::ScrollLock => todo!(),
//         glfw::Key::NumLock => todo!(),
//         glfw::Key::PrintScreen => todo!(),
//         glfw::Key::Pause => todo!(),
//         glfw::Key::Kp0 => todo!(),
//         glfw::Key::Kp1 => todo!(),
//         glfw::Key::Kp2 => todo!(),
//         glfw::Key::Kp3 => todo!(),
//         glfw::Key::Kp4 => todo!(),
//         glfw::Key::Kp5 => todo!(),
//         glfw::Key::Kp6 => todo!(),
//         glfw::Key::Kp7 => todo!(),
//         glfw::Key::Kp8 => todo!(),
//         glfw::Key::Kp9 => todo!(),
//         glfw::Key::KpEnter => todo!(),
//         glfw::Key::LeftShift => todo!(),
//         glfw::Key::LeftControl => todo!(),
//         glfw::Key::LeftAlt => todo!(),
//         glfw::Key::LeftSuper => todo!(),
//         glfw::Key::RightShift => todo!(),
//         glfw::Key::RightControl => todo!(),
//         glfw::Key::RightAlt => todo!(),
//         glfw::Key::RightSuper => todo!(),
//         glfw::Key::Menu => todo!(),
//         glfw::Key::Unknown => todo!(),
//     }
// }
