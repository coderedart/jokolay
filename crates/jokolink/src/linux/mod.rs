use crate::ctypes::{CMumbleLink, C_MUMBLE_LINK_SIZE_FULL};
use crate::MumbleLink;
use joko_core::prelude::*;
use std::fs::File;
use std::io::{Read, Seek};
// use x11rb::protocol::xproto::{change_property, intern_atom, AtomEnum, GetGeometryReply, PropMode};
// use x11rb::rust_connection::ConnectError;

pub use x11rb::rust_connection::RustConnection;

/// This is the bak
pub struct MumbleLinuxImpl {
    mfile: File,
    // xc: X11Connection,
    link_buffer: LinkBuffer,
}

type LinkBuffer = Box<[u8; C_MUMBLE_LINK_SIZE_FULL]>;

impl MumbleLinuxImpl {
    pub fn new(link_name: &str) -> Result<Self> {
        let mumble_file_name = format!("/dev/shm/{link_name}");
        info!("creating mumble file at {mumble_file_name}");
        let mut mfile = File::options()
            .read(true)
            .write(true) // write/append is needed for the create flag
            .create(true)
            .open(&mumble_file_name)
            .into_diagnostic()
            .wrap_err("failed to create mumble file")?;
        let mut link_buffer = LinkBuffer::new([0u8; C_MUMBLE_LINK_SIZE_FULL]);
        mfile.rewind().into_diagnostic()?;
        mfile
            .read(link_buffer.as_mut())
            .into_diagnostic()
            .wrap_err("failed to get link buffer from mfile")?;

        Ok(MumbleLinuxImpl { mfile, link_buffer })
    }
    pub unsafe fn tick(&mut self) -> Result<()> {
        Ok(())
    }
    pub fn get_link(&mut self) -> Result<MumbleLink> {
        self.mfile.rewind().into_diagnostic()?;
        self.mfile
            .read(self.link_buffer.as_mut())
            .into_diagnostic()
            .wrap_err("failed to get link buffer")?;
        let link = unsafe { MumbleLink::unsafe_load_from_pointer(self.link_buffer.as_ptr() as _)? };

        Ok(link)
    }

    pub fn win_pos_size(&self) -> [i32; 4] {
        CMumbleLink::get_cmumble_link(self.link_buffer.as_ptr() as _)
            .context
            .window_pos_size
    }
    pub fn is_alive(&self) -> bool {
        OffsetDateTime::now_utc().unix_timestamp()
            - CMumbleLink::get_cmumble_link(self.link_buffer.as_ptr() as _)
                .context
                .timestamp as i64
            > 5
    }
    // pub fn set_transient_for(&self) -> Result<()> {
    //     Ok(())
    // Ok(self
    //     .xc
    //     .set_transient_for(xid_from_buffer(&self.link_buffer))?)
    // }
}

// struct X11Connection {
//     jokolay_window_id: u32,
//     transient_for_atom: u32,
//     // net_wm_pid_atom: u32,
//     xc: RustConnection,
// }
// impl X11Connection {
//     pub const WM_TRANSIENT_FOR: &'static str = "WM_TRANSIENT_FOR";
//     // pub const NET_WM_PID: &'static str = "_NET_WM_PID";
//     fn new(jokolay_window_id: u32) -> Result<Self, X11Error> {
//         let (xc, _) = RustConnection::connect(None).expect("failed to create x11 connection");
//         let transient_for_atom = intern_atom(&xc, true, Self::WM_TRANSIENT_FOR.as_bytes())
//             .map_err(|e| X11Error::AtomQueryError {
//                 source: e,
//                 atom_str: Self::WM_TRANSIENT_FOR,
//             })?
//             .reply()
//             .map_err(|e| X11Error::AtomReplyError {
//                 source: e,
//                 atom_str: Self::WM_TRANSIENT_FOR,
//             })?
//             .atom;
//         // let net_wm_pid_atom = intern_atom(&xc, true, Self::NET_WM_PID.as_bytes())
//         //     .map_err(|e| X11Error::AtomQueryError {
//         //         source: e,
//         //         atom_str: Self::NET_WM_PID,
//         //     })?
//         //     .reply()
//         //     .map_err(|e| X11Error::AtomReplyError {
//         //         source: e,
//         //         atom_str: Self::NET_WM_PID,
//         //     })?
//         //     .atom;

//         Ok(Self {
//             jokolay_window_id,
//             transient_for_atom,
//             xc,
//             // net_wm_pid_atom,
//         })
//     }
//     pub fn set_transient_for(&self, parent_window: u32) -> Result<(), X11Error> {
//         if let Ok(xst) = std::env::var("XDG_SESSION_TYPE") {
//             if xst == "wayland" {
//                 tracing::warn!("skipping transient_for because we are on wayland");
//                 return Ok(());
//             }
//             if xst != "x11" {
//                 tracing::warn!("xdg session type is neither wayland not x11: {xst}");
//             }
//         }
//         assert_ne!(parent_window, 0);
//         change_property(
//             &self.xc,
//             PropMode::REPLACE,
//             self.jokolay_window_id,
//             self.transient_for_atom,
//             AtomEnum::WINDOW,
//             32,
//             1,
//             &parent_window.to_ne_bytes(),
//         )
//         .map_err(|e| X11Error::TransientForError {
//             source: e,
//             parent: parent_window,
//             child: self.jokolay_window_id,
//         })?
//         .check()
//         .map_err(|e| X11Error::TransientForReplyError {
//             source: e,
//             parent: parent_window,
//             child: self.jokolay_window_id,
//         })?;
//         Ok(())
//     }

