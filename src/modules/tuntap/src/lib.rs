use std::ffi::CStr;
use std::fs::{File, OpenOptions};
use std::io::{Error, Read, Result, Write};
use std::os::raw::{c_char, c_int};
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};

#[cfg(feature = "tokio")]
pub mod asynclib;

extern "C" {
    fn tuntap_setup(fd: c_int, name: *mut u8, mode: c_int, packet_info: c_int) -> c_int;
}

/// The mode in which open the virtual network adapter.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Mode {
    Tun = 1,
    Tap = 2,
}

#[derive(Debug)]
pub struct Iface {
    fd: File,
    mode: Mode,
    name: String,
}

impl Iface {
    /// Creates a new virtual interface.
    pub fn new(ifname: &str, mode: Mode) -> Result<Self> {
        Iface::with_options(ifname, mode, true)
    }

    /// Creates a new virtual interface without the prepended packet info.
    pub fn without_packet_info(ifname: &str, mode: Mode) -> Result<Self> {
        Iface::with_options(ifname, mode, false)
    }

    #[rustfmt::skip]
    fn with_options(ifname: &str, mode: Mode, packet_info: bool) -> Result<Self> {
        let fd = OpenOptions::new().read(true).write(true).open("/dev/net/tun")?;
        let mut name_buffer = Vec::new();
        name_buffer.extend_from_slice(ifname.as_bytes());
        name_buffer.extend_from_slice(&[0; 33]);
        let name_ptr: *mut u8 = name_buffer.as_mut_ptr();
        let result = unsafe { tuntap_setup(fd.as_raw_fd(), name_ptr, mode as c_int, if packet_info { 1 } else { 0 }) };
        if result < 0 {
            return Err(Error::last_os_error());
        }
        let name = unsafe { CStr::from_ptr(name_ptr as *const c_char).to_string_lossy().into_owned() };
        Ok(Iface { fd, mode, name })
    }

    /// Returns the mode of the adapter.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Returns the real name of the adapter.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Receives a packet from the interface.
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        (&self.fd).read(buf)
    }

    /// Sends a packet into the interface.
    pub fn send(&self, buf: &[u8]) -> Result<usize> {
        (&self.fd).write(buf)
    }

    /// Sets the interface to be non-blocking
    #[cfg(feature = "libc")]
    pub fn set_non_blocking(&self) -> Result<()> {
        let fd = self.as_raw_fd();
        let mut nonblock: c_int = 1;
        let result = unsafe { libc::ioctl(fd, libc::FIONBIO, &mut nonblock) };
        if result == -1 {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl AsRawFd for Iface {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl IntoRawFd for Iface {
    fn into_raw_fd(self) -> RawFd {
        self.fd.into_raw_fd()
    }
}
