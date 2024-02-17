use std::process::Command;
use std::sync::Arc;
use std::{thread, time};
use tokio::net::UdpSocket;
// use tokio::runtime::Runtime;
use tuntap::{Iface, Mode};

fn cmd(cmd: &str, args: &[&str]) {
    let ecode = Command::new(cmd)
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(ecode.success(), "Failed to execte {}", cmd);
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Listen address
    let loc_addr = "163.43.185.240:3000";

    // Remote interface address
    let remif_address = "172.16.42.2/24";

    // Create sockets
    let socket = Arc::new(UdpSocket::bind(loc_addr).await.unwrap());

    // Get remote address
    let (_, rem_address) = match socket.recv_from(&mut [0; 1]).await {
        Ok(f) => {
            println!("[ OK ] SYN recived: {}", f.1);
            f
        }
        Err(e) => panic!("[ FAILED ] SYN: {}", e),
    };

    // Send interface address to remote
    let buf = remif_address.as_bytes();
    match socket.send_to(buf, rem_address).await {
        Ok(_) => {
            println!("[ OK ] ACK")
        }
        Err(e) => {
            panic!("[ FAILED ] ACK cannot send: {}", e);
        }
    };

    // Create a „local“ (kernel) endpoint.
    let iface = Arc::new(Iface::new("cm0", Mode::Tap).unwrap());
    #[rustfmt::skip] cmd("ip", &["addr", "add", "dev", iface.name(), "172.16.42.1/24"]);
    #[rustfmt::skip] cmd("ip", &["link", "set", "up", "dev", iface.name()]);

    let rsocket = socket.clone();
    let ssocket = socket.clone();
    let riface = iface.clone();
    let siface = iface.clone();

    // Handling for receive packet
    let _ = tokio::spawn(async move {
        loop {
            let mut buf = [0; 1518];
            let (len, _) = rsocket.recv_from(&mut buf).await.unwrap();
            println!("[ OK ] Recived: {}", len);
            if len == 0 {
                continue;
            }
            if len > 0 {
                siface.send(&buf[..len]).unwrap();
            }
        }
    });

    // Handling for send packet
    let th_send = tokio::spawn(async move {
        loop {
            let mut buf = [0; 1518];
            let len = match riface.recv(&mut buf) {
                Ok(len) => len,
                Err(_) => continue,
            };
            ssocket.send_to(&buf[..len], rem_address).await.unwrap();
            println!("[ OK ] Send: {}", len);
        }
    });

    // Keep alive until the thread is terminated by an error
    loop {
        if th_send.is_finished() {
            break;
        }
        thread::sleep(time::Duration::from_millis(100));
    }
}
