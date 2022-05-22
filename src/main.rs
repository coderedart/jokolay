// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::app::CoreStage;
use bevy::prelude::{
    App, ClearColor, Color, Msaa, NonSendMut, Res, ResMut, WindowDescriptor, Windows,
};

use bevy::window::PresentMode;
use bevy::DefaultPlugins;
use bevy_egui::{EguiContext, EguiPlugin};

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor(Color::NONE))
        .insert_resource(WindowDescriptor {
            width: 800.,
            height: 600.,
            title: "Bevy game".to_string(), // ToDo
            present_mode: PresentMode::Fifo,
            transparent: true,
            ..Default::default()
        });
    // only enable bevy if targeting wasm. otherwise, disable bevy from the default plugins
    app.add_plugins_with(DefaultPlugins, |group| {
        #[cfg(not(target_arch = "wasm32"))]
        group.disable::<bevy::winit::WinitPlugin>();

        group
    });
    // only enable glfw if we are targeting a non-wasm platform. otherwise, we will use winit above
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugin(bevy_glfw::GlfwPlugin);

    app.add_plugin(EguiPlugin);
    app.add_system_to_stage(CoreStage::Last, egui_glfw_passthrough);
    app.run();
}
#[cfg(not(target_arch = "wasm32"))]
fn egui_glfw_passthrough(
    mut ectx: ResMut<EguiContext>,
    mut glfw_backend: NonSendMut<bevy_glfw::GlfwBackend>,
    windows: Res<Windows>,
) {
    for win in windows.iter() {
        let window_id = win.id();
        let ctx = ectx.ctx_for_window_mut(window_id);
        if let Some(window_state) = glfw_backend.get_window_mut(&window_id) {
            if ctx.wants_keyboard_input() || ctx.wants_pointer_input() || ctx.is_using_pointer() {
                window_state.set_passthrough(false);
            } else {
                window_state.set_passthrough(true);
            }
        }
    }
}
