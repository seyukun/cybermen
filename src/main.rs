use std::process::Command;
use std::sync::Arc;
use std::{thread, time};
use tokio::net::UdpSocket;
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
    // Create sockets
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:30000").await.unwrap());

    // Create a „local“ (kernel) endpoint.
    let iface = Arc::new(Iface::new("cyber0", Mode::Tap).unwrap());
    #[rustfmt::skip] cmd("ip", &["addr", "add", "dev", iface.name(), "172.16.42.1/24"]);
    #[rustfmt::skip] cmd("ip", &["link", "set", "up", "dev", iface.name()]);

    let rsocket = socket.clone();
    let ssocket = socket.clone();
    let riface = iface.clone();
    let siface = iface.clone();

    // Handling for receive packet
    let _ = thread::spawn(move || 
        {async move {
        loop {
            let mut buf = [0; 1518];
            let (len, _) = rsocket.recv_from(&mut buf).await.unwrap();
            if len == 0 {
                continue;
            }
            if len > 0 {
                siface.send(&buf[..len]).unwrap();
            }
        }
    }});

    // Handling for send packet
    let th_send = thread::spawn(move || loop {
        let mut buf = [0; 1518];
        let len = match riface.recv(&mut buf) {
            Ok(len) => len,
            Err(_) => continue,
        };
        let _ = ssocket.send_to(&buf[..len], "172.16.42.2");
    });

    // Keep alive until the thread is terminated by an error
    loop {
        if th_send.is_finished() {
            break;
        }
        thread::sleep(time::Duration::from_millis(100));
    }
}
