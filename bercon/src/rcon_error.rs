use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
#[allow(dead_code)]
pub enum RconError {
    Network(io::Error),
    Command(String),
}

impl fmt::Display for RconError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            RconError::Network(ref err) => write!(f, "network error: {}", err),
            RconError::Command(ref err) => write!(f, "command error: {}", err),
        }
    }
}

impl error::Error for RconError {
    fn description(&self) -> &str {
        match *self {
            RconError::Network(ref err) => err.description(),
            RconError::Command(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            RconError::Network(ref err) => Some(err),
            RconError::Command(_) => None,
        }
    }
}

impl From<io::Error> for RconError {
    fn from(err: io::Error) -> RconError {
        RconError::Network(err)
    }
}