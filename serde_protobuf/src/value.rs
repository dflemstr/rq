use std::collections;

use protobuf;
use protobuf::stream::wire_format;

use descriptor;
use error;

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bytes(Vec<u8>),
    String(String),
    Enum(i32),
    Message(Message),
}

#[derive(Clone, Debug)]
pub struct Message {
    pub fields: collections::BTreeMap<i32, Field>,
    pub unknown: protobuf::UnknownFields,
}

#[derive(Clone, Debug)]
pub enum Field {
    Singular(Option<Value>),
    Repeated(Vec<Value>),
}

impl Message {
    #[inline]
    pub fn new(message: &descriptor::MessageDescriptor) -> Message {
        let mut m = Message {
            fields: collections::BTreeMap::new(),
            unknown: protobuf::UnknownFields::new(),
        };

        for field in message.fields() {
            m.fields.insert(field.number(), if field.is_repeated() {
                Field::Repeated(Vec::new())
            } else {
                Field::Singular(field.default_value().cloned())
            });
        }

        m
    }

    #[inline]
    pub fn merge_from(&mut self,
                      descriptors: &descriptor::Descriptors,
                      message: &descriptor::MessageDescriptor,
                      input: &mut protobuf::CodedInputStream)
                      -> error::Result<()> {
        while !try!(input.eof()) {
            let (number, wire_type) = try!(input.read_tag_unpack());

            if let Some(field) = message.field_by_number(number as i32) {
                let value = self.ensure_field(field);
                try!(value.merge_from(descriptors, field, input, wire_type));
            } else {
                use protobuf::rt::read_unknown_or_skip_group as u;
                try!(u(number, wire_type, input, &mut self.unknown));
            }
        }
        Ok(())
    }

    #[inline]
    fn ensure_field(&mut self, field: &descriptor::FieldDescriptor) -> &mut Field {
        self.fields.entry(field.number()).or_insert_with(|| Field::new(field))
    }
}

impl Field {
    #[inline]
    pub fn new(field: &descriptor::FieldDescriptor) -> Field {
        if field.is_repeated() { Field::Repeated(Vec::new()) } else { Field::Singular(None) }
    }

    #[inline]
    pub fn merge_from(&mut self,
                      descriptors: &descriptor::Descriptors,
                      field: &descriptor::FieldDescriptor,
                      input: &mut protobuf::CodedInputStream,
                      wire_type: protobuf::stream::wire_format::WireType)
                      -> error::Result<()> {
        // Make the type dispatch below more compact
        use descriptor::FieldType::*;
        use protobuf::CodedInputStream as I;
        use protobuf::stream::wire_format::WireType::*;

        // Singular scalar
        macro_rules! ss {
            ($expected_wire_type:expr, $visit_func:expr, $reader:expr) => {
                self.merge_scalar(input, wire_type, $expected_wire_type, $visit_func, $reader)
            }
        }

        // Packable scalar
        macro_rules! ps {
            ($expected_wire_type:expr, $visit_func:expr, $reader:expr) => {
                self.merge_packable_scalar(input, wire_type, $expected_wire_type, $visit_func, $reader)
            };
            ($expected_wire_type:expr, $size:expr, $visit_func:expr, $reader:expr) => {
        // TODO: use size to pre-allocate buffer space
                self.merge_packable_scalar(input, wire_type, $expected_wire_type, $visit_func, $reader)
            }
        }

        match field.field_type(descriptors) {
            Bool => ps!(WireTypeVarint, Value::Bool, I::read_bool),
            Int32 => ps!(WireTypeVarint, Value::I32, I::read_int32),
            Int64 => ps!(WireTypeVarint, Value::I64, I::read_int64),
            SInt32 => ps!(WireTypeVarint, Value::I32, I::read_sint32),
            SInt64 => ps!(WireTypeVarint, Value::I64, I::read_sint64),
            UInt32 => ps!(WireTypeVarint, Value::U32, I::read_uint32),
            UInt64 => ps!(WireTypeVarint, Value::U64, I::read_uint64),
            Fixed32 => ps!(WireTypeFixed32, 4, Value::U32, I::read_fixed32),
            Fixed64 => ps!(WireTypeFixed64, 8, Value::U64, I::read_fixed64),
            SFixed32 => ps!(WireTypeFixed32, 4, Value::I32, I::read_sfixed32),
            SFixed64 => ps!(WireTypeFixed64, 8, Value::I64, I::read_sfixed64),
            Float => ps!(WireTypeFixed32, 4, Value::F32, I::read_float),
            Double => ps!(WireTypeFixed64, 8, Value::F64, I::read_double),
            Bytes => ss!(WireTypeLengthDelimited, Value::Bytes, I::read_bytes),
            String => ss!(WireTypeLengthDelimited, Value::String, I::read_string),
            Enum(_) => self.merge_enum(input, wire_type),
            Message(ref m) => self.merge_message(input, descriptors, m, wire_type),
            Group => unimplemented!(),
            UnresolvedEnum(e) => Err(error::Error::UnknownEnum(e.to_owned())),
            UnresolvedMessage(m) => Err(error::Error::UnknownMessage(m.to_owned())),
        }
    }

