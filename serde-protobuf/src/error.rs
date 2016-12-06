use protobuf;
use protobuf::stream::wire_format;
use serde;

error_chain! {
    foreign_links {
        Protobuf(protobuf::ProtobufError);
    }

    errors {
        EndOfStream {
            description("end of stream")
            display("end of stream")
        }
        UnknownEnum(name: String) {
            description("unknown enum")
            display("unknown enum: {:?}", name)
        }
        UnknownEnumValue(value: i32) {
            description("unknown enum value")
            display("unknown enum value: {}", value)
        }
        UnknownMessage(name: String) {
            description("unknown message")
            display("unknown message: {:?}", name)
        }
        BadWireType(wire_type: wire_format::WireType) {
            description("bad wire type")
            display("bad wire type: {:?}", wire_type)
        }
        BadDefaultValue(default_value: String) {
            description("bad default value")
            display("bad default value: {:?}", default_value)
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
