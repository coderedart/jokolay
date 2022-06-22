//! Jokolink is a crate to deal with Mumble Link data exposed by other games/apps on windows via shared memory

//! Joko link is a windows only crate. designed to primarily get the MumbleLink or the window
//! size of the GW2 window for Jokolay (an crossplatform overlay for Guild Wars 2). It can also
//! expose the data through a server. can easily be modified to get data from other applications too.
//! on windows, you can use it to get the pointer. and on linux, you can run jokolink in wine,
//! so that you can easily request the data from a linux native application.
//! It can multiple accessing data of multiple MumbleLinks, and allows multiple clients
//! to request the data.

use serde::{Deserialize, Serialize};

pub mod mlink;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod win;

#[cfg(target_os = "linux")]
pub use linux::MumbleOnly;

#[cfg(target_os = "windows")]
pub use win::MumbleOnly;

use crate::mlink::MumbleLink;

/// The default mumble link name. can only be changed by passing the `-mumble` options to gw2 for multiboxing
pub const DEFAULT_MUMBLELINK_NAME: &str = "MumbleLink";

pub struct Gw2Data {
    pub window_handle: u32,
    pub pid: u32,
    pub dim: WindowDimensions,
    pub monitor: u32,
    pub workspace: u32,
    pub link: MumbleLink,
}

/// The Window dimensions struct used to represent the window position/sizes.
/// has lots of derives, so we don't have to update this again when requiring something like Hash
#[repr(C)]
#[derive(
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    bytemuck::Zeroable,
    bytemuck::Pod,
)]
pub struct WindowDimensions {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
