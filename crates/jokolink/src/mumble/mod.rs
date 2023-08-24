#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod ctypes;

use joko_core::prelude::*;

use num_derive::FromPrimitive;
use num_derive::ToPrimitive;

/// As the CMumbleLink has all the fields multiple
#[derive(Clone, Debug, Default)]
pub struct MumbleLink {
    /// ui tick. (more or less represents the frame number of gw2)
    pub ui_tick: u32,
    /// character position
    pub f_avatar_position: Vec3,
    /// direction char is facing
    pub f_avatar_front: Vec3,
    /// camera position
    pub f_camera_position: Vec3,
    /// direction camera is facing
    pub f_camera_front: Vec3,
    /// The name of the character
    pub name: String,
    /// API:2/maps
    pub map_id: u32,
    /// Vertical field-of-view
    pub fov: f32,
    /// A value corresponding to the user's current UI scaling.
    pub uisz: UISize,
    pub window_pos: IVec2,
    pub window_size: IVec2,
    /// changes since last mumble link update
    pub changes: BitFlags<MumbleChanges>,
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
