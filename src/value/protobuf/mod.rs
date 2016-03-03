use std::collections;
use std::rc;

use protobuf;

use error;
use value;

mod descriptor;

pub struct ProtobufValues<'a> {
    descriptors: descriptor::Descriptors,
    type_name: String,
    context: Context<'a>,
}

struct Context<'a> {
    input: protobuf::CodedInputStream<'a>,
}

impl<'a> ProtobufValues<'a> {
    pub fn new(descriptor: protobuf::descriptor::FileDescriptorSet,
               input: protobuf::CodedInputStream<'a>)
               -> ProtobufValues<'a> {
        unimplemented!()
    }

    fn try_next(&mut self) -> error::Result<value::Value> {
        unimplemented!()
    }
}

// impl<'a> Context<'a> {
// fn read_message(&mut self,
// descriptor: &protobuf::descriptor::DescriptorProto)
// -> error::Result<value::Value> {
// let mut result = collections::BTreeMap::new();
// let fields = descriptor.get_field();
//
// while !try!(self.input.eof()) {
// let (field_number, wire_type) = try!(self.input.read_tag_unpack());
//
// Only handle known fields for now
// if let Some(i) = self.number_index.get(&field_number).map(|i| *i) {
// let ref field = fields[i];
// let value = try!(self.read_field(field, wire_type));
// result.insert(field.get_name().to_owned(), value);
// }
// }
//
// Ok(value::Value::Map(result))
// }
//
// fn read_field(&mut self,
// field: &protobuf::descriptor::FieldDescriptorProto,
// wire_type: protobuf::stream::wire_format::WireType)
// -> error::Result<value::Value> {
// use protobuf::descriptor::FieldDescriptorProto_Type::*;
// use protobuf::stream::wire_format::WireType::*;
//
// let bad_wire_type = || {
// Err(error::Error::from(
// protobuf::rt::unexpected_wire_type(wire_type)))
// };
//
// match field.get_field_type() {
// TYPE_DOUBLE => if wire_type != WireTypeFixed64 {
// bad_wire_type()
// } else {
// Ok(value::Value::F64(try!(self.input.read_double())))
// },
// TYPE_FLOAT => if wire_type != WireTypeFixed32 {
// bad_wire_type()
// } else {
// Ok(value::Value::F32(try!(self.input.read_float())))
// },
// TYPE_INT64 => if wire_type != WireTypeVarint {
// bad_wire_type()
// } else {
// Ok(value::Value::I64(try!(self.input.read_int64())))
// },
// TYPE_UINT64 => if wire_type != WireTypeVarint {
// bad_wire_type()
// } else {
// Ok(value::Value::U64(try!(self.input.read_uint64())))
// },
// TYPE_INT32 => if wire_type != WireTypeVarint {
// bad_wire_type()
// } else {
// Ok(value::Value::I32(try!(self.input.read_int32())))
// },
// TYPE_FIXED64 => if wire_type != WireTypeFixed64 {
// bad_wire_type()
// } else {
// Ok(value::Value::U64(try!(self.input.read_fixed64())))
// },
// TYPE_FIXED32 => if wire_type != WireTypeFixed32 {
// bad_wire_type()
// } else {
// Ok(value::Value::U32(try!(self.input.read_fixed32())))
// },
// TYPE_BOOL => if wire_type != WireTypeVarint {
// bad_wire_type()
// } else {
// Ok(value::Value::Bool(try!(self.input.read_bool())))
// },
// TYPE_STRING => if wire_type != WireTypeLengthDelimited {
// bad_wire_type()
// } else {
// Ok(value::Value::String(try!(self.input.read_string())))
// },
// TYPE_GROUP => unimplemented!(),
// TYPE_MESSAGE => if wire_type != WireTypeLengthDelimited {
// bad_wire_type()
// } else {
// unimplemented!()
// },
// TYPE_BYTES => if wire_type != WireTypeLengthDelimited {
// bad_wire_type()
// } else {
// unimplemented!()
// },
// TYPE_UINT32 => if wire_type != WireTypeVarint {
// bad_wire_type()
// } else {
// Ok(value::Value::U32(try!(self.input.read_uint32())))
// },
// TYPE_ENUM => if wire_type != WireTypeVarint {
// bad_wire_type()
// } else {
// unimplemented!()
// },
// TYPE_SFIXED32 => if wire_type != WireTypeFixed32 {
// bad_wire_type()
// } else {
// Ok(value::Value::I32(try!(self.input.read_sfixed32())))
// },
// TYPE_SFIXED64 => if wire_type != WireTypeFixed64 {
// bad_wire_type()
// } else {
// Ok(value::Value::I64(try!(self.input.read_sfixed64())))
// },
// TYPE_SINT32 => if wire_type != WireTypeVarint {
// bad_wire_type()
// } else {
// Ok(value::Value::I32(try!(self.input.read_sint32())))
// },
// TYPE_SINT64 => if wire_type != WireTypeVarint {
// bad_wire_type()
// } else {
// Ok(value::Value::I64(try!(self.input.read_sint64())))
// },
// }
// }
// }
//

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
