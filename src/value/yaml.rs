use crate::error;
use crate::value;
use serde_yaml;
use std::io;

#[derive(Debug)]
pub struct Source<R>(Option<R>);

#[derive(Debug)]
pub struct Sink<W>(W)
where
    W: io::Write;

#[inline]
pub fn source<R>(r: R) -> Source<R>
where
    R: io::Read,
{
    Source(Some(r))
}

#[inline]
pub fn sink<W>(w: W) -> Sink<W>
where
    W: io::Write,
{
    Sink(w)
}

impl<R> value::Source for Source<R>
where
    R: io::Read,
{
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        if let Some(r) = self.0.take() {
            match serde_yaml::from_reader(r) {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(error::Error::from(e)),
            }
        } else {
            Ok(None)
        }
    }
}

impl<W> value::Sink for Sink<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, value: value::Value) -> error::Result<()> {
        serde_yaml::to_writer(&mut self.0, &value)?;
        self.0.write_all(b"\n")?;
        Ok(())
    }
}
