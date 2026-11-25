use std::fs::File;
use std::io::{self, BufRead};
use std::iter::FusedIterator;
use std::num::ParseIntError;
use std::path::Path;
use std::str::{self, FromStr};

use crate::{Error, ErrorKind, PafRecord, Result, Tag};

/// Reads PAF records from a file or any [`io::Read`].
///
/// Blank lines are skipped. Errors report the 1-based line number they
/// occurred on.
pub struct Reader<R> {
    reader: io::BufReader<R>,
    /// Reused between lines to avoid an allocation per record.
    buf: Vec<u8>,
    /// Number of lines read so far.
    line: u64,
}

impl Reader<File> {
    /// Opens the file at `path` for reading.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Reader<File>> {
        Ok(Reader::new(File::open(path)?))
    }
}

impl<R: io::Read> Reader<R> {
    /// Creates a reader over `rdr`, adding a buffer.
    pub fn new(rdr: R) -> Self {
        Reader {
            reader: io::BufReader::new(rdr),
            buf: Vec::new(),
            line: 0,
        }
    }

    /// Creates a reader over `rdr`. The same as [`Reader::new`].
    pub fn from_reader(rdr: R) -> Self {
        Reader::new(rdr)
    }

    /// The number of lines read so far, which is the line number of the
    /// record most recently returned.
    pub fn line(&self) -> u64 {
        self.line
    }

    /// A borrowed iterator over the records of a PAF file.
    ///
    /// The iterator yields an error for each malformed line and carries on
    /// with the next one, but stops after an I/O error.
    pub fn records(&mut self) -> RecordsIter<'_, R> {
        RecordsIter {
            rdr: self,
            done: false,
        }
    }

    /// An owned iterator over the records of a PAF file. It behaves like
    /// [`Reader::records`].
    pub fn into_records(self) -> RecordsIntoIter<R> {
        RecordsIntoIter {
            rdr: self,
            done: false,
        }
    }

    /// Reads the next record, skipping blank lines. Returns `Ok(None)` at the
    /// end of the input.
    pub fn read_record(&mut self) -> Result<Option<PafRecord>> {
        loop {
            self.buf.clear();
            if self.reader.read_until(b'\n', &mut self.buf)? == 0 {
                return Ok(None);
            }
            self.line += 1;

            let line = str::from_utf8(&self.buf).map_err(|_| {
                Error::new(ErrorKind::ReadRecord("line is not valid UTF-8".into()))
                    .with_line(self.line)
            })?;
            let line = line.trim_end();
            if line.is_empty() {
                continue;
            }
            return parse_record(line)
                .map(Some)
                .map_err(|e| e.with_line(self.line));
        }
    }
}

/// The tab-separated columns of a line, counting how many have been taken.
struct Columns<'a> {
    iter: str::Split<'a, char>,
    taken: usize,
}

impl<'a> Columns<'a> {
    fn text(&mut self) -> Result<&'a str> {
        let col = self.iter.next().ok_or_else(|| {
            Error::new(ErrorKind::ReadRecord(format!(
                "expected at least 12 tab-separated fields, found {}",
                self.taken
            )))
        })?;
        self.taken += 1;
        Ok(col)
    }

    fn name(&mut self, field: &str) -> Result<String> {
        let col = self.text()?;
        if col.is_empty() {
            return Err(Error::new(ErrorKind::ReadRecord("name is empty".into())).with_field(field));
        }
        Ok(col.to_owned())
    }

    fn int<T: FromStr<Err = ParseIntError>>(&mut self, field: &str) -> Result<T> {
        self.text()?
            .parse()
            .map_err(|e| Error::from(e).with_field(field))
    }
}

/// Parses one non-empty line, with the line ending already removed.
fn parse_record(line: &str) -> Result<PafRecord> {
    let mut cols = Columns {
        iter: line.split('\t'),
        taken: 0,
    };
    // Struct fields are evaluated in the order written, matching the columns.
    Ok(PafRecord {
        query_name: cols.name("query_name")?,
        query_len: cols.int("query_len")?,
        query_start: cols.int("query_start")?,
        query_end: cols.int("query_end")?,
        strand: cols
            .text()?
            .parse()
            .map_err(|e: Error| e.with_field("strand"))?,
        target_name: cols.name("target_name")?,
        target_len: cols.int("target_len")?,
        target_start: cols.int("target_start")?,
        target_end: cols.int("target_end")?,
        residue_matches: cols.int("residue_matches")?,
        alignment_block_len: cols.int("alignment_block_len")?,
        mapping_quality: cols.int("mapping_quality")?,
        tags: cols.iter.map(str::parse::<Tag>).collect::<Result<_>>()?,
    })
}

