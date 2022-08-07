use crate::mlink::{MumbleLink, USEFUL_C_MUMBLE_LINK_SIZE};
use crate::WindowDimensions;
use color_eyre::eyre::{bail, WrapErr};
use color_eyre::Result;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;
use sysinfo::{Pid, ProcessRefreshKind, System, SystemExt};
use x11rb::protocol::xproto::{change_property, get_property, intern_atom, AtomEnum, PropMode};

use super::{MumbleFile, MumbleFileTrait, UpdatedMumbleData};

pub use x11rb::rust_connection::RustConnection;

pub type MumbleBackend = std::fs::File;
const LINK_BUFFER_SIZE: usize = USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<u32>();
type LinkBuffer = [u8; LINK_BUFFER_SIZE];

impl MumbleFileTrait for MumbleFile {
    fn new(link_name: &str, latest_time: f64) -> Result<Self> {
        let mut f = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("/dev/shm/{}", link_name))
            .wrap_err_with(|| format!("MumbleFile open error: {link_name}."))?;
        let buffer = get_link_buffer(&mut f)?;

        let mut link = MumbleLink::default();
        let link = if link.update_from_slice(&buffer).is_ok() {
            link
        } else {
            MumbleLink::default()
        };
        let xid: u32 = xid_from_buffer(&buffer);

        Ok(Self {
            link_name: Arc::from(link_name),
            backend: f,
            last_ui_tick_changed_time: latest_time,
            last_link_update: latest_time,
            previous_ui_tick: link.ui_tick,
            previous_unique_id: xid,
        })
    }
    fn get_link(&mut self, latest_time: f64) -> Result<Option<UpdatedMumbleData>> {
        let previous_tick = self.previous_ui_tick;

        let mut present_link = MumbleLink::default();
        let buffer = get_link_buffer(&mut self.backend)?;
        present_link.update_from_slice(&buffer)?;
        let present_tick = present_link.ui_tick;
        let xid: u32 = xid_from_buffer(&buffer);
        // because we could successfully parse the mumble link, we can update the latest updatetime.
        self.last_link_update = latest_time;
        // if present_tick zero, mumble is not init, so we return None
        // if previous unique id and present unique id are same AND previous ui tick and present ui tick are same, then there's been no update to mumble data
        // since last frame.
        // we check for unique id too to cover the edge case of two gw2 instances running at the same time and updating the link with the same uitick.
        // by checking that the unique id is same, we can guarantee that there's been no change at all
        if present_tick == 0 || (previous_tick == present_tick && xid == self.previous_unique_id) {
            Ok(None)
        } else {
            // there's been a change in mumble link, so we are in this branch. update the previous values to current values.
            self.previous_ui_tick = present_tick;
            self.last_ui_tick_changed_time = latest_time;
            Ok(Some(UpdatedMumbleData {
                unique_id: xid,
                link: present_link,
            }))
        }
    }
}

pub struct GW2InstanceData {
    xid: u32,
    pid: i32,
}
impl GW2InstanceData {
    pub fn get_xid(&self) -> u32 {
        self.xid
    }
    pub fn get_pid(&self) -> i32 {
        self.pid
    }
    pub fn get_unique_id(&self) -> u32 {
        self.xid
    }
    /// try to open the Mumble Link file created by jokolink under /dev/shm . creates empty file if it doesn't exist
    pub fn new(unique_id: usize, xc: &RustConnection) -> Result<Self> {
        let xid = unique_id.try_into().expect("cannot fit window id into u32");
        let pid = get_pid_from_xid(xc, xid)
            .wrap_err("failed to get PID from xid")?
            .try_into()
            .expect("failed to fit pid into i32");
        Ok(Self { xid, pid })
    }
    pub fn is_alive(&self, sys: &mut System) -> bool {
        sys.refresh_process_specifics(Pid::from(self.pid), ProcessRefreshKind::new())
    }
    pub fn get_window_dimensions(&self, xc: &RustConnection) -> Result<WindowDimensions> {
        let (x, y, w, h) =
            get_window_dimensions(&xc, self.xid).wrap_err("failed to get window dimensions")?;
        Ok(WindowDimensions {
            x,
            y,
            width: w,
            height: h,
        })
    }
}

