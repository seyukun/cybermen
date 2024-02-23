use std::process::{Command, ExitStatus};

pub fn cmd(cmd: &str, args: &[&str]) -> Result<ExitStatus, std::io::Error> {
    let mut child = Command::new(cmd).args(args).spawn().unwrap();
    child.wait()
}
