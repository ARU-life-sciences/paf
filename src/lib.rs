mod error;
mod reader;

pub use crate::{
    error::{Error, ErrorKind, Result},
    reader::{PafRecord, Reader, RecordsIntoIter, RecordsIter, Tag, Type},
};
