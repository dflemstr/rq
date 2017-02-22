use error;
use serde;
use serde_xml;
use std::io;
use value;

pub struct XmlSource<R>(serde_xml::Deserializer<io::Bytes<R>>) where R: io::Read;

#[inline]
pub fn source<R>(r: R) -> error::Result<XmlSource<R>>
    where R: io::Read
{
    Ok(XmlSource(serde_xml::Deserializer::new(r.bytes())))
}

impl<R> value::Source for XmlSource<R> where R: io::Read {
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match serde::Deserialize::deserialize(&mut self.0) {
            Ok(v) => Ok(Some(v)),
            Err(serde_xml::Error::SyntaxError(serde_xml::ErrorCode::EOF, _, _)) => Ok(None),
            Err(e) => Err(error::Error::from(e)),
        }
    }
}