    #[inline]
    fn merge_scalar<'a, A, V, R>(&mut self,
                                 input: &mut protobuf::CodedInputStream<'a>,
                                 actual_wire_type: wire_format::WireType,
                                 expected_wire_type: wire_format::WireType,
                                 value_ctor: V,
                                 reader: R)
                                 -> error::Result<()>
        where V: Fn(A) -> Value,
              R: Fn(&mut protobuf::CodedInputStream<'a>) -> protobuf::ProtobufResult<A>
    {
        if expected_wire_type == actual_wire_type {
            self.put(value_ctor(try!(reader(input))));
            Ok(())
        } else {
            Err(error::Error::BadWireType(actual_wire_type))
        }
    }

    #[inline]
    fn merge_packable_scalar<'a, A, V, R>(&mut self,
                                          input: &mut protobuf::CodedInputStream<'a>,
                                          actual_wire_type: wire_format::WireType,
                                          expected_wire_type: wire_format::WireType,
                                          value_ctor: V,
                                          reader: R)
                                          -> error::Result<()>
        where V: Fn(A) -> Value,
              R: Fn(&mut protobuf::CodedInputStream<'a>) -> protobuf::ProtobufResult<A>
    {
        if wire_format::WireType::WireTypeLengthDelimited == actual_wire_type {
            let len = try!(input.read_raw_varint32());

            let old_limit = try!(input.push_limit(len));
            while !try!(input.eof()) {
                self.put(value_ctor(try!(reader(input))));
            }
            input.pop_limit(old_limit);

            Ok(())
        } else {
            self.merge_scalar(input,
                              actual_wire_type,
                              expected_wire_type,
                              value_ctor,
                              reader)
        }
    }

    #[inline]
    fn merge_enum(&mut self,
                  input: &mut protobuf::CodedInputStream,
                  actual_wire_type: wire_format::WireType)
                  -> error::Result<()> {
        if wire_format::WireType::WireTypeVarint == actual_wire_type {
            let v = try!(input.read_raw_varint32()) as i32;
            self.put(Value::Enum(v));
            Ok(())
        } else {
            Err(error::Error::BadWireType(actual_wire_type))
        }
    }

    #[inline]
    fn merge_message(&mut self,
                     input: &mut protobuf::CodedInputStream,
                     descriptors: &descriptor::Descriptors,
                     message: &descriptor::MessageDescriptor,
                     actual_wire_type: wire_format::WireType)
                     -> error::Result<()> {
        if wire_format::WireType::WireTypeLengthDelimited == actual_wire_type {
            let len = try!(input.read_raw_varint32());
            let mut msg = match *self {
                Field::Singular(ref mut o) => {
                    if let Some(Value::Message(m)) = o.take() { m } else { Message::new(message) }
                },
                _ => Message::new(message),
            };

            let old_limit = try!(input.push_limit(len));
            try!(msg.merge_from(descriptors, message, input));
            input.pop_limit(old_limit);

            self.put(Value::Message(msg));
            Ok(())
        } else {
            Err(error::Error::BadWireType(actual_wire_type))
        }
    }

    #[inline]
    fn put(&mut self, value: Value) {
        match *self {
            Field::Singular(ref mut s) => *s = Some(value),
            Field::Repeated(ref mut r) => r.push(value),
        }
    }
}
