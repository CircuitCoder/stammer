use stammer::{Engine, TrainingStore};

use failure::Error;
use hashbrown::HashSet;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufRead;
use std::io::{BufReader, BufWriter};
use std::iter::FromIterator;
use std::path::Path;
use jieba_rs::Jieba;

#[derive(Deserialize)]
struct Raw {
    html: String,
    // Ignores other fields
}

struct Scope {
    store: TrainingStore,
    chars: HashSet<char>,
    jieba: Jieba,
}

impl Scope {
    fn new(chars: &str) -> Scope {
        Scope {
            chars: HashSet::from_iter(chars.chars()),
            store: TrainingStore::new(),
            jieba: Jieba::new(),
        }
    }

    fn input(&mut self, s: &str) {
        let segs = self.jieba.cut(s, false); // No HMM for consistent wording

        let mut last: Option<&str> = None;

        for s in segs.iter() {
            if s.chars().all(|ref c| self.chars.contains(c)) {
                if let Some(l) = last {
                    self.store.add_pair((*s).to_owned(), l.to_owned());
                }

                last = Some(s);
            } else {
                last = None;
            }
        }
    }

    fn unwrap(self) -> TrainingStore {
        self.store
    }
}

fn read_file<P: AsRef<Path>>(path: P, scope: &mut Scope) -> Result<(), Error> {
    println!("Reading from {}", path.as_ref().display());
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let raw: Raw = serde_json::from_str(&line)?;
        scope.input(&raw.html);
    }

    Ok(())
}

fn read_all<P: AsRef<Path>>(path: P, scope: &mut Scope) -> Result<(), Error> {
    println!("Reading inputs...");
    // Assume path is dir
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        read_file(entry.path(), scope)?;
    }

    println!("Read finished.");
    Ok(())
}

const WORDING_SIZE: usize = 50000000;

fn main() -> Result<(), Error> {
    println!("Reading chars...");
    let chars = fs::read_to_string("./provided/chars.txt")?;
    println!("Read finished.");

    let mut scope = Scope::new(&chars);
    read_all(Path::new("./provided/data"), &mut scope)?;

    let engine: Engine = scope.unwrap().extract(WORDING_SIZE);
    let engine_file = File::create("./engine.json")?;
    let mut output = BufWriter::new(engine_file);
    serde_json::to_writer(&mut output, &engine)?;

    Ok(())
}
