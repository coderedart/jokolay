use jokolink::mlink::*;
use log::error;
use x11rb::rust_connection::RustConnection;

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct MumbleSource {
    #[cfg(target_os = "linux")]
    pub mumble_src: File,
    #[cfg(target_os = "windows")]
    pub mumble_src: *const CMumbleLink,
}

#[cfg(target_os = "windows")]
impl MumbleSource {
    fn new(key: &str) -> MumbleSource {
        MumbleSource {
            mumble_src: jokolink::win::create_link_shared_mem(key)
                .map_err(|e| {
                    error!("MumbleLink pointer Creation failed. {:?}", &e);
                    e
                })
                .unwrap(),
        }
    }
}

#[cfg(target_os = "linux")]
impl MumbleSource {
    pub fn new(key: &str) -> MumbleSource {
        let mut counter = 0;
        let mut max_counter = 60;
        let mut src = loop {
            match File::open(format!("/dev/shm/{}", key)).map_err(|e| {
                error!(
                    "MumbleFile open error: {:?}. attempt number: {}. maximum attempts: {}",
                    &e, counter, max_counter
                );
                e
            }) {
                Ok(mumble_src) => break MumbleSource { mumble_src },
                Err(_) => {
                    if counter > max_counter {
                        panic!("cannot open mumble file after maximum attempts");
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1))
                }
            }
        };
   
        counter = 0;
        max_counter = 60;
        let tick = src.get_link().ui_tick;
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let present_tick = src.get_link().ui_tick;
            // if we are reading from a previous mumble file, our previous tick would be nonzero. and when jokolink opens the file, it will write it to zero.
            // so, if present_tick is zero, it means our previous tick was from the old file, so we will wait until the present tick is non-zero which means that
            // mumblelink is active and updating now.
            if tick != present_tick && present_tick != 0 {
                break src;
            } else {
                counter += 1;
                error!("Mumble link checking whether uitick is changing. previous tick = {}. present tick = {}. attempt number: {}. max attempts: {}", tick, present_tick, counter, max_counter);
                if counter > max_counter {
                    panic!("uitick is not incrementing, so gw2 is probably not open.");
                }
            }
        }

    }
    fn get_link_buffer(
        &mut self,
    ) -> [u8; USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<isize>()] {
        #[cfg(target_os = "linux")]
        {
            let mfile = &mut self.mumble_src;
            mfile.seek(SeekFrom::Start(0)).unwrap();
            let mut buffer = [0u8; USEFUL_C_MUMBLE_LINK_SIZE + std::mem::size_of::<isize>()];
            mfile.read(&mut buffer).unwrap();
            buffer
        }
    }
    pub fn get_link(&mut self) -> MumbleLink {
        let mut link = MumbleLink::default();
        let buffer = self.get_link_buffer();
        link.update_from_slice(&buffer);
        link
    }
    pub fn get_gw2_window_handle(&mut self) -> isize {
        let buffer = self.get_link_buffer();
        let mut xid_buffer = [0u8; std::mem::size_of::<isize>()];
        xid_buffer.copy_from_slice(&buffer[USEFUL_C_MUMBLE_LINK_SIZE..]);
        isize::from_ne_bytes(xid_buffer)
    }

    pub fn get_gw2_pid(&mut self, conn: &RustConnection) -> u32 {
        let pid_atom = x11rb::protocol::xproto::intern_atom(conn, true, b"_NET_WM_PID")
            .map_err(|e| {
                error!(
                    "could not intern atom '_NET_WM_PID' because of error: {:#?} ",
                    &e
                );
                e
            })
            .unwrap()
            .reply()
            .map_err(|e| {
                error!(
                    "reply error while interning '_NET_WM_PID'. error: {:#?}",
                    &e
                );
                e
            })
            .unwrap()
            .atom;
        let handle = self.get_gw2_window_handle() as u32;
        let reply = x11rb::protocol::xproto::get_property(
            conn,
            false,
            handle,
            pid_atom,
            x11rb::protocol::xproto::AtomEnum::CARDINAL,
            0,
            1,
        )
        .map_err(|e| {
            error!(
                "could not request '_NET_WM_PID' for gw2 window handle due to error: {:#?}",
                &e
            );
            e
        })
        .unwrap()
        .reply()
        .map_err(|e| {
            error!(
                "the reply for '_NET_WM_PID' of gw2 handle had error: {:#?}",
                &e
            );
            e
        })
        .unwrap();

        let pid_format = 32;
        if pid_format != reply.format {
            error!("pid_format is not 32. so, type is wrong");
            panic!();
        }
        let pid_buffer_size = 4;
        if pid_buffer_size != reply.value.len() {
            error!("pid_buffer is not 4 bytes");
            panic!()
        }
        let value_len = 1;
        if value_len != reply.value_len {
            error!("pid reply's value_len is not 1");
            panic!()
        }
        let remaining_bytes_len = 0;
        if remaining_bytes_len != reply.bytes_after {
            error!("we still have too many bytes remaining after reading '_NET_WM_PID'");
            panic!()
        }
        let mut buffer = [0u8; 4];
        buffer.copy_from_slice(&reply.value);
        u32::from_ne_bytes(buffer)
    }
}

/// This is used to update
#[derive(Debug)]
pub struct MumbleManager {
    src: MumbleSource,
    pub link: MumbleLink,
    pub last_update: Instant,
}
#[derive(Debug)]
pub struct MumbleConfig {
    pub link_name: String,
}
impl MumbleConfig {
    pub const DEFAULT_MUMBLELINK_NAME: &'static str = "MumbleLink";
}
impl Default for MumbleConfig {
    fn default() -> Self {
        Self {
            link_name: Self::DEFAULT_MUMBLELINK_NAME.to_string(),
        }
    }
}

impl MumbleManager {
    pub fn new(mut src: MumbleSource) -> anyhow::Result<MumbleManager> {
        let link = src.get_link();
        if link.ui_tick == 0 {
            error!("mumble link manager started with an uninitialized link");
            panic!("invalid link");
        }
        let manager = MumbleManager {
            src,
            link,
            last_update: Instant::now(),
        };
        Ok(manager)
    }

    pub fn get_link(&self) -> &MumbleLink {
        &self.link
    }

    pub fn last_updated(&self) -> Instant {
        self.last_update
    }

    pub fn tick(&mut self) {
        let ui_tick = self.link.ui_tick;
        self.link = self.src.get_link();
        if ui_tick < self.link.ui_tick {
            self.last_update = Instant::now();
        }
    }
}
