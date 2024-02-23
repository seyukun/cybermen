use std::io::Result;
use std::net::SocketAddr;
use std::process::Command;
use std::str;

use futures::{Future, Stream};
use tokio_core::net::UdpCodec;
use tokio_core::reactor::Core;

use tuntap::asynclib::Async;
use tuntap::{Iface, Mode};

fn cmd(cmd: &str, args: &[&str]) {
    let mut child = Command::new(cmd).args(args).spawn().unwrap();
    let ecode = child.wait().unwrap();
    assert!(ecode.success(), "Failed to execte {}", cmd);
}

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
    // Read Local & Remote IP from args
    // let loc_address = "0.0.0.0:0"
    //     .parse()
    //     .unwrap_or_else(|e| panic!("[ FAILED ] LOCAL ADDRESS is broken: {e}"));
    // let rem_address = env::args()
    //     .nth(2)
    //     .unwrap()
    //     .parse()
    //     .unwrap_or_else(|e| panic!("[ FAILED ] REMOTE ADDRESS is broken: {e}"));
    let loc_address: SocketAddr = "0.0.0.0:0".parse().unwrap();
    let rem_address: SocketAddr = "163.43.185.240:3000".parse().unwrap();
    let loc_if_address: &str;
    let if_name = "cm0";
    let mut ip_buf = [0; 18];

    // Create socket
    let std_socket = std::net::UdpSocket::bind(&loc_address).unwrap();

    // Create interface
    let iface = Iface::new(&if_name, Mode::Tap)
        .unwrap_or_else(|err| panic!("[ FAILED ] Cannot create interface: {}", err));

    // Connection
    match std_socket.connect(&rem_address) {
        Ok(_) => println!(
            "[ OK ] Connection established with {}",
            rem_address.to_string()
        ),
        Err(e) => panic!("[ FAILED ] Connection unestablished: {}", e),
    };
    match std_socket.send(&[0; 1]) {
        Ok(_) => println!("[ OK ] SYN"),
        Err(e) => panic!("[ FAILED ] SYN Error: {}", e),
    };
    match std_socket.recv(&mut ip_buf) {
        Ok(size) => match str::from_utf8(&mut ip_buf[..size]) {
            Ok(s) => {
                loc_if_address = s;
                println!("[ OK ] ACK");
            }
            Err(e) => panic!("[ FAILED ] ACK packet is broken: {e}"),
        },
        Err(e) => panic!("[ FAILED ] ACK Error: {}", e),
    };

    // Configure the „local“ (kernel) endpoint.
    cmd("ip", &["addr", "add", "dev", iface.name(), &loc_if_address]);
    cmd("ip", &["link", "set", "up", "dev", iface.name()]);

    // Packet handling
    let mut core = Core::new().unwrap();
    let core_socket = tokio_core::net::UdpSocket::from_socket(std_socket, &core.handle()).unwrap();
    let (socket_sink, socket_stream) = core_socket.framed(VecCodec(rem_address)).split();
    let (iface_sink, iface_stream) = Async::new(iface, &core.handle()).unwrap().split();
    let sender = iface_stream.forward(socket_sink);
    let receiver = socket_stream.forward(iface_sink);
    core.run(sender.join(receiver)).unwrap();
}
