use std::collections;

use protobuf;
use protobuf::stream::wire_format;

use error;
use value;

pub mod descriptor;

pub struct ProtobufValues<'a> {
    descriptors: descriptor::Descriptors,
    name: String,
    context: Context<'a>,
}

struct Context<'a> {
    input: protobuf::CodedInputStream<'a>,
}

impl<'a> ProtobufValues<'a> {
    pub fn new(descriptors: descriptor::Descriptors,
               name: String,
               input: protobuf::CodedInputStream<'a>)
               -> ProtobufValues<'a> {
        ProtobufValues {
            descriptors: descriptors,
            name: name,
            context: Context { input: input },
        }
    }

    fn try_next(&mut self) -> error::Result<value::Value> {
        match self.descriptors.message_by_name(&self.name) {
            Some(message) => self.context.read_message(&self.descriptors, message),
            None => {
                let msg = format!("Message type not found: {}", self.name);
                Err(error::Error::General(msg))
            },
        }
    }
}

impl<'a> Context<'a> {
    fn read_message(&mut self,
                    descriptors: &descriptor::Descriptors,
                    message: &descriptor::MessageDescriptor)
                    -> error::Result<value::Value> {
        let mut result = collections::BTreeMap::new();
        let mut repeateds = collections::HashMap::new();

        while !try!(self.input.eof()) {
            let (field_number, wire_type) = try!(self.input.read_tag_unpack());
            let number = field_number as i32;

            debug!("Encountered field with number {} type {:?}",
                   number,
                   wire_type);

            // Only handle known fields for now
            if let Some(field) = message.field_by_number(number) {
                debug!("Field is known: {:?}", field);

                if field.field_label() == descriptor::FieldLabel::Repeated {
                    debug!("Field is repeated");
                    let mut values = repeateds.entry(field.name().to_owned())
                                              .or_insert_with(Vec::new);

                    try!(self.read_repeated_field(descriptors, &field, wire_type, &mut values));
                    debug!("Values so far {:?}", values);
                } else {
                    debug!("Field is singular");
                    let value = try!(self.read_single_field(descriptors, &field, wire_type));
                    debug!("Value is {:?}", value);
                    result.insert(field.name().to_owned(), value);
                }
            } else {
                use protobuf::stream::wire_format::WireType;
                match wire_type {
                    WireType::WireTypeStartGroup => {
                        debug!("Skipping unknown group");
                        loop {
                            let (_, wire_type) = try!(self.input
                                                          .read_tag_unpack());
                            if wire_type == WireType::WireTypeEndGroup {
                                break;
                            }
                            try!(self.input.skip_field(wire_type));
                        }
                    },
                    _ => {
                        debug!("Skipping unknown field");
                        try!(self.input.skip_field(wire_type));
                    },
                }
            }
        }

        for (key, values) in repeateds.into_iter() {
            result.insert(key, value::Value::Sequence(values));
        }

        Ok(value::Value::Map(result))
    }

