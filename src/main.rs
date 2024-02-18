use std::process::Command;
use std::str;
use std::sync::Arc;
use std::{thread, time};
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
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
    // Interface name
    let iface: Arc<Iface>;
    let if_addr: &str;
    let if_name: &str = "cm0";
    let loc_addr: &str = "0.0.0.0:0";
    let rem_addr: &str = "163.43.185.240:3000";
    let socket: Arc<UdpSocket> = Arc::new(UdpSocket::bind(loc_addr).await.unwrap());
    let mut buf_if_addr = [0; 18];

    // Connect
    socket
        .connect(rem_addr)
        .await
        .expect("[ FAILED ] Connection unestablished");
    println!("[ OK ] Connection established: {}", rem_addr);

    // Send request
    socket
        .send(&[0; 1])
        .await
        .expect("[ FAILED ] SYN cannot send");
    println!("[ OK ] SYN");

    // Get interface address
    match socket.recv(&mut buf_if_addr).await {
        Ok(len) => {
            if_addr =
                str::from_utf8(&buf_if_addr[..len]).expect("[ FAILED ] ACK packet is invalid");
        }
        Err(e) => panic!("[ FAILED ] ACK dose not recived: {}", e),
    }
    println!("[ OK ] ACK recived: {}", if_addr);

    // Create a „local“ (kernel) endpoint.
    iface = Arc::new(Iface::new(if_name, Mode::Tun).unwrap());
    cmd("ip", &["addr", "add", "dev", iface.name(), if_addr]);
    cmd("ip", &["link", "set", "up", "dev", iface.name()]);

    let rsocket = socket.clone();
    let ssocket = socket.clone();
    let riface = iface.clone();
    let siface = iface.clone();

    // Handling for receive packet
    let runtime = Runtime::new().expect("[ FAILED ] Unable to create a runtime");
    let _ = thread::spawn(move || loop {
        let mut buf = [0; 1518];
        match runtime.block_on(rsocket.recv(&mut buf)) {
            Ok(len) => {
                if len > 0 {
                    siface.send(&buf[..len]).unwrap();
                }
            }
            Err(e) => {
                println!("[ FAILED ] Unable to block socket: {}", e);
            }
        };
    });

    // Handling for send packet
    let runtime = Runtime::new().expect("[ FAILED ] Unable to create a runtime");
    let th_send = thread::spawn(move || loop {
        let mut buf = [0; 1518];
        match riface.recv(&mut buf) {
            Ok(len) => {
                runtime
                    .block_on(ssocket.send(&buf[..len]))
                    .expect("[ FAILED ] Unable to block socket");
            }
            Err(_) => continue,
        };
    });

    // Keep alive until the thread is terminated by an error
    loop {
        if th_send.is_finished() {
            break;
        }
        thread::sleep(time::Duration::from_millis(100));
    }
}
