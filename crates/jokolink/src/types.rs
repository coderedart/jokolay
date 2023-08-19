#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::net::Ipv4Addr;

use bitflags::bitflags;
use glam::Vec3;
use miette::IntoDiagnostic;
use miette::{bail, Result, WrapErr};
use num_derive::FromPrimitive;
use num_derive::ToPrimitive;
use serde::{Deserialize, Serialize};

// use jmf::INCHES_PER_METER;

/// As the CMumbleLink has all the fields multiple
#[derive(Clone, Debug, Default)]
pub struct MumbleLink {
    pub ui_tick: u32,
    pub f_avatar_position: Vec3,
    pub f_avatar_front: Vec3,
    pub f_camera_position: Vec3,
    pub f_camera_front: Vec3,
    pub identity: CIdentity,
    pub context: CMumbleContext,
}

impl MumbleLink {
    /// The most used function probably. will check if the `link_ptr->ui_tick > &self.ui_tick`
    /// and update self's fields based on that.
    pub fn update(&mut self, link_ptr: *const CMumbleLink) -> Result<()> {
        let cmlink = match unsafe { link_ptr.as_ref() } {
            Some(cmlink) => cmlink,
            None => bail!("CMumbleLink pointer is null"),
        };

        if self.ui_tick != cmlink.ui_tick {
            self.ui_tick = cmlink.ui_tick;

            self.f_avatar_position = cmlink.f_avatar_position.into();
            // self.f_avatar_position *= INCHES_PER_METER;
            self.f_avatar_front = cmlink.f_avatar_front.into();
            // self.f_avatar_front *= INCHES_PER_METER;
            self.f_camera_position = cmlink.f_camera_position.into();
            // self.f_camera_position *= INCHES_PER_METER;
            self.f_camera_front = cmlink.f_camera_front.into();
            // self.f_camera_front *= INCHES_PER_METER;
            self.identity.update(link_ptr)?;

            self.context.update(link_ptr);
        }

        Ok(())
    }
    pub fn update_from_slice(&mut self, buffer: &[u8; 1093]) -> Result<()> {
        self.update(buffer.as_ptr() as *const CMumbleLink)
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
/// The mumble context as stored inside the context field of CMumbleLink.
/// the first 48 bytes Mumble uses for identification is upto `build_id` field
/// the rest of the fields after `build_id` are provided by gw2 for addon devs.
pub struct CMumbleContext {
    /// first byte is `2` if ipv4. and `[4..7]` bytes contain the ipv4 octets.
    pub server_address: [u8; 28], // contains sockaddr_in or sockaddr_in6
    /// Map ID https://wiki.guildwars2.com/wiki/API:2/maps
    pub map_id: u32,
    pub map_type: u32,
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
    /// Identifies whether the character is currently mounted, if so, identifies the specific mount. does not match api
    pub mount_index: u8,
}
impl CMumbleContext {
    pub fn get_ui_state(&self) -> Option<UIState> {
        UIState::from_bits(self.ui_state)
    }

    pub fn update(&mut self, link_ptr: *const CMumbleLink) {
        let mc = unsafe {
            std::ptr::read_volatile(&(*link_ptr).context as *const u8 as *const CMumbleContext)
        };
        *self = mc;
    }
    /// first byte is `2` if ipv4. and `[4..7]` bytes contain the ipv4 octets.
    /// contains sockaddr_in or sockaddr_in6
    pub fn get_map_ip(&self) -> miette::Result<Ipv4Addr> {
        if self.server_address[0] != 2 {
            // add ipv6 support when gw2 servers add ipv6 support.
            bail!("ipaddr parsing failed for CMumble Context");
        }

        let ip = Ipv4Addr::from([
            self.server_address[4],
            self.server_address[5],
            self.server_address[6],
            self.server_address[7],
        ]);
        Ok(ip)
    }

    pub fn get_mount(&self) -> Option<Mount> {
        use num_traits::FromPrimitive;
        Mount::from_u8(self.mount_index)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
/// The json structure of the Identity field inside Cmumblelink.
/// the json string is null terminated and utf-16 encoded. so, need to use
/// Widestring crate's U16Cstring to first parse the bytes and then, convert to
/// String before deserializing to CIdentity
pub struct CIdentity {
    /// The name of the character
    pub name: String,
    /// The core profession id of the character. matches the ids of v2/professions endpoint
    pub profession: u32,
    /// Character's third specialization, or 0 if no specialization is present. See /v2/specializations for valid IDs.
    pub spec: u32,
    /// The race of the character. does not match api
    pub race: u32,
    /// API:2/maps
    pub map_id: u32,
    /// useless field from pre-megaserver days. is just shard_id from context struct
    pub world_id: u32,
    /// Team color per API:2/colors (0 = white)
    pub team_color_id: u32,
    /// Whether the character has a commander tag active
    pub commander: bool,
    /// Vertical field-of-view
    pub fov: f32,
    /// A value corresponding to the user's current UI scaling.
    pub uisz: u32,
}

impl CIdentity {
    pub fn get_uisz(&self) -> Option<UISize> {
        use num_traits::FromPrimitive;
        UISize::from_u32(self.uisz)
    }
    pub fn get_race(&self) -> Option<Race> {
        use num_traits::FromPrimitive;
        Race::from_u32(self.uisz)
    }
    pub fn update(&mut self, link_ptr: *const CMumbleLink) -> Result<()> {
        use widestring::U16CStr;
        let id = U16CStr::from_slice_truncate(unsafe { &(*link_ptr).identity })
            .into_diagnostic()
            .wrap_err_with(|| {
                format!("no null terminator in CIdentity. bytes: {:?}", unsafe {
                    &(*link_ptr).identity
                })
            })?;
        let id = id.to_string().into_diagnostic().wrap_err_with(|| {
            format!("failed to convert CIdentity widestring into string. wide_string: {id:?}")
        })?;
        *self = serde_json::from_str::<CIdentity>(&id)
            .into_diagnostic()
            .wrap_err_with(|| {
                format!("failed to deserialize CIdentity using serde_json. identity string: {id}")
            })?;
        Ok(())
    }
}

/// The total size of the CMumbleLink struct. used to know the amount of memory to give to win32 call that creates the shared memory
pub const C_MUMBLE_LINK_SIZE: usize = std::mem::size_of::<CMumbleLink>();
/// This is how much of the CMumbleLink memory that is actually useful and updated. the rest is just zeroed out. might change in future
pub const USEFUL_C_MUMBLE_LINK_SIZE: usize = 1193;

/// The CMumblelink is how it is represented in the memory. But we rarely use it as it is and instead convert it into MumbleLink before using it for convenience
/// Many of the fields are documentad in the actual MumbleLink struct
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(C)]
pub struct CMumbleLink {
    //// The ui_version will always be same as mumble doesn't change. we will come back to change it IF there's a new version.
    pub ui_version: u32,
    //// This tick represents the update count of the link (which is usually the frame count ) since mumble was initialized. not from the start of game, but the start of mumble
    pub ui_tick: u32,
    //// position of the character
    pub f_avatar_position: [f32; 3],
    //// direction towards which the character is facing
    pub f_avatar_front: [f32; 3],
    //// the up direction vector of the character.
    pub f_avatar_top: [f32; 3],
    //// The name of the character currently logged in
    pub name: [u8; 512],
    //// The position of the camera
    pub f_camera_position: [f32; 3],
    //// The direction towards which the camera is facing
    pub f_camera_front: [f32; 3],
    //// The up direction for the camera
    pub f_camera_top: [f32; 3],
    //// This is a widestring of json containing the serialized data of [CIdentity]
    pub identity: [u16; 256],
    //// The [Self::context] field is 256 bytes, but the game only uses the first few bytes.
    //// The first 48 bytes are used by mumble to uniquely identify the map/instance/room of the player
    //// So, this field is always set to 48 bytes.
    //// But gw2 writes even more data for the sake of addon functionality like minimap position etc..
    //// So, adding another 37 bytes which gw2 writes to. The total length of context is roughly 88 bytes if we consider the alignment.
    pub context_len: u32,
    //// 88 bytes are useful context written by gw2. Jokolink writes some more additional data beyond the 88 bytes like
    ////     X11 ID or window size or the timestamp when it last wrote data to this link etc.. which is useful for linux native clients like jokolay
    pub context: [u8; 256],
    /// Useless for now. Nothing is ever written here.
    pub description: [u16; 2048],
}
impl CMumbleLink {
    /// This takes a point and reads out the CMumbleLink struct from it. wrapper for unsafe ptr read
    pub fn get_cmumble_link(link_ptr: *const CMumbleLink) -> CMumbleLink {
        unsafe { std::ptr::read_volatile(link_ptr) }
    }

    /// Checks if the uitick is actually initialized
    pub fn is_valid(link_ptr: *const CMumbleLink) -> bool {
        unsafe { (*link_ptr).ui_tick > 0 }
    }

    /// gets uitick if we want to know the frame number since initialization of CMumbleLink
    pub fn get_ui_tick(link_ptr: *const CMumbleLink) -> u32 {
        unsafe { (*link_ptr).ui_tick }
    }
    pub fn get_pid(link_ptr: *const CMumbleLink) -> u32 {
        unsafe {
            (*((*link_ptr).context.as_ptr() as *const u8 as *const CMumbleContext)).process_id
        }
    }
    // /// creates the shared memory using win32 calls and returns the pointer
    // #[cfg(target_os = "windows")]
    // pub fn new_ptr(key: &str) -> Result<(_, *const CMumbleLink)> {
    //     crate::win::create_link_shared_mem(key)
    // }
    /// we will copy the bytes of the struct memory into the slice. we check that we will only copy upto C_MUMBLE_LINK_SIZE or buffer.len() whichever is smaller to avoid buffer overflow reads
    pub fn copy_raw_bytes_into(link_ptr: *const CMumbleLink, buffer: &mut [u8]) {
        let max_len = usize::min(buffer.len(), C_MUMBLE_LINK_SIZE);
        unsafe {
            std::ptr::copy_nonoverlapping(link_ptr as *const u8, buffer.as_mut_ptr(), max_len);
        }
    }
}

/// represents the ui scale set in settings -> graphics options -> interface size
#[derive(
    Debug,
    Clone,
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
pub enum UISize {
    Small = 0,
    Normal = 1,
    Large = 2,
    Larger = 3,
}

/// Race Enum that DOES NOT match the gw2 api endpoint ids
#[derive(
    Debug,
    Clone,
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
pub enum Race {
    Asura = 0,
    Charr = 1,
    Human = 2,
    Norn = 3,
    Sylvari = 4,
}
/// This is completely different from the api's mount ids. so, its find to define this here
#[derive(Debug, FromPrimitive, ToPrimitive, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mount {
    None = 0,
    Jackal = 1,
    Griffon = 2,
    Springer = 3,
    Skimmer = 4,
    Raptor = 5,
    RollerBeetle = 6,
    Warclaw = 7,
    Skyscale = 8,
}
impl Default for Mount {
    fn default() -> Self {
        Self::None
    }
}

bitflags! {
    /// The Uistate enum to represent the status of the UI in game
    pub struct UIState: u32 {
        const IS_MAP_OPEN = 0b00000001;
        const IS_COMPASS_TOP_RIGHT = 0b00000010;
        const DOES_COMPASS_HAVE_ROTATION_ENABLED = 0b00000100;
        const GAME_HAS_FOCUS = 0b00001000;
        const IN_COMPETITIVE_GAMEMODE = 0b00010000;
        const TEXTBOX_FOCUS = 0b00100000;
        const IS_IN_COMBAT = 0b01000000;
    }
}
