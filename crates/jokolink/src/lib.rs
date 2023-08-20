//! Jokolink is a crate to deal with Mumble Link data exposed by games/apps on windows via shared memory

//! Joko link is designed to primarily get the MumbleLink or the window size
//! of the GW2 window for Jokolay (an crossplatform overlay for Guild Wars 2).
//! on windows, you can use it to create/open shared memory.
//! and on linux, you can run jokolink binary in wine, which will create/open shared memory and copy-paste it into /dev/shm.
//! then, you can easily read the /dev/shm file from a any number of linux native applications.
//! along with mumblelink data, it also copies the x11 window id of gw2. you can use this to get the size of gw2 window.
//!
//! NOTE: Although you can just get the window size and copy it into the /dev/shm file. there's a reason we instead use the x11 window id.
//!         Overlays which use "always on top" feature cannot stay on top of (windowed) fullscreen windows, so we use something called `transient_for`
//!         attribute of x11. when we set the attribute for our overlay with the value of the parent (gw2 window) id, the overlay can stay on top of fullscreen gw2.
//!         

use raw_window_handle::RawWindowHandle;

mod types;
use miette::Result;
use tracing::warn;
pub use types::*;
#[cfg(target_os = "windows")]
pub use win::{create_link_shared_mem, get_link_buffer, get_xid};

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
    changes: MumbleChanges,
    /// latest mumble link
    link: MumbleLink,
}
impl MumbleManager {
    pub fn new(name: &str, jokolay_window_id: RawWindowHandle) -> Result<Self> {
        let backend = MumblePlatformImpl::new(name, jokolay_window_id)?;
        Ok(Self {
            backend,
            changes: MumbleChanges::empty(),
            link: MumbleLink::default(),
        })
    }
    pub fn tick(&mut self) -> Result<()> {
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
                warn!(
                    "mumble link process_id changed from {} to {}",
                    self.link.context.process_id, link.context.process_id
                );
                self.changes.toggle(MumbleChanges::GAME);

                #[cfg(target_os = "linux")]
                if link.context.process_id != 0 {
                    let _ = self.backend.set_transient_for();
                }
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
    pub fn get_latest_window_dimensions(&mut self) -> Result<[i32; 4]> {
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

bitflags::bitflags! {
    /// These flags represent the changes in mumble link compared to previous values
    struct MumbleChanges: u8 {
        const UI_TICK   =   1;
        const MAP       =   1 << 1;
        const CHARACTER =   1 << 2;
        const GAME      =   1 << 3;
    }
}

use windows::{
    Win32::Foundation::*,
    Win32::{System::SystemServices::DLL_PROCESS_ATTACH, *},
};

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn DllMain(dll_module: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    match call_reason {
        DLL_PROCESS_ATTACH => unsafe {
            const HELLO: &str = "hello from dll\0";
            const TITLE: &str = "my box\0";
            UI::WindowsAndMessaging::MessageBoxA(
                HWND::default(),
                windows::core::PCSTR(HELLO.as_ptr() as _),
                windows::core::PCSTR(TITLE.as_ptr() as _),
                UI::WindowsAndMessaging::MESSAGEBOX_STYLE::default(),
            );
        },
        _ => (),
    }

    true
}

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
extern "system" fn DirectDrawCreate(
    lpguid: *mut windows::core::GUID,
    lplpdd: *mut *mut ::core::ffi::c_void,
    punkouter: *mut ::core::ffi::c_void,
) -> windows::core::HRESULT {
    unsafe {
        load_original_pointers();
        if let Some(p) = DIRECT_DRAW_CREATE_FNPTR {
            const MESSAGE: &str = "Calling ddcreate fn pointer\0";
            const TITLE: &str = "DDCreate Call Success\0";
            UI::WindowsAndMessaging::MessageBoxA(
                HWND::default(),
                windows::core::PCSTR(MESSAGE.as_ptr() as _),
                windows::core::PCSTR(TITLE.as_ptr() as _),
                UI::WindowsAndMessaging::MESSAGEBOX_STYLE::default(),
            );
            p(lpguid, lplpdd, punkouter)
        } else {
            const MESSAGE: &str = "Missing Original ddcreate fn pointer\0";
            const TITLE: &str = "DDCreate Error\0";
            UI::WindowsAndMessaging::MessageBoxA(
                HWND::default(),
                windows::core::PCSTR(MESSAGE.as_ptr() as _),
                windows::core::PCSTR(TITLE.as_ptr() as _),
                UI::WindowsAndMessaging::MESSAGEBOX_STYLE::default(),
            );
            windows::core::HRESULT::default()
        }
    }
}

static mut DIRECT_DRAW_CREATE_FNPTR: Option<
    extern "system" fn(
        *mut windows::core::GUID,
        *mut *mut ::core::ffi::c_void,
        *mut ::core::ffi::c_void,
    ) -> windows::core::HRESULT,
> = None;
static mut DLL_PTR: HMODULE = HMODULE(0);

unsafe fn load_original_pointers() {
    if DLL_PTR.is_invalid() {
        let mut path = [0u8; MAX_PATH as _];
        let len = System::SystemInformation::GetSystemDirectoryA(Some(&mut path)) as usize;
        // we make sure that len is not zero. It means that GetSystemDirectoryA fn didn't fail.
        // we also check if length is above 200, because then we might be reaching the limit of maximum path length supported by windows.
        if len == 0 || len > 200 {
            return;
        }
        const DDRAW_DLL_PATH: &str = "\\ddraw.dll\0";
        path[len..(len + DDRAW_DLL_PATH.len())].copy_from_slice(DDRAW_DLL_PATH.as_bytes());
        // let system_path = CStr::from_bytes_until_nul(&path).unwrap();

        match System::LibraryLoader::LoadLibraryA(windows::core::PCSTR::from_raw(path.as_ptr())) {
            Ok(p) => {
                DLL_PTR = p;
            }
            Err(_) => {
                const MESSAGE: &str = "Could Not Locate Original ddraw DLL\0";
                const TITLE: &str = "LoadLibrary Error\0";
                UI::WindowsAndMessaging::MessageBoxA(
                    HWND::default(),
                    windows::core::PCSTR(MESSAGE.as_ptr() as _),
                    windows::core::PCSTR(TITLE.as_ptr() as _),
                    UI::WindowsAndMessaging::MESSAGEBOX_STYLE::default(),
                );
                return;
            }
        }
    }
    if DIRECT_DRAW_CREATE_FNPTR.is_none() {
        if let Some(p) = System::LibraryLoader::GetProcAddress(
            DLL_PTR,
            windows::core::PCSTR("DirectDrawCreate\0".as_ptr()),
        ) {
            let _ = DIRECT_DRAW_CREATE_FNPTR.insert(std::mem::transmute(p));
        } else {
            const MESSAGE: &str = "Could Not get address of ddcreate\0";
            const TITLE: &str = "GetProcessAddress Error\0";
            UI::WindowsAndMessaging::MessageBoxA(
                HWND::default(),
                windows::core::PCSTR(MESSAGE.as_ptr() as _),
                windows::core::PCSTR(TITLE.as_ptr() as _),
                UI::WindowsAndMessaging::MESSAGEBOX_STYLE::default(),
            );
        }
    }
}
