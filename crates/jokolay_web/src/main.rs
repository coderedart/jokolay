// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::{debug, App, Local, Msaa, ResMut};

use bevy_egui::{EguiContext, EguiPlugin};

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa { samples: 1 });

    app.add_plugins(bevy::DefaultPlugins);

    // add the rest of the stuff which is used regardless of the platform
    app.add_plugin(EguiPlugin);
    app.add_system(temp_window); // to have *something* be shown when we launch the app. eventually removed before release. TODO

    app.run();
}

fn temp_window(mut print_count: Local<usize>, mut ectx: ResMut<EguiContext>) {
    let ctx = ectx.ctx_mut();
    bevy_egui::egui::Window::new("title").show(ctx, |ui| {
        ui.label("hello");

        if ui.button("print something").clicked() {
            *print_count += 1;
            let count = *print_count;
            debug!("button_clicked_count: {count}");
        }
    });
}
