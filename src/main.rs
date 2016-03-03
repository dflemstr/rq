#![feature(advanced_slice_patterns, plugin, slice_patterns)]
#![plugin(mod_path)]

extern crate clap;
extern crate glob;
extern crate protobuf;
extern crate serde;
extern crate serde_json;
extern crate xdg_basedir;

mod config;
mod error;
mod proto_index;
mod query;
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
By default, the input and output format is JSON.

See 'man rq' for in-depth documentation."#;

const INPUT_FORMAT_GROUP: &'static str = "input-format";
const OUTPUT_FORMAT_GROUP: &'static str = "output-format";

const INPUT_JSON_ARG: &'static str = "input-json";
const OUTPUT_JSON_ARG: &'static str = "output-json";
const INPUT_PROTOBUF_ARG: &'static str = "input-protobuf";
const OUTPUT_PROTOBUF_ARG: &'static str = "output-protobuf";

const QUERY_ARG: &'static str = "query";

fn main() {
    use std::io::Read;

    let paths = config::Paths::new().unwrap();
    let matches = match_args();

    let query = query::Query::parse(matches.value_of(QUERY_ARG).unwrap());

    let stdin = io::stdin();
    let input = stdin.lock();

    if matches.is_present(INPUT_PROTOBUF_ARG) {
        let descriptors = proto_index::compile_descriptor_set(&paths).unwrap();
        unimplemented!()
    } else {
        run(value::json::JsonValues::new(input.bytes()), query);
    }
}

fn match_args<'a>() -> clap::ArgMatches<'a> {
    clap::App::new("rq - Record query")
        .version(env!("CARGO_PKG_VERSION"))
        .author("David Flemström <dflemstr@spotify.com>")
        .about(ABOUT)
        .group(clap::ArgGroup::with_name(INPUT_FORMAT_GROUP))
        .group(clap::ArgGroup::with_name(OUTPUT_FORMAT_GROUP))
        .arg(clap::Arg::with_name(INPUT_JSON_ARG)
                 .group(INPUT_FORMAT_GROUP)
                 .short("j")
                 .long("input-json")
                 .help("Input is white-space separated JSON values."))
        .arg(clap::Arg::with_name(OUTPUT_JSON_ARG)
                 .group(OUTPUT_FORMAT_GROUP)
                 .short("J")
                 .long("output-json")
                 .help("Output should be formatted as JSON values."))
        .arg(clap::Arg::with_name(INPUT_PROTOBUF_ARG)
                 .group(INPUT_FORMAT_GROUP)
                 .short("p")
                 .long("input-protobuf")
                 .takes_value(true)
                 .value_name("schema-alias:MessageType")
                 .next_line_help(true)
                 .help("Input is a single protocol buffer object.  The \
                        argument refers to a schema alias defined in the \
                        config."))
        .arg(clap::Arg::with_name(OUTPUT_PROTOBUF_ARG)
                 .group(OUTPUT_FORMAT_GROUP)
                 .short("P")
                 .long("output-protobuf")
                 .takes_value(true)
                 .value_name("schema-alias:MessageType")
                 .next_line_help(true)
                 .help("Output should be formatted as protocol buffer \
                        objects.  The argument refers to a schema alias \
                        defined in the config, but if it is omitted and -p \
                        was used, the input schema is used instead."))
        .arg(clap::Arg::with_name(QUERY_ARG)
                 .required(true)
                 .help("The query to apply."))
        .get_matches()
}

fn run<Iter>(input: Iter, query: query::Query)
    where Iter: Iterator<Item = value::Value>
{
    use std::io::Write;
    let mut stdout = io::stdout();
    let context = query::Context::new();
    for value in input {
        query.evaluate(&context, value).to_json(&mut stdout).unwrap();
        stdout.write(&[10]).unwrap();
        stdout.flush().unwrap();
    }
}
