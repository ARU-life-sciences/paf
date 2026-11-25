//! Write two records to `example.paf`.
//!
//! cargo run --example write_paf

use paf::{PafRecord, Result, Strand, Tag, Type, Writer};

fn main() -> Result<()> {
    let mut writer = Writer::from_path("example.paf")?;

    let record1 = PafRecord {
        query_name: "query1".into(),
        query_len: 1000,
        query_start: 100,
        query_end: 500,
        strand: Strand::Forward,
        target_name: "target1".into(),
        target_len: 1500,
        target_start: 200,
        target_end: 600,
        residue_matches: 300,
        alignment_block_len: 400,
        mapping_quality: 60,
        tags: vec![Tag::new("tp", Type::Char('P'))?],
    };

    let record2 = PafRecord {
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
        // Tags can also be parsed from their PAF text.
        tags: vec!["s1:i:99".parse()?, "cm:i:42".parse()?],
    };

    writer.write_record(&record1)?;
    writer.write_record(&record2)?;

    // Flush explicitly so that a failed final write is reported.
    writer.flush()
}
