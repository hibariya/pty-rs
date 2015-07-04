extern crate libc;

use std::io;

mod ffi;

macro_rules! unsafe_try {
    ( $x:expr ) => {
        try!(int_result(unsafe { $x }))
    };
}

pub fn fork() -> io::Result<(libc::pid_t, libc::c_int)>
{
    let pty_master = unsafe_try!(ffi::posix_openpt(libc::O_RDWR));

    unsafe_try!(ffi::grantpt(pty_master));
    unsafe_try!(ffi::unlockpt(pty_master));

    let pid = unsafe_try!(libc::fork());

    if pid == 0 {
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

        return Ok((0, -1));
    }
    else {
        return Ok((pid, pty_master));
    }
}

#[inline]
fn int_result(value: libc::c_int) -> io::Result<libc::c_int> {
    if value < 0 {
        return Err(io::Error::last_os_error())
    }

    Ok(value)
}

#[test]
fn it_works() {
}
