use std::io;

use serde;
use toml;

use crate::error;
use crate::value;

#[derive(Debug)]
pub struct Source(Option<String>);

#[derive(Debug)]
pub struct Sink<W: io::Write>(W);

#[inline]
pub fn source<R>(mut r: R) -> error::Result<Source>
where
    R: io::Read,
{
    let mut string = String::new();
    r.read_to_string(&mut string)?;
    Ok(Source(Some(string)))
}

#[inline]
pub fn sink<W>(w: W) -> Sink<W>
where
    W: io::Write,
{
    Sink(w)
}

impl value::Source for Source {
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match self.0.take() {
            Some(v) => {
                let mut de = toml::de::Deserializer::new(v.as_str());
                match serde::Deserialize::deserialize(&mut de) {
                    Ok(v) => Ok(Some(v)),
                    Err(e) => Err(error::Error::from(e)),
                }
            }
            None => Ok(None),
        }
    }
}

impl<W> value::Sink for Sink<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, value: value::Value) -> error::Result<()> {
        let mut string = String::new();
        {
            let mut ser = toml::ser::Serializer::new(&mut string);
            serde::Serialize::serialize(&value, &mut ser)?;
        }

        self.0.write_all(string.as_bytes())?;
        self.0.write_all(b"\n")?;
        Ok(())
    }
}
