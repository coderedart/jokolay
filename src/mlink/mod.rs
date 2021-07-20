use anyhow::Context;
use jokolink::{
    cross_platform::{
        cmltypes::{CMumbleLink, USEFUL_C_MUMBLE_LINK_SIZE},
        MLRequestCode,
    },
    mlp::{mumble_link_response::MumbleStatus, MumbleLink, WindowDimensions},
};

use std::{
    net::UdpSocket,
    sync::mpsc::Receiver,
};

/// This is used to update
#[derive(Debug)]
pub struct MumbleManager {
    pub key: String,
    pub link: Option<MumbleLink>,
    pub window_dimensions: Option<WindowDimensions>,
    pub receiver: Receiver<MumbleResponse>,
}

impl MumbleManager {
    pub fn new(key: &str, receiver: Receiver<MumbleResponse>) -> MumbleManager {
        let mut manager = MumbleManager {
            key: key.to_string(),
            link: None,
            window_dimensions: None,
            receiver,
        };
        manager.try_update();
        manager
    }

    pub fn try_update(&mut self) {
        for response in self.receiver.try_iter() {
            match response {
                MumbleResponse::Link(link) => self.link = Some(link),
                MumbleResponse::WinDim(window_dimensions) => {
                    self.window_dimensions = Some(window_dimensions)
                }
            }
        }
    }
    pub fn get_window_dimensions(&self) -> (i32, i32, i32, i32) {
        if let Some(windim) = self.window_dimensions {
            (
                windim.x as i32,
                windim.y as i32,
                windim.width as i32,
                windim.height as i32,
            )
        } else {
            panic!("no windim");
        }
    }

    // // async fn grpc_async(key: &str, request_type: MLRequestCode ) {}
}

#[allow(dead_code)]
fn request_udp_sync(
    key: &str,
    socket: &UdpSocket,
    request_type: MLRequestCode,
) -> anyhow::Result<MumbleResponse> {
    if key.len() > 60 {
        panic!("name length more than 60");
    }

    let sending_buffer = encode_request(key, request_type);
    socket
        .send(&sending_buffer)
        .expect("failed to send message");

    let mut response_buffer = [0_u8; USEFUL_C_MUMBLE_LINK_SIZE + 4];
    socket.recv(&mut response_buffer)?;
    let ml = decode_response(&response_buffer, request_type)?;
    Ok(ml)
}
pub fn encode_request(key: &str, request_type: MLRequestCode) -> [u8; 64] {
    let mut request_buffer = [0 as u8; 64];
    request_buffer[0] = request_type as u8;
    request_buffer[1] = key.len() as u8;
    //leave two bytes for padding and future use
    request_buffer[4..key.len() + 4].copy_from_slice(key.as_bytes());
    request_buffer
}
pub fn decode_request(request_buffer: &[u8; 64]) -> anyhow::Result<(MLRequestCode, &str)> {
    use num_traits::FromPrimitive;
    let request_type = MLRequestCode::from_u8(request_buffer[0]);
    if request_type.is_none() {
        anyhow::anyhow!("wrong request type in buffer");
    }
    let request_type = request_type.unwrap();
    let size = request_buffer[1] + 4;
    let key = std::str::from_utf8(&request_buffer[4..size as usize])
        .context("could not get string from buffer")?;
    Ok((request_type, key))
}
#[allow(dead_code)]
fn decode_response(
    response_buffer: &[u8],
    response_type: MLRequestCode,
) -> anyhow::Result<MumbleResponse> {
    let response = MumbleStatus::from_i32(response_buffer[0] as i32);

    match response {
        Some(MumbleStatus::Success) => match response_type {
            MLRequestCode::CML => {
                let mut ml = MumbleLink::default();

                ml.update(response_buffer[4..].as_ptr() as *const CMumbleLink)?;
                Ok(MumbleResponse::Link(ml))
            }
            MLRequestCode::WD => {
                use std::ptr::read_volatile;

                let wd;
                unsafe {
                    wd = read_volatile(response_buffer[1..].as_ptr() as *const WindowDimensions);
                }
                Ok(MumbleResponse::WinDim(wd))
            }
            MLRequestCode::Nothing => anyhow::bail!(""),
        },
        _ => anyhow::bail!("response is not success. buffer = {:?}", &response_buffer),
    }
}
pub enum MumbleResponse {
    Link(MumbleLink),
    WinDim(WindowDimensions),
}
