#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod win;

use crate::jokolink::{mlink::MumbleLink, WindowDimensions};
#[cfg(target_os = "linux")]
pub use linux::MumbleLinuxError as MumbleBackendError;
#[cfg(target_os = "linux")]
use linux::MumbleLinuxImpl as MumblePlatformImpl;
#[cfg(target_os = "windows")]
pub use win::MumbleWinError as MumbleBackendError;
#[cfg(target_os = "windows")]
use win::MumbleWinImpl as MumblePlatformImpl;

/// This is an abstraction over MumbleLink implementations for windows and linux.
/// Purpose:
/// 1. create Mumble Backend
/// 2. get mumble link from the live shared memory (/dev/shm on linux)
/// 3. get window dimensions using the data from mumble link (pid on windows gives window handle gives size. xid on linux gives size)
pub struct MumbleBackend(MumblePlatformImpl);

impl MumbleBackend {
    pub fn new(name: &str, window_id: u32) -> Result<Self, MumbleBackendError> {
        Ok(Self(MumblePlatformImpl::new(name, window_id)?))
    }

    pub fn get_link(&mut self) -> Result<MumbleLink, MumbleBackendError> {
        self.0.get_link()
    }

    pub fn get_window_dimensions(&mut self) -> Result<WindowDimensions, MumbleBackendError> {
        self.0.get_window_dimensions()
    }
    #[cfg(target_os = "linux")]
    pub fn set_transient_for(&mut self) -> Result<(), MumbleBackendError> {
        self.0.set_transient_for()
    }
}

// #[cfg(feature = "egui")]
// impl egui::Widget for &mut MumbleBackend {
//     fn ui(self, ui: &mut egui::Ui) -> egui::Response {
//         // self.0.ui(ui)
//         ui.label("mumble backend ui")
//     }
// }
