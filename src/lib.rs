#![deny(trivial_casts, trivial_numeric_casts,
        unstable_features,
        unused_import_braces, unused_qualifications)]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

extern crate libc;
extern crate nix;

use nix::errno;
use nix::sys::wait;
use std::fmt;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::result;

mod ffi;

macro_rules! unsafe_try {
    ( $x:expr ) => {{
        let ret = unsafe { $x };

        if ret < 0 {
            return Err($crate::last_error());
        } else {
            ret
        }
    }};
}

#[derive(Debug)]
pub enum Error {
    Sys(i32),
}

pub type Result<T> = result::Result<T, Error>;

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Sys(n) => errno::from_i32(n).desc(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        std::error::Error::description(self).fmt(f)
    }
}

impl From<i32> for Error {
    fn from(n: i32) -> Error {
        Error::Sys(n)
    }
}

impl From<::nix::Error> for Error {
    fn from(e: ::nix::Error) -> Error {
        Error::Sys(e.errno() as i32)
    }
}

fn last_error() -> Error {
    Error::from(errno::errno())
}

/// A type representing child process' pty.
#[derive(Clone)]
pub struct ChildPTY {
    fd: libc::c_int,
}

/// A type representing child process.
#[derive(Clone)]
pub struct Child {
    pid: libc::pid_t,
    pty: Option<ChildPTY>,
}

impl Child {
    /// Returns its pid.
    pub fn pid(&self) -> libc::pid_t {
        self.pid
    }

    /// Returns a copy of its pty.
    pub fn pty(&self) -> Option<ChildPTY> {
        self.pty.clone()
    }

    /// Waits until it's terminated. Then closes its pty.
    pub fn wait(&self) -> Result<()> {
        loop {
            match try!(wait::waitpid(self.pid, None)) {
                wait::WaitStatus::StillAlive => continue,
                _ => return self.pty().unwrap().close(),
            }
        }
    }
}

impl ChildPTY {
    /// Closes own file descriptor.
    pub fn close(&self) -> Result<()> {
        if unsafe { libc::close(self.as_raw_fd()) } < 0 {
            Err(Error::from(errno::errno()))
        } else {
            Ok(())
        }
    }
}

impl AsRawFd for ChildPTY {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Read for ChildPTY {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let nread = unsafe {
            libc::read(self.fd,
                       buf.as_mut_ptr() as *mut libc::c_void,
                       buf.len() as libc::size_t)
        };

        if nread < 0 {
            Ok(0)
        } else {
            Ok(nread as usize)
        }
    }
}

impl Write for ChildPTY {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let ret = unsafe {
            libc::write(self.fd,
                        buf.as_ptr() as *const libc::c_void,
                        buf.len() as libc::size_t)
        };

        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Fork with new pseudo-terminal (PTY).
///
/// # Examples
///
/// ```rust
/// extern crate libc;
/// extern crate pty;
///
/// use std::ffi::CString;
/// use std::io::Read;
/// use std::ptr;
///
/// fn main()
/// {
///     let child = pty::fork().unwrap();
///
///     if child.pid() == 0 {
///         // Child process just exec `tty`
///         let cmd  = CString::new("tty").unwrap().as_ptr();
///         let args = [cmd, ptr::null()].as_mut_ptr();
///
///         unsafe { libc::execvp(cmd, args) };
///     }
///     else {
///         // Read output via PTY master
///         let mut output     = String::new();
///         let mut pty_master = child.pty().unwrap();
///
///         match pty_master.read_to_string(&mut output) {
///             Ok(_nread) => println!("child tty is: {}", output.trim()),
///             Err(e)     => panic!("read error: {}", e)
///         }
///
///         let _ = child.wait();
///     }
/// }
/// ```
pub fn fork() -> Result<Child> {
    let pty_master = try!(open_ptm());
    let pid = unsafe_try!(libc::fork());

    if pid == 0 {
        try!(attach_pts(pty_master));

        Ok(Child {
            pid: pid,
            pty: None,
        })
    } else {
        Ok(Child {
            pid: pid,
            pty: Some(ChildPTY { fd: pty_master }),
        })
    }
}

fn open_ptm() -> Result<libc::c_int> {
    let pty_master = unsafe_try!(ffi::posix_openpt(libc::O_RDWR));

    unsafe_try!(ffi::grantpt(pty_master));
    unsafe_try!(ffi::unlockpt(pty_master));

    Ok(pty_master)
}

fn attach_pts(pty_master: libc::c_int) -> Result<()> {
    let pts_name = unsafe { ffi::ptsname(pty_master) };

    if (pts_name as *const i32) == std::ptr::null() {
        return Err(last_error());
    }

    unsafe_try!(libc::close(pty_master));
    unsafe_try!(libc::setsid());

    let pty_slave = unsafe_try!(libc::open(pts_name, libc::O_RDWR, 0));

    unsafe_try!(libc::dup2(pty_slave, libc::STDIN_FILENO));
    unsafe_try!(libc::dup2(pty_slave, libc::STDOUT_FILENO));
    unsafe_try!(libc::dup2(pty_slave, libc::STDERR_FILENO));

    unsafe_try!(libc::close(pty_slave));

    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate libc;

    use std::ffi::CString;
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::ptr;
    use std::string::String;
    use super::fork;

    #[test]
    fn it_fork_with_new_pty() {
        let child = fork().unwrap();

        if child.pid() == 0 {
            let mut ptrs = [CString::new("tty").unwrap().as_ptr(), ptr::null()];

            let _ = unsafe { libc::execvp(*ptrs.as_ptr(), ptrs.as_mut_ptr()) };
        } else {
            let mut pty = child.pty().unwrap();
            let mut string = String::new();

            pty.read_to_string(&mut string).unwrap_or_else(|e| panic!(e));

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

        let _ = child.wait();
    }

    #[test]
    fn it_can_read_write() {
        let child = fork().unwrap();

        if child.pid() == 0 {
            let mut ptrs = [CString::new("bash").unwrap().as_ptr(), ptr::null()];

            print!(" "); // FIXME I'm not sure but this is needed to prevent read-block.

            let _ = unsafe { libc::execvp(*ptrs.as_ptr(), ptrs.as_mut_ptr()) };
        } else {
            let mut pty = child.pty().unwrap();
            let _ = pty.write("echo readme!\n".to_string().as_bytes());

            let mut string = String::new();

            pty.read_to_string(&mut string).unwrap_or_else(|e| panic!(e));

            assert!(string.contains("readme!"));

            let _ = pty.write("exit\n".to_string().as_bytes());
        }

        let _ = child.wait();
    }
}
