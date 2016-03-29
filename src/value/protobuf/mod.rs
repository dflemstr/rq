use std::collections;

use protobuf;

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
        match self.descriptors.messages_by_name.get(&self.name) {
            Some(message) => {
                self.context.read_message(&self.descriptors, &message.upgrade().unwrap())
            },
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
            if let Some(field) = message.fields_by_number.get(&number) {
                let field = &field.upgrade().unwrap();

                debug!("Field is known: {:?}", field);

                if field.label == descriptor::Label::Repeated {
                    debug!("Field is repeated");
                    let mut values = repeateds.entry(field.name.clone())
                                              .or_insert_with(|| Vec::new());

                    try!(self.read_repeated_field(descriptors, field, wire_type, &mut values));
                } else {
                    debug!("Field is singular");
                    let value = try!(self.read_single_field(descriptors, field, wire_type));
                    result.insert(field.name.clone(), value);
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
                         field: &descriptor::FieldDescriptor,
                         wire_type: protobuf::stream::wire_format::WireType)
                         -> error::Result<value::Value> {
        use value::protobuf::descriptor::FieldType;
        use protobuf::stream::wire_format::WireType::*;
        use value::Value::*;

        macro_rules! wrap {
            ($wire_type:pat => $wrapper:expr, $read:expr) => {
                match wire_type {
                    $wire_type => {
                        debug!("Reading a {}", stringify!($wrapper));
                        Ok($wrapper(try!($read)))
                    }
                    _ => Err(bad_wire_type(field, wire_type)),
                }
            }
        };

        match field.field_type {
            FieldType::UnresolvedMessage(ref m) => {
                let msg = format!("Tried to use a field with an unresolved message type: {}", m);
                return Err(error::Error::General(msg))
            },
            FieldType::UnresolvedEnum(ref e) => {
                let msg = format!("Tried to use a field with an unresolved enum type: {}", e);
                return Err(error::Error::General(msg))
            },
            FieldType::Double => wrap!(WireTypeFixed64 => F64, self.input.read_double()),
            FieldType::Float => wrap!(WireTypeFixed32 => F32, self.input.read_float()),
            FieldType::Int64 => wrap!(WireTypeVarint => I64, self.input.read_int64()),
            FieldType::UInt64 => wrap!(WireTypeVarint => U64, self.input.read_uint64()),
            FieldType::Int32 => wrap!(WireTypeVarint => I32, self.input.read_int32()),
            FieldType::Fixed64 => wrap!(WireTypeFixed64 => U64, self.input.read_fixed64()),
            FieldType::Fixed32 => wrap!(WireTypeFixed32 => U32, self.input.read_fixed32()),
            FieldType::Bool => wrap!(WireTypeVarint => Bool, self.input.read_bool()),
            FieldType::String => wrap!(WireTypeLengthDelimited => String, self.input.read_string()),
            FieldType::Group => unimplemented!(),
            FieldType::Message(ref message) => {
                match wire_type {
                    WireTypeLengthDelimited => {
                        let message = message.upgrade().unwrap();
                        debug!("Reading a message");

                        let len = try!(self.input.read_raw_varint32());
                        let old_limit = try!(self.input.push_limit(len));
                        let result = try!(self.read_message(descriptors, &message));
                        self.input.pop_limit(old_limit);
                        Ok(result)
                    },
                    _ => Err(bad_wire_type(field, wire_type)),
                }
            },
            FieldType::Bytes => wrap!(WireTypeLengthDelimited => Bytes, self.input.read_bytes()),
            FieldType::UInt32 => wrap!(WireTypeVarint => U32, self.input.read_uint32()),
            FieldType::Enum(ref enu) => {
                match wire_type {
                    WireTypeVarint => {
                        try!(self.input.read_raw_varint32());
                        Ok(String("Unimplemented".to_owned()))
                    },
                    _ => Err(bad_wire_type(field, wire_type)),
                }
            },
            FieldType::SFixed32 => wrap!(WireTypeFixed32 => I32, self.input.read_sfixed32()),
            FieldType::SFixed64 => wrap!(WireTypeFixed64 => I64, self.input.read_sfixed64()),
            FieldType::SInt32 => wrap!(WireTypeVarint => I32, self.input.read_sint32()),
            FieldType::SInt64 => wrap!(WireTypeVarint => I64, self.input.read_sint64()),
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

        match field.field_type {
            FieldType::UnresolvedMessage(ref m) => {
                let msg = format!("Tried to use a field with an unresolved message type: {}", m);
                return Err(error::Error::General(msg))
            },
            FieldType::UnresolvedEnum(ref e) => {
                let msg = format!("Tried to use a field with an unresolved enum type: {}", e);
                return Err(error::Error::General(msg))
            },
            FieldType::Double => packable!(WireTypeFixed64 => 8, F64, self.input.read_double()),
            FieldType::Float => packable!(WireTypeFixed32 => 4, F32, self.input.read_float()),
            FieldType::Int64 => packable!(WireTypeVarint => I64, self.input.read_int64()),
            FieldType::UInt64 => packable!(WireTypeVarint => U64, self.input.read_uint64()),
            FieldType::Int32 => packable!(WireTypeVarint => I32, self.input.read_int32()),
            FieldType::Fixed64 => packable!(WireTypeFixed64 => U64, self.input.read_fixed64()),
            FieldType::Fixed32 => packable!(WireTypeFixed32 => U32, self.input.read_fixed32()),
            FieldType::Bool => packable!(WireTypeVarint => Bool, self.input.read_bool()),
            FieldType::String => scalar!(WireTypeLengthDelimited => String, self.input.read_string()),
            FieldType::Group => unimplemented!(),
            FieldType::Message(ref message) => {
                match wire_type {
                    WireTypeLengthDelimited => {
                        let message = message.upgrade().unwrap();
                        let len = try!(self.input.read_raw_varint32());
                        let old_limit = try!(self.input.push_limit(len));
                        let result = try!(self.read_message(descriptors, &message));
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

fn bad_wire_type(field: &descriptor::FieldDescriptor,
                 wire_type: protobuf::stream::wire_format::WireType)
                 -> error::Error {
    error::Error::General(format!("Unexpected wire type {:?} for field {}", wire_type, field))
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
