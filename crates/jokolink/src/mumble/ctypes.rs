use enumflags2::BitFlags;
use jokoapi::end_point::{mounts::Mount, races::Race};
use miette::bail;
use serde::{Deserialize, Serialize};

use crate::{UISize, UIState};

/// The total size of the CMumbleLink struct. used to know the amount of memory to give to win32 call that creates the shared memory
pub const C_MUMBLE_LINK_SIZE_FULL: usize = std::mem::size_of::<CMumbleLink>();
/// This is how much of the CMumbleLink memory that is actually useful and updated. the rest is just zeroed out.
pub const USEFUL_C_MUMBLE_LINK_SIZE: usize = 1196;

/// The CMumblelink is how it is represented in the memory. But we rarely use it as it is and instead convert it into MumbleLink before using it for convenience
/// Many of the fields are documentad in the actual MumbleLink struct
#[derive(Debug, Clone, Copy)]
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
    pub name: [u16; 256],
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
    pub context: CMumbleContext,
    // Useless for now. Nothing is ever written here.
    // we will just remove this field and add the size when creating shared memory.
    // no point in copying more than 5kb when we only care about the first 1kb.
    // pub description: [u16; 2048],
}
impl Default for CMumbleLink {
    fn default() -> Self {
        Self {
            ui_version: Default::default(),
            ui_tick: Default::default(),
            f_avatar_position: Default::default(),
            f_avatar_front: Default::default(),
            f_avatar_top: Default::default(),
            name: [0; 256],
            f_camera_position: Default::default(),
            f_camera_front: Default::default(),
            f_camera_top: Default::default(),
            identity: [0; 256],
            context_len: Default::default(),
            context: Default::default(),
            // description: [0; 2048],
        }
    }
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
    pub unsafe fn get_ui_tick(link_ptr: *const CMumbleLink) -> u32 {
        (*link_ptr).ui_tick
    }
    pub unsafe fn get_pid(link_ptr: *const CMumbleLink) -> u32 {
        (*link_ptr).context.process_id
    }
    #[cfg(unix)]
    pub unsafe fn get_xid(link_ptr: *const CMumbleLink) -> u32 {
        (*link_ptr).context.xid
    }
    #[cfg(unix)]
    pub unsafe fn get_pos_size(link_ptr: *const CMumbleLink) -> [i32; 4] {
        (*link_ptr).context.window_pos_size
    }
    #[cfg(unix)]
    pub unsafe fn get_timestamp(link_ptr: *const CMumbleLink) -> i128 {
        let bytes = (*link_ptr).context.timestamp;
        i128::from_le_bytes(bytes)
    }
}

#[derive(Debug, Clone, Copy)]
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
    /// This is where the context fields provided by gw2 end.
    /// From here on, these are custom fields set by jokolink.dll for the use of jokolay
    /// These fields will be set before writing the link data to the `/dev/shm/MumbleLink` file from which jokolay can pick it up
    ///
    /// timestamp when jokolink wrote this data. unix nanoseconds
    /// This timestamp will be written every frame by jokolink even if mumble link is uninitialized.
    /// This is [i128] in little endian byte order. We use a byte array instead of [i128] directly because context is aligned to 4 by default. And
    /// [i64]/[i128] will change that alignment to 8. This will lead to 4 bytes padding between [CMumbleLink::context_len] and [CMumbleLink::context]
    ///
    /// If jokolink doesn't write for more than 1 or 2 seconds, it can be safely assumed that gw2 was closed/crashed.
    /// This is in nanoseconds since unix epoch in UTC timezone.
    pub timestamp: [u8; 16],
    /// x, y, width, height of guild wars 2 window relative to top left corner of the screen.
    /// This is populated with `GetWindowRect` fn
    /// DPI aware. In screen coordinate. But includes drop shadow too :(.
    pub window_pos_size: [i32; 4],
    /// This represents the x11 window id of the gw2 window. AFAIK, wine uses x11 only (no wayland), so this could be useful to set transient for
    pub xid: u32,
    pub window_pos_size_without_borders: [i32; 4],
    pub dpi_awareness: i32,
    pub client_pos_size: [i32; 4],
    /// to make the struct the right size. everything upto now is 120 bytes, so this rounds upto 256 bytes.
    pub padding: [u8; 96],
}
impl Default for CMumbleContext {
    fn default() -> Self {
        assert_eq!(std::mem::size_of::<CMumbleContext>(), 256);
        Self {
            server_address: Default::default(),
            map_id: Default::default(),
            map_type: Default::default(),
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
            mount_index: Default::default(),
            timestamp: Default::default(),
            window_pos_size: Default::default(),
            padding: [0; 96],
            xid: Default::default(),
            window_pos_size_without_borders: Default::default(),
            dpi_awareness: Default::default(),
            client_pos_size: Default::default(),
        }
    }
}
impl CMumbleContext {
    pub fn get_ui_state(&self) -> Option<BitFlags<UIState>> {
        BitFlags::from_bits(self.ui_state).ok()
    }

    /// first byte is `2` if ipv4. and `[4..7]` bytes contain the ipv4 octets.
    /// contains sockaddr_in or sockaddr_in6
    pub fn get_map_ip(&self) -> miette::Result<std::net::Ipv4Addr> {
        if self.server_address[0] != 2 {
            // add ipv6 support when gw2 servers add ipv6 support.
            bail!("ipaddr parsing failed for CMumble Context");
        }
        let ip = std::net::Ipv4Addr::from([
            self.server_address[4],
            self.server_address[5],
            self.server_address[6],
            self.server_address[7],
        ]);
        Ok(ip)
    }

    pub fn get_mount(&self) -> Option<Mount> {
        Some(match self.mount_index {
            1 => Mount::Jackal,
            2 => Mount::Griffon,
            3 => Mount::Springer,
            4 => Mount::Skimmer,
            5 => Mount::Raptor,
            6 => Mount::RollerBeetle,
            7 => Mount::Warclaw,
            8 => Mount::Skyscale,
            9 => Mount::Skiff,
            10 => Mount::SiegeTurtle,
            _ => return None,
        })
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(crate = "serde")]
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
        Some(match self.uisz {
            0 => UISize::Small,
            1 => UISize::Normal,
            2 => UISize::Large,
            3 => UISize::Larger,
            _ => return None,
        })
    }
    pub fn get_race(&self) -> Option<Race> {
        Some(match self.race {
            0 => Race::ASURA,
            1 => Race::CHARR,
            2 => Race::HUMAN,
            3 => Race::NORN,
            4 => Race::SYLVARI,
            _ => return None,
        })
    }
}
