use egui::CtxRef;

#[derive(Debug)]
pub struct MainWindow {
    name: &'static str,
}

impl Default for MainWindow {
    fn default() -> Self {
        Self {
            name: "Main Window",
        }
    }
}

impl MainWindow {
    pub fn tick(_ctx: CtxRef) {}
}
