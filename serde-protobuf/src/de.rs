//! Deserialization of binary protocol buffer encoded data.
//!
//! All deserialization operations require a previously loaded set of schema descriptors; see the
//! [`descriptor`](../descriptor/index.html) module for more information.
//!
//! Provided that a set of descriptors have been loaded, a `Deserializer` can be used to deserialize
//! a stream of bytes into something that implements `Deserialize`.
//!
//! ```
//! extern crate serde;
//! extern crate protobuf;
//! extern crate serde_protobuf;
//! extern crate serde_value;
//!
//! use std::fs;
//! use serde::de::Deserialize;
//! use serde_protobuf::descriptor::Descriptors;
//! use serde_protobuf::de::Deserializer;
//! use serde_value::Value;
//!
//! # use std::io;
//! # #[derive(Debug)] struct Error;
//! # impl From<protobuf::ProtobufError> for Error {
//! #   fn from(a: protobuf::ProtobufError) -> Error {
//! #     Error
//! #   }
//! # }
//! # impl From<io::Error> for Error {
//! #   fn from(a: io::Error) -> Error {
//! #     Error
//! #   }
//! # }
//! # impl From<serde_protobuf::Error> for Error {
//! #   fn from(a: serde_protobuf::Error) -> Error {
//! #     Error
//! #   }
//! # }
//! # fn foo() -> Result<(), Error> {
//! // Load a descriptor registry (see descriptor module)
//! let mut file = fs::File::open("testdata/descriptors.pb")?;
//! let proto = protobuf::parse_from_reader(&mut file)?;
//! let descriptors = Descriptors::from_proto(&proto);
//!
//! // Set up some data to read
//! let data = &[8, 42];
//! let mut input = protobuf::CodedInputStream::from_bytes(data);
//!
//! // Create a deserializer
//! let name = ".protobuf_unittest.TestAllTypes";
//! let mut deserializer = Deserializer::for_named_message(&descriptors, name, input)?;
//!
//! // Deserialize some struct
//! let value = Value::deserialize(&mut deserializer)?;
//! # println!("{:?}", value);
//! # Ok(())
//! # }
//! # fn main() {
//! #   foo().unwrap();
//! # }
//! ```


use descriptor;
use error;

use protobuf;
use serde;
use std::collections;
use std::vec;
use value;

/// A deserializer that can deserialize a single message type.
pub struct Deserializer<'de> {
    descriptors: &'de descriptor::Descriptors,
    descriptor: &'de descriptor::MessageDescriptor,
    input: protobuf::CodedInputStream<'de>,
}

struct MessageVisitor<'de> {
    descriptors: &'de descriptor::Descriptors,
    descriptor: &'de descriptor::MessageDescriptor,
    fields: collections::btree_map::IntoIter<i32, value::Field>,
    field: Option<(&'de descriptor::FieldDescriptor, value::Field)>,
}

struct MessageKeyDeserializer<'de> {
    descriptor: &'de descriptor::FieldDescriptor,
}

struct MessageFieldDeserializer<'de> {
    descriptors: &'de descriptor::Descriptors,
    descriptor: &'de descriptor::FieldDescriptor,
    field: Option<value::Field>,
}

struct RepeatedValueVisitor<'de> {
    descriptors: &'de descriptor::Descriptors,
    descriptor: &'de descriptor::FieldDescriptor,
    values: vec::IntoIter<value::Value>,
}

struct ValueDeserializer<'de> {
    descriptors: &'de descriptor::Descriptors,
    descriptor: &'de descriptor::FieldDescriptor,
    value: Option<value::Value>,
}

impl<'de> Deserializer<'de> {
    /// Constructs a new protocol buffer deserializer for the specified message type.
    ///
    /// The caller must ensure that all of the information needed by the specified message
    /// descriptor is available in the associated descriptors registry.
    pub fn new(descriptors: &'de descriptor::Descriptors,
               descriptor: &'de descriptor::MessageDescriptor,
               input: protobuf::CodedInputStream<'de>)
               -> Deserializer<'de> {
        Deserializer {
            descriptors: descriptors,
            descriptor: descriptor,
            input: input,
        }
    }

    /// Constructs a new protocol buffer deserializer for the specified named message type.
    ///
    /// The message type name must be fully quailified (for example
    /// `".google.protobuf.FileDescriptorSet"`).
    pub fn for_named_message(descriptors: &'de descriptor::Descriptors,
                             message_name: &str,
                             input: protobuf::CodedInputStream<'de>)
                             -> error::Result<Deserializer<'de>> {
        if let Some(message) = descriptors.message_by_name(message_name) {
            Ok(Deserializer::new(descriptors, message, input))
        } else {
            Err(error::ErrorKind::UnknownMessage(message_name.to_owned()).into())
        }
    }
}

impl<'de, 'b> serde::Deserializer<'de> for &'b mut Deserializer<'de> {
    type Error = error::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de>
    {
        let mut message = value::Message::new(self.descriptor);
        message.merge_from(self.descriptors, self.descriptor, &mut self.input)?;
        visitor.visit_map(MessageVisitor::new(self.descriptors, self.descriptor, message))
    }
}

impl<'de> MessageVisitor<'de> {
    #[inline]
    fn new(descriptors: &'de descriptor::Descriptors,
           descriptor: &'de descriptor::MessageDescriptor,
           value: value::Message)
           -> MessageVisitor<'de> {
        MessageVisitor {
            descriptors: descriptors,
            descriptor: descriptor,
            fields: value.fields.into_iter(),
            field: None,
        }
    }
}

