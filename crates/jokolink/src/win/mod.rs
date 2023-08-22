#![allow(clippy::not_unsafe_ptr_arg_deref)]
pub mod dll;
//putting all the winapi specific stuff here. so that i can lock it all behind a cfg attr at the mod declaration

use std::time::{Duration, Instant};

use crate::{mumble::ctypes::*, MumbleLink};
use joko_core::prelude::*;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::*,
        System::{
            Memory::*,
            Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
        },
        UI::WindowsAndMessaging::*,
    },
};

/// This source will be the used to abstract the linux/windows way of getting MumbleLink
/// on windows, this represents the shared memory pointer to mumblelink, and as long as one of gw2 or a client like us is alive, the shared memory will stay alive
/// on linux, this will be a File in /dev/shm that will only exist if jokolink created it at some point in time. this lives in ram, so reading from it is pretty much free.
#[derive(Debug)]
pub struct MumbleWinImpl {
    link_ptr: *const CMumbleLink,
    mumble_handle: HANDLE,
    window_handle: isize,
    process_handle: HANDLE,
    window_size_last_checked: Instant,
    window_pos_size: [i32; 4],
    last_ui_tick: u32,
}
impl Drop for MumbleWinImpl {
    fn drop(&mut self) {
        unsafe {
            warn!("dropping mumble link windows impl");
            if let Err(e) = UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.link_ptr as _,
            }) {
                error!("failed to unmap view of mumble file: {e:#?}");
            }
            if let Err(e) = CloseHandle(self.mumble_handle) {
                error!("failed to close handle of mumble link : {e:#?}")
            }
            if !self.process_handle.is_invalid() {
                if let Err(e) = CloseHandle(self.process_handle) {
                    error!("failed to close handle of mumble link : {e:#?}")
                }
            }
        }
    }
}
impl MumbleWinImpl {
    pub fn new(key: &str) -> Result<Self> {
        let (handle, link_ptr) =
            unsafe { create_link_shared_mem(key) }.wrap_err("failed to create mumblelink shm ")?;

        Ok(Self {
            link_ptr,
            mumble_handle: handle,
            window_handle: 0,
            window_size_last_checked: Instant::now(),
            last_ui_tick: unsafe { (*link_ptr).ui_tick },
            window_pos_size: [0; 4],
            process_handle: HANDLE::default(),
        })
    }
    pub fn is_alive(&self) -> bool {
        !self.process_handle.is_invalid()
    }
    pub fn win_pos_size(&self) -> [i32; 4] {
        self.window_pos_size
    }
    pub unsafe fn tick(&mut self) -> Result<()> {
        let ui_tick = CMumbleLink::get_ui_tick(self.link_ptr);
        let pid = CMumbleLink::get_pid(self.link_ptr);

        if self.last_ui_tick != ui_tick && ui_tick != 0 {
            // if gw2 is restarted, then the current ui tick will be less than the previous one.
            // So, we will remove the resources, so that we can check for gw2 process/window handle / pos/size from scratch.
            if ui_tick < self.last_ui_tick {
                warn!(
                    "found new gw2 process. last_tick: {}, new_tick: {}, new_pid: {}",
                    self.last_ui_tick, ui_tick, pid
                );
                if self.window_handle != 0 {
                    self.window_handle = 0;
                }
                if !self.process_handle.is_invalid() {
                    if let Err(e) = CloseHandle(self.process_handle) {
                        error!("failed to close process handle of old gw2: {e:#?}");
                    }
                    self.process_handle = HANDLE::default();
                }
            }
            self.last_ui_tick = ui_tick;
            if self.window_size_last_checked.elapsed() > Duration::from_secs(2) {
                self.window_size_last_checked = Instant::now();
                if !self.is_alive() {
                    self.process_handle = get_process_handle(pid)
                        .wrap_err("failed to get process handle from pid")?;
                    if self.is_alive() {
                        self.window_pos_size = self.update_pos_size()?;
                    }
                }
            }
        }
        // If there has been any activity in the mumble link, then we woiuld never reach 3 seconds duration with last checked of window size
        // But if mumble is not updating for ever three seconds, then it could be either because the gw2 process id dead or it is in char select screen/map loading screen.
        // So, we check if the process handle is valid. If it is, then we should check for alive status.
        // if process handle is invalid, then mumble must be dead already.
        if self.window_size_last_checked.elapsed() > Duration::from_secs(3)
            && !self.process_handle.is_invalid()
        {
            let alive = check_process_alive(self.process_handle)
                .wrap_err("failed to get process alive status")?;

            if !alive {
                self.window_handle = 0;
                self.window_pos_size = [0; 4];
                let process_handle = self.process_handle;
                self.process_handle = Default::default(); // to make sure that handle will stay invalid until there is some change in mumble link
                CloseHandle(process_handle)
                    .into_diagnostic()
                    .wrap_err("failed to close process handle when gw2 process is dead")?;
            }
            self.window_size_last_checked = Instant::now();
        }
        Ok(())
    }
    pub fn get_link(&mut self) -> Result<MumbleLink> {
        unsafe { MumbleLink::unsafe_load_from_pointer(self.link_ptr) }
    }
    pub unsafe fn get_cmumble_link(&self) -> CMumbleLink {
        let mut link = std::ptr::read_volatile(self.link_ptr);
        link.context.timestamp = OffsetDateTime::now_utc()
            .unix_timestamp()
            .try_into()
            .expect("should be good until 2038");
        link.context.window_pos_size = self.window_pos_size;
        link
    }
    unsafe fn update_pos_size(&mut self) -> miette::Result<[i32; 4]> {
        if !self.is_alive() {
            self.window_pos_size = Default::default();
            self.window_handle = 0;
            bail!("cannot get window dimensions when gw2 is dead");
        }
        if self.window_handle == 0 && CMumbleLink::is_valid(self.link_ptr) {
            self.window_handle = unsafe {
                get_gw2_window_handle(CMumbleLink::get_pid(self.link_ptr))
                    .wrap_err("failed to get window handle when tick is greater than zero")
            }?;
        }
        assert_ne!(self.window_handle, 0);

        // if gw2 is alive, we get the dimensions and set them.
        let wd = get_win_pos_dim(self.window_handle)
            .wrap_err("failed to get window dimensions from gw2 window handle")?;
        Ok(wd)
    }
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

fn get_win_pos_dim(window_handle: isize) -> Result<[i32; 4]> {
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
            (rect.right - rect.left)
                .try_into()
                .into_diagnostic()
                .wrap_err("gw2 window width could not be cast into u32")?,
            (rect.bottom - rect.top)
                .try_into()
                .into_diagnostic()
                .wrap_err("gw2 height could not be cast into u32")?,
        ])
    }
}

