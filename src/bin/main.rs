use stammer::{Engine, Dict};

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use failure::Error;
use serde_json;

fn main() -> Result<(), Error> {
    println!("Reading dict...");
    let dict = Dict::from_file("./provided/dict.txt")?;
    println!("Reading engine...");
    let engine_file = File::open("./engine.json")?;
    let engine_reader = BufReader::new(engine_file);
    let engine: Engine = serde_json::from_reader(engine_reader)?;

    let input_file = File::open("./provided/input.txt")?;
    let input_reader = BufReader::new(input_file);
    for line in input_reader.lines() {
        let line = line?;
        println!("Query: {}", line);
        let segs = line.split(' ');
        let result = engine.query(segs, &dict);
        let result_string: String = result.into_iter().collect();
        println!("{}", result_string);
    }

    Ok(())
}
