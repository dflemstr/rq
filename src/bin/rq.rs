#[macro_use]
extern crate log;
#[macro_use]
extern crate structopt;

use record_query as rq;
use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path;
use std::str;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "rq",
    version = record_query::VERSION,
    about = r#"
A tool for manipulating data records.

Records are read from stdin, processed, and written to stdout.  The tool accepts
a query in the custom rq query language as its main command-line arguments.

See https://github.com/dflemstr/rq for in-depth documentation.
"#
)]
pub struct Options {
    #[structopt(subcommand)]
    pub subcmd: Option<Subcmd>,

    /// A query indicating how to transform each record.
    pub arg_query: Option<String>,

    /// Force stylistic output formatting.  Can be one of 'compact',
    /// 'readable' (with color) or 'indented' (without color) and the default is
    /// inferred from the terminal environment.
    #[structopt(long = "format")]
    pub flag_format: Option<Format>,
    #[structopt(long = "codec")]
    pub flag_codec: Option<String>,

    /// Input is an Apache Avro container file.
    #[structopt(short = "a", long = "input-avro")]
    pub flag_input_avro: bool,
    /// Input is a series of CBOR values.
    #[structopt(short = "c", long = "input-cbor")]
    pub flag_input_cbor: bool,
    /// Input is white-space separated JSON values (default).
    #[structopt(short = "j", long = "input-json")]
    pub flag_input_json: bool,
    /// Input is CSV.
    #[structopt(short = "v", long = "input-csv")]
    pub flag_input_csv: bool,
    /// Input is formatted as MessagePack.
    #[structopt(short = "m", long = "input-message-pack")]
    pub flag_input_message_pack: bool,
    #[structopt(short = "p", long = "input-protobuf")]
    pub flag_input_protobuf: Option<String>,
    /// Input is plain text.
    #[structopt(short = "r", long = "input-raw")]
    pub flag_input_raw: bool,
    /// Input is formatted as TOML document.
    #[structopt(short = "t", long = "input-toml")]
    pub flag_input_toml: bool,
    /// Input is a series of YAML documents.
    #[structopt(short = "y", long = "input-yaml")]
    pub flag_input_yaml: bool,

    #[structopt(short = "A", long = "output-avro")]
    pub flag_output_avro: Option<String>,
    #[structopt(short = "C", long = "output-cbor")]
    pub flag_output_cbor: bool,
    #[structopt(short = "J", long = "output-json")]
    pub flag_output_json: bool,
    #[structopt(short = "R", long = "output-raw")]
    pub flag_output_raw: bool,
    #[structopt(short = "V", long = "output-csv")]
    pub flag_output_csv: bool,
    #[structopt(short = "M", long = "output-message-pack")]
    pub flag_output_message_pack: bool,
    #[structopt(short = "P", long = "output-protobuf")]
    pub flag_output_protobuf: Option<String>,
    #[structopt(short = "T", long = "output-toml")]
    pub flag_output_toml: bool,
    #[structopt(short = "Y", long = "output-yaml")]
    pub flag_output_yaml: bool,

    #[structopt(short = "l", long = "log")]
    pub flag_log: Option<String>,
    #[structopt(short = "q", long = "quiet")]
    pub flag_quiet: bool,
    #[structopt(long = "trace")]
    pub flag_trace: bool,
}

#[derive(Debug, StructOpt)]
pub enum Subcmd {
    #[structopt(name = "protobuf")]
    Protobuf {
        #[structopt(subcommand)]
        subcmd: ProtobufSubcmd,
    },
}