/// Can get the Window handle using the provided process id
/// This function iterates through all windows using `EnumWindows` fn and gets their process id.
/// Then, it checks if it matches argument `pid` and returns that handle. errors if we can't find find any window handle with the provided process id.
unsafe fn get_gw2_window_handle(pid: u32) -> miette::Result<isize> {
    // get pid from mumble link
    let mut lparam_pid = pid as isize;

    // enumerate windows and get the handle and assign it to the pid variable if the process id of the handle actually matches the pid
    let _ = EnumWindows(
        Some(get_handle_by_pid),
        LPARAM(((&mut lparam_pid) as *mut isize) as isize),
    );
    if lparam_pid == pid as isize {
        bail!("failed to find a window with our gw2 pid. something must have gone very wrong");
    }
    Ok(lparam_pid)
}

/// The function helps get the x11 window id by using the pid in mumblelink.
unsafe fn _get_xid(link_ptr: *const CMumbleLink) -> miette::Result<u32> {
    // no point in getting xid if mumble is not valid -_-
    assert!(CMumbleLink::is_valid(link_ptr));
    let pid = CMumbleLink::get_pid(link_ptr);
    let window_handle = get_gw2_window_handle(pid)?;
    // get the x11 window id from the win32 window handle
    let atom_string = std::ffi::CString::new("__wine_x11_whole_window")
        .into_diagnostic()
        .wrap_err("unreachable as cstr is static")?;
    let xid = GetPropA(HWND(window_handle), PCSTR(atom_string.as_ptr() as _));
    // check if the xid is actually null, in which case we have failed
    if xid.is_invalid() {
        miette::bail!("xid is invalid {xid:?}");
    }

    let xid = xid
        .0
        .try_into()
        .into_diagnostic()
        .wrap_err("failed to put xid into u32 from isize")?;
    Ok(xid)
}

fn get_process_handle(pid: u32) -> Result<HANDLE> {
    unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .into_diagnostic()
            .wrap_err_with(|| miette::miette!("failed to open process handle. pid: {pid}"))
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

// pub fn is_pid_alive(pid: u32) -> Result<bool> {
//     let process_handle = get_process_handle(pid)?;
//     let alive = check_process_alive(process_handle);
//     close_process_handle(process_handle);
//     alive
// }
/// This function creates shared memory for mumble link using Key as the link name
unsafe fn create_link_shared_mem(key: &str) -> Result<(HANDLE, *const CMumbleLink)> {
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
            C_MUMBLE_LINK_SIZE_FULL as u32,
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
            C_MUMBLE_LINK_SIZE_FULL,
        )
        .Value;

        // check if we were successful
        if cml_ptr.is_null() {
            bail!(
                "could not map view of file, error code: {:#?}",
                GetLastError()
            );
        }
        Ok((file_handle, cml_ptr as *const CMumbleLink))
    }
}
