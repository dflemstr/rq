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
    fn write(&mut self, mut value: value::Value) -> error::Result<()> {
        enforce_toml_output_order(&mut value);
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

fn enforce_toml_output_order(value: &mut value::Value) {
    match value {
        value::Value::Sequence(seq) => seq.iter_mut().for_each(enforce_toml_output_order),
        value::Value::Map(map) => {
            map.iter_mut()
                .for_each(|(_, v)| enforce_toml_output_order(v));
            map.sort_by_key(|(_, v)| Category::of(v));
        }
        _ => (),
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Category {
    Primitive,
    Array,
    Table,
}

impl Category {
    fn of(value: &value::Value) -> Category {
        match value {
            value::Value::Unit
            | value::Value::Bool(_)
            | value::Value::I8(_)
            | value::Value::I16(_)
            | value::Value::I32(_)
            | value::Value::I64(_)
            | value::Value::U8(_)
            | value::Value::U16(_)
            | value::Value::U32(_)
            | value::Value::U64(_)
            | value::Value::F32(_)
            | value::Value::F64(_)
            | value::Value::Char(_)
            | value::Value::String(_)
            | value::Value::Bytes(_) => Category::Primitive,
            value::Value::Sequence(_) => Category::Array,
            value::Value::Map(_) => Category::Table,
        }
    }
}
