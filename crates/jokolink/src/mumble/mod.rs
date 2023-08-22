#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod ctypes;

use joko_core::prelude::*;

use num_derive::FromPrimitive;
use num_derive::ToPrimitive;

/// As the CMumbleLink has all the fields multiple
#[derive(Clone, Debug)]
pub struct MumbleLink {
    pub ui_tick: u32,
    pub f_avatar_position: Vec3,
    pub f_avatar_front: Vec3,
    pub f_camera_position: Vec3,
    pub f_camera_front: Vec3,
    /// The name of the character
    pub name: String,
    /// API:2/maps
    pub map_id: u32,
    /// Vertical field-of-view
    pub fov: f32,
    /// A value corresponding to the user's current UI scaling.
    pub uisz: UISize,
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
