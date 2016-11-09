extern crate pty;
extern crate libc;
extern crate errno;

use pty::fork::*;
use std::ffi;
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
        let cmd = ffi::CString::new("tty").unwrap();
        let mut args: Vec<*const libc::c_char> = Vec::with_capacity(1);

        args.push(cmd.as_ptr());
        args.push(ptr::null());
        unsafe {
            if libc::execvp(cmd.as_ptr(), args.as_mut_ptr()).eq(&-1) {
                panic!("{}: {}", cmd.to_string_lossy(), ::errno::errno());
            }
        };
    }
}
