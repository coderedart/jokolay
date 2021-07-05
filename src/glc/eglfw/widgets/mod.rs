use std::net::UdpSocket;

use egui::{CtxRef, Window};
use jokolink::mlp::MumbleLink;

use crate::mlink::{GetMLMode, MumbleCache};




pub struct MainWindow {
    pub name: String,
    pub mumble_window: MumbleLinkWindow,
    pub mumble_window_show: bool,
    // pub timer_window_show: bool,
    // pub marker_window_show: bool,
}

impl Default for MainWindow {
    fn default() -> Self {
        let mumble_window = MumbleLinkWindow::default();
        MainWindow {
            name: "MainWindow".to_string(),
            mumble_window,
            mumble_window_show: true,
        }
    }
}

impl MainWindow {
    pub fn add_widgets_to_ui(&mut self, ctx: &CtxRef) {
        Window::new(&self.name).show(&ctx, |ui | {
            if ui.checkbox(&mut self.mumble_window_show, "show mumble Window").changed() {
                if self.mumble_window_show {
                    self.mumble_window.add_widgets_to_ui(&ctx);
                }
            };
        });
        
    }
}


pub struct MumbleLinkWindow {
    pub name: String,
    pub cache: MumbleCache,
    // pub timer_window_show: bool,
    // pub marker_window_show: bool,
}

impl Default for MumbleLinkWindow {
    fn default() -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        socket.connect("127.0.0.1:7187").unwrap();
        let cache = MumbleCache::new("MumbleLink", std::time::Duration::from_millis(20),GetMLMode::UdpSync(socket)).unwrap();
        MumbleLinkWindow {
            name: "MumbleLinkWindow".to_string(),
            cache,
        }
    }
}

impl MumbleLinkWindow {
    pub fn add_widgets_to_ui(&mut self, ctx: &CtxRef) {
        self.cache.update_link().unwrap();
        dbg!(&self.cache);
        Window::new(&self.name).show(&ctx, |ui | {
            ui.label(self.cache.link.identity.as_ref().unwrap().name.clone());
            
        });
        
    }
}
