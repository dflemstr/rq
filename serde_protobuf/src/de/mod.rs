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
//! let mut file = try!(fs::File::open("testdata/descriptors.pb"));
//! let proto = try!(protobuf::parse_from_reader(&mut file));
//! let descriptors = Descriptors::from_proto(&proto);
//!
//! // Set up some data to read
//! let data = &[8, 42];
//! let mut input = protobuf::CodedInputStream::from_bytes(data);
//!
//! // Create a deserializer
//! let name = ".protobuf_unittest.TestAllTypes";
//! let mut deserializer = try!(Deserializer::for_named_message(&descriptors, name, &mut input));
//!
//! // Deserialize some struct
//! let value = try!(Value::deserialize(&mut deserializer));
//! # println!("{:?}", value);
//! # Ok(())
//! # }
//! # fn main() {
//! #   foo().unwrap();
//! # }
//! ```

use std::collections;
use std::vec;

use protobuf;
use serde;

use descriptor;
use error;
use value;

/// A deserializer that can deserialize a single message type.
pub struct Deserializer<'a> {
    descriptors: &'a descriptor::Descriptors,
    descriptor: &'a descriptor::MessageDescriptor,
    input: &'a mut protobuf::CodedInputStream<'a>,
}

struct MessageVisitor<'a> {
    descriptors: &'a descriptor::Descriptors,
    descriptor: &'a descriptor::MessageDescriptor,
    fields: collections::btree_map::IntoIter<i32, value::Field>,
    field: Option<(&'a descriptor::FieldDescriptor, value::Field)>,
}

struct MessageKeyDeserializer<'a> {
    descriptor: &'a descriptor::FieldDescriptor,
}

struct MessageFieldDeserializer<'a> {
    descriptors: &'a descriptor::Descriptors,
    descriptor: &'a descriptor::FieldDescriptor,
    field: Option<value::Field>,
}

struct RepeatedValueVisitor<'a> {
    descriptors: &'a descriptor::Descriptors,
    descriptor: &'a descriptor::FieldDescriptor,
    values: vec::IntoIter<value::Value>,
}

struct ValueDeserializer<'a> {
    descriptors: &'a descriptor::Descriptors,
    descriptor: &'a descriptor::FieldDescriptor,
    value: Option<value::Value>,
}

impl<'a> Deserializer<'a> {
    /// Constructs a new protocol buffer deserializer for the specified message type.
    ///
    /// The caller must ensure that all of the information needed by the specified message
    /// descriptor is available in the associated descriptors registry.
    pub fn new(descriptors: &'a descriptor::Descriptors,
               descriptor: &'a descriptor::MessageDescriptor,
               input: &'a mut protobuf::CodedInputStream<'a>)
               -> Deserializer<'a> {
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
    pub fn for_named_message(descriptors: &'a descriptor::Descriptors,
                             message_name: &str,
                             input: &'a mut protobuf::CodedInputStream<'a>)
                             -> error::Result<Deserializer<'a>> {
        if let Some(message) = descriptors.message_by_name(message_name) {
            Ok(Deserializer::new(descriptors, message, input))
        } else {
            Err(error::Error::UnknownMessage(message_name.to_owned()))
        }
    }
}

impl<'a> serde::Deserializer for Deserializer<'a> {
    type Error = error::Error;

    #[inline]
    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor
    {
        let mut message = value::Message::new(self.descriptor);
        try!(message.merge_from(self.descriptors, self.descriptor, self.input));
        visitor.visit_map(MessageVisitor::new(self.descriptors, self.descriptor, message))
    }
}

impl<'a> MessageVisitor<'a> {
    #[inline]
    fn new(descriptors: &'a descriptor::Descriptors,
           descriptor: &'a descriptor::MessageDescriptor,
           value: value::Message)
           -> MessageVisitor<'a> {
        MessageVisitor {
            descriptors: descriptors,
            descriptor: descriptor,
            fields: value.fields.into_iter(),
            field: None,
        }
    }
}

impl<'a> serde::de::MapVisitor for MessageVisitor<'a> {
    type Error = error::Error;

