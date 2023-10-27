mod knife;
mod matcher;
mod parser;

use clap::Parser;
use knife::Knife;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::PathBuf,
};

const DETAILS: &str = color_print::cstr!(
    "<u><s>Details:</s></u>

The <<FIELDS>> are specified using a pattern language where N stands for for N-th field (starting at 1), \
-N for all the fields up to N-th (inclusive), N- for all the fields starting from N-th (inclusive), \
N-M for a closed range, and comma-separated list for a combination of the patterns. \
It is also possible to use : instead of - for defining ranges.

The extracted fields are printed in the order they appeared in the input.");

/// Like the cut command, but delimits fields with whitespaces.
#[derive(Parser, Debug)]
#[command(after_long_help = DETAILS)]
struct Args {
    /// Select those fields, for example, 1,3-5 means fields 1, 3, 4, and 5.
    #[arg(allow_hyphen_values = true)]
    fields: Knife,

    /// Paths to the files to process, if not given, use Stdin.
    #[arg(trailing_var_arg(true))]
    file: Vec<PathBuf>,
}

type Reader = BufReader<Box<dyn Read>>;

#[inline]
fn process_lines(reader: Reader, knife: &Knife) {
    reader
        .lines()
        .filter_map(|line| {
            match line {
                Ok(line) => Some(line),
                Err(err) => {
                    // print errors to stderr and carry on
                    eprintln!("{}", err);
                    None
                }
            }
        })
        .for_each(|ref line| {
            let fields = knife.extract(line);
            println!("{}", fields.join(" "));
        })
}

fn main() {
    let args = Args::parse();

    let mut reader: Reader;
    if args.file.is_empty() {
        reader = BufReader::new(Box::new(io::stdin()));
        process_lines(reader, &args.fields);
    } else {
        for path in &args.file {
            reader = match File::open(path) {
                Ok(file) => BufReader::new(Box::new(file)),
                Err(msg) => {
                    eprintln!("{}", msg);
                    std::process::exit(1);
                }
            };
            process_lines(reader, &args.fields);
        }
    }
}