/// read the file to get a buffer which has the USEFUL mumble link data and the x11 window id of gw2
pub fn get_link_buffer(mfile: &mut File) -> Result<LinkBuffer> {
    mfile
        .seek(SeekFrom::Start(0))
        .wrap_err("failed to seek to start on mumble file")?;
    let mut buffer = [0u8; LINK_BUFFER_SIZE];
    mfile
        .read(&mut buffer)
        .wrap_err("failed to read to buffer from mumble file due to error")?;
    Ok(buffer)
}

/// get the isize xid from the gw2 mumble file
/// panics if ui_tick is zero
pub fn xid_from_buffer(buffer: &LinkBuffer) -> u32 {
    let mut xid_buffer = [0u8; std::mem::size_of::<u32>()];
    assert_eq!(xid_buffer.len(), 4);
    xid_buffer.copy_from_slice(&buffer[USEFUL_C_MUMBLE_LINK_SIZE..]);
    u32::from_ne_bytes(xid_buffer)
}

pub fn set_transient_for(xc: &RustConnection, child_window: u32, parent_window: u32) -> Result<()> {
    assert_ne!(child_window, 0);
    assert_ne!(parent_window, 0);
    let transient_for_atom = intern_atom(xc, true, b"WM_TRANSIENT_FOR")
        .wrap_err("coudn't intern WM_TRANSIENT_FOR")?
        .reply()
        .wrap_err("coudn't parse reply for intern WM_TRANSIENT_FOR ")?
        .atom;
    change_property(
        xc,
        PropMode::REPLACE,
        child_window,
        transient_for_atom,
        AtomEnum::WINDOW,
        32,
        1,
        &parent_window.to_ne_bytes(),
    )
    .wrap_err("coudn't send WM_TRANSIENT_FOR change property request")?
    .check()
    .wrap_err("reply for changing transient_for")?;
    Ok(())
}

