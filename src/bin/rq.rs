extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate protobuf;
extern crate record_query;

use std::io;
use std::path;

use record_query as rq;

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

const AUTHOR: &'static str = "David Flemström <david.flemstrom@gmail.com>";

const INPUT_FORMAT_GROUP: &'static str = "input-format";
const OUTPUT_FORMAT_GROUP: &'static str = "output-format";

const PROTOBUF_CMD: &'static str = "protobuf";
const PROTOBUF_ADD_CMD: &'static str = "add";
const PROTOBUF_ADD_INPUT_ARG: &'static str = "protobuf-add-input";

const INPUT_JSON_ARG: &'static str = "input-json";
const OUTPUT_JSON_ARG: &'static str = "output-json";
const INPUT_PROTOBUF_ARG: &'static str = "input-protobuf";
const OUTPUT_PROTOBUF_ARG: &'static str = "output-protobuf";

const QUERY_ARG: &'static str = "query";

fn main() {
    use std::io::Read;

    env_logger::init().unwrap();

    let paths = rq::config::Paths::new().unwrap();
    let matches = match_args();

    if let Some(matches) = matches.subcommand_matches(PROTOBUF_CMD) {
        if let Some(matches) = matches.subcommand_matches(PROTOBUF_ADD_CMD) {
            let input = matches.value_of(PROTOBUF_ADD_INPUT_ARG).unwrap();
            rq::proto_index::add_file(&paths, path::Path::new(input)).unwrap();
        }
    } else {

        let query = rq::query::Query::parse(matches.value_of(QUERY_ARG).unwrap());

        let stdin = io::stdin();
        let mut input = stdin.lock();

        if matches.is_present(INPUT_PROTOBUF_ARG) {
            let name = matches.value_of(INPUT_PROTOBUF_ARG).unwrap();
            debug!("Input is protobuf with argument {}", name);

            let descriptors_proto = rq::proto_index::compile_descriptor_set(&paths).unwrap();
            let descriptors =
                rq::value::protobuf::descriptor::Descriptors::from_proto(&descriptors_proto);
            let stream = protobuf::CodedInputStream::new(&mut input);
            let values = rq::value::protobuf::ProtobufValues::new(descriptors,
                                                                  name.to_owned(),
                                                                  stream);
            run(values, query);
        } else {
            run(rq::value::json::JsonValues::new(input.bytes()), query);
        }
    }
}

fn match_args<'a>() -> clap::ArgMatches<'a> {
    clap::App::new("rq - Record query")
        .version(env!("CARGO_PKG_VERSION"))
        .author(AUTHOR)
        .about(ABOUT)
        .setting(clap::AppSettings::SubcommandsNegateReqs)
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
        .subcommand(clap::SubCommand::with_name(PROTOBUF_CMD)
                        .about("Control protobuf configuration and data")
                        .author(AUTHOR)
                        .subcommand(clap::SubCommand::with_name(PROTOBUF_ADD_CMD)
                                        .about("Add a schema file to the rq registry")
                                        .author(AUTHOR)
                                        .arg(clap::Arg::with_name(PROTOBUF_ADD_INPUT_ARG)
                                                 .required(true)
                                                 .value_name("schema")
                                                 .help("The path to a .proto file to add to \
                                                        the rq registry"))))
        .get_matches()
}

fn run<Iter>(input: Iter, query: rq::query::Query)
    where Iter: Iterator<Item = rq::error::Result<rq::value::Value>>
{
    use std::io::Write;
    let mut stdout = io::stdout();
    let context = rq::query::Context::new();

    debug!("Starting input consumption");

    for value in input {
        debug!("Consuming an input value");
        query.evaluate(&context, value.unwrap()).to_json(&mut stdout).unwrap();
        stdout.write(&[10]).unwrap();
        stdout.flush().unwrap();
    }
}
