use std::collections;
use std::fmt;
use std::rc;

use protobuf::descriptor;

#[derive(Debug)]
pub struct Descriptors {
    // All found descriptors
    pub messages: Vec<rc::Rc<MessageDescriptor>>,
    pub enums: Vec<rc::Rc<EnumDescriptor>>,

    // Indices
    pub messages_by_name: collections::HashMap<String, rc::Weak<MessageDescriptor>>,
    pub enums_by_name: collections::HashMap<String, rc::Weak<EnumDescriptor>>,
}

// TODO: Support oneof?
#[derive(Debug)]
pub struct MessageDescriptor {
    pub name: String,

    // All found descriptors
    pub fields: Vec<rc::Rc<FieldDescriptor>>,

    // Indices
    pub fields_by_name: collections::HashMap<String, rc::Weak<FieldDescriptor>>,
    pub fields_by_number: collections::HashMap<i32, rc::Weak<FieldDescriptor>>,
}

#[derive(Debug)]
pub struct EnumDescriptor {
    pub name: String,

    // All found descriptors
    pub values: Vec<rc::Rc<EnumValueDescriptor>>,

    // Indices
    pub values_by_name: collections::HashMap<String, rc::Weak<EnumValueDescriptor>>,
    pub values_by_number: collections::HashMap<i32, rc::Weak<EnumValueDescriptor>>,
}

#[derive(Debug)]
pub struct EnumValueDescriptor {
    pub name: String,
    pub number: i32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Label {
    Optional,
    Required,
    Repeated,
}

#[derive(Debug)]
pub enum FieldType {
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
    Message(rc::Weak<MessageDescriptor>),
    Bytes,
    UInt32,
    Enum(rc::Weak<EnumDescriptor>),
    SFixed32,
    SFixed64,
    SInt32,
    SInt64,
}

#[derive(Debug)]
pub struct FieldDescriptor {
    pub name: String,
    pub number: i32,
    pub label: Label,
    pub field_type: FieldType,
}

impl Descriptors {
    pub fn from_proto(proto: &descriptor::FileDescriptorSet) -> Descriptors {
        let mut descriptors = Descriptors {
            messages: Vec::new(),
            enums: Vec::new(),

            messages_by_name: collections::HashMap::new(),
            enums_by_name: collections::HashMap::new(),
        };

        descriptors.add_file_set(proto);

        descriptors
    }

    fn add_file_set(&mut self, file_set: &descriptor::FileDescriptorSet) {
        for file in file_set.get_file().iter() {
            self.add_file(file);
        }
    }

    fn add_file(&mut self, file: &descriptor::FileDescriptorProto) {
        let path = if file.has_package() {
            format!(".{}", file.get_package())
        } else {
            "".to_owned()
        };

        for message_type in file.get_message_type().iter() {
            self.add_message_type(&path, message_type);
        }

        for enum_type in file.get_enum_type().iter() {
            self.add_enum_type(&path, enum_type);
        }
    }

    fn add_message_type(&mut self, path: &str, message_type: &descriptor::DescriptorProto) {
        let path = format!("{}.{}", path, message_type.get_name());

        for nested_type in message_type.get_nested_type().iter() {
            self.add_message_type(&path, nested_type);
        }

        for enum_type in message_type.get_enum_type().iter() {
            self.add_enum_type(&path, enum_type);
        }

        let mut message_descriptor = MessageDescriptor {
            name: path.clone(),
            fields: vec![],
            fields_by_name: collections::HashMap::new(),
            fields_by_number: collections::HashMap::new(),
        };

        for field in message_type.get_field().iter() {
            message_descriptor.add_field(field);
        }

        let message_ref = rc::Rc::new(message_descriptor);
        let weak_by_name = rc::Rc::downgrade(&message_ref);

        self.messages_by_name.insert(path, weak_by_name);
        self.messages.push(message_ref);
    }

