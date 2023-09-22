#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod ctypes;
use std::net::IpAddr;

use enumflags2::{bitflags, BitFlags};
use glam::{IVec2, Vec3};
use jokoapi::end_point::mounts::Mount;
use num_derive::FromPrimitive;
use num_derive::ToPrimitive;
use serde::Deserialize;
use serde::Serialize;

/// As the CMumbleLink has all the fields multiple
#[derive(Clone, Debug)]
pub struct MumbleLink {
    /// ui tick. (more or less represents the frame number of gw2)
    pub ui_tick: u32,
    /// character position
    pub player_pos: Vec3,
    /// direction char is facing
    pub f_avatar_front: Vec3,
    /// camera position
    pub cam_pos: Vec3,
    /// direction camera is facing
    pub f_camera_front: Vec3,
    /// The name of the character
    pub name: String,
    /// API:2/maps
    pub map_id: u32,
    pub map_type: u32,
    /// first byte is `2` if ipv4. and `[4..7]` bytes contain the ipv4 octets.
    pub server_address: IpAddr, // contains sockaddr_in or sockaddr_in6
    pub shard_id: u32,
    pub instance: u32,
    pub build_id: u32,
    /// The fields until now are provided for mumble.
    /// The rest of the data from here is what gw2 provides for the benefit of addons.
    /// This is the current UI state of the game. refer to [UIState]
    /// // Bitmask: Bit 1 = IsMapOpen, Bit 2 = IsCompassTopRight, Bit 3 = DoesCompassHaveRotationEnabled, Bit 4 = Game has focus, Bit 5 = Is in Competitive game mode, Bit 6 = Textbox has focus, Bit 7 = Is in Combat
    pub ui_state: u32,
    pub compass_width: u16,    // pixels
    pub compass_height: u16,   // pixels
    pub compass_rotation: f32, // radians
    pub player_x: f32,         // continentCoords
    pub player_y: f32,         // continentCoords
    pub map_center_x: f32,     // continentCoords
    pub map_center_y: f32,     // continentCoords
    pub map_scale: f32,
    /// The ID of the process that last updated the MumbleLink data. If working with multiple instances, this could be used to serve the correct MumbleLink data.
    /// but jokolink doesn't care, it just updates from whatever data. so, it is upto the user to deal with the change of pid
    /// on windows, we use this to get window handle which can give us a window size.
    /// On linux, this is useless because this is the process ID inside wine, and not the actual linux pid
    /// But, the jokolink binary uses this to get the window handle and then the X Window ID of gw2
    pub process_id: u32,
    /// refers to [Mount]
    /// Identifies whether the character is currently mounted, if so, identifies the specific mount. does not match gw2 api
    pub mount: Option<Mount>,

    /// Vertical field-of-view
    pub fov: f32,
    /// A value corresponding to the user's current UI scaling.
    pub uisz: UISize,
    // pub window_pos: IVec2,
    // pub window_size: IVec2,
    // pub window_pos_without_borders: IVec2,
    // pub window_size_without_borders: IVec2,
    /// This is the dpi of gw2 window. 96dpi is the default for a non-hidpi monitor with scaling 1.0
    /// for a scaling of 2.0, it becomes 192 and so on.
    pub dpi: i32,
    /// This is whether gw2 is scaling its UI elements to match the dpi. So, if the dpi is bigger than 96, gw2 will make text/ui bigger.
    /// -1 means we couldn't get the setting from gw2's config file in appdata/roaming
    /// 0 means scaling is disabled (false)
    /// 1 means scaling is enabled (true).
    pub dpi_scaling: i32,
    /// This is the position of the gw2's viewport (client area. x/y) relative to the top left corner of the desktop in *screen coords*
    pub client_pos: IVec2,
    /// This is the size of gw2's viewport (width/height) in screen coordinates
    pub client_size: IVec2,
    /// changes since last mumble link update
    pub changes: BitFlags<MumbleChanges>,
}
impl Default for MumbleLink {
    fn default() -> Self {
        Self {
            ui_tick: Default::default(),
            player_pos: Default::default(),
            f_avatar_front: Default::default(),
            cam_pos: Default::default(),
            f_camera_front: Default::default(),
            name: Default::default(),
            map_id: Default::default(),
            map_type: Default::default(),
            server_address: std::net::Ipv4Addr::UNSPECIFIED.into(),
            shard_id: Default::default(),
            instance: Default::default(),
            build_id: Default::default(),
            ui_state: Default::default(),
            compass_width: Default::default(),
            compass_height: Default::default(),
            compass_rotation: Default::default(),
            player_x: Default::default(),
            player_y: Default::default(),
            map_center_x: Default::default(),
            map_center_y: Default::default(),
            map_scale: Default::default(),
            process_id: Default::default(),
            mount: Default::default(),
            fov: Default::default(),
            uisz: Default::default(),
            dpi: Default::default(),
            dpi_scaling: Default::default(),
            client_pos: Default::default(),
            client_size: Default::default(),
            changes: Default::default(),
        }
    }
}
/// These flags represent the changes in mumble link compared to previous values
#[bitflags]
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum MumbleChanges {
    UiTick = 1,
    Map = 1 << 1,
    Character = 1 << 2,
    WindowPosition = 1 << 3,
    WindowSize = 1 << 4,
}

/// represents the ui scale set in settings -> graphics options -> interface size
#[derive(
    Debug,
    Clone,
    Default,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    FromPrimitive,
    ToPrimitive,
)]
#[serde(crate = "serde")]
pub enum UISize {
    Small = 0,
    #[default]
    Normal = 1,
    Large = 2,
    Larger = 3,
}

#[bitflags]
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
/// The Uistate enum to represent the status of the UI in game
pub enum UIState {
    IsMapOpen = 0b00000001,
    IsCompassTopRight = 0b00000010,
    DoesCompassHaveRotationEnabled = 0b00000100,
    GameHasFocus = 0b00001000,
    InCompetitiveGamemode = 0b00010000,
    TextboxFocus = 0b00100000,
    IsInCombat = 0b01000000,
}
