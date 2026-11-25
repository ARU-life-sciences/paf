/*!
Read and write PAF files.

PAF (Pairwise mApping Format) is a tab-separated format describing how
sequences in one set map onto sequences in another. It is written by
[minimap2](https://lh3.github.io/minimap2/minimap2.html#10) and many other
aligners. Each line has 12 mandatory columns, followed by optional
`NAME:TYPE:VALUE` tags such as `NM:i:5`.

# Reading

```
use paf::{Reader, Strand};

let data = "read1\t1000\t100\t500\t+\tchr1\t5000\t200\t600\t390\t400\t60\tNM:i:10\ttp:A:P\n";
let mut reader = Reader::new(data.as_bytes());

for record in reader.records() {
    let record = record?;
    assert_eq!(record.query_name, "read1");
    assert_eq!(record.strand, Strand::Forward);
    assert_eq!(record.nm(), Some(10));
}
# Ok::<(), paf::Error>(())
```

Use [`Reader::from_path`] to read a file. Errors report the line and the
column or tag they occurred in.

# Writing

```
use paf::{PafRecord, Strand, Tag, Type, Writer};

let record = PafRecord {
    query_name: "read1".into(),
    query_len: 1000,
    query_start: 100,
    query_end: 500,
    strand: Strand::Reverse,
    target_name: "chr1".into(),
    target_len: 5000,
    target_start: 200,
    target_end: 600,
    residue_matches: 390,
    alignment_block_len: 400,
    mapping_quality: 60,
    tags: vec![Tag::new("tp", Type::Char('P'))?, "NM:i:10".parse()?],
};

let mut writer = Writer::new(Vec::new());
writer.write_record(&record)?;
let output = writer.into_inner()?;
assert_eq!(
    String::from_utf8(output).unwrap(),
    "read1\t1000\t100\t500\t-\tchr1\t5000\t200\t600\t390\t400\t60\ttp:A:P\tNM:i:10\n"
);
# Ok::<(), paf::Error>(())
```

Use [`Writer::from_path`] to write a buffered file, and call
[`Writer::flush`] at the end so that write errors are reported.

# Filtering and editing

Records can be changed in place and written back out. Tags you don't touch,
including ones this crate has no accessor for, are kept in their original
order.

```
use paf::{Reader, Writer};

let data = "r1\t100\t0\t100\t+\tc1\t900\t0\t100\t100\t100\t60\ttp:A:P\tcg:Z:100M\n\
            r2\t100\t0\t100\t+\tc1\t900\t0\t100\t100\t100\t0\ttp:A:S\tcg:Z:100M\n";
let mut writer = Writer::new(Vec::new());

for record in Reader::new(data.as_bytes()).records() {
    let mut record = record?;
    if record.tp() != Some('P') {
        continue;
    }
    record.remove_tag("cg");
    record.query_name.push_str("/1");
    writer.write_record(&record)?;
}

let output = String::from_utf8(writer.into_inner()?).unwrap();
assert_eq!(output, "r1/1\t100\t0\t100\t+\tc1\t900\t0\t100\t100\t100\t60\ttp:A:P\n");
# Ok::<(), paf::Error>(())
```
*/

#![warn(missing_docs)]

/// The error module provides the error type and kind for the crate.
mod error;
/// The reader module provides the reader and its iterators.
mod reader;
/// The record module provides the record, strand and tag types.
mod record;
/// The writer module provides the writer type.
mod writer;

pub use crate::{
    error::{Error, ErrorKind, Result},
    reader::{Reader, RecordsIntoIter, RecordsIter},
    record::{PafRecord, Strand, Tag, Type},
    writer::Writer,
};

// Compile and run the README examples as doc tests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
