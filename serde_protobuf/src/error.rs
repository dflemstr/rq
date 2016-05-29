use std::error;
use std::fmt;
use std::result;

use protobuf;
use protobuf::stream::wire_format;
use serde;

pub type Result<A> = result::Result<A, Error>;

#[derive(Debug)]
pub enum Error {
    EndOfStream,
    Protobuf(protobuf::ProtobufError),
    UnknownEnum(String),
    UnknownEnumValue(i32),
    UnknownMessage(String),
    BadWireType(wire_format::WireType),
    BadDefaultValue(String),
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::EndOfStream => write!(f, "end of stream"),
            Error::Protobuf(ref e) => write!(f, "protobuf error: {}", e),
            Error::UnknownEnum(ref e) => write!(f, "unknown enum: {:?}", e),
            Error::UnknownEnumValue(v) => write!(f, "unknown enum value: {:?}", v),
            Error::UnknownMessage(ref m) => write!(f, "unknown message: {:?}", m),
            Error::BadWireType(wt) => write!(f, "bad wire type: {:?}", wt),
            Error::BadDefaultValue(ref d) => write!(f, "bad default value: {:?}", d),
            Error::Custom(ref m) => write!(f, "error: {}", m),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::EndOfStream => "end of stream",
            Error::Protobuf(_) => "protobuf",
            Error::UnknownEnum(_) => "unknown enum",
            Error::UnknownEnumValue(_) => "unknown enum value",
            Error::UnknownMessage(_) => "unknown message",
            Error::BadWireType(_) => "bad wire type",
            Error::BadDefaultValue(_) => "bad default value",
            Error::Custom(ref m) => m,
        }
    }
}

impl serde::Error for Error {
    fn custom<S>(msg: S) -> Error
        where S: Into<String>
    {
        Error::Custom(msg.into())
    }

    fn end_of_stream() -> Error {
        Error::EndOfStream
    }
}

impl From<protobuf::ProtobufError> for Error {
    fn from(e: protobuf::ProtobufError) -> Error {
        Error::Protobuf(e)
    }
}
