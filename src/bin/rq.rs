#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate ansi_term;
extern crate docopt;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate nix;
extern crate protobuf;
extern crate record_query;
extern crate rustc_serialize;
extern crate serde_protobuf;

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
  rq [-l <level>|-q] protobuf add <schema> [--base <path>]

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

  --base <path>
      Directories are significant when dealing with protocol buffer
      schemas.  This specifies the base directory used to normalize schema
      file paths [default: .]

  -l <level>, --log <level>
      Display log messages at and above the specified log level.  The value can
      be one of 'off', 'error', 'warn', 'info', 'debug' or 'trace'.
  -q, --quiet
      Log nothing (alias for '-l off').
"),
        flag_input_protobuf: Option<String>,
        flag_output_protobuf: Option<String>,
        flag_log: Option<String>,
        flag_base: Option<String>);

fn main() {
    let args: Args = Args::docopt()
                         .version(Some(VERSION.to_owned()))
                         .decode()
                         .unwrap_or_else(|e| e.exit());

    main_with_args(&args).unwrap();
}

fn main_with_args(args: &Args) -> rq::error::Result<()> {
    setup_log(args.flag_log.as_ref().map(String::as_ref), args.flag_quiet);

    let paths = rq::config::Paths::new().unwrap();

    if args.cmd_protobuf {
        if args.cmd_add {
            let schema = path::Path::new(&args.arg_schema);
            let base = path::Path::new(if let Some(ref b) = args.flag_base {
                b.as_str()
            } else {
                "."
            });
            rq::proto_index::add_file(&paths, base, schema)
        } else {
            unreachable!()
        }
    } else {
        run(&args, &paths)
    }
}

fn run(args: &Args, paths: &rq::config::Paths) -> rq::error::Result<()> {

    let stdin = io::stdin();
    let mut input = stdin.lock();

    if let Some(ref name) = args.flag_input_protobuf {
        let proto_descriptors = try!(load_descriptors(&paths));
        let stream = protobuf::CodedInputStream::new(&mut input);
        let source = try!(rq::value::protobuf::source(&proto_descriptors, name, stream));
        run_source(args, paths, source)
    } else if args.flag_input_cbor {
        let source = rq::value::cbor::source(&mut input);
        run_source(args, paths, source)
    } else {
        let source = rq::value::json::source(&mut input);
        run_source(args, paths, source)
    }
}

fn run_source<I>(args: &Args, paths: &rq::config::Paths, source: I) -> rq::error::Result<()>
    where I: rq::value::Source
{
    let mut output = io::stdout();

    if let Some(_) = args.flag_output_protobuf {
        Err(rq::error::Error::unimplemented("protobuf serialization".to_owned()))
    } else if args.flag_output_cbor {
        let sink = rq::value::cbor::sink(&mut output);
        run_source_sink(args, paths, source, sink)
    } else {
        let sink = rq::value::json::sink(&mut output);
        run_source_sink(args, paths, source, sink)
    }
}

fn run_source_sink<I, O>(args: &Args,
                         _paths: &rq::config::Paths,
                         source: I,
                         mut sink: O)
                         -> rq::error::Result<()>
    where I: rq::value::Source,
          O: rq::value::Sink
{
    use record_query::value::Source;

    let query = rq::query::Query::parse(&args.arg_query);
    let query_context = rq::query::Context::new();
    let mut results = try!(query.evaluate(&query_context, source));
    while let Some(result) = try!(results.read()) {
        try!(sink.write(result));
    }
    Ok(())
}

fn load_descriptors(paths: &rq::config::Paths)
                    -> rq::error::Result<serde_protobuf::descriptor::Descriptors> {
    let descriptors_proto = try!(rq::proto_index::compile_descriptor_set(paths));
    Ok(serde_protobuf::descriptor::Descriptors::from_proto(&descriptors_proto))
}

fn setup_log(level: Option<&str>, quiet: bool) {
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
    } else if let Some(l) = level {
        LogLevelFilter::from_str(l).unwrap_or(LogLevelFilter::Info)
    } else {
        LogLevelFilter::Info
    };

    builder.format(format).filter(None, filter);

    if let Ok(spec) = env::var("RUST_LOG") {
        builder.parse(&spec);
    }

    builder.init().unwrap();
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
        assert_eq!(a.flag_output_protobuf, Some(".foo.Bar".to_owned()));
        assert_eq!(a.flag_log, Some("info".to_owned()));
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
        assert_eq!(a.flag_input_protobuf, Some(".foo.Bar".to_owned()));
    }

    #[test]
    fn test_docopt_input_protobuf_long() {
        let a = parse_args(&["rq", "--input-protobuf", ".foo.Bar"]);
        assert_eq!(a.flag_input_protobuf, Some(".foo.Bar".to_owned()));
    }

    #[test]
    fn test_docopt_output_protobuf() {
        let a = parse_args(&["rq", "-P", ".foo.Bar"]);
        assert_eq!(a.flag_output_protobuf, Some(".foo.Bar".to_owned()));
    }

    #[test]
    fn test_docopt_output_protobuf_long() {
        let a = parse_args(&["rq", "--output-protobuf", ".foo.Bar"]);
        assert_eq!(a.flag_output_protobuf, Some(".foo.Bar".to_owned()));
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
        assert_eq!(a.flag_log, Some("info".to_owned()));
        assert!(a.cmd_protobuf);
        assert!(a.cmd_add);
        assert_eq!(a.arg_schema, "schema.proto");
    }
}