    fn add_enum_type(&mut self, path: &str, enum_type: &descriptor::EnumDescriptorProto) {
        let enum_name = format!("{}.{}", path, enum_type.get_name());

        let mut enum_descriptor = EnumDescriptor {
            name: enum_name.clone(),
            values: vec![], // TODO
            values_by_name: collections::HashMap::new(), // TODO
            values_by_number: collections::HashMap::new(), // TODO
        };

        for value in enum_type.get_value().iter() {
            enum_descriptor.add_value(value);
        }

        let enum_ref = rc::Rc::new(enum_descriptor);
        let weak_by_name = rc::Rc::downgrade(&enum_ref);

        self.enums_by_name.insert(enum_name, weak_by_name);
        self.enums.push(enum_ref);
    }
}

impl MessageDescriptor {
    fn add_field(&mut self, field: &descriptor::FieldDescriptorProto) {
        use protobuf::descriptor::FieldDescriptorProto_Label::*;
        use protobuf::descriptor::FieldDescriptorProto_Type::*;

        let name = field.get_name().to_owned();
        let number = field.get_number();

        let label = match field.get_label() {
            LABEL_OPTIONAL => Label::Optional,
            LABEL_REQUIRED => Label::Required,
            LABEL_REPEATED => Label::Repeated,
        };

        let field_type = match field.get_field_type() {
            TYPE_DOUBLE => FieldType::Double,
            TYPE_FLOAT => FieldType::Float,
            TYPE_INT64 => FieldType::Int64,
            TYPE_UINT64 => FieldType::UInt64,
            TYPE_INT32 => FieldType::Int32,
            TYPE_FIXED64 => FieldType::Fixed64,
            TYPE_FIXED32 => FieldType::Fixed32,
            TYPE_BOOL => FieldType::Bool,
            TYPE_STRING => FieldType::String,
            TYPE_GROUP => FieldType::Group,
            TYPE_MESSAGE =>
                FieldType::UnresolvedMessage(field.get_type_name().to_owned()),
            TYPE_BYTES => FieldType::Bytes,
            TYPE_UINT32 => FieldType::UInt32,
            TYPE_ENUM =>
                FieldType::UnresolvedEnum(field.get_type_name().to_owned()),
            TYPE_SFIXED32 => FieldType::SFixed32,
            TYPE_SFIXED64 => FieldType::SFixed64,
            TYPE_SINT32 => FieldType::SInt32,
            TYPE_SINT64 => FieldType::SInt64,
        };

        let field_descriptor = FieldDescriptor {
            name: name.clone(),
            number: number,
            label: label,
            field_type: field_type,
        };

        let field_ref = rc::Rc::new(field_descriptor);
        let weak_by_name = rc::Rc::downgrade(&field_ref);
        let weak_by_number = rc::Rc::downgrade(&field_ref);

        self.fields_by_name.insert(name, weak_by_name);
        self.fields_by_number.insert(number, weak_by_number);
        self.fields.push(field_ref);
    }
}

impl EnumDescriptor {
    fn add_value(&mut self, value: &descriptor::EnumValueDescriptorProto) {
        let name = value.get_name().to_owned();
        let number = value.get_number();

        let value_descriptor = EnumValueDescriptor {
            name: name.clone(),
            number: number,
        };

        let value_ref = rc::Rc::new(value_descriptor);
        let weak_by_name = rc::Rc::downgrade(&value_ref);
        let weak_by_number = rc::Rc::downgrade(&value_ref);

        self.values_by_name.insert(name, weak_by_name);
        self.values_by_number.insert(number, weak_by_number);
        self.values.push(value_ref);
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Label::Optional => write!(f, "optional"),
            Label::Required => write!(f, "required"),
            Label::Repeated => write!(f, "repeated"),
        }
    }
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldType::UnresolvedMessage(ref n) => write!(f, "(Unresolved message {:?})", n),
            FieldType::UnresolvedEnum(ref n) => write!(f, "(Unresolved enum {:?})", n),
            FieldType::Double => write!(f, "double"),
            FieldType::Float => write!(f, "float"),
            FieldType::Int64 => write!(f, "int64"),
            FieldType::UInt64 => write!(f, "uint64"),
            FieldType::Int32 => write!(f, "int32"),
            FieldType::Fixed64 => write!(f, "fixed64"),
            FieldType::Fixed32 => write!(f, "fixed32"),
            FieldType::Bool => write!(f, "bool"),
            FieldType::String => write!(f, "string"),
            FieldType::Group => write!(f, "group"),
            FieldType::Message(ref msg) =>
                match msg.upgrade() {
                    Some(m) => write!(f, "message {}", m.name),
                    None => write!(f, "deallocated message"),
                },
            FieldType::Bytes => write!(f, "bytes"),
            FieldType::UInt32 => write!(f, "uint32"),
            FieldType::Enum(ref enu) =>
                match enu.upgrade() {
                    Some(e) => write!(f, "enum {}", e.name),
                    None => write!(f, "deallocated enum"),
                },
            FieldType::SFixed32 => write!(f, "sfixed32"),
            FieldType::SFixed64 => write!(f, "sfixed64"),
            FieldType::SInt32 => write!(f, "sint32"),
            FieldType::SInt64 => write!(f, "sint64"),
        }
    }
}

impl fmt::Display for FieldDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{:?} (number {}, {}, {})",
               self.name,
               self.number,
               self.label,
               self.field_type)
    }
}
