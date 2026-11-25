use std::{
    error::Error as StdError,
    fmt, io,
    num::{ParseFloatError, ParseIntError},
    result::Result as StdResult,
};

/// A type alias for `Result<T, paf::Error>`.
pub type Result<T> = StdResult<T, Error>;

/// An error that occurred while reading or writing PAF.
///
/// Errors from [`Reader`](crate::Reader) carry the 1-based line number
/// ([`Error::line`]) and, where it applies, the column or tag that could not
/// be parsed ([`Error::field`]). Both are included in the `Display` output,
/// for example `line 7, field query_len: invalid integer: invalid digit found in string`.
#[derive(Debug)]
pub struct Error(Box<Inner>);

#[derive(Debug)]
struct Inner {
    kind: ErrorKind,
    line: Option<u64>,
    field: Option<String>,
}

impl Error {
    /// A crate private constructor for `Error`.
    pub(crate) fn new(kind: ErrorKind) -> Error {
        Error(Box::new(Inner {
            kind,
            line: None,
            field: None,
        }))
    }

    /// Records the line the error occurred on.
    pub(crate) fn with_line(mut self, line: u64) -> Error {
        self.0.line = Some(line);
        self
    }

    /// Records the column or tag the error occurred in.
    pub(crate) fn with_field(mut self, field: impl Into<String>) -> Error {
        self.0.field = Some(field.into());
        self
    }

    /// Return the specific type of this error.
    pub fn kind(&self) -> &ErrorKind {
        &self.0.kind
    }

    /// Unwrap this error into its underlying type.
    pub fn into_kind(self) -> ErrorKind {
        self.0.kind
    }

    /// The 1-based line number the error occurred on, if it came from reading.
    pub fn line(&self) -> Option<u64> {
        self.0.line
    }

    /// The column (such as `query_len`) or tag (such as `NM`) the error
    /// occurred in, if known.
    pub fn field(&self) -> Option<&str> {
        self.0.field.as_deref()
    }
}

/// Specific errors that can happen.
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// I/O error.
    Io(io::Error),
    /// Could not convert a field into an integer.
    Int(ParseIntError),
    /// Could not convert a field into a float.
    Float(ParseFloatError),
    /// A line is not a valid PAF record.
    ReadRecord(String),
    /// An optional field is not a valid `NAME:TYPE:VALUE` tag.
    InvalidTag(String),
    /// A record cannot be written as valid PAF, for example because a name
    /// contains a tab.
    WriteRecord(String),
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.0.line, &self.0.field) {
            (Some(line), Some(field)) => write!(f, "line {line}, field {field}: ")?,
            (Some(line), None) => write!(f, "line {line}: ")?,
            (None, Some(field)) => write!(f, "field {field}: ")?,
            (None, None) => {}
        }
        match &self.0.kind {
            ErrorKind::Io(err) => write!(f, "I/O error: {err}"),
            ErrorKind::Int(err) => write!(f, "invalid integer: {err}"),
            ErrorKind::Float(err) => write!(f, "invalid float: {err}"),
            ErrorKind::ReadRecord(msg) => write!(f, "invalid record: {msg}"),
            ErrorKind::InvalidTag(msg) => write!(f, "invalid tag: {msg}"),
            ErrorKind::WriteRecord(msg) => write!(f, "cannot write record: {msg}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.0.kind {
            ErrorKind::Io(err) => Some(err),
            ErrorKind::Int(err) => Some(err),
            ErrorKind::Float(err) => Some(err),
            ErrorKind::ReadRecord(_) | ErrorKind::InvalidTag(_) | ErrorKind::WriteRecord(_) => None,
        }
    }
}
