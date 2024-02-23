use std::net::{SocketAddr, UdpSocket};

pub fn create(loc_address: SocketAddr) -> UdpSocket {
    match UdpSocket::bind(&loc_address) {
        Ok(s) => s,
        Err(e) => panic!("[ FAILED ] Cannot create socket: {}", e),
    }
}
