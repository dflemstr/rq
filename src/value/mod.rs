use error;

use ordered_float;
use serde;
use serde_json;
use std::collections;
use std::fmt;
use std::io;

pub mod avro;
pub mod cbor;
pub mod hjson;
pub mod json;
pub mod messagepack;
pub mod protobuf;
pub mod toml;
pub mod yaml;

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

pub trait Source {
    fn read(&mut self) -> error::Result<Option<Value>>;
}

pub trait Sink {
    fn write(&mut self, v: Value) -> error::Result<()>;
}

struct ValueVisitor;

impl Value {
    pub fn to_json<W>(&self, w: &mut W) -> error::Result<()>
        where W: io::Write
    {
        try!(serde_json::to_writer(w, self));
        try!(w.write(&[10])); // Newline
        Ok(())
    }

    pub fn from_f32(v: f32) -> Value {
        Value::F32(ordered_float::OrderedFloat(v))
    }

    pub fn from_f64(v: f64) -> Value {
        Value::F64(ordered_float::OrderedFloat(v))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Unit => write!(f, "()"),
            Value::Bool(v) => write!(f, "{}", v),

            Value::ISize(v) => write!(f, "{}", v),
            Value::I8(v) => write!(f, "{}", v),
            Value::I16(v) => write!(f, "{}", v),
            Value::I32(v) => write!(f, "{}", v),
            Value::I64(v) => write!(f, "{}", v),

            Value::USize(v) => write!(f, "{}", v),
            Value::U8(v) => write!(f, "{}", v),
            Value::U16(v) => write!(f, "{}", v),
            Value::U32(v) => write!(f, "{}", v),
            Value::U64(v) => write!(f, "{}", v),

            Value::F32(v) => write!(f, "{}", v),
            Value::F64(v) => write!(f, "{}", v),

            Value::Char(v) => write!(f, "{}", v),
            Value::String(ref v) => write!(f, "{}", v),
            Value::Bytes(ref v) => {
                for b in v {
                    try!(write!(f, "{:02x}", b));
                }
                Ok(())
            },

            Value::Sequence(ref seq) => {
                let mut needs_sep = false;
                try!(write!(f, "["));
                for v in seq {
                    if needs_sep {
                        try!(write!(f, ", "));
                    }
                    try!(write!(f, "{}", v));
                    needs_sep = true;
                }
                try!(write!(f, "]"));
                Ok(())
            },
            Value::Map(ref map) => {
                let mut needs_sep = false;
                try!(write!(f, "{{"));
                for (k, v) in map {
                    if needs_sep {
                        try!(write!(f, ", "));
                    }
                    try!(write!(f, "{}: {}", k, v));
                    needs_sep = true;
                }
                try!(write!(f, "}}"));
                Ok(())
            },
        }
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
