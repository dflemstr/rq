use glob;
use protobuf;
use rmpv;
use serde_cbor;
use serde_hjson;
use serde_json;
use serde_protobuf;
use serde_yaml;
use std::io;
use std::string;
use toml;
#[cfg(feature = "v8")]
use v8;
use xdg_basedir;
use yaml_rust;
use csv;
use failure;

error_chain! {
    links {
        Protobuf(serde_protobuf::error::Error, serde_protobuf::error::ErrorKind);
        V8(v8::error::Error, v8::error::ErrorKind) #[cfg(feature = "v8")];
    }

    foreign_links {
        IO(io::Error);
        Utf8(string::FromUtf8Error);
        NativeProtobuf(protobuf::ProtobufError);
        MessagePackEncode(rmpv::encode::Error);
        Cbor(serde_cbor::Error);
        Hjson(serde_hjson::Error);
        Json(serde_json::Error);
        Yaml(serde_yaml::Error);
        YamlScan(yaml_rust::ScanError);
        TomlDeserialize(toml::de::Error);
        TomlSerialize(toml::ser::Error);
        XdgBasedir(xdg_basedir::Error);
        Glob(glob::GlobError);
        GlobPattern(glob::PatternError);
        Csv(csv::Error);
    }

    errors {
        // TODO: this should be a foreign_link but the type does not implement Display or Error
        MessagePackDecode(cause: rmpv::decode::value::Error) {
            description("message pack decode error")
            display("message pack decode error: {}", format_rmpv_decode_cause(cause))
        }
        Unimplemented(msg: String) {
            description("unimplemented")
            display("unimplemented: {}", msg)
        }
        IllegalState(msg: String) {
            description("illegal state")
            display("illegal state: {}", msg)
        }
        SyntaxError(msg: String) {
            description("syntax error")
            display("syntax error: {}", msg)
        }
        ProcessNotFound(name: String) {
            description("process not found")
            display("no such process: {}", name)
        }
        Failure(msg: String) {
            description("failure")
            display("{}", msg)
        }
    }
}

impl Error {
    pub fn unimplemented(msg: String) -> Error {
        ErrorKind::Unimplemented(msg).into()
    }

    pub fn illegal_state(msg: String) -> Error {
        ErrorKind::IllegalState(msg).into()
    }
}

fn format_rmpv_decode_cause(cause: &rmpv::decode::value::Error) -> String {
    match *cause {
        rmpv::decode::value::Error::InvalidMarkerRead(ref e) => {
            format!("error while reading marker byte: {}", e)
        },
        rmpv::decode::value::Error::InvalidDataRead(ref e) => {
            format!("error while reading data: {}", e)
        },
        rmpv::decode::value::Error::TypeMismatch(ref m) => {
            format!("decoded value type isn't equal with the expected one: {:?}",
                    m)
        },
        rmpv::decode::value::Error::FromUtf8Error(ref e) => {
            format!("failed to properly decode UTF-8: {}", e)
        },
    }
}

impl From<failure::Error> for Error {
    fn from(error: failure::Error) -> Self {
        Error::from_kind(ErrorKind::Failure(error.as_fail().to_string()))
    }
}