use serde::Deserialize;
use std::path::Path;
use std::fs::{self, File};
use std::io::BufReader;
use failure::Error;
use std::iter::Extend;
use std::io::BufRead;

#[derive(Deserialize)]
struct Raw {
    html: String,
    // Ignores other fields
}

fn read_input_into<P: AsRef<Path>>(path: P, store: &mut Vec<Raw>) -> Result<(), Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let iter = reader.lines().filter_map(|e| e.ok().and_then(|e| serde_json::from_str(&e).ok())); // Automatically drops malformed inputs
    store.extend(iter);

    Ok(())
}

fn read_all<P: AsRef<Path>>(path: P) -> Result<Vec<Raw>, Error> {
    println!("Reading inputs...");
    let mut result = Vec::new();

    // Assume path is dir
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        println!("Reading from {}", entry.path().as_path().display());
        read_input_into(entry.path(), &mut result)?;
    }

    println!("Read finished.");

    Ok(result)
}

fn main() {
    read_all(Path::new("./provided/data"));
}