impl<'de> serde::de::MapAccess<'de> for MessageVisitor<'de> {
    type Error = error::Error;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> error::Result<Option<K::Value>>
        where K: serde::de::DeserializeSeed<'de>
    {
        if let Some((k, v)) = self.fields.next() {
            let descriptor = self.descriptor.field_by_number(k).expect("Lost track of field");
            let key = seed.deserialize(MessageKeyDeserializer::new(descriptor))?;
            self.field = Some((descriptor, v));
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> error::Result<V::Value>
        where V: serde::de::DeserializeSeed<'de>
    {
        let (descriptor, field) = self.field
            .take()
            .expect("visit_value was called before visit_key");

        seed.deserialize(MessageFieldDeserializer::new(self.descriptors, descriptor, field))
    }
}

impl<'de> MessageKeyDeserializer<'de> {
    #[inline]
    fn new(descriptor: &'de descriptor::FieldDescriptor) -> MessageKeyDeserializer<'de> {
        MessageKeyDeserializer { descriptor: descriptor }
    }
}

impl<'de> serde::Deserializer<'de> for MessageKeyDeserializer<'de> {
    type Error = error::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> error::Result<V::Value>
        where V: serde::de::Visitor<'de>
    {
        visitor.visit_str(self.descriptor.name())
    }
}

impl<'de> MessageFieldDeserializer<'de> {
    #[inline]
    fn new(descriptors: &'de descriptor::Descriptors,
           descriptor: &'de descriptor::FieldDescriptor,
           field: value::Field)
           -> MessageFieldDeserializer<'de> {
        MessageFieldDeserializer {
            descriptors: descriptors,
            descriptor: descriptor,
            field: Some(field),
        }
    }
}

impl<'de> serde::Deserializer<'de> for MessageFieldDeserializer<'de> {
    type Error = error::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> error::Result<V::Value>
        where V: serde::de::Visitor<'de>
    {
        let ds = self.descriptors;
        let d = self.descriptor;
        match self.field.take() {
            Some(value::Field::Singular(None)) => {
                if d.field_label() == descriptor::FieldLabel::Optional {
                    visitor.visit_none()
                } else {
                    visitor.visit_unit()
                }
            },
            Some(value::Field::Singular(Some(v))) => {
                if d.field_label() == descriptor::FieldLabel::Optional {
                    visitor.visit_some(ValueDeserializer::new(ds, d, v))
                } else {
                    visit_value(ds, d, v, visitor)
                }
            },
            Some(value::Field::Repeated(vs)) => {
                visitor.visit_seq(&mut RepeatedValueVisitor::new(ds, d, vs.into_iter()))
            },
            None => bail!(error::ErrorKind::EndOfStream),
        }
    }
}

impl<'de> RepeatedValueVisitor<'de> {
    #[inline]
    fn new(descriptors: &'de descriptor::Descriptors,
           descriptor: &'de descriptor::FieldDescriptor,
           values: vec::IntoIter<value::Value>)
           -> RepeatedValueVisitor<'de> {
        RepeatedValueVisitor {
            descriptors: descriptors,
            descriptor: descriptor,
            values: values,
        }
    }
}

impl<'de> serde::de::SeqAccess<'de> for RepeatedValueVisitor<'de> {
    type Error = error::Error;

    #[inline]
    fn next_element_seed<A>(&mut self, seed: A) -> error::Result<Option<A::Value>>
        where A: serde::de::DeserializeSeed<'de>
    {
        let ds = self.descriptors;
        let d = self.descriptor;
        match self.values.next() {
            Some(v) => Ok(Some(seed.deserialize(ValueDeserializer::new(ds, d, v))?)),
            None => Ok(None),
        }
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.values.size_hint().1
    }
}

impl<'de> ValueDeserializer<'de> {
    #[inline]
    fn new(descriptors: &'de descriptor::Descriptors,
           descriptor: &'de descriptor::FieldDescriptor,
           value: value::Value)
           -> ValueDeserializer<'de> {
        ValueDeserializer {
            descriptors: descriptors,
            descriptor: descriptor,
            value: Some(value),
        }
    }
}

impl<'de> serde::Deserializer<'de> for ValueDeserializer<'de> {
    type Error = error::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> error::Result<V::Value>
        where V: serde::de::Visitor<'de>
    {
        match self.value.take() {
            Some(value) => visit_value(self.descriptors, self.descriptor, value, visitor),
            None => bail!(error::ErrorKind::EndOfStream),
        }
    }
}

#[inline]
fn visit_value<'de, V>(descriptors: &'de descriptor::Descriptors,
                       descriptor: &'de descriptor::FieldDescriptor,
                       value: value::Value,
                       visitor: V)
                       -> error::Result<V::Value>
    where V: serde::de::Visitor<'de>
{
    match value {
        value::Value::Bool(v) => visitor.visit_bool(v),
        value::Value::I32(v) => visitor.visit_i32(v),
        value::Value::I64(v) => visitor.visit_i64(v),
        value::Value::U32(v) => visitor.visit_u32(v),
        value::Value::U64(v) => visitor.visit_u64(v),
        value::Value::F32(v) => visitor.visit_f32(v),
        value::Value::F64(v) => visitor.visit_f64(v),
        value::Value::Bytes(v) => visitor.visit_byte_buf(v),
        value::Value::String(v) => visitor.visit_string(v),
        value::Value::Message(m) => {
            if let descriptor::FieldType::Message(d) = descriptor.field_type(descriptors) {
                visitor.visit_map(MessageVisitor::new(descriptors, d, m))
            } else {
                panic!("A field with a message value doesn't have a message type!")
            }
        },
        value::Value::Enum(e) => {
            if let descriptor::FieldType::Enum(d) = descriptor.field_type(descriptors) {
                visitor.visit_str(d.value_by_number(e).unwrap().name())
            } else {
                panic!("A field with an enum value doesn't have an enum type!")
            }
        },
    }
}