/// Shared `next` for both iterators: stop for good after an I/O error, since
/// retrying usually fails the same way forever.
fn next_record<R: io::Read>(rdr: &mut Reader<R>, done: &mut bool) -> Option<Result<PafRecord>> {
    if *done {
        return None;
    }
    match rdr.read_record() {
        Ok(Some(record)) => Some(Ok(record)),
        Ok(None) => {
            *done = true;
            None
        }
        Err(e) => {
            if matches!(e.kind(), ErrorKind::Io(_)) {
                *done = true;
            }
            Some(Err(e))
        }
    }
}

/// A borrowed iterator over the records of a PAF file.
pub struct RecordsIter<'r, R> {
    rdr: &'r mut Reader<R>,
    done: bool,
}

impl<R: io::Read> RecordsIter<'_, R> {
    /// Return a reference to the underlying reader.
    pub fn reader(&self) -> &Reader<R> {
        self.rdr
    }

    /// Return a mutable reference to the underlying reader.
    pub fn reader_mut(&mut self) -> &mut Reader<R> {
        self.rdr
    }
}

impl<R: io::Read> Iterator for RecordsIter<'_, R> {
    type Item = Result<PafRecord>;

    fn next(&mut self) -> Option<Result<PafRecord>> {
        next_record(self.rdr, &mut self.done)
    }
}

impl<R: io::Read> FusedIterator for RecordsIter<'_, R> {}

/// An owned iterator over the records of a PAF file.
pub struct RecordsIntoIter<R> {
    rdr: Reader<R>,
    done: bool,
}

impl<R: io::Read> RecordsIntoIter<R> {
    /// Return a reference to the underlying reader.
    pub fn reader(&self) -> &Reader<R> {
        &self.rdr
    }

    /// Return a mutable reference to the underlying reader.
    pub fn reader_mut(&mut self) -> &mut Reader<R> {
        &mut self.rdr
    }

    /// Drop this iterator and return the underlying reader.
    pub fn into_reader(self) -> Reader<R> {
        self.rdr
    }
}

impl<R: io::Read> Iterator for RecordsIntoIter<R> {
    type Item = Result<PafRecord>;

    fn next(&mut self) -> Option<Result<PafRecord>> {
        next_record(&mut self.rdr, &mut self.done)
    }
}

impl<R: io::Read> FusedIterator for RecordsIntoIter<R> {}

#[cfg(test)]
mod tests {
    use super::Reader;
    use crate::{ErrorKind, Strand, Type};

    const PAF_RECORD_1: &[u8] = b"NC_041798.1	41841605	28850796	29394458	+	SUPER_10	44636193	31974877	32470190	495111	515145	60	NM:i:48730	ms:i:488389	AS:i:439775	nn:i:28696	tp:A:P	cm:i:46495	s1:i:466570	s2:i:10896	de:f:0.0003	zd:i:3	rl:i:3568165	cg:Z:770M1D945M1D389M1I9141M1I356M1D196M1I30268M2D789M3I992M2D1819M1D7M1D7M1I10M6D2922M1D17899M2D1010M4D12324M1I1376M1D5549M6D1839M1I2206M1D770M1D2287M1D16103M1D3238M1D2014M1D140M5I14M1D8496M2I2151M1I335M1D14424M1D1093M1I567M1D1835M2D1995M1D5257M1D639M1I699M1I133M1I52M1I99M2I26M1I195M1I1543M1I240M1I176M1I412M2D159M1I261M1D1158M1I933M2D12836M1D993M1D12263M2D4975M2I16452M3I396M1I3924M2D929M3I3015M1D225M1D4225M1D717M2D752M1D2051M1D5110M1D15073M1D1053M2D4369M1D619M3I13564M2I4386M1D1431M2D617M1I612M2I3445M2I252M1D220M1D237M1I903M1I145M1I53M1I197M1I1280M1D4201M1D1736M1D1289M1I3344M2D5456M1D488M1I1655M2D1830M1D796M1I19341M2D1165M1D1926M1D6041M1D2170M1D3917M1D926M1D759M1D400M2I8802M1I836M1I381M48451I166M1I4896M2D1522M49D2729M1D947M2D927M6D911M2D800M2D3040M1D13213M1D8999M3D847M1D220M1I673M1D165M1I901M1I2887M1I105M2I597M1I1201M1I53M2I494M1I23M1D99M1I146M1D29906M1D5661M1I27598M1D520M1I166M2D11600M1D388M1D844M1D4583M1D8390M1D5789M2D3773M1D4494M1D448M1D846M3D531M";

