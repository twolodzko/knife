mod fields;
mod knife;
mod set;

use clap::Parser;
use knife::Knife;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::PathBuf,
};

const FIELDS_HELP: &str = "Select the fields specified using a pattern language:
 N for N-th field (starting at 1),
 -N for all the fields up to N-th (inclusive),
 N- for all the fields from N-th (inclusive),
 N-M for a closed range,
 and comma-separated list for a combination, like 1,3-5.
The extracted fields are printed in the order they appeared in the input.";

/// Like the cut command, but delimits fields with whitespaces.
#[derive(Parser, Debug)]
struct Args {
    /// Select those fields, for example 1,3-5 for fields 1, 3, 4, and 5.
    #[arg(allow_hyphen_values = true, long_help = FIELDS_HELP)]
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
