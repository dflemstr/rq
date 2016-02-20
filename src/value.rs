use std::collections;
use std::io;

use serde;
use serde_json;

use error;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Unit,
    Bool(bool),

    ISize(isize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),

    USize(usize),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    F32(f32),
    F64(f64),

    Char(char),
    String(String),
    Bytes(Vec<u8>),

    Sequence(Vec<Value>),
    // TODO: Use a container that preserves insertion order
    Map(collections::BTreeMap<String, Value>),
}

impl Value {
    pub fn to_json<W>(&self, write: &mut W) -> error::Result<()>
          where W: io::Write {

        Ok(try!(serde_json::to_writer(write, self)))
    }
}

impl serde::ser::Serialize for Value {
    #[inline]
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
          where S: serde::ser::Serializer {

        match *self {
            Value::Unit => serializer.visit_unit(),
            Value::Bool(v) => v.serialize(serializer),

            Value::ISize(v) => v.serialize(serializer),
            Value::I8(v) => v.serialize(serializer),
            Value::I16(v) => v.serialize(serializer),
            Value::I32(v) => v.serialize(serializer),
            Value::I64(v) => v.serialize(serializer),

            Value::USize(v) => v.serialize(serializer),
            Value::U8(v) => v.serialize(serializer),
            Value::U16(v) => v.serialize(serializer),
            Value::U32(v) => v.serialize(serializer),
            Value::U64(v) => v.serialize(serializer),

            Value::F32(v) => v.serialize(serializer),
            Value::F64(v) => v.serialize(serializer),

            Value::Char(v) => v.serialize(serializer),
            Value::String(ref v) => v.serialize(serializer),
            Value::Bytes(ref v) => v.serialize(serializer),

            Value::Sequence(ref v) => v.serialize(serializer),
            Value::Map(ref v) => v.serialize(serializer),
        }
    }
}
