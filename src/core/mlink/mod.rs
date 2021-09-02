use jokolink::mlink::*;
use log::error;

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    time::Instant,
};

/// This is used to update
#[derive(Debug)]
pub struct MumbleManager {
    pub key: String,
    pub link: MumbleLink,
    pub window_dimensions: WindowDimensions,
    #[cfg(target_os = "linux")]
    pub mumble_file: Option<File>,
    #[cfg(target_os = "windows")]
    pub cmlptr: Option<*const CMumbleLink>,
    pub last_update: Instant,
}

impl MumbleManager {
    pub fn new(key: &str) -> anyhow::Result<MumbleManager> {
        #[cfg(target_os = "linux")]
        let mumble_file = File::open(format!("/dev/shm/{}", key))
            .map_err(|e| {
                error!("MumbleFile open error: {:?}", &e);
                e
            })
            .ok();
        #[cfg(target_os = "windows")]
        let cmlptr = jokolink::win::create_link_shared_mem(key)
            .map_err(|e| {
                error!("MumbleLink pointer Creation failed. {:?}", &e);
                e
            })
            .ok();
        let link = MumbleLink::default();
        let window_dimensions = WindowDimensions::default();
        let manager = MumbleManager {
            key: key.to_string(),
            link,
            window_dimensions,
            #[cfg(target_os = "linux")]
            mumble_file,
            last_update: Instant::now(),
            #[cfg(target_os = "windows")]
            cmlptr,
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
        #[cfg(target_os = "linux")]
        {
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
                if self.last_update.elapsed() > std::time::Duration::from_secs(5) {
                    let mut win_buffer = [0u8; 16];
                    win_buffer.copy_from_slice(
                        &buffer[USEFUL_C_MUMBLE_LINK_SIZE..USEFUL_C_MUMBLE_LINK_SIZE + 16],
                    );
                    self.window_dimensions =
                        bytemuck::try_from_bytes::<WindowDimensions>(&win_buffer)
                            .map_err(|e| {
                                error!("{:?}", &e);
                                e
                            })
                            .unwrap()
                            .clone();
                }
            }
        }
        #[cfg(target_os = "windows")]
        {
            if let Some(link_ptr) = self.cmlptr {
                self.link.update(link_ptr);
                if self.last_update.elapsed() > std::time::Duration::from_secs(5) {
                    self.last_update = Instant::now();
                    self.window_dimensions = jokolink::win::get_win_pos_dim(link_ptr).map_err(|e| {
                        error!("could not get window dimensions of gw2 based on the mumblelink pid. error: {:?}", &e);
                        e
                    }).unwrap();
                }
            }
        }
    }
}
