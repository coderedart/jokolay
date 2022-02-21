#![allow(clippy::not_unsafe_ptr_arg_deref)]

//putting all the winapi specific stuff here. so that i can lock it all behind a cfg attr at the mod declaration

use crate::{
    mlink::{
        CMumbleContext, CMumbleLink, MumbleLink, C_MUMBLE_LINK_SIZE, USEFUL_C_MUMBLE_LINK_SIZE,
    },
    MumbleConfig,
};
use anyhow::{bail, Context};
use tracing::*;
use windows::Win32::{
    Foundation::*,
    System::{Memory::*, Threading::*},
    UI::WindowsAndMessaging::*,
};
/// This source will be the used to abstract the linux/windows way of getting MumbleLink
/// on windows, this represents the shared memory pointer to mumblelink, and as long as one of gw2 or a client like us is alive, the shared memory will stay alive
/// on linux, this will be a File in /dev/shm that will only exist if jokolink created it at some point in time. this lives in ram, so reading from it is pretty much free.
#[derive(Debug)]
pub struct MumbleSource {
    pub link_ptr: *const CMumbleLink,
    mumble_handle: HANDLE,
    pub gw2_window_handle: isize,
    pub gw2_pid: u32,
    pub gw2_process_handle: HANDLE,
    pub gw2_pos: [i32; 2],
    pub gw2_size: [u32; 2],
    pub link: MumbleLink,
    pub last_uitick_update: f64,
    pub last_pos_size_update: f64,
}
impl Drop for MumbleSource {
    fn drop(&mut self) {
        unsafe {
            UnmapViewOfFile(self.link_ptr as *const std::ffi::c_void);
            CloseHandle(self.mumble_handle);
        }
    }
}
impl MumbleSource {
    pub fn new(
        config: &MumbleConfig,
        latest_time: f64,
        _ow_window_id: u32,
    ) -> anyhow::Result<Self> {
        let (handle, link_ptr) = create_link_shared_mem(&config.link_name)
            .context("failed to create mumblelink shm ")?;
        let mut link = MumbleLink::default();
        let link = if link.update(link_ptr).is_ok() {
            link
        } else {
            MumbleLink::default()
        };

        Ok(Self {
            link_ptr,
            mumble_handle: handle,
            gw2_window_handle: 0,
            gw2_pid: 0,
            gw2_process_handle: HANDLE::default(),
            gw2_pos: [0, 0],
            gw2_size: [0, 0],
            link,
            last_uitick_update: latest_time,
            last_pos_size_update: latest_time,
        })
    }
    pub fn tick(&mut self, latest_time: f64, _sys: &mut sysinfo::System) -> anyhow::Result<()> {
        let mut present_link = MumbleLink::default();
        present_link.update(self.link_ptr)?;
        let present_tick = present_link.ui_tick;
        let previous_tick = self.link.ui_tick;
        // case where mumble is not initialized or mumble is not changed from last frame (either game dead or game fps low)
        if present_tick == 0 || previous_tick == present_tick {
            return Ok(());
        }

        // if gw2 crashes, then it will start uitick from zero. so, we need to get new pid/window ids
        // case where new game resets present_tick
        if previous_tick > present_tick {
            warn!("previous tick: {previous_tick} is greater than present_tick: {present_tick}. ");
            self.gw2_window_handle = 0;
            self.gw2_pid = 0;
            if !self.gw2_process_handle.is_invalid() {
                close_process_handle(self.gw2_process_handle);
            }
        }
        // we handled present == previous and present < previous. so, we know that present is greater than previous
        self.last_uitick_update = latest_time;
        self.link = present_link;

        if latest_time - self.last_pos_size_update > 1.0 {
            self.gw2_pid = get_gw2_pid(self.link_ptr);

            // if gw2_pid is set, it means we got gw2 window as well as process id.
            if self.gw2_pid != 0 {
                if self.gw2_process_handle.is_invalid() {
                    self.gw2_process_handle = get_process_handle(self.gw2_pid)
                        .context("failed to get gw2 process handle, when pid is not zero")?;
                }
                assert!(!self.gw2_process_handle.is_invalid());
                if check_process_alive(self.gw2_process_handle)? {
                    if self.gw2_window_handle == 0 {
                        self.gw2_window_handle = get_gw2_window_handle(self.link_ptr).context(
                            "failed to get window handle when tick is greater than zero",
                        )?;
                    }
                    assert_ne!(self.gw2_window_handle, 0);
                    // before we do anything, we first check if gw2 is still alive, otherwise, we just set pid/xid to zero, so that we can start over
                    // pid_t is i32 in libc

                    // if gw2 is alive, we get the dimensions and set them.
                    let (x, y, w, h) = get_win_pos_dim(self.gw2_window_handle)?;
                    self.gw2_pos = [x, y];
                    self.gw2_size = [w, h];
                    self.last_pos_size_update = latest_time;
                }
            }
        }

        Ok(())
    }
    pub fn get_link(&self) -> &MumbleLink {
        &self.link
    }
}
/// This function gets called by EnumWindows as a lambda function. it will be given a handle to all windows one by one,
/// and the pid of the process we want to match against that handle's pid. if handle's pid is matched against our pid, we will
/// assign the handle to our pid pointer so that the they can use it after EnumWindows returns
unsafe extern "system" fn get_handle_by_pid(window_handle: HWND, gw2_pid: LPARAM) -> BOOL {
    // make a varible to hold the process id of a window handle given to us.
    let mut handle_pid: u32 = 0;
    // get the process id of the handle and then store it in the handle_pid variable.
    GetWindowThreadProcessId(window_handle, (&mut handle_pid) as *mut u32);
    // if handle_pid is null, it means we failed to get the pid. so, we return true so that enumWindows can call us again with the handle to the next window.
    if handle_pid == 0 {
        return BOOL(1);
    }
    // gw2_pid is a long pointer TO a HWND. we cast gw2_pid from isize to a * mut isize.
    let gw2_pid = gw2_pid.0 as *mut isize;
    // we check if the pid which gw2_pid references is equal to handle_pid
    if *gw2_pid == handle_pid as isize {
        // we now assign the window_handle to the memory pointed by gw2_pid pointer.
        *gw2_pid = window_handle.0;
        return BOOL(0);
    }
    BOOL(1)
}

