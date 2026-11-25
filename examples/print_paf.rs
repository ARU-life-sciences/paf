//! Print every record in a PAF file.
//!
//! cargo run --release --example print_paf -- data/5_GD_domestica.paf

use paf::Reader;
use std::{env, process};

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("Usage: cargo run --release --example print_paf -- <file.paf>");
        process::exit(1);
    };
    if let Err(err) = run(&path) {
        eprintln!("{path}: {err}");
        process::exit(1);
    }
}

fn run(path: &str) -> paf::Result<()> {
    let mut reader = Reader::from_path(path)?;
    for record in reader.records() {
        println!("{:?}", record?);
    }
    Ok(())
}