#[derive(Debug, StructOpt)]
pub enum ProtobufSubcmd {
    #[structopt(name = "add")]
    Add {
        schema: path::PathBuf,
        #[structopt(short = "b", long = "base")]
        base: Option<path::PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Format {
    Compact,
    Readable,
    Indented,
}

fn main() {
    use structopt::StructOpt;


    let args: Options = match Options::clap().get_matches_safe() {
        Err(e) => {
            match e.kind {
                structopt::clap::ErrorKind::HelpDisplayed => set_ran_cmd("help").unwrap(),
                structopt::clap::ErrorKind::VersionDisplayed => {
                    set_ran_cmd("version").unwrap()
                }
                _ => (),
            }
            e.exit()
        }
        Ok(a) => Options::from_clap(&a),
    };

    setup_log(args.flag_log.as_ref().map(String::as_ref), args.flag_quiet);

    main_with_args(&args).unwrap_or_else(|e| log_error(&args, &e));
}

fn main_with_args(args: &Options) -> rq::error::Result<()> {
    match args.subcmd {
        Some(Subcmd::Protobuf { ref subcmd }) => match subcmd {
            ProtobufSubcmd::Add { schema, base } => {
                let base = base
                    .as_ref()
                    .map_or_else(|| path::Path::new("."), |p| p.as_path());
                let paths = rq::config::Paths::new()?;
                rq::proto_index::add_file(&paths, base, &schema)
            }
        },
        None => run(&args),
    }
}

fn run(args: &Options) -> rq::error::Result<()> {
    let stdin = io::stdin();
    let mut input = stdin.lock();

    if let Some(ref name) = args.flag_input_protobuf {
        let paths = rq::config::Paths::new()?;
        let proto_descriptors = load_descriptors(&paths)?;
        let stream = protobuf::CodedInputStream::new(&mut input);
        let source = rq::value::protobuf::source(&proto_descriptors, name, stream)?;
        run_source(args, source)
    } else if args.flag_input_avro {
        let source = rq::value::avro::source(&mut input)?;
        run_source(args, source)
    } else if args.flag_input_cbor {
        let source = rq::value::cbor::source(&mut input);
        run_source(args, source)
    } else if args.flag_input_message_pack {
        let source = rq::value::messagepack::source(&mut input);
        run_source(args, source)
    } else if args.flag_input_toml {
        let source = rq::value::toml::source(&mut input)?;
        run_source(args, source)
    } else if args.flag_input_yaml {
        let source = rq::value::yaml::source(&mut input);
        run_source(args, source)
    } else if args.flag_input_raw {
        let source = rq::value::raw::source(&mut input);
        run_source(args, source)
    } else if args.flag_input_csv {
        if env::args().skip(1).any(|v| v == "-v") && !has_ran_cmd("help")? {
            warn!("You started rq -v, which puts it in CSV input mode.");
            warn!("It's now waiting for CSV input, which might not be what you wanted.");
            warn!(
                "Specify --input-csv explicitly or run rq --help once to suppress this \
                 warning."
            );
        }
        let source = rq::value::csv::source(&mut input);
        run_source(args, source)
    } else {
        if !args.flag_input_json && !has_ran_cmd("help")? {
            warn!("You started rq without any input flags, which puts it in JSON input mode.");
            warn!("It's now waiting for JSON input, which might not be what you wanted.");
            warn!(
                "Specify (-j|--input-json) explicitly or run rq --help once to suppress this \
                 warning."
            );
        }
        let source = rq::value::json::source(&mut input);
        run_source(args, source)
    }
}

fn run_source<I>(args: &Options, source: I) -> rq::error::Result<()>
where
    I: rq::value::Source,
{
    let mut output = io::stdout();

    let format = args.flag_format.unwrap_or_else(infer_format);

    macro_rules! dispatch_format {
        ($compact:expr, $readable:expr, $indented:expr) => {
            match format {
                Format::Compact => {
                    let sink = $compact(&mut output);
                    run_source_sink(source, sink)
                }
                Format::Readable => {
                    let sink = $readable(&mut output);
                    run_source_sink(source, sink)
                }
                Format::Indented => {
                    let sink = $indented(&mut output);
                    run_source_sink(source, sink)
                }
            }
        };
    }

    if args.flag_output_protobuf.is_some() {
        Err(rq::error::Error::unimplemented(
            "protobuf serialization".to_owned(),
        ))
    } else if let Some(ref schema_filename) = args.flag_output_avro {
        use std::str::FromStr;

        let schema = read_avro_schema_from_file(path::Path::new(schema_filename))?;
        let codec_string = if let Some(ref c) = args.flag_codec {
            c.as_str()
        } else {
            "null"
        };
        let codec = if let Ok(v) = avro_rs::Codec::from_str(&codec_string) {
            v
        } else {
            return Err(rq::error::Error::Message(format!(
                "illegal Avro codec: {}",
                codec_string
            )));
        };
        let sink = rq::value::avro::sink(&schema, &mut output, codec)?;
        run_source_sink(source, sink)
    } else if args.flag_output_cbor {
        let sink = rq::value::cbor::sink(&mut output);
        run_source_sink(source, sink)
    } else if args.flag_output_message_pack {
        let sink = rq::value::messagepack::sink(&mut output);
        run_source_sink(source, sink)
    } else if args.flag_output_toml {
        // TODO: add TOML ugly printing eventually; now it's always "readable"
        dispatch_format!(
            rq::value::toml::sink,
            rq::value::toml::sink,
            rq::value::toml::sink
        )
    } else if args.flag_output_yaml {
        // TODO: add YAML ugly printing eventually; now it's always "readable"
        dispatch_format!(
            rq::value::yaml::sink,
            rq::value::yaml::sink,
            rq::value::yaml::sink
        )
    } else if args.flag_output_raw {
        let sink = rq::value::raw::sink(&mut output);
        run_source_sink(source, sink)
    } else if args.flag_output_csv {
        let sink = rq::value::csv::sink(&mut output);
        run_source_sink(source, sink)
    } else {
        dispatch_format!(
            rq::value::json::sink_compact,
            rq::value::json::sink_readable,
            rq::value::json::sink_indented
        )
    }
}

fn read_avro_schema_from_file(path: &path::Path) -> rq::error::Result<avro_rs::Schema> {
    let mut file = fs::File::open(path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    Ok(avro_rs::Schema::parse_str(&buffer)
        .map_err(|e| rq::error::Error::Avro(rq::error::Avro::downcast(e)))?)
}

fn run_source_sink<I, O>(
    mut source: I,
    mut sink: O,
) -> rq::error::Result<()>
where
    I: rq::value::Source,
    O: rq::value::Sink,
{
    while let Some(result) = rq::value::Source::read(&mut source)? {
        sink.write(result)?;
    }
    Ok(())
}

fn load_descriptors(
    paths: &rq::config::Paths,
) -> rq::error::Result<serde_protobuf::descriptor::Descriptors> {
    let descriptors_proto = rq::proto_index::compile_descriptor_set(paths)?;
    Ok(serde_protobuf::descriptor::Descriptors::from_proto(
        &descriptors_proto,
    ))
}

fn infer_format() -> Format {
    if atty::is(atty::Stream::Stdout) {
        Format::Readable
    } else {
        Format::Compact
    }
}

fn has_ran_cmd(cmd: &str) -> rq::error::Result<bool> {
    let paths = match rq::config::Paths::new() {
        Ok(paths) => paths,
        Err(_) => return Ok(false),
    };
    paths
        .find_config(&format!("{}{}", "has-ran-", cmd))
        .map(|v| !v.is_empty())
        .map_err(From::from)
}

fn set_ran_cmd(cmd: &str) -> rq::error::Result<()> {
    let paths = match rq::config::Paths::new() {
        Ok(paths) => paths,
        Err(_) => return Ok(()),
    };

    let file = paths.preferred_config(format!("{}{}", "has-ran-", cmd));

    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::File::create(&file)?;

    Ok(())
}

fn log_error(args: &Options, error: &rq::error::Error) {
    use failure::Fail;

    let main_str = format!("{}", error);
    let mut main_lines = main_str.lines();
    error!("Encountered: {}", main_lines.next().unwrap());
    for line in main_lines {
        error!("  {}", line);
    }
    for e in <dyn failure::Fail>::iter_causes(error) {
        let sub_str = format!("{}", e);
        let mut sub_lines = sub_str.lines();
        error!("Caused by: {}", sub_lines.next().unwrap());
        for line in sub_lines {
            error!("  {}", line);
        }
    }

    if args.flag_trace || env::var("RUST_BACKTRACE").as_ref().map(String::as_str) == Ok("1") {
        error!("");
        if let Some(backtrace) = error.backtrace() {
            error!("Backtrace:");
            for line in format!("{:?}", backtrace).lines() {
                error!("  {}", line);
            }
        } else {
            error!("(No backtrace available)");
        }
    } else {
        error!("(Re-run with --trace or RUST_BACKTRACE=1 for a backtrace)");
    }
}

fn setup_log(spec: Option<&str>, quiet: bool) {
    let mut builder = env_logger::Builder::new();

    if quiet {
        builder.filter(None, log::LevelFilter::Off);
    } else if let Some(s) = spec {
        builder.parse_filters(s);
    } else if let Ok(s) = env::var("RUST_LOG") {
        builder.parse_filters(&s);
    } else {
        builder.filter(None, log::LevelFilter::Info);
    };

    builder.format(format_log_record);

    builder.init();
}

impl str::FromStr for Format {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "compact" => Ok(Self::Compact),
            "readable" => Ok(Self::Readable),
            "indented" => Ok(Self::Indented),
            _ => Err(failure::err_msg(format!("unrecognized format: {}", s))),
        }
    }
}

fn format_log_record(
    formatter: &mut env_logger::fmt::Formatter,
    record: &log::Record,
) -> io::Result<()> {
    use ansi_term::ANSIStrings;
    use ansi_term::Colour;
    use ansi_term::Style;

    if atty::is(atty::Stream::Stderr) {
        let normal = Style::new();
        let (front, back) = match record.level() {
            log::Level::Error => (Colour::Red.normal(), Colour::Red.dimmed()),
            log::Level::Warn => (Colour::Yellow.normal(), Colour::Yellow.dimmed()),
            log::Level::Info => (Colour::Blue.normal(), Colour::Blue.dimmed()),
            log::Level::Debug => (Colour::Purple.normal(), Colour::Purple.dimmed()),
            log::Level::Trace => (Colour::White.dimmed(), Colour::Black.normal()),
        };

        let strings = &[
            back.paint("["),
            front.paint(format!("{}", record.level())),
            back.paint("]"),
            normal.paint(" "),
            back.paint("["),
            front.paint(record.module_path().unwrap_or("<unknown>")),
            back.paint("]"),
            normal.paint(" "),
            front.paint(format!("{}", record.args())),
        ];

        writeln!(formatter, "{}", ANSIStrings(strings))
    } else {
        writeln!(
            formatter,
            "[{}] [{}] {}",
            record.level(),
            record.module_path().unwrap_or("<unknown>"),
            record.args()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse_args(args: &[&str]) -> Options {
        use structopt::StructOpt;
        let a = Options::from_iter_safe(args.iter()).unwrap();
        println!("{:?}", a);
        a
    }

    #[test]
    fn test_docopt_kitchen_sink() {
        let a = parse_args(&["rq", "-l", "info", "-jP", ".foo.Bar", "select x"]);
        assert!(a.flag_input_json);
        assert_eq!(a.flag_output_protobuf, Some(".foo.Bar".to_owned()));
        assert_eq!(a.flag_log, Some("info".to_owned()));
        assert_eq!(a.arg_query, Some("select x".to_owned()));
    }

    #[test]
    fn test_docopt_no_args() {
        parse_args(&["rq"]);
    }

    #[test]
    #[cfg_attr(
        all(target_arch = "x86", target_pointer_width = "32", target_env = "musl"),
        ignore
    )]
    #[should_panic(expected = "Help")]
    fn test_docopt_help() {
        parse_args(&["rq", "--help"]);
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
    fn test_docopt_input_raw() {
        let a = parse_args(&["rq", "-r"]);
        assert!(a.flag_input_raw);
    }

    #[test]
    fn test_docopt_input_raw_long() {
        let a = parse_args(&["rq", "--input-raw"]);
        assert!(a.flag_input_raw);
    }

    #[test]
    fn test_docopt_output_raw() {
        let a = parse_args(&["rq", "-R"]);
        assert!(a.flag_output_raw);
    }

    #[test]
    fn test_docopt_output_raw_long() {
        let a = parse_args(&["rq", "--output-raw"]);
        assert!(a.flag_output_raw);
    }

    #[test]
    fn test_docopt_input_csv() {
        let a = parse_args(&["rq", "-v"]);
        assert!(a.flag_input_csv);
    }

    #[test]
    fn test_docopt_input_csv_long() {
        let a = parse_args(&["rq", "--input-csv"]);
        assert!(a.flag_input_csv);
    }

    #[test]
    fn test_docopt_output_csv() {
        let a = parse_args(&["rq", "-V"]);
        assert!(a.flag_output_csv);
    }

    #[test]
    fn test_docopt_output_csv_long() {
        let a = parse_args(&["rq", "--output-csv"]);
        assert!(a.flag_output_csv);
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
    fn test_docopt_protobuf_add_schema() {
        let a = parse_args(&["rq", "-l", "info", "protobuf", "add", "schema.proto"]);
        assert_eq!(a.flag_log, Some("info".to_owned()));
        assert_eq!(
            Some(path::PathBuf::from("schema.proto")),
            match a.subcmd {
                Some(Subcmd::Protobuf { subcmd }) => match subcmd {
                    ProtobufSubcmd::Add { schema, .. } => Some(schema),
                },
                _ => None,
            }
        );
    }

    #[test]
    fn test_docopt_format_compact() {
        let a = parse_args(&["rq", "--format", "compact"]);
        assert_eq!(a.flag_format, Some(Format::Compact));
    }

    #[test]
    fn test_docopt_format_readable() {
        let a = parse_args(&["rq", "--format", "readable"]);
        assert_eq!(a.flag_format, Some(Format::Readable));
    }

    #[test]
    fn test_docopt_format_indented() {
        let a = parse_args(&["rq", "--format", "indented"]);
        assert_eq!(a.flag_format, Some(Format::Indented));
    }
}
