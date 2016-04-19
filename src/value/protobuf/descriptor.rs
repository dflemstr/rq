use std::collections;

use protobuf::descriptor;

#[derive(Debug)]
pub struct Descriptors {
    // All found descriptors
    messages: Vec<MessageDescriptor>,
    enums: Vec<EnumDescriptor>,

    // Indices
    messages_by_name: collections::HashMap<String, usize>,
    enums_by_name: collections::HashMap<String, usize>,
}

// TODO: Support oneof?
#[derive(Debug)]
pub struct MessageDescriptor {
    name: String,

    // All found descriptors
    fields: Vec<FieldDescriptor>,

    // Indices
    fields_by_name: collections::HashMap<String, usize>,
    fields_by_number: collections::HashMap<i32, usize>,
}

#[derive(Debug)]
pub struct EnumDescriptor {
    name: String,

    // All found descriptors
    values: Vec<EnumValueDescriptor>,

    // Indices
    values_by_name: collections::HashMap<String, usize>,
    values_by_number: collections::HashMap<i32, usize>,
}

#[derive(Debug)]
pub struct EnumValueDescriptor {
    name: String,
    number: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldLabel {
    Optional,
    Required,
    Repeated,
}

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

#[derive(Debug)]
enum InternalFieldType {
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
    Message(usize),
    Bytes,
    UInt32,
    Enum(usize),
    SFixed32,
    SFixed64,
    SInt32,
    SInt64,
}

#[derive(Debug)]
pub struct FieldDescriptor {
    name: String,
    number: i32,
    field_label: FieldLabel,
    field_type: InternalFieldType,
}

impl Descriptors {
    pub fn from_protobuf(file_set_proto: &descriptor::FileDescriptorSet) -> Descriptors {
        let mut descriptors = Descriptors::empty();
        descriptors.add_file_set(file_set_proto);
        descriptors.resolve_refs();
        descriptors
    }

    pub fn message_by_name(&self, name: &str) -> Option<&MessageDescriptor> {
        self.messages_by_name.get(name).map(|m| &self.messages[*m])
    }

    pub fn enum_by_name(&self, name: &str) -> Option<&EnumDescriptor> {
        self.enums_by_name.get(name).map(|e| &self.enums[*e])
    }

    fn empty() -> Descriptors {
        Descriptors {
            messages: Vec::new(),
            enums: Vec::new(),

            messages_by_name: collections::HashMap::new(),
            enums_by_name: collections::HashMap::new(),
        }
    }

    fn add_file_set(&mut self, file_set_proto: &descriptor::FileDescriptorSet) {
        for file_proto in file_set_proto.get_file().iter() {
            self.add_file(file_proto);
        }
    }

    fn add_file(&mut self, file_proto: &descriptor::FileDescriptorProto) {
        let path = if file_proto.has_package() {
            format!(".{}", file_proto.get_package())
        } else {
            "".to_owned()
        };

        for message_proto in file_proto.get_message_type().iter() {
            self.add_message(&path, message_proto);
        }

        for enum_proto in file_proto.get_enum_type().iter() {
            self.add_enum(&path, enum_proto);
        }
    }

    fn add_message(&mut self, path: &str, message_proto: &descriptor::DescriptorProto) {
        let path = format!("{}.{}", path, message_proto.get_name());

        for nested_message_proto in message_proto.get_nested_type().iter() {
            self.add_message(&path, nested_message_proto);
        }

        for nested_enum_proto in message_proto.get_enum_type().iter() {
            self.add_enum(&path, nested_enum_proto);
        }

        let mut message_descriptor = MessageDescriptor {
            name: path.clone(),
            fields: vec![],
            fields_by_name: collections::HashMap::new(),
            fields_by_number: collections::HashMap::new(),
        };

        for field_proto in message_proto.get_field().iter() {
            message_descriptor.add_field(field_proto);
        }

        let message_idx = store(&mut self.messages, message_descriptor);
        self.messages_by_name.insert(path, message_idx);
    }

    fn add_enum(&mut self, path: &str, enum_proto: &descriptor::EnumDescriptorProto) {
        let enum_name = format!("{}.{}", path, enum_proto.get_name());

        let mut enum_descriptor = EnumDescriptor {
            name: enum_name.clone(),
            values: vec![], // TODO
            values_by_name: collections::HashMap::new(), // TODO
            values_by_number: collections::HashMap::new(), // TODO
        };

        for value_proto in enum_proto.get_value().iter() {
            enum_descriptor.add_value(value_proto);
        }

        let enum_idx = store(&mut self.enums, enum_descriptor);
        self.enums_by_name.insert(enum_name, enum_idx);
    }

