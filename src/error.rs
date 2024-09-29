use std::{
    error::Error as StdError,
    fmt, io,
    num::{ParseFloatError, ParseIntError},
    result::Result as StdResult,
};

/// A type alias for `Result<T, paf::Error>`.
pub type Result<T> = StdResult<T, Error>;

/// An error type for this crate.
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    /// A crate private constructor for `Error`.
    pub(crate) fn new(kind: ErrorKind) -> Error {
        Error(Box::new(kind))
    }

    /// Return the specific type of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    /// Unwrap this error into its underlying type.
    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }
}

/// Specific errors that can happen.
#[derive(Debug)]
pub enum ErrorKind {
    /// I/O error.
    Io(io::Error),
    /// Could not convert a field into an integer.
    Int(ParseIntError),
    /// Could not convert a field into a float.
    Float(ParseFloatError),
    /// Error whilst reading a record.
    ReadRecord(String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::new(ErrorKind::Io(err))
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::new(ErrorKind::Int(err))
    }
}
impl From<ParseFloatError> for Error {
    fn from(err: ParseFloatError) -> Self {
        Error::new(ErrorKind::Float(err))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            ErrorKind::Io(ref err) => write!(f, "I/O error - {}", err),
            ErrorKind::Int(ref err) => write!(f, "parsing integer error - {}", err),
            ErrorKind::Float(ref err) => write!(f, "parsing float error - {}", err),
            ErrorKind::ReadRecord(ref err) => write!(f, "reading record - {}", err),
        }
    }
}

impl StdError for Error {}
