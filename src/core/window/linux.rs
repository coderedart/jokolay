use std::convert::TryInto;

use jokolink::{MumbleSource, WindowDimensions};
use log::error;
use x11rb::{
    protocol::xproto::{change_property, get_property, intern_atom, AtomEnum, PropMode},
    rust_connection::RustConnection,
};


pub struct LinuxPlatformData {
    pub xc: RustConnection,
    pub ow_window_handle: u32,
    pub gw2_window_handle: u32,
    pub gw2_pid: u32,
}
impl LinuxPlatformData {
    pub fn new(window: &glfw::Window, mumble_src: &mut MumbleSource) -> LinuxPlatformData {
        let ow_window_handle: u32;
        let gw2_window_handle: u32;
        let gw2_pid: u32;
        let xc;
        // set transient for
        use raw_window_handle::HasRawWindowHandle;
        let jokolay_xid = window.raw_window_handle();
        match jokolay_xid {
            raw_window_handle::RawWindowHandle::Xlib(xid) => {
                let (conn, _screen_num) = x11rb::connect(None)
                    .map_err(|e| {
                        log::error!("couldn't connect to xserver. error: {:#?}", &e);
                        e
                    })
                    .unwrap();
                xc = conn;
                ow_window_handle = xid
                    .window
                    .try_into()
                    .map_err(|e| {
                        log::error!("could not fit ow_window_handle into u32");
                        e
                    })
                    .unwrap();
                gw2_window_handle = mumble_src
                    .get_gw2_window_handle()
                    .map_err(|e| {
                        log::error!("could not get gw2 xid. error: {:?}", &e);
                        e
                    })
                    .unwrap()
                    .try_into()
                    .map_err(|e| {
                        log::error!("could not fit gw_window_handle into u32");
                        e
                    })
                    .unwrap();
                gw2_pid = Self::get_gw2_xpid(&xc, gw2_window_handle);

                // setting transient for
                Self::set_transient_for(&xc, ow_window_handle, gw2_window_handle as u32);

                LinuxPlatformData {
                    xc,
                    ow_window_handle,
                    gw2_window_handle,
                    gw2_pid,
                }
            }
            _ => todo!(),
        }
    }

    fn set_transient_for(xc: &RustConnection, ow_window_handle: u32, gw2_window_handle: u32) {
        let transient_for_atom = intern_atom(xc, true, b"WM_TRANSIENT_FOR")
            .map_err(|e| {
                log::error!("coudn't intern WM_TRANSIENT_FOR due to error: {:#?}", &e);
                e
            })
            .unwrap()
            .reply()
            .map_err(|e| {
                log::error!(
                    "coudn't parse reply for intern WM_TRANSIENT_FOR due to error: {:#?}",
                    &e
                );
                e
            })
            .unwrap()
            .atom;
        let _ = change_property(
            xc,
            PropMode::REPLACE,
            ow_window_handle,
            transient_for_atom,
            AtomEnum::WINDOW,
            32,
            1,
            &(gw2_window_handle as u32).to_ne_bytes(),
        )
        .map_err(|e| {
            log::error!(
                "coudn't send WM_TRANSIENT_FOR change property request due to error: {:#?}",
                &e
            );
            e
        })
        .unwrap()
        .check()
        .map_err(|e| {
            log::error!("reply for changing transient_for has error: {:#?}", &e);
            e
        })
        .unwrap();
    }

