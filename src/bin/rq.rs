#![feature(plugin)]
#![plugin(clippy, docopt_macros)]

extern crate ansi_term;
extern crate docopt;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate nix;
extern crate protobuf;
extern crate record_query;
extern crate rustc_serialize;

use std::env;
use std::io;
use std::path;

use record_query as rq;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

docopt!(pub Args derive Debug, concat!("
rq - record query v", env!("CARGO_PKG_VERSION"), "

A tool for manipulating data records.

Records are read from stdin, processed, and written to stdout.  The tool accepts
a query in the custom rq query language as its main command-line arguments.

See 'man rq' for in-depth documentation.

Usage:
  rq (--help|--version)
  rq [-j|-c|-p <type>] [-J|-C|-P <type>] [-l <level>|-q] [--] [<query>]
  rq [-l <level>|-q] protobuf add <schema>

Options:
  --help
      Show this screen.
  --version
      Show the program name and version.

  -j, --input-json
      Input is white-space separated JSON values.
  -J, --output-json
      Output should be formatted as JSON values.
  -c, --input-cbor
      Input is a series of CBOR values.
  -C, --output-cbor
      Output is a series of CBOR values.
  -p <type>, --input-protobuf <type>
      Input is a single protocol buffer object.  The argument refers to the
      fully qualified name of the message type (including the leading '.').
  -P <type>, --output-protobuf <type>
      Output should be formatted as protocol buffer objects.  The argument
      refers to the fully qualified name of the message type (including the
      leading '.').

  <query>
      A query indicating how to transform each record.

  -l <level>, --log <level>
      Display log messages at and above the specified log level.  The value can
      be one of 'off', 'error', 'warn', 'info', 'debug' or 'trace'.
  -q, --quiet
      Log nothing (alias for '-l off').
"));

fn main() {
    use std::io::Read;

    let args: Args = Args::docopt()
        .version(Some(VERSION.to_owned()))
        .decode().unwrap_or_else(|e| e.exit());

    setup_log(&args.flag_log, args.flag_quiet);

    let paths = rq::config::Paths::new().unwrap();

    if args.cmd_protobuf {
        if args.cmd_add {
            let schema = path::Path::new(&args.arg_schema);
            rq::proto_index::add_file(&paths, schema).unwrap();
        }
    } else {
        let query = rq::query::Query::parse(&args.arg_query);

        let stdin = io::stdin();
        let mut input = stdin.lock();

        if !args.flag_input_protobuf.is_empty() {
            let name = args.flag_input_protobuf;
            debug!("Input is protobuf with argument {}", name);

            let descriptors_proto = rq::proto_index::compile_descriptor_set(&paths).unwrap();
            let descriptors =
                rq::value::protobuf::descriptor::Descriptors::from_protobuf(&descriptors_proto);
            let stream = protobuf::CodedInputStream::new(&mut input);
            let values = rq::value::protobuf::ProtobufValues::new(descriptors,
                                                                  name.to_owned(),
                                                                  stream);
            run(values, query).unwrap_or_else(|e| error!("{:?}", e));
        } else if args.flag_input_cbor {
            run(rq::value::cbor::CborValues::new(input), query)
                .unwrap_or_else(|e| error!("{:?}", e));
        } else {
            run(rq::value::json::JsonValues::new(input.bytes()), query)
                .unwrap_or_else(|e| error!("{:?}", e));
        }
    }
}

fn setup_log(level: &str, quiet: bool) {
    use std::str::FromStr;
    use ansi_term::ANSIStrings;
    use ansi_term::Colour;
    use ansi_term::Style;
    use log::LogLevel;
    use log::LogLevelFilter;
    use nix::unistd;
    use nix::sys::ioctl;

    let normal = Style::new();

    let format = move |record: &log::LogRecord| {
        if unistd::isatty(ioctl::libc::STDERR_FILENO).unwrap_or(false) {
            let (front, back) = match record.level() {
                LogLevel::Error => (Colour::Red.normal(), Colour::Red.dimmed()),
                LogLevel::Warn => (Colour::Yellow.normal(), Colour::Yellow.dimmed()),
                LogLevel::Info => (Colour::Blue.normal(), Colour::Blue.dimmed()),
                LogLevel::Debug => (Colour::Purple.normal(), Colour::Purple.dimmed()),
                LogLevel::Trace => (Colour::White.dimmed(), Colour::Black.normal()),
            };

            let strings = &[back.paint("["),
                            front.paint(format!("{}", record.level())),
                            back.paint("]"),
                            normal.paint(" "),
                            back.paint("["),
                            front.paint(record.location().module_path()),
                            back.paint("]"),
                            normal.paint(" "),
                            back.paint(format!("{}", record.args()))];

            format!("{}", ANSIStrings(strings))
        } else {
            format!("[{}] [{}] {}",
                    record.level(),
                    record.location().module_path(),
                    record.args())
        }
    };

    let mut builder = env_logger::LogBuilder::new();

    let filter = if quiet {
        LogLevelFilter::Off
    } else {
        LogLevelFilter::from_str(level).unwrap_or(LogLevelFilter::Info)
    };

    builder.format(format).filter(None, filter);

    if let Ok(spec) = env::var("RUST_LOG") {
        builder.parse(&spec);
    }

    builder.init().unwrap();
}

fn run<Iter>(input: Iter, query: rq::query::Query) -> rq::error::Result<()>
    where Iter: Iterator<Item = rq::error::Result<rq::value::Value>>
{
    use std::io::Write;
    let mut stdout = io::stdout();
    let context = rq::query::Context::new();

    debug!("Starting input consumption");

    for value in input {
        let value = try!(value);
        debug!("Consuming an input value");
        try!(query.evaluate(&context, value).to_json(&mut stdout));
        try!(stdout.write(&[10]));
        try!(stdout.flush());
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse_args(args: &[&str]) -> Args {
        let a = Args::docopt().argv(args).decode().unwrap();
        println!("{:?}", a);
        a
    }

    #[test]
    fn test_docopt_kitchen_sink() {
        let a = parse_args(&["rq", "-l", "info", "-jP", ".foo.Bar", "select x"]);
        assert!(a.flag_input_json);
        assert_eq!(a.flag_output_protobuf, ".foo.Bar");
        assert_eq!(a.flag_log, "info");
        assert_eq!(a.arg_query, "select x");
    }

    #[test]
    fn test_docopt_no_args() {
        parse_args(&["rq"]);
    }

    #[test]
    #[should_panic(expected = "Help")]
    fn test_docopt_help() {
        parse_args(&["rq", "--help"]);
    }

    #[test]
    fn test_docopt_version() {
        let a = parse_args(&["rq", "--version"]);
        assert!(a.flag_version);
    }

    #[test]
    fn test_docopt_input_json() {
        let a = parse_args(&["rq", "-j"]);
        assert!(a.flag_input_json);
    }

    #[test]
    fn test_docopt_input_json_long() {
        let a = parse_args(&["rq", "--input-json"]);
        assert!(a.flag_input_json);
    }

    #[test]
    fn test_docopt_output_json() {
        let a = parse_args(&["rq", "-J"]);
        assert!(a.flag_output_json);
    }

    #[test]
    fn test_docopt_output_json_long() {
        let a = parse_args(&["rq", "--output-json"]);
        assert!(a.flag_output_json);
    }

    #[test]
    fn test_docopt_input_cbor() {
        let a = parse_args(&["rq", "-c"]);
        assert!(a.flag_input_cbor);
    }

    #[test]
    fn test_docopt_input_cbor_long() {
        let a = parse_args(&["rq", "--input-cbor"]);
        assert!(a.flag_input_cbor);
    }

    #[test]
    fn test_docopt_output_cbor() {
        let a = parse_args(&["rq", "-C"]);
        assert!(a.flag_output_cbor);
    }

    #[test]
    fn test_docopt_output_cbor_long() {
        let a = parse_args(&["rq", "--output-cbor"]);
        assert!(a.flag_output_cbor);
    }

    #[test]
    fn test_docopt_input_protobuf() {
        let a = parse_args(&["rq", "-p", ".foo.Bar"]);
        assert_eq!(a.flag_input_protobuf, ".foo.Bar");
    }

    #[test]
    fn test_docopt_input_protobuf_long() {
        let a = parse_args(&["rq", "--input-protobuf", ".foo.Bar"]);
        assert_eq!(a.flag_input_protobuf, ".foo.Bar");
    }

    #[test]
    fn test_docopt_output_protobuf() {
        let a = parse_args(&["rq", "-P", ".foo.Bar"]);
        assert_eq!(a.flag_output_protobuf, ".foo.Bar");
    }

    #[test]
    fn test_docopt_output_protobuf_long() {
        let a = parse_args(&["rq", "--output-protobuf", ".foo.Bar"]);
        assert_eq!(a.flag_output_protobuf, ".foo.Bar");
    }

    #[test]
    #[should_panic(expected = "NoMatch")]
    fn test_docopt_input_conflict() {
        parse_args(&["rq", "-jc"]);
    }

    #[test]
    #[should_panic(expected = "NoMatch")]
    fn test_docopt_output_conflict() {
        parse_args(&["rq", "-JC"]);
    }

    #[test]
    fn test_docopt_protobuf_add_schema() {
        let a = parse_args(&["rq", "-l", "info", "protobuf", "add", "schema.proto"]);
        assert_eq!(a.flag_log, "info");
        assert!(a.cmd_protobuf);
        assert!(a.cmd_add);
        assert_eq!(a.arg_schema, "schema.proto");
    }
}
