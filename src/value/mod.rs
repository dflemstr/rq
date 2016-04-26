use std::collections;
use std::io;

use ordered_float;
use serde;
use serde_json;

use error;

pub mod cbor;
pub mod json;
pub mod protobuf;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
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

    F32(ordered_float::OrderedFloat<f32>),
    F64(ordered_float::OrderedFloat<f64>),

    Char(char),
    String(String),
    Bytes(Vec<u8>),

    Sequence(Vec<Value>),
    // TODO: Use a container that preserves insertion order
    Map(collections::BTreeMap<Value, Value>),
}

struct ValueVisitor;

impl Value {
    pub fn to_json<W>(&self, write: &mut W) -> error::Result<()>
        where W: io::Write
    {
        Ok(try!(serde_json::to_writer(write, self)))
    }

    pub fn from_f32(v: f32) -> Value {
        Value::F32(ordered_float::OrderedFloat(v))
    }

    pub fn from_f64(v: f64) -> Value {
        Value::F64(ordered_float::OrderedFloat(v))
    }
}

impl serde::ser::Serialize for Value {
    #[inline]
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: serde::ser::Serializer
    {
        match *self {
            Value::Unit => ().serialize(s),
            Value::Bool(v) => v.serialize(s),

            Value::ISize(v) => v.serialize(s),
            Value::I8(v) => v.serialize(s),
            Value::I16(v) => v.serialize(s),
            Value::I32(v) => v.serialize(s),
            Value::I64(v) => v.serialize(s),

            Value::USize(v) => v.serialize(s),
            Value::U8(v) => v.serialize(s),
            Value::U16(v) => v.serialize(s),
            Value::U32(v) => v.serialize(s),
            Value::U64(v) => v.serialize(s),

            Value::F32(v) => v.serialize(s),
            Value::F64(v) => v.serialize(s),

            Value::Char(v) => v.serialize(s),
            Value::String(ref v) => v.serialize(s),
            Value::Bytes(ref v) => v.serialize(s),

            Value::Sequence(ref v) => v.serialize(s),
            Value::Map(ref v) => v.serialize(s),
        }
    }
}

impl serde::de::Deserialize for Value {
    #[inline]
    fn deserialize<D>(d: &mut D) -> Result<Value, D::Error>
        where D: serde::Deserializer
    {
        d.deserialize(ValueVisitor)
    }
}

impl serde::de::Visitor for ValueVisitor {
    type Value = Value;

    #[inline]
    fn visit_bool<E>(&mut self, v: bool) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::Bool(v))
    }

    #[inline]
    fn visit_isize<E>(&mut self, v: isize) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::ISize(v))
    }

    #[inline]
    fn visit_i8<E>(&mut self, v: i8) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::I8(v))
    }

    #[inline]
    fn visit_i16<E>(&mut self, v: i16) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::I16(v))
    }

    #[inline]
    fn visit_i32<E>(&mut self, v: i32) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::I32(v))
    }

    #[inline]
    fn visit_i64<E>(&mut self, v: i64) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::I64(v))
    }

    #[inline]
    fn visit_usize<E>(&mut self, v: usize) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::USize(v))
    }

    #[inline]
    fn visit_u8<E>(&mut self, v: u8) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::U8(v))
    }

    #[inline]
    fn visit_u16<E>(&mut self, v: u16) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::U16(v))
    }

    #[inline]
    fn visit_u32<E>(&mut self, v: u32) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::U32(v))
    }

    #[inline]
    fn visit_u64<E>(&mut self, v: u64) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::U64(v))
    }

    #[inline]
    fn visit_f32<E>(&mut self, v: f32) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::from_f32(v))
    }

    #[inline]
    fn visit_f64<E>(&mut self, v: f64) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::from_f64(v))
    }

    #[inline]
    fn visit_char<E>(&mut self, v: char) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::Char(v))
    }

    #[inline]
    fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::String(v.to_owned()))
    }

    #[inline]
    fn visit_string<E>(&mut self, v: String) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::String(v))
    }

    #[inline]
    fn visit_unit<E>(&mut self) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::Unit)
    }

    #[inline]
    fn visit_none<E>(&mut self) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::Unit)
    }

    #[inline]
    fn visit_some<D>(&mut self, d: &mut D) -> Result<Self::Value, D::Error>
        where D: serde::de::Deserializer
    {
        serde::de::Deserialize::deserialize(d)
    }

    #[inline]
    fn visit_seq<V>(&mut self, v: V) -> Result<Self::Value, V::Error>
        where V: serde::de::SeqVisitor
    {
        let values = try!(serde::de::impls::VecVisitor::new().visit_seq(v));
        Ok(Value::Sequence(values))
    }

    #[inline]
    fn visit_map<V>(&mut self, v: V) -> Result<Self::Value, V::Error>
        where V: serde::de::MapVisitor
    {
        let values = try!(serde::de::impls::BTreeMapVisitor::new().visit_map(v));
        Ok(Value::Map(values))
    }

    #[inline]
    fn visit_bytes<E>(&mut self, v: &[u8]) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::Bytes(v.to_vec()))
    }

    #[inline]
    fn visit_byte_buf<E>(&mut self, v: Vec<u8>) -> Result<Self::Value, E>
        where E: serde::de::Error
    {
        Ok(Value::Bytes(v))
    }
}
