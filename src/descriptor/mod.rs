mod err;

use std::os::unix::io::{AsRawFd, RawFd};

use ::libc;

pub use self::err::DescriptorError;

pub trait Descriptor : AsRawFd {

  /// The constructor function `open` opens the path
  /// and returns the fd.
  fn open (
    path: *const libc::c_char,
    flag: libc::c_int,
    mode: Option<libc::mode_t>,
  ) -> Result<RawFd, DescriptorError> {
    unsafe {
      match libc::open(path, flag, mode.unwrap_or_default()) {
       -1 => Err(DescriptorError::OpenFail),
       fd => Ok(fd),
      }
    }
  }

  /// The desctructor function `close` closes the fd.
  fn close (
    &self
  ) -> Result<(), DescriptorError> {
    unsafe {
      match libc::close(self.as_raw_fd()) {
        -1 => Err(DescriptorError::CloseFail),
        _ => Ok(()),
      }
    }
  }
}
