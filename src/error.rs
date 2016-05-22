use std::io;
use std::result;
use std::string;

use glob;
use protobuf;
use serde_cbor;
use serde_json;
use serde_protobuf;
use xdg_basedir;

pub type Result<A> = result::Result<A, Error>;

#[derive(Debug)]
pub enum Error {
    General(String),
    IO(io::Error),
    FromUtf8(string::FromUtf8Error),
    Protobuf(protobuf::ProtobufError),
    XdgBasedir(xdg_basedir::Error),
    Glob(glob::GlobError),
    Pattern(glob::PatternError),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<serde_cbor::Error> for Error {
    fn from(e: serde_cbor::Error) -> Error {
        unimplemented!()
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        match e {
            serde_json::Error::Io(e) => Error::IO(e),
            serde_json::Error::FromUtf8(e) => Error::FromUtf8(e),
            serde_json::Error::Syntax(_, _, _) => unimplemented!(),
        }
    }
}

impl From<serde_protobuf::Error> for Error {
    fn from(e: serde_protobuf::Error) -> Error {
        unimplemented!()
    }
}

impl From<xdg_basedir::Error> for Error {
    fn from(e: xdg_basedir::Error) -> Error {
        Error::XdgBasedir(e)
    }
}

impl From<glob::GlobError> for Error {
    fn from(e: glob::GlobError) -> Error {
        Error::Glob(e)
    }
}

impl From<glob::PatternError> for Error {
    fn from(e: glob::PatternError) -> Error {
        Error::Pattern(e)
    }
}

impl From<protobuf::ProtobufError> for Error {
    fn from(e: protobuf::ProtobufError) -> Error {
        Error::Protobuf(e)
    }
}
