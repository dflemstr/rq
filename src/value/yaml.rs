use error;
use serde_yaml;
use std::io;
use value;

#[derive(Debug)]
pub struct YamlSource<R>(Option<R>);

#[derive(Debug)]
pub struct YamlSink<W>(W)
where
    W: io::Write;

#[inline]
pub fn source<R>(r: R) -> YamlSource<R>
where
    R: io::Read,
{
    YamlSource(Some(r))
}

#[inline]
pub fn sink<W>(w: W) -> YamlSink<W>
where
    W: io::Write,
{
    YamlSink(w)
}

impl<R> value::Source for YamlSource<R>
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

impl<W> value::Sink for YamlSink<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, value: value::Value) -> error::Result<()> {
        try!(serde_yaml::to_writer(&mut self.0, &value));
        try!(self.0.write(b"\n"));
        Ok(())
    }
}
