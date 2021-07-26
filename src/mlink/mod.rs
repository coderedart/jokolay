use jokolink::mlink::*;
use log::error;

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

/// This is used to update
#[derive(Debug)]
pub struct MumbleManager {
    pub key: String,
    pub link: MumbleLink,
    pub window_dimensions: WindowDimensions,
    #[cfg(target_os = "linux")]
    pub mumble_file: Option<File>,
}

impl MumbleManager {
    pub fn new(key: &str) -> anyhow::Result<MumbleManager> {
        let mumble_file = File::open(format!("/dev/shm/{}", key))
            .map_err(|e| {
                error!("MumbleFile open error: {:?}", &e);
                e
            })
            .ok();
        let link = MumbleLink::default();
        let window_dimensions = WindowDimensions::default();
        let manager = MumbleManager {
            key: key.to_string(),
            link,
            window_dimensions,
            mumble_file,
        };
        Ok(manager)
    }

    pub fn get_window_dimensions(&self) -> WindowDimensions {
        self.window_dimensions
    }
    pub fn get_link(&self) -> &MumbleLink {
        &self.link
    }
    pub fn update(&mut self) {
        if self.mumble_file.is_none() {
            self.mumble_file = File::open(format!("/dev/shm/{}", &self.key))
                .map_err(|e| {
                    error!("{:?}", &e);
                    e
                })
                .ok();
        }
        if let Some(ref mut mfile) = self.mumble_file {
            let mut buffer = [0u8; USEFUL_C_MUMBLE_LINK_SIZE + 16];
            mfile.read(&mut buffer).unwrap();
            mfile.seek(SeekFrom::Start(0)).unwrap();
            self.link.update_from_slice(&buffer).unwrap();

            let mut win_buffer = [0u8; 16];
            win_buffer.copy_from_slice(
                &buffer[USEFUL_C_MUMBLE_LINK_SIZE..USEFUL_C_MUMBLE_LINK_SIZE + 16],
            );
            self.window_dimensions = bytemuck::try_from_bytes::<WindowDimensions>(&win_buffer)
                .map_err(|e| {
                    error!("{:?}", &e);
                    e
                })
                .unwrap()
                .clone();
        }
    }
}
