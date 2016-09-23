use std::io;

use serde;
use toml;

use error;
use value;

pub struct TomlSource(Option<toml::Table>);
pub struct TomlSink<W: io::Write>(W);

#[inline]
pub fn source<R>(mut r: R) -> error::Result<TomlSource>
    where R: io::Read
{
    let mut string = String::new();
    try!(r.read_to_string(&mut string));
    let mut parser = toml::Parser::new(&string);
    let table = parser.parse();
    let table = match table {
        Some(t) => t,
        None => {
            return Err(parser.errors.remove(0).into());
        }
    };
    Ok(TomlSource(Some(table)))
}

#[inline]
pub fn sink<W>(w: W) -> TomlSink<W>
    where W: io::Write
{
    TomlSink(w)
}

impl value::Source for TomlSource {
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match self.0.take() {
            Some(v) => {
                let mut de = toml::Decoder::new(toml::Value::Table(v));
                match serde::Deserialize::deserialize(&mut de) {
                    Ok(v) => Ok(Some(v)),
                    Err(e) => Err(error::Error::from(e)),
                }
            },
            None => Ok(None),
        }
    }
}

impl<W> value::Sink for TomlSink<W>
    where W: io::Write
{
    #[inline]
    fn write(&mut self, value: value::Value) -> error::Result<()> {
        let mut e = toml::Encoder::new();
        try!(serde::Serialize::serialize(&value, &mut e));
        try!(write!(self.0, "{}", toml::Value::Table(e.toml).to_string()));
        try!(self.0.write(b"\n"));
        Ok(())
    }
}
