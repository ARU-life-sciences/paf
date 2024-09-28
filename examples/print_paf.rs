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
        eprintln!("Usage: print_paf <filename>");
        std::process::exit(1);
    }
}
