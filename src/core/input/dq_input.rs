use std::collections::BTreeSet;

use copypasta::ClipboardProvider;
use device_query::{DeviceState, Keycode};
use egui::{Event, Key, Pos2, RawInput};

pub struct DQState {
    pub global_mouse_position: (i32, i32),
    pub dq: DeviceState,
    pub mouse_buttons: [bool; 3],
    pub keys_pressed: BTreeSet<Key>,
    pub clipboard: copypasta::ClipboardContext,
}

impl DQState {
    /// collect inputs using only device query. cannot get scroll, or window events.
    #[allow(dead_code)]
    pub fn query_input_events(
        &mut self,
        input: &mut RawInput,
        width: i32,
        height: i32,
        xpos: i32,
        ypos: i32,
    ) {
        let mut input_state = self;
        let mut events = Vec::new();

        let keys = input_state.dq.query_keymap();
        let mouse = input_state.dq.query_pointer();
        let mut modifiers = egui::Modifiers::default();

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
                    input_state.clipboard.get_contents().unwrap_or_default(),
                ));
            }
        }
        if keys.contains(&Keycode::LShift) | keys.contains(&Keycode::RShift) {
            modifiers.shift = true;
        }
        if keys.contains(&Keycode::LAlt) | keys.contains(&Keycode::RAlt) {
            modifiers.alt = true;
        }

        let egui_mouse_position = Pos2::new(
            (mouse.coords.0 - xpos) as f32,
            (mouse.coords.1 - ypos) as f32,
        );
        let egui_mouse_position =
            egui_mouse_position.clamp(Pos2::new(0.0, 0.0), Pos2::new(width as f32, height as f32));

        //mouse buttons start at 1 and can go upto 5 buttons in query. so, we compare index zero in our array to index 1 in query.
        //left click at one. but instead we swap around the right/left clicks so that our overlay is based on right clicking to avoid
        // spawning accidental background clicks that passthrough to gw2.
        if input_state.mouse_buttons[0] != mouse.button_pressed[1] {
            input_state.mouse_buttons[0] = !input_state.mouse_buttons[0];
            events.push(Event::PointerButton {
                pos: egui_mouse_position,
                button: egui::PointerButton::Primary,
                pressed: input_state.mouse_buttons[0],
                modifiers,
            });
        }
        //middle click at two
        if input_state.mouse_buttons[1] != mouse.button_pressed[2] {
            input_state.mouse_buttons[1] = !input_state.mouse_buttons[1];
            events.push(Event::PointerButton {
                pos: egui_mouse_position,
                button: egui::PointerButton::Middle,
                pressed: input_state.mouse_buttons[1],
                modifiers,
            });
        }
        // right click at third
        if input_state.mouse_buttons[2] != mouse.button_pressed[3] {
            input_state.mouse_buttons[2] = !input_state.mouse_buttons[2];
            events.push(Event::PointerButton {
                pos: egui_mouse_position,
                button: egui::PointerButton::Secondary,
                pressed: input_state.mouse_buttons[2],
                modifiers,
            });
        }

        let keys: Vec<Key> = keys
            .into_iter()
            .filter_map(|k| Self::dq_key_to_egui_key(k))
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

        // dbg!(mouse.coords, egui_mouse_position);
        // check for mouse position changes
        if input_state.global_mouse_position != mouse.coords {
            events.push(Event::PointerMoved(egui_mouse_position));
            input_state.global_mouse_position = mouse.coords;
            // dbg!(egui_mouse_position, input_state.global_mouse_position, input_state.window_pos);
        }
        input.events = events;
    }
    #[allow(dead_code)]
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
}
