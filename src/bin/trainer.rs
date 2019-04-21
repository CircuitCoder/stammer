#![feature(try_blocks)]
#![feature(custom_attribute)]

use stammer::{Engine, TrainingStore, Raw};

use failure::Error;
use hashbrown::HashSet;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufRead;
use std::io::{BufReader, BufWriter};
use std::iter::FromIterator;
use std::path::Path;
use jieba_rs::Jieba;
use std::collections::VecDeque;
use std::path::PathBuf;

use structopt::StructOpt;

struct Scope {
    store: TrainingStore,
    chars: HashSet<char>,
    jieba: Jieba,
}

const N_GRAM: usize = 3;

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

        let mut store = VecDeque::with_capacity(N_GRAM);
        for _ in 0..N_GRAM {
            store.push_back(None);
        }

        for s in segs.iter() {
            store.pop_front();
            if s.chars().all(|ref c| self.chars.contains(c)) {
                store.push_back(Some(s.to_owned()));
            } else {
                store.push_back(None);
            }

            self.store.add_tuple(store.iter());
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
        let _: Result<(), Error> = try {
            let line = line?;
            let raw = if line.chars().next() == Some('{') {
                serde_json::from_str(&line)?
            } else {
                Raw::Plain(line)
            };
            scope.input(&raw.to_string());
        };
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

#[derive(StructOpt, Debug)]
struct Opts {
    /// Path to the allowed character file
    #[structopt(short = "c", parse(from_os_str), default_value = "./data/chars.txt")]
    chars: PathBuf,

    /// Path to the directory containing all training data
    #[structopt(short = "d", parse(from_os_str), default_value = "./provided/data")]
    data: PathBuf,

    /// Output path for the engine state file
    #[structopt(short = "o", parse(from_os_str), default_value = "./data/engine.json")]
    output: PathBuf,
}

fn main() -> Result<(), Error> {
    let opts = Opts::from_args();

    println!("Reading chars...");
    let chars = fs::read_to_string(&opts.chars)?;
    println!("Read finished.");

    let mut scope = Scope::new(&chars);
    read_all(Path::new(&opts.data), &mut scope)?;

    let engine: Engine = scope.unwrap().extract(WORDING_SIZE);
    let engine_file = File::create(&opts.output)?;
    let mut output = BufWriter::new(engine_file);

    println!("Serializing engine into {}", &opts.output.display());
    serde_json::to_writer(&mut output, &engine)?;

    Ok(())
}
