# tuntap
A TUN/TAP bindings for Rust.
This is a basic interface to create userspace virtual network adapter.
Creating the devices requires `CAP_NETADM` privileges (most commonly done by running as root).

# Known issues
 * It is tested only on Linux and probably doesn't work anywhere else, even though other systems
   have some TUN/TAP support. Reports that it works (or not) and pull request to add other
   sustem's support are welcome.
 * The [`Async`](async/struct.Async.html) interface is very minimal and will require extention
   for further use cases and better performance.
 * This doesn't support advanced usage patters, like reusing already created device or creating
   persistent devices. Again, pull requests are welcome.
 * There are no automated tests. Any idea how to test this in a reasonable way?

# Creates a new virtual interface.
## Examples
```rust,no_run
# use tun_tap::*;
let iface = Iface::new("mytun", Mode::Tun).expect("Failed to create a TUN device");
let name = iface.name();
// Configure the device ‒ set IP address on it, bring it up.
let mut buffer = vec![0; 1504]; // MTU + 4 for the header
iface.recv(&mut buffer).unwrap();
```