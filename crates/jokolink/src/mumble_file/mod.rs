#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod win;

use std::sync::Arc;

use crate::mlink::MumbleLink;
use color_eyre::Result;
#[cfg(target_os = "linux")]
pub use linux::GW2InstanceData;
#[cfg(target_os = "linux")]
use linux::MumbleBackend;
#[cfg(target_os = "windows")]
use win::MumbleBackend;

/// The Source of MumbleLink Data for a MumbleLink Name.
pub struct MumbleFile {
    link_name: Arc<str>,
    backend: MumbleBackend,
    last_ui_tick_changed_time: f64,
    last_link_update: f64,
    previous_ui_tick: u32,
    previous_unique_id: u32,
}
impl MumbleFile {
    pub fn get_link_name(&self) -> Arc<str> {
        self.link_name.clone()
    }
    pub fn get_last_ui_tick_changed_time(&self) -> f64 {
        self.last_ui_tick_changed_time
    }
    pub fn get_last_link_update_attempt_time(&self) -> f64 {
        self.last_link_update
    }
    pub fn get_previous_tick(&self) -> u32 {
        self.previous_ui_tick
    }
    pub fn get_previous_unique_id(&self) -> u32 {
        self.previous_unique_id
    }
}

pub trait MumbleFileTrait {
    fn get_link(&mut self, latest_time: f64) -> Result<Option<UpdatedMumbleData>>;
    fn new(link_name: &str, latest_time: f64) -> Result<MumbleFile>;
}

#[derive(Debug, Clone)]
pub struct UpdatedMumbleData {
    /// on linux, x11 window id is the unique id per gw2 instance
    /// on winows, gw2 process id is the unique id per gw2 instance
    pub unique_id: u32,
    pub link: MumbleLink,
}
