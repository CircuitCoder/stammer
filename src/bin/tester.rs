use stammer::{Dict, Engine};

use pinyin;
use std::io::BufRead;
use std::io::BufReader;
use std::fs::File;
use failure::Error;

fn main() -> Result<(), Error> {
    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());

    let dict = Dict::from_file("./provided/dict.txt")?;

    let engine_file = File::open("./engine.json")?;
    let engine_reader = BufReader::new(engine_file);
    let mut engine: Engine = serde_json::from_reader(engine_reader)?;
    engine.init_trie();

    let mut total = 0;
    let mut correct = 0;

    let args = pinyin::Args::new();

    for line in reader.lines() {
        let line = line?;
        total += 1;
        let py = pinyin::lazy_pinyin(&line, &args);

        let result = engine.query(&py, &dict);
        if result == line {
            correct += 1;
        }
    }

    println!("{:5}%: {} / {}", 100 as f64 * correct as f64 / total as f64, correct, total);

    Ok(())
}
