// use std::{cell::RefCell, rc::Rc};

// use copypasta::{ClipboardContext, ClipboardProvider};
// use device_query::{DeviceState, Keycode};
// use egui::{Event, Key, Modifiers, RawInput};
// use egui::Pos2;

// use nalgebra_glm::{make_vec2, I32Vec2};
// use std::collections::BTreeSet;

// pub struct GlobalInputState {
//     pub global_mouse_position: I32Vec2,
//     pub mouse_buttons: [bool; 3],
//     pub keys_pressed: BTreeSet<Key>,
//     pub clipboard: copypasta::ClipboardContext,
//     pub dq: DeviceState,
// }
// impl GlobalInputState {
//     pub fn new() -> Self {
//         let clipboard = ClipboardContext::new().expect("couldn't get clipboard");

//         GlobalInputState {
//             global_mouse_position: Default::default(),
//             mouse_buttons: [false, false, false],
//             keys_pressed: BTreeSet::new(),
//             dq: DeviceState::new(),
//             clipboard,
//         }
//     }
// }

// /// converts device_query key code into egui key. none if egui doesn't have that key
// fn dq_key_to_egui_key(key: Keycode) -> Option<Key> {
//     match key {
//         Keycode::Key0 | Keycode::Numpad0 => Some(Key::Num0),
//         Keycode::Key1 | Keycode::Numpad1 => Some(Key::Num1),
//         Keycode::Key2 | Keycode::Numpad2 => Some(Key::Num2),
//         Keycode::Key3 | Keycode::Numpad3 => Some(Key::Num3),
//         Keycode::Key4 | Keycode::Numpad4 => Some(Key::Num4),
//         Keycode::Key5 | Keycode::Numpad5 => Some(Key::Num5),
//         Keycode::Key6 | Keycode::Numpad6 => Some(Key::Num6),
//         Keycode::Key7 | Keycode::Numpad7 => Some(Key::Num7),
//         Keycode::Key8 | Keycode::Numpad8 => Some(Key::Num8),
//         Keycode::Key9 | Keycode::Numpad9 => Some(Key::Num9),
//         Keycode::A => Some(Key::A),
//         Keycode::B => Some(Key::B),
//         Keycode::C => Some(Key::C),
//         Keycode::D => Some(Key::D),
//         Keycode::E => Some(Key::E),
//         Keycode::F => Some(Key::F),
//         Keycode::G => Some(Key::G),
//         Keycode::H => Some(Key::H),
//         Keycode::I => Some(Key::I),
//         Keycode::J => Some(Key::J),
//         Keycode::K => Some(Key::K),
//         Keycode::L => Some(Key::L),
//         Keycode::M => Some(Key::M),
//         Keycode::N => Some(Key::N),
//         Keycode::O => Some(Key::O),
//         Keycode::P => Some(Key::P),
//         Keycode::Q => Some(Key::Q),
//         Keycode::R => Some(Key::R),
//         Keycode::S => Some(Key::S),
//         Keycode::T => Some(Key::T),
//         Keycode::U => Some(Key::U),
//         Keycode::V => Some(Key::V),
//         Keycode::W => Some(Key::W),
//         Keycode::X => Some(Key::X),
//         Keycode::Y => Some(Key::Y),
//         Keycode::Z => Some(Key::Z),
//         Keycode::Enter => Some(Key::Enter),
//         Keycode::Up => Some(Key::ArrowUp),
//         Keycode::Down => Some(Key::ArrowDown),
//         Keycode::Left => Some(Key::ArrowLeft),
//         Keycode::Right => Some(Key::ArrowRight),
//         Keycode::Backspace => Some(Key::Backspace),
//         Keycode::Tab => Some(Key::Tab),
//         Keycode::Home => Some(Key::Home),
//         Keycode::End => Some(Key::End),
//         Keycode::PageUp => Some(Key::PageUp),
//         Keycode::PageDown => Some(Key::PageDown),
//         Keycode::Insert => Some(Key::Insert),
//         Keycode::Delete => Some(Key::Delete),
//         _ => None,
//     }
// }

