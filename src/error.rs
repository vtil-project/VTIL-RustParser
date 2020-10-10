use std::fmt::{self, Display};
use std::{error, io, str};

#[derive(Debug)]
/// Custom `Error` for VTIL parsing
pub enum Error {
    Malformed(String),
    Io(io::Error),
    Scroll(scroll::Error),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Malformed(_) => "Data is malformed",
            Error::Io(_) => "I/O error",
            Error::Scroll(_) => "Scroll error",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::Malformed(_) => None,
            Error::Io(ref err) => err.source(),
            Error::Scroll(ref err) => err.source(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<scroll::Error> for Error {
    fn from(err: scroll::Error) -> Error {
        Error::Scroll(err)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Error {
        Error::Malformed(err.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Malformed(ref message) => write!(fmt, "Error while reading: {}", message),
            Error::Io(ref err) => write!(fmt, "{}", err),
            Error::Scroll(ref err) => write!(fmt, "{}", err),
        }
    }
}