pub fn get_window_dimensions(xc: &RustConnection, xid: u32) -> Result<(i32, i32, u32, u32)> {
    assert_ne!(xid, 0);
    let geometry = x11rb::protocol::xproto::get_geometry(xc, xid)
        .wrap_err("coudn't get geometry of gw2")?
        .reply()
        .wrap_err("reply for getting geometry")?;
    let translated_coordinates = x11rb::protocol::xproto::translate_coordinates(
        xc,
        xid,
        geometry.root,
        geometry.x,
        geometry.y,
    )
    .wrap_err("coudn't get translation coords of gw2")?
    .reply()
    .wrap_err("reply for getting translation coords")?;
    let x_outer = translated_coordinates.dst_x as i32;
    let y_outer = translated_coordinates.dst_y as i32;
    let width = geometry.width;
    let height = geometry.height;

    tracing::debug!(
        "translated_x: {}, translated_y: {}, width: {}, height: {}, geo_x: {}, geo_y: {}",
        x_outer,
        y_outer,
        width,
        height,
        geometry.x,
        geometry.y
    );
    Ok((x_outer, y_outer, width as u32, height as u32))
}
pub fn get_frame_extents(xc: &RustConnection, xid: u32) -> Result<(u32, u32, u32, u32)> {
    assert_ne!(xid, 0);
    let net_frame_extents_atom = intern_atom(xc, true, b"_NET_FRAME_EXTENTS")
        .wrap_err("coudn't intern atom for _NET_FRAME_EXTENTS ")?
        .reply()
        .wrap_err("reply for intern atom for _NET_FRAME_EXTENTS")?
        .atom;
    let frame_prop = get_property(
        xc,
        false,
        xid,
        net_frame_extents_atom,
        AtomEnum::ANY,
        0,
        100,
    )
    .wrap_err("coudn't get frame property gw2")?
    .reply()
    .wrap_err("reply for frame property gw2")?;

    if frame_prop.bytes_after != 0 {
        bail!(
            "bytes after in frame property is {}",
            frame_prop.bytes_after
        );
    }
    if frame_prop.format != 32 {
        bail!("frame_prop format is {}", frame_prop.format);
    }
    if frame_prop.value_len != 4 {
        bail!("frame_prop value_len is {}", frame_prop.value_len);
    }
    if frame_prop.value.len() != 16 {
        bail!("frame_prop.value.len() is {}", frame_prop.value.len());
    }
    let mut buffer = [0u32; 4];
    buffer.copy_from_slice(bytemuck::cast_slice(&frame_prop.value));
    let left_border = buffer[0];
    let right_border = buffer[1];
    let top_border = buffer[2];
    let bottom_border = buffer[3];
    Ok((left_border, right_border, top_border, bottom_border))
}
pub fn get_pid_from_xid(xc: &RustConnection, xid: u32) -> Result<u32> {
    assert_ne!(xid, 0);
    let pid_atom = intern_atom(xc, true, b"_NET_WM_PID")
        .wrap_err("coudn't intern atom for _NET_WM_PID")?
        .reply()
        .wrap_err("reply for intern atom for _NET_WM_PID")?;

    let pid_prop = get_property(xc, false, xid, pid_atom.atom, AtomEnum::CARDINAL, 0, 1)
        .wrap_err("coudn't get _NET_WM_PID property gw2")?
        .reply()
        .wrap_err("reply for _NET_WM_PID property gw2 ")?;

    if pid_prop.bytes_after != 0 {
        bail!(
            "bytes after in _NET_WM_PID property is {}",
            pid_prop.bytes_after
        );
    }
    if pid_prop.format != 32 {
        bail!("_NET_WM_PID format is {}", pid_prop.format);
    }
    if pid_prop.value_len != 1 {
        bail!("_NET_WM_PID value_len is {}", pid_prop.value_len);
    }
    if pid_prop.value.len() != 4 {
        bail!("_NET_WM_PID.value.len() is {}", pid_prop.value.len());
    }
    let mut buffer = [0u8; 4];
    buffer.copy_from_slice(&pid_prop.value);
    Ok(u32::from_ne_bytes(buffer))
}

// pub fn get_gw2_pid(&mut self) -> color_eyre::Result<u32> {
//     assert_ne!(self.gw2_window_handle, 0);
//     let pid_atom = x11rb::protocol::xproto::intern_atom(&self.xc, true, b"_NET_WM_PID")
//         .wrap_err("could not intern atom '_NET_WM_PID'")?
//         .reply()
//         .wrap_err("reply error while interning '_NET_WM_PID'.")?
//         .atom;
//     let reply = x11rb::protocol::xproto::get_property(
//         &self.xc,
//         false,
//         self.gw2_window_handle,
//         pid_atom,
//         x11rb::protocol::xproto::AtomEnum::CARDINAL,
//         0,
//         1,
//     )
//     .wrap_err("could not request '_NET_WM_PID' for gw2 window handle ")?
//     .reply()
//     .wrap_err("the reply for '_NET_WM_PID' of gw2 handle ")?;

//     let pid_format = 32;
//     if pid_format != reply.format {
//         bail!("pid_format is not 32. so, type is wrong");
//     }
//     let pid_buffer_size = 4;
//     if pid_buffer_size != reply.value.len() {
//         bail!("pid_buffer is not 4 bytes");
//     }
//     let value_len = 1;
//     if value_len != reply.value_len {
//         bail!("pid reply's value_len is not 1");
//     }
//     let remaining_bytes_len = 0;
//     if remaining_bytes_len != reply.bytes_after {
//         bail!("we still have too many bytes remaining after reading '_NET_WM_PID'");
//     }
//     let mut buffer = [0u8; 4];
//     buffer.copy_from_slice(&reply.value);
//     Ok(u32::from_ne_bytes(buffer))
// }
