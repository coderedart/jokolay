//! Jokolink is a crate to deal with Mumble Link data exposed by games/apps on windows via shared memory

//! Joko link is designed to primarily get the MumbleLink or the window size
//! of the GW2 window for Jokolay (an crossplatform overlay for Guild Wars 2).
//! on windows, you can use it to create/open shared memory.
//! and on linux, you can run jokolink binary in wine, which will create/open shared memory and copy-paste it into /dev/shm.
//! then, you can easily read the /dev/shm file from a any number of linux native applications.
//! along with mumblelink data, it also copies the x11 window id of gw2. you can use this to get the size of gw2 window.
//!

mod mumble;
use joko_core::prelude::*;
pub use mumble::*;

/// The default mumble link name. can only be changed by passing the `-mumble` options to gw2 for multiboxing
pub const DEFAULT_MUMBLELINK_NAME: &str = "MumbleLink";
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod win;

#[cfg(target_os = "linux")]
use linux::MumbleLinuxImpl as MumblePlatformImpl;
#[cfg(target_os = "windows")]
use win::MumbleWinImpl as MumblePlatformImpl;
// Useful link size is only [ctypes::USEFUL_C_MUMBLE_LINK_SIZE] . And we add 100 more bytes so that jokolink can put some extra stuff in there
// pub(crate) const JOKOLINK_MUMBLE_BUFFER_SIZE: usize = ctypes::USEFUL_C_MUMBLE_LINK_SIZE + 100;
/// This primarily manages the mumble backend.
/// the purpose of `MumbleBackend` is to get mumble link data and window dimensions when asked.
/// Manager also caches the previous mumble link details like window dimensions or mapid etc..
/// and every frame gets the latest mumble link data, and compares with the previous frame.
/// if any of the changed this frame, it will set the relevant changed flags so that plugins
/// or other parts of program which care can run the relevant code.
pub struct MumbleManager {
    /// This abstracts over the windows and linux impl of mumble link functionality.
    /// we use this to get the latest mumble link and latest window dimensions of the current mumble link
    backend: MumblePlatformImpl,
    changes: BitFlags<MumbleChanges>,
    /// latest mumble link
    link: Option<MumbleLink>,
}
impl MumbleManager {
    pub fn new(name: &str, _jokolay_window_id: Option<u32>) -> Result<Self> {
        let backend = MumblePlatformImpl::new(name)?;
        Ok(Self {
            backend,
            changes: BitFlags::empty(),
            link: None,
        })
    }
    pub fn tick(&mut self) -> Result<()> {
        unsafe { self.backend.tick()? };
        self.changes = BitFlags::empty();
        let link = self.backend.get_link()?;
        if let Some(previous_link) = self.link.take() {
            if previous_link.ui_tick != link.ui_tick {
                self.changes.toggle(MumbleChanges::UiTick);
                if previous_link.name != link.name {
                    self.changes.toggle(MumbleChanges::Character);
                }
                if previous_link.map_id != link.map_id {
                    self.changes.toggle(MumbleChanges::Map);
                }
            }
        }
        Ok(())
    }
    pub fn get_mumble_link(&self) -> &Option<MumbleLink> {
        &self.link
    }
    pub fn get_pos_size(&mut self) -> [i32; 4] {
        self.backend.win_pos_size()
    }
    pub fn ui_tick_changed(&self) -> bool {
        self.changes.contains(MumbleChanges::UiTick)
    }
    pub fn map_changed(&self) -> bool {
        self.changes.contains(MumbleChanges::Map)
    }

    pub fn character_changed(&self) -> bool {
        self.changes.contains(MumbleChanges::Character)
    }
}

/// These flags represent the changes in mumble link compared to previous values
#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum MumbleChanges {
    UiTick = 1,
    Map = 1 << 1,
    Character = 1 << 2,
    Game = 1 << 3,
}
