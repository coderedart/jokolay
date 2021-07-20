use anyhow::Context;
use jokolink::{
    cross_platform::{
        cmltypes::{CMumbleLink, USEFUL_C_MUMBLE_LINK_SIZE},
        MLRequestCode,
    },
    mlp::{mumble_link_response::MumbleStatus, MumbleLink, WindowDimensions},
};
use parking_lot::Mutex;
use tokio::time::Instant;

use std::sync::Arc;

/// This is used to update
#[derive(Debug)]
pub struct MumbleManager {
    pub key: String,
    pub link: Option<MumbleLink>,
    pub window_dimensions: Option<WindowDimensions>,
    shared_link: Arc<Mutex<Option<MumbleLink>>>,
    shared_window_dimensions: Arc<Mutex<Option<WindowDimensions>>>,
}

impl MumbleManager {
    pub fn new(key: &str, addr: &str, handle: tokio::runtime::Handle) -> MumbleManager {
        let shared_link = Arc::new(Mutex::new(None));
        let shared_window_dimensions = Arc::new(Mutex::new(None));
        let sl = shared_link.clone();
        let swd = shared_window_dimensions.clone();
        // let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let akey = key.to_string();
        let socket_addr = addr.to_string();
        handle.spawn(async move {
            let ml_request_buffer = encode_request(&akey, MLRequestCode::CML);
            let wd_request_buffer = encode_request(&akey, MLRequestCode::WD);
            let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
            socket.connect(&socket_addr).await.unwrap();
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(15));
            let mut window_updated_time = tokio::time::Instant::now();
            loop {
                if Arc::strong_count(&sl) < 2 {
                    break;
                }
                socket
                    .send(&ml_request_buffer)
                    .await
                    .expect("failed to send message");

                let mut response_buffer = [0_u8; USEFUL_C_MUMBLE_LINK_SIZE + 4];
                socket.recv(&mut response_buffer).await.unwrap();
                let ml = decode_response(&response_buffer, MLRequestCode::CML).ok();
                if let Some(response) = ml {
                    match response {
                        MumbleResponse::Link(link) => {
                            let mut guard = sl.lock();
                            *guard = Some(link);
                        }
                        MumbleResponse::WinDim(_) => todo!(),
                    }
                }
                if window_updated_time.elapsed() > std::time::Duration::from_secs(3) {
                    window_updated_time = Instant::now();
                    socket
                        .send(&wd_request_buffer)
                        .await
                        .expect("failed to send message");

                    let mut response_buffer = [0_u8; 20];
                    socket.recv(&mut response_buffer).await.unwrap();
                    let ml = decode_response(&response_buffer, MLRequestCode::WD).ok();
                    if let Some(response) = ml {
                        match response {
                            MumbleResponse::Link(_) => todo!(),
                            MumbleResponse::WinDim(wd) => {
                                let mut guard = swd.lock();
                                *guard = Some(wd);
                            }
                        }
                    }
                }
                interval.tick().await;
            }
        });
        let manager = MumbleManager {
            key: key.to_string(),
            link: None,
            window_dimensions: None,
            shared_link,
            shared_window_dimensions,
        };
        manager
    }

    pub fn get_window_dimensions(&self) -> Option<WindowDimensions> {
        self.window_dimensions
    }
    pub fn get_link(&self) -> Option<MumbleLink> {
        self.link.clone()
    }
    pub fn update(&mut self) {
        if let Some(link) = self.shared_link.try_lock() {
            if let Some(ref ml) = *link {
                self.link = Some(ml.clone());
            }
        }
        if let Some(dimensions) = self.shared_window_dimensions.try_lock() {
            if let Some(wd) = *dimensions {
                self.window_dimensions = Some(wd);
            }
        }
    }

    // // async fn grpc_async(key: &str, request_type: MLRequestCode ) {}
}

// fn request_udp_sync(
//     key: &str,
//     socket: &UdpSocket,
//     request_type: MLRequestCode,
// ) -> anyhow::Result<MumbleResponse> {
//     if key.len() > 60 {
//         panic!("name length more than 60");
//     }

//     let sending_buffer = encode_request(key, request_type);
//     socket
//         .send(&sending_buffer)
//         .expect("failed to send message");

//     let mut response_buffer = [0_u8; USEFUL_C_MUMBLE_LINK_SIZE + 4];
//     socket.recv(&mut response_buffer)?;
//     let ml = decode_response(&response_buffer, request_type)?;
//     Ok(ml)
// }
fn encode_request(key: &str, request_type: MLRequestCode) -> [u8; 64] {
    let mut request_buffer = [0 as u8; 64];
    request_buffer[0] = request_type as u8;
    request_buffer[1] = key.len() as u8;
    //leave two bytes for padding and future use
    request_buffer[4..key.len() + 4].copy_from_slice(key.as_bytes());
    request_buffer
}
#[allow(dead_code)]
fn decode_request(request_buffer: &[u8; 64]) -> anyhow::Result<(MLRequestCode, &str)> {
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
                    wd = read_volatile(response_buffer[4..].as_ptr() as *const WindowDimensions);
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
