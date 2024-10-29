use paf::{PafRecord, Result, Tag, Type, Writer};
use std::collections::HashMap;

fn main() -> Result<()> {
    // Create a new PAF file writer
    let mut writer = Writer::from_path("example.paf")?;

    // Create some fake PAF records
    let mut optional_fields1 = HashMap::new();
    optional_fields1.insert("tp".to_string(), Tag::tp(Type::Char('P')));
    let record1 = PafRecord::new(
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
        optional_fields1,
    );

    let mut optional_fields2 = HashMap::new();
    optional_fields2.insert("s1".to_string(), Tag::s1(Type::Int(99)));
    optional_fields2.insert("cm".to_string(), Tag::cm(Type::Int(42)));
    let record2 = PafRecord::new(
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
        optional_fields2,
    );

    // Write records to the PAF file
    writer.write_record(&record1)?;
    writer.write_record(&record2)?;

    Ok(())
}
