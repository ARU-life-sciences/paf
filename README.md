# A PAF file reader and writer

Read and write PAF (Pairwise mApping Format) files, as produced by
[minimap2](https://github.com/lh3/minimap2) and other aligners.

- Iterate over records with errors that name the line and column at fault.
- Typed accessors for the tags minimap2 writes (`record.nm()`, `record.tp()`, `record.cs()`, …).
- Any other tag is kept too, with its SAM type, in its original order.
- Records are plain structs: change them and write them back out.

Upgrading from 0.2? See the [changelog](CHANGELOG.md).

## Reading

```rust,no_run
use paf::Reader;

fn main() -> paf::Result<()> {
    let mut reader = Reader::from_path("data/5_GD_domestica.paf")?;
    for record in reader.records() {
        let record = record?;
        println!(
            "{} -> {}: {:?} mismatches and gaps",
            record.query_name,
            record.target_name,
            record.nm()
        );
    }
    Ok(())
}
```

Try it on the sample data in this repository:

```sh
cargo run --release --example print_paf -- data/5_GD_domestica.paf
```

## Writing

```rust,no_run
use paf::{PafRecord, Strand, Tag, Type, Writer};

fn main() -> paf::Result<()> {
    let mut writer = Writer::from_path("example.paf")?;

    let record = PafRecord {
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
        tags: vec![Tag::new("tp", Type::Char('P'))?, "NM:i:12".parse()?],
    };
    writer.write_record(&record)?;

    // Flush explicitly so that a failed final write is reported.
    writer.flush()
}
```

## Filtering and editing

```rust,no_run
use paf::{Reader, Writer};

fn main() -> paf::Result<()> {
    let mut reader = Reader::from_path("input.paf")?;
    let mut writer = Writer::from_path("primary.paf")?;

    for record in reader.records() {
        let mut record = record?;
        if record.tp() != Some('P') {
            continue;
        }
        record.remove_tag("cg");
        writer.write_record(&record)?;
    }
    writer.flush()
}
```
