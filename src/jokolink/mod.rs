//! Jokolink is a crate to deal with Mumble Link data exposed by games/apps on windows via shared memory

//! Joko link is designed to primarily get the MumbleLink or the window
//! size of the GW2 window for Jokolay (an crossplatform overlay for Guild Wars 2).
//! on windows, you can use it to create/open shared memory.
//! and on linux, you can run jokolink binary in wine, which will create/open shared memory and copy-paste it into /dev/shm.
//! then, you can easily read the /dev/shm file from a any number of linux native applications.
//! along with mumblelink data, it also copies the x11 window id of gw2. you can use this to get the size of gw2 window.
//!
//! NOTE: Although you can just get the window size and copy it into the /dev/shm file. there's a reason we instead use the x11 window id.
//!         Overlays which use "always on top" feature cannot stay on top of (windowed) fullscreen windows, so we use something called `transient_for`
//!         attribute of x11. when we set the attribute for our overlay with the value of the parent (gw2 window) id, the overlay can stay on top of fullscreen gw2.
//!         

use mumble_backend::MumbleBackend;

mod mlink;
mod mumble_backend;
pub use mlink::*;
#[cfg(target_os = "windows")]
pub use mumble_backend::win::{create_link_shared_mem, get_link_buffer, get_xid};

pub use mumble_backend::MumbleBackendError as MumbleError;
/// The default mumble link name. can only be changed by passing the `-mumble` options to gw2 for multiboxing
pub const DEFAULT_MUMBLELINK_NAME: &str = "MumbleLink";

/// This primarily manages the mumble backend.
/// the purpose of `MumbleBackend` is to get mumble link data and window dimensions when asked.
/// Manager also caches the previous mumble link details like window dimensions or mapid etc..
/// and every frame gets the latest mumble link data, and compares with the previous frame.
/// if any of the changed this frame, it will set the relevant changed flags so that plugins
/// or other parts of program which care can run the relevant code.
pub struct MumbleManager {
    /// This abstracts over the windows and linux impl of mumble link functionality.
    /// we use this to get the latest mumble link and latest window dimensions of the current mumble link
    backend: MumbleBackend,
    changes: MumbleChanges,
    /// latest mumble link
    link: MumbleLink,
}
impl MumbleManager {
    pub fn new(name: &str, jokolay_window_id: u32) -> Result<Self, MumbleError> {
        let backend = MumbleBackend::new(name, jokolay_window_id)?;
        Ok(Self {
            backend,
            changes: MumbleChanges::empty(),
            link: MumbleLink::default(),
        })
    }
    pub fn tick(&mut self) -> Result<(), MumbleError> {
        self.changes = MumbleChanges::empty();
        let link = self.backend.get_link()?;

        if self.link.ui_tick != link.ui_tick {
            self.changes.toggle(MumbleChanges::UI_TICK);
            if self.link.identity.name != link.identity.name {
                self.changes.toggle(MumbleChanges::CHARACTER);
            }
            if self.link.identity.map_id != link.identity.map_id {
                self.changes.toggle(MumbleChanges::MAP);
            }
            if self.link.context.process_id != link.context.process_id {
                self.changes.toggle(MumbleChanges::GAME);
                #[cfg(target_os = "linux")]
                let _ = self.backend.set_transient_for();
            }
            self.link = link;
        }
        Ok(())
    }
    pub fn get_mumble_link(&self) -> Option<&MumbleLink> {
        if self.link.ui_tick == 0 {
            None
        } else {
            Some(&self.link)
        }
    }
    pub fn get_latest_window_dimensions(&mut self) -> Result<WindowDimensions, MumbleError> {
        self.backend.get_window_dimensions()
    }
    pub fn ui_tick_changed(&self) -> bool {
        self.changes.contains(MumbleChanges::UI_TICK)
    }
    pub fn map_changed(&self) -> bool {
        self.changes.contains(MumbleChanges::MAP)
    }

    pub fn character_changed(&self) -> bool {
        self.changes.contains(MumbleChanges::CHARACTER)
    }

    pub fn game_changed(&self) -> bool {
        self.changes.contains(MumbleChanges::GAME)
    }
}
/// The Window dimensions struct used to represent the window position/sizes.
/// has lots of derives, so we don't have to update this again when requiring something like Hash
#[repr(C)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct WindowDimensions {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
impl Default for WindowDimensions {
    fn default() -> Self {
        Self {
            x: Default::default(),
            y: Default::default(),
            width: 800,
            height: 600,
        }
    }
}

bitflags::bitflags! {
    /// These flags represent the changes in mumble link compared to previous values
    struct MumbleChanges: u8 {
        const UI_TICK   =   1;
        const MAP       =   1 << 1;
        const CHARACTER =   1 << 2;
        const GAME      =   1 << 3;
    }
}
