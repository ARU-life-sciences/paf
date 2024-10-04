use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::{Error, ErrorKind, Result};

/// Enum representing the possible types of optional fields.
#[derive(Debug)]
pub enum Type {
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
}

impl Type {
    fn parse(field_type: &str, value: &str) -> Option<Self> {
        match field_type {
            "i" => value.parse::<i64>().ok().map(Type::Int),
            "f" => value.parse::<f64>().ok().map(Type::Float),
            "Z" => Some(Type::String(value.to_string())),
            "A" => value.chars().next().map(Type::Char),
            _ => Some(Type::String(value.to_string())), // Default to string
        }
    }

    /// Get the inner integer out.
    pub fn get_int(&self) -> Option<&i64> {
        match self {
            Type::Int(v) => Some(v),
            _ => None,
        }
    }

    /// Get the inner float out.
    pub fn get_float(&self) -> Option<&f64> {
        match self {
            Type::Float(v) => Some(v),
            _ => None,
        }
    }

    /// Get the inner string out.
    pub fn get_string(&self) -> Option<&String> {
        match self {
            Type::String(v) => Some(v),
            _ => None,
        }
    }

    /// Get the inner char out.
    pub fn get_char(&self) -> Option<&char> {
        match self {
            Type::Char(v) => Some(v),
            _ => None,
        }
    }
}

/// Enum representing the possible types of tags.
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Tag {
    /// Type of aln: P/primary, S/secondary and I,i/inversion.
    tp(Type),
    /// Number of minimizers on the chain.
    cm(Type),
    /// Chaining score.
    s1(Type),
    /// Chaining score of the best secondary chain.
    s2(Type),
    /// Total number of mismatches and gaps in the alignment.
    NM(Type),
    /// To generate the ref sequence in the alignment.
    MD(Type),
    /// DP alignment score.
    AS(Type),
    /// List of other supplementary alignments.
    SA(Type),
    /// DP score of the max scoring segment in the alignment.
    ms(Type),
    /// Number of ambiguous bases in the alignment.
    nn(Type),
    /// Transcript strand (splice mode only).
    ts(Type),
    /// CIGAR string.
    cg(Type),
    /// Difference string.
    cs(Type),
    /// Approximate per-base sequence divergence.
    dv(Type),
    /// Gap-compressed per-base sequence divergence.
    de(Type),
    /// Length of query regions harboring repetitive seeds.
    rl(Type),
    /// ZD?
    zd(Type),
}

impl Tag {
    /// Parse a tag from a string.
    pub fn parse(tag: &str, value: Type) -> Result<Self> {
        match tag {
            "tp" => Ok(Tag::tp(value)),
            "cm" => Ok(Tag::cm(value)),
            "s1" => Ok(Tag::s1(value)),
            "s2" => Ok(Tag::s2(value)),
            "NM" => Ok(Tag::NM(value)),
            "MD" => Ok(Tag::MD(value)),
            "AS" => Ok(Tag::AS(value)),
            "SA" => Ok(Tag::SA(value)),
            "ms" => Ok(Tag::ms(value)),
            "nn" => Ok(Tag::nn(value)),
            "ts" => Ok(Tag::ts(value)),
            "cg" => Ok(Tag::cg(value)),
            "cs" => Ok(Tag::cs(value)),
            "dv" => Ok(Tag::dv(value)),
            "de" => Ok(Tag::de(value)),
            "rl" => Ok(Tag::rl(value)),
            "zd" => Ok(Tag::zd(value)),
            _ => Err(Error::new(ErrorKind::ReadRecord(format!(
                "Invalid PAF tag: {}",
                tag
            )))),
        }
    }

    /// Tag to string function.
    fn to_string(&self) -> String {
        match self {
            Tag::tp(_) => "tp".into(),
            Tag::cm(_) => "cm".into(),
            Tag::s1(_) => "s1".into(),
            Tag::s2(_) => "s2".into(),
            Tag::NM(_) => "NM".into(),
            Tag::MD(_) => "MD".into(),
            Tag::AS(_) => "AS".into(),
            Tag::SA(_) => "SA".into(),
            Tag::ms(_) => "ms".into(),
            Tag::nn(_) => "nn".into(),
            Tag::ts(_) => "ts".into(),
            Tag::cg(_) => "cg".into(),
            Tag::cs(_) => "cs".into(),
            Tag::dv(_) => "dv".into(),
            Tag::de(_) => "de".into(),
            Tag::rl(_) => "rl".into(),
            Tag::zd(_) => "zd".into(),
        }
    }
}

