//! Dynamic descriptors for protocol buffer schemata.
//!
//! The descriptors are optimized for read performance, i.e. to be used by a parser to parse actual
//! protocol buffer data.
//!
//! They can be constructed either from pre-compiled protocol buffer-serialized descriptors as
//! defined in [`descriptor.proto`][1], or manually by incrementally building a custom protocol
//! buffer schema.
//!
//! ## Pre-compiled schemas
//!
//! Given a protocol buffer schema `schema.proto`, it can be compiled into a binary file descriptor
//! set using the `protoc` tool:
//!
//! ```text
//! protoc schema.proto -o testdata/descriptors.pb
//! ```
//!
//! The binary file descriptor set can then be parsed into a `Descriptors` instance:
//!
//! ```
//! extern crate serde_protobuf;
//! extern crate protobuf;
//!
//! use std::fs;
//! use serde_protobuf::descriptor::Descriptors;
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
//! # fn foo() -> Result<(), Error> {
//! let mut file = try!(fs::File::open("testdata/descriptors.pb"));
//! let proto = try!(protobuf::parse_from_reader(&mut file));
//! let descriptors = Descriptors::from_proto(&proto);
//! # Ok(())
//! # }
//! # fn main() {
//! #   foo().unwrap();
//! # }
//! ```
//!
//! ## Manually built schemas
//!
//! A descriptor can be built at run-time by incrementally adding new message types and fields:
//!
//! ```
//! use serde_protobuf::descriptor::*;
//!
//! // Create a new message type
//! let mut m = MessageDescriptor::new(".mypackage.Person");
//! m.add_field(FieldDescriptor::new("name", 1, FieldLabel::Optional,
//!                                  InternalFieldType::String, None));
//! m.add_field(FieldDescriptor::new("age", 2, FieldLabel::Optional,
//!                                  InternalFieldType::Int32, None));
//!
//! // Create a new enum type
//! let mut e = EnumDescriptor::new(".mypackage.Color");
//! e.add_value(EnumValueDescriptor::new("BLUE", 1));
//! e.add_value(EnumValueDescriptor::new("RED", 2));
//!
//! // Add the generated types to a descriptor registry
//! let mut descriptors = Descriptors::new();
//! descriptors.add_message(m);
//! descriptors.add_enum(e);
//! ```
//!
//! ## Exploring descriptors
//!
//! The descriptors contain various indices that can be used to quickly look up information:
//!
//! ```
//! # extern crate serde_protobuf;
//! # extern crate protobuf;
//! # use std::fs;
//! # use serde_protobuf::descriptor::Descriptors;
//! # fn main() {
//! # let mut file = fs::File::open("testdata/descriptors.pb").unwrap();
//! # let proto = protobuf::parse_from_reader(&mut file).unwrap();
//! // Given a set of descriptors using one of the above methods:
//! let descriptors = Descriptors::from_proto(&proto);
//! assert_eq!(7, descriptors.message_by_name(".protobuf_unittest.TestAllTypes").unwrap()
//!                          .field_by_name("optional_fixed32").unwrap()
//!                          .number());
//! # }
//! ```
//!
//! ## Optimizing reference look-ups
//!
//! Certain descriptor look-ups require following references that can be quite expensive to look up.
//! Instead, a one-time cost can be payed to resolve these references and make all following
//! look-ups cheaper.  This should be done after all needed descriptors have been loaded:
//!
//! ```
//! # extern crate serde_protobuf;
//! # extern crate protobuf;
//! # use std::fs;
//! # use serde_protobuf::descriptor::*;
//! # fn main() {
//! # let mut file = fs::File::open("testdata/descriptors.pb").unwrap();
//! # let proto = protobuf::parse_from_reader(&mut file).unwrap();
//! // Load some descriptors as usual:
//! let mut descriptors = Descriptors::from_proto(&proto);
//!
//! // Resolve references internally to speed up lookups:
//! descriptors.resolve_refs();
//!
//! // This should now be faster
//! match descriptors.message_by_name(".protobuf_unittest.TestAllTypes").unwrap()
//!                  .field_by_name("optional_nested_message").unwrap()
//!                  .field_type(&descriptors) {
//!   FieldType::Message(m) =>
//!     assert_eq!(1, m.field_by_name("bb").unwrap()
//!                    .number()),
//!   _ => unreachable!(),
//! }
//! # }
//! ```
//!
//! [1]: https://github.com/google/protobuf/blob/master/src/google/protobuf/descriptor.proto
use std::f32;
use std::f64;

