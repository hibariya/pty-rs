extern crate pty;
extern crate libc;
extern crate errno;

use self::pty::prelude::*;

use std::ffi;

use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::ptr;
use std::string::String;

#[test]
fn it_fork_with_new_pty() {
    let fork = Fork::from_ptmx().unwrap();

    if let Some(mut master) = fork.is_parent().ok() {
        let mut string = String::new();

        master.read_to_string(&mut string).unwrap_or_else(|e| panic!(e));

        let output = Command::new("tty")
            .stdin(Stdio::inherit())
            .output()
            .unwrap()
            .stdout;
        let output_str = String::from_utf8_lossy(&output);

        let parent_tty = output_str.trim();
        let child_tty = string.trim();

        assert!(child_tty != "");
        assert!(child_tty != parent_tty);

        // only compare if parent is tty
        // travis runs the tests without tty
        if parent_tty != "not a tty" {
            let mut parent_tty_dir: Vec<&str> = parent_tty.split("/").collect();
            let mut child_tty_dir: Vec<&str> = child_tty.split("/").collect();

            parent_tty_dir.pop();
            child_tty_dir.pop();

            assert_eq!(parent_tty_dir, child_tty_dir);
        }
    } else {
        let cmd = ffi::CString::new("tty").unwrap();
        let mut args: Vec<*const libc::c_char> = Vec::with_capacity(1);

        args.push(cmd.as_ptr());
        args.push(ptr::null());
        unsafe {
            if libc::execvp(cmd.as_ptr(), args.as_mut_ptr()).eq(&-1) {
                panic!("{}: {}", cmd.to_string_lossy(), self::errno::errno());
            }
        }
    }
}
