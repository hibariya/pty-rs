#![feature(zero_one)]

extern crate libc;

use std::io;
use std::num::One;
use std::ops::Neg;

mod ffi;

macro_rules! unsafe_try {
    ( $x:expr ) => {
        try!(to_result(unsafe { $x }))
    };
}

pub struct Child {
    pid: libc::pid_t
}

impl Child {
    pub fn pid(&self) -> libc::pid_t {
        self.pid
    }

    pub fn wait(&self) {
        unsafe { libc::waitpid(self.pid, std::ptr::null(), 0) };
    }
}

pub struct Master {
    fd: libc::c_int
}

impl Master {
    pub fn raw(&self) -> libc::c_int {
        self.fd
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = unsafe_try!(
            libc::read(self.fd,
                       buf.as_mut_ptr() as *mut libc::c_void,
                       buf.len() as libc::size_t)
        );

        Ok(ret as usize)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = unsafe_try!(
            libc::write(self.fd,
                        buf.as_ptr() as *const libc::c_void,
                        buf.len() as libc::size_t)
        );

        Ok(ret as usize)
    }

    pub fn close(&self) {
        unsafe { libc::close(self.fd) };
    }
}

impl Copy for Master {}

impl Clone for Master {
    fn clone(&self) -> Master { *self }
}

pub fn fork() -> io::Result<(Child, Master)>
{
    let pty_master = try!(open_ptm());
    let pid        = unsafe_try!(libc::fork());

    if pid == 0 {
        try!(attach_pts(pty_master));

        return Ok((Child { pid: 0 }, Master { fd: -1 }));
    }
    else {
        return Ok((Child { pid: pid }, Master { fd: pty_master }));
    }
}

fn open_ptm() -> io::Result<libc::c_int> {
    let pty_master = unsafe_try!(ffi::posix_openpt(libc::O_RDWR));

    unsafe_try!(ffi::grantpt(pty_master));
    unsafe_try!(ffi::unlockpt(pty_master));

    Ok(pty_master)
}

fn attach_pts(pty_master: libc::c_int) -> io::Result<()> {
    let pts_name = unsafe { ffi::ptsname(pty_master) };

    if (pts_name as i32) == 0 {
        return Err(io::Error::last_os_error())
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

#[inline]
fn to_result<T: One + PartialEq + Neg<Output=T>>(t: T) -> io::Result<T> {
    let one: T = T::one();

    if t == -one {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

#[test]
fn it_works() {
}
