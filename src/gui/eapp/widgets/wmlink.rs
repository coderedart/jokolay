use std::{net::UdpSocket, sync::Arc};

use egui::{CtxRef, Window};
use parking_lot::Mutex;

use crate::mlink::{GetMLMode, MumbleCache};

pub struct MumbleLinkSetupWindow {
    pub name: String,
    pub cache: Arc<Mutex<Option<MumbleCache>>>,
    pub link_name: String,
    pub server_ip_port: String,
    pub show_mumble: bool,
    // pub timer_window_show: bool,
    // pub marker_window_show: bool,
}

impl MumbleLinkSetupWindow {
    pub fn new(cache: Arc<Mutex<Option<MumbleCache>>>) -> Self {
        MumbleLinkSetupWindow {
            name: "MumbleLinkWindow".to_string(),
            cache,
            link_name: "MumbleLink".to_string(),
            server_ip_port: "127.0.0.1:7187".to_string(),
            show_mumble: false,
        }
    }
}

impl MumbleLinkSetupWindow {
    pub fn add_widgets_to_ui(&mut self, ctx: &CtxRef, mcache: Arc<Mutex<Option<MumbleCache>>>) {
        Window::new(&self.name).show(&ctx, |ui| {
            ui.text_edit_singleline(&mut self.link_name);
            ui.text_edit_singleline(&mut self.server_ip_port);
            let mut mc = mcache.lock();
            if ui.button("connect").clicked() {
                let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
                socket.connect(&self.server_ip_port).unwrap();
                let cache = MumbleCache::new(
                    &self.link_name,
                    std::time::Duration::from_millis(10),
                    GetMLMode::UdpSync(socket),
                )
                .unwrap();
                mc.replace(cache);
            }
            if mc.is_none() {
                ui.label("status: not connected");
            } else {
                ui.label("status: connected");
                ui.checkbox(&mut self.show_mumble, "show mumble info");
            }
            if self.show_mumble {
                mc.as_mut().unwrap().update_link().unwrap();
                Window::new("Mumble Info").show(&ctx, |ui| {
                    ui.label(format!("{:#?}", mc.as_ref().unwrap().link));
                });
            }
        });
    }
}
