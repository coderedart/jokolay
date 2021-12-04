use log::error;

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use crate::{
    mlink::{MumbleLink, USEFUL_C_MUMBLE_LINK_SIZE},
    MumbleSource,
};

impl MumbleSource {
    /// try to open the Mumble Link file created by jokolink under /dev/shm . fails if jokolink didn't create it.
    pub fn new(key: &str) -> Option<MumbleSource> {
        let f = File::open(format!("/dev/shm/{}", key))
            .map_err(|e| {
                error!("MumbleFile open error: {:?}.", &e);
                e
            })
            .ok()?;
        Some(MumbleSource { mumble_src: f })
    }
    /// read the file to get a buffer which has the USEFUL mumble link data and the x11 window id of gw2
    pub fn get_link_buffer(
        &mut self,
    ) -> anyhow::Result<[u8; USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<isize>()]> {
        let mfile = &mut self.mumble_src;
        mfile.seek(SeekFrom::Start(0)).map_err(|e| {
            log::error!(
                "failed to seek to start on mumble file due to error: {:?}",
                &e
            );
            e
        })?;
        let mut buffer = [0u8; USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<isize>()];
        mfile.read(&mut buffer).map_err(|e| {
            log::error!(
                "failed to read to buffer from mumble file due to error: {:?}",
                &e
            );
            e
        })?;
        Ok(buffer)
    }
    /// creates a default mumble link and tries to update from the buffer read from file. then returns the link
    /// if there's a error, it ignores that, so the link might be wrong
    pub fn get_link(&mut self) -> anyhow::Result<MumbleLink> {
        let mut link = MumbleLink::default();
        let buffer = self.get_link_buffer()?;
        link.update_from_slice(&buffer).map_err(|e| {
            log::error!(
                "failed to update mumble from buffer slice due to error: {:?}",
                &e
            );
            e
        })?;
        Ok(link)
    }
    /// get the isize xid from the gw2 mumble file
    pub fn get_gw2_window_handle(&mut self) -> anyhow::Result<isize> {
        let buffer = self.get_link_buffer()?;
        let mut xid_buffer = [0u8; std::mem::size_of::<isize>()];
        xid_buffer.copy_from_slice(&buffer[USEFUL_C_MUMBLE_LINK_SIZE..]);
        Ok(isize::from_ne_bytes(xid_buffer))
    }
}
