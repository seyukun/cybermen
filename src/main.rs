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
    // Listen address
    let loc_addr = "0.0.0.0:0";

    // Remote address
    let rem_address = "163.43.185.240:3000";

    // Create sockets
    let socket = Arc::new(UdpSocket::bind(loc_addr).await.unwrap());

    // Connect
    match socket.connect(rem_address).await {
        Ok(_) => {
            println!("[ OK ] Connection established: {}", rem_address);
        }
        Err(e) => panic!("[ FAILED ] Connection unestablished: {}", e),
    };

    // Send request
    match socket.send(&[0; 1]).await {
        Ok(_) => {
            println!("[ OK ] SYN");
        }
        Err(e) => {
            panic!("[ FAILED ] SYN cannot send: {}", e);
        }
    };

    // Get interface address
    let mut if_addr = [0; 18];
    let if_addr_len = match socket.recv(&mut if_addr).await {
        Ok(f) => {
            println!("[ OK ] ACK recived: {}", str::from_utf8(&if_addr).unwrap());
            f
        }
        Err(e) => {
            panic!("[ FAILED ] ACK dose not recived: {}", e);
        }
    };
    let if_addr = str::from_utf8(&if_addr[..if_addr_len]).unwrap();

    // Create a „local“ (kernel) endpoint.
    let iface = Arc::new(Iface::new("cm0", Mode::Tap).unwrap());
    #[rustfmt::skip] cmd("ip", &["addr", "add", "dev", iface.name(), if_addr]);
    #[rustfmt::skip] cmd("ip", &["link", "set", "up", "dev", iface.name()]);

    let rsocket = socket.clone();
    let ssocket = socket.clone();
    let riface = iface.clone();
    let siface = iface.clone();

    // Handling for receive packet
    let _ = thread::spawn(move || loop {
        let mut buf = [0; 1518];
        let runtime = Runtime::new().expect("Unable to create a runtime");
        let len = runtime.block_on(rsocket.recv(&mut buf)).unwrap();
        println!("[ OK ] Recived: {}", len);
        if len == 0 {
            continue;
        }
        if len > 0 {
            siface.send(&buf[..len]).unwrap();
        }
    });

    // Handling for send packet
    let th_send = thread::spawn(move || loop {
        let mut buf = [0; 1518];
        let len = match riface.recv(&mut buf) {
            Ok(len) => len,
            Err(_) => continue,
        };
        let runtime = Runtime::new().expect("Unable to create a runtime");
        runtime.block_on(ssocket.send(&buf[..len])).unwrap();
        println!("[ OK ] Send: {}", len);
    });

    // Keep alive until the thread is terminated by an error
    loop {
        if th_send.is_finished() {
            break;
        }
        thread::sleep(time::Duration::from_millis(100));
    }
}
