use ::libc;

#[link(name = "c")]
extern {
  pub fn read(fd: libc::c_int, buf: *mut libc::c_uchar, count: libc::size_t) -> libc::ssize_t;
  pub fn write(fd: libc::c_int, buf: *const libc::c_uchar, count: libc::size_t) -> libc::ssize_t;
  pub fn grantpt(fd: libc::c_int) -> libc::c_int;
  pub fn unlockpt(fd: libc::c_int) -> libc::c_int;
  pub fn ptsname(fd: libc::c_int) -> *const libc::c_schar;
}
