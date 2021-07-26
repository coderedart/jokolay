pub mod wmarker;

use std::{rc::Rc};

use egui::{CtxRef, Window};
use jokolink::mlink::MumbleLink;




use self::wmarker::MarkersWindow;

pub struct MainWindow {
    pub mumble_window_show: bool,
    pub timer_window_show: bool,
    pub marker_window_show: bool,
    pub marker_window: MarkersWindow,
}

impl MainWindow {
    pub fn new(gl: Rc<glow::Context>) -> Self {
        let marker_window = MarkersWindow::new(gl);
        MainWindow {
            mumble_window_show: false,
            marker_window,
            marker_window_show: false,
            timer_window_show: false,
        }
    }
}

impl MainWindow {
    pub fn add_widgets_to_ui(&mut self, ctx: &CtxRef, link: &MumbleLink) {
        Window::new("Jokolay").show(&ctx, |ui| {
            ui.checkbox(&mut self.mumble_window_show, "show Mumble Setup");

            ui.checkbox(&mut self.marker_window_show, "show Marker Window");
        });
        if self.mumble_window_show {
            Window::new("Mumble Info").scroll(true).show(&ctx, |ui| {
                ui.label(format!("{:#?}", link));
            });
        };
        if self.marker_window_show {
            self.marker_window.add_widgets_to_ui(&ctx, link);
        };
    }
}
