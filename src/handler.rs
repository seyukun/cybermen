use futures::{Future, Stream};

struct VecCodec(std::net::SocketAddr);

impl tokio_core::net::UdpCodec for VecCodec {
    type In = Vec<u8>;
    type Out = Vec<u8>;

    fn decode(&mut self, _src: &std::net::SocketAddr, buf: &[u8]) -> std::io::Result<Self::In> {
        Ok(buf.to_owned())
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> std::net::SocketAddr {
        buf.extend(&msg);
        self.0
    }
}

pub fn handler(
    iface: tuntap::Iface,
    socket: std::net::UdpSocket,
    rem_address: std::net::SocketAddr,
) {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let core_socket = tokio_core::net::UdpSocket::from_socket(socket, &core.handle()).unwrap();
    let (socket_sink, socket_stream) = core_socket.framed(VecCodec(rem_address)).split();
    let (iface_sink, iface_stream) = tuntap::asynclib::Async::new(iface, &core.handle())
        .unwrap()
        .split();
    let sender = iface_stream.forward(socket_sink);
    let receiver = socket_stream.forward(iface_sink);
    core.run(sender.join(receiver)).unwrap();
}
