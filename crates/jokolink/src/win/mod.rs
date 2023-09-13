#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod dll;
//putting all the winapi specific stuff here. so that i can lock it all behind a cfg attr at the mod declaration

use crate::mumble::ctypes::*;
use miette::{bail, Context, IntoDiagnostic, Result};
use notify::Watcher;
use std::{
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
};
use time::OffsetDateTime;
use tracing::{debug, error, info, warn};
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::*,
        Graphics::{
            Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
            Gdi::ClientToScreen,
        },
        System::{
            Com::CoTaskMemFree,
            Memory::*,
            Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
        },
        UI::{
            HiDpi::{GetDpiForWindow, GetProcessDpiAwareness},
            Shell::{FOLDERID_RoamingAppData, SHGetKnownFolderPath},
            WindowsAndMessaging::*,
        },
    },
};

/// This source will be the used to abstract the linux/windows way of getting MumbleLink
/// on windows, this represents the shared memory pointer to mumblelink, and as long as one of gw2 or a client like us is alive, the shared memory will stay alive
/// on linux, this will be a File in /dev/shm that will only exist if jokolink created it at some point in time. this lives in ram, so reading from it is pretty much free.
#[derive(Debug)]
pub struct MumbleWinImpl {
    /// This is the pointer to shared memory which we mapped into our address space
    /// This is NEVER null. Because we consider failing to create MumbleLink as a hard error.
    /// ## Unsafe:
    /// Must unmap this pointer when we are dropping
    link_ptr: *const CMumbleLink,
    /// This is the handle to shared memory. We must close the handle when we are quitting
    /// This also never invalid. Because we consider failing to create MumbleLink as a hard error.
    /// ## Unsafe:
    /// Must close this handle when we are dropping
    mumble_handle: HANDLE,
    /// this is the previous ui_tick. We use this to check if there has been any change in mumble link memory
    /// If there is a change, then we check if the new pid is the same as old pid
    previous_ui_tick: u32,
    /// This is the previous pid of the mumble link
    /// If the current pid has changed, then it means we are dealing with a new gw2 process.
    previous_pid: u32,
    /// This is the process handle for gw2.
    /// when we see a change in pid, we will close the handle (if its valid) and open a new handle to the new gw2 process
    ///
    /// This handle is very important, because its validity shows that the gw2 process is "alive".
    /// If ui_tick has not changed for more than a second, then we will check using windows api if the process is still alive.
    /// If not, we will reset everything in our struct except for last_pid and last_ui_tick.
    process_handle: HANDLE,
    /// if ui_tick updates, we set this to now.
    /// If ui_tick doesn't update for more than 1 second AND we are alive, we will check if gw2 is still alive and reset the timestamp.
    last_ui_tick_update: Instant,
    /// if ui_tick changes this frame and we are alive, we get window size/pos of gw2 and reset this.
    /// if we are not alive, then we simply skip this check.
    last_pos_size_check: Instant,

    /// this is the position and size of gw2 window's client area. So, no borders or titlebar stuff. Just the viewport.
    client_pos_size: [i32; 4],
    /// Whether dpi scaling is enbaled or not in gw2. we parse this setting from gw2's configuration stored in AppData/Roaming/Guild Wars 2/GFXSettings.Gw2-64.exe.xml
    /// 0 for false
    /// 1 for true
    /// -1 for no idea. maybe because we couldn't find the config or read it or whatever.
    /// I recommend just assuming that it is true when in doubt. Because the text is too small to read when dpi scaling is turned off.
    dpi_scaling: i32,
    /// DPI of the gw2 window
    /// We get this via win32 api
    dpi: i32,
    /// This is the window handle of gw2.
    /// This is automatically set when we try to get window size/pos. and will be reset if gw2 process dies or if we find a new gw2 process.
    window_handle: isize,
    /// X11 window id. This is only useful for jokolink when it is run as dll on wine
    /// When the struct is initialized, we also try to get xid. and keep it here. On windows, we will just keep it at zero.
    xid: u32,
    /// This is the $USER/AppData/Roaming/Guild Wars 2/GFXSettings.Gw2-64.exe.xml
    /// But we get this programmatically via ShGetKnownFolderPath
    _gw2_config_watcher: notify::RecommendedWatcher,
    gw2_config_changed: std::sync::Arc<std::sync::atomic::AtomicBool>,
    gw2_config_path: PathBuf, /*
                              /// This is the position and size of gw2 window. This also includes a few hidden pixels around gw2 which serve as the border
                              /// Every time we check if the process is alive
                              window_pos_size: [i32; 4],
                              /// same as above. But we use DwmGetWindowAttribute, to exclude the drop shadow borders from the window rect
                              window_pos_size_without_borders: [i32; 4],
                              */
}

