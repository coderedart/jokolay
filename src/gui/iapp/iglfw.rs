//! Provides a glfw-based backend platform for imgui-rs. This crate is modeled
//! after the winit version.
//!
//! ## Usage
//!
//! 1. Initialize a `GlfwPlatform`
//! 2. Attach it to a glfw `Window`
//! 3. Optionally, enable platform clipboard integration
//! 4. Pass events to the platform (every frame)
//! 5. Call frame preparation (every frame)
//! 6. Call render preperation (every frame)
//!
//! ## Examples
//!
//! The [examples](https://github.com/aloucks/imgui-glfw-support/tree/master/examples) can be found on github.

use glfw::{
    Action, Cursor, CursorMode, Key as GlfwKey, Modifiers, MouseButton, StandardCursor, Window,
    WindowEvent,
};
use imgui::{BackendFlags, ConfigFlags, Context, ImString, Io, Key, Ui};

// pub struct GlfwPlatform {
// hidpi_mode: ActiveHiDpiMode,
// hidpi_factor: f64,
// }

// #[derive(Copy, Clone, Debug, PartialEq)]
// enum ActiveHiDpiMode {
//     Default,
//     Rounded,
//     Locked,
// }

// #[derive(Copy, Clone, Debug, PartialEq)]
// pub enum HiDpiMode {
//     /// The DPI factor from glfw is used directly without adjustment
//     Default,
//     /// The DPI factor from glfw is rounded to an integer value.
//     ///
//     /// This prevents the user interface from becoming blurry with non-integer scaling.
//     Rounded,
//     /// The DPI factor from glfw is ignored, and the included value is used instead.
//     ///
//     /// This is useful if you want to force some DPI factor (e.g. 1.0) and not care about the value
//     /// coming from glfw.
//     Locked(f64),
// }

struct Clipboard {
    window_ptr: *mut glfw::ffi::GLFWwindow,
}

impl imgui::ClipboardBackend for Clipboard {
    fn set(&mut self, s: &imgui::ImStr) {
        unsafe {
            glfw::ffi::glfwSetClipboardString(self.window_ptr, s.as_ptr());
        }
    }
    fn get(&mut self) -> std::option::Option<imgui::ImString> {
        unsafe {
            let s = glfw::ffi::glfwGetClipboardString(self.window_ptr);
            let s = std::ffi::CStr::from_ptr(s);
            let bytes = s.to_bytes();
            if !bytes.is_empty() {
                let v = String::from_utf8_lossy(bytes);
                Some(imgui::ImString::new(v))
            } else {
                None
            }
        }
    }
}

// impl HiDpiMode {
//     fn apply(&self, hidpi_factor: f64) -> (ActiveHiDpiMode, f64) {
//         match *self {
//             HiDpiMode::Default => (ActiveHiDpiMode::Default, hidpi_factor),
//             HiDpiMode::Rounded => (ActiveHiDpiMode::Rounded, hidpi_factor.round()),
//             HiDpiMode::Locked(value) => (ActiveHiDpiMode::Locked, value),
//         }
//     }
// }

// impl GlfwPlatform {
/// Initializes a glfw platform instance and configures imgui.
///
/// * backend flgs are updated
/// * keys are configured
/// * platform name is set
pub fn init(imgui: &mut Context) {
    let io = imgui.io_mut();
    io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
    io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
    io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
    io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
    io[Key::Tab] = GlfwKey::Tab as _;
    io[Key::LeftArrow] = GlfwKey::Left as _;
    io[Key::RightArrow] = GlfwKey::Right as _;
    io[Key::UpArrow] = GlfwKey::Up as _;
    io[Key::DownArrow] = GlfwKey::Down as _;
    io[Key::PageUp] = GlfwKey::PageUp as _;
    io[Key::PageDown] = GlfwKey::PageDown as _;
    io[Key::Home] = GlfwKey::Home as _;
    io[Key::End] = GlfwKey::End as _;
    io[Key::Insert] = GlfwKey::Insert as _;
    io[Key::Delete] = GlfwKey::Delete as _;
    io[Key::Backspace] = GlfwKey::Backspace as _;
    io[Key::Space] = GlfwKey::Space as _;
    io[Key::Enter] = GlfwKey::Enter as _;
    io[Key::Escape] = GlfwKey::Escape as _;
    io[Key::KeyPadEnter] = GlfwKey::KpEnter as _;
    io[Key::A] = GlfwKey::A as _;
    io[Key::C] = GlfwKey::C as _;
    io[Key::V] = GlfwKey::V as _;
    io[Key::X] = GlfwKey::X as _;
    io[Key::Y] = GlfwKey::Y as _;
    io[Key::Z] = GlfwKey::Z as _;
    imgui.set_platform_name(Some(ImString::from(format!(
        "imgui-glfw-support {}",
        env!("CARGO_PKG_VERSION")
    ))));
}

