mod pty;
mod err;

use std::ffi::CString;

pub use self::pty::{Master, MasterError};
use self::pty::Slave;
pub use self::err::{ForkError, Result};

use ::libc;

#[derive(Debug)]
pub enum Fork {
  // Father child's pid and master's pty.
  Father(libc::pid_t, Master),
  // Child pid 0.
  Child,
}

impl Fork {

  /// The constructor function `new` forks the program
  /// and returns the current pid.
  pub fn new (
    path: &'static str,
  ) -> Result<Self> {
    match Master::new(
      CString::new(path).ok().unwrap_or_default().as_ptr()
    ) {
      Err(cause) => Err(ForkError::BadMaster(cause)),
      Ok(master) => unsafe {
        if let Some(cause) = master.grantpt().err().or(
                             master.unlockpt().err()) {
          Err(ForkError::BadMaster(cause))
        }
        else {
          match libc::fork() {
            -1 => Err(ForkError::Failure),
            0 => match master.ptsname() {
              Err(cause) => Err(ForkError::BadMaster(cause)),
              Ok(name) => Fork::from_pts(name), 
            },
            pid => Ok(Fork::Father(pid, master)),
          }
        }
      },
    }
  }

  /// The constructor function `from_pts` is a private
  /// extention from the constructor function `new` who
  /// prepares and returns the child.
  fn from_pts (
    ptsname: *const ::libc::c_char,
  ) -> Result<Self> {
    unsafe {
      if libc::setsid() == -1 {
        Err(ForkError::SetsidFail)
      }
      else {
        match Slave::new(ptsname) {
          Err(cause) => Err(ForkError::BadSlave(cause)),
          Ok(slave) => {
            if let Some(cause) = slave.dup2(libc::STDIN_FILENO).err().or(
                                 slave.dup2(libc::STDOUT_FILENO).err().or(
                                 slave.dup2(libc::STDERR_FILENO).err())) {
              Err(ForkError::BadSlave(cause))
            }
            else {
              Ok(Fork::Child)
            }
          },
        }
      }
    }
  }

  /// The constructor function `from_ptmx` forks the program
  /// and returns the current pid for a default PTMX's path.
  pub fn from_ptmx() -> Result<Self> {
    Fork::new(::DEFAULT_PTMX)
  }

  /// Waits until it's terminated.
  pub fn wait(&self) -> Result<libc::pid_t> {
    match *self {
      Fork::Child => Err(ForkError::IsChild),
      Fork::Father(pid, _) => loop {
        unsafe {
          match libc::waitpid(pid, &mut 0, 0) {
            0 => continue ,
            -1 => return Err(ForkError::WaitpidFail),
            _ => return Ok(pid),
          }
        }
      },
    }
  }

  /// The function `is_father` returns the pid or father
  /// or none.
  pub fn is_father(&mut self) -> Result<&mut Master> {
    match *self {
      Fork::Child => Err(ForkError::IsChild),
      Fork::Father(_, ref mut master) => Ok(master),
    }
  }

  /// The function `is_child` returns the pid or child
  /// or none.
  pub fn is_child(&self) -> Result<libc::pid_t> {
    match *self {
      Fork::Father(_, _) => Err(ForkError::IsFather),
      Fork::Child => Ok(0),
    }
  }
}