impl MumbleWinImpl {
    pub fn new(key: &str) -> Result<Self> {
        unsafe {
            let (handle, link_ptr) =
                create_link_shared_mem(key).wrap_err("failed to create mumblelink shm ")?;
            let gw2_config_changed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let gw2_config_path = {
                let roaming_appdata_pwstr = SHGetKnownFolderPath(
                    &FOLDERID_RoamingAppData as *const _,
                    Default::default(),
                    HANDLE::default(),
                )
                .into_diagnostic()
                .wrap_err("failed to get known folder roaming app data path")?;

                let mut roaming_str = roaming_appdata_pwstr
                    .to_string()
                    .into_diagnostic()
                    .wrap_err("appdata/roaming is not a utf-8 path")?;
                info!(roaming_str, "RoamingAppData path");
                CoTaskMemFree(Some(roaming_appdata_pwstr.0 as _));
                if !roaming_str.ends_with('\\') {
                    roaming_str.push('\\');
                }
                roaming_str.push_str("Guild Wars 2\\GFXSettings.Gw2-64.exe.xml");
                info!(roaming_str, "gw2 config path");
                roaming_str
            };
            let gw2_config_path = std::path::PathBuf::from_str(&gw2_config_path)
                .into_diagnostic()
                .wrap_err("failed to create pathbuf from gw2 config path in roaming appdata")?;
            std::fs::create_dir_all(gw2_config_path.parent().unwrap())
                .into_diagnostic()
                .wrap_err("failed to create gw2 config dir in appdata roaming ")?;
            if !gw2_config_path.exists() {
                std::fs::File::create(&gw2_config_path)
                    .into_diagnostic()
                    .wrap_err("failed to create empty gw2 config file ")?;
            }
            let dpi_scaling = check_dpi_scaling_enabled(&gw2_config_path);

            info!(
                ?dpi_scaling,
                ?gw2_config_path,
                "dpi scaling when we are starting out"
            );
            // lets just assume that the scaling is true by default
            let dpi_scaling = dpi_scaling.unwrap_or(1);
            gw2_config_changed.store(false, std::sync::atomic::Ordering::Relaxed);
            let gw2_config_changed_2 = gw2_config_changed.clone();
            let mut gw2_config_watcher = notify::recommended_watcher(move |ev| {
                debug!(?ev, "gw2 config changed");
                gw2_config_changed_2.store(true, std::sync::atomic::Ordering::Relaxed);
            })
            .into_diagnostic()
            .wrap_err("failed to create gw2 config directory watcher")?;
            gw2_config_watcher
                .watch(&gw2_config_path, notify::RecursiveMode::NonRecursive)
                .into_diagnostic()
                .wrap_err("faield to watch gw2 config dir")?;

            Ok(Self {
                link_ptr,
                mumble_handle: handle,
                window_handle: 0,
                last_ui_tick_update: Instant::now(),
                previous_ui_tick: CMumbleLink::get_ui_tick(link_ptr),
                // window_pos_size: [0; 4],
                process_handle: HANDLE::default(),
                previous_pid: 0,
                xid: 0,
                last_pos_size_check: Instant::now(),
                // window_pos_size_without_borders: [0; 4],
                dpi_scaling,
                client_pos_size: [0; 4],
                dpi: 0,
                _gw2_config_watcher: gw2_config_watcher,
                gw2_config_changed,
                gw2_config_path,
            })
        }
    }
    pub fn is_alive(&self) -> bool {
        !self.process_handle.is_invalid()
    }
    pub fn get_cmumble_link(&mut self) -> CMumbleLink {
        let mut link = unsafe { std::ptr::read_volatile(self.link_ptr) };
        link.context.timestamp = OffsetDateTime::now_utc()
            .unix_timestamp_nanos()
            .to_le_bytes();
        // link.context.window_pos_size = self.window_pos_size;
        // link.context.window_pos_size_without_borders = self.window_pos_size_without_borders;
        link.context.dpi_scaling = self.dpi_scaling;
        link.context.dpi = self.dpi;
        link.context.xid = self.xid;
        link.context.client_pos_size = self.client_pos_size;
        link
    }
    /// This is the most important function which will be called every frame
    /// 1. it gets the ui_tick from the link pointer
    /// 2. checks if it has changed compared to previous ui_tick. If it didn't change, then we have nothing to do and we return.
    /// 3. If it changed, we check if it is less than previous_ui_tick OR if the pid is differnet from previous_pid or if our process handle is invalid
    /// 4. If any of the above conditions are true, we reset and reinitialize the gw2 process handle + window handle + window size etc..
    /// 5. If ui_tick simply increased and nothing else changed, then we proceed with the usual stuf which is check the timer and get updated window pos/size
    pub fn tick(&mut self) -> Result<()> {
        unsafe {
            // if ui_tick is zero, we return
            if !CMumbleLink::is_valid(self.link_ptr) {
                // if we alive, that means ui_tick turned zero this frame for whatever reason, so we reset.
                if self.is_alive() {
                    self.reset();
                }
                return Ok(());
            }
            let ui_tick = CMumbleLink::get_ui_tick(self.link_ptr);
            let pid = CMumbleLink::get_pid(self.link_ptr);
            let previous_ui_tick = self.previous_ui_tick;
            // if ui tick didn't change. Then it means either we are in loading scree / character select screen or gw2 was closed (or crashed)
            if ui_tick == previous_ui_tick {
                // if we are not alive, then we just return because it just means mumble is not being updated.
                // but if we are alive, then we need to check whehter gw2 is still alive (in loading screen) or dead
                if self.is_alive() {
                    // we don't want to check every frame. Instead, we check in intervals of 3 seconds until gw2 finally loads into a map or it closes (so we can reset)
                    if self.last_ui_tick_update.elapsed() > Duration::from_secs(3) {
                        self.last_ui_tick_update = Instant::now();
                        match check_process_alive(self.process_handle) {
                            Ok(alive) => {
                                if !alive {
                                    self.reset();
                                }
                            }
                            Err(e) => {
                                error!(?e, "failed to get GetExitCodeProcess");
                                self.reset();
                            }
                        }
                    }
                }
                return Ok(());
            }
            // if ui_tick has changed, then we have some stuff to do.
            if ui_tick < previous_ui_tick // only happens if process changes
        || pid != self.previous_pid // gw2 process changed. need to get new handles/sizes etc..
        || !self.is_alive()
            // if we are in reset status, then its our chance to reinitialize because mumble just updated.
            {
                info!(ui_tick, notify = 2u64, "found new gw2 process");
                self.reinitialize();
            }
            // if reinitialization failed, then we can try again next frame.
            // if we are alive, that means everything is working as expected.
            // we update the previous ui_tick and check if we need to update window pos/size
            if self.is_alive() {
                self.last_ui_tick_update = Instant::now();
                self.previous_ui_tick = ui_tick;
                // check in 2 seconds intervals because it rarely changes
                if self.last_pos_size_check.elapsed() > Duration::from_secs(2) {
                    self.last_pos_size_check = Instant::now();

                    // self.window_pos_size = match get_window_pos_size(self.window_handle) {
                    //     Ok(window_pos_size) => {
                    //         if self.window_pos_size != window_pos_size {
                    //             info!(
                    //                 ?self.window_pos_size, ?window_pos_size,
                    //                 "window position size changed"
                    //             );
                    //         }
                    //         window_pos_size
                    //     }
                    //     Err(e) => {
                    //         error!(?e, "failed to get window position size");
                    //         self.reset(); // go back to being dead because it shouldn't usually fail
                    //         return Ok(());
                    //     }
                    // };
                    // let dpi_awareness = match GetProcessDpiAwareness(self.process_handle) {
                    //     Ok(dpi) => dpi.0,
                    //     Err(e) => {
                    //         error!(?e, "failed to get dpi awareness");
                    //         0
                    //     }
                    // };
                    // if self.dpi_scaling != dpi_awareness {
                    //     info!(dpi_awareness, self.dpi_scaling, "dpi scaling changed");
                    // }
                    // self.dpi_scaling = dpi_awareness;

                    let dpi = GetDpiForWindow(HWND(self.window_handle)) as i32;
                    if dpi != self.dpi {
                        info!(dpi, self.dpi, "dpi changed for gw2 window");
                    }
                    self.dpi = dpi;
                    // if the config changed, we will attempt to read dpi scaling.
                    // if we fail, we will just ignore it, and try again during next check of window pos (2 secs?)
                    // if we succeed, we will store false in the atomic bool
                    if self
                        .gw2_config_changed
                        .load(std::sync::atomic::Ordering::Relaxed)
                    {
                        match check_dpi_scaling_enabled(&self.gw2_config_path) {
                            Ok(dpi_scaling) => {
                                if self.dpi_scaling != dpi_scaling {
                                    info!(self.dpi_scaling, dpi_scaling, "dpi scaling changed");
                                }
                                self.dpi_scaling = dpi_scaling;
                                self.gw2_config_changed
                                    .store(false, std::sync::atomic::Ordering::Relaxed);
                            }
                            Err(e) => {
                                error!(notify = 0.0f64, ?e, "failed to open gw2 config file to check for dpi scaling changes");
                            }
                        }
                    }
                    // self.window_pos_size_without_borders =
                    //     match get_window_pos_size_without_borders(HWND(self.window_handle)) {
                    //         Ok(window_pos_size_without_borders) => {
                    //             if self.window_pos_size_without_borders
                    //                 != window_pos_size_without_borders
                    //             {
                    //                 info!(
                    //                     ?self.window_pos_size_without_borders,
                    //                     ?window_pos_size_without_borders,
                    //                     "window position size changed"
                    //                 );
                    //             }
                    //             window_pos_size_without_borders
                    //         }
                    //         Err(e) => {
                    //             error!(?e, "failed to get window position size");
                    //             self.reset(); // go back to being dead because it shouldn't usually fail
                    //             return Ok(());
                    //         }
                    //     };
                    self.client_pos_size =
                        match get_client_rect_in_screen_coords(HWND(self.window_handle)) {
                            Ok(client_pos_size) => {
                                if self.client_pos_size != client_pos_size {
                                    info!(
                                        ?self.client_pos_size,
                                        ?client_pos_size,
                                        "window position size changed"
                                    );
                                }
                                client_pos_size
                            }
                            Err(e) => {
                                error!(?e, "failed to get client position size");
                                self.reset(); // go back to being dead because it shouldn't usually fail
                                return Ok(());
                            }
                        };
                }
            }
        }
        Ok(())
    }
    /// A function which clears all the gw2 related resources like process/window handles
    unsafe fn reset(&mut self) {
        warn!("resetting mumble data");
        self.window_handle = 0;
        if !self.process_handle.is_invalid() {
            if let Err(e) = CloseHandle(self.process_handle) {
                error!(?e, "failed to close process handle of old gw2");
            }
        }
        self.process_handle = HANDLE::default();
        // self.window_pos_size = [0; 4];
        // self.window_pos_size_without_borders = [0; 4];
        self.dpi = 0;
        self.client_pos_size = [0; 4];
        self.previous_pid = 0;
        self.xid = 0;
    }
    unsafe fn reinitialize(&mut self) {
        warn!("we are reinitializing our mumble data");
        info!(
            "printing cmumblelink as it might be useful for debugging. {:?}",
            self.get_cmumble_link()
        );
        assert!(
            CMumbleLink::is_valid(self.link_ptr),
            "attempting to reinitialize when mumble is still unintialized"
        );
        let pid = CMumbleLink::get_pid(self.link_ptr);
        assert!(pid != 0, "attempting to initialize with pid == 0");
        self.reset();
        info!(
            "ui_tick: {}. pid: {pid}",
            CMumbleLink::get_ui_tick(self.link_ptr)
        );
        match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(process_handle) => {
                info!("got process handle: {process_handle:?}");
                // get pid from mumble link
                let mut window_handle = pid as isize;

                // enumerate windows and get the handle and assign it to the pid variable if the process id of the handle actually matches the pid
                let _ = EnumWindows(
                    Some(get_handle_by_pid),
                    LPARAM(((&mut window_handle) as *mut isize) as isize),
                );
                // if lparam_pid is still the same as pid, then we couldn't find the relevant window handle
                if window_handle == pid as isize {
                    if let Err(e) = CloseHandle(process_handle) {
                        error!(
                            ?e,
                            "failed to close process handle when we couldn't get window handle."
                        );
                    }
                    error!(
                        "failed to initialize mumble data because we couldn't find window handle"
                    );
                    return;
                }
                info!("found window handle too. yay");
                // now we have both process_handle and window_handle. We just need the window size to initialize our struct
                // this function only gets the suface/viewport pos/size without any borders/decoraitons.
                match get_client_rect_in_screen_coords(HWND(window_handle)) {
                    Ok(client_pos_size) => {
                        // this block is purely for logging purposes only to verify that all sizes are working properly.
                        {
                            // GetWindowRect includes drop shadow borders and titlebar
                            match get_window_pos_size(window_handle) {
                                Ok(pos_size) => {
                                    info!(
                                        ?pos_size,
                                        "get window position and size using GetWindowRect"
                                    );
                                }
                                Err(e) => {
                                    error!(?e, "failed to initialize mumble data because we coudln't get window position and size");
                                }
                            }
                            // DwmGetWindowAttribute doesn't include drop shadow borders, but includes titlebar
                            match get_window_pos_size_without_borders(HWND(window_handle)) {
                                Ok(window_pos_size_without_borders) => {
                                    info!(?window_pos_size_without_borders, "got window pos/size without borders using DwmGetWindowAttribute");
                                }
                                Err(e) => {
                                    error!(
                                        ?e,
                                        "failed to get window position size without borders"
                                    );
                                }
                            };
                        }
                        // only useful in wine
                        match std::ffi::CString::new("__wine_x11_whole_window") {
                            Ok(atom_string) => {
                                let xid =
                                    GetPropA(HWND(window_handle), PCSTR(atom_string.as_ptr() as _));
                                // check if the xid is actually null
                                if xid.is_invalid() {
                                    // will happen on windows. But this is harmless
                                    info!(?xid, "xid is invalid. This is completely fine on windows. This is only for linux users");
                                } else {
                                    info!("found xid too <3. {xid:?}");
                                    self.xid = xid
                                        .0
                                        .try_into()
                                        .map_err(|e| {
                                            error!(
                                                ?e,
                                                ?xid,
                                                "failed to fit x11 window id into u32"
                                            );
                                        })
                                        .unwrap_or_default();
                                }
                            }
                            Err(e) => {
                                error!(?e, notify = 0u64, "impossible. But __wine_x11_whole_window apparently not a valid cstring.");
                            }
                        }
                        // again, just for logging purposes and verify against lutris settings of dpi
                        let dpi_awareness = match GetProcessDpiAwareness(process_handle) {
                            Ok(dpi) => dpi.0,
                            Err(e) => {
                                error!(?e, "failed to get dpi awareness");
                                0
                            }
                        };
                        let dpi = GetDpiForWindow(HWND(self.window_handle)) as i32;
                        if dpi != self.dpi {
                            info!(dpi, self.dpi, "dpi changed for gw2 window");
                        }
                        info!(
                            ?client_pos_size,
                            dpi_awareness,
                            dpi,
                            pid,
                            ?process_handle,
                            ?window_handle,
                            "reinitialization complete "
                        );
                        self.process_handle = process_handle;
                        self.window_handle = window_handle;
                        self.dpi = dpi;
                        self.client_pos_size = client_pos_size;
                        self.last_ui_tick_update = Instant::now();
                        self.previous_pid = pid;
                    }
                    Err(e) => {
                        error!(?e, "failed to get client rect");
                    }
                }
            }
            Err(e) => {
                error!(?e, pid, "failed to open process handle");
            }
        }
    }
}

