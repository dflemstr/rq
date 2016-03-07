use std::collections;
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

#[derive(Debug)]
pub struct FieldDescriptor {
    pub name: String,
    pub number: i32,

    pub proto_label: descriptor::FieldDescriptorProto_Label,
    pub proto_type: descriptor::FieldDescriptorProto_Type,
    pub proto_type_name: String,
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
        let name = field.get_name().to_owned();
        let number = field.get_number();

        let field_descriptor = FieldDescriptor {
            name: name.clone(),
            number: number,

            proto_label: field.get_label(),
            proto_type: field.get_field_type(),
            proto_type_name: field.get_type_name().to_owned(),
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
