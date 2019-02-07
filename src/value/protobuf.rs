use std::fmt;

use error;
use protobuf;
use serde;

use serde_protobuf;
use serde_protobuf::descriptor;
use value;

pub struct ProtobufSource<'a>(serde_protobuf::de::Deserializer<'a>, bool);

#[inline]
pub fn source<'a>(
    descriptors: &'a descriptor::Descriptors,
    message_name: &str,
    input: protobuf::CodedInputStream<'a>,
) -> error::Result<ProtobufSource<'a>> {
    let de = try!(serde_protobuf::de::Deserializer::for_named_message(
        descriptors,
        message_name,
        input,
    ));
    Ok(ProtobufSource(de, true))
}

impl<'a> value::Source for ProtobufSource<'a> {
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        if self.1 {
            self.1 = false;
            match serde::Deserialize::deserialize(&mut self.0).map_err(|e| e.into_error()) {
                Ok(v) => Ok(Some(v)),
                Err(serde_protobuf::error::Error::EndOfStream) => Ok(None),
                Err(e) => Err(error::Error::from(e)),
            }
        } else {
            Ok(None)
        }
    }
}

impl<'a> fmt::Debug for ProtobufSource<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ProtobufSource").finish()
    }
}
