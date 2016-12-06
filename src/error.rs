use glob;
use protobuf;
use rmp;
use serde_avro;
use serde_cbor;
use serde_hjson;
use serde_json;
use serde_protobuf;
use serde_yaml;
use std::io;
use std::string;
use toml;
use v8;
use xdg_basedir;
use yaml_rust;

error_chain! {
    links {
        Avro(serde_avro::error::Error, serde_avro::error::ErrorKind);
        Protobuf(serde_protobuf::error::Error, serde_protobuf::error::ErrorKind);
        V8(v8::error::Error, v8::error::ErrorKind);
    }

    foreign_links {
        IO(io::Error);
        Utf8(string::FromUtf8Error);
        NativeProtobuf(protobuf::ProtobufError);
        MessagePackDecode(rmp::decode::value::Error);
        MessagePackEncode(rmp::encode::value::Error);
        Cbor(serde_cbor::Error);
        Hjson(serde_hjson::Error);
        Json(serde_json::Error);
        Yaml(serde_yaml::Error);
        YamlDecode(yaml_rust::ScanError);
        TomlParse(toml::ParserError);
        TomlDecode(toml::DecodeError);
        TomlEncode(toml::Error);
        XdgBasedir(xdg_basedir::Error);
        Glob(glob::GlobError);
        GlobPattern(glob::PatternError);
    }

    errors {
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