pub fn get_win_pos_dim(window_handle: isize) -> anyhow::Result<(i32, i32, u32, u32)> {
    unsafe {
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let status = GetWindowRect(HWND(window_handle), &mut rect as *mut RECT);
        if !status.as_bool() {
            anyhow::bail!("could not get gw2 window size");
        }
        Ok((
            rect.left,
            rect.top,
            (rect.right - rect.left)
                .try_into()
                .context("gw2 window width could not be cast into u32")?,
            (rect.bottom - rect.top)
                .try_into()
                .context("gw2 height could not be cast into u32")?,
        ))
    }
}
pub fn get_link_buffer(
    link_ptr: *const CMumbleLink,
) -> [u8; USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<isize>()] {
    {
        let mut buffer = [0u8; USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<isize>()];
        buffer.copy_from_slice(unsafe {
            std::slice::from_raw_parts(link_ptr as *const u8, C_MUMBLE_LINK_SIZE)
        });
        buffer
    }
}

pub fn get_gw2_window_handle(link_ptr: *const CMumbleLink) -> anyhow::Result<isize> {
    unsafe {
        // get pid from mumble link

        let mut pid = get_gw2_pid(link_ptr) as isize;

        // enumerate windows and get the handle and assign it to the pid variable if the process id of the handle actually matches the pid
        let result: BOOL = EnumWindows(
            Some(get_handle_by_pid),
            LPARAM(((&mut pid) as *mut isize) as isize),
        );
        // check if successful
        if result.as_bool() {
            anyhow::bail!(
                "couldn't find gw2 window. error code: {:#?}",
                GetLastError()
            );
        }
        Ok(pid as isize)
    }
}

