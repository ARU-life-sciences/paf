use std::fmt;
use std::mem;
use std::str::FromStr;

use crate::{Error, ErrorKind, Result};

/// The relative strand of the query and the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Strand {
    /// `+`: query and target are on the same strand.
    #[default]
    Forward,
    /// `-`: query and target are on opposite strands.
    Reverse,
}

impl Strand {
    /// Returns `'+'` or `'-'`.
    pub fn as_char(self) -> char {
        match self {
            Strand::Forward => '+',
            Strand::Reverse => '-',
        }
    }
}

impl fmt::Display for Strand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Strand::Forward => "+",
            Strand::Reverse => "-",
        })
    }
}

impl FromStr for Strand {
    type Err = Error;

    /// Parses `+` or `-`.
    fn from_str(s: &str) -> Result<Strand> {
        match s {
            "+" => Ok(Strand::Forward),
            "-" => Ok(Strand::Reverse),
            _ => Err(Error::new(ErrorKind::ReadRecord(format!(
                "expected '+' or '-', found {s:?}"
            )))),
        }
    }
}

/// The value of an optional field, with its SAM type.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// `A`: a single character.
    Char(char),
    /// `i`: a signed integer.
    Int(i64),
    /// `f`: a floating-point number.
    Float(f64),
    /// `Z`: a string.
    String(String),
    /// `H`: a hex-encoded byte array, kept as written.
    Hex(String),
    /// `B`: a numeric array, kept as written (for example `C,1,2,3`).
    Array(String),
}

impl Type {
    /// Parses `value` as the SAM type named by `type_code`.
    fn parse(type_code: &str, value: &str) -> Result<Type> {
        Ok(match type_code {
            "A" => {
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => Type::Char(c),
                    _ => {
                        return Err(Error::new(ErrorKind::InvalidTag(format!(
                            "type A needs exactly one character, found {value:?}"
                        ))))
                    }
                }
            }
            "i" => Type::Int(value.parse()?),
            "f" => Type::Float(value.parse()?),
            "Z" => Type::String(value.to_owned()),
            "H" => Type::Hex(value.to_owned()),
            "B" => Type::Array(value.to_owned()),
            _ => {
                return Err(Error::new(ErrorKind::InvalidTag(format!(
                    "unknown type {type_code:?}, expected one of A, i, f, Z, H, B"
                ))))
            }
        })
    }

    /// The SAM type code: `A`, `i`, `f`, `Z`, `H` or `B`.
    pub fn type_code(&self) -> char {
        match self {
            Type::Char(_) => 'A',
            Type::Int(_) => 'i',
            Type::Float(_) => 'f',
            Type::String(_) => 'Z',
            Type::Hex(_) => 'H',
            Type::Array(_) => 'B',
        }
    }

    /// The integer, if this is a [`Type::Int`].
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Type::Int(v) => Some(*v),
            _ => None,
        }
    }

    /// The float, if this is a [`Type::Float`].
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Type::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// The string, if this is a [`Type::String`].
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Type::String(v) => Some(v),
            _ => None,
        }
    }

    /// The character, if this is a [`Type::Char`].
    pub fn as_char(&self) -> Option<char> {
        match self {
            Type::Char(v) => Some(*v),
            _ => None,
        }
    }
}

/// Formats the value alone, without the tag name or type code.
///
/// Floats are written in the shortest form that parses back to the same
/// value, so no precision is lost.
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Char(v) => write!(f, "{v}"),
            Type::Int(v) => write!(f, "{v}"),
            Type::Float(v) => write!(f, "{v}"),
            Type::String(v) | Type::Hex(v) | Type::Array(v) => f.write_str(v),
        }
    }
}

/// An optional field such as `NM:i:5`: a name and a typed value.
///
/// Any name is accepted, not only the tags minimap2 writes, so files from
/// other tools (or minimap2 run with `-y`) can be read and written back.
///
/// ```
/// use paf::{Tag, Type};
///
/// let tag: Tag = "cs:Z::303*cg:51".parse()?;
/// assert_eq!(tag.name(), "cs");
/// assert_eq!(tag.value(), &Type::String(":303*cg:51".into()));
///
/// let tag = Tag::new("tp", Type::Char('P'))?;
/// assert_eq!(tag.to_string(), "tp:A:P");
/// # Ok::<(), paf::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    name: String,
    value: Type,
}

impl Tag {
    /// Creates a tag. Fails if `name` is empty or contains `:` or whitespace.
    pub fn new(name: impl Into<String>, value: Type) -> Result<Tag> {
        let name = name.into();
        check_tag_name(&name)?;
        Ok(Tag { name, value })
    }

