use crate::error;

use ordered_float;
use serde;
use serde_json;
use std::collections;
use std::fmt;
use std::io;

pub mod avro;
pub mod cbor;
pub mod csv;
#[cfg(feature = "hjson_serde_0_9_support")]
pub mod hjson;
pub mod json;
pub mod messagepack;
pub mod protobuf;
pub mod raw;
pub mod toml;
pub mod yaml;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Value {
    Unit,
    Bool(bool),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),

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
    pub fn to_json<W>(&self, mut w: &mut W) -> error::Result<()>
    where
        W: io::Write,
    {
        serde_json::to_writer(&mut w, self)?;
        w.write_all(&[10])?; // Newline
        Ok(())
    }

    pub fn from_f32(v: f32) -> Self {
        Self::F32(ordered_float::OrderedFloat(v))
    }

    pub fn from_f64(v: f64) -> Self {
        Self::F64(ordered_float::OrderedFloat(v))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Unit => write!(f, "()"),
            Self::Bool(v) => write!(f, "{}", v),

            Self::I8(v) => write!(f, "{}", v),
            Self::I16(v) => write!(f, "{}", v),
            Self::I32(v) => write!(f, "{}", v),
            Self::I64(v) => write!(f, "{}", v),

            Self::U8(v) => write!(f, "{}", v),
            Self::U16(v) => write!(f, "{}", v),
            Self::U32(v) => write!(f, "{}", v),
            Self::U64(v) => write!(f, "{}", v),

            Self::F32(v) => write!(f, "{}", v),
            Self::F64(v) => write!(f, "{}", v),

            Self::Char(v) => write!(f, "{}", v),
            Self::String(ref v) => write!(f, "{}", v),
            Self::Bytes(ref v) => {
                for b in v {
                    write!(f, "{:02x}", b)?;
                }
                Ok(())
            }

            Self::Sequence(ref seq) => {
                let mut needs_sep = false;
                write!(f, "[")?;
                for v in seq {
                    if needs_sep {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                    needs_sep = true;
                }
                write!(f, "]")?;
                Ok(())
            }
            Self::Map(ref map) => {
                let mut needs_sep = false;
                write!(f, "{{")?;
                for (k, v) in map {
                    if needs_sep {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                    needs_sep = true;
                }
                write!(f, "}}")?;
                Ok(())
            }
        }
    }
}

impl serde::ser::Serialize for Value {
    #[inline]
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match *self {
            Self::Unit => ().serialize(s),
            Self::Bool(v) => v.serialize(s),

            Self::I8(v) => v.serialize(s),
            Self::I16(v) => v.serialize(s),
            Self::I32(v) => v.serialize(s),
            Self::I64(v) => v.serialize(s),

            Self::U8(v) => v.serialize(s),
            Self::U16(v) => v.serialize(s),
            Self::U32(v) => v.serialize(s),
            Self::U64(v) => v.serialize(s),

            Self::F32(v) => v.serialize(s),
            Self::F64(v) => v.serialize(s),

            Self::Char(v) => v.serialize(s),
            Self::String(ref v) => v.serialize(s),
            Self::Bytes(ref v) => v.serialize(s),

            Self::Sequence(ref v) => v.serialize(s),
            Self::Map(ref v) => v.serialize(s),
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_any(ValueVisitor)
    }
}

impl<'de> serde::de::Visitor<'de> for ValueVisitor {
    type Value = Value;

    #[inline]
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "any value")
    }

    #[inline]
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bool(v))
    }

    #[inline]
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::I8(v))
    }

    #[inline]
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::I16(v))
    }

    #[inline]
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::I32(v))
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::I64(v))
    }

    #[inline]
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::U8(v))
    }

    #[inline]
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::U16(v))
    }

    #[inline]
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::U32(v))
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::U64(v))
    }

    #[inline]
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::from_f32(v))
    }

    #[inline]
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::from_f64(v))
    }

    #[inline]
    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Char(v))
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(v.to_owned()))
    }

    #[inline]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(v))
    }

    #[inline]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bytes(v.to_vec()))
    }

    #[inline]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bytes(v))
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Unit)
    }

    #[inline]
    fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        serde::de::Deserialize::deserialize(d)
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Unit)
    }

    #[inline]
    fn visit_seq<V>(self, mut v: V) -> Result<Self::Value, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let mut values = v.size_hint().map_or(Vec::new(), Vec::with_capacity);

        while let Some(element) = v.next_element()? {
            values.push(element);
        }

        Ok(Value::Sequence(values))
    }

    #[inline]
    fn visit_map<V>(self, mut v: V) -> Result<Self::Value, V::Error>
    where
        V: serde::de::MapAccess<'de>,
    {
        let mut values = collections::BTreeMap::new();

        while let Some((key, value)) = v.next_entry()? {
            values.insert(key, value);
        }

        Ok(Value::Map(values))
    }
}
