extern crate core;

mod converters;
mod events;
mod glfw_windows;

use bevy_log::info;
pub use glfw_windows::*;

use bevy_app::{App, AppExit, CoreStage, Plugin};
use bevy_ecs::{
    event::{Events, ManualEventReader},
    system::IntoExclusiveSystem,
    world::World,
};
use bevy_utils::tracing::trace;
use bevy_window::{CreateWindow, WindowCreated, WindowScaleFactorChanged, Windows};

use crate::converters::{
    convert_cursor_icon, convert_element_state, convert_mouse_button, convert_virtual_key_code,
};
use crate::events::handle_glfw_events;
pub use crate::glfw_windows::{GlfwBackend, WindowState};
use glfw::*;

/// The GlfwPlugin will add support for using glfw-rs as a window backend to create and run windows.
/// It will create a new `GlfwBackend` insert it into app, set the `glfw_runner` functions as the runner,
/// and add the window command handler function `change_window` to PostUpdate stage, and finally,
/// handle any create window events for the first time while building the plugin.
#[derive(Default)]
pub struct GlfwPlugin;

impl Plugin for GlfwPlugin {
    fn build(&self, app: &mut App) {
        let glfw_backend = GlfwBackend::new();
        app.insert_non_send_resource(glfw_backend);
        app.set_runner(glfw_runner_with)
            .add_system_to_stage(CoreStage::PostUpdate, change_window.exclusive_system());
        handle_create_window_events(&mut app.world);
    }
}

/// This system will run the windowing related commands such as resizing or decorations etc..
/// will be run during the PostUpdate stage.
fn change_window(world: &mut World) {
    let world = world.cell();
    let mut glfw_windows = world.get_non_send_mut::<GlfwBackend>().unwrap();
    let mut windows = world.get_resource_mut::<Windows>().unwrap();

    for bevy_window in windows.iter_mut() {
        let id = bevy_window.id();
        let window: &mut Window = match glfw_windows.windows.get_mut(&id) {
            Some(state) => &mut state.window,
            None => panic!(
                "failed to find window. list of windows: {:#?}",
                &glfw_windows.windows
            ),
        };
        for command in bevy_window.drain_commands() {
            match command {
                bevy_window::WindowCommand::SetWindowMode {
                    mode: _,
                    resolution: (_width, _height),
                } => {
                    unimplemented!();
                }
                bevy_window::WindowCommand::SetTitle { title } => {
                    window.set_title(&title);
                }
                bevy_window::WindowCommand::SetScaleFactor { scale_factor } => {
                    let mut window_dpi_changed_events = world
                        .get_resource_mut::<Events<WindowScaleFactorChanged>>()
                        .unwrap();
                    window_dpi_changed_events.send(WindowScaleFactorChanged { id, scale_factor });
                }
                bevy_window::WindowCommand::SetResolution {
                    logical_resolution: (width, height),
                    scale_factor: _,
                } => {
                    window.set_size(width as i32, height as i32);
                }
                bevy_window::WindowCommand::SetPresentMode { .. } => (),
                bevy_window::WindowCommand::SetResizable { resizable } => {
                    window.set_resizable(resizable);
                }
                bevy_window::WindowCommand::SetDecorations { decorations } => {
                    window.set_decorated(decorations)
                }
                bevy_window::WindowCommand::SetCursorIcon { icon } => {
                    window.set_cursor(Some(Cursor::standard(convert_cursor_icon(icon))));
                }
                bevy_window::WindowCommand::SetCursorLockMode { locked: _ } => {
                    panic!("failed to lock cursor")
                }
                bevy_window::WindowCommand::SetCursorVisibility { visible: _ } => {
                    panic!("failed ot set cursor visibility")
                }
                bevy_window::WindowCommand::SetCursorPosition { position } => {
                    window.set_cursor_pos(position.x as f64, position.y as f64);
                }
                bevy_window::WindowCommand::SetMaximized { maximized } => {
                    if maximized {
                        window.maximize();
                    } else {
                        window.restore();
                    }
                }
                bevy_window::WindowCommand::SetMinimized { minimized } => {
                    if minimized {
                        window.iconify();
                    } else {
                        window.restore();
                    }
                }
                bevy_window::WindowCommand::SetPosition { position } => {
                    window.set_pos(position.x, position.y);
                }
                bevy_window::WindowCommand::SetResizeConstraints {
                    resize_constraints: _,
                } => {
                    unimplemented!()
                }
            }
        }
    }
}

/// This drives the event loop. polls the events, and calls bevy::app::update()
/// after running out of events this frame.
pub fn glfw_runner_with(mut app: App) {
    let mut app_exit_event_reader = ManualEventReader::<AppExit>::default();

    trace!("Entering glfw event loop");

    loop {
        handle_create_window_events(&mut app.world);
        {
            // check if there's any new events
            app.world
                .non_send_resource_mut::<GlfwBackend>()
                .glfw_context
                .poll_events();
            // check if we need to exit app
            let app_exit_events = app.world.resource::<Events<AppExit>>();
            if app_exit_event_reader.iter(app_exit_events).last().is_some() {
                break;
            }
        }

        {
            let world = app.world.cell();
            let mut glfw_backend = world
                .get_non_send_mut::<GlfwBackend>()
                .expect("failed to get glfw windows in event loop");
            for (window_id, window_state) in glfw_backend.windows.iter_mut() {
                let mut bevy_window_list = world.get_resource_mut::<Windows>().unwrap();
                // bevy window
                let b_window = bevy_window_list
                    .get_mut(*window_id)
                    .expect("failed to get bevy_window from Windows");
                handle_glfw_events(&world, *window_id, b_window, window_state);
            }
        }
        app.update();
    }
    // if window is closed before its surface is destroyed, then we get a segmentation fault.
    // so, we extract the windows and drop the app first to avoid that.
    // upon panic, app will be dropped directly and lead to segmentation fault.
    let glfw_backend = app.world.remove_non_send_resource::<GlfwBackend>();
    dbg!("extracted glfw backend");
    drop(app);
    dbg!("dropped app");
    drop(glfw_backend);
    dbg!("dropped glfw backend");
}

/// This handles events related to creating a window BEFORE entering the event loop.
/// so, it is only run when we add the GlfwPlugin to the app.
fn handle_create_window_events(world: &mut World) {
    let world = world.cell();
    for create_window_event in world
        .get_resource_mut::<Events<CreateWindow>>()
        .unwrap()
        .drain()
    {
        let mut windows = world.get_resource_mut::<Windows>().unwrap();
        let mut window_created_events = world.get_resource_mut::<Events<WindowCreated>>().unwrap();
        let window = world
            .get_non_send_mut::<GlfwBackend>()
            .expect("failed to get glfw context from world")
            .create_window(create_window_event.id, &create_window_event.descriptor);
        windows.add(window);
        info!(
            "new glfw window created with window_id: {:#?}",
            &create_window_event.id
        );
        window_created_events.send(WindowCreated {
            id: create_window_event.id,
        });
    }
}
