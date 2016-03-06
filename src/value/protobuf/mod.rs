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

            // Only handle known fields for now
            if let Some(field) = message.fields_by_number
                                        .get(&(field_number as i32)) {
                use protobuf::descriptor::FieldDescriptorProto_Label;
                let field = &field.upgrade().unwrap();
                if field.proto_label ==
                   FieldDescriptorProto_Label::LABEL_REPEATED {
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

        let bad_wire_type = || {
            Err(error::Error::from(unexpected_wire_type(wire_type)))
        };

        match field.proto_type {
            TYPE_DOUBLE => {
                match wire_type {
                    WireTypeFixed64 => {
                        Ok(value::Value::F64(try!(self.input.read_double())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_FLOAT => {
                match wire_type {
                    WireTypeFixed32 => {
                        Ok(value::Value::F32(try!(self.input.read_float())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_INT64 => {
                match wire_type {
                    WireTypeVarint => {
                        Ok(value::Value::I64(try!(self.input.read_int64())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_UINT64 => {
                match wire_type {
                    WireTypeVarint => {
                        Ok(value::Value::U64(try!(self.input.read_uint64())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_INT32 => {
                match wire_type {
                    WireTypeVarint => {
                        Ok(value::Value::I32(try!(self.input.read_int32())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_FIXED64 => {
                match wire_type {
                    WireTypeFixed64 => {
                        Ok(value::Value::U64(try!(self.input.read_fixed64())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_FIXED32 => {
                match wire_type {
                    WireTypeFixed32 => {
                        Ok(value::Value::U32(try!(self.input.read_fixed32())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_BOOL => {
                match wire_type {
                    WireTypeVarint => {
                        Ok(value::Value::Bool(try!(self.input.read_bool())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_STRING => {
                match wire_type {
                    WireTypeLengthDelimited => {
                        Ok(value::Value::String(try!(self.input.read_string())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_GROUP => unimplemented!(),
            TYPE_MESSAGE => {
                match wire_type {
                    WireTypeLengthDelimited => unimplemented!(),
                    _ => bad_wire_type(),
                }
            }
            TYPE_BYTES => {
                match wire_type {
                    WireTypeLengthDelimited => unimplemented!(),
                    _ => bad_wire_type(),
                }
            }
            TYPE_UINT32 => {
                match wire_type {
                    WireTypeVarint => {
                        Ok(value::Value::U32(try!(self.input.read_uint32())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_ENUM => {
                match wire_type {
                    WireTypeVarint => unimplemented!(),
                    _ => bad_wire_type(),
                }
            }
            TYPE_SFIXED32 => {
                match wire_type {
                    WireTypeFixed32 => {
                        Ok(value::Value::I32(try!(self.input.read_sfixed32())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_SFIXED64 => {
                match wire_type {
                    WireTypeFixed64 => {
                        Ok(value::Value::I64(try!(self.input.read_sfixed64())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_SINT32 => {
                match wire_type {
                    WireTypeVarint => {
                        Ok(value::Value::I32(try!(self.input.read_sint32())))
                    }
                    _ => bad_wire_type(),
                }
            }
            TYPE_SINT64 => {
                match wire_type {
                    WireTypeVarint => {
                        Ok(value::Value::I64(try!(self.input.read_sint64())))
                    }
                    _ => bad_wire_type(),
                }
            }
        }
    }

    fn read_repeated_field(&mut self,
                           descriptors: &descriptor::Descriptors,
                           field: &descriptor::FieldDescriptor,
                           wire_type: protobuf::stream::wire_format::WireType,
                           values: &mut Vec<value::Value>)
                           -> error::Result<()> {
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