    fn read_single_field(&mut self,
                         descriptors: &descriptor::Descriptors,
                         f: &descriptor::FieldDescriptor,
                         wt: protobuf::stream::wire_format::WireType)
                         -> error::Result<value::Value> {
        use value::protobuf::descriptor::FieldType as T;
        use protobuf::CodedInputStream as I;
        use protobuf::stream::wire_format::WireType as W;
        use value::Value as V;

        match f.field_type(descriptors) {
            T::UnresolvedMessage(ref m) => Err(unresolved_message(m)),
            T::UnresolvedEnum(ref e) => Err(unresolved_enum(e)),
            T::Double => self.ss(f, W::WireTypeFixed64, wt, V::F64, I::read_double),
            T::Float => self.ss(f, W::WireTypeFixed32, wt, V::F32, I::read_float),
            T::Int64 => self.ss(f, W::WireTypeVarint, wt, V::I64, I::read_int64),
            T::UInt64 => self.ss(f, W::WireTypeVarint, wt, V::U64, I::read_uint64),
            T::Int32 => self.ss(f, W::WireTypeVarint, wt, V::I32, I::read_int32),
            T::Fixed64 => self.ss(f, W::WireTypeFixed64, wt, V::U64, I::read_fixed64),
            T::Fixed32 => self.ss(f, W::WireTypeFixed32, wt, V::U32, I::read_fixed32),
            T::Bool => self.ss(f, W::WireTypeVarint, wt, V::Bool, I::read_bool),
            T::String => self.ss(f, W::WireTypeLengthDelimited, wt, V::String, I::read_string),
            T::Group => unimplemented!(),
            T::Message(ref m) => self.sm(descriptors, f, m, wt),
            T::Bytes => self.ss(f, W::WireTypeLengthDelimited, wt, V::Bytes, I::read_bytes),
            T::UInt32 => self.ss(f, W::WireTypeVarint, wt, V::U32, I::read_uint32),
            T::Enum(ref enu) => {
                match wt {
                    W::WireTypeVarint => {
                        try!(self.input.read_raw_varint32());
                        Ok(V::String("Unimplemented".to_owned()))
                    },
                    _ => Err(bad_wire_type(f, wt)),
                }
            },
            T::SFixed32 => self.ss(f, W::WireTypeFixed32, wt, V::I32, I::read_sfixed32),
            T::SFixed64 => self.ss(f, W::WireTypeFixed64, wt, V::I64, I::read_sfixed64),
            T::SInt32 => self.ss(f, W::WireTypeVarint, wt, V::I32, I::read_sint32),
            T::SInt64 => self.ss(f, W::WireTypeVarint, wt, V::I64, I::read_sint64),
        }
    }

