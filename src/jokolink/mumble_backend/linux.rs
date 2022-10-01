use crate::jokolink::mlink::{MumbleLink, MumbleUpdateError, USEFUL_C_MUMBLE_LINK_SIZE};
use crate::jokolink::WindowDimensions;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use tracing::error;
use x11rb::protocol::xproto::{change_property, intern_atom, AtomEnum, GetGeometryReply, PropMode};
use x11rb::rust_connection::ConnectError;

pub use x11rb::rust_connection::RustConnection;

/// This is the bak
pub struct MumbleLinuxImpl {
    mfile: File,
    xc: X11Connection,
    link_buffer: LinkBuffer,
}

const LINK_BUFFER_SIZE: usize = USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<u32>();
type LinkBuffer = Box<[u8; LINK_BUFFER_SIZE]>;

impl MumbleLinuxImpl {
    pub fn new(link_name: &str, jokolay_window_id: u32) -> Result<Self, MumbleLinuxError> {
        let mumble_file_name = format!("/dev/shm/{}", link_name);
        let mut mfile = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(&mumble_file_name)
            .map_err(|e| MumbleLinuxError::FileCreateError {
                name: mumble_file_name,
                error: e,
            })?;
        let mut link_buffer = LinkBuffer::new([0u8; LINK_BUFFER_SIZE]);
        get_link_buffer(&mut mfile, link_buffer.as_mut())?;


        let xc = X11Connection::new(jokolay_window_id)?;
        Ok(MumbleLinuxImpl {
            mfile,
            xc,
            link_buffer,
        })
    }
    pub fn get_link(&mut self) -> Result<MumbleLink, MumbleLinuxError> {
        let mut present_link = MumbleLink::default();
        get_link_buffer(&mut self.mfile, &mut self.link_buffer)?;
        present_link.update_from_slice((&self.link_buffer[0..1093]).try_into().unwrap())?;
        present_link.context.process_id = xid_from_buffer(&self.link_buffer);
        Ok(present_link)
    }

    pub fn get_window_dimensions(&self) -> Result<WindowDimensions, MumbleLinuxError> {
        let xid = xid_from_buffer(&self.link_buffer);
        if xid == 0 {
            return Err(MumbleLinuxError::MumbleNotInit);
        }
        Ok(self.xc.get_window_dimensions(xid)?)
    }
    pub fn set_transient_for(&self) -> Result<(), MumbleLinuxError> {
        Ok(self
            .xc
            .set_transient_for(xid_from_buffer(&self.link_buffer))?)
    }
}

/// read the file to get a buffer which has the USEFUL mumble link data and the x11 window id of gw2
fn get_link_buffer(
    mfile: &mut File,
    buffer: &mut [u8; LINK_BUFFER_SIZE],
) -> Result<(), MumbleLinuxError> {
    mfile
        .seek(SeekFrom::Start(0))
        .map_err(MumbleLinuxError::FileSeekError)?;
    mfile
        .read(buffer.as_mut())
        .map_err(MumbleLinuxError::FileReadError)?;
    Ok(())
}

/// get the x11 window id of gw2 window from the gw2 mumble link buffer
/// panics if the link buffer is not the required length
fn xid_from_buffer(buffer: &LinkBuffer) -> u32 {
    u32::from_ne_bytes(
        buffer[USEFUL_C_MUMBLE_LINK_SIZE..]
            .try_into()
            .expect("link buffer doesn't have trailing 4 bytes space filled with xid"),
    )
}

struct X11Connection {
    jokolay_window_id: u32,
    transient_for_atom: u32,
    // net_wm_pid_atom: u32,
    xc: RustConnection,
}
impl X11Connection {
    pub const WM_TRANSIENT_FOR: &'static str = "WM_TRANSIENT_FOR";
    // pub const NET_WM_PID: &'static str = "_NET_WM_PID";
    fn new(jokolay_window_id: u32) -> Result<Self, X11Error> {
        let (xc, _) = RustConnection::connect(None).expect("failed to create x11 connection");
        let transient_for_atom = intern_atom(&xc, true, Self::WM_TRANSIENT_FOR.as_bytes())
            .map_err(|e| X11Error::AtomQueryError {
                source: e,
                atom_str: Self::WM_TRANSIENT_FOR,
            })?
            .reply()
            .map_err(|e| X11Error::AtomReplyError {
                source: e,
                atom_str: Self::WM_TRANSIENT_FOR,
            })?
            .atom;
        // let net_wm_pid_atom = intern_atom(&xc, true, Self::NET_WM_PID.as_bytes())
        //     .map_err(|e| X11Error::AtomQueryError {
        //         source: e,
        //         atom_str: Self::NET_WM_PID,
        //     })?
        //     .reply()
        //     .map_err(|e| X11Error::AtomReplyError {
        //         source: e,
        //         atom_str: Self::NET_WM_PID,
        //     })?
        //     .atom;

        Ok(Self {
            jokolay_window_id,
            transient_for_atom,
            xc,
            // net_wm_pid_atom,
        })
    }
    pub fn set_transient_for(&self, parent_window: u32) -> Result<(), X11Error> {
        assert_ne!(parent_window, 0);

        change_property(
            &self.xc,
            PropMode::REPLACE,
            self.jokolay_window_id,
            self.transient_for_atom,
            AtomEnum::WINDOW,
            32,
            1,
            &parent_window.to_ne_bytes(),
        )
        .map_err(|e| X11Error::TransientForError {
            source: e,
            parent: parent_window,
            child: self.jokolay_window_id,
        })?
        .check()
        .map_err(|e| X11Error::TransientForReplyError {
            source: e,
            parent: parent_window,
            child: self.jokolay_window_id,
        })?;
        Ok(())
    }

