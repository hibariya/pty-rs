extern crate pty;
extern crate libc;

use self::pty::prelude::*;

use std::io::prelude::*;

use std::ffi::CString;
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

    let parent_tty = String::from_utf8_lossy(&output);
    let child_tty = string.trim();

    assert!(child_tty != "");
    assert!(child_tty != parent_tty);

    let mut parent_tty_dir: Vec<&str> = parent_tty.split("/").collect();
    let mut child_tty_dir: Vec<&str> = child_tty.split("/").collect();

    parent_tty_dir.pop();
    child_tty_dir.pop();

    assert_eq!(parent_tty_dir, child_tty_dir);
  }
  else {
    let mut ptrs = [CString::new("tty").unwrap().as_ptr(), ptr::null()];
    let _ = unsafe { libc::execvp(*ptrs.as_ptr(), ptrs.as_mut_ptr()) };
  }
}
