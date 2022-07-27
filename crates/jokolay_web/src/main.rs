// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::{App, Local, Msaa, ResMut, Windows};

use bevy_egui::{EguiContext, EguiPlugin};

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 1 });

    // if on desktop, let joko_desktop take care of adding the plugins like first and some resources that default plugins use
    #[cfg(not(target_arch = "wasm32"))]
    joko_desktop::add_desktop_addons(&mut app);

    // if on wasm, just go with the default plugins with winit.
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(bevy::DefaultPlugins);

    // add the rest of the stuff which is used regardless of the platform
    app.add_plugin(EguiPlugin);
    app.add_system(temp_window); // to have *something* be shown when we launch the app. eventually removed before release. TODO

    app.run();
}

fn temp_window(
    mut print_count: Local<usize>,
    mut ectx: ResMut<EguiContext>,
    mut windows: ResMut<Windows>,
) {
    let ctx = ectx.ctx_mut();
    bevy_egui::egui::Window::new("title").show(ctx, |ui| {
        ui.label("hello");
        if ui.button("enable decorations").clicked() {
            windows.get_primary_mut().unwrap().set_decorations(true);
        }
        if ui.button("disable decorations").clicked() {
            windows.get_primary_mut().unwrap().set_decorations(false);
        }
        if ui.button("print something").clicked() {
            *print_count += 1;
            dbg!(print_count);
        }
    });
}
