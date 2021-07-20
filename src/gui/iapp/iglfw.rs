use glfw::{Cursor, CursorMode, Key as GlfwKey, StandardCursor, Window};
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

pub fn set_imgui_style(style: &mut imgui::Style) {
    style.window_padding = [15.0, 15.0];
    style.window_rounding = 5.0;
    style.frame_padding = [5.0, 5.0];
    style.frame_rounding = 4.0;
    style.item_spacing = [12.0, 8.0];
    style.item_inner_spacing = [8.0, 6.0];
    style.indent_spacing = 25.0;
    style.scrollbar_size = 15.0;
    style.scrollbar_rounding = 9.0;
    style.grab_min_size = 5.0;
    style.grab_rounding = 3.0;
 
    style.colors[imgui::sys::ImGuiCol_Text as usize as usize] = [0.80, 0.80, 0.83, 1.00];
    style.colors[imgui::sys::ImGuiCol_TextDisabled as usize] = [0.24, 0.23, 0.29, 1.00];
    style.colors[imgui::sys::ImGuiCol_WindowBg as usize] = [0.06, 0.05, 0.07, 1.00];
    style.colors[imgui::sys::ImGuiCol_ChildBg as usize] = [0.07, 0.07, 0.09, 1.00];
    style.colors[imgui::sys::ImGuiCol_PopupBg as usize] = [0.07, 0.07, 0.09, 1.00];
    style.colors[imgui::sys::ImGuiCol_Border as usize] = [0.80, 0.80, 0.83, 0.88];
    style.colors[imgui::sys::ImGuiCol_BorderShadow as usize] = [0.92, 0.91, 0.88, 0.00];
    style.colors[imgui::sys::ImGuiCol_FrameBg as usize] = [0.10, 0.09, 0.12, 1.00];
    style.colors[imgui::sys::ImGuiCol_FrameBgHovered as usize] = [0.24, 0.23, 0.29, 1.00];
    style.colors[imgui::sys::ImGuiCol_FrameBgActive as usize] = [0.56, 0.56, 0.58, 1.00];
    style.colors[imgui::sys::ImGuiCol_TitleBg as usize] = [0.10, 0.09, 0.12, 1.00];
    style.colors[imgui::sys::ImGuiCol_TitleBgCollapsed as usize] = [1.00, 0.98, 0.95, 0.75];
    style.colors[imgui::sys::ImGuiCol_TitleBgActive as usize] = [0.07, 0.07, 0.09, 1.00];
    style.colors[imgui::sys::ImGuiCol_MenuBarBg as usize] = [0.10, 0.09, 0.12, 1.00];
    style.colors[imgui::sys::ImGuiCol_ScrollbarBg as usize] = [0.10, 0.09, 0.12, 1.00];
    style.colors[imgui::sys::ImGuiCol_ScrollbarGrab as usize] = [0.80, 0.80, 0.83, 0.31];
    style.colors[imgui::sys::ImGuiCol_ScrollbarGrabHovered as usize] = [0.56, 0.56, 0.58, 1.00];
    style.colors[imgui::sys::ImGuiCol_ScrollbarGrabActive as usize] = [0.06, 0.05, 0.07, 1.00];
    // style.colors[imgui::sys::ImGuiCol_ComboBg as usize] = [0.19, 0.18, 0.21, 1.00];
    style.colors[imgui::sys::ImGuiCol_CheckMark as usize] = [0.80, 0.80, 0.83, 0.31];
    style.colors[imgui::sys::ImGuiCol_SliderGrab as usize] = [0.80, 0.80, 0.83, 0.31];
    style.colors[imgui::sys::ImGuiCol_SliderGrabActive as usize] = [0.06, 0.05, 0.07, 1.00];
    style.colors[imgui::sys::ImGuiCol_Button as usize] = [0.10, 0.09, 0.12, 1.00];
    style.colors[imgui::sys::ImGuiCol_ButtonHovered as usize] = [0.24, 0.23, 0.29, 1.00];
    style.colors[imgui::sys::ImGuiCol_ButtonActive as usize] = [0.56, 0.56, 0.58, 1.00];
    style.colors[imgui::sys::ImGuiCol_Header as usize] = [0.10, 0.09, 0.12, 1.00];
    style.colors[imgui::sys::ImGuiCol_HeaderHovered as usize] = [0.56, 0.56, 0.58, 1.00];
    style.colors[imgui::sys::ImGuiCol_HeaderActive as usize] = [0.06, 0.05, 0.07, 1.00];
    // style.colors[imgui::sys::ImGuiCol_Column as usize] = [0.56, 0.56, 0.58, 1.00];
    // style.colors[imgui::sys::ImGuiCol_ColumnHovered as usize] = [0.24, 0.23, 0.29, 1.00];
    // style.colors[imgui::sys::ImGuiCol_ColumnActive as usize] = [0.56, 0.56, 0.58, 1.00];
    style.colors[imgui::sys::ImGuiCol_ResizeGrip as usize] = [0.00, 0.00, 0.00, 0.00];
    style.colors[imgui::sys::ImGuiCol_ResizeGripHovered as usize] = [0.56, 0.56, 0.58, 1.00];
    style.colors[imgui::sys::ImGuiCol_ResizeGripActive as usize] = [0.06, 0.05, 0.07, 1.00];
    // style.colors[imgui::sys::ImGuiCol_CloseButton as usize] = [0.40, 0.39, 0.38, 0.16];
    // style.colors[imgui::sys::ImGuiCol_CloseButtonHovered as usize] = [0.40, 0.39, 0.38, 0.39];
    // style.colors[imgui::sys::ImGuiCol_CloseButtonActive as usize] = [0.40, 0.39, 0.38, 1.00];
    style.colors[imgui::sys::ImGuiCol_PlotLines as usize] = [0.40, 0.39, 0.38, 0.63];
    style.colors[imgui::sys::ImGuiCol_PlotLinesHovered as usize] = [0.25, 1.00, 0.00, 1.00];
    style.colors[imgui::sys::ImGuiCol_PlotHistogram as usize] = [0.40, 0.39, 0.38, 0.63];
    style.colors[imgui::sys::ImGuiCol_PlotHistogramHovered as usize] = [0.25, 1.00, 0.00, 1.00];
    style.colors[imgui::sys::ImGuiCol_TextSelectedBg as usize] = [0.25, 1.00, 0.00, 0.43];
    // style.colors[imgui::sys::ImGuiCol_ModalWindowDarkening as usize] = [1.00, 0.98, 0.95, 0.73];
}

