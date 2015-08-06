extern crate libc;

use std::io::{self, Read, Write};

mod ffi;

macro_rules! unsafe_try {
    ( $x:expr ) => {
        try!($crate::to_result(unsafe { $x }))
    };
}

pub struct Child {
    pid: libc::pid_t
}

impl Child {
    pub fn pid(&self) -> libc::pid_t { self.pid }

    pub fn wait(&self) {
        unsafe { libc::waitpid(self.pid, std::ptr::null(), 0) };
    }
}

#[derive(Clone, Copy)]
pub struct Master {
    fd: libc::c_int
}

impl Master {
    pub fn fd(&self) -> libc::c_int { self.fd }

    pub fn close(&self) {
        unsafe { libc::close(self.fd) };
    }
}

impl Read for Master {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match to_result(unsafe {
            libc::read(self.fd,
                       buf.as_mut_ptr() as *mut libc::c_void,
                       buf.len() as libc::size_t)
        }) {
            Ok(nread) => Ok(nread as usize),
            Err(_)    => Ok(0 as usize)
        }
    }
}

impl Write for Master {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let ret = unsafe_try!(
            libc::write(self.fd,
                        buf.as_ptr() as *const libc::c_void,
                        buf.len() as libc::size_t)
        );

        Ok(ret as usize)
    }

    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

pub fn fork() -> io::Result<(Child, Master)>
{
    let pty_master = try!(open_ptm());
    let pid        = unsafe_try!(libc::fork());

    if pid == 0 {
        try!(attach_pts(pty_master));

        Ok((Child { pid: pid }, Master { fd: -1 }))
    }
    else {
        Ok((Child { pid: pid }, Master { fd: pty_master }))
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

    if (pts_name as *const i32) == std::ptr::null() {
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

// XXX use <T: Neg<Output=T> + One + PartialEq> trait instead
trait CReturnValue { fn as_c_return_value_is_error(&self) -> bool; }

macro_rules! impl_as_c_return_value_is_error {
    () => {
        fn as_c_return_value_is_error(&self) -> bool { *self == -1 }
    }
}

impl CReturnValue for i32 { impl_as_c_return_value_is_error!(); }
impl CReturnValue for i64 { impl_as_c_return_value_is_error!(); }

#[inline]
fn to_result<T: CReturnValue>(r: T) -> io::Result<T> {
    if r.as_c_return_value_is_error() {
        Err(io::Error::last_os_error())
    } else {
        Ok(r)
    }
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
        let (child, mut master) = fork().unwrap();

        if child.pid() == 0 {
            let mut ptrs = [CString::new("tty").unwrap().as_ptr(), ptr::null()];

            unsafe { libc::execvp(*ptrs.as_ptr(), ptrs.as_mut_ptr()) };
        }
        else {
            let mut string = String::new();

            match master.read_to_string(&mut string) {
                Ok(_)  => {
                    let output = Command::new("tty").stdin(Stdio::inherit()).output().unwrap().stdout;

                    let parent_tty = String::from_utf8_lossy(&output);
                    let child_tty  = string.trim();

                    assert!(child_tty != "");
                    assert!(child_tty != parent_tty);

                    let mut parent_tty_dir: Vec<&str> = parent_tty.split("/").collect();
                    let mut child_tty_dir:  Vec<&str> = child_tty.split("/").collect();

                    parent_tty_dir.pop();
                    child_tty_dir.pop();

                    assert_eq!(parent_tty_dir, child_tty_dir);
                },
                Err(e) => panic!("{}", e)
            }

            child.wait();
            master.close();
        }
    }

    #[test]
    fn it_can_read_write() {
        let (child, mut master) = fork().unwrap();

        if child.pid() == 0 {
            let mut ptrs = [CString::new("bash").unwrap().as_ptr(), ptr::null()];

            print!(" "); // FIXME I'm not sure but this is needed to prevent read-block.

            unsafe { libc::execvp(*ptrs.as_ptr(), ptrs.as_mut_ptr()) };
        }
        else {
            let _ = master.write("echo readme!\n".to_string().as_bytes());

            let mut string = String::new();

            match master.read_to_string(&mut string) {
                Ok(_)  => {
                    assert!(string.contains("readme!"));
                },
                Err(e) => panic!("{}", e)
            }

            let _ = master.write("exit\n".to_string().as_bytes());

            child.wait();
            master.close();
        }
    }
}
