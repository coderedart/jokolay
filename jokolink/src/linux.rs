use crate::mlink::{MumbleLink, USEFUL_C_MUMBLE_LINK_SIZE};
use crate::MumbleConfig;
use anyhow::{bail, Context};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use sysinfo::SystemExt;
use tracing::warn;
use x11rb::protocol::xproto::{change_property, get_property, intern_atom, AtomEnum, PropMode};
use x11rb::rust_connection::RustConnection;

const LINK_BUFFER_SIZE: usize = USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<isize>();
type LinkBuffer = [u8; LINK_BUFFER_SIZE];
/// This source will be the used to abstract the linux/windows way of getting MumbleLink
/// on windows, this represents the shared memory pointer to mumblelink, and as long as one of gw2 or a client like us is alive, the shared memory will stay alive
/// on linux, this will be a File in /dev/shm that will only exist if jokolink created it at some point in time. this lives in ram, so reading from it is pretty much free.
#[derive(Debug)]
pub struct MumbleSource {
    pub mfile: std::fs::File,
    pub xc: RustConnection,
    pub ow_window_handle: u32,
    pub gw2_window_handle: u32,
    pub gw2_pid: u32,
    pub gw2_pos: [i32; 2],
    pub gw2_size: [u32; 2],
    pub link: MumbleLink,
    pub last_uitick_update: f64,
    pub last_pos_size_update: f64,
}

impl MumbleSource {
    /// try to open the Mumble Link file created by jokolink under /dev/shm . creates empty file if it doesn't exist
    pub fn new(
        config: &MumbleConfig,
        glfw_time: f64,
        ow_window_handle: u32,
    ) -> anyhow::Result<MumbleSource> {
        let key = &config.link_name;
        let mut f = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("/dev/shm/{}", key))
            .with_context(|| format!("MumbleFile open error: {key}."))?;
        let (xc, _display) = RustConnection::connect(None)
            .context("failed to connect to x11 with rust connection")?;

        // we pre-initialize mumble link. if there's garbage data from gw2's previous run,
        // then when we check for previouslink.tick != present_tick in tick() method,
        // we don't know if it is due to frame advancing with a normal gw2 or
        // present frame uitick being from garbage and previoustick from default link being 0.
        let buffer = get_link_buffer(&mut f)?;

        let mut link = MumbleLink::default();
        let link = if link.update_from_slice(&buffer).is_ok() {
            link
        } else {
            MumbleLink::default()
        };

        let resultsrc = MumbleSource {
            mfile: f,
            ow_window_handle,
            xc,
            gw2_pid: 0,
            gw2_window_handle: 0,
            gw2_pos: [0, 0],
            gw2_size: [0, 0],
            last_pos_size_update: glfw_time,
            link,
            last_uitick_update: glfw_time,
        };
        Ok(resultsrc)
    }

    pub fn tick(&mut self, latest_time: f64, sys: &mut sysinfo::System) -> anyhow::Result<()> {
        let previous_tick = self.link.ui_tick;

        // to make sure that self.link is always "valid" and without any errors while updating from buffer,
        // we will use a new link and after checking that its valid, we will assign it.
        let mut present_link = MumbleLink::default();
        let buffer = get_link_buffer(&mut self.mfile)?;
        present_link.update_from_slice(&buffer)?;
        let present_tick = present_link.ui_tick;

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
        }
        // we handled present == previous and present < previous. so, we know that present is greater than previous
        self.last_uitick_update = latest_time;
        // by updating link here, if gw2 is dead, we make sure that next frame, if mumble doesn't update, we don't reach the checking for xid again.
        self.link = present_link;

        // we only update pos/size atleast a second after previous successful update
        if latest_time - self.last_pos_size_update > 1.0 {
            // for linux, first get window xid from jokolink using "wine_x11" thingy and then get pid from NET_WM_PID
            let xid: isize = xid_from_buffer(&buffer);
            self.gw2_window_handle = xid
                .try_into()
                .with_context(|| format!("failed to fit gw2 xid {} into u32", xid))?;
            // if window handle is still zero, it means jokolink didn't update the window xid yet, so we skip getting sizes
            // if gw2pid is zero, but window handle is not zero, it means we just got gw2's window handle, so we try to set gw2 pid from "NET WM PID"
            // and as we just found gw2 window, we set transient for
            if self.gw2_window_handle != 0 && self.gw2_pid == 0 {
                self.gw2_pid = get_pid_from_xid(&self.xc, self.gw2_window_handle)
                    .context("failed to get pid when tick is greater than zero")?;
                assert_ne!(self.gw2_pid, 0);
                set_transient_for(&self.xc, self.ow_window_handle, self.gw2_window_handle)
                    .context("failed to set transient for")?;
            }
            // if gw2_pid is set, it means we got gw2 window as well as process id.
            if self.gw2_pid != 0 {
                // before we do anything, we first check if gw2 is still alive, otherwise, we just set pid/xid to zero, so that we can start over
                // pid_t is i32 in libc
                let pid: i32 = self
                    .gw2_pid
                    .try_into()
                    .context("failed to convert gw2 pid into unix pid")?;
                if !sys.refresh_process(sysinfo::Pid::from(pid)) {
                    self.gw2_pid = 0;
                    self.gw2_window_handle = 0;
                    return Ok(());
                }
                // if gw2 is alive, we get the dimensions and set them.
                let (x, y, w, h) = get_window_dimensions(&self.xc, self.gw2_window_handle)?;
                self.gw2_pos = [x, y];
                self.gw2_size = [w, h];
                self.last_pos_size_update = latest_time;
            }
        }
        Ok(())
    }
    pub fn get_link(&self) -> &MumbleLink {
        &self.link
    }
}

