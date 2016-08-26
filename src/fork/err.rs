use std::fmt;
use std::error::Error;

use super::pty::{MasterError, SlaveError};

use ::descriptor::DescriptorError;

/// The alias `Result` learns `ForkError` possibility.

pub type Result<T> = ::std::result::Result<T, ForkError>;

/// The enum `ForkError` defines the possible errors from constructor Fork.

#[derive(Clone, Copy, Debug)]
pub enum ForkError {
    /// Can't creates the child.
    Failure,
    /// Can't set the id group.
    SetsidFail,
    /// Can't suspending the calling process.
    WaitpidFail,
    /// Is child and not father.
    IsChild,
    /// Is father and not child.
    IsFather,
    /// The Master meet a error.
    BadMaster(MasterError),
    /// The Slave meet a error.
    BadSlave(SlaveError),
    /// The Master's Descriptor meet a error.
    BadDescriptorMaster(DescriptorError),
    /// The Slave's Descriptor meet a error.
    BadDescriptorSlave(DescriptorError),
}

impl fmt::Display for ForkError {

    /// The function `fmt` formats the value using the given formatter.

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", ::errno::errno())
    }
}

impl Error for ForkError {

    /// The function `description` returns a short description of the error.

    fn description(&self) -> &str {
        match *self {
            ForkError::Failure => "On failure, -1 is returned in the parent,\
                                   no child process is created, and errno is\
                                   set appropriately.",
            ForkError::SetsidFail => "fails if the calling process is already\
                                      a process group leader.",
            ForkError::WaitpidFail => "Can't suspending the calling process.",
            ForkError::IsChild => "is child and not father",
            ForkError::IsFather => "is father and not child",
            ForkError::BadMaster(_) => "the master as meet an error",
            ForkError::BadSlave(_) => "the slave as meet an error",
            ForkError::BadDescriptorMaster(_) => "the master's descriptor as meet an error",
            ForkError::BadDescriptorSlave(_) => "the slave's descriptor as meet an error",

        }
    }

    /// The function `cause` returns the lower-level cause of this error, if any.

    fn cause(&self) -> Option<&Error> {
        match *self {
          ForkError::BadMaster(ref err) => Some(err),
          ForkError::BadSlave(ref err) => Some(err),
          ForkError::BadDescriptorMaster(ref err) => Some(err),
          ForkError::BadDescriptorSlave(ref err) => Some(err),
          _ => None,
        }
    }
}