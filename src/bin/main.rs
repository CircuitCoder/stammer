#![feature(custom_attribute)]

use stammer::{Engine, Dict};

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::io::BufRead;
use failure::Error;
use serde_json;
use structopt::StructOpt;
use std::path::PathBuf;
use std::io::Write;

#[derive(StructOpt, Debug)]
struct Opts {
    /// Path to the pinyin-character dictionary
    #[structopt(short="d", parse(from_os_str), default_value="./provided/dict.txt")]
    dict: PathBuf,

    /// Path to the engine state file
    #[structopt(short="e", parse(from_os_str), default_value="./engine.json")]
    engine: PathBuf,

    /// Path to the input file, omitted to read from STDIN
    #[structopt(name="INPUT", parse(from_os_str))]
    input: Option<PathBuf>,

    /// Path to the output file, omitted to print to STDOUT
    #[structopt(short="o", parse(from_os_str))]
    output: Option<PathBuf>,

    /// Quiet
    #[structopt(short="q")]
    quiet: bool,
}

fn main() -> Result<(), Error> {
    let opts = Opts::from_args();

    if !opts.quiet { println!("Reading dict..."); }
    let dict = Dict::from_file(&opts.dict)?;
    if !opts.quiet { println!("Reading engine..."); }
    let engine_file = File::open(&opts.engine)?;
    let engine_reader = BufReader::new(engine_file);
    let engine: Engine = serde_json::from_reader(engine_reader)?;

    let stdin = std::io::stdin();
    let input_reader: Box<dyn BufRead> = match opts.input {
        Some(ref p) => Box::new(BufReader::new(File::open(p)?)),
        None => Box::new(stdin.lock()),
    };

    if !opts.quiet && opts.input.is_none() { println!("Reading from STDIN..."); }
    if opts.input.is_none() { print!("> "); std::io::stdout().flush()?; }

    let mut output_file = match opts.output {
        Some(ref p) => Some(BufWriter::new(File::create(p)?)),
        None => None,
    };

    for line in input_reader.lines() {
        let line = line?;
        if !opts.quiet && opts.input.is_some() { println!("In:  {}", line); }
        let segs = line.split(' ');
        let result = engine.query(segs, &dict);
        let result_string: String = result.into_iter().collect();

        if !opts.quiet { println!("Out: {}", result_string); }
        else if opts.output.is_none() { println!("{}", result_string); }

        if let Some(ref mut of) = output_file { writeln!(of, "{}", result_string)?; }

        if opts.input.is_none() { print!("> "); std::io::stdout().flush()?; }
    }

    Ok(())
}
