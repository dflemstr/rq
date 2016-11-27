use serde;
use serde_json;
use std::io;
use std::string;

error_chain! {
    foreign_links {
        io::Error, IO;
        string::FromUtf8Error, Utf8;
        serde_json::Error, JsonDecode;
    }

    errors {
        NoSuchType(type_name: String) {
            description("no such type")
            display("no such type: {:?}", type_name)
        }
        InvalidSchema {
            description("invalid schema")
            display("invalid schema")
        }
        BadFileMagic(magic: Vec<u8>) {
            description("bad file magic")
            display("bad file magic: {:?}", magic)
        }
        NoSchema {
            description("the container has no schema")
            display("the container has no schema")
        }
        NoRootType {
            description("the container schema has no root type")
            display("the container schema has no root type")
        }
        UnsupportedCodec(codec: String) {
            description("unsupported codec")
            display("unsupported codec: {:?}", codec)
        }
        FieldTypeMismatch(field: &'static str, expected: &'static str) {
            description("unexpected field type")
            display("the value for {:?} was not a {}", field, expected)
        }
        RequiredFieldMissing(field: &'static str) {
            description("required field missing")
            display("the {:?} field is required", field)
        }
        DuplicateSchema(name: String) {
            description("there are two schemata with the same name")
            display("there are two schemata called {:?}", name)
        }
        EndOfStream {
            description("end of stream")
            display("end of stream")
        }
        IntegerOverflow {
            description("integer overflow")
            display("integer overflow")
        }
        NegativeLength {
            description("negative length")
            display("negative length")
        }
    }
}

impl serde::Error for Error {
    fn custom<S>(msg: S) -> Error
        where S: Into<String>
    {
        msg.into().into()
    }

    fn end_of_stream() -> Error {
        ErrorKind::EndOfStream.into()
    }
}
