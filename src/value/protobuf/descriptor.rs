use std::collections;
use std::rc;

use protobuf::descriptor;

pub struct Descriptors {
    // All found descriptors
    messages: collections::HashSet<rc::Rc<MessageDescriptor>>,
    enums: collections::HashSet<rc::Rc<EnumDescriptor>>,
    fields: collections::HashSet<rc::Rc<FieldDescriptor>>,
}

struct MessageDescriptor {
    name: String,
}

struct EnumDescriptor {
    name: String,
}

struct FieldDescriptor {
    name: String,
}

impl Descriptors {
    pub fn from_proto(descriptors: descriptor::FileDescriptorSet) -> Descriptors {
        for file in descriptors.get_file().iter() {
            println!("{:?}", file);
        }
        unimplemented!();
    }
}
