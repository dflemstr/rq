use crate::error;

use crate::value;
use serde;
use serde_smile;
use std::fmt;
use std::io;

pub struct Source<R>(
    serde_smile::de::StreamDeserializer<
        'static,
        serde_smile::de::IoRead<io::BufReader<R>>,
        value::Value,
    >,
)
where
    R: io::Read;

pub struct Sink<W>(serde_smile::ser::Serializer<W>)
where
    W: io::Write;

#[inline]
pub fn source<R>(r: R) -> error::Result<Source<R>>
where
    R: io::Read,
{
    Ok(Source(
        serde_smile::de::Deserializer::new(serde_smile::de::IoRead::new(io::BufReader::new(r)))?
            .into_iter(),
    ))
}

#[inline]
pub fn sink<W>(w: W) -> error::Result<Sink<W>>
where
    W: io::Write,
{
    Ok(Sink(
        serde_smile::ser::Serializer::builder()
            .shared_strings(true)
            .build(w)?,
    ))
}

impl<R> value::Source for Source<R>
where
    R: io::Read,
{
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        Ok(self.0.next().transpose()?)
    }
}

impl<W> value::Sink for Sink<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, v: value::Value) -> error::Result<()> {
        Ok(serde::Serialize::serialize(&v, &mut self.0)?)
    }
}

impl<R> fmt::Debug for Source<R>
where
    R: io::Read,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SmileSource").finish()
    }
}

impl<W> fmt::Debug for Sink<W>
where
    W: io::Write,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SmileSink").finish()
    }
}
