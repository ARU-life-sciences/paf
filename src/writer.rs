use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::{PafRecord, Result, Tag, Type};

/// Struct representing a PAF file writer.
pub struct Writer<W: Write> {
    writer: W,
}

impl Writer<File> {
    /// Creates a new PAF writer from a file path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Writer<File>> {
        let file = File::create(path)?;
        Ok(Writer::new(file))
    }
}

impl<W: Write> Writer<W> {
    /// Creates a new PAF writer from a writer instance.
    pub fn new(writer: W) -> Self {
        Writer { writer }
    }

    /// Writes a single `PafRecord` to the PAF file.
    pub fn write_record(&mut self, record: &PafRecord) -> Result<()> {
        write!(
            self.writer,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            record.query_name(),
            record.query_len(),
            record.query_start(),
            record.query_end(),
            record.strand(),
            record.target_name(),
            record.target_len(),
            record.target_start(),
            record.target_end(),
            record.residue_matches(),
            record.alignment_block_len(),
            record.mapping_quality(),
        )?;

        for (key, tag) in record.optional_fields() {
            match tag {
                Tag::tp(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::cm(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::s1(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::s2(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::NM(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::MD(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::AS(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::SA(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::ms(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::nn(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::ts(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::cg(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::cs(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::dv(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::de(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::rl(value) => write_optional_field(&mut self.writer, key, value)?,
                Tag::zd(value) => write_optional_field(&mut self.writer, key, value)?,
            }
        }

        writeln!(self.writer).map_err(Into::into)
    }
}

/// Helper function to write optional fields based on their types.
fn write_optional_field<W: Write>(writer: &mut W, tag: &str, value: &Type) -> Result<()> {
    match value {
        Type::Int(v) => write!(writer, "\t{}:i:{}", tag, v).map_err(Into::into),
        Type::Float(v) => write!(writer, "\t{}:f:{:.4}", tag, v).map_err(Into::into),
        Type::String(v) => write!(writer, "\t{}:Z:{}", tag, v).map_err(Into::into),
        Type::Char(v) => write!(writer, "\t{}:A:{}", tag, v).map_err(Into::into),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{PafRecord, Tag, Type};

    #[test]
    fn test_write_record_mandatory_fields() {
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer);

        let record = PafRecord::new(
            "query1".to_owned(),
            1000,
            100,
            500,
            '+',
            "target1".to_owned(),
            1500,
            200,
            600,
            300,
            400,
            60,
            HashMap::new(),
        );

        writer.write_record(&record).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert_eq!(
            output,
            "query1\t1000\t100\t500\t+\ttarget1\t1500\t200\t600\t300\t400\t60\n"
        );
    }

    #[test]
    fn test_write_record_with_optional_fields() {
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer);

        let mut optional_fields = HashMap::new();

        optional_fields.insert("tp".to_string(), Tag::tp(Type::Char('P')));
        optional_fields.insert("cm".to_string(), Tag::cm(Type::Int(42)));
        optional_fields.insert("s1".to_string(), Tag::s1(Type::Int(99)));

        let record = PafRecord::new(
            "query2".to_owned(),
            2000,
            150,
            900,
            '-',
            "target2".to_owned(),
            2500,
            300,
            1000,
            400,
            800,
            70,
            optional_fields,
        );

        writer.write_record(&record).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // as the optional fields are a map, these are printed in an arbitrary order
        // so we have to check the output differently

        assert!(
            output.contains("query2\t2000\t150\t900\t-\ttarget2\t2500\t300\t1000\t400\t800\t70")
        );
        assert!(output.contains("\ttp:A:P"));
        assert!(output.contains("\tcm:i:42"));
        assert!(output.contains("\ts1:i:99"));
    }
}
