use std::sync::Arc;

use egui::{CtxRef, Window};
use parking_lot::Mutex;
use tokio::runtime::Handle;

use crate::mlink::MumbleManager;

pub struct MumbleLinkSetupWindow {
    pub name: String,
    pub manager: Rc,
    pub link_name: String,
    pub server_ip_port: String,
    pub show_mumble: bool,
    // pub timer_window_show: bool,
    // pub marker_window_show: bool,
}

impl MumbleLinkSetupWindow {
    pub fn new(manager: Arc<Mutex<Option<MumbleManager>>>) -> Self {
        MumbleLinkSetupWindow {
            name: "MumbleLinkWindow".to_string(),
            manager,
            link_name: "MumbleLink".to_string(),
            server_ip_port: "127.0.0.1:7187".to_string(),
            show_mumble: false,
        }
    }
}

impl MumbleLinkSetupWindow {
    pub fn add_widgets_to_ui(&mut self, ctx: &CtxRef, handle: Handle) {
        Window::new(&self.name).show(&ctx, |ui| {
            ui.text_edit_singleline(&mut self.link_name);
            ui.text_edit_singleline(&mut self.server_ip_port);
            let mut mc = mcache.lock();
            if ui.button("connect").clicked() {
                mc.replace(MumbleManager::new(&self.link_name, receiver));
            }
            if mc.is_none() {
                ui.label("status: not connected");
            } else {
                ui.label("status: connected");
                ui.checkbox(&mut self.show_mumble, "show mumble info");
            }
            if self.show_mumble {
                mc.as_mut().unwrap().try_update();
                Window::new("Mumble Info")
                    .scroll(true)
                    .default_height(150.0)
                    .show(&ctx, |ui| {
                        ui.label(format!("{:#?}", mc.as_ref().unwrap().link));
                    });
            }
        });
    }
}