/*
arcdps mock colors
ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, { 4.f, 4.f });
	ImGui::PushStyleVar(ImGuiStyleVar_WindowBorderSize, 0.f);
	ImGui::PushStyleVar(ImGuiStyleVar_WindowMinSize, { 5.f, 3.f });
	ImGui::PushStyleVar(ImGuiStyleVar_ChildBorderSize, 0.f);
	ImGui::PushStyleVar(ImGuiStyleVar_PopupBorderSize, 0.f);
	ImGui::PushStyleVar(ImGuiStyleVar_FramePadding, { 4.f, 4.f });
	ImGui::PushStyleVar(ImGuiStyleVar_ItemSpacing, { 5.f, 3.f });
	ImGui::PushStyleVar(ImGuiStyleVar_ItemInnerSpacing, { 5.f, 3.f });
	ImGui::PushStyleVar(ImGuiStyleVar_IndentSpacing, 25.f);
	ImGui::PushStyleVar(ImGuiStyleVar_ScrollbarSize, 9.f);
	ImGui::PushStyleVar(ImGuiStyleVar_ScrollbarRounding, 0.f);
	ImGui::PushStyleVar(ImGuiStyleVar_GrabMinSize, 25.f);
	ImGui::PushStyleVar(ImGuiStyleVar_TabRounding, 0.f);

	ImGui::PushStyleColor(ImGuiCol_Text, ImVec4(0.8f, 0.8f, 0.83f, 1.f));
	ImGui::PushStyleColor(ImGuiCol_TextDisabled, ImVec4(0.24f, 0.23f, 0.29f, 1.f));
	ImGui::PushStyleColor(ImGuiCol_WindowBg, ImVec4(0.06f, 0.05f, 0.07f, 0.75f));
	ImGui::PushStyleColor(ImGuiCol_ChildBg, ImVec4(0.07f, 0.07f, 0.09f, 0.f));
	ImGui::PushStyleColor(ImGuiCol_PopupBg, ImVec4(0.07f, 0.07f, 0.09f, 0.85f));
	ImGui::PushStyleColor(ImGuiCol_Border, ImVec4(0.64f, 0.57f, 0.7f, 0.2f));
	ImGui::PushStyleColor(ImGuiCol_BorderShadow, ImVec4(0.64f, 0.62f, 0.67f, 0.f));
	ImGui::PushStyleColor(ImGuiCol_FrameBg, ImVec4(0.62f, 0.6f, 0.65f, 0.2f));
	ImGui::PushStyleColor(ImGuiCol_FrameBgHovered, ImVec4(0.62f, 0.6f, 0.65f, 0.75f));
	ImGui::PushStyleColor(ImGuiCol_FrameBgActive, ImVec4(0.56f, 0.56f, 0.58f, 0.75f));
	ImGui::PushStyleColor(ImGuiCol_TitleBg, ImVec4(0.1f, 0.09f, 0.12f, 0.85f));
	ImGui::PushStyleColor(ImGuiCol_TitleBgActive, ImVec4(0.1f, 0.09f, 0.12f, 0.85f));
	ImGui::PushStyleColor(ImGuiCol_TitleBgCollapsed, ImVec4(0.1f, 0.09f, 0.12f, 0.85f));
	ImGui::PushStyleColor(ImGuiCol_MenuBarBg, ImVec4(0.1f, 0.09f, 0.12f, 0.7f));
	ImGui::PushStyleColor(ImGuiCol_ScrollbarBg, ImVec4(0.1f, 0.09f, 0.12f, 0.8f));
	ImGui::PushStyleColor(ImGuiCol_ScrollbarGrab, ImVec4(0.46f, 0.45f, 0.47f, 0.78f));
	ImGui::PushStyleColor(ImGuiCol_ScrollbarGrabHovered, ImVec4(0.67f, 0.67f, 0.69f, 0.78f));
	ImGui::PushStyleColor(ImGuiCol_ScrollbarGrabActive, ImVec4(0.78f, 0.78f, 0.8f, 0.78f));
	ImGui::PushStyleColor(ImGuiCol_CheckMark, ImVec4(0.8f, 0.8f, 0.83f, 0.81f));
	ImGui::PushStyleColor(ImGuiCol_SliderGrab, ImVec4(0.8f, 0.8f, 0.83f, 0.31f));
	ImGui::PushStyleColor(ImGuiCol_SliderGrabActive, ImVec4(0.06f, 0.05f, 0.07f, 1.f));
	ImGui::PushStyleColor(ImGuiCol_Button, ImVec4(0.62f, 0.6f, 0.65f, 0.3f));
	ImGui::PushStyleColor(ImGuiCol_ButtonHovered, ImVec4(0.62f, 0.6f, 0.65f, 0.6f));
	ImGui::PushStyleColor(ImGuiCol_ButtonActive, ImVec4(0.62f, 0.6f, 0.65f, 0.9f));
	ImGui::PushStyleColor(ImGuiCol_Header, ImVec4(0.36f, 0.36f, 0.38f, 0.7f));
	ImGui::PushStyleColor(ImGuiCol_HeaderHovered, ImVec4(0.36f, 0.36f, 0.38f, 0.35f));
	ImGui::PushStyleColor(ImGuiCol_HeaderActive, ImVec4(0.36f, 0.36f, 0.38f, 0.7f));
	ImGui::PushStyleColor(ImGuiCol_ResizeGrip, ImVec4(0.f, 0.f, 0.f, 0.f));
	ImGui::PushStyleColor(ImGuiCol_ResizeGripHovered, ImVec4(0.56f, 0.56f, 0.58f, 1.f));
	ImGui::PushStyleColor(ImGuiCol_ResizeGripActive, ImVec4(0.06f, 0.05f, 0.07f, 1.f));
	ImGui::PushStyleColor(ImGuiCol_Tab, ImVec4(0.7f, 0.68f, 0.69f, 0.1f));
	ImGui::PushStyleColor(ImGuiCol_TabHovered, ImVec4(0.7f, 0.68f, 0.69f, 0.3f));
	ImGui::PushStyleColor(ImGuiCol_TabActive, ImVec4(0.7f, 0.68f, 0.69f, 0.43f));
	ImGui::PushStyleColor(ImGuiCol_PlotLines, ImVec4(0.7f, 0.68f, 0.66f, 0.56f));
	ImGui::PushStyleColor(ImGuiCol_PlotLinesHovered, ImVec4(0.25f, 1.f, 0.f, 1.f));
	ImGui::PushStyleColor(ImGuiCol_PlotHistogram, ImVec4(0.7f, 0.68f, 0.66f, 0.48f));
	ImGui::PushStyleColor(ImGuiCol_PlotHistogramHovered, ImVec4(0.25f, 1.f, 0.f, 1.f));
	ImGui::PushStyleColor(ImGuiCol_TextSelectedBg, ImVec4(0.36f, 0.36f, 0.88f, 0.55f));
*/

/// Prepare the window for rendering.
///
/// Call before calling the imgui backend renderer function (e.g. `imgui_wgpu::Renderer::render`).
///
/// * the mouse cursor is changed or hidden if requested by imgui
pub fn mouse_cursor_change(ui: &Ui, window: &mut Window) {
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
