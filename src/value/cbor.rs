use error;

use serde;
use serde_cbor;
use std::fmt;
use std::io;
use value;

pub struct CborSource<R>(serde_cbor::de::Deserializer<serde_cbor::de::IoRead<R>>)
where
    R: io::Read;

pub struct CborSink<W>(serde_cbor::ser::Serializer<W>)
where
    W: io::Write;

#[inline]
pub fn source<R>(r: R) -> CborSource<R>
where
    R: io::Read,
{
    CborSource(serde_cbor::de::Deserializer::new(
        serde_cbor::de::IoRead::new(r),
    ))
}

#[inline]
pub fn sink<W>(w: W) -> CborSink<W>
where
    W: io::Write,
{
    CborSink(serde_cbor::ser::Serializer::new(w))
}

impl<R> value::Source for CborSource<R>
where
    R: io::Read,
{
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match serde::Deserialize::deserialize(&mut self.0) {
            Ok(v) => Ok(Some(v)),
            Err(e) => match e.classify() {
                serde_cbor::error::Category::Eof => Ok(None),
                _ => Err(error::Error::from(e)),
            },
        }
    }
}

impl<W> value::Sink for CborSink<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, v: value::Value) -> error::Result<()> {
        serde::Serialize::serialize(&v, &mut self.0).map_err(From::from)
    }
}

impl<R> fmt::Debug for CborSource<R>
where
    R: io::Read,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CborSource").finish()
    }
}

impl<W> fmt::Debug for CborSink<W>
where
    W: io::Write,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CborSink").finish()
    }
}