use linked_hash_map;
use protobuf::descriptor;

use error;
use value;

/// An ID used for internal tracking of resolved message descriptors.
///
/// It is not possible to construct a value of this type from outside this module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageId(usize);

/// An ID used for internal tracking of resolved enum descriptors.
///
/// It is not possible to construct a value of this type from outside this module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnumId(usize);

/// An ID used for internal tracking of resolved enum values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EnumValueId(usize);

/// An ID used for internal tracking of resolved fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FieldId(usize);

/// A registry for any number of protocol buffer descriptors.
#[derive(Debug)]
pub struct Descriptors {
    // All found descriptors
    messages: Vec<MessageDescriptor>,
    enums: Vec<EnumDescriptor>,

    // Indices
    messages_by_name: linked_hash_map::LinkedHashMap<String, MessageId>,
    enums_by_name: linked_hash_map::LinkedHashMap<String, EnumId>,
}

/// A descriptor for a single protocol buffer message type.
// TODO: Support oneof?
#[derive(Debug)]
pub struct MessageDescriptor {
    name: String,

    // All found descriptors
    fields: Vec<FieldDescriptor>,

    // Indices
    fields_by_name: linked_hash_map::LinkedHashMap<String, FieldId>,
    fields_by_number: linked_hash_map::LinkedHashMap<i32, FieldId>,
}

/// A descriptor for a single protocol buffer enum type.
#[derive(Debug)]
pub struct EnumDescriptor {
    name: String,

    // All found descriptors
    values: Vec<EnumValueDescriptor>,

    // Indices
    values_by_name: linked_hash_map::LinkedHashMap<String, EnumValueId>,
    values_by_number: linked_hash_map::LinkedHashMap<i32, EnumValueId>,
}

/// A descriptor for a single protocol buffer enum value.
#[derive(Debug)]
pub struct EnumValueDescriptor {
    name: String,
    number: i32,
}

/// A label that a field can be given to indicate its cardinality.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldLabel {
    /// There can be zero or one value.
    Optional,
    /// There must be exactly one value.
    Required,
    /// There can be any number of values.
    Repeated,
}

