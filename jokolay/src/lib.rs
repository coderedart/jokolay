pub mod app;
pub mod client;
pub mod config;
pub mod core;
use rfd::{MessageButtons, MessageDialog, MessageLevel};

#[macro_export]
macro_rules! gl_error {
    ($gl:expr) => {
        let e = $gl.get_error();
        if e != glow::NO_ERROR {
            log::error!("glerror {} at {} {} {}", e, file!(), line!(), column!());
        }
    };
}

pub fn show_msg_box(title: &str, msg: &str, buttons: MessageButtons, lvl: MessageLevel) -> bool {
    MessageDialog::new()
        .set_level(lvl)
        .set_title(title)
        .set_description(msg)
        .set_buttons(buttons)
        .show()
}
