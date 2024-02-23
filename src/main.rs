use std::io::Result;
use std::net::{SocketAddr, UdpSocket};

use futures::{Future, Stream};
use tokio_core::net::UdpCodec;
use tokio_core::reactor::Core;

// use clap::Parser;

use tuntap::asynclib::Async;
use tuntap::Iface;

mod handshake;
mod if_tap;
mod std_sock;

struct VecCodec(SocketAddr);

impl UdpCodec for VecCodec {
    type In = Vec<u8>;
    type Out = Vec<u8>;

    fn decode(&mut self, _src: &SocketAddr, buf: &[u8]) -> Result<Self::In> {
        Ok(buf.to_owned())
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        buf.extend(&msg);
        self.0
    }
}

fn main() {
    let loc_address: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    let rem_address: SocketAddr;
    let loc_if_address: &str = "172.16.42.1/24";
    let rem_if_address: &str = "172.16.42.2/24";
    let if_name: &str = "cm0";
    let socket: UdpSocket;
    let iface: Iface;

    // Create socket
    socket = std_sock::create(loc_address);

    // Create interface
    iface = if_tap::create(if_name);

    // Configure the „local“ (kernel) endpoint.
    if_tap::set_ip(iface.name(), loc_if_address);
    if_tap::set_linkup(iface.name());

    // Connection
    rem_address = handshake::server_handshake(&socket, rem_if_address);

    // Packet handling
    let mut core = Core::new().unwrap();
    let core_socket = tokio_core::net::UdpSocket::from_socket(socket, &core.handle()).unwrap();
    let (socket_sink, socket_stream) = core_socket.framed(VecCodec(rem_address)).split();
    let (iface_sink, iface_stream) = Async::new(iface, &core.handle()).unwrap().split();
    let sender = iface_stream.forward(socket_sink);
    let receiver = socket_stream.forward(iface_sink);
    core.run(sender.join(receiver)).unwrap();
}
