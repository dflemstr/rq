use csv;
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
use yaml_rust;

use std::result;

pub type Result<A> = result::Result<A, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "protobuf error")]
    Protobuf(#[cause] serde_protobuf::error::Error),
    #[fail(display = "IO error")]
    Io(#[cause] io::Error),
    #[fail(display = "UTF-8 error")]
    Utf8(#[cause] string::FromUtf8Error),
    #[fail(display = "native protobuf error")]
    NativeProtobuf(#[cause] protobuf::ProtobufError),
    #[fail(display = "MessagePack encode error")]
    MessagePackEncode(#[cause] rmpv::encode::Error),
    #[fail(display = "Avro error")]
    Avro(#[cause] Avro),
    #[fail(display = "CBOR error")]
    Cbor(#[cause] serde_cbor::error::Error),
    #[fail(display = "HJSON error")]
    Hjson(#[cause] serde_hjson::Error),
    #[fail(display = "JSON error")]
    Json(#[cause] serde_json::Error),
    #[fail(display = "YAML error")]
    Yaml(#[cause] serde_yaml::Error),
    #[fail(display = "YAML scan error")]
    YamlScan(#[cause] yaml_rust::ScanError),
    #[fail(display = "TOML deserialize error")]
    TomlDeserialize(#[cause] toml::de::Error),
    #[fail(display = "TOML serialize error")]
    TomlSerialize(#[cause] toml::ser::Error),
    #[fail(display = "SMILE error")]
    Smile(#[cause] serde_smile::Error),
    #[fail(display = "glob error")]
    Glob(#[cause] glob::GlobError),
    #[fail(display = "glob pattern error")]
    GlobPattern(#[cause] glob::PatternError),
    #[fail(display = "CSV error")]
    Csv(#[cause] csv::Error),
    #[fail(display = "MessagePack decode error")]
    MessagePackDecode(#[cause] rmpv::decode::Error),
    #[fail(display = "unimplemented: {}", msg)]
    Unimplemented { msg: String },
    #[fail(display = "illegal state: {}", msg)]
    IllegalState { msg: String },
    #[fail(display = "format error: {}", msg)]
    Format { msg: String },
    #[fail(display = "internal error: {}", _0)]
    Internal(&'static str),
    #[fail(display = "{}", _0)]
    Message(String),
}

#[derive(Debug, Fail)]
pub enum Avro {
    #[fail(display = "decode error")]
    Decode(#[cause] avro_rs::DecodeError),
    #[fail(display = "error when parsing schema")]
    ParseSchema(#[cause] avro_rs::ParseSchemaError),
    #[fail(display = "schema resolution error")]
    SchemaResolution(#[cause] avro_rs::SchemaResolutionError),
    #[fail(display = "validation error")]
    Validation(#[cause] avro_rs::ValidationError),
    #[fail(display = "{}", message)]
    Custom { message: String },
}

impl Error {
    pub fn unimplemented(msg: String) -> Self {
        Self::Unimplemented { msg }
    }

    pub fn illegal_state(msg: String) -> Self {
        Self::IllegalState { msg }
    }
}

impl Avro {
    pub fn downcast(error: failure::Error) -> Self {
        let error = match error.downcast::<avro_rs::DecodeError>() {
            Ok(error) => return Self::Decode(error),
            Err(error) => error,
        };

        let error = match error.downcast::<avro_rs::ParseSchemaError>() {
            Ok(error) => return Self::ParseSchema(error),
            Err(error) => error,
        };

        let error = match error.downcast::<avro_rs::SchemaResolutionError>() {
            Ok(error) => return Self::SchemaResolution(error),
            Err(error) => error,
        };

        let error = match error.downcast::<avro_rs::ValidationError>() {
            Ok(error) => return Self::Validation(error),
            Err(error) => error,
        };

        Self::Custom {
            message: error.to_string(),
        }
    }
}

macro_rules! gen_from {
    ($t:ty, $i:ident) => {
        impl From<$t> for Error {
            fn from(e: $t) -> Self {
                Self::$i(e)
            }
        }
    };
}

gen_from!(serde_protobuf::error::Error, Protobuf);
gen_from!(io::Error, Io);
#[cfg(feature = "js")]
gen_from!(v8::error::Error, Js);
gen_from!(string::FromUtf8Error, Utf8);
gen_from!(protobuf::ProtobufError, NativeProtobuf);
gen_from!(rmpv::encode::Error, MessagePackEncode);
gen_from!(serde_cbor::error::Error, Cbor);
gen_from!(serde_hjson::Error, Hjson);
gen_from!(serde_json::Error, Json);
gen_from!(serde_yaml::Error, Yaml);
gen_from!(yaml_rust::ScanError, YamlScan);
gen_from!(toml::de::Error, TomlDeserialize);
gen_from!(toml::ser::Error, TomlSerialize);
gen_from!(serde_smile::Error, Smile);
gen_from!(glob::GlobError, Glob);
gen_from!(glob::PatternError, GlobPattern);
gen_from!(csv::Error, Csv);
gen_from!(rmpv::decode::Error, MessagePackDecode);
