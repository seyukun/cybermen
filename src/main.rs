use std::net::UdpSocket;
use std::process::Command;
use std::sync::Arc;
use std::{thread, time};
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
    let if_name = "cm0";
    let loc_addr = "163.43.185.240:3000";
    let locif_address = "172.16.42.1/24";
    let remif_address = "172.16.42.2/24";
    let socket = Arc::new(UdpSocket::bind(loc_addr).unwrap());

    // Get remote address
    let (_, rem_address) = match socket.recv_from(&mut [0; 1]) {
        Ok(f) => {
            println!("[ OK ] SYN recived: {}", f.1);
            f
        }
        Err(e) => panic!("[ FAILED ] SYN: {}", e),
    };

    // Send interface address to remote
    let buf = remif_address.as_bytes();
    match socket.send_to(buf, rem_address) {
        Ok(_) => {
            println!("[ OK ] ACK")
        }
        Err(e) => {
            panic!("[ FAILED ] ACK cannot send: {}", e);
        }
    };

    // Create a „local“ (kernel) endpoint.
    let iface = Arc::new(Iface::new(if_name, Mode::Tun).unwrap());
    cmd("ip", &["addr", "add", "dev", iface.name(), locif_address]);
    cmd("ip", &["link", "set", "up", "dev", iface.name()]);

    let rsocket = socket.clone();
    let ssocket = socket.clone();
    let riface = iface.clone();
    let siface = iface.clone();

    // Handling for receive packet
    let _ = tokio::spawn(async move {
        loop {
            let mut buf = [0; 1518];
            match rsocket.recv_from(&mut buf) {
                Ok(len) => {
                    if len.0 > 0 {
                        siface.send(&buf[..len.0]).unwrap();
                    }
                }
                Err(e) => {
                    println!("[ FAILED R ] Unable to block socket: {}", e);
                }
            }
        }
    });

    // Handling for send packet
    let th_send = tokio::spawn(async move {
        loop {
            let mut buf = [0; 1518];
            match riface.recv(&mut buf) {
                Ok(len) => {
                    if len > 0 {
                        ssocket
                            .send_to(&buf[..len], rem_address)
                            .expect("[ FAILED S ] Unable to block socket");
                    }
                }
                Err(_) => continue,
            };
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
