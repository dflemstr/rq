use std::collections;
use std::vec;

use protobuf;
use serde;

use descriptor;
use error;
use value;

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
        if try!(self.input.eof()) {
            Err(serde::de::Error::end_of_stream())
        } else {
            let mut message = value::Message::new(self.descriptor);
            try!(message.merge_from(self.descriptors, self.descriptor, self.input));
            visitor.visit_map(MessageVisitor::new(self.descriptors, self.descriptor, message))
        }
    }
}

impl<'a> MessageVisitor<'a> {
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

    fn end(&mut self) -> error::Result<()> {
        Ok(())
    }
}

impl<'a> MessageKeyDeserializer<'a> {
    fn new(descriptor: &'a descriptor::FieldDescriptor) -> MessageKeyDeserializer<'a> {
        MessageKeyDeserializer { descriptor: descriptor }
    }
}

impl<'a> serde::Deserializer for MessageKeyDeserializer<'a> {
    type Error = error::Error;

    fn deserialize<V>(&mut self, mut visitor: V) -> error::Result<V::Value>
        where V: serde::de::Visitor
    {
        visitor.visit_str(self.descriptor.name())
    }

    fn deserialize_i64<V>(&mut self, mut visitor: V) -> error::Result<V::Value>
        where V: serde::de::Visitor
    {
        visitor.visit_i32(self.descriptor.number())
    }

    fn deserialize_u64<V>(&mut self, mut visitor: V) -> error::Result<V::Value>
        where V: serde::de::Visitor
    {
        visitor.visit_u32(self.descriptor.number() as u32)
    }
}

impl<'a> MessageFieldDeserializer<'a> {
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

    fn end(&mut self) -> error::Result<()> {
        let len = self.values.size_hint().0;
        if len == 0 { Ok(()) } else { Err(serde::de::Error::invalid_length(len)) }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.values.size_hint()
    }
}

impl<'a> ValueDeserializer<'a> {
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

    fn deserialize<V>(&mut self, visitor: V) -> error::Result<V::Value>
        where V: serde::de::Visitor
    {
        match self.value.take() {
            Some(value) => visit_value(self.descriptors, self.descriptor, value, visitor),
            None => Err(serde::de::Error::end_of_stream()),
        }
    }
}

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
