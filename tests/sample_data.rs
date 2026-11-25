//! Tests against the minimap2 output in `data/`.

use paf::{PafRecord, Reader, Writer};

const SAMPLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/5_GD_domestica.paf");

fn read_sample() -> Vec<PafRecord> {
    Reader::from_path(SAMPLE)
        .unwrap()
        .into_records()
        .collect::<paf::Result<_>>()
        .unwrap()
}

#[test]
fn reads_every_record_in_the_sample_file() {
    let records = read_sample();
    assert_eq!(records.len(), 20_495);
    assert!(records.iter().all(|r| r.tp().is_some() && r.cg().is_some()));
}

#[test]
fn sample_file_round_trips() {
    let original = read_sample();

    let mut writer = Writer::new(Vec::new());
    for record in &original {
        writer.write_record(record).unwrap();
    }
    let written = writer.into_inner().unwrap();

    let reread: Vec<PafRecord> = Reader::new(&written[..])
        .into_records()
        .collect::<paf::Result<_>>()
        .unwrap();
    assert_eq!(reread, original);

    // The text is unchanged too, apart from minimap2's zero-padded floats
    // (`de:f:0.0110` is written back as `de:f:0.011`, the same value).
    let source = std::fs::read_to_string(SAMPLE).unwrap();
    let written = String::from_utf8(written).unwrap();
    assert_eq!(source.lines().count(), written.lines().count());
    for (before, after) in source.lines().zip(written.lines()) {
        let before: Vec<_> = before.split('\t').collect();
        let after: Vec<_> = after.split('\t').collect();
        assert_eq!(before.len(), after.len());
        for (b, a) in before.iter().zip(&after) {
            match (b.strip_prefix("de:f:"), a.strip_prefix("de:f:")) {
                (Some(b), Some(a)) => {
                    assert_eq!(b.parse::<f64>().unwrap(), a.parse::<f64>().unwrap())
                }
                _ => assert_eq!(b, a),
            }
        }
    }
}
