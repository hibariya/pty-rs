extern crate pty;
extern crate libc;

use self::pty::prelude::*;

use std::io::prelude::*;

use std::ffi::CString;
use std::ptr;
use std::string::String;

#[test]
fn it_can_read_write() {
  let fork = Fork::from_ptmx().unwrap();

  if let Some(mut master) = fork.is_father().ok() {
    let _ = master.write("echo readme!\n".to_string().as_bytes());

    let mut string = String::new();

    master.read_to_string(&mut string).unwrap_or_else(|e| panic!(e));

    assert!(string.contains("readme!"));

    let _ = master.write("exit\n".to_string().as_bytes());
  }
  else {
    let mut ptrs = [CString::new("bash").unwrap().as_ptr(), ptr::null()];

    print!(" "); // FIXME I'm not sure but this is needed to prevent read-block.

    let _ = unsafe { libc::execvp(*ptrs.as_ptr(), ptrs.as_mut_ptr()) };
  }
}
