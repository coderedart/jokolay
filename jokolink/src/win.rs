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
use winapi::{
    shared::{
        minwindef::{BOOL, FALSE, LPARAM, LPDWORD},
        ntdef::{HANDLE, NULL},
        windef::{HWND, LPRECT, RECT},
    },
    um::{
        errhandlingapi::GetLastError,
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        memoryapi::{MapViewOfFile, FILE_MAP_ALL_ACCESS},
        minwinbase::{SECURITY_ATTRIBUTES, STILL_ACTIVE},
        processthreadsapi::{GetExitCodeProcess, OpenProcess},
        winbase::CreateFileMappingA,
        winnt::{PAGE_READWRITE, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION},
        winuser::{EnumWindows, GetPropA, GetWindowRect, GetWindowThreadProcessId},
    },
};

/// This function creates shared memory for mumble link using Key as the link name
pub fn create_link_shared_mem(key: &str) -> anyhow::Result<*const CMumbleLink> {
    // prepare the key as a cstr to pass to windows functions
    let key_cstr = std::ffi::CString::new(key)?;
    let key_cstr_ptr = key_cstr.as_ptr();
    unsafe {
        // create a Mumble Link shared memory file
        // the file handle will need not be stored because when process exits, the handle will be dropped by windows

        let file_handle = CreateFileMappingA(
            INVALID_HANDLE_VALUE,
            NULL as *mut SECURITY_ATTRIBUTES,
            PAGE_READWRITE,
            0,
            C_MUMBLE_LINK_SIZE as u32,
            key_cstr_ptr,
        );

        // if failed to create shared memory
        if file_handle == NULL {
            anyhow::bail!(
                "could not create file map handle, error code: {}",
                GetLastError()
            );
        }

        // map the shared memory into the address space of our process using the handle we got from creating the shm
        let cml_ptr = MapViewOfFile(file_handle, FILE_MAP_ALL_ACCESS, 0, 0, C_MUMBLE_LINK_SIZE);

        // check if we were successful
        if cml_ptr == NULL {
            anyhow::bail!("could not map view of file, error code: {}", GetLastError());
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
    GetWindowThreadProcessId(window_handle, (&mut handle_pid) as LPDWORD);
    // if handle_pid is null, it means we failed to get the pid. so, we return true so that enumWindows can call us again with the handle to the next window.
    if handle_pid == 0 {
        return 1;
    }
    let handle_pid = handle_pid as HWND;
    // gw2_pid is a long pointer TO a HWND. we cast gw2_pid from isize to a * mut HWND.
    let gw2_pid = gw2_pid as *mut HWND;
    // we check if the pid which gw2_pid references is equal to handle_pid
    if *gw2_pid == handle_pid {
        // we now assign the window_handle to the memory pointed by gw2_pid pointer.
        *gw2_pid = window_handle as HWND;
        return 0;
    }
    return 1;
}

pub fn get_win_pos_dim(link_ptr: *const CMumbleLink) -> anyhow::Result<WindowDimensions> {
    unsafe {
        if !CMumbleLink::is_valid(link_ptr) {
            anyhow::bail!("the MumbleLink is not init yet. so, getting window position is not valid operation");
        }
        let context = (*link_ptr).context.as_ptr() as *const CMumbleContext;
        let mut pid: isize = (*context).process_id as isize;

        let result: BOOL = EnumWindows(Some(get_handle_by_pid), &mut pid as *mut isize as LPARAM);
        if result != 0 {
            anyhow::bail!("couldn't find gw2 window. error code: {}", GetLastError());
        }

        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let status = GetWindowRect(pid as isize as HWND, &mut rect as LPRECT);
        if status == 0 {
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

        let mut pid = get_gw2_pid(link_ptr) as HWND;

        // enumerate windows and get the handle and assign it to the pid variable if the process id of the handle actually matches the pid
        let result: BOOL = EnumWindows(Some(get_handle_by_pid), (&mut pid) as *mut HWND as LPARAM);
        // check if successful
        if result != 0 {
            error!("couldn't find gw2 window. error code: {}", GetLastError());
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
        let atom_string_ptr = atom_string.as_ptr();
        let xid: HANDLE = GetPropA(window_handle as HWND, atom_string_ptr) as HANDLE;
        // check if the xid is actually null, in which case we have failed
        if xid.is_null() {
            error!("xid is NULL");
            bail!("xid is NULL");
        }

        Ok(xid as isize)
    }
}

pub fn is_pid_alive(pid: u32) -> Option<bool> {
    unsafe {
        let process_handle: HANDLE = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION,
            FALSE,
            pid,
        );
        if process_handle == NULL {
            error!(
                "failed to get handle for process id: {} due to error {:?}",
                pid,
                GetLastError()
            );
            return None;
        }
        let mut exit_code = 0u32;
        let result = GetExitCodeProcess(process_handle, &mut exit_code as *mut u32);
        CloseHandle(process_handle);

        if result == 0 {
            error!(
                "failed to get exit code for process due to error: {:?}",
                GetLastError()
            );
            return None;
        }

        if exit_code == STILL_ACTIVE {
            return Some(true);
        } else {
            return Some(false);
        }
    }
}