/// The externally visible type of a field.
///
/// This type representation borrows references to any referenced descriptors.
#[derive(Debug)]
pub enum FieldType<'a> {
    UnresolvedMessage(&'a str),
    UnresolvedEnum(&'a str),
    Double,
    Float,
    Int64,
    UInt64,
    Int32,
    Fixed64,
    Fixed32,
    Bool,
    String,
    Group,
    Message(&'a MessageDescriptor),
    Bytes,
    UInt32,
    Enum(&'a EnumDescriptor),
    SFixed32,
    SFixed64,
    SInt32,
    SInt64,
}

/// The internally tracked type of a field.
///
/// The type owns all of its data, and can refer to an internally tracked ID for resolved type
/// references.  It's by design not possible to construct those IDs from outside this module.
#[derive(Debug, Eq, PartialEq)]
pub enum InternalFieldType {
    UnresolvedMessage(String),
    UnresolvedEnum(String),
    Double,
    Float,
    Int64,
    UInt64,
    Int32,
    Fixed64,
    Fixed32,
    Bool,
    String,
    Group,
    Message(MessageId),
    Bytes,
    UInt32,
    Enum(EnumId),
    SFixed32,
    SFixed64,
    SInt32,
    SInt64,
}

/// A descriptor for a single protocol buffer message field.
#[derive(Debug)]
pub struct FieldDescriptor {
    name: String,
    number: i32,
    field_label: FieldLabel,
    field_type: InternalFieldType,
    default_value: Option<value::Value>,
}

impl Descriptors {
    /// Creates a new empty descriptor set.
    pub fn new() -> Descriptors {
        Descriptors {
            messages: Vec::new(),
            enums: Vec::new(),

            messages_by_name: linked_hash_map::LinkedHashMap::new(),
            enums_by_name: linked_hash_map::LinkedHashMap::new(),
        }
    }

    /// Builds a descriptor set from the specified protocol buffer file descriptor set.
    pub fn from_proto(file_set_proto: &descriptor::FileDescriptorSet) -> Descriptors {
        let mut descriptors = Descriptors::new();
        descriptors.add_file_set_proto(file_set_proto);
        descriptors
    }

    /// Looks up a message by its fully qualified name (i.e. `.foo.package.Message`).
    #[inline]
    pub fn message_by_name(&self, name: &str) -> Option<&MessageDescriptor> {
        self.messages_by_name.get(name).map(|m| &self.messages[m.0])
    }

    /// Looks up an enum by its fully qualified name (i.e. `.foo.package.Enum`).
    #[inline]
    pub fn enum_by_name(&self, name: &str) -> Option<&EnumDescriptor> {
        self.enums_by_name.get(name).map(|e| &self.enums[e.0])
    }

    /// Adds all types defined in the specified protocol buffer file descriptor set to this
    /// registry.
    pub fn add_file_set_proto(&mut self, file_set_proto: &descriptor::FileDescriptorSet) {
        for file_proto in file_set_proto.get_file().iter() {
            self.add_file_proto(file_proto);
        }
    }

    /// Adds all types defined in the specified protocol buffer file descriptor to this registry.
    pub fn add_file_proto(&mut self, file_proto: &descriptor::FileDescriptorProto) {
        let path = if file_proto.has_package() {
            format!(".{}", file_proto.get_package())
        } else {
            "".to_owned()
        };

        for message_proto in file_proto.get_message_type().iter() {
            self.add_message_proto(&path, message_proto);
        }

        for enum_proto in file_proto.get_enum_type().iter() {
            self.add_enum(EnumDescriptor::from_proto(&path, enum_proto));
        }
    }

    /// Adds a message and all nested types within that message from the specified protocol buffer
    /// descriptor.
    pub fn add_message_proto(&mut self, path: &str, message_proto: &descriptor::DescriptorProto) {
        let message_descriptor = MessageDescriptor::from_proto(path, message_proto);

        for nested_message_proto in message_proto.get_nested_type().iter() {
            self.add_message_proto(message_descriptor.name(), nested_message_proto);
        }

        for nested_enum_proto in message_proto.get_enum_type().iter() {
            self.add_enum(EnumDescriptor::from_proto(message_descriptor.name(), nested_enum_proto));
        }

        self.add_message(message_descriptor);
    }

    /// Adds a single custom built message descriptor.
    pub fn add_message(&mut self, descriptor: MessageDescriptor) {
        let name = descriptor.name.clone();
        let message_id = MessageId(store(&mut self.messages, descriptor));
        self.messages_by_name.insert(name, message_id);
    }

    /// Adds a single custom built enum descriptor.
    pub fn add_enum(&mut self, descriptor: EnumDescriptor) {
        let name = descriptor.name.clone();
        let enum_id = EnumId(store(&mut self.enums, descriptor));
        self.enums_by_name.insert(name, enum_id);
    }

    /// Resolves all internal descriptor type references, making them cheaper to follow.
    pub fn resolve_refs(&mut self) {
        for ref mut m in &mut self.messages {
            for f in &mut m.fields {
                let field_type = &mut f.field_type;
                let new = match *field_type {
                    InternalFieldType::UnresolvedMessage(ref name) => {
                        if let Some(res) = self.messages_by_name.get(name) {
                            Some(InternalFieldType::Message(*res))
                        } else {
                            warn!("Inconsistent schema; unknown message type {}", name);
                            None
                        }
                    },
                    InternalFieldType::UnresolvedEnum(ref name) => {
                        if let Some(res) = self.enums_by_name.get(name) {
                            Some(InternalFieldType::Enum(*res))
                        } else {
                            warn!("Inconsistent schema; unknown enum type {}", name);
                            None
                        }
                    },
                    _ => None,
                };

                if let Some(t) = new {
                    *field_type = t;
                }
            }
        }
    }
}

impl MessageDescriptor {
    pub fn new<S>(name: S) -> MessageDescriptor
        where S: Into<String>
    {
        MessageDescriptor {
            name: name.into(),
            fields: Vec::new(),
            fields_by_name: linked_hash_map::LinkedHashMap::new(),
            fields_by_number: linked_hash_map::LinkedHashMap::new(),
        }
    }

    pub fn from_proto(path: &str, proto: &descriptor::DescriptorProto) -> MessageDescriptor {
        let name = format!("{}.{}", path, proto.get_name());
        let mut message_descriptor = MessageDescriptor::new(name);

        for field_proto in proto.get_field().iter() {
            message_descriptor.add_field(FieldDescriptor::from_proto(field_proto));
        }

        message_descriptor
    }

    pub fn fields(&self) -> &[FieldDescriptor] {
        &self.fields
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn field_by_name(&self, name: &str) -> Option<&FieldDescriptor> {
        self.fields_by_name.get(name).map(|f| &self.fields[f.0])
    }

    #[inline]
    pub fn field_by_number(&self, number: i32) -> Option<&FieldDescriptor> {
        self.fields_by_number.get(&number).map(|f| &self.fields[f.0])
    }

    pub fn add_field(&mut self, descriptor: FieldDescriptor) {
        let name = descriptor.name.clone();
        let number = descriptor.number;

        let field_id = FieldId(store(&mut self.fields, descriptor));

        self.fields_by_name.insert(name, field_id);
        self.fields_by_number.insert(number, field_id);
    }
}

impl EnumDescriptor {
    pub fn new<S>(name: S) -> EnumDescriptor
        where S: Into<String>
    {
        EnumDescriptor {
            name: name.into(),
            values: Vec::new(),
            values_by_name: linked_hash_map::LinkedHashMap::new(),
            values_by_number: linked_hash_map::LinkedHashMap::new(),
        }
    }

    pub fn from_proto(path: &str, proto: &descriptor::EnumDescriptorProto) -> EnumDescriptor {
        let enum_name = format!("{}.{}", path, proto.get_name());

        let mut enum_descriptor = EnumDescriptor::new(enum_name);

        for value_proto in proto.get_value().iter() {
            enum_descriptor.add_value(EnumValueDescriptor::from_proto(value_proto));
        }

        enum_descriptor
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_value(&mut self, descriptor: EnumValueDescriptor) {
        let name = descriptor.name.clone();
        let number = descriptor.number;

        let value_id = EnumValueId(store(&mut self.values, descriptor));

        self.values_by_name.insert(name, value_id);
        self.values_by_number.insert(number, value_id);
    }

    #[inline]
    pub fn value_by_name(&self, name: &str) -> Option<&EnumValueDescriptor> {
        self.values_by_name.get(name).map(|v| &self.values[v.0])
    }

    #[inline]
    pub fn value_by_number(&self, number: i32) -> Option<&EnumValueDescriptor> {
        self.values_by_number.get(&number).map(|v| &self.values[v.0])
    }
}

impl EnumValueDescriptor {
    pub fn new<S>(name: S, number: i32) -> EnumValueDescriptor
        where S: Into<String>
    {
        EnumValueDescriptor {
            name: name.into(),
            number: number,
        }
    }

    pub fn from_proto(proto: &descriptor::EnumValueDescriptorProto) -> EnumValueDescriptor {
        EnumValueDescriptor::new(proto.get_name().to_owned(), proto.get_number())
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn number(&self) -> i32 {
        self.number
    }
}

impl FieldLabel {
    pub fn from_proto(proto: descriptor::FieldDescriptorProto_Label) -> FieldLabel {
        use protobuf::descriptor::FieldDescriptorProto_Label::*;

        match proto {
            LABEL_OPTIONAL => FieldLabel::Optional,
            LABEL_REQUIRED => FieldLabel::Required,
            LABEL_REPEATED => FieldLabel::Repeated,
        }
    }

    #[inline]
    pub fn is_repeated(&self) -> bool {
        *self == FieldLabel::Repeated
    }
}

impl InternalFieldType {
    pub fn from_proto(proto: descriptor::FieldDescriptorProto_Type,
                      type_name: &str)
                      -> InternalFieldType {
        use protobuf::descriptor::FieldDescriptorProto_Type::*;
        match proto {
            TYPE_DOUBLE => InternalFieldType::Double,
            TYPE_FLOAT => InternalFieldType::Float,
            TYPE_INT64 => InternalFieldType::Int64,
            TYPE_UINT64 => InternalFieldType::UInt64,
            TYPE_INT32 => InternalFieldType::Int32,
            TYPE_FIXED64 => InternalFieldType::Fixed64,
            TYPE_FIXED32 => InternalFieldType::Fixed32,
            TYPE_BOOL => InternalFieldType::Bool,
            TYPE_STRING => InternalFieldType::String,
            TYPE_GROUP => InternalFieldType::Group,
            TYPE_MESSAGE => InternalFieldType::UnresolvedMessage(type_name.to_owned()),
            TYPE_BYTES => InternalFieldType::Bytes,
            TYPE_UINT32 => InternalFieldType::UInt32,
            TYPE_ENUM => InternalFieldType::UnresolvedEnum(type_name.to_owned()),
            TYPE_SFIXED32 => InternalFieldType::SFixed32,
            TYPE_SFIXED64 => InternalFieldType::SFixed64,
            TYPE_SINT32 => InternalFieldType::SInt32,
            TYPE_SINT64 => InternalFieldType::SInt64,
        }
    }

    #[inline]
    fn resolve<'a>(&'a self, descriptors: &'a Descriptors) -> FieldType<'a> {
        match *self {
            InternalFieldType::UnresolvedMessage(ref n) => {
                if let Some(m) = descriptors.message_by_name(n) {
                    FieldType::Message(m)
                } else {
                    FieldType::UnresolvedMessage(n)
                }
            },
            InternalFieldType::UnresolvedEnum(ref n) => {
                if let Some(e) = descriptors.enum_by_name(n) {
                    FieldType::Enum(e)
                } else {
                    FieldType::UnresolvedEnum(n)
                }
            },
            InternalFieldType::Double => FieldType::Double,
            InternalFieldType::Float => FieldType::Float,
            InternalFieldType::Int64 => FieldType::Int64,
            InternalFieldType::UInt64 => FieldType::UInt64,
            InternalFieldType::Int32 => FieldType::Int32,
            InternalFieldType::Fixed64 => FieldType::Fixed64,
            InternalFieldType::Fixed32 => FieldType::Fixed32,
            InternalFieldType::Bool => FieldType::Bool,
            InternalFieldType::String => FieldType::String,
            InternalFieldType::Group => FieldType::Group,
            InternalFieldType::Message(m) => FieldType::Message(&descriptors.messages[m.0]),
            InternalFieldType::Bytes => FieldType::Bytes,
            InternalFieldType::UInt32 => FieldType::UInt32,
            InternalFieldType::Enum(e) => FieldType::Enum(&descriptors.enums[e.0]),
            InternalFieldType::SFixed32 => FieldType::SFixed32,
            InternalFieldType::SFixed64 => FieldType::SFixed64,
            InternalFieldType::SInt32 => FieldType::SInt32,
            InternalFieldType::SInt64 => FieldType::SInt64,
        }
    }
}

impl FieldDescriptor {
    pub fn new<S>(name: S,
                  number: i32,
                  field_label: FieldLabel,
                  field_type: InternalFieldType,
                  default_value: Option<value::Value>)
                  -> FieldDescriptor
        where S: Into<String>
    {
        FieldDescriptor {
            name: name.into(),
            number: number,
            field_label: field_label,
            field_type: field_type,
            default_value: default_value,
        }
    }

    pub fn from_proto(proto: &descriptor::FieldDescriptorProto) -> FieldDescriptor {
        let name = proto.get_name().to_owned();
        let number = proto.get_number();
        let field_label = FieldLabel::from_proto(proto.get_label());
        let field_type = InternalFieldType::from_proto(proto.get_field_type(),
                                                       proto.get_type_name());
        let default_value = if proto.has_default_value() {
            // TODO: report error?
            parse_default_value(proto.get_default_value(), &field_type).ok()
        } else {
            None
        };

        FieldDescriptor::new(name, number, field_label, field_type, default_value)
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn number(&self) -> i32 {
        self.number
    }

    #[inline]
    pub fn field_label(&self) -> FieldLabel {
        self.field_label
    }

    #[inline]
    pub fn is_repeated(&self) -> bool {
        self.field_label == FieldLabel::Repeated
    }

    #[inline]
    pub fn field_type<'a>(&'a self, descriptors: &'a Descriptors) -> FieldType<'a> {
        self.field_type.resolve(descriptors)
    }

    #[inline]
    pub fn default_value(&self) -> Option<&value::Value> {
        self.default_value.as_ref()
    }
}

fn store<A>(vec: &mut Vec<A>, elem: A) -> usize {
    let idx = vec.len();
    vec.push(elem);
    idx
}

fn parse_default_value(value: &str, field_type: &InternalFieldType) -> error::Result<value::Value> {
    use std::str::FromStr;

    fn bad(v: &str) -> error::Error {
        error::Error::BadDefaultValue(v.to_owned())
    }

    match *field_type {
        InternalFieldType::UnresolvedMessage(_) |
        InternalFieldType::UnresolvedEnum(_) |
        InternalFieldType::Message(_) |
        InternalFieldType::Enum(_) => Err(bad(value)),
        InternalFieldType::Bool => {
            bool::from_str(value).map(value::Value::Bool).map_err(|_| bad(value))
        },
        InternalFieldType::Double => {
            match value {
                "inf" => Ok(value::Value::F64(f64::INFINITY)),
                "-inf" => Ok(value::Value::F64(f64::NEG_INFINITY)),
                "nan" => Ok(value::Value::F64(f64::NAN)),
                _ => f64::from_str(value).map(value::Value::F64).map_err(|_| bad(value)),
            }
        },
        InternalFieldType::Float => {
            match value {
                "inf" => Ok(value::Value::F32(f32::INFINITY)),
                "-inf" => Ok(value::Value::F32(f32::NEG_INFINITY)),
                "nan" => Ok(value::Value::F32(f32::NAN)),
                _ => f32::from_str(value).map(value::Value::F32).map_err(|_| bad(value)),
            }
        },
        InternalFieldType::Int32 |
        InternalFieldType::SFixed32 |
        InternalFieldType::SInt32 => {
            i32::from_str(value).map(value::Value::I32).map_err(|_| bad(value))
        },
        InternalFieldType::Int64 |
        InternalFieldType::SFixed64 |
        InternalFieldType::SInt64 => {
            i64::from_str(value).map(value::Value::I64).map_err(|_| bad(value))
        },
        InternalFieldType::UInt32 |
        InternalFieldType::Fixed32 => {
            u32::from_str(value).map(value::Value::U32).map_err(|_| bad(value))
        },
        InternalFieldType::UInt64 |
        InternalFieldType::Fixed64 => {
            u64::from_str(value).map(value::Value::U64).map_err(|_| bad(value))
        },
        InternalFieldType::String => Ok(value::Value::String(value.to_owned())),
        InternalFieldType::Group => unimplemented!(),
        InternalFieldType::Bytes => {
            Ok(value::Value::Bytes(value.chars().map(|c| c as u8).collect()))
        },
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use protobuf;

    use super::*;
    use super::FieldType::*;
    use super::FieldLabel::*;

    fn load_descriptors() -> Descriptors {
        let mut file = fs::File::open("testdata/descriptors.pb").unwrap();
        let proto = protobuf::parse_from_reader(&mut file).unwrap();

        Descriptors::from_proto(&proto)
    }

    macro_rules! check_field {
        ($id:ident, $msg:expr, $field:expr, $t:pat, $label:expr, $num:expr) => {
            #[test]
            fn $id() {
                let mut d = load_descriptors();
                d.resolve_refs();
                let msg = d.message_by_name($msg).unwrap();
                let field_by_name = msg.field_by_name($field).unwrap();
                match field_by_name.field_type(&d) {
                    $t => (),
                    t => panic!("Expected type {}, got {:?}", stringify!($t), t),
                }
                assert_eq!(field_by_name.name(), $field);
                assert_eq!(field_by_name.number(), $num);
                assert_eq!(field_by_name.field_label(), $label);

                let field_by_number = msg.field_by_number($num).unwrap();
                match field_by_number.field_type(&d) {
                    $t => (),
                    t => panic!("Expected type {}, got {:?}", stringify!($t), t),
                }
                assert_eq!(field_by_number.name(), $field);
                assert_eq!(field_by_number.number(), $num);
                assert_eq!(field_by_number.field_label(), $label);
            }
        }
    }

    macro_rules! check_enum_value {
        ($id:ident, $enu:expr, $value:expr, $num:expr) => {
            #[test]
            fn $id() {
                let mut d = load_descriptors();
                d.resolve_refs();
                let enu = d.enum_by_name($enu).unwrap();
                let value_by_name = enu.value_by_name($value).unwrap();
                assert_eq!(value_by_name.name(), $value);
                assert_eq!(value_by_name.number(), $num);

                let value_by_number = enu.value_by_number($num).unwrap();
                assert_eq!(value_by_number.name(), $value);
                assert_eq!(value_by_number.number(), $num);
            }
        }
    }

    check_field!(optional_int32_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_int32",
                 Int32,
                 Optional,
                 1);

    check_field!(optional_int64_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_int64",
                 Int64,
                 Optional,
                 2);

    check_field!(optional_uint32_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_uint32",
                 UInt32,
                 Optional,
                 3);

    check_field!(optional_uint64_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_uint64",
                 UInt64,
                 Optional,
                 4);

    check_field!(optional_sint32_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_sint32",
                 SInt32,
                 Optional,
                 5);

    check_field!(optional_sint64_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_sint64",
                 SInt64,
                 Optional,
                 6);

    check_field!(optional_fixed32_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_fixed32",
                 Fixed32,
                 Optional,
                 7);

    check_field!(optional_fixed64_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_fixed64",
                 Fixed64,
                 Optional,
                 8);

    check_field!(optional_sfixed32_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_sfixed32",
                 SFixed32,
                 Optional,
                 9);

    check_field!(optional_sfixed64_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_sfixed64",
                 SFixed64,
                 Optional,
                 10);

    check_field!(optional_float_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_float",
                 Float,
                 Optional,
                 11);

    check_field!(optional_double_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_double",
                 Double,
                 Optional,
                 12);

    check_field!(optional_bool_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_bool",
                 Bool,
                 Optional,
                 13);

    check_field!(optional_string_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_string",
                 String,
                 Optional,
                 14);

    check_field!(optional_bytes_field,
                 ".protobuf_unittest.TestAllTypes",
                 "optional_bytes",
                 Bytes,
                 Optional,
                 15);

    check_field!(repeated_int32_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_int32",
                 Int32,
                 Repeated,
                 31);

    check_field!(repeated_int64_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_int64",
                 Int64,
                 Repeated,
                 32);

    check_field!(repeated_uint32_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_uint32",
                 UInt32,
                 Repeated,
                 33);

    check_field!(repeated_uint64_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_uint64",
                 UInt64,
                 Repeated,
                 34);

    check_field!(repeated_sint32_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_sint32",
                 SInt32,
                 Repeated,
                 35);

    check_field!(repeated_sint64_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_sint64",
                 SInt64,
                 Repeated,
                 36);

    check_field!(repeated_fixed32_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_fixed32",
                 Fixed32,
                 Repeated,
                 37);

    check_field!(repeated_fixed64_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_fixed64",
                 Fixed64,
                 Repeated,
                 38);

    check_field!(repeated_sfixed32_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_sfixed32",
                 SFixed32,
                 Repeated,
                 39);

    check_field!(repeated_sfixed64_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_sfixed64",
                 SFixed64,
                 Repeated,
                 40);

    check_field!(repeated_float_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_float",
                 Float,
                 Repeated,
                 41);

    check_field!(repeated_double_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_double",
                 Double,
                 Repeated,
                 42);

    check_field!(repeated_bool_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_bool",
                 Bool,
                 Repeated,
                 43);

    check_field!(repeated_string_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_string",
                 String,
                 Repeated,
                 44);

    check_field!(repeated_bytes_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_bytes",
                 Bytes,
                 Repeated,
                 45);


    check_field!(repppeated_message_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_foreign_message",
                 Message(..),
                 Repeated,
                 49);

    check_field!(repeated_enum_field,
                 ".protobuf_unittest.TestAllTypes",
                 "repeated_foreign_enum",
                 Enum(..),
                 Repeated,
                 52);

    check_field!(required_field_a,
                 ".protobuf_unittest.TestRequired",
                 "a",
                 Int32,
                 Required,
                 1);

    check_field!(required_field_b,
                 ".protobuf_unittest.TestRequired",
                 "b",
                 Int32,
                 Required,
                 3);

    check_enum_value!(enum_value_foo,
                      ".protobuf_unittest.ForeignEnum",
                      "FOREIGN_FOO",
                      4);

    check_enum_value!(enum_value_bar,
                      ".protobuf_unittest.ForeignEnum",
                      "FOREIGN_BAR",
                      5);

    check_enum_value!(enum_value_baz,
                      ".protobuf_unittest.ForeignEnum",
                      "FOREIGN_BAZ",
                      6);
}