fn check_dpi_scaling_enabled(path: &std::path::Path) -> Result<i32> {
    // from $USER/AppData/Roaming/Guild Wars 2/GFXSettings.Gw2-64.exe.xml
    // life is too short to parse an xml out of this file. just find the following strings
    const DPI_SCALING_TRUE: &str = r#"dpiScaling" Registered="True" Type="Bool" Value="true"#;
    const DPI_SCALING_FALSE: &str = r#"dpiScaling" Registered="True" Type="Bool" Value="false"#;
    let contents = std::fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err("failed to read gw2 file")?;

    if contents.contains(DPI_SCALING_FALSE) {
        return Ok(0);
    };
    if contents.contains(DPI_SCALING_TRUE) {
        return Ok(1);
    };
    error!(contents, "failed to read dpi scaling from gw2 config file");
    Ok(-1)
}
/// This function creates/opens the shared memory with the key as the name.
/// Then, it maps the shared memory into the address space of our process.
/// Finally, we are provided the Handle of shared memory and the pointer to the starting address of the mapped memory.
/// can fail if
/// 1. key is not a valid cstring
/// 2. creating shared memory fails
/// 3. mapping shared memory into our addres space fails and we get a null pointer instead
unsafe fn create_link_shared_mem(key: &str) -> Result<(HANDLE, *mut CMumbleLink)> {
    info!("creating MumbleLink shared memory: {key}");
    // prepare the key as a cstr to pass to windows functions
    let key_cstr = std::ffi::CString::new(key)
        .into_diagnostic()
        .wrap_err(miette::miette!("invalid mumble link name {key}"))?;
    unsafe {
        // create a Mumble Link shared memory file
        // the file handle will need not be stored because when process exits, the handle will be dropped by windows
        let file_handle = CreateFileMappingA(
            INVALID_HANDLE_VALUE,
            None,
            PAGE_READWRITE,
            0,
            C_MUMBLE_LINK_SIZE_FULL as u32 + 4096, // we add the size of description field here.
            PCSTR(key_cstr.as_ptr() as _),
        )
        .into_diagnostic()
        .wrap_err("failed to create file mapping for MumbleLink")?;
        // map the shared memory into the address space of our process using the handle we got from creating the shm
        let cml_ptr = MapViewOfFile(
            file_handle,
            FILE_MAP_ALL_ACCESS,
            0,
            0,
            C_MUMBLE_LINK_SIZE_FULL + 4096, // adding the description field size here
        )
        .Value;
        // check if we were successful
        if cml_ptr.is_null() {
            bail!(
                "could not map view of file, error code: {:#?}",
                GetLastError()
            )
        }
        Ok((file_handle, cml_ptr.cast()))
    }
}

