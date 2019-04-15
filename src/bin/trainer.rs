use stammer::{Engine, Store};

use failure::Error;
use hashbrown::HashSet;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufRead;
use std::io::{BufReader, BufWriter};
use std::iter::FromIterator;
use std::path::Path;

#[derive(Deserialize)]
struct Raw {
    html: String,
    // Ignores other fields
}

struct Scope {
    store: Store,
    chars: HashSet<char>,
}

impl Scope {
    fn new(chars: &str) -> Scope {
        Scope {
            chars: HashSet::from_iter(chars.chars()),
            store: Store::new(),
        }
    }

    fn input(&mut self, s: &str) {
        let mut last_char = None;

        for c in s.chars() {
            if !self.chars.contains(&c) {
                continue;
            }
            self.store.put_single(c);

            if let Some(last) = last_char {
                if self.chars.contains(&last) {
                    self.store.put_pair(last, c);
                }
            }

            last_char = Some(c);
        }
    }

    fn unwrap(self) -> Store {
        self.store
    }
}

fn read_file<P: AsRef<Path>>(path: P, scope: &mut Scope) -> Result<(), Error> {
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
        println!("Reading from {}", entry.path().as_path().display());
        read_file(entry.path(), scope)?;
    }

    println!("Read finished.");
    Ok(())
}

fn main() -> Result<(), Error> {
    println!("Reading chars...");
    let chars = fs::read_to_string("./provided/chars.txt")?;
    println!("Read finished.");

    let mut scope = Scope::new(&chars);
    read_all(Path::new("./provided/data"), &mut scope)?;

    let engine: Engine = scope.unwrap().into();
    let engine_file = File::create("./engine.json")?;
    let mut output = BufWriter::new(engine_file);
    serde_json::to_writer(&mut output, &engine)?;

    Ok(())
}
