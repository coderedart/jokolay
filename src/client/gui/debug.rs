use egui::CtxRef;

use crate::{
    client::JokoClient,
    core::{window::WindowCommand, CoreFrameCommands},
};

#[derive(Debug)]
pub struct DebugWindow {
    pub window_name: &'static str,
    pub vsync: glfw::SwapInterval,
}
impl Default for DebugWindow {
    fn default() -> Self {
        Self {
            window_name: "Debug Window",
            vsync: glfw::SwapInterval::Sync(1),
        }
    }
}

#[cfg(debug_assertions)]
pub fn show_debug_window(
    ctx: CtxRef,
    client: &mut JokoClient,
    average_fps: usize,
    cfc: &mut CoreFrameCommands,
) {
    egui::Window::new(client.debug_window.window_name)
        .scroll2([true, true])
        .show(&ctx, |ui| {
            ui.label("average fps: ");
            ui.add(egui::widgets::DragValue::new(&mut (average_fps as f32)));
            ui.horizontal_wrapped(|ui| {
                ui.radio_value(
                    &mut client.debug_window.vsync,
                    glfw::SwapInterval::Sync(1),
                    "vsync",
                );
                ui.radio_value(
                    &mut client.debug_window.vsync,
                    glfw::SwapInterval::Adaptive,
                    "adaptive",
                );
                ui.radio_value(
                    &mut client.debug_window.vsync,
                    glfw::SwapInterval::None,
                    "no_sync",
                );
                if ui.button("apply vsync mode").clicked() {
                    cfc.window_commads
                        .push(WindowCommand::SwapInterval(client.debug_window.vsync));
                }
            });
            let mut soft_restart = client
                .soft_restart
                .load(std::sync::atomic::Ordering::Relaxed);
            if ui.checkbox(&mut soft_restart, "soft restart").changed() {
                client
                    .soft_restart
                    .store(soft_restart, std::sync::atomic::Ordering::Relaxed);
            }
        });
}
