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
    time::{Duration, Instant},
};

#[derive(Debug)]
pub enum GetMLMode {
    UdpAsync,
    UdpSync(std::net::UdpSocket),
    GrpcAsync,
    #[cfg(target_os = "windows")]
    RawPtrWin,
}
/// This is used to update
#[derive(Debug)]
pub struct MumbleCache {
    pub key: String,
    pub last_updated: Instant,
    pub last_accessed: Instant,
    pub link: Option<MumbleLink>,
    pub update_interval: Duration,
    pub auto_update: bool,
    pub get_mode: GetMLMode,
}

impl MumbleCache {
    pub fn new(key: &str, update_interval: Duration, get_mode: GetMLMode) -> anyhow::Result<Self> {
        let link = MumbleCache::get_ml(key, &get_mode).ok();
        Ok(MumbleCache {
            key: key.to_string(),
            last_updated: Instant::now(),
            last_accessed: Instant::now(),
            link,
            update_interval,
            auto_update: false,
            get_mode,
        })
    }
    pub fn update_link(&mut self) -> anyhow::Result<()> {
        if self.last_updated.elapsed() < self.update_interval {
            return Ok(());
        }
        self.link = MumbleCache::get_ml(&self.key, &self.get_mode).ok();
        self.last_updated = Instant::now();
        Ok(())
    }
    pub fn get_ml(key: &str, get_mode: &GetMLMode) -> anyhow::Result<MumbleLink> {
        let ml;
        match get_mode {
            GetMLMode::UdpAsync => anyhow::bail!(""),
            GetMLMode::UdpSync(socket) => {
                ml = MumbleCache::request_udp_sync(key, &socket, MLRequestCode::CML)?
            }
            GetMLMode::GrpcAsync => anyhow::bail!(""),
            #[cfg(target_os = "windows")]
            GetMLMode::RawPtrWin => anyhow::bail!(""),
        }
        match ml {
            ResponseResult::Link(link) => Ok(link),
            ResponseResult::WinDim(_) => anyhow::bail!(""),
        }
    }
    // // async fn grpc_async(key: &str, request_type: MLRequestCode ) {}

    fn request_udp_sync(
        key: &str,
        socket: &UdpSocket,
        request_type: MLRequestCode,
    ) -> anyhow::Result<ResponseResult> {
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

fn decode_response(
    response_buffer: &[u8],
    response_type: MLRequestCode,
) -> anyhow::Result<ResponseResult> {
    let response = MumbleStatus::from_i32(response_buffer[0] as i32);

    match response {
        Some(MumbleStatus::Success) => match response_type {
            MLRequestCode::CML => {
                let mut ml = MumbleLink::default();

                ml.update(response_buffer[4..].as_ptr() as *const CMumbleLink)?;
                Ok(ResponseResult::Link(ml))
            }
            MLRequestCode::WD => {
                use std::ptr::read_volatile;

                let wd;
                unsafe {
                    wd = read_volatile(response_buffer[1..].as_ptr() as *const WindowDimensions);
                }
                Ok(ResponseResult::WinDim(wd))
            }
            MLRequestCode::Nothing => anyhow::bail!(""),
        },
        _ => anyhow::bail!("response is not success. buffer = {:?}", &response_buffer),
    }
}
enum ResponseResult {
    Link(MumbleLink),
    WinDim(WindowDimensions),
}