/// Adds platform clipboard integration for the provided window. The caller **must** ensure that
/// the `Window` outlives the imgui `Context` **and** that any imgui functions that may access
/// the clipboard are called from the **main thread** (the thread that's executing the event polling).
pub unsafe fn set_clipboard_backend(imgui: &mut Context, window: &Window) {
    use glfw::Context;
    let window_ptr = window.window_ptr();
    imgui.set_clipboard_backend(Box::new(Clipboard { window_ptr }));
}

/// Attaches the platform instance to a glfw window.
///
/// * framebuffer sacle (i.e. DPI factor) is set
/// * display size is set
pub fn attach_window(io: &mut Io, window: &Window) {
    // let (scale_factor_x, _scale_factor_y) = window.get_content_scale();
    // let (hidpi_mode, hidpi_factor) = hidpi_mode.apply(scale_factor_x as _);
    // self.hidpi_mode = hidpi_mode;
    // self.hidpi_factor = hidpi_factor;
    // io.display_framebuffer_scale = [hidpi_factor as f32, hidpi_factor as f32];
    let (width, height) = window.get_size();
    io.display_size = [width as f32, height as f32];
}

/// Handles a glfw window event
///
/// * keyboard state is updated
/// * mouse state is updated
pub fn handle_event(io: &mut Io, event: &WindowEvent) {
    match *event {
        WindowEvent::Key(key, _scancode, action, modifiers) => {
            if key as i32 >= 0 {
                if action == Action::Release {
                    io.keys_down[key as usize] = false;
                } else {
                    io.keys_down[key as usize] = true;
                }
            }
            io.key_shift = modifiers.contains(Modifiers::Shift);
            io.key_ctrl = modifiers.contains(Modifiers::Control);
            io.key_alt = modifiers.contains(Modifiers::Alt);
            io.key_super = modifiers.contains(Modifiers::Super);
        }
        WindowEvent::Size(width, height) => {
            io.display_size = [width as _, height as _];
        }
        WindowEvent::Char(ch) => {
            // Exclude the backspace key
            if ch != '\u{7f}' {
                io.add_input_character(ch);
            }
        }
        WindowEvent::CursorPos(x, y) => {
            io.mouse_pos = [x as _, y as _];
        }
        WindowEvent::Scroll(x, y) => {
            io.mouse_wheel_h = x as _;
            io.mouse_wheel = y as _;
        }
        WindowEvent::MouseButton(button, action, _modifiers) => {
            let pressed = action == Action::Press;
            match button {
                MouseButton::Button1 => io.mouse_down[0] = pressed,
                MouseButton::Button2 => io.mouse_down[1] = pressed,
                MouseButton::Button3 => io.mouse_down[2] = pressed,
                _ => (),
            }
        }
        _ => {}
    }
}

/// Prepare the window for the next frame.
///
/// Call before calling the imgui-rs `Context::frame` function.
///
/// * mouse cursor is repositioned if requested by imgui
pub fn prepare_frame(io: &mut Io, window: &mut Window) -> Result<(), String> {
    if io.want_set_mouse_pos {
        let [x, y] = io.mouse_pos;
        window.set_cursor_pos(x as _, y as _);
        Ok(())
    } else {
        Ok(())
    }
}

/// Prepare the window for rendering.
///
/// Call before calling the imgui backend renderer function (e.g. `imgui_wgpu::Renderer::render`).
///
/// * the mouse cursor is changed or hidden if requested by imgui
pub fn prepare_render(ui: &Ui, window: &mut Window) {
    let io = ui.io();
    if !io
        .config_flags
        .contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE)
    {
        match ui.mouse_cursor() {
            Some(mouse_cursor) if !io.mouse_draw_cursor => {
                window.set_cursor_mode(CursorMode::Normal);
                window.set_cursor(Some(match mouse_cursor {
                    // TODO: GLFW has more cursor options on master, but they aren't released yet
                    imgui::MouseCursor::Arrow => Cursor::standard(StandardCursor::Arrow),
                    imgui::MouseCursor::ResizeAll => Cursor::standard(StandardCursor::Arrow),
                    imgui::MouseCursor::ResizeNS => Cursor::standard(StandardCursor::VResize),
                    imgui::MouseCursor::ResizeEW => Cursor::standard(StandardCursor::HResize),
                    imgui::MouseCursor::ResizeNESW => Cursor::standard(StandardCursor::Arrow),
                    imgui::MouseCursor::ResizeNWSE => Cursor::standard(StandardCursor::Arrow),
                    imgui::MouseCursor::Hand => Cursor::standard(StandardCursor::Hand),
                    imgui::MouseCursor::NotAllowed => Cursor::standard(StandardCursor::Crosshair),
                    imgui::MouseCursor::TextInput => Cursor::standard(StandardCursor::IBeam),
                }));
            }
            _ => window.set_cursor_mode(CursorMode::Hidden),
        }
    }
}
// }
