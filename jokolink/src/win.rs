#![allow(clippy::not_unsafe_ptr_arg_deref)]

//putting all the winapi specific stuff here. so that i can lock it all behind a cfg attr at the mod declaration

use crate::{
    mlink::{
        CMumbleContext, CMumbleLink, MumbleLink, C_MUMBLE_LINK_SIZE, USEFUL_C_MUMBLE_LINK_SIZE,
    },
    MumbleSource, WindowDimensions,
};
use anyhow::bail;
use tracing::*;
use windows::{
    runtime::Handle,
    Win32::{
        Foundation::*,
        System::{Memory::*, Threading::*},
        UI::WindowsAndMessaging::*,
    },
};
/// This function creates shared memory for mumble link using Key as the link name
pub fn create_link_shared_mem(key: &str) -> anyhow::Result<*const CMumbleLink> {
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
        Ok(cml_ptr as *const CMumbleLink)
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

pub fn get_win_pos_dim(link_ptr: *const CMumbleLink) -> anyhow::Result<WindowDimensions> {
    unsafe {
        if !CMumbleLink::is_valid(link_ptr) {
            anyhow::bail!("the MumbleLink is not init yet. so, getting window position is not valid operation");
        }
        let context = (*link_ptr).context.as_ptr() as *const CMumbleContext;
        // right now this is gw2's process id from mumble link, but we pass in the pointer to this into get_handle_by_pid function which will check this pid AND if it matches the window
        // it will deref the pointer and assign the window handle to this variable.
        let mut window_handle: isize = (*context).process_id as isize;

        let result: BOOL = EnumWindows(
            Some(get_handle_by_pid),
            LPARAM((&mut window_handle as *mut isize) as isize),
        );
        if result.as_bool() {
            anyhow::bail!(
                "couldn't find gw2 window. error code: {:#?}",
                GetLastError()
            );
        }
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
        Ok(WindowDimensions {
            x: rect.left,
            y: rect.top,
            width: (rect.right - rect.left),
            height: (rect.bottom - rect.top),
        })
    }
}

impl MumbleSource {
    pub fn new(key: &str) -> Option<MumbleSource> {
        Some(MumbleSource {
            mumble_src: create_link_shared_mem(key)
                .map_err(|e| {
                    error!("MumbleLink pointer Creation failed. {:?}", &e);
                    e
                })
                .ok()?,
        })
    }

    pub fn get_link_buffer(
        &mut self,
    ) -> [u8; USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<isize>()] {
        {
            let mut buffer = [0u8; USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<isize>()];
            buffer.copy_from_slice(unsafe {
                std::slice::from_raw_parts(self.mumble_src as *const u8, C_MUMBLE_LINK_SIZE)
            });
            buffer
        }
    }
    pub fn get_link(&mut self) -> anyhow::Result<MumbleLink> {
        let mut link = MumbleLink::default();
        link.update(self.mumble_src).map_err(|e| {
            error!("mumble link updated failed due to {:?}", &e);
            e
        })?;
        Ok(link)
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
            error!(
                "couldn't find gw2 window. error code: {:#?}",
                GetLastError()
            );
            anyhow::bail!("couldn't find gw2 window");
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
    if !CMumbleLink::is_valid(link_ptr) {
        error!("mumble not init. so getting xid is a failure");
        anyhow::bail!("mumble not init");
    }

    unsafe {
        let window_handle = get_gw2_window_handle(link_ptr)?;
        // get the x11 window id from the win32 window handle
        let atom_string = std::ffi::CString::new("__wine_x11_whole_window")?;
        let xid = GetPropA(HWND(window_handle), PSTR(atom_string.as_ptr() as _));
        // check if the xid is actually null, in which case we have failed
        if xid.is_invalid() {
            error!("xid is NULL");
            bail!("xid is NULL");
        }

        Ok(xid.0)
    }
}

pub fn get_process_handle(pid: u32) -> Option<HANDLE> {
    unsafe {
        let process_handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            false,
            pid,
        );
        if process_handle.is_invalid() {
            error!(
                "failed to get handle for process id: {} due to error {:?}",
                pid,
                GetLastError()
            );
            None
        } else {
            Some(process_handle)
        }
    }
}
pub fn check_process_alive(process_handle: HANDLE) -> Option<bool> {
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
        Some(false)
    } else if result == WAIT_TIMEOUT {
        Some(true)
    } else {
        None
    }
}
pub fn close_process_handle(process_handle: HANDLE) {
    unsafe {
        CloseHandle(process_handle);
    }
}
pub fn is_pid_alive(pid: u32) -> Option<bool> {
    if let Some(process_handle) = get_process_handle(pid) {
        let alive = check_process_alive(process_handle);
        close_process_handle(process_handle);
        alive
    } else {
        None
    }
}
