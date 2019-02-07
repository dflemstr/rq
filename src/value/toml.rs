use std::io;

use serde;
use toml;

use crate::error;
use crate::value;

#[derive(Debug)]
pub struct TomlSource(Option<String>);

#[derive(Debug)]
pub struct TomlSink<W: io::Write>(W);

#[inline]
pub fn source<R>(mut r: R) -> error::Result<TomlSource>
where
    R: io::Read,
{
    let mut string = String::new();
    r#try!(r.read_to_string(&mut string));
    Ok(TomlSource(Some(string)))
}

#[inline]
pub fn sink<W>(w: W) -> TomlSink<W>
where
    W: io::Write,
{
    TomlSink(w)
}

impl value::Source for TomlSource {
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

impl<W> value::Sink for TomlSink<W>
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
        r#try!(self.0.write_all(b"\n"));
        Ok(())
    }
}