    /// The tag name, such as `NM`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The tag value.
    pub fn value(&self) -> &Type {
        &self.value
    }

    /// A mutable reference to the tag value.
    pub fn value_mut(&mut self) -> &mut Type {
        &mut self.value
    }

    /// Consumes the tag and returns its value.
    pub fn into_value(self) -> Type {
        self.value
    }
}

fn check_tag_name(name: &str) -> Result<()> {
    if name.is_empty() || name.contains(|c: char| c == ':' || c.is_whitespace()) {
        return Err(Error::new(ErrorKind::InvalidTag(format!(
            "name must be non-empty with no ':' or whitespace, found {name:?}"
        ))));
    }
    Ok(())
}

/// Formats the tag as `NAME:TYPE:VALUE`.
impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.name, self.value.type_code(), self.value)
    }
}

impl FromStr for Tag {
    type Err = Error;

    /// Parses `NAME:TYPE:VALUE`. The value may itself contain `:`.
    fn from_str(s: &str) -> Result<Tag> {
        let mut parts = s.splitn(3, ':');
        let (Some(name), Some(type_code), Some(value)) = (parts.next(), parts.next(), parts.next())
        else {
            return Err(Error::new(ErrorKind::InvalidTag(
                "expected NAME:TYPE:VALUE".into(),
            )));
        };
        check_tag_name(name)?;
        let value = Type::parse(type_code, value).map_err(|e| e.with_field(name))?;
        Ok(Tag {
            name: name.to_owned(),
            value,
        })
    }
}

/// One PAF line: the 12 mandatory columns followed by optional tags.
///
/// The fields are public, so a record can be built with a struct literal
/// and changed in place before being written back out.
///
/// ```
/// use paf::{PafRecord, Strand};
///
/// let record = PafRecord {
///     query_name: "read1".into(),
///     query_len: 1000,
///     query_start: 100,
///     query_end: 500,
///     strand: Strand::Forward,
///     target_name: "chr1".into(),
///     target_len: 1500,
///     target_start: 200,
///     target_end: 600,
///     residue_matches: 300,
///     alignment_block_len: 400,
///     mapping_quality: 60,
///     tags: vec!["NM:i:12".parse()?, "tp:A:P".parse()?],
/// };
/// assert_eq!(record.nm(), Some(12));
/// assert_eq!(
///     record.to_string(),
///     "read1\t1000\t100\t500\t+\tchr1\t1500\t200\t600\t300\t400\t60\tNM:i:12\ttp:A:P"
/// );
/// # Ok::<(), paf::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PafRecord {
    /// Query sequence name.
    pub query_name: String,
    /// Query sequence length.
    pub query_len: u64,
    /// Query start coordinate (0-based).
    pub query_start: u64,
    /// Query end coordinate (0-based, exclusive).
    pub query_end: u64,
    /// Whether the query and target are on the same strand.
    pub strand: Strand,
    /// Target sequence name.
    pub target_name: String,
    /// Target sequence length.
    pub target_len: u64,
    /// Target start coordinate on the original strand (0-based).
    pub target_start: u64,
    /// Target end coordinate on the original strand (0-based, exclusive).
    pub target_end: u64,
    /// Number of matching bases in the mapping.
    pub residue_matches: u64,
    /// Number of bases, including gaps, in the mapping.
    pub alignment_block_len: u64,
    /// Mapping quality (0-255, with 255 meaning missing).
    pub mapping_quality: u8,
    /// Optional fields, in the order they appear on the line.
    pub tags: Vec<Tag>,
}

impl PafRecord {
    /// The first tag called `name`, if there is one.
    pub fn tag(&self, name: &str) -> Option<&Tag> {
        self.tags.iter().find(|t| t.name == name)
    }

    /// A mutable reference to the first tag called `name`, if there is one.
    pub fn tag_mut(&mut self, name: &str) -> Option<&mut Tag> {
        self.tags.iter_mut().find(|t| t.name == name)
    }

    /// Replaces the first tag with the same name as `tag` and returns the old
    /// one, or appends `tag` if there is no such tag.
    pub fn set_tag(&mut self, tag: Tag) -> Option<Tag> {
        match self.tag_mut(&tag.name) {
            Some(existing) => Some(mem::replace(existing, tag)),
            None => {
                self.tags.push(tag);
                None
            }
        }
    }

    /// Removes and returns the first tag called `name`.
    pub fn remove_tag(&mut self, name: &str) -> Option<Tag> {
        let index = self.tags.iter().position(|t| t.name == name)?;
        Some(self.tags.remove(index))
    }

