pub mod wmarker;
pub mod wmlink;

use std::{rc::Rc, sync::Arc};

use egui::{CtxRef, Window};
use parking_lot::Mutex;
use wmlink::MumbleLinkSetupWindow;

use crate::mlink::MumbleManager;

use self::wmarker::MarkersWindow;

pub struct MainWindow {
    pub name: String,
    pub mumble_window: MumbleLinkSetupWindow,
    pub mumble_window_show: bool,
    pub mcache: Arc<Mutex<Option<MumbleManager>>>,
    pub timer_window_show: bool,
    pub marker_window_show: bool,
    pub marker_window: MarkersWindow,
}

impl MainWindow {
    pub fn new(gl: Rc<glow::Context>) -> Self {
        let mcache = Arc::new(Mutex::new(None));

        let mumble_window = MumbleLinkSetupWindow::new(mcache.clone());
        let marker_window = MarkersWindow::new(gl);
        MainWindow {
            name: "MainWindow".to_string(),
            mumble_window,
            mumble_window_show: false,
            marker_window,
            marker_window_show: false,
            mcache,
            timer_window_show: false,
        }
    }
}

impl MainWindow {
    pub fn add_widgets_to_ui(&mut self, ctx: &CtxRef) {
        Window::new(&self.name).show(&ctx, |ui| {
            ui.checkbox(&mut self.mumble_window_show, "show Mumble Setup");
            if self.mumble_window_show {
                self.mumble_window
                    .add_widgets_to_ui(&ctx, self.mcache.clone());
            };
            ui.checkbox(&mut self.marker_window_show, "show Marker Window");
            if self.marker_window_show {
                self.marker_window
                    .add_widgets_to_ui(&ctx, self.mcache.clone());
            };
        });
    }
}
