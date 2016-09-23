

use error;

use serde;
use serde_cbor;
use std::io;
use value;

pub struct CborSource<R>(serde_cbor::de::Deserializer<R>) where R: io::Read;

pub struct CborSink<W>(serde_cbor::ser::Serializer<W>) where W: io::Write;

#[inline]
pub fn source<R>(r: R) -> CborSource<R>
    where R: io::Read
{
    CborSource(serde_cbor::de::Deserializer::new(r))
}

#[inline]
pub fn sink<W>(w: W) -> CborSink<W>
    where W: io::Write
{
    CborSink(serde_cbor::ser::Serializer::new(w))
}

impl<R> value::Source for CborSource<R>
    where R: io::Read
{
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match serde::Deserialize::deserialize(&mut self.0) {
            Ok(v) => Ok(Some(v)),
            Err(serde_cbor::error::Error::Eof) => Ok(None),
            Err(e) => Err(error::Error::from(e)),
        }
    }
}

impl<W> value::Sink for CborSink<W>
    where W: io::Write
{
    #[inline]
    fn write(&mut self, v: value::Value) -> error::Result<()> {
        serde::Serialize::serialize(&v, &mut self.0).map_err(From::from)
    }
}
