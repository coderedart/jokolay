use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use bevy_window::WindowDescriptor;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::NONE))
        .insert_resource(WindowDescriptor {
            width: 800.,
            height: 600.,
            title: "Bevy glfw example".to_string(), // ToDo
            transparent: true,
            ..Default::default()
        });
    // -> The following two lines are the only changes you need to replace winit with glfw
    // we disable winit plugin from the default plugins, as glfw will be the winoowing plugin here.
    app.add_plugins_with(DefaultPlugins, |group| group.disable::<WinitPlugin>());
    app.add_plugin(bevy_glfw::GlfwPlugin);
    // -> done. just continue with the usual setup of your app.
    app.run();
}