    fn resolve_refs(&mut self) {
        for ref mut m in &mut self.messages {
            for f in &mut m.fields {
                let field_type = &mut f.field_type;
                let new = match *field_type {
                    InternalFieldType::UnresolvedMessage(ref name) => {
                        if let Some(res) = self.messages_by_name.get(name) {
                            Some(InternalFieldType::Message(*res))
                        } else {
                            warn!("Inconsistent schema; unknown message type {}", name);
                            info!("(This might cause parsing to fail later)");
                            None
                        }
                    },
                    InternalFieldType::UnresolvedEnum(ref name) => {
                        if let Some(res) = self.enums_by_name.get(name) {
                            Some(InternalFieldType::Enum(*res))
                        } else {
                            warn!("Inconsistent schema; unknown enum type {}", name);
                            info!("(This might cause parsing to fail later)");
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
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn field_by_name(&self, name: &str) -> Option<&FieldDescriptor> {
        self.fields_by_name.get(name).map(|f| &self.fields[*f])
    }

    pub fn field_by_number(&self, number: i32) -> Option<&FieldDescriptor> {
        self.fields_by_number.get(&number).map(|f| &self.fields[*f])
    }

    fn add_field(&mut self, field: &descriptor::FieldDescriptorProto) {
        use protobuf::descriptor::FieldDescriptorProto_Label::*;
        use protobuf::descriptor::FieldDescriptorProto_Type::*;

        let name = field.get_name().to_owned();
        let number = field.get_number();

        let field_label = match field.get_label() {
            LABEL_OPTIONAL => FieldLabel::Optional,
            LABEL_REQUIRED => FieldLabel::Required,
            LABEL_REPEATED => FieldLabel::Repeated,
        };

        let field_type = match field.get_field_type() {
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
            TYPE_MESSAGE => InternalFieldType::UnresolvedMessage(field.get_type_name().to_owned()),
            TYPE_BYTES => InternalFieldType::Bytes,
            TYPE_UINT32 => InternalFieldType::UInt32,
            TYPE_ENUM => InternalFieldType::UnresolvedEnum(field.get_type_name().to_owned()),
            TYPE_SFIXED32 => InternalFieldType::SFixed32,
            TYPE_SFIXED64 => InternalFieldType::SFixed64,
            TYPE_SINT32 => InternalFieldType::SInt32,
            TYPE_SINT64 => InternalFieldType::SInt64,
        };

        let field_descriptor = FieldDescriptor {
            name: name.clone(),
            number: number,
            field_label: field_label,
            field_type: field_type,
        };

        let field_idx = store(&mut self.fields, field_descriptor);
        self.fields_by_name.insert(name, field_idx);
        self.fields_by_number.insert(number, field_idx);
    }
}

impl EnumDescriptor {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value_by_name(&self, name: &str) -> Option<&EnumValueDescriptor> {
        self.values_by_name.get(name).map(|v| &self.values[*v])
    }

    pub fn value_by_number(&self, number: i32) -> Option<&EnumValueDescriptor> {
        self.values_by_number.get(&number).map(|v| &self.values[*v])
    }

    fn add_value(&mut self, value_proto: &descriptor::EnumValueDescriptorProto) {
        let name = value_proto.get_name().to_owned();
        let number = value_proto.get_number();

        let value_descriptor = EnumValueDescriptor {
            name: name.clone(),
            number: number,
        };

        let value_idx = store(&mut self.values, value_descriptor);
        self.values_by_name.insert(name, value_idx);
        self.values_by_number.insert(number, value_idx);
    }
}

impl EnumValueDescriptor {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn number(&self) -> i32 {
        self.number
    }
}

impl FieldDescriptor {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn number(&self) -> i32 {
        self.number
    }

    pub fn field_label(&self) -> FieldLabel {
        self.field_label
    }

    pub fn field_type<'a>(&'a self, descriptors: &'a Descriptors) -> FieldType<'a> {
        match self.field_type {
            InternalFieldType::UnresolvedMessage(ref m) => FieldType::UnresolvedMessage(m),
            InternalFieldType::UnresolvedEnum(ref e) => FieldType::UnresolvedEnum(e),
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
            InternalFieldType::Message(m) => FieldType::Message(&descriptors.messages[m]),
            InternalFieldType::Bytes => FieldType::Bytes,
            InternalFieldType::UInt32 => FieldType::UInt32,
            InternalFieldType::Enum(e) => FieldType::Enum(&descriptors.enums[e]),
            InternalFieldType::SFixed32 => FieldType::SFixed32,
            InternalFieldType::SFixed64 => FieldType::SFixed64,
            InternalFieldType::SInt32 => FieldType::SInt32,
            InternalFieldType::SInt64 => FieldType::SInt64,
        }
    }
}

fn store<A>(vec: &mut Vec<A>, elem: A) -> usize {
    let idx = vec.len();
    vec.push(elem);
    idx
}
