use jokolink::engine::{
    mumble_link::{MumbleLink, MUMBLE_LINK_SIZE},
    RequestFor, Response,
};
use num_traits::cast::FromPrimitive;
use std::net::UdpSocket;
pub fn get_ml(name: &str) -> Option<MumbleLink> {
    if name.len() > 60 {
        panic!("name length more than 60");
    }
    let socket = UdpSocket::bind("127.0.0.1:0").expect("failed to bind to socket");
    socket
        .connect("127.0.0.1:7187")
        .expect("failed to connect to socket");
    let mut sending_buffer = [0 as u8; 64];
    sending_buffer[0] = RequestFor::MumbleLinkData as u8;
    sending_buffer[1] = name.len() as u8;

    sending_buffer[2..name.len() + 2].copy_from_slice(name.as_ref());
    socket.send(&sending_buffer).expect("failed to send message");
    let mut recv_buffer = [0 as u8; MUMBLE_LINK_SIZE + 1];
    socket.recv(&mut recv_buffer).expect("failed to receive message");
    let response = Response::from_u8(recv_buffer[0]);
    
    match response {
        Some(Response::Success) => {
            use std::ptr::read_volatile;
            unsafe { Some(read_volatile(recv_buffer[1..].as_ptr() as *const MumbleLink)) }
        },
        _ =>  None
    
    }
}
