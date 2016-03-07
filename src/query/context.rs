use std::collections;

use value;

pub type Function = Fn(&[value::Value]) -> value::Value;

pub struct Context {
    functions: collections::HashMap<String, Box<Function>>,
}

impl Context {
    pub fn new() -> Context {
        let mut functions: collections::HashMap<String, Box<Function>> =
            collections::HashMap::new();

        functions.insert("select".to_owned(),
                         Box::new(|values: &[value::Value]| {
                             match values {
                                 [value::Value::Map(ref m), value::Value::String(ref s)] => {
                                     m.get(s).map(|v| v.clone()).unwrap_or(value::Value::Unit)
                                 },
                                 _ => value::Value::Unit,
                             }
                         }));
        functions.insert("id".to_owned(), Box::new(|values: &[value::Value]| values[0].clone()));

        Context { functions: functions }
    }

    pub fn function(&self, name: &str) -> Option<&Box<Function>> {
        self.functions.get(name)
    }
}