// pub fn glfw_to_egui_key(key: glfw::Key) -> Option<Key> {
//     match key {
//         glfw::Key::Space => Some(Key::Space),
//         glfw::Key::Num0 => Some(Key::Num0),
//         glfw::Key::Num1 => Some(Key::Num1),
//         glfw::Key::Num2 => Some(Key::Num2),
//         glfw::Key::Num3 => Some(Key::Num3),
//         glfw::Key::Num4 => Some(Key::Num4),
//         glfw::Key::Num5 => Some(Key::Num5),
//         glfw::Key::Num6 => Some(Key::Num6),
//         glfw::Key::Num7 => Some(Key::Num7),
//         glfw::Key::Num8 => Some(Key::Num8),
//         glfw::Key::Num9 => Some(Key::Num9),
//         glfw::Key::A => Some(Key::A),
//         glfw::Key::B => Some(Key::B),
//         glfw::Key::C => Some(Key::C),
//         glfw::Key::D => Some(Key::D),
//         glfw::Key::E => Some(Key::E),
//         glfw::Key::F => Some(Key::F),
//         glfw::Key::G => Some(Key::G),
//         glfw::Key::H => Some(Key::H),
//         glfw::Key::I => Some(Key::I),
//         glfw::Key::J => Some(Key::J),
//         glfw::Key::K => Some(Key::K),
//         glfw::Key::L => Some(Key::L),
//         glfw::Key::M => Some(Key::M),
//         glfw::Key::N => Some(Key::N),
//         glfw::Key::O => Some(Key::O),
//         glfw::Key::P => Some(Key::P),
//         glfw::Key::Q => Some(Key::Q),
//         glfw::Key::R => Some(Key::R),
//         glfw::Key::S => Some(Key::S),
//         glfw::Key::T => Some(Key::T),
//         glfw::Key::U => Some(Key::U),
//         glfw::Key::V => Some(Key::V),
//         glfw::Key::W => Some(Key::W),
//         glfw::Key::X => Some(Key::X),
//         glfw::Key::Y => Some(Key::Y),
//         glfw::Key::Z => Some(Key::Z),
//         glfw::Key::Escape => Some(Key::Escape),
//         glfw::Key::Enter => Some(Key::Enter),
//         glfw::Key::Tab => Some(Key::Tab),
//         glfw::Key::Backspace => Some(Key::Backspace),
//         glfw::Key::Insert => Some(Key::Insert),
//         glfw::Key::Delete => Some(Key::Delete),
//         glfw::Key::Right => Some(Key::ArrowRight),
//         glfw::Key::Left => Some(Key::ArrowLeft),
//         glfw::Key::Down => Some(Key::ArrowDown),
//         glfw::Key::Up => Some(Key::ArrowUp),
//         glfw::Key::PageUp => Some(Key::PageUp),
//         glfw::Key::PageDown => Some(Key::PageDown),
//         glfw::Key::Home => Some(Key::Home),
//         glfw::Key::End => Some(Key::End),
//         _ => None,
//     }
// }
// pub fn query_input_events(
//     global_input_state: Rc<RefCell<GlobalInputState>>,
//     input: &mut RawInput,
//     width: i32,
//     height: i32,
//     xpos: i32,
//     ypos: i32,
// ) {
//     let mut input_state = global_input_state.borrow_mut();
//     let mut events = Vec::new();

//     let keys = input_state.dq.query_keymap();
//     let mouse = input_state.dq.query_pointer();
//     let mut modifiers = Modifiers::default();

