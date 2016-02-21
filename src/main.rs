extern crate clap;
extern crate serde;
extern crate serde_json;

mod error;
mod json_values;
mod value;

use std::io;

const ABOUT: &'static str = r#"
A tool for manipulating data records.  Similar in spirit to awk(1),
but on steroids.  Inspired by jq(1), but supports more record formats
and operations.

Records are read from stdin, processed, and written to stdout.  The
tool accepts a query in the custom rq query language as its main
command-line arguments.

Flags can be used to control the input and output formats of the tool.
By default, the input format is guessed, and the output format is the
same as the input format.

See 'man rq' for in-depth documentation."#;

const INPUT_JSON_ARG: &'static str = "input-json";
const OUTPUT_JSON_ARG: &'static str = "output-json";

const QUERY_ARG: &'static str = "query";

fn main() {
    use std::io::Read;

    let matches = match_args();

    let stdin = io::stdin();
    let input = stdin.lock();

    if matches.is_present(INPUT_JSON_ARG) {
        run(json_values::JsonValues::new(input.bytes()));
    } else {
        panic!("Only JSON parsing (-j) is implemented for now (see --help)");
    }
}

fn match_args<'a>() -> clap::ArgMatches<'a> {
    clap::App::new("rq - Record query")
        .version(env!("CARGO_PKG_VERSION"))
        .author("David Flemström <dflemstr@spotify.com>")
        .about(ABOUT)
        .arg(clap::Arg::with_name(INPUT_JSON_ARG)
                 .short("j")
                 .long("input-json")
                 .help("Input is white-space separated JSON values."))
        .arg(clap::Arg::with_name(OUTPUT_JSON_ARG)
                 .short("J")
                 .long("output-json")
                 .help("Output should be formatted as JSON values."))
        .arg(clap::Arg::with_name(QUERY_ARG)
                 .multiple(true)
                 .help("The query to apply."))
        .get_matches()
}

fn run<Iter>(input: Iter)
    where Iter: Iterator<Item = value::Value>
{
    use std::io::Write;
    let mut stdout = io::stdout();
    for value in input {
        value.to_json(&mut stdout).unwrap();
        stdout.write(&[10]).unwrap();
        stdout.flush().unwrap();
    }
}
