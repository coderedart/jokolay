//! Jokolink is a crate to deal with Mumble Link data exposed by other games/apps on windows via shared memory

//! Joko link is a windows only crate. designed to primarily get the MumbleLink or the window
//! size of the GW2 window for Jokolay (an crossplatform overlay for Guild Wars 2). It can also
//! expose the data through a server. can easily be modified to get data from other applications too.
//! on windows, you can use it to get the pointer. and on linux, you can run jokolink in wine,
//! so that you can easily request the data from a linux native application.
//! It can multiple accessing data of multiple MumbleLinks, and allows multiple clients
//! to request the data.

use serde::{Deserialize, Serialize};

use anyhow::Context;

pub mod mlink;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod win;

#[cfg(target_os = "linux")]
use linux::MumbleSource;

#[cfg(target_os = "windows")]
use win::MumbleSource;

/// This is used to update the link from the mumble source. when src is none, the link is usually a default. check if its valid before using
#[derive(Debug)]
pub struct MumbleCtx {
    pub src: MumbleSource,
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

impl MumbleCtx {
    /// creates a mumble manager based on the config. it is upto users to check if its valid by checking the last instant and whether src is none
    pub fn new(config: MumbleConfig, ow_window_id: u32, latest_time: f64) -> anyhow::Result<Self> {
        let src = MumbleSource::new(&config, latest_time, ow_window_id)
            .context("failed to create mumble src")?;
        Ok(Self { src, config })
    }

    // just gets the already cached mumble link. call `tick()` to update
    // pub fn get_link(&self) -> &MumbleLink {
    //     &self.src.get_link()
    // }

    // if src is none, it represents the last instant when we tried to update src. when src is some, it represents the last instant mumble's uitick changed
    // pub fn last_updated(&self) -> Instant {
    //     self.last_link_update
    // }

    /// this will check previous cache's uitick and if src is valid, will try to update mumble link. IF uitick is different, it will update the last_update instant
    /// if src is not valid, tries to create src again if it has been atleast a second from the last attempt to create. after creation attempt, we will update the last_update instant to now
    /// this keeps cpu usage low by not checking every frame. which is not that useful anyway.
    pub fn tick(&mut self, latest_time: f64, sys: &mut sysinfo::System) -> anyhow::Result<()> {
        self.src.tick(latest_time, sys)?;

        Ok(())
    }
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