    fn tag_value(&self, name: &str) -> Option<&Type> {
        self.tag(name).map(Tag::value)
    }

    // Typed accessors for the tags minimap2 writes. Each returns `None` if
    // the tag is missing or holds a value of a different type.

    /// `tp`: type of alignment: `P` primary, `S` secondary, `I`/`i` inversion.
    pub fn tp(&self) -> Option<char> {
        self.tag_value("tp")?.as_char()
    }
    /// `cm`: number of minimizers on the chain.
    pub fn cm(&self) -> Option<i64> {
        self.tag_value("cm")?.as_int()
    }
    /// `s1`: chaining score.
    pub fn s1(&self) -> Option<i64> {
        self.tag_value("s1")?.as_int()
    }
    /// `s2`: chaining score of the best secondary chain.
    pub fn s2(&self) -> Option<i64> {
        self.tag_value("s2")?.as_int()
    }
    /// `NM`: total number of mismatches and gaps in the alignment.
    pub fn nm(&self) -> Option<i64> {
        self.tag_value("NM")?.as_int()
    }
    /// `MD`: string for regenerating the reference sequence in the alignment.
    pub fn md(&self) -> Option<&str> {
        self.tag_value("MD")?.as_str()
    }
    /// `AS`: DP alignment score.
    pub fn as_(&self) -> Option<i64> {
        self.tag_value("AS")?.as_int()
    }
    /// `SA`: list of other supplementary alignments.
    pub fn sa(&self) -> Option<&str> {
        self.tag_value("SA")?.as_str()
    }
    /// `ms`: DP score of the max scoring segment in the alignment.
    pub fn ms(&self) -> Option<i64> {
        self.tag_value("ms")?.as_int()
    }
    /// `nn`: number of ambiguous bases in the alignment.
    pub fn nn(&self) -> Option<i64> {
        self.tag_value("nn")?.as_int()
    }
    /// `ts`: transcript strand (splice mode only).
    pub fn ts(&self) -> Option<char> {
        self.tag_value("ts")?.as_char()
    }
    /// `cg`: CIGAR string.
    pub fn cg(&self) -> Option<&str> {
        self.tag_value("cg")?.as_str()
    }
    /// `cs`: difference string.
    pub fn cs(&self) -> Option<&str> {
        self.tag_value("cs")?.as_str()
    }
    /// `ds`: difference string with indel annotations (minimap2 `--ds`).
    pub fn ds(&self) -> Option<&str> {
        self.tag_value("ds")?.as_str()
    }
    /// `dv`: approximate per-base sequence divergence.
    pub fn dv(&self) -> Option<f64> {
        self.tag_value("dv")?.as_float()
    }
    /// `de`: gap-compressed per-base sequence divergence.
    pub fn de(&self) -> Option<f64> {
        self.tag_value("de")?.as_float()
    }
    /// `rl`: length of query regions harboring repetitive seeds.
    pub fn rl(&self) -> Option<i64> {
        self.tag_value("rl")?.as_int()
    }
}

// Getters kept so code written against 0.2 still compiles.
#[allow(missing_docs)]
impl PafRecord {
    #[deprecated(since = "0.3.0", note = "use the `query_name` field")]
    pub fn query_name(&self) -> &str {
        &self.query_name
    }
    #[deprecated(since = "0.3.0", note = "use the `query_len` field")]
    pub fn query_len(&self) -> u64 {
        self.query_len
    }
    #[deprecated(since = "0.3.0", note = "use the `query_start` field")]
    pub fn query_start(&self) -> u64 {
        self.query_start
    }
    #[deprecated(since = "0.3.0", note = "use the `query_end` field")]
    pub fn query_end(&self) -> u64 {
        self.query_end
    }
    #[deprecated(since = "0.3.0", note = "use the `strand` field")]
    pub fn strand(&self) -> Strand {
        self.strand
    }
    #[deprecated(since = "0.3.0", note = "use the `target_name` field")]
    pub fn target_name(&self) -> &str {
        &self.target_name
    }
    #[deprecated(since = "0.3.0", note = "use the `target_len` field")]
    pub fn target_len(&self) -> u64 {
        self.target_len
    }
    #[deprecated(since = "0.3.0", note = "use the `target_start` field")]
    pub fn target_start(&self) -> u64 {
        self.target_start
    }
    #[deprecated(since = "0.3.0", note = "use the `target_end` field")]
    pub fn target_end(&self) -> u64 {
        self.target_end
    }
    #[deprecated(since = "0.3.0", note = "use the `residue_matches` field")]
    pub fn residue_matches(&self) -> u64 {
        self.residue_matches
    }
    #[deprecated(since = "0.3.0", note = "use the `alignment_block_len` field")]
    pub fn alignment_block_len(&self) -> u64 {
        self.alignment_block_len
    }
    #[deprecated(since = "0.3.0", note = "use the `mapping_quality` field")]
    pub fn mapping_quality(&self) -> u8 {
        self.mapping_quality
    }
}

