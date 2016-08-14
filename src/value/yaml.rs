use std::io;
use std::vec;

use serde;
use serde_yaml;
use yaml_rust;

use error;
use value;

pub struct YamlSource(vec::IntoIter<yaml_rust::Yaml>);
pub struct YamlSink<W>(W) where W: io::Write;

#[inline]
pub fn source<R>(mut r: R) -> error::Result<YamlSource> where R: io::Read {
    let mut string = String::new();
    try!(r.read_to_string(&mut string));
    let values = try!(yaml_rust::YamlLoader::load_from_str(&string));
    Ok(YamlSource(values.into_iter()))
}

#[inline]
pub fn sink<W>(w: W) -> YamlSink<W> where W: io::Write {
    YamlSink(w)
}

impl value::Source for YamlSource {
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match self.0.next() {
            Some(v) => {
                let mut de = serde_yaml::Deserializer::new(&v);
                match serde::Deserialize::deserialize(&mut de) {
                    Ok(v) => Ok(Some(v)),
                    Err(e) => Err(error::Error::from(e)),
                }
            },
            None => Ok(None),
        }
    }
}

impl<W> value::Sink for YamlSink<W> where W: io::Write {
    #[inline]
    fn write(&mut self, value: value::Value) -> error::Result<()> {
        try!(serde_yaml::ser::to_writer(&mut self.0, &value));
        try!(self.0.write(b"\n"));
        Ok(())
    }
}