    fn ss<A, W, R>(&mut self,
                   field: &descriptor::FieldDescriptor,
                   expected_wire_type: wire_format::WireType,
                   actual_wire_type: wire_format::WireType,
                   wrapper: W,
                   reader: R)
                   -> error::Result<value::Value>
        where W: Fn(A) -> value::Value,
              R: Fn(&mut protobuf::CodedInputStream<'a>) -> protobuf::ProtobufResult<A>
    {
        if expected_wire_type == actual_wire_type {
            Ok(wrapper(try!(reader(&mut self.input))))
        } else {
            Err(bad_wire_type(field, actual_wire_type))
        }
    }

    fn sm(&mut self,
          descriptors: &descriptor::Descriptors,
          field: &descriptor::FieldDescriptor,
          message: &descriptor::MessageDescriptor,
          actual_wire_type: wire_format::WireType)
          -> error::Result<value::Value> {
        if wire_format::WireType::WireTypeLengthDelimited == actual_wire_type {
            debug!("Reading a message");

            let len = try!(self.input.read_raw_varint32());
            let old_limit = try!(self.input.push_limit(len));
            let result = try!(self.read_message(descriptors, message));
            self.input.pop_limit(old_limit);
            Ok(result)
        } else {
            Err(bad_wire_type(field, actual_wire_type))
        }
    }

    fn read_repeated_field(&mut self,
                           descriptors: &descriptor::Descriptors,
                           field: &descriptor::FieldDescriptor,
                           wire_type: protobuf::stream::wire_format::WireType,
                           values: &mut Vec<value::Value>)
                           -> error::Result<()> {
        use value::protobuf::descriptor::FieldType;
        use protobuf::stream::wire_format::WireType::*;
        use value::Value::*;

        macro_rules! packable {
            ($wire_type:pat => $wrapper:expr, $read:expr) => {
                packable!($wire_type => 1, $wrapper, $read)
            };
            ($wire_type:pat => $size:expr, $wrapper:expr, $read:expr) => {
                match wire_type {
                    WireTypeLengthDelimited => {
                        let len = try!(self.input.read_raw_varint32());
                        values.reserve((len / $size) as usize);

                        let old_limit = try!(self.input.push_limit(len));
                        while !try!(self.input.eof()) {
                            values.push($wrapper(try!($read)));
                        }
                        self.input.pop_limit(old_limit);
                    }
                    $wire_type => {
                        values.push($wrapper(try!($read)));
                    }
                    _ => return Err(bad_wire_type(field, wire_type)),
                }
            }
        };

        macro_rules! scalar {
            ($wire_type:pat => $wrapper:expr, $read:expr) => {
                match wire_type {
                    $wire_type => {
                        values.push($wrapper(try!($read)));
                    }
                    _ => return Err(bad_wire_type(field, wire_type)),
                }
            }
        };

        match field.field_type(descriptors) {
            FieldType::UnresolvedMessage(ref m) => {
                return Err(unresolved_message(m));
            },
            FieldType::UnresolvedEnum(ref e) => {
                return Err(unresolved_enum(e));
            },
            FieldType::Double => packable!(WireTypeFixed64 => 8, F64, self.input.read_double()),
            FieldType::Float => packable!(WireTypeFixed32 => 4, F32, self.input.read_float()),
            FieldType::Int64 => packable!(WireTypeVarint => I64, self.input.read_int64()),
            FieldType::UInt64 => packable!(WireTypeVarint => U64, self.input.read_uint64()),
            FieldType::Int32 => packable!(WireTypeVarint => I32, self.input.read_int32()),
            FieldType::Fixed64 => packable!(WireTypeFixed64 => U64, self.input.read_fixed64()),
            FieldType::Fixed32 => packable!(WireTypeFixed32 => U32, self.input.read_fixed32()),
            FieldType::Bool => packable!(WireTypeVarint => Bool, self.input.read_bool()),
            FieldType::String => {
                scalar!(WireTypeLengthDelimited => String, self.input.read_string())
            },
            FieldType::Group => unimplemented!(),
            FieldType::Message(ref message) => {
                match wire_type {
                    WireTypeLengthDelimited => {
                        let len = try!(self.input.read_raw_varint32());
                        let old_limit = try!(self.input.push_limit(len));
                        let result = try!(self.read_message(descriptors, message));
                        self.input.pop_limit(old_limit);
                        values.push(result);
                    },
                    _ => return Err(bad_wire_type(field, wire_type)),
                }
            },
            FieldType::Bytes => scalar!(WireTypeLengthDelimited => Bytes, self.input.read_bytes()),
            FieldType::UInt32 => packable!(WireTypeVarint => U32, self.input.read_uint32()),
            FieldType::Enum(ref enu) => {
                match wire_type {
                    WireTypeVarint => {
                        try!(self.input.read_raw_varint32());
                        values.push(String("Unimplemented".to_owned()));
                    },
                    _ => return Err(bad_wire_type(field, wire_type)),
                }
            },
            FieldType::SFixed32 => packable!(WireTypeFixed32 => 4, I32, self.input.read_sfixed32()),
            FieldType::SFixed64 => packable!(WireTypeFixed64 => 8, I64, self.input.read_sfixed64()),
            FieldType::SInt32 => packable!(WireTypeVarint => 4, I32, self.input.read_sint32()),
            FieldType::SInt64 => packable!(WireTypeVarint => 8, I64, self.input.read_sint64()),
        }

        Ok(())
    }
}

fn unresolved_message(type_name: &str) -> error::Error {
    let msg = format!("Tried to use a field with an unresolved message type: {}",
                      type_name);
    error::Error::General(msg)
}

fn unresolved_enum(type_name: &str) -> error::Error {
    let msg = format!("Tried to use a field with an unresolved enum type: {}",
                      type_name);
    error::Error::General(msg)
}

fn bad_wire_type(field: &descriptor::FieldDescriptor,
                 wire_type: protobuf::stream::wire_format::WireType)
                 -> error::Error {
    let msg = format!("Unexpected wire type {:?} for field {}",
                      wire_type,
                      field.name());
    error::Error::General(msg)
}

impl<'a> Iterator for ProtobufValues<'a> {
    type Item = error::Result<value::Value>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.context.input.eof() {
            Ok(false) => Some(self.try_next()),
            Ok(true) => None,
            Err(e) => Some(Err(error::Error::from(e))),
        }
    }
}
