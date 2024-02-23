use std::net::{SocketAddr, UdpSocket};

pub fn server_handshake(socket: &UdpSocket, rem_if_address: &str) -> SocketAddr {
    let rem_address: SocketAddr;
    match socket.recv_from(&mut [0; 1]) {
        Ok((_, addr)) => {
            rem_address = addr;
            println!("[ OK ] SYN from {}", rem_address);
        }
        Err(e) => panic!("[ FAILED ] Connection unestablished: {}", e),
    };
    match socket.send_to(rem_if_address.as_bytes(), &rem_address) {
        Ok(_) => println!("[ OK ] ACK to {}", rem_address.to_string()),
        Err(e) => panic!("[ FAILED ] ACK Error: {}", e),
    }
    rem_address
}
