//! Jokolink is a crate to deal with Mumble Link data exposed by other games/apps on windows via shared memory

//! Joko link is a windows only crate. designed to primarily get the MumbleLink or the window
//! size of the GW2 window for Jokolay (an crossplatform overlay for Guild Wars 2). It can also
//! expose the data through a server. can easily be modified to get data from other applications too.
//! on windows, you can use it to get the pointer. and on linux, you can run jokolink in wine,
//! so that you can easily request the data from a linux native application.
//! It can multiple accessing data of multiple MumbleLinks, and allows multiple clients
//! to request the data.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::*;

#[cfg(target_os = "windows")]
use crate::mlink::CMumbleLink;
use crate::mlink::MumbleLink;
pub mod mlink;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod win;

/// This is used to update the link from the mumble source. when src is none, the link is usually a default. check if its valid before using
#[derive(Debug)]
pub struct MumbleManager {
    pub src: Option<MumbleSource>,
    pub link: MumbleLink,
    pub last_update: Instant,
    pub config: MumbleConfig,
}
/// The configuration that mumble needs. just a mumble link name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MumbleConfig {
    /// This is used for identifying the shared memory of mumble link exposed by gw2
    pub link_name: String,
}

impl MumbleConfig {
    /// The default mumble link name. can only be changed by passing the `-mumble` options to gw2 for multiboxing
    pub const DEFAULT_MUMBLELINK_NAME: &'static str = "MumbleLink";
}

impl Default for MumbleConfig {
    /// Provides the default mumble_link_name as a string
    fn default() -> Self {
        Self {
            link_name: Self::DEFAULT_MUMBLELINK_NAME.to_string(),
        }
    }
}

impl MumbleManager {
    /// creates a mumble manager based on the config. it is upto users to check if its valid by checking the last instant and whether src is none
    pub fn new(config: MumbleConfig) -> anyhow::Result<MumbleManager> {
        let mut src = MumbleSource::new(&config.link_name);
        let mut link = MumbleLink::default();
        if let Some(ref mut msrc) = src {
            link = msrc.get_link()?
        }
        if link.ui_tick == 0 {
            error!("mumble link manager started with an uninitialized link");
        }
        let manager = MumbleManager {
            src,
            link,
            last_update: Instant::now(),
            config,
        };
        Ok(manager)
    }

    // just gets the already cached mumble link. call `tick()` to update
    pub fn get_link(&self) -> &MumbleLink {
        &self.link
    }

    // if src is none, it represents the last instant when we tried to update src. when src is some, it represents the last instant mumble's uitick changed
    pub fn last_updated(&self) -> Instant {
        self.last_update
    }

    /// this will check previous cache's uitick and if src is valid, will try to update mumble link. IF uitick is different, it will update the last_update instant
    /// if src is not valid, tries to create src again if it has been atleast a second from the last attempt to create. after creation attempt, we will update the last_update instant to now
    /// this keeps cpu usage low by not checking every frame. which is not that useful anyway.
    pub fn tick(&mut self) -> anyhow::Result<()> {
        let ui_tick = self.link.ui_tick;
        match self.src {
            Some(ref mut msrc) => {
                self.link = msrc.get_link()?;
                if ui_tick != self.link.ui_tick {
                    self.last_update = Instant::now();
                }
            }
            None => {
                if self.last_updated().elapsed() > Duration::from_secs(1) {
                    self.src = MumbleSource::new(&self.config.link_name);
                    warn!("mumble link is not initalized");
                    self.last_update = Instant::now();
                }
            }
        }
        Ok(())
    }
}
/// This source will be the used to abstract the linux/windows way of getting MumbleLink
/// on windows, this represents the shared memory pointer to mumblelink, and as long as one of gw2 or a client like us is alive, the shared memory will stay alive
/// on linux, this will be a File in /dev/shm that will only exist if jokolink created it at some point in time. this lives in ram, so reading from it is pretty much free.
#[derive(Debug)]
pub struct MumbleSource {
    #[cfg(target_os = "linux")]
    pub mumble_src: std::fs::File,
    #[cfg(target_os = "windows")]
    pub mumble_src: *const CMumbleLink,
}

/// The Window dimensions struct used to represent the window position/sizes.
/// has lots of derives, so we don't have to update this again when requiring something like Hash
#[repr(C)]
#[derive(
    Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Clone, Copy,
)]
pub struct WindowDimensions {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

unsafe impl bytemuck::Zeroable for WindowDimensions {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl bytemuck::Pod for WindowDimensions {}
