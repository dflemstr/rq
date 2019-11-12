use std::fmt;

use crate::error;
use protobuf;
use serde;

use crate::value;
use serde_protobuf;
use serde_protobuf::descriptor;

pub struct Source<'a>(serde_protobuf::de::Deserializer<'a>, bool);

#[inline]
pub fn source<'a>(
    descriptors: &'a descriptor::Descriptors,
    message_name: &str,
    input: protobuf::CodedInputStream<'a>,
) -> error::Result<Source<'a>> {
    let de = serde_protobuf::de::Deserializer::for_named_message(descriptors, message_name, input)?;
    Ok(Source(de, true))
}

impl<'a> value::Source for Source<'a> {
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        if self.1 {
            self.1 = false;
            match serde::Deserialize::deserialize(&mut self.0)
                .map_err(serde_protobuf::error::CompatError::into_error)
            {
                Ok(v) => Ok(Some(v)),
                Err(serde_protobuf::error::Error::EndOfStream) => Ok(None),
                Err(e) => Err(error::Error::from(e)),
            }
        } else {
            Ok(None)
        }
    }
}

impl<'a> fmt::Debug for Source<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ProtobufSource").finish()
    }
}