    const MANDATORY: &str = "q\t100\t0\t50\t+\tt\t200\t10\t60\t45\t50\t60";

    fn read_all(data: &str) -> Vec<crate::Result<crate::PafRecord>> {
        Reader::new(data.as_bytes()).into_records().collect()
    }

    #[test]
    fn test_read_record() {
        let mut parser = Reader::new(PAF_RECORD_1);
        let record = parser.read_record().unwrap().unwrap();

        assert_eq!(record.query_name, "NC_041798.1");
        assert_eq!(record.query_len, 41841605);
        assert_eq!(record.query_start, 28850796);
        assert_eq!(record.query_end, 29394458);
        assert_eq!(record.strand, Strand::Forward);
        assert_eq!(record.target_name, "SUPER_10");
        assert_eq!(record.target_len, 44636193);
        assert_eq!(record.target_start, 31974877);
        assert_eq!(record.target_end, 32470190);
        assert_eq!(record.residue_matches, 495111);
        assert_eq!(record.alignment_block_len, 515145);
        assert_eq!(record.mapping_quality, 60);

        assert_eq!(record.nm(), Some(48730));
        assert_eq!(record.tp(), Some('P'));
        assert_eq!(record.de(), Some(0.0003));
        assert!(record.cg().unwrap().starts_with("770M1D945M"));
        assert_eq!(record.tag("zd").unwrap().value(), &Type::Int(3));

        let names: Vec<_> = record.tags.iter().map(|t| t.name()).collect();
        assert_eq!(
            names,
            ["NM", "ms", "AS", "nn", "tp", "cm", "s1", "s2", "de", "zd", "rl", "cg"],
            "tags keep their order from the line"
        );

        assert!(parser.read_record().unwrap().is_none());
    }

    #[test]
    fn values_containing_colons_are_kept_whole() {
        let line = format!("{MANDATORY}\tcs:Z::303*cg:51\tds:Z::10+[a]:5");
        let record = Reader::new(line.as_bytes()).read_record().unwrap().unwrap();
        assert_eq!(record.cs(), Some(":303*cg:51"));
        assert_eq!(record.ds(), Some(":10+[a]:5"));
    }

    #[test]
    fn any_tag_name_and_every_sam_type_is_accepted() {
        let line = format!("{MANDATORY}\tRG:Z:grp1\tML:B:C,1,2\tXH:H:1AE3\tzz:A:x");
        let record = Reader::new(line.as_bytes()).read_record().unwrap().unwrap();
        assert_eq!(
            record.tag("RG").unwrap().value(),
            &Type::String("grp1".into())
        );
        assert_eq!(
            record.tag("ML").unwrap().value(),
            &Type::Array("C,1,2".into())
        );
        assert_eq!(record.tag("XH").unwrap().value(), &Type::Hex("1AE3".into()));
        assert_eq!(record.tag("zz").unwrap().value(), &Type::Char('x'));
    }

    #[test]
    fn wrong_tag_type_gives_none_not_a_panic() {
        let line = format!("{MANDATORY}\ttp:i:5");
        let record = Reader::new(line.as_bytes()).read_record().unwrap().unwrap();
        assert_eq!(record.tp(), None);
    }