/// Formats the record as one PAF line, without a trailing newline.
impl fmt::Display for PafRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.query_name,
            self.query_len,
            self.query_start,
            self.query_end,
            self.strand,
            self.target_name,
            self.target_len,
            self.target_start,
            self.target_end,
            self.residue_matches,
            self.alignment_block_len,
            self.mapping_quality,
        )?;
        for tag in &self.tags {
            write!(f, "\t{tag}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_round_trip_through_text_for_every_type() {
        for text in [
            "tp:A:P",
            "NM:i:-42",
            "de:f:0.00012345",
            "cs:Z::303*cg:51",
            "XH:H:1AE301",
            "ML:B:C,1,2,3",
            "RG:Z:",
        ] {
            let tag: Tag = text.parse().unwrap();
            assert_eq!(tag.to_string(), text);
        }
    }

    #[test]
    fn malformed_tags_are_rejected() {
        let kind = |s: &str| s.parse::<Tag>().unwrap_err().into_kind();
        assert!(matches!(kind("NM"), ErrorKind::InvalidTag(_)));
        assert!(matches!(kind("NM:i"), ErrorKind::InvalidTag(_)));
        assert!(matches!(kind(":i:5"), ErrorKind::InvalidTag(_)));
        assert!(matches!(kind("NM:Q:5"), ErrorKind::InvalidTag(_)));
        assert!(matches!(kind("tp:A:"), ErrorKind::InvalidTag(_)));
        assert!(matches!(kind("tp:A:PS"), ErrorKind::InvalidTag(_)));
        assert!(matches!(kind("NM:i:abc"), ErrorKind::Int(_)));
        assert!(matches!(kind("de:f:x"), ErrorKind::Float(_)));

        let err = "NM:i:abc".parse::<Tag>().unwrap_err();
        assert_eq!(err.field(), Some("NM"));
    }

    #[test]
    fn tag_new_validates_the_name() {
        assert!(Tag::new("NM", Type::Int(1)).is_ok());
        assert!(Tag::new("", Type::Int(1)).is_err());
        assert!(Tag::new("N:M", Type::Int(1)).is_err());
        assert!(Tag::new("N M", Type::Int(1)).is_err());
        assert!(Tag::new("N\tM", Type::Int(1)).is_err());
    }

    #[test]
    fn typed_accessors_return_none_for_the_wrong_type() {
        let record = PafRecord {
            tags: vec!["tp:i:5".parse().unwrap(), "NM:Z:x".parse().unwrap()],
            ..Default::default()
        };
        assert_eq!(record.tp(), None);
        assert_eq!(record.nm(), None);
        assert_eq!(record.tag("tp").unwrap().value(), &Type::Int(5));
    }

    #[test]
    fn set_and_remove_tags() {
        let mut record = PafRecord {
            tags: vec!["NM:i:1".parse().unwrap(), "tp:A:P".parse().unwrap()],
            ..Default::default()
        };

        let old = record.set_tag(Tag::new("NM", Type::Int(2)).unwrap());
        assert_eq!(old.unwrap().value(), &Type::Int(1));
        assert_eq!(record.nm(), Some(2));
        assert_eq!(record.tags[0].name(), "NM", "replaced in place");

        assert!(record.set_tag("cm:i:7".parse().unwrap()).is_none());
        assert_eq!(record.tags.last().unwrap().name(), "cm");

        assert_eq!(record.remove_tag("tp").unwrap().name(), "tp");
        assert!(record.remove_tag("tp").is_none());
        let names: Vec<_> = record.tags.iter().map(Tag::name).collect();
        assert_eq!(names, ["NM", "cm"]);
    }

    #[test]
    fn strand_parses_and_displays() {
        assert_eq!("+".parse::<Strand>().unwrap(), Strand::Forward);
        assert_eq!("-".parse::<Strand>().unwrap(), Strand::Reverse);
        assert!("*".parse::<Strand>().is_err());
        assert!("".parse::<Strand>().is_err());
        assert_eq!(Strand::Reverse.to_string(), "-");
        assert_eq!(Strand::Forward.as_char(), '+');
    }
}
