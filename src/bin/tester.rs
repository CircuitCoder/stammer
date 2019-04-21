#![feature(try_blocks)]

use stammer::{Dict, Engine, Raw};

use hashbrown::HashSet;
use std::iter::FromIterator;
use pinyin;
use std::io::BufRead;
use std::io::BufReader;
use std::fs::File;
use failure::Error;
use jieba_rs::Jieba;

const TEST_SIZE: usize = 10000;
const STEP: usize = 100;
const MAX_STEN_WORD: usize = 6;

fn main() -> Result<(), Error> {
    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());

    let dict = Dict::from_file("./data/dict.txt")?;

    let chars_str = std::fs::read_to_string("./data/chars.txt")?;
    let chars: HashSet<char> = HashSet::from_iter(chars_str.chars());

    let engine_file = File::open("./data/engine.json")?;
    let engine_reader = BufReader::new(engine_file);
    let mut engine: Engine = serde_json::from_reader(engine_reader)?;
    engine.init_trie();

    let mut total = 0;
    let mut correct = 0;

    let args = pinyin::Args::new();

    let jieba = Jieba::new();

    'outer: for line in reader.lines() {
        let _: Result<(), Error> = try {
            let line = line?;
            let raw: Raw = serde_json::from_str(&line)?;
            let raw = raw.to_string();

            let mut segs = jieba.cut(&raw, false);
            segs.truncate(MAX_STEN_WORD);
            let raw = segs.join("");

            for sub in raw.split(|c: char| !c.is_ascii_alphanumeric() && !chars.contains(&c)) {
                if sub.chars().any(|c| c.is_ascii_alphanumeric()) {
                    continue; // Skip any alphanumeric chars
                }

                if sub == "" { continue; }
                total += 1;
                let mut py = pinyin::lazy_pinyin(&sub.to_string(), &args);

                for seg in py.iter_mut() {
                    if seg == "lve" {
                        *seg = "lue".to_owned();
                    }

                    if seg == "nve" {
                        *seg = "nue".to_owned();
                    }
                }

                let result = engine.query(&py, &dict);
                if result == sub {
                    correct += 1;
                }

                if total >= TEST_SIZE {
                    break 'outer;
                }

                if total % STEP == 0 {
                    println!("{:5}%: {} / {}", 100 as f64 * correct as f64 / total as f64, correct, total);
                }
            }
        };
    }

    println!("{:5}%: {} / {}", 100 as f64 * correct as f64 / total as f64, correct, total);

    Ok(())
}
