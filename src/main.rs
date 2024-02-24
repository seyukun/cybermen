use std::net::{SocketAddr, UdpSocket};

// use clap::{Parser, Subcommand, Arg};
use tuntap::Iface;

mod handler;
mod handshake;
mod if_tap;
mod std_sock;

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
    handler::handler(iface, socket, rem_address);
}
