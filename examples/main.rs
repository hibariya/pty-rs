extern crate pty;
extern crate libc;

use pty::fork::*;
use std::ffi::CString;
use std::io::Read;
use std::ptr;

fn main() {
    let fork = Fork::from_ptmx().unwrap();

    if let Some(mut master) = fork.is_parent().ok() {
        // Read output via PTY master
        let mut output = String::new();

        match master.read_to_string(&mut output) {
            Ok(_nread) => println!("child tty is: {}", output.trim()),
            Err(e) => panic!("read error: {}", e),
        }
    } else {
        // Child process just exec `tty`
        let cmd = CString::new("tty").unwrap().as_ptr();
        let args = [cmd, ptr::null()].as_mut_ptr();

        unsafe { libc::execvp(cmd, args) };
    }
}
