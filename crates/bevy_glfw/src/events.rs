use crate::{convert_element_state, convert_mouse_button, convert_virtual_key_code, WindowState};
use bevy_ecs::event::Events;
use bevy_ecs::world::WorldCell;
use bevy_input::keyboard::KeyboardInput;
use bevy_input::mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel};
use bevy_math::dvec2;
use bevy_window::{
    CursorEntered, CursorLeft, CursorMoved, FileDragAndDrop, ReceivedCharacter, Window,
    WindowBackendScaleFactorChanged, WindowCloseRequested, WindowFocused, WindowId, WindowMoved,
    WindowResized, WindowScaleFactorChanged,
};
use glfw::WindowEvent;
/// Glfw and Bevy event conversion.
/// on kde x11 with FHD 24" monitor, with scale 1.0, cursor position was given by glfw in same
/// coordinates as the framebuffer size.
/// on kde x11 with FHD 14" laptop, with scale 1.5 and 2.0, cursor position was also using the
/// same physical coordinates. so, we only need to use logical coords for event writers. window
/// update backend functions of bevy Window use physical coords for both size and cursor position,
/// so that needs no conversion.
/// still need to check on 4k displays and non-kde / x11 backends.
pub fn handle_glfw_events(
    world: &WorldCell,
    window_id: WindowId,
    bwindow: &mut Window,
    window_state: &mut WindowState,
) {
    // it seems bevy gives cursor position in physical pixel coordinates instead of logical coords.
    // so, we divide by scale to get logical.
    if let Some(updated_cursor_position) = window_state.update_cursor_position() {
        // WARNING: same code used inside the events iteartion below for the CursorPos update
        // so, if anything needs to be changed here, then it needs to be changed there too.
        let (x, y) = (updated_cursor_position.x, updated_cursor_position.y);
        // bevy wants origin to be bottom left.
        // but glfw uses top left as origin.
        // flip y to transform to bevy space
        let height = window_state.dimensions.y as f32;
        let position: bevy_math::Vec2 = [x as f32, height as f32 - y as f32].into();

        bwindow.update_cursor_physical_position_from_backend(Some(position.as_dvec2()));

        world
            .get_resource_mut::<Events<CursorMoved>>()
            .unwrap()
            .send(CursorMoved {
                id: window_id,
                position: position / bwindow.scale_factor() as f32,
            });
    }
    for (_event_timestamp, event) in glfw::flush_messages(&window_state.events_receiver) {
        match event {
            WindowEvent::Pos(x, y) => {
                bwindow.update_actual_position_from_backend([x, y].into());
                world
                    .get_resource_mut::<Events<WindowMoved>>()
                    .unwrap()
                    .send(WindowMoved {
                        id: window_id,
                        position: [x, y].into(),
                    })
            }
            WindowEvent::Size(w, h) => {
                window_state.dimensions = [w as f64, h as f64].into();
                world
                    .get_resource_mut::<Events<WindowResized>>()
                    .unwrap()
                    .send(WindowResized {
                        id: window_id,
                        // from what i understand, glfw's screen coordinates are winit's logical size. so, no conversion with scale needed.
                        width: w as f32,
                        height: h as f32,
                    });
            }
            WindowEvent::FramebufferSize(w, h) => {
                bwindow.update_actual_size_from_backend(w as u32, h as u32);
            }
            WindowEvent::Close => {
                world
                    .get_resource_mut::<Events<WindowCloseRequested>>()
                    .unwrap()
                    .send(WindowCloseRequested { id: window_id });
            }
            WindowEvent::Focus(focused) => {
                bwindow.update_focused_status_from_backend(focused);
                world
                    .get_resource_mut::<Events<WindowFocused>>()
                    .unwrap()
                    .send(WindowFocused {
                        id: window_id,
                        focused,
                    });
            }
            WindowEvent::MouseButton(button, action, _) => {
                world
                    .get_resource_mut::<Events<MouseButtonInput>>()
                    .unwrap()
                    .send(MouseButtonInput {
                        button: convert_mouse_button(button),
                        state: convert_element_state(action),
                    });
            }
            WindowEvent::CursorPos(x, y) => {
                window_state.cursor_position = dvec2(x, y);
                // bevy wants origin to be bottom left.
                // but glfw uses top left as origin.
                // flip y to transform to bevy space
                let height = window_state.dimensions.y as f32;
                let position: bevy_math::Vec2 = [x as f32, height as f32 - y as f32].into();

                bwindow.update_cursor_physical_position_from_backend(Some(position.as_dvec2()));
                world
                    .get_resource_mut::<Events<CursorMoved>>()
                    .unwrap()
                    .send(CursorMoved {
                        id: window_id,
                        position: position / bwindow.scale_factor() as f32,
                    });
            }
            WindowEvent::CursorEnter(entered) => {
                if entered {
                    world
                        .get_resource_mut::<Events<CursorEntered>>()
                        .unwrap()
                        .send(CursorEntered { id: window_id });
                } else {
                    bwindow.update_cursor_physical_position_from_backend(None);
                    world
                        .get_resource_mut::<Events<CursorLeft>>()
                        .unwrap()
                        .send(CursorLeft { id: window_id });
                }
            }
            WindowEvent::Scroll(x, y) => {
                world
                    .get_resource_mut::<Events<MouseWheel>>()
                    .unwrap()
                    .send(MouseWheel {
                        unit: MouseScrollUnit::Line,
                        x: x as f32,
                        y: y as f32,
                    });
            }
            WindowEvent::Key(k, scan_code, action, _) => {
                world
                    .get_resource_mut::<Events<KeyboardInput>>()
                    .unwrap()
                    .send(KeyboardInput {
                        scan_code: scan_code as u32,
                        key_code: Some(convert_virtual_key_code(k)),
                        state: convert_element_state(action),
                    });
            }
            WindowEvent::Char(c) => {
                world
                    .get_resource_mut::<Events<ReceivedCharacter>>()
                    .unwrap()
                    .send(ReceivedCharacter {
                        id: window_id,
                        char: c,
                    });
            }
            WindowEvent::FileDrop(dropped_file_paths) => {
                let mut file_drop_events_writer =
                    world.get_resource_mut::<Events<FileDragAndDrop>>().unwrap();
                for file_path in dropped_file_paths {
                    file_drop_events_writer.send(FileDragAndDrop::DroppedFile {
                        id: window_id,
                        path_buf: file_path,
                    });
                }
            }
            WindowEvent::ContentScale(x, y) => {
                assert_eq!(x, y, "content scale {x} and {y} are not equal");
                bwindow.update_scale_factor_from_backend(x as f64);
                world
                    .get_resource_mut::<Events<WindowScaleFactorChanged>>()
                    .unwrap()
                    .send(WindowScaleFactorChanged {
                        id: window_id,
                        scale_factor: x as f64,
                    });
                world
                    .get_resource_mut::<Events<WindowBackendScaleFactorChanged>>()
                    .unwrap()
                    .send(WindowBackendScaleFactorChanged {
                        id: window_id,
                        scale_factor: x as f64,
                    });
            }
            _rest => {}
        }
    }
}