unsafe fn check_process_alive(process_handle: HANDLE) -> Result<bool> {
    let mut exit_code = 0u32;
    GetExitCodeProcess(process_handle, &mut exit_code as *mut u32)
        .into_diagnostic()
        .wrap_err("failed to get exit code of process ")?;
    Ok(exit_code == STATUS_PENDING.0 as u32)

    // this is slightly faster than using the GetExitCodeProcess method.
    // GetExitCodeProcess takes around 3 us on average with lowest being 2.5 us.
    // WaitForSingleObject takes around 2 us on average withe lowest being 1.5 us.
    // let result = unsafe { WaitForSingleObject(process_handle, 0) };

    // if result == WAIT_ABANDONED || result == WAIT_OBJECT_0 {
    //     Ok(false)
    // } else if result == WAIT_TIMEOUT.0 {
    //     Ok(true)
    // } else {
    //     bail!("WaitForSingleObject returned code: {:#?}", result)
    // }
}
/// This function gets called by EnumWindows as a lambda function. it will be given a handle to all windows one by one,
/// and the pid of the process we want to match against that handle's pid. if handle's pid is matched against our pid, we will
/// assign the handle to our pid pointer so that the they can use it after EnumWindows returns
unsafe extern "system" fn get_handle_by_pid(window_handle: HWND, gw2_pid_ptr: LPARAM) -> BOOL {
    // gw2_pid is a long pointer TO a HWND. we cast gw2_pid from isize to a * mut isize.
    let local_gw2_pid = *(gw2_pid_ptr.0 as *mut isize);

    // make a varible to hold the process id of a window handle given to us.
    let mut window_handle_pid: u32 = 0;
    // get the process id of the handle and then store it in the handle_pid variable.
    GetWindowThreadProcessId(window_handle, Some((&mut window_handle_pid) as *mut u32));
    // if handle_pid is null, it means we failed to get the pid. so, we return true so that enumWindows can call us again with the handle to the next window.
    if window_handle_pid == 0 {
        info!("failed to get process id of window handle {window_handle:?}");
        return BOOL(1);
    }

    info!("window handle {window_handle:?} has pid {window_handle_pid}");

    // we check if the pid which gw2_pid references is equal to handle_pid
    if local_gw2_pid == window_handle_pid as isize {
        info!(
            "successfully found the handle: {window_handle:?} of our gw2 with pid {local_gw2_pid}"
        );
        // we now assign the window_handle to the memory pointed by gw2_pid pointer.
        *(gw2_pid_ptr.0 as *mut isize) = window_handle.0;
        return BOOL(0);
    }
    BOOL(1)
}
/// Quirk: GetWindowRect also includes the invisible "borders" which windows uses for resizing or whatever
/// If you check the logs of jokolink and you use `xwininfo` command to check the actual gw2 window size, you can see the difference.
/// On my 4k monitor, it adds 5 pixels on left, right and bottom. And 56 pixels on top. Need to check if dpi affects this (or wayland).
/// If these border sizes are universal, then we can subtract those inside this function to get the actual pos/size without borders.
fn get_window_pos_size(window_handle: isize) -> Result<[i32; 4]> {
    unsafe {
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if let Err(e) = GetWindowRect(HWND(window_handle), &mut rect as *mut RECT) {
            bail!("GetWindowRect call failed {e:#?}");
        }
        Ok([
            rect.left,
            rect.top,
            (rect.right - rect.left),
            (rect.bottom - rect.top),
        ])
    }
}
fn get_window_pos_size_without_borders(window_handle: HWND) -> Result<[i32; 4]> {
    unsafe {
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if let Err(e) = DwmGetWindowAttribute(
            window_handle,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut RECT as _,
            std::mem::size_of::<RECT>() as _,
        ) {
            bail!("DwmGetWindowAttribute call failed {e:#?}");
        }
        Ok([
            rect.left,
            rect.top,
            (rect.right - rect.left),
            (rect.bottom - rect.top),
        ])
    }
}
fn get_client_rect_in_screen_coords(window_handle: HWND) -> Result<[i32; 4]> {
    unsafe {
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if let Err(e) = GetClientRect(window_handle, &mut rect as *mut RECT) {
            bail!("GetClientRect call failed {e:#?}");
        }
        let mut point: POINT = POINT {
            x: rect.left,
            y: rect.top,
        };
        if !ClientToScreen(window_handle, &mut point as *mut POINT).as_bool() {
            bail!("ClientToScreen call failed");
        }
        Ok([
            point.x,
            point.y,
            (rect.right - rect.left),
            (rect.bottom - rect.top),
        ])
    }
}
impl Drop for MumbleWinImpl {
    fn drop(&mut self) {
        unsafe {
            warn!("dropping mumble link windows impl");
            if let Err(e) = UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.link_ptr as _,
            }) {
                error!(?e, "failed to unmap view of mumble file");
            }
            if let Err(e) = CloseHandle(self.mumble_handle) {
                error!(?e, "failed to close handle of mumble link ")
            }
            if !self.process_handle.is_invalid() {
                if let Err(e) = CloseHandle(self.process_handle) {
                    error!(?e, "failed to close handle of mumble link ")
                }
            }
        }
    }
}