    #[test]
    fn errors_report_line_and_field() {
        let data = format!("{MANDATORY}\nq\tNOTNUM\t0\t50\t+\tt\t200\t10\t60\t45\t50\t60\n");
        let results = read_all(&data);
        let err = results[1].as_ref().unwrap_err();
        assert_eq!(err.line(), Some(2));
        assert_eq!(err.field(), Some("query_len"));
        assert!(matches!(err.kind(), ErrorKind::Int(_)));
        assert_eq!(
            err.to_string(),
            "line 2, field query_len: invalid integer: invalid digit found in string"
        );

        let data = format!("{MANDATORY}\tNM:i:abc\n");
        let err = Reader::new(data.as_bytes()).read_record().unwrap_err();
        assert_eq!((err.line(), err.field()), (Some(1), Some("NM")));

        let data = format!("{MANDATORY}\ttp:A\n");
        let err = Reader::new(data.as_bytes()).read_record().unwrap_err();
        assert!(matches!(err.kind(), ErrorKind::InvalidTag(_)));
        assert_eq!(err.line(), Some(1));
    }

    #[test]
    fn line_numbers_count_every_line() {
        let data = format!("{MANDATORY}\n\n{MANDATORY}\nbad line\n{MANDATORY}\n");
        let results = read_all(&data);
        assert_eq!(results.len(), 4);
        let err = results[2].as_ref().unwrap_err();
        assert_eq!(err.line(), Some(4));
        assert_eq!(
            err.to_string(),
            "line 4: invalid record: expected at least 12 tab-separated fields, found 1"
        );
        assert!(results[3].is_ok(), "reading continues after a bad line");

        let mut reader = Reader::new(data.as_bytes());
        reader.read_record().unwrap();
        assert_eq!(reader.line(), 1);
        reader.read_record().unwrap();
        assert_eq!(reader.line(), 3, "blank line 2 is counted but skipped");
    }

    #[test]
    fn blank_lines_and_crlf_are_handled() {
        let data = format!("\n{MANDATORY}\r\n\r\n{MANDATORY}\ttp:A:P\r\n\n\n");
        let records: Vec<_> = read_all(&data).into_iter().map(Result::unwrap).collect();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].mapping_quality, 60);
        assert_eq!(records[1].tp(), Some('P'));
    }

    #[test]
    fn bad_mandatory_fields_are_rejected() {
        let bad_strand = MANDATORY.replace('+', "*");
        let err = Reader::new(bad_strand.as_bytes())
            .read_record()
            .unwrap_err();
        assert_eq!(err.field(), Some("strand"));

        let empty_name = MANDATORY.replacen('q', "", 1);
        let err = Reader::new(empty_name.as_bytes())
            .read_record()
            .unwrap_err();
        assert_eq!(err.field(), Some("query_name"));

        let mapq_too_big = MANDATORY.replace("\t60", "\t256");
        let err = Reader::new(mapq_too_big.as_bytes())
            .read_record()
            .unwrap_err();
        assert_eq!(err.field(), Some("mapping_quality"));
    }

    #[test]
    fn coordinates_beyond_u32_are_supported() {
        let line = "q\t5000000000\t0\t4999999999\t-\tt\t5000000000\t1\t5000000000\t1\t1\t255";
        let record = Reader::new(line.as_bytes()).read_record().unwrap().unwrap();
        assert_eq!(record.query_len, 5_000_000_000);
        assert_eq!(record.strand, Strand::Reverse);
    }

    #[test]
    fn invalid_utf8_is_an_error_for_that_line_only() {
        let mut data = b"q\xff\t1\t0\t1\t+\tt\t1\t0\t1\t1\t1\t60\n".to_vec();
        data.extend_from_slice(MANDATORY.as_bytes());
        let results: Vec<_> = Reader::new(&data[..]).into_records().collect();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].as_ref().unwrap_err().line(), Some(1));
        assert!(results[1].is_ok());
    }

    #[test]
    fn iteration_stops_after_an_io_error() {
        struct Broken;
        impl std::io::Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("broken"))
            }
        }
        let results: Vec<_> = Reader::new(Broken).into_records().take(10).collect();
        assert_eq!(results.len(), 1);
        assert!(matches!(
            results[0].as_ref().unwrap_err().kind(),
            ErrorKind::Io(_)
        ));
    }
}