    pub fn get_gw2_windim(&self) -> WindowDimensions {
        let geometry = x11rb::protocol::xproto::get_geometry(&self.xc, self.gw2_window_handle)
            .map_err(|e| {
                log::error!("coudn't get geometry of gw2 due to error: {:#?}", &e);
                e
            })
            .unwrap()
            .reply()
            .map_err(|e| {
                log::error!("reply for getting geometry has error: {:#?}", &e);
                e
            })
            .unwrap();
        let translated_coordinates = x11rb::protocol::xproto::translate_coordinates(
            &self.xc,
            self.gw2_window_handle,
            geometry.root,
            geometry.x,
            geometry.y,
        )
        .map_err(|e| {
            log::error!(
                "coudn't get translation coords of gw2 due to error: {:#?}",
                &e
            );
            e
        })
        .unwrap()
        .reply()
        .map_err(|e| {
            log::error!("reply for getting translation coords has error: {:#?}", &e);
            e
        })
        .unwrap();
        let x_outer = translated_coordinates.dst_x as i32;
        let y_outer = translated_coordinates.dst_y as i32;
        let width = geometry.width;
        let height = geometry.height;

        log::debug!(
            "translated_x: {}, translated_y: {}, width: {}, height: {}, geo_x: {}, geo_y: {}",
            x_outer,
            y_outer,
            width,
            height,
            geometry.x,
            geometry.y
        );
        WindowDimensions {
            x: x_outer,
            y: y_outer,
            width: width as i32,
            height: height as i32,
        }
    }
    pub fn get_gw2_frame_extents(&self) -> (u32, u32, u32, u32) {
        let net_frame_extents_atom = intern_atom(&self.xc, true, b"_NET_FRAME_EXTENTS")
            .map_err(|e| {
                log::error!(
                    "coudn't intern atom for _NET_FRAME_EXTENTS  due to error: {:#?}",
                    &e
                );
                e
            })
            .unwrap()
            .reply()
            .map_err(|e| {
                log::error!(
                    "reply for intern atom for _NET_FRAME_EXTENTS has error: {:#?}",
                    &e
                );
                e
            })
            .unwrap()
            .atom;
        let frame_prop = get_property(
            &self.xc,
            false,
            self.gw2_window_handle,
            net_frame_extents_atom,
            AtomEnum::ANY,
            0,
            100,
        )
        .map_err(|e| {
            log::error!("coudn't get frame property gw2 due to error: {:#?}", &e);
            e
        })
        .unwrap()
        .reply()
        .map_err(|e| {
            log::error!("reply for frame property gw2 has error: {:#?}", &e);
            e
        })
        .unwrap();

        if frame_prop.bytes_after != 0 {
            log::error!(
                "bytes after in frame property is {}",
                frame_prop.bytes_after
            );
            panic!()
        }
        if frame_prop.format != 32 {
            log::error!("frame_prop format is {}", frame_prop.format);
            panic!()
        }
        if frame_prop.value_len != 4 {
            log::error!("frame_prop value_len is {}", frame_prop.value_len);
            panic!()
        }
        if frame_prop.value.len() != 16 {
            log::error!("frame_prop.value.len() is {}", frame_prop.value.len());
            panic!()
        }
        let mut buffer = [0u32; 4];
        buffer.copy_from_slice(bytemuck::cast_slice(&frame_prop.value));
        let left_border = buffer[0];
        let right_border = buffer[1];
        let top_border = buffer[2];
        let bottom_border = buffer[3];
        (left_border, right_border, top_border, bottom_border)
    }
    pub fn get_gw2_xpid(xc: &RustConnection, gw2_window_handle: u32) -> u32 {
        let pid_atom = intern_atom(xc, true, b"_NET_WM_PID")
            .map_err(|e| {
                log::error!(
                    "coudn't intern atom for _NET_WM_PID  due to error: {:#?}",
                    &e
                );
                e
            })
            .unwrap()
            .reply()
            .map_err(|e| {
                log::error!("reply for intern atom for _NET_WM_PID has error: {:#?}", &e);
                e
            })
            .unwrap();

        let pid_prop = get_property(
            xc,
            false,
            gw2_window_handle,
            pid_atom.atom,
            AtomEnum::ANY,
            0,
            100,
        )
        .map_err(|e| {
            log::error!(
                "coudn't get _NET_WM_PID property gw2 due to error: {:#?}",
                &e
            );
            e
        })
        .unwrap()
        .reply()
        .map_err(|e| {
            log::error!("reply for _NET_WM_PID property gw2 has error: {:#?}", &e);
            e
        })
        .unwrap();

        if pid_prop.bytes_after != 0 {
            log::error!(
                "bytes after in _NET_WM_PID property is {}",
                pid_prop.bytes_after
            );
            panic!()
        }
        if pid_prop.format != 32 {
            log::error!("_NET_WM_PID format is {}", pid_prop.format);
            panic!()
        }
        if pid_prop.value_len != 1 {
            log::error!("_NET_WM_PID value_len is {}", pid_prop.value_len);
            panic!()
        }
        if pid_prop.value.len() != 4 {
            log::error!("_NET_WM_PID.value.len() is {}", pid_prop.value.len());
            panic!()
        }
        let mut buffer = [0u8; 4];
        buffer.copy_from_slice(&pid_prop.value);
        u32::from_ne_bytes(buffer)
    }
    pub fn is_gw2_alive(&self) -> bool {
        true
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
        let handle = self.gw2_window_handle as u32;
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
// impl OverlayWindow {
//     pub fn is_gw2_alive(&self) -> bool {
//         self.platform_data.is_gw2_alive()
//     }
//     pub fn get_gw2_windim(&self) -> WindowDimensions {
//         self.platform_data.get_gw2_windim()
//     }
// }