/// Struct representing a PAF record.
#[derive(Debug)]
pub struct PafRecord {
    /// Query sequence name.
    query_name: String,
    /// Query sequence length.
    query_len: u32,
    /// Query start coordinate (0-based).
    query_start: u32,
    /// Query end coordinate (0-based).
    query_end: u32,
    /// ‘+’ if query/target on the same strand; ‘-’ if opposite.
    strand: char,
    /// Target sequence name.
    target_name: String,
    /// Target sequence length.
    target_len: u32,
    /// Target start coordinate on the original strand.
    target_start: u32,
    /// Target end coordinate on the original strand.
    target_end: u32,
    /// Number of matching bases in the mapping.
    residue_matches: u32,
    /// Number bases, including gaps, in the mapping.
    alignment_block_len: u32,
    /// Mapping quality (0-255 with 255 for missing).
    mapping_quality: u8,

    /// The optional fields.
    optional: HashMap<String, Tag>,
}

impl PafRecord {
    /// Get the query name.
    pub fn query_name(&self) -> &str {
        &self.query_name
    }
    /// Get the query length.
    pub fn query_len(&self) -> u32 {
        self.query_len
    }
    /// Get the query start position.
    pub fn query_start(&self) -> u32 {
        self.query_start
    }
    /// Get the query end position.
    pub fn query_end(&self) -> u32 {
        self.query_end
    }
    /// Get the target name.
    pub fn target_name(&self) -> &str {
        &self.target_name
    }
    /// Get the target length.
    pub fn target_len(&self) -> u32 {
        self.target_len
    }
    /// Get the target start position.
    pub fn target_start(&self) -> u32 {
        self.target_start
    }
    /// Get the target end position.
    pub fn target_end(&self) -> u32 {
        self.target_end
    }
    /// Get the number of residue matches.
    pub fn residue_matches(&self) -> u32 {
        self.residue_matches
    }
    /// Get the alignment block length.
    pub fn alignment_block_len(&self) -> u32 {
        self.alignment_block_len
    }
    /// Get the mapping quality.
    pub fn mapping_quality(&self) -> u8 {
        self.mapping_quality
    }
    /// Get the strand.
    pub fn strand(&self) -> char {
        self.strand
    }
    /// Get all the optional fields.
    pub fn optional_fields(&self) -> &HashMap<String, Tag> {
        &self.optional
    }
    /// Get type of aln: P/primary, S/secondary and I,i/inversion.
    pub fn tp(&self) -> Option<&char> {
        self.optional.get("tp").map(|tag| match tag {
            Tag::tp(t) => t.get_char().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get number of minimizers on the chain
    pub fn cm(&self) -> Option<&i64> {
        self.optional.get("cm").map(|tag| match tag {
            Tag::cm(t) => t.get_int().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get chaining score.
    pub fn s1(&self) -> Option<&i64> {
        self.optional.get("s1").map(|tag| match tag {
            Tag::s1(t) => t.get_int().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get chaining score of the best secondary chain.
    pub fn s2(&self) -> Option<&i64> {
        self.optional.get("s2").map(|tag| match tag {
            Tag::s2(t) => t.get_int().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get total number of mismatches and gaps in the alignment.
    pub fn nm(&self) -> Option<&i64> {
        self.optional.get("NM").map(|tag| match tag {
            Tag::NM(t) => t.get_int().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get the ref sequence in the alignment.
    pub fn md(&self) -> Option<&String> {
        self.optional.get("MD").map(|tag| match tag {
            Tag::MD(t) => t.get_string().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get DP alignment score.
    pub fn as_(&self) -> Option<&i64> {
        self.optional.get("AS").map(|tag| match tag {
            Tag::AS(t) => t.get_int().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get a list of other supplementary alignments.
    pub fn sa(&self) -> Option<&String> {
        self.optional.get("SA").map(|tag| match tag {
            Tag::SA(t) => t.get_string().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get DP score of the max scoring segment in the alignment.
    pub fn ms(&self) -> Option<&i64> {
        self.optional.get("ms").map(|tag| match tag {
            Tag::ms(t) => t.get_int().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get number of ambiguous bases in the alignment.
    pub fn nn(&self) -> Option<&i64> {
        self.optional.get("nn").map(|tag| match tag {
            Tag::nn(t) => t.get_int().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get transcript strand (splice mode only).
    pub fn ts(&self) -> Option<&char> {
        self.optional.get("ts").map(|tag| match tag {
            Tag::ts(t) => t.get_char().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get CIGAR string (only in PAF).
    pub fn cg(&self) -> Option<&String> {
        self.optional.get("cg").map(|tag| match tag {
            Tag::cg(t) => t.get_string().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get difference string.
    pub fn cs(&self) -> Option<&String> {
        self.optional.get("cs").map(|tag| match tag {
            Tag::cs(t) => t.get_string().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get approximate per-base sequence divergence.
    pub fn dv(&self) -> Option<&f64> {
        self.optional.get("dv").map(|tag| match tag {
            Tag::dv(t) => t.get_float().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get gap-compressed per-base sequence divergence.
    pub fn de(&self) -> Option<&f64> {
        self.optional.get("de").map(|tag| match tag {
            Tag::de(t) => t.get_float().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
    /// Get length of query regions harboring repetitive seeds.
    pub fn rl(&self) -> Option<&i64> {
        self.optional.get("rl").map(|tag| match tag {
            Tag::rl(t) => t.get_int().unwrap(),
            _ => panic!("Invalid tag"),
        })
    }
}

/// Struct representing a PAF parser iterator.
pub struct Reader<R> {
    reader: io::BufReader<R>,
    line: u64,
}

impl Reader<File> {
    /// Creates a new PAF parser from a file path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Reader<File>> {
        Ok(Reader::new(File::open(path)?))
    }

    /// Creates a new PAF parser from a reader.
    pub fn from_reader<R: io::Read>(rdr: R) -> Reader<R> {
        Reader::new(rdr)
    }
}

/// Parse optional fields from the PAF line.
fn parse_optional_fields(fields: &[&str]) -> Result<HashMap<String, Tag>> {
    let mut map = HashMap::new();

    // NM:i:48730
    for field in fields {
        let parts: Vec<&str> = field.split(':').collect();
        if parts.len() < 3 {
            return Err(Error::new(ErrorKind::ReadRecord(
                "Invalid PAF line: invalid optional field - too few parts".into(),
            )));
        }

        let tag = parts[0];
        let type_ = parts[1];
        let inner = parts[2];

        let type_ = Type::parse(type_, inner).ok_or_else(|| {
            Error::new(ErrorKind::ReadRecord(format!(
                "Invalid PAF line: invalid optional field type: {}",
                type_
            )))
        })?;

        let tag = Tag::parse(tag, type_)?;

        map.insert(tag.to_string(), tag);
    }
    Ok(map)
}

impl<R: io::Read> Reader<R> {
    /// Creates a new PAF parser from a buffered reader.
    pub fn new(rdr: R) -> Self {
        Reader {
            reader: io::BufReader::new(rdr),
            line: 0,
        }
    }

    /// A borrowed iterator over the records of a PAF file.
    pub fn records(&mut self) -> RecordsIter<R> {
        RecordsIter::new(self)
    }

    /// An owned iterator over the records of a PAF file.
    pub fn into_records(self) -> RecordsIntoIter<R> {
        RecordsIntoIter::new(self)
    }

    /// Read a single record.
    pub fn read_record(&mut self) -> Result<Option<PafRecord>> {
        let mut line = String::new();
        let bytes_read = match self.reader.read_line(&mut line) {
            Ok(b) => b,
            Err(e) => return Err(Error::new(ErrorKind::Io(e))),
        };

        if bytes_read == 0 {
            return Ok(None); // EOF
        }

        let columns: Vec<&str> = line.trim().split('\t').collect();
        if columns.len() < 12 {
            return Err(Error::new(ErrorKind::ReadRecord(format!(
                "Invalid PAF at line {}: less than 12 mandatory fields",
                self.line
            ))));
        }

        // parse the mandatory fields
        let query_name = columns[0].to_string();
        let query_len = columns[1].parse::<u32>()?;
        let query_start = columns[2].parse::<u32>()?;
        let query_end = columns[3].parse::<u32>()?;
        let strand = columns[4]
            .chars()
            .next()
            .ok_or_else(|| Error::new(ErrorKind::ReadRecord("Empty strand field".into())))?;

        if strand != '+' && strand != '-' {
            return Err(Error::new(ErrorKind::ReadRecord(format!(
                "Invalid strand field at line {}: {}",
                self.line, strand
            ))));
        }

        let target_name = columns[5].to_string();
        let target_len = columns[6].parse::<u32>()?;
        let target_start = columns[7].parse::<u32>()?;
        let target_end = columns[8].parse::<u32>()?;
        let residue_matches = columns[9].parse::<u32>()?;
        let alignment_block_len = columns[10].parse::<u32>()?;
        let mapping_quality = columns[11].parse::<u8>()?;

        let optional = parse_optional_fields(&columns[12..])?;

        let record = PafRecord {
            query_name,
            query_len,
            query_start,
            query_end,
            strand,
            target_name,
            target_len,
            target_start,
            target_end,
            residue_matches,
            alignment_block_len,
            mapping_quality,
            optional,
        };

        Ok(Some(record))
    }
}

/// A borrowed iterator over the records of a PAF file.
pub struct RecordsIter<'r, R: 'r> {
    /// The underlying reader
    rdr: &'r mut Reader<R>,
}

impl<'r, R: io::Read> RecordsIter<'r, R> {
    /// Create a new iterator.
    fn new(rdr: &'r mut Reader<R>) -> RecordsIter<'r, R> {
        RecordsIter { rdr }
    }
    /// Return a reference to the underlying reader.
    pub fn reader(&self) -> &Reader<R> {
        self.rdr
    }

    /// Return a mutable reference to the underlying reader.
    pub fn reader_mut(&mut self) -> &mut Reader<R> {
        self.rdr
    }
}

impl<'r, R: io::Read> Iterator for RecordsIter<'r, R> {
    type Item = Result<PafRecord>;

    fn next(&mut self) -> Option<Result<PafRecord>> {
        match self.rdr.read_record() {
            Ok(Some(r)) => {
                self.rdr.line += 1;
                Some(Ok(r))
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// An owned iterator over the records of a PAF file.
pub struct RecordsIntoIter<R> {
    /// The underlying reader.
    rdr: Reader<R>,
}

impl<R: io::Read> RecordsIntoIter<R> {
    /// Create a new iterator.
    fn new(rdr: Reader<R>) -> RecordsIntoIter<R> {
        RecordsIntoIter { rdr }
    }
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
        match self.rdr.read_record() {
            Ok(Some(r)) => {
                self.rdr.line += 1;
                Some(Ok(r))
            }
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;

    const PAF_RECORD_1: &[u8] = b"NC_041798.1	41841605	28850796	29394458	+	SUPER_10	44636193	31974877	32470190	495111	515145	60	NM:i:48730	ms:i:488389	AS:i:439775	nn:i:28696	tp:A:P	cm:i:46495	s1:i:466570	s2:i:10896	de:f:0.0003	zd:i:3	rl:i:3568165	cg:Z:770M1D945M1D389M1I9141M1I356M1D196M1I30268M2D789M3I992M2D1819M1D7M1D7M1I10M6D2922M1D17899M2D1010M4D12324M1I1376M1D5549M6D1839M1I2206M1D770M1D2287M1D16103M1D3238M1D2014M1D140M5I14M1D8496M2I2151M1I335M1D14424M1D1093M1I567M1D1835M2D1995M1D5257M1D639M1I699M1I133M1I52M1I99M2I26M1I195M1I1543M1I240M1I176M1I412M2D159M1I261M1D1158M1I933M2D12836M1D993M1D12263M2D4975M2I16452M3I396M1I3924M2D929M3I3015M1D225M1D4225M1D717M2D752M1D2051M1D5110M1D15073M1D1053M2D4369M1D619M3I13564M2I4386M1D1431M2D617M1I612M2I3445M2I252M1D220M1D237M1I903M1I145M1I53M1I197M1I1280M1D4201M1D1736M1D1289M1I3344M2D5456M1D488M1I1655M2D1830M1D796M1I19341M2D1165M1D1926M1D6041M1D2170M1D3917M1D926M1D759M1D400M2I8802M1I836M1I381M48451I166M1I4896M2D1522M49D2729M1D947M2D927M6D911M2D800M2D3040M1D13213M1D8999M3D847M1D220M1I673M1D165M1I901M1I2887M1I105M2I597M1I1201M1I53M2I494M1I23M1D99M1I146M1D29906M1D5661M1I27598M1D520M1I166M2D11600M1D388M1D844M1D4583M1D8390M1D5789M2D3773M1D4494M1D448M1D846M3D531M";

    #[test]
    fn test_read_record() {
        let mut parser = Reader::from_reader(&PAF_RECORD_1[..]);
        let record = parser.read_record().unwrap().unwrap();

        assert_eq!(record.query_name(), "NC_041798.1");
        assert_eq!(record.query_len(), 41841605);
        assert_eq!(record.query_start(), 28850796);
        assert_eq!(record.query_end(), 29394458);
        assert_eq!(record.strand(), '+');
        assert_eq!(record.target_name(), "SUPER_10");
        assert_eq!(record.target_len(), 44636193);
        assert_eq!(record.target_start(), 31974877);
        assert_eq!(record.target_end(), 32470190);
        assert_eq!(record.residue_matches(), 495111);
        assert_eq!(record.alignment_block_len(), 515145);
        assert_eq!(record.mapping_quality(), 60);

        let nm = record.nm().unwrap();
        assert_eq!(nm, &48730);
    }
}
