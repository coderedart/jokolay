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
    pub uisz: u32,
    /// changes since last mumble link update
    pub changes: BitFlags<MumbleChanges>,
}

/// These flags represent the changes in mumble link compared to previous values
#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum MumbleChanges {
    UiTick = 1,
    Map = 1 << 1,
    Character = 1 << 2,
    Game = 1 << 3,
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

// impl MumbleLink {
//     /// takes a pointer to [CMumbleLink] and uses it to construct a [MumbleLink].
//     /// will return error if
//     /// 1. pointer is null
//     /// 2. mumble is not initialized
//     /// 3. if name or identity json is invalid utf-16 or invalid json.
//     /// 4. any of the enums/bitflags have invalid values.
//     /// ## Unsafe
//     /// If the pointer points to invalid memory, it will lead to undefined behavior.
//     pub(crate) unsafe fn unsafe_update_from_pointer(
//         &mut self,
//         link_ptr: *const ctypes::CMumbleLink,
//     ) -> miette::Result<Self> {

//         if !ctypes::CMumbleLink::is_valid(link_ptr) {
//             bail!("mumble link uninitialized");
//         }
//         self.changes = Default::default();
//         // safety. as the link is valid, we can use as_ref
//         if let Some(cmumblelink) = link_ptr.as_ref().cloned() {

//             let json_string = widestring::U16CStr::from_slice_truncate(&cmumblelink.identity)
//                 .into_diagnostic()?
//                 .to_string()
//                 .into_diagnostic()?;
//             let identity: CIdentity = from_str(&json_string).into_diagnostic()?;
//             // Ok(Self {
//             //     ui_tick: cmumblelink.ui_tick,
//             //     f_avatar_position: cmumblelink.f_avatar_position.into(),
//             //     f_avatar_front: cmumblelink.f_avatar_front.into(),
//             //     f_camera_position: cmumblelink.f_camera_position.into(),
//             //     f_camera_front: cmumblelink.f_camera_front.into(),
//             //     name : identity.name,
//             //     map_id: cmumblelink.context.map_id,
//             //     fov: identity.fov,
//             //     uisz: identity
//             //         .get_uisz()
//             //         .ok_or(miette::miette!("ui size is invalid"))?,
//             // })
//         } else {
//             bail!("link_ptr is null ");
//         }
//     }
// }
