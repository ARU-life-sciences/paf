# A PAF file reader and writer

Simple iterator API for PAF files, along with functionality to write PAF files.

## Examples

Run this example with `cargo run --release --examples ./data/5_GD_domestica.paf`. The data is in this repository.

```rust
use paf::Reader;
use std::env;

fn main() {
    let data = env::args().skip(1).next();

    if let Some(filename) = data {
        let mut reader = Reader::from_path(&filename).unwrap();
        for record in reader.records() {
            let record = record.unwrap();
            println!("{:?}", record);
        }
    } else {
        eprintln!("Usage: cargo run --release --examples print_paf <filename>");
        std::process::exit(1);
    }
}
```

Write PAF files using this API.

```rust
use std::collections::HashMap;
use crate::{Writer, PafRecord, Tag, Type, Result};

fn main() -> Result<()> {
    // Create a new PAF file writer
    let mut writer = Writer::from_path("example.paf")?;

    // Create some fake PAF records
    let mut optional_fields1 = HashMap::new();
    optional_fields1.insert("tp".to_string(), Tag::tp(Type::Char('P')));
    let record1 = PafRecord::new(
        "query1".to_owned(), 1000, 100, 500, '+',
        "target1".to_owned(), 1500, 200, 600,
        300, 400, 60,
        optional_fields1,
    );

    let mut optional_fields2 = HashMap::new();
    optional_fields2.insert("s1".to_string(), Tag::s1(Type::Int(99)));
    optional_fields2.insert("cm".to_string(), Tag::cm(Type::Int(42)));
    let record2 = PafRecord::new(
        "query2".to_owned(), 2000, 150, 900, '-',
        "target2".to_owned(), 2500, 300, 1000,
        400, 800, 70,
        optional_fields2,
    );

    // Write records to the PAF file
    writer.write_record(&record1)?;
    writer.write_record(&record2)?;

    Ok(())
}
```
