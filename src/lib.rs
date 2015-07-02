extern crate libc;

extern {
    fn posix_openpt(flags: libc::c_int) -> libc::c_int;
    fn grantpt(fd: libc::c_int) -> libc::c_int;
    fn unlockpt(fd: libc::c_int) -> libc::c_int;
    fn ptsname(fd: libc::c_int) -> *mut libc::c_schar;
}

pub fn fork() -> (libc::pid_t, libc::c_int)
{
    let pty_master = unsafe { posix_openpt(libc::O_RDWR) };

    unsafe { grantpt(pty_master) };
    unsafe { unlockpt(pty_master) };

    let pts_name = unsafe { ptsname(pty_master) };
    let pid      = unsafe { libc::fork() };

    if pid == 0 {
        unsafe { libc::setsid() };

        let pty_slave = unsafe { libc::open(pts_name, libc::O_RDWR, 0) };

        unsafe { libc::close(pty_master) };

        unsafe { libc::dup2(pty_slave, libc::STDIN_FILENO) };
        unsafe { libc::dup2(pty_slave, libc::STDOUT_FILENO) };
        unsafe { libc::dup2(pty_slave, libc::STDERR_FILENO) };

        unsafe { libc::close(pty_slave) };

        return (0, -1);
    }
    else {
        return (pid, pty_master);
    }
}

#[test]
fn it_works() {
}
