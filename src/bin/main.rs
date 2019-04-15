use stammer::{Engine, Dict};

use std::str::Split;
use std::fs::File;
use std::io::BufReader;
use failure::Error;
use serde_json;

fn main() -> Result<(), Error> {
    println!("Reading dict...");
    let dict = Dict::from_file("./provided/dict.txt")?;
    println!("Reading engine...");
    let file = File::open("./engine.json")?;
    let reader = BufReader::new(file);
    let engine: Engine = serde_json::from_reader(reader)?;

    println!("\n{:?}", engine.query(&["ni", "shuo", "shen", "me"], &dict));
    println!("\n{:?}", engine.query(&["qing", "hua", "da", "xue", "ji", "suan", "ji", "xi"], &dict));

    Ok(())
}