/// read the file to get a buffer which has the USEFUL mumble link data and the x11 window id of gw2
pub fn get_link_buffer(mfile: &mut File) -> anyhow::Result<LinkBuffer> {
    mfile
        .seek(SeekFrom::Start(0))
        .context("failed to seek to start on mumble file")?;
    let mut buffer = [0u8; LINK_BUFFER_SIZE];
    mfile
        .read(&mut buffer)
        .context("failed to read to buffer from mumble file due to error")?;
    Ok(buffer)
}

/// get the isize xid from the gw2 mumble file
/// panics if ui_tick is zero
pub fn xid_from_buffer(buffer: &LinkBuffer) -> isize {
    let mut xid_buffer = [0u8; std::mem::size_of::<isize>()];
    xid_buffer.copy_from_slice(&buffer[USEFUL_C_MUMBLE_LINK_SIZE..]);
    isize::from_ne_bytes(xid_buffer)
}

pub fn set_transient_for(
    xc: &RustConnection,
    child_window: u32,
    parent_window: u32,
) -> anyhow::Result<()> {
    assert_ne!(child_window, 0);
    assert_ne!(parent_window, 0);
    let transient_for_atom = intern_atom(xc, true, b"WM_TRANSIENT_FOR")
        .context("coudn't intern WM_TRANSIENT_FOR")?
        .reply()
        .context("coudn't parse reply for intern WM_TRANSIENT_FOR ")?
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
    .context("coudn't send WM_TRANSIENT_FOR change property request")?
    .check()
    .context("reply for changing transient_for")?;
    Ok(())
}

pub fn get_window_dimensions(
    xc: &RustConnection,
    xid: u32,
) -> anyhow::Result<(i32, i32, u32, u32)> {
    assert_ne!(xid, 0);
    let geometry = x11rb::protocol::xproto::get_geometry(xc, xid)
        .context("coudn't get geometry of gw2")?
        .reply()
        .context("reply for getting geometry")?;
    let translated_coordinates = x11rb::protocol::xproto::translate_coordinates(
        xc,
        xid,
        geometry.root,
        geometry.x,
        geometry.y,
    )
    .context("coudn't get translation coords of gw2")?
    .reply()
    .context("reply for getting translation coords")?;
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
pub fn get_frame_extents(xc: &RustConnection, xid: u32) -> anyhow::Result<(u32, u32, u32, u32)> {
    assert_ne!(xid, 0);
    let net_frame_extents_atom = intern_atom(xc, true, b"_NET_FRAME_EXTENTS")
        .context("coudn't intern atom for _NET_FRAME_EXTENTS ")?
        .reply()
        .context("reply for intern atom for _NET_FRAME_EXTENTS")?
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
    .context("coudn't get frame property gw2")?
    .reply()
    .context("reply for frame property gw2")?;

    if frame_prop.bytes_after != 0 {
        anyhow::bail!(
            "bytes after in frame property is {}",
            frame_prop.bytes_after
        );
    }
    if frame_prop.format != 32 {
        anyhow::bail!("frame_prop format is {}", frame_prop.format);
    }
    if frame_prop.value_len != 4 {
        anyhow::bail!("frame_prop value_len is {}", frame_prop.value_len);
    }
    if frame_prop.value.len() != 16 {
        anyhow::bail!("frame_prop.value.len() is {}", frame_prop.value.len());
    }
    let mut buffer = [0u32; 4];
    buffer.copy_from_slice(bytemuck::cast_slice(&frame_prop.value));
    let left_border = buffer[0];
    let right_border = buffer[1];
    let top_border = buffer[2];
    let bottom_border = buffer[3];
    Ok((left_border, right_border, top_border, bottom_border))
}
pub fn get_pid_from_xid(xc: &RustConnection, xid: u32) -> anyhow::Result<u32> {
    assert_ne!(xid, 0);
    let pid_atom = intern_atom(xc, true, b"_NET_WM_PID")
        .context("coudn't intern atom for _NET_WM_PID")?
        .reply()
        .context("reply for intern atom for _NET_WM_PID")?;

    let pid_prop = get_property(xc, false, xid, pid_atom.atom, AtomEnum::CARDINAL, 0, 1)
        .context("coudn't get _NET_WM_PID property gw2")?
        .reply()
        .context("reply for _NET_WM_PID property gw2 ")?;

    if pid_prop.bytes_after != 0 {
        anyhow::bail!(
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

// pub fn get_gw2_pid(&mut self) -> anyhow::Result<u32> {
//     assert_ne!(self.gw2_window_handle, 0);
//     let pid_atom = x11rb::protocol::xproto::intern_atom(&self.xc, true, b"_NET_WM_PID")
//         .context("could not intern atom '_NET_WM_PID'")?
//         .reply()
//         .context("reply error while interning '_NET_WM_PID'.")?
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
//     .context("could not request '_NET_WM_PID' for gw2 window handle ")?
//     .reply()
//     .context("the reply for '_NET_WM_PID' of gw2 handle ")?;

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
