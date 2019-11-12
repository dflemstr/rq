use crate::error;

use crate::value;
use serde;
use serde_cbor;
use std::fmt;
use std::io;

pub struct Source<R>(serde_cbor::de::Deserializer<serde_cbor::de::IoRead<R>>)
where
    R: io::Read;

pub struct Sink<W>(serde_cbor::ser::Serializer<serde_cbor::ser::IoWrite<W>>)
where
    W: io::Write;

#[inline]
pub fn source<R>(r: R) -> Source<R>
where
    R: io::Read,
{
    Source(serde_cbor::de::Deserializer::new(
        serde_cbor::de::IoRead::new(r),
    ))
}

#[inline]
pub fn sink<W>(w: W) -> Sink<W>
where
    W: io::Write,
{
    Sink(serde_cbor::ser::Serializer::new(
        serde_cbor::ser::IoWrite::new(w),
    ))
}

impl<R> value::Source for Source<R>
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

impl<W> value::Sink for Sink<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, v: value::Value) -> error::Result<()> {
        serde::Serialize::serialize(&v, &mut self.0).map_err(From::from)
    }
}

impl<R> fmt::Debug for Source<R>
where
    R: io::Read,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CborSource").finish()
    }
}

impl<W> fmt::Debug for Sink<W>
where
    W: io::Write,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CborSink").finish()
    }
}
