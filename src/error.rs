use std::io;
use std::string;

use duk;
use glob;
use protobuf;
use serde_cbor;
use serde_hjson;
use serde_json;
use serde_protobuf;
use xdg_basedir;

error_chain! {
    types {
        Error, ErrorKind, ChainErr, Result;
    }

    links {
        duk::Error, duk::ErrorKind, Duk;
    }

    foreign_links {
        io::Error, IO, "IO error";
        string::FromUtf8Error, Utf8, "UTF-8 error";
        protobuf::ProtobufError, NativeProtobuf, "native protobuf error";
        serde_cbor::Error, Cbor, "CBOR error";
        serde_hjson::Error, Hjson, "HJSON error";
        serde_json::Error, Json, "JSON error";
        serde_protobuf::Error, Protobuf, "protobuf error";
        xdg_basedir::Error, XdgBasedir, "XDG basedir error";
        glob::GlobError, Glob, "glob error";
        glob::PatternError, GlobPattern, "glob pattern error";
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
