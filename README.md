# A PAF file parser

Simple iterator API for PAF files.

## Example

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
