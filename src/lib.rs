/*!
The `paf` crate parses PAF files. PAF is a Pairwise mApping Format which is
commonly used to represent the mapping between two sets of sequences. This
crate is based on output produced from minimap2, so please see the documentation
of minimap2 to see the full set of fields that can be present in a PAF file.

# Example

```no_run
use paf::Reader;
use std::env;

fn main() {
    let data = env::args().skip(1).next();

    if let Some(filename) = data {
        let mut reader = Reader::from_path(&filename).unwrap();
        for record in reader.records() {
            let record = record.unwrap();
            println!("{:?}", record);
        }
    } else {
        eprintln!("Usage: print_paf <filename>");
        std::process::exit(1);
    }
}

```

*/

/// The error module provides the error type and kind for the crate.
mod error;
/// The reader module provides the reader and record types.
mod reader;
/// The writer module provides the writer type.
mod writer;

pub use crate::{
    error::{Error, ErrorKind, Result},
    reader::{PafRecord, Reader, RecordsIntoIter, RecordsIter, Tag, Type},
    writer::Writer,
};
