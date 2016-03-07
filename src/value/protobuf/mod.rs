use std::collections;
use std::rc;

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
                self.context.read_message(&self.descriptors,
                                          &message.upgrade().unwrap())
            }
            None => {
                let msg = format!("Message type not found: {}", self.name);
                Err(error::Error::General(msg))
            }
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

            // Only handle known fields for now
            if let Some(field) = message.fields_by_number.get(&number) {
                use protobuf::descriptor::FieldDescriptorProto_Label::*;

                let field = &field.upgrade().unwrap();
                if field.proto_label == LABEL_REPEATED {
                    let mut values = repeateds.entry(field.name.clone())
                                              .or_insert_with(|| Vec::new());

                    try!(self.read_repeated_field(descriptors,
                                                  field,
                                                  wire_type,
                                                  &mut values));
                } else {
                    let value = try!(self.read_single_field(descriptors,
                                                            field,
                                                            wire_type));
                    result.insert(field.name.clone(), value);
                }
            } else {
                use protobuf::stream::wire_format::WireType;
                match wire_type {
                    WireType::WireTypeStartGroup => {
                        loop {
                            let (_, wire_type) = try!(self.input
                                                          .read_tag_unpack());
                            if wire_type == WireType::WireTypeEndGroup {
                                break;
                            }
                            try!(self.input.skip_field(wire_type));
                        }
                    }
                    _ => try!(self.input.skip_field(wire_type)),
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
        use protobuf::descriptor::FieldDescriptorProto_Type::*;
        use protobuf::stream::wire_format::WireType::*;
        use protobuf::rt::unexpected_wire_type;
        use value::Value::*;

        macro_rules! wrap {
            ($wire_type:pat => $wrapper:expr, $read:expr) => {
                match wire_type {
                    $wire_type => {
                        Ok($wrapper(try!($read)))
                    }
                    _ => Err(error::Error::from(unexpected_wire_type(wire_type))),
                }
            }
        };

        match field.proto_type {
            TYPE_DOUBLE => {
                wrap!(WireTypeFixed64 => F64, self.input.read_double())
            }
            TYPE_FLOAT => {
                wrap!(WireTypeFixed32 => F32, self.input.read_float())
            }
            TYPE_INT64 => wrap!(WireTypeVarint => I64, self.input.read_int64()),
            TYPE_UINT64 => {
                wrap!(WireTypeVarint => U64, self.input.read_uint64())
            }
            TYPE_INT32 => wrap!(WireTypeVarint => I32, self.input.read_int32()),
            TYPE_FIXED64 => {
                wrap!(WireTypeFixed64 => U64, self.input.read_fixed64())
            }
            TYPE_FIXED32 => {
                wrap!(WireTypeFixed32 => U32, self.input.read_fixed32())
            }
            TYPE_BOOL => wrap!(WireTypeVarint => Bool, self.input.read_bool()),
            TYPE_STRING =>
                wrap!(WireTypeLengthDelimited => String, self.input.read_string()),
            TYPE_GROUP => unimplemented!(),
            TYPE_MESSAGE => {
                match wire_type {
                    WireTypeLengthDelimited => {
                        if let Some(message) =
                               descriptors.messages_by_name
                                          .get(&field.proto_type_name) {
                            let message = message.upgrade().unwrap();
                            let len = try!(self.input.read_raw_varint32());
                            let old_limit = try!(self.input.push_limit(len));
                            let result = try!(self.read_message(descriptors,
                                                                &message));
                            self.input.pop_limit(old_limit);
                            Ok(result)
                        } else {
                            Err(error::Error::General(format!("Missing type in schema: {}", field.proto_type_name)))
                        }
                    }
                    _ => {
                        Err(error::Error::from(unexpected_wire_type(wire_type)))
                    }
                }
            }
            TYPE_BYTES => {
                wrap!(WireTypeLengthDelimited => Bytes, self.input.read_bytes())
            }
            TYPE_UINT32 => {
                wrap!(WireTypeVarint => U32, self.input.read_uint32())
            }
            TYPE_ENUM => {
                match wire_type {
                    WireTypeVarint => unimplemented!(),
                    _ => {
                        Err(error::Error::from(unexpected_wire_type(wire_type)))
                    }
                }
            }
            TYPE_SFIXED32 => {
                wrap!(WireTypeFixed32 => I32, self.input.read_sfixed32())
            }
            TYPE_SFIXED64 => {
                wrap!(WireTypeFixed64 => I64, self.input.read_sfixed64())
            }
            TYPE_SINT32 => {
                wrap!(WireTypeVarint => I32, self.input.read_sint32())
            }
            TYPE_SINT64 => {
                wrap!(WireTypeVarint => I64, self.input.read_sint64())
            }
        }
    }

    fn read_repeated_field(&mut self,
                           descriptors: &descriptor::Descriptors,
                           field: &descriptor::FieldDescriptor,
                           wire_type: protobuf::stream::wire_format::WireType,
                           values: &mut Vec<value::Value>)
                           -> error::Result<()> {
        use protobuf::descriptor::FieldDescriptorProto_Type::*;
        use protobuf::stream::wire_format::WireType::*;
        use protobuf::rt::unexpected_wire_type;
        use value::Value::*;

        let mut i = &mut self.input;

        macro_rules! packable {
            ($wire_type:pat => $wrapper:expr, $read:expr) => {
                packable!($wire_type => 1, $wrapper, $read)
            };
            ($wire_type:pat => $size:expr, $wrapper:expr, $read:expr) => {
                match wire_type {
                    WireTypeLengthDelimited => {
                        let len = try!(i.read_raw_varint32());
                        values.reserve((len / $size) as usize);

                        let old_limit = try!(i.push_limit(len));
                        while !try!(i.eof()) {
                            values.push($wrapper(try!($read)));
                        }
                        i.pop_limit(old_limit);
                    }
                    $wire_type => {
                        values.push($wrapper(try!($read)));
                    }
                    _ => return Err(error::Error::from(unexpected_wire_type(wire_type))),
                }
            }
        };

        macro_rules! scalar {
            ($wire_type:pat => $wrapper:expr, $read:expr) => {
                match wire_type {
                    $wire_type => {
                        values.push($wrapper(try!($read)));
                    }
                    _ => return Err(error::Error::from(unexpected_wire_type(wire_type))),
                }
            }
        };

        match field.proto_type {
            TYPE_DOUBLE => {
                packable!(WireTypeFixed64 => 8, F64, i.read_double())
            }
            TYPE_FLOAT => packable!(WireTypeFixed32 => 4, F32, i.read_float()),
            TYPE_INT64 => packable!(WireTypeVarint => I64, i.read_int64()),
            TYPE_UINT64 => packable!(WireTypeVarint => U64, i.read_uint64()),
            TYPE_INT32 => packable!(WireTypeVarint => I32, i.read_int32()),
            TYPE_FIXED64 => packable!(WireTypeFixed64 => U64, i.read_fixed64()),
            TYPE_FIXED32 => packable!(WireTypeFixed32 => U32, i.read_fixed32()),
            TYPE_BOOL => packable!(WireTypeVarint => Bool, i.read_bool()),
            TYPE_STRING => {
                scalar!(WireTypeLengthDelimited => String, i.read_string())
            }
            TYPE_GROUP => unimplemented!(),
            TYPE_MESSAGE => {
                match wire_type {
                    WireTypeLengthDelimited => unimplemented!(),
                    _ => unimplemented!(),
                }
            }
            TYPE_BYTES => {
                match wire_type {
                    WireTypeLengthDelimited => unimplemented!(),
                    _ => unimplemented!(),
                }
            }
            TYPE_UINT32 => packable!(WireTypeVarint => U32, i.read_uint32()),
            TYPE_ENUM => {
                match wire_type {
                    WireTypeVarint => unimplemented!(),
                    _ => unimplemented!(),
                }
            }
            TYPE_SFIXED32 => {
                packable!(WireTypeFixed32 => 4, I32, i.read_sfixed32())
            }
            TYPE_SFIXED64 => {
                packable!(WireTypeFixed64 => 8, I64, i.read_sfixed64())
            }
            TYPE_SINT32 => packable!(WireTypeVarint => 4, I32, i.read_sint32()),
            TYPE_SINT64 => packable!(WireTypeVarint => 8, I64, i.read_sint64()),
        }

        Ok(())
    }
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
