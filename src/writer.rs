use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::{Error, ErrorKind, PafRecord, Result, Type};

/// Writes PAF records to a file or any [`Write`].
///
/// Call [`Writer::flush`] (or [`Writer::into_inner`]) when you have finished
/// writing. A buffered writer that is simply dropped still flushes, but any
/// error from that final write is lost.
pub struct Writer<W: Write> {
    writer: W,
}

impl Writer<BufWriter<File>> {
    /// Creates (or truncates) the file at `path` and writes to it through a
    /// buffer.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Writer::new(BufWriter::new(File::create(path)?)))
    }
}

impl<W: Write> Writer<W> {
    /// Creates a writer over `writer`. This does not add a buffer, so wrap
    /// an unbuffered writer such as a [`File`] in a [`BufWriter`] first.
    pub fn new(writer: W) -> Self {
        Writer { writer }
    }

    /// Writes one record as a line, with tags in the order of
    /// [`PafRecord::tags`].
    ///
    /// Fails without writing anything if the record cannot be represented
    /// as PAF: a sequence name is empty, or a name or tag value contains a
    /// tab or newline.
    pub fn write_record(&mut self, record: &PafRecord) -> Result<()> {
        check_record(record)?;
        writeln!(self.writer, "{record}")?;
        Ok(())
    }

    /// Flushes buffered output to the underlying writer.
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    /// Returns a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        &self.writer
    }

    /// Returns a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Flushes and returns the underlying writer.
    pub fn into_inner(mut self) -> Result<W> {
        self.writer.flush()?;
        Ok(self.writer)
    }
}

fn is_line_break_or_tab(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r')
}

fn write_error(field: &str, msg: &str) -> Error {
    Error::new(ErrorKind::WriteRecord(msg.into())).with_field(field)
}

/// Checks that writing `record` produces exactly one parseable PAF line.
fn check_record(record: &PafRecord) -> Result<()> {
    for (field, name) in [
        ("query_name", &record.query_name),
        ("target_name", &record.target_name),
    ] {
        if name.is_empty() {
            return Err(write_error(field, "name is empty"));
        }
        if name.contains(is_line_break_or_tab) {
            return Err(write_error(field, "name contains a tab or line break"));
        }
    }
    for tag in &record.tags {
        let ok = match tag.value() {
            Type::Char(c) => !is_line_break_or_tab(*c),
            Type::String(s) | Type::Hex(s) | Type::Array(s) => !s.contains(is_line_break_or_tab),
            Type::Int(_) | Type::Float(_) => true,
        };
        if !ok {
            return Err(write_error(
                tag.name(),
                "value contains a tab or line break",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ErrorKind, Reader, Strand, Tag};

    fn record(tags: Vec<Tag>) -> PafRecord {
        PafRecord {
            query_name: "query2".into(),
            query_len: 2000,
            query_start: 150,
            query_end: 900,
            strand: Strand::Reverse,
            target_name: "target2".into(),
            target_len: 2500,
            target_start: 300,
            target_end: 1000,
            residue_matches: 400,
            alignment_block_len: 800,
            mapping_quality: 70,
            tags,
        }
    }

    fn write(record: &PafRecord) -> Result<String> {
        let mut writer = Writer::new(Vec::new());
        writer.write_record(record)?;
        Ok(String::from_utf8(writer.into_inner()?).unwrap())
    }

    #[test]
    fn test_write_record_mandatory_fields() {
        assert_eq!(
            write(&record(vec![])).unwrap(),
            "query2\t2000\t150\t900\t-\ttarget2\t2500\t300\t1000\t400\t800\t70\n"
        );
    }

    #[test]
    fn tags_are_written_in_order() {
        let tags = vec![
            Tag::new("tp", Type::Char('P')).unwrap(),
            Tag::new("cm", Type::Int(42)).unwrap(),
            Tag::new("s1", Type::Int(99)).unwrap(),
            "cs:Z::10*ag:5".parse().unwrap(),
            "ML:B:C,1,2".parse().unwrap(),
        ];
        assert_eq!(
            write(&record(tags)).unwrap(),
            "query2\t2000\t150\t900\t-\ttarget2\t2500\t300\t1000\t400\t800\t70\
             \ttp:A:P\tcm:i:42\ts1:i:99\tcs:Z::10*ag:5\tML:B:C,1,2\n"
        );
    }

    #[test]
    fn floats_keep_full_precision() {
        let output = write(&record(vec![
            Tag::new("dv", Type::Float(0.00012345)).unwrap()
        ]));
        assert!(output.unwrap().ends_with("\tdv:f:0.00012345\n"));
    }

    #[test]
    fn written_records_read_back_identically() {
        let original = record(vec![
            "NM:i:-3".parse().unwrap(),
            "de:f:0.1".parse().unwrap(),
            "cs:Z::3*ag".parse().unwrap(),
            "XH:H:1AE3".parse().unwrap(),
        ]);
        let text = write(&original).unwrap();
        let read = Reader::new(text.as_bytes()).read_record().unwrap().unwrap();
        assert_eq!(read, original);
    }

    #[test]
    fn records_that_would_corrupt_the_file_are_rejected() {
        let mut bad = record(vec![]);
        bad.query_name = "a\tb".into();
        let err = write(&bad).unwrap_err();
        assert!(matches!(err.kind(), ErrorKind::WriteRecord(_)));
        assert_eq!(err.field(), Some("query_name"));

        let mut bad = record(vec![]);
        bad.target_name.clear();
        assert_eq!(write(&bad).unwrap_err().field(), Some("target_name"));

        let bad = record(vec![Tag::new("cs", Type::String("a\nb".into())).unwrap()]);
        assert_eq!(write(&bad).unwrap_err().field(), Some("cs"));

        let bad = record(vec![Tag::new("tp", Type::Char('\t')).unwrap()]);
        assert_eq!(write(&bad).unwrap_err().field(), Some("tp"));
    }

    #[test]
    fn nothing_is_written_for_a_rejected_record() {
        let mut writer = Writer::new(Vec::new());
        let mut bad = record(vec![]);
        bad.query_name = "a\nb".into();
        assert!(writer.write_record(&bad).is_err());
        assert!(writer.get_ref().is_empty());
    }

    #[test]
    fn into_inner_flushes_a_buffered_writer() {
        let mut writer = Writer::new(BufWriter::new(Vec::new()));
        writer.write_record(&record(vec![])).unwrap();
        let buffered = writer.into_inner().unwrap();
        assert!(!buffered.get_ref().is_empty());
    }
}
