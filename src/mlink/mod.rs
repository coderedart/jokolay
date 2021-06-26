use jokolink::{
    cross_platform::{
        cmltypes::{CMumbleLink, USEFUL_C_MUMBLE_LINK_SIZE},
        MLRequestCode,
    },
    mlp::{
        mumble_link_response::MumbleStatus, MumbleLink,
        WindowDimensions,
    },
};
use std::net::UdpSocket;

pub enum GetMLMode {
    UdpAsync,
    UdpSync,
    GrpcAsync,
    #[cfg(target_os = "windows")]
    RawPtrWin,
}

pub async fn get_ml_grpc(_key: &str) {}

pub fn get_ml_udp(name: &str, socket: &UdpSocket) -> anyhow::Result<MumbleLink> {
    if name.len() > 60 {
        panic!("name length more than 60");
    }
    
    let mut sending_buffer = [0 as u8; 64];
    sending_buffer[0] = MLRequestCode::CML as i32 as u8;
    sending_buffer[1] = name.len() as u8;

    sending_buffer[2..name.len() + 2].copy_from_slice(name.as_bytes());
    socket
        .send(&sending_buffer)
        .expect("failed to send message");
    let mut recv_buffer = [0 as u8; USEFUL_C_MUMBLE_LINK_SIZE + 4];
    let received_bytes_count = socket
        .recv(&mut recv_buffer)
        .expect("failed to receive message");
    let response = MumbleStatus::from_i32(recv_buffer[0] as i32);

    match response {
        Some(MumbleStatus::Success) => {
            let mut ml = MumbleLink::default();
            ml.update(recv_buffer[4..].as_ptr() as *const CMumbleLink)?;
            Ok(ml)
        }
        _ => anyhow::bail!("response is not success. count = {}. buffer = {:?}", received_bytes_count, &recv_buffer[..received_bytes_count]),
    }
}

pub fn get_win_dim_udp(name: &str) -> Option<WindowDimensions> {
    if name.len() > 60 {
        panic!("name length more than 60");
    }
    let socket = UdpSocket::bind("127.0.0.1:0").expect("failed to bind to socket");
    socket
        .connect("127.0.0.1:7187")
        .expect("failed to connect to socket");
    let mut sending_buffer = [0 as u8; 64];
    sending_buffer[0] = MLRequestCode::WD as u8;
    sending_buffer[1] = name.len() as u8;

    sending_buffer[2..name.len() + 2].copy_from_slice(name.as_ref());
    socket
        .send(&sending_buffer)
        .expect("failed to send message");
    let mut recv_buffer = [0 as u8; std::mem::size_of::<WindowDimensions>() + 1];
    socket
        .recv(&mut recv_buffer)
        .expect("failed to receive message");
    let response = MumbleStatus::from_i32(recv_buffer[0] as i32);

    match response {
        Some(MumbleStatus::Success) => {
            use std::ptr::read_volatile;
            unsafe {
                Some(read_volatile(
                    recv_buffer[1..].as_ptr() as *const WindowDimensions
                ))
            }
        }
        _ => None,
    }
}
