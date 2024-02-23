pub fn create(name: &str) -> tuntap::Iface {
    tuntap::Iface::new(name, tuntap::Mode::Tap)
        .unwrap_or_else(|err| panic!("[ FAILED ] Cannot create interface: {}", err))
}

pub fn set_ip(name: &str, ip: &str) -> std::process::ExitStatus {
    match cmd::cmd("ip", &["addr", "add", "dev", name, ip]) {
        Ok(status) => status,
        Err(e) => panic!("[ FAILED ] Cannot configure interface: {}", e),
    }
}

pub fn set_linkup(name: &str) -> std::process::ExitStatus {
    match cmd::cmd("ip", &["link", "set", "up", "dev", name]) {
        Ok(status) => status,
        Err(e) => panic!("[ FAILED ] Cannot configure interface: {}", e),
    }
}
