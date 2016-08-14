use std::io;
use std::string;

use duk;
use glob;
use protobuf;
use rmp;
use serde_cbor;
use serde_hjson;
use serde_json;
use serde_protobuf;
use serde_yaml;
use xdg_basedir;
use yaml_rust;

error_chain! {
    types {
        Error, ErrorKind, ChainErr, Result;
    }

    links {
        duk::Error, duk::ErrorKind, Duk;
    }

    foreign_links {
        io::Error, IO;
        string::FromUtf8Error, Utf8;
        protobuf::ProtobufError, NativeProtobuf;
        rmp::decode::value::Error, MessagePackDecode;
        rmp::encode::value::Error, MessagePackEncode;
        serde_cbor::Error, Cbor;
        serde_hjson::Error, Hjson;
        serde_json::Error, Json;
        serde_protobuf::Error, Protobuf;
        serde_yaml::Error, Yaml;
        yaml_rust::ScanError, YamlDecode;
        xdg_basedir::Error, XdgBasedir;
        glob::GlobError, Glob;
        glob::PatternError, GlobPattern;
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
