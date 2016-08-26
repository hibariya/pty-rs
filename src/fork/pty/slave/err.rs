use std::fmt;
use std::error::Error;

use ::descriptor::DescriptorError;

/// The alias `Result` learns `SlaveError` possibility.

pub type Result<T> = ::std::result::Result<T, SlaveError>;

/// The enum `SlaveError` defines the possible errors from constructor Slave.

#[derive(Clone, Copy, Debug)]
pub enum SlaveError {
  BadDescriptor(DescriptorError),
  Dup2Error,
}

impl fmt::Display for SlaveError {

  /// The function `fmt` formats the value using the given formatter.

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", ::errno::errno())
  }
}

impl Error for SlaveError {

  /// The function `description` returns a short description of the error.

  fn description(&self) -> &str {
    match *self {
      SlaveError::BadDescriptor(_) => "the descriptor as meet an error",
      SlaveError::Dup2Error => "the `dup2` has a error, errno is\
                                set appropriately.",
    }
  }

  /// The function `cause` returns the lower-level cause of this error, if any.

  fn cause(&self) -> Option<&Error> {
    match *self {
      SlaveError::BadDescriptor(ref err) => Some(err),
      _ => None,
    }
  }
}