    pub fn get_window_dimensions(&self, xid: u32) -> Result<WindowDimensions, X11Error> {
        assert_ne!(xid, 0);
        let geometry = x11rb::protocol::xproto::get_geometry(&self.xc, xid)
            .map_err(|e| X11Error::GeometryError {
                source: e,
                window: xid,
            })?
            .reply()
            .map_err(|e| X11Error::GeometryReplyError {
                source: e,
                window: xid,
            })?;
        let translated_coordinates = x11rb::protocol::xproto::translate_coordinates(
            &self.xc,
            xid,
            geometry.root,
            geometry.x,
            geometry.y,
        )
        .map_err(|e| X11Error::TranslateCoordsError {
            source: e,
            window: xid,
            geometry,
        })?
        .reply()
        .map_err(|e| X11Error::TranslateCoordsReplyError {
            source: e,
            window: xid,
            geometry,
        })?;
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
        Ok(WindowDimensions {
            x: x_outer,
            y: y_outer,
            width: width as u32,
            height: height as u32,
        })
    }
    // pub fn get_pid_from_xid(&self, xid: u32) -> Result<u32, String> {
    //     assert_ne!(xid, 0);

    //     let pid_prop = get_property(
    //         &self.xc,
    //         false,
    //         xid,
    //         self.net_wm_pid_atom,
    //         AtomEnum::CARDINAL,
    //         0,
    //         1,
    //     )
    //     .expect("coudn't get _NET_WM_PID property gw2")
    //     .reply()
    //     .expect("reply for _NET_WM_PID property gw2 ");

    //     if pid_prop.bytes_after != 0
    //         && pid_prop.format != 32
    //         && pid_prop.value_len != 1
    //         && pid_prop.value.len() != 4
    //     {
    //         panic!("invalid pid property {:#?}", pid_prop);
    //     }
    //     Ok(u32::from_ne_bytes(pid_prop.value.try_into().expect(
    //         "pid property value has a bytes length of less than 4",
    //     )))
    // }
}
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

// pub fn get_gw2_pid(&mut self) -> color_eyre::Result<u32> {
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
#[derive(Debug, thiserror::Error)]
pub enum MumbleLinuxError {
    #[error("Mumble File Create Error")]
    FileCreateError { name: String, error: std::io::Error },
    #[error("Mumble File Seek Error")]
    FileSeekError(std::io::Error),
    #[error("Mumble File Read Error")]
    FileReadError(std::io::Error),
    #[error("Mumble is not initialized yet")]
    MumbleNotInit,
    #[error("Mumble Update Error")]
    MumbleUpdateError(#[from] MumbleUpdateError),
    #[error("X11 Error: {0}")]
    X11Error(#[from] X11Error),
}
#[derive(Debug, thiserror::Error)]
pub enum X11Error {
    #[error("Failed to create a new Rust X11 connection due to error: {0}")]
    X11ConnectionFailure(#[from] ConnectError),
    #[error("failed to query for atom {atom_str} due to error: {source}")]
    AtomQueryError {
        source: x11rb::errors::ConnectionError,
        atom_str: &'static str,
    },
    #[error("failed to get reply for atom query: {atom_str} due to error: {source}")]
    AtomReplyError {
        source: x11rb::errors::ReplyError,
        atom_str: &'static str,
    },
    #[error(
        "failed to set transient for child: {child} to parent: {parent} due to error: {source} "
    )]
    TransientForError {
        source: x11rb::errors::ConnectionError,
        parent: u32,
        child: u32,
    },
    #[error("failed to get reply of request of set transient for child: {child} to parent: {parent} due to error: {source} ")]
    TransientForReplyError {
        source: x11rb::errors::ReplyError,
        parent: u32,
        child: u32,
    },
    #[error("failed to get geometry of window: {window} due to error: {source} ")]
    GeometryError {
        source: x11rb::errors::ConnectionError,
        window: u32,
    },
    #[error("failed to get reply get geometry of window: {window} due to error: {source} ")]
    GeometryReplyError {
        source: x11rb::errors::ReplyError,
        window: u32,
    },

    #[error("failed to get geometry of window: {window} due to error: {source} ")]
    TranslateCoordsError {
        source: x11rb::errors::ConnectionError,
        window: u32,
        geometry: GetGeometryReply,
    },
    #[error("failed to get reply get geometry of window: {window} due to error: {source} ")]
    TranslateCoordsReplyError {
        source: x11rb::errors::ReplyError,
        window: u32,
        geometry: GetGeometryReply,
    },
}
