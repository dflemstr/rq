#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate protobuf;
extern crate record_query;

use std::io;
use std::path;

use record_query as rq;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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

const AUTHOR: &'static str = "David Flemström <dflemstr@spotify.com>";

fn main() {
    use std::io::Read;

    env_logger::init().unwrap();

    let paths = rq::config::Paths::new().unwrap();
    let matches = match_args();

    if let Some(matches) = matches.subcommand_matches("protobuf") {
        if let Some(matches) = matches.subcommand_matches("add") {
            let schema = matches.value_of("schema").unwrap();
            rq::proto_index::add_file(&paths, path::Path::new(schema)).unwrap();
        }
    } else {
        let query = rq::query::Query::parse(matches.value_of("query").unwrap());

        let stdin = io::stdin();
        let mut input = stdin.lock();

        if matches.is_present("input_protobuf") {
            let name = matches.value_of("input_protobuf").unwrap();
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
    let app = clap_app!(rq =>
      (version: VERSION)
      (author: AUTHOR)
      (about: ABOUT)
      (@setting SubcommandsNegateReqs)
      (@arg query: +required
       "The query to apply.")
      (@group input =>
        (@attributes +required)
        (@arg input_json: -j --input-json
         "Input is white-space separated JSON values.")
        (@arg input_protobuf: -p --input-protobuf <message> +next_line_help
         "Input is a single protocol buffer message.  The argument must be the \
          fully qualified message type e.g. \
          '.google.protobuf.DescriptorProto'.")
      )
      (@group output =>
        (@arg output_json: -J --output-json
         "Output should be formatted as JSON values.")
        (@arg output_protobuf: -P --output-protobuf <message> +next_line_help
         "Output should be formatted as protocol buffer objects.  The argument \
          must be the fully qualified message type e.g. \
          '.google.protobuf.DescriptorProto'.")
      )
      (@subcommand protobuf =>
        (about: "Control protobuf configuration and data.")
        (@subcommand add =>
          (about: "Add a schema file to the rq registry.")
          (@arg schema: +required
           "The path to a .proto file to add to the rq registry.")
        )
      )
    );
    app.get_matches()
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