//     if keys.contains(&Keycode::LControl) | keys.contains(&Keycode::RControl) {
//         modifiers.ctrl = true;
//         modifiers.command = true;
//         // check for copy
//         if keys.contains(&Keycode::C) {
//             events.push(Event::Copy);
//         }
//         // cut
//         if keys.contains(&Keycode::X) {
//             events.push(Event::Cut);
//         }
//         // paste
//         if keys.contains(&Keycode::V) {
//             events.push(Event::Text(
//                 input_state.clipboard.get_contents().unwrap_or_default(),
//             ));
//         }
//     }
//     if keys.contains(&Keycode::LShift) | keys.contains(&Keycode::RShift) {
//         modifiers.shift = true;
//     }
//     if keys.contains(&Keycode::LAlt) | keys.contains(&Keycode::RAlt) {
//         modifiers.alt = true;
//     }

//     let egui_mouse_position = Pos2::new(
//         (mouse.coords.0 - xpos) as f32,
//         (mouse.coords.1 - ypos) as f32,
//     );
//     let egui_mouse_position =
//         egui_mouse_position.clamp(Pos2::new(0.0, 0.0), Pos2::new(width as f32, height as f32));

//     //mouse buttons start at 1 and can go upto 5 buttons in query. so, we compare index zero in our array to index 1 in query.
//     //left click at one. but instead we swap around the right/left clicks so that our overlay is based on right clicking to avoid
//     // spawning accidental background clicks that passthrough to gw2.
//     if input_state.mouse_buttons[0] != mouse.button_pressed[1] {
//         input_state.mouse_buttons[0] = !input_state.mouse_buttons[0];
//         events.push(Event::PointerButton {
//             pos: egui_mouse_position,
//             button: egui::PointerButton::Primary,
//             pressed: input_state.mouse_buttons[0],
//             modifiers,
//         });
//     }
//     //middle click at two
//     if input_state.mouse_buttons[1] != mouse.button_pressed[2] {
//         input_state.mouse_buttons[1] = !input_state.mouse_buttons[1];
//         events.push(Event::PointerButton {
//             pos: egui_mouse_position,
//             button: egui::PointerButton::Middle,
//             pressed: input_state.mouse_buttons[1],
//             modifiers,
//         });
//     }
//     // right click at third
//     if input_state.mouse_buttons[2] != mouse.button_pressed[3] {
//         input_state.mouse_buttons[2] = !input_state.mouse_buttons[2];
//         events.push(Event::PointerButton {
//             pos: egui_mouse_position,
//             button: egui::PointerButton::Secondary,
//             pressed: input_state.mouse_buttons[2],
//             modifiers,
//         });
//     }

//     let keys: Vec<Key> = keys
//         .into_iter()
//         .filter_map(|k| dq_key_to_egui_key(k))
//         .collect();
//     input_state.keys_pressed.retain(|&k| {
//         if !keys.contains(&k) {
//             events.push(Event::Key {
//                 key: k,
//                 pressed: false,
//                 modifiers,
//             });
//             false
//         } else {
//             true
//         }
//     });
//     for k in keys {
//         let new_press = input_state.keys_pressed.insert(k);
//         if modifiers.ctrl {
//             match k {
//                 Key::C | Key::V | Key::X => continue,
//                 _ => (),
//             }
//         }
//         if new_press {
//             events.push(Event::Key {
//                 key: k,
//                 pressed: true,
//                 modifiers,
//             });
//         }
//     }

//     // dbg!(mouse.coords, egui_mouse_position);
//     // check for mouse position changes
//     if input_state.global_mouse_position[0] != mouse.coords.0
//         || input_state.global_mouse_position[1] != mouse.coords.1
//     {
//         events.push(Event::PointerMoved(egui_mouse_position));
//         input_state.global_mouse_position = make_vec2(&[mouse.coords.0, mouse.coords.1]);
//         // dbg!(egui_mouse_position, input_state.global_mouse_position, input_state.window_pos);
//     }
//     input.events = events;
// }

use device_query::DeviceState;
use egui::Key;
use nalgebra_glm::I32Vec2;

pub struct GlobalInputState {
    pub global_mouse_position: I32Vec2,
    pub dq: DeviceState,
}
impl GlobalInputState {
    pub fn new() -> Self {
        GlobalInputState {
            global_mouse_position: Default::default(),
            dq: DeviceState::new(),
        }
    }
}

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