pub fn get_gw2_pid(link_ptr: *const CMumbleLink) -> u32 {
    unsafe { (*((*link_ptr).context.as_ptr() as *const CMumbleContext)).process_id }
}
/// The function helps get the x11 window id by using the pid in mumblelink.
pub fn get_xid(link_ptr: *const CMumbleLink) -> anyhow::Result<isize> {
    // no point in getting xid if mumble is not valid -_-
    assert!(CMumbleLink::is_valid(link_ptr));

    unsafe {
        let window_handle = get_gw2_window_handle(link_ptr)?;
        // get the x11 window id from the win32 window handle
        let atom_string = std::ffi::CString::new("__wine_x11_whole_window")?;
        let xid = GetPropA(HWND(window_handle), PSTR(atom_string.as_ptr() as _));
        // check if the xid is actually null, in which case we have failed
        if xid.is_invalid() {
            bail!("xid is NULL");
        }

        Ok(xid.0)
    }
}

pub fn get_process_handle(pid: u32) -> anyhow::Result<HANDLE> {
    unsafe {
        let process_handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            false,
            pid,
        );
        if process_handle.is_invalid() {
            anyhow::bail!(
                "failed to get handle for process id: {} due to error {:?}",
                pid,
                GetLastError()
            )
        } else {
            Ok(process_handle)
        }
    }
}
pub fn check_process_alive(process_handle: HANDLE) -> anyhow::Result<bool> {
    // let mut exit_code = 0u32;
    // let result = unsafe { GetExitCodeProcess(process_handle, &mut exit_code as *mut u32) };
    // if !result.as_bool() {
    //     error!(
    //         "failed to get exit code for process due to error: {:?}",
    //         unsafe { GetLastError() }
    //     );
    //     return None;
    // }
    // if exit_code == STATUS_PENDING.0 as u32 {
    //     Some(true)
    // } else {
    //     Some(false)
    // }

    // this is slightly faster than using the GetExitCodeProcess method.
    // GetExitCodeProcess takes around 3 us on average with lowest being 2.5 us.
    // WaitForSingleObject takes around 2 us on average withe lowest being 1.5 us.
    let result = unsafe { WaitForSingleObject(process_handle, 0) };

    if result == WAIT_ABANDONED || result == WAIT_OBJECT_0 {
        Ok(false)
    } else if result == WAIT_TIMEOUT.0 {
        Ok(true)
    } else {
        anyhow::bail!("WaitForSingleObject returned code: {:#?}", result)
    }
}
pub fn close_process_handle(process_handle: HANDLE) {
    unsafe {
        CloseHandle(process_handle);
    }
}
pub fn is_pid_alive(pid: u32) -> anyhow::Result<bool> {
    let process_handle = get_process_handle(pid)?;
    let alive = check_process_alive(process_handle);
    close_process_handle(process_handle);
    alive
}
/// This function creates shared memory for mumble link using Key as the link name
pub fn create_link_shared_mem(key: &str) -> anyhow::Result<(HANDLE, *const CMumbleLink)> {
    // prepare the key as a cstr to pass to windows functions
    let key_cstr = std::ffi::CString::new(key)?;
    unsafe {
        // create a Mumble Link shared memory file
        // the file handle will need not be stored because when process exits, the handle will be dropped by windows

        let file_handle = CreateFileMappingA(
            INVALID_HANDLE_VALUE,
            std::ptr::null_mut(),
            PAGE_READWRITE,
            0,
            C_MUMBLE_LINK_SIZE as u32,
            PSTR(key_cstr.as_ptr() as _),
        );

        // if failed to create shared memory
        if file_handle.is_invalid() {
            anyhow::bail!(
                "could not create file map handle, error code: {:#?}",
                GetLastError()
            );
        }

        // map the shared memory into the address space of our process using the handle we got from creating the shm
        let cml_ptr = MapViewOfFile(file_handle, FILE_MAP_ALL_ACCESS, 0, 0, C_MUMBLE_LINK_SIZE);

        // check if we were successful
        if cml_ptr.is_null() {
            anyhow::bail!(
                "could not map view of file, error code: {:#?}",
                GetLastError()
            );
        }
        Ok((file_handle, cml_ptr as *const CMumbleLink))
    }
}