//     pub fn get_window_dimensions(&self, xid: u32) -> Result<[i32; 4]> {
//         assert_ne!(xid, 0);
//         let geometry = x11rb::protocol::xproto::get_geometry(&self.xc, xid)
//             .into_diagnostic()
//             .wrap_err("get geometry fn failed")?
//             .reply()
//             .into_diagnostic()
//             .wrap_err("geometry reply is wrong")?;
//         let translated_coordinates = x11rb::protocol::xproto::translate_coordinates(
//             &self.xc,
//             xid,
//             geometry.root,
//             geometry.x,
//             geometry.y,
//         )
//         .into_diagnostic()
//         .wrap_err("failed to translate coords")?
//         .reply()
//         .into_diagnostic()
//         .wrap_err("translate coords reply error")?;
//         let x_outer = translated_coordinates.dst_x as i32;
//         let y_outer = translated_coordinates.dst_y as i32;
//         let width = geometry.width;
//         let height = geometry.height;

//         tracing::debug!(
//             "translated_x: {}, translated_y: {}, width: {}, height: {}, geo_x: {}, geo_y: {}",
//             x_outer,
//             y_outer,
//             width,
//             height,
//             geometry.x,
//             geometry.y
//         );
//         Ok([x_outer, y_outer, width as _, height as _])
//     }
//     // pub fn get_pid_from_xid(&self, xid: u32) -> Result<u32, String> {
//     //     assert_ne!(xid, 0);

//     //     let pid_prop = get_property(
//     //         &self.xc,
//     //         false,
//     //         xid,
//     //         self.net_wm_pid_atom,
//     //         AtomEnum::CARDINAL,
//     //         0,
//     //         1,
//     //     )
//     //     .expect("coudn't get _NET_WM_PID property gw2")
//     //     .reply()
//     //     .expect("reply for _NET_WM_PID property gw2 ");

//     //     if pid_prop.bytes_after != 0
//     //         && pid_prop.format != 32
//     //         && pid_prop.value_len != 1
//     //         && pid_prop.value.len() != 4
//     //     {
//     //         panic!("invalid pid property {:#?}", pid_prop);
//     //     }
//     //     Ok(u32::from_ne_bytes(pid_prop.value.try_into().expect(
//     //         "pid property value has a bytes length of less than 4",
//     //     )))
//     // }
// }
// pub fn get_frame_extents(xc: &RustConnection, xid: u32) -> Result<(u32, u32, u32, u32)> {
//     assert_ne!(xid, 0);
//     let net_frame_extents_atom = intern_atom(&self.xc, true, b"_NET_FRAME_EXTENTS")
//         .expect("coudn't intern atom for _NET_FRAME_EXTENTS ")?
//         .reply()
//         .expect("reply for intern atom for _NET_FRAME_EXTENTS")?
//         .atom;
//     let frame_prop = get_property(
//         &self.xc,
//         false,
//         xid,
//         net_frame_extents_atom,
//         AtomEnum::ANY,
//         0,
//         100,
//     )
//     .expect("coudn't get frame property gw2")?
//     .reply()
//     .expect("reply for frame property gw2")?;

//     if frame_prop.bytes_after != 0 {
//         bail!(
//             "bytes after in frame property is {}",
//             frame_prop.bytes_after
//         );
//     }
//     if frame_prop.format != 32 {
//         bail!("frame_prop format is {}", frame_prop.format);
//     }
//     if frame_prop.value_len != 4 {
//         bail!("frame_prop value_len is {}", frame_prop.value_len);
//     }
//     if frame_prop.value.len() != 16 {
//         bail!("frame_prop.value.len() is {}", frame_prop.value.len());
//     }
//     // avoid bytemuck dependency and just do this raw.
//     let mut arr = [0u8; 4];
//     arr.copy_from_slice(&frame_prop.value[0..4]);
//     let left_border = u32::from_ne_bytes(arr);
//     arr.copy_from_slice(&frame_prop.value[4..8]);
//     let right_border = u32::from_ne_bytes(arr);
//     arr.copy_from_slice(&frame_prop.value[8..12]);
//     let top_border = u32::from_ne_bytes(arr);
//     arr.copy_from_slice(&frame_prop.value[12..16]);
//     let bottom_border = u32::from_ne_bytes(arr);
//     Ok((left_border, right_border, top_border, bottom_border))
// }

// pub fn get_gw2_pid(&mut self) -> Result<u32> {
//     assert_ne!(self.gw2_window_handle, 0);
//     let pid_atom = x11rb::protocol::xproto::intern_atom(&self.&self.xc, true, b"_NET_WM_PID")
//         .expect("could not intern atom '_NET_WM_PID'")?
//         .reply()
//         .expect("reply error while interning '_NET_WM_PID'.")?
//         .atom;
//     let reply = x11rb::protocol::xproto::get_property(
//         &self.&self.xc,
//         false,
//         self.gw2_window_handle,
//         pid_atom,
//         x11rb::protocol::xproto::AtomEnum::CARDINAL,
//         0,
//         1,
//     )
//     .expect("could not request '_NET_WM_PID' for gw2 window handle ")?
//     .reply()
//     .expect("the reply for '_NET_WM_PID' of gw2 handle ")?;

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
