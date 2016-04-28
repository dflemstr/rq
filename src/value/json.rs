use std::io;
use std::mem;

use serde;
use serde_json;

use error;
use value;

pub struct JsonSource<R>(serde_json::StreamDeserializer<value::Value, io::Bytes<R>>)
    where R: io::Read;

pub struct JsonSink<W>(Option<serde_json::Serializer<W>>)
    where W: io::Write;

#[inline]
pub fn source<R>(r: R) -> JsonSource<R> where R: io::Read {
    JsonSource(serde_json::StreamDeserializer::new(r.bytes()))
}

#[inline]
pub fn sink<W>(w: W) -> JsonSink<W> where W: io::Write {
    JsonSink(Some(serde_json::Serializer::new(w)))
}

impl<R> value::Source for JsonSource<R>
    where R: io::Read
{
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match self.0.next() {
            Some(Ok(v)) => Ok(Some(v)),
            Some(Err(e)) => Err(error::Error::from(e)),
            None => Ok(None),
        }
    }
}

impl<W> value::Sink for JsonSink<W>
    where W: io::Write
{
    #[inline]
    fn write(&mut self, v: value::Value) -> error::Result<()> {
        if let Some(ref mut w) = self.0 {
            try!(serde::Serialize::serialize(&v, w));
        }

        // Some juggling required here to get the underlying writer temporarily, to write a newline.
        let mut w = mem::replace(&mut self.0, None).unwrap().into_inner();
        let result = w.write_all(&[10]);
        mem::replace(&mut self.0, Some(serde_json::Serializer::new(w)));

        result.map_err(From::from)
    }
}