    #[inline]
    fn visit_key<K>(&mut self) -> error::Result<Option<K>>
        where K: serde::Deserialize
    {
        if let Some((k, v)) = self.fields.next() {
            let descriptor = self.descriptor.field_by_number(k).expect("Lost track of field");
            let key = try!(K::deserialize(&mut MessageKeyDeserializer::new(descriptor)));
            self.field = Some((descriptor, v));
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    #[inline]
    fn visit_value<V>(&mut self) -> error::Result<V>
        where V: serde::Deserialize
    {
        let (descriptor, field) = self.field
                                      .take()
                                      .expect("visit_value was called before visit_key");

        Ok(try!(V::deserialize(&mut MessageFieldDeserializer::new(self.descriptors,
                                                                  descriptor,
                                                                  field))))
    }

    #[inline]
    fn end(&mut self) -> error::Result<()> {
        Ok(())
    }
}

impl<'a> MessageKeyDeserializer<'a> {
    #[inline]
    fn new(descriptor: &'a descriptor::FieldDescriptor) -> MessageKeyDeserializer<'a> {
        MessageKeyDeserializer { descriptor: descriptor }
    }
}

impl<'a> serde::Deserializer for MessageKeyDeserializer<'a> {
    type Error = error::Error;

    #[inline]
    fn deserialize<V>(&mut self, mut visitor: V) -> error::Result<V::Value>
        where V: serde::de::Visitor
    {
        visitor.visit_str(self.descriptor.name())
    }

    #[inline]
    fn deserialize_i64<V>(&mut self, mut visitor: V) -> error::Result<V::Value>
        where V: serde::de::Visitor
    {
        visitor.visit_i32(self.descriptor.number())
    }

    #[inline]
    fn deserialize_u64<V>(&mut self, mut visitor: V) -> error::Result<V::Value>
        where V: serde::de::Visitor
    {
        visitor.visit_u32(self.descriptor.number() as u32)
    }
}

impl<'a> MessageFieldDeserializer<'a> {
    #[inline]
    fn new(descriptors: &'a descriptor::Descriptors,
           descriptor: &'a descriptor::FieldDescriptor,
           field: value::Field)
           -> MessageFieldDeserializer<'a> {
        MessageFieldDeserializer {
            descriptors: descriptors,
            descriptor: descriptor,
            field: Some(field),
        }
    }
}

impl<'a> serde::Deserializer for MessageFieldDeserializer<'a> {
    type Error = error::Error;

    #[inline]
    fn deserialize<V>(&mut self, mut visitor: V) -> error::Result<V::Value>
        where V: serde::de::Visitor
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
                    visitor.visit_some(&mut ValueDeserializer::new(ds, d, v))
                } else {
                    visit_value(ds, d, v, visitor)
                }
            },
            Some(value::Field::Repeated(vs)) => {
                visitor.visit_seq(&mut RepeatedValueVisitor::new(ds, d, vs.into_iter()))
            },
            None => Err(serde::de::Error::end_of_stream()),
        }
    }
}

impl<'a> RepeatedValueVisitor<'a> {
    #[inline]
    fn new(descriptors: &'a descriptor::Descriptors,
           descriptor: &'a descriptor::FieldDescriptor,
           values: vec::IntoIter<value::Value>)
           -> RepeatedValueVisitor<'a> {
        RepeatedValueVisitor {
            descriptors: descriptors,
            descriptor: descriptor,
            values: values,
        }
    }
}

impl<'a> serde::de::SeqVisitor for RepeatedValueVisitor<'a> {
    type Error = error::Error;

    #[inline]
    fn visit<A>(&mut self) -> error::Result<Option<A>>
        where A: serde::de::Deserialize
    {
        let ds = self.descriptors;
        let d = self.descriptor;
        match self.values.next() {
            Some(v) => Ok(Some(try!(A::deserialize(&mut ValueDeserializer::new(ds, d, v))))),
            None => Ok(None),
        }
    }

    #[inline]
    fn end(&mut self) -> error::Result<()> {
        let len = self.values.size_hint().0;
        if len == 0 { Ok(()) } else { Err(serde::de::Error::invalid_length(len)) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.values.size_hint()
    }
}

impl<'a> ValueDeserializer<'a> {
    #[inline]
    fn new(descriptors: &'a descriptor::Descriptors,
           descriptor: &'a descriptor::FieldDescriptor,
           value: value::Value)
           -> ValueDeserializer<'a> {
        ValueDeserializer {
            descriptors: descriptors,
            descriptor: descriptor,
            value: Some(value),
        }
    }
}

impl<'a> serde::Deserializer for ValueDeserializer<'a> {
    type Error = error::Error;

    #[inline]
    fn deserialize<V>(&mut self, visitor: V) -> error::Result<V::Value>
        where V: serde::de::Visitor
    {
        match self.value.take() {
            Some(value) => visit_value(self.descriptors, self.descriptor, value, visitor),
            None => Err(serde::de::Error::end_of_stream()),
        }
    }
}

#[inline]
fn visit_value<V>(descriptors: &descriptor::Descriptors,
                  descriptor: &descriptor::FieldDescriptor,
                  value: value::Value,
                  mut visitor: V)
                  -> error::Result<V::Value>
    where V: serde::de::Visitor
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
