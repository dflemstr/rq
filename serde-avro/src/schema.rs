use error::{Error, ErrorKind, ChainErr, Result};

use linked_hash_map;
use serde_json;
use std::collections;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SchemaId(usize);

#[derive(Clone, Debug)]
pub enum SchemaRef {
    Direct(Schema),
    Indirect(SchemaId),
}

#[derive(Clone, Debug)]
pub enum Schema {
    Null,
    Boolean,
    Int,
    Long,
    Float,
    Double,
    Bytes,
    String,
    Record(RecordSchema),
    Enum(EnumSchema),
    Array(Box<SchemaRef>),
    Map(Box<SchemaRef>),
    Union(Vec<SchemaRef>),
    Fixed(FixedSchema),
}

#[derive(Clone, Debug)]
pub struct SchemaRegistry {
    schemata: Vec<Schema>,
    next_id: usize,
    schemata_by_name: collections::HashMap<String, SchemaId>,
}

pub struct RecordFields<'a>(linked_hash_map::Values<'a, String, FieldSchema>);

#[derive(Clone, Debug)]
pub struct RecordSchema {
    name: String,
    doc: Option<String>,
    fields: linked_hash_map::LinkedHashMap<String, FieldSchema>,
}

#[derive(Clone, Debug)]
pub struct FieldSchema {
    name: String,
    doc: Option<String>,
    field_type: SchemaRef,
    default: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct EnumSchema {
    name: String,
    doc: Option<String>,
    symbols: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct FixedSchema {
    name: String,
    doc: Option<String>,
    size: i32,
}

lazy_static! {
    pub static ref EMPTY_REGISTRY: SchemaRegistry = {
        SchemaRegistry::new()
    };

    pub static ref FILE_HEADER: Schema = {
        let mut fields = linked_hash_map::LinkedHashMap::new();

        fields.insert("magic".to_owned(), FieldSchema {
            name: "magic".to_owned(),
            doc: None,
            default: None,
            field_type: SchemaRef::Direct(Schema::Fixed(FixedSchema {
                name: "org.apache.avro.file.Magic".to_owned(),
                doc: None,
                size: 4,
            })),
        });

        fields.insert("meta".to_owned(), FieldSchema {
            name: "meta".to_owned(),
            doc: None,
            default: None,
            field_type: SchemaRef::Direct(Schema::Map(Box::new(SchemaRef::Direct(Schema::Bytes)))),
        });

        fields.insert("sync".to_owned(), FieldSchema {
            name: "sync".to_owned(),
            doc: None,
            default: None,
            field_type: SchemaRef::Direct(Schema::Fixed(FixedSchema {
                name: "org.apache.avro.file.Sync".to_owned(),
                doc: None,
                size: 16,
            })),
        });

        Schema::Record(RecordSchema {
            name: "org.apache.avro.file.Header".to_owned(),
            doc: None,
            fields: fields,
        })
    };
}

impl SchemaRef {
    pub fn resolve(&self, registry: &SchemaRegistry) -> Schema {
        // TODO: figure out the lifetimes here (the result *either* has the lifetime of self or
        // registry) so we don't have to clone
        match *self {
            SchemaRef::Direct(ref schema) => schema.clone(),
            SchemaRef::Indirect(id) => registry.schemata[id.0].clone(),
        }
    }

    pub fn into_resolved(self, registry: &SchemaRegistry) -> Schema {
        match self {
            SchemaRef::Direct(schema) => schema,
            SchemaRef::Indirect(id) => registry.schemata[id.0].clone(),
        }
    }
}

impl SchemaRegistry {
    pub fn new() -> SchemaRegistry {
        SchemaRegistry {
            schemata: Vec::new(),
            next_id: 0,
            schemata_by_name: collections::HashMap::new(),
        }
    }

    pub fn from_json(json: &serde_json::Value) -> Result<(SchemaRegistry, Option<SchemaRef>)> {
        let mut result = SchemaRegistry::new();
        let r = try!(result.add_json(json));
        Ok((result, r))
    }

    pub fn add_json(&mut self, json: &serde_json::Value) -> Result<Option<SchemaRef>> {
        match json {
            &serde_json::Value::Array(ref vs) => {
                for v in vs {
                    try!(self.create_schema_ref(None, v));
                }
                Ok(None)
            },
            _ => self.create_schema_ref(None, json).map(Some)
        }
    }

    pub fn schema_by_name(&self, name: &str) -> Option<&Schema> {
        self.schemata_by_name.get(name).map(|id| &self.schemata[id.0])
    }

    fn create_schema_ref(&mut self,
                         namespace: Option<&str>,
                         json: &serde_json::Value)
                         -> Result<SchemaRef> {
        use serde_json::Value;
        use serde_json::Value::*;
        use error::ChainErr;

        match *json {
            String(ref name) => {
                if let Some(primitive) = primitive_schema(name) {
                    Ok(primitive)
                } else {
                    registered_schema(&self.schemata_by_name, namespace, name)
                }
            },
            Object(ref obj) => {
                let name = try!(obj.get("type")
                    .and_then(Value::as_str)
                    .ok_or(Error::from(ErrorKind::InvalidSchema))
                    .chain_err(|| ErrorKind::FieldTypeMismatch("type", "string")));
                if let Some(primitive) = primitive_schema(name) {
                    Ok(primitive)
                } else {
                    match name {
                        "record" => self.create_record(namespace, obj),
                        "enum" => self.create_enum(namespace, obj),
                        "array" => self.create_array(namespace, obj),
                        "map" => self.create_map(namespace, obj),
                        "fixed" => self.create_fixed(namespace, obj),
                        _ => registered_schema(&self.schemata_by_name, namespace, name),
                    }
                }
            },
            Array(ref elems) => {
                let schemas =
                    try!(elems.iter().map(|e| self.create_schema_ref(namespace, e)).collect());
                Ok(SchemaRef::Direct(Schema::Union(schemas)))
            },
            _ => {
                Err(Error::from(ErrorKind::InvalidSchema))
                    .chain_err(|| ErrorKind::FieldTypeMismatch("type", "string, object or array"))
            },
        }
    }

    fn create_record(&mut self,
                     namespace: Option<&str>,
                     obj: &collections::BTreeMap<String, serde_json::Value>)
                     -> Result<SchemaRef> {
        let (namespace, schema_name) = try!(full_name(namespace, obj));
        let schema_id = try!(self.alloc_schema_name(schema_name.clone()));
        // Temporary, replaced below
        self.schemata.push(Schema::Null);

        let fields = try!(obj.get("fields")
            .ok_or(Error::from(ErrorKind::InvalidSchema))
            .chain_err(|| ErrorKind::RequiredFieldMissing("fields"))
            .and_then(|v| {
                v.as_array()
                    .ok_or(Error::from(ErrorKind::InvalidSchema))
                    .chain_err(|| ErrorKind::FieldTypeMismatch("fields", "array"))
            })
            .and_then(|vs| {
                vs.iter()
                    .map(|v| self.create_field(namespace, v))
                    .collect()
            }));

        let doc = try!(obj.get("doc")
            .map(|v| {
                v.as_str()
                    .map(ToOwned::to_owned)
                    .map(Some)
                    .ok_or(Error::from(ErrorKind::InvalidSchema))
                    .chain_err(|| ErrorKind::FieldTypeMismatch("doc", "string"))
            })
            .unwrap_or(Ok(None)));

        self.schemata[schema_id.0] = Schema::Record(RecordSchema {
            name: schema_name,
            doc: doc,
            fields: fields,
        });

        Ok(SchemaRef::Indirect(schema_id))
    }

    fn create_enum(&mut self,
                   namespace: Option<&str>,
                   obj: &collections::BTreeMap<String, serde_json::Value>)
                   -> Result<SchemaRef> {
        let (_, schema_name) = try!(full_name(namespace, obj));
        let schema_id = try!(self.alloc_schema_name(schema_name.clone()));

        let doc = try!(obj.get("doc")
            .map(|v| {
                v.as_str()
                    .map(ToOwned::to_owned)
                    .map(Some)
                    .ok_or(Error::from(ErrorKind::InvalidSchema))
                    .chain_err(|| ErrorKind::FieldTypeMismatch("doc", "string"))
            })
            .unwrap_or(Ok(None)));

        let symbols_array = try!(obj.get("symbols")
            .ok_or(Error::from(ErrorKind::InvalidSchema))
            .chain_err(|| ErrorKind::RequiredFieldMissing("symbols")));
        let symbols = try!(symbols_array.as_array()
            .and_then(|vs| vs.iter().map(|v| v.as_str().map(|s| s.to_owned())).collect())
            .ok_or(Error::from(ErrorKind::InvalidSchema))
            .chain_err(|| ErrorKind::FieldTypeMismatch("symbols", "array of strings")));

        self.schemata.push(Schema::Enum(EnumSchema {
            name: schema_name,
            doc: doc,
            symbols: symbols,
        }));

        Ok(SchemaRef::Indirect(schema_id))
    }

    fn create_array(&mut self,
                    namespace: Option<&str>,
                    obj: &collections::BTreeMap<String, serde_json::Value>)
                    -> Result<SchemaRef> {
        let items = try!(obj.get("items")
            .ok_or(Error::from(ErrorKind::InvalidSchema))
            .chain_err(|| ErrorKind::RequiredFieldMissing("items")));
        let items_schema = try!(self.create_schema_ref(namespace, items));

        Ok(SchemaRef::Direct(Schema::Array(Box::new(items_schema))))
    }

    fn create_map(&mut self,
                  namespace: Option<&str>,
                  obj: &collections::BTreeMap<String, serde_json::Value>)
                  -> Result<SchemaRef> {
        let values = try!(obj.get("values")
            .ok_or(Error::from(ErrorKind::InvalidSchema))
            .chain_err(|| ErrorKind::RequiredFieldMissing("values")));
        let values_schema = try!(self.create_schema_ref(namespace, values));

        Ok(SchemaRef::Direct(Schema::Map(Box::new(values_schema))))
    }

    fn create_fixed(&mut self,
                    namespace: Option<&str>,
                    obj: &collections::BTreeMap<String, serde_json::Value>)
                    -> Result<SchemaRef> {
        let (_, schema_name) = try!(full_name(namespace, obj));
        let schema_id = try!(self.alloc_schema_name(schema_name.clone()));

        let doc = try!(obj.get("doc")
            .map(|v| {
                v.as_str()
                    .map(ToOwned::to_owned)
                    .map(Some)
                    .ok_or(Error::from(ErrorKind::InvalidSchema))
                    .chain_err(|| ErrorKind::FieldTypeMismatch("doc", "string"))
            })
            .unwrap_or(Ok(None)));

        let size = try!(obj.get("size")
            .and_then(serde_json::Value::as_i64)
            .ok_or(Error::from(ErrorKind::InvalidSchema))
            .chain_err(|| ErrorKind::RequiredFieldMissing("size")));

        self.schemata.push(Schema::Fixed(FixedSchema {
            name: schema_name,
            doc: doc,
            size: size as i32,
        }));

        Ok(SchemaRef::Indirect(schema_id))
    }

    fn alloc_schema_name(&mut self, name: String) -> Result<SchemaId> {
        use std::collections::hash_map::Entry;

        match self.schemata_by_name.entry(name) {
            Entry::Occupied(e) => Err(Error::from(ErrorKind::DuplicateSchema(e.key().clone()))),
            Entry::Vacant(e) => {
                let schema_id = SchemaId(self.next_id);
                self.next_id += 1;

                e.insert(schema_id);

                Ok(schema_id)
            },
        }
    }

    fn create_field(&mut self,
                    namespace: Option<&str>,
                    json: &serde_json::Value)
                    -> Result<(String, FieldSchema)> {
        let name = try!(json.find("name")
            .and_then(serde_json::Value::as_str)
            .ok_or(Error::from(ErrorKind::InvalidSchema))
            .chain_err(|| ErrorKind::RequiredFieldMissing("name")));
        let doc = try!(json.find("doc")
            .map(|v| {
                v.as_str()
                    .map(ToOwned::to_owned)
                    .map(Some)
                    .ok_or(Error::from(ErrorKind::InvalidSchema))
                    .chain_err(|| ErrorKind::FieldTypeMismatch("doc", "string"))
            })
            .unwrap_or(Ok(None)));
        let field_type = try!(json.find("type")
            .ok_or(Error::from(ErrorKind::InvalidSchema))
            .chain_err(|| ErrorKind::RequiredFieldMissing("name"))
            .and_then(|t| self.create_schema_ref(namespace, t)));

        let schema = FieldSchema {
            name: name.to_owned(),
            doc: doc,
            field_type: field_type,
            default: json.find("default").cloned(),
        };

        Ok((name.to_owned(), schema))
    }
}

impl<'a> Iterator for RecordFields<'a> {
    type Item = &'a FieldSchema;

    fn next(&mut self) -> Option<&'a FieldSchema> {
        self.0.next()
    }
}

impl<'a> ExactSizeIterator for RecordFields<'a> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl RecordSchema {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> Option<&str> {
        if let Some(ref doc) = self.doc {
            Some(doc.as_str())
        } else {
            None
        }
    }

    pub fn fields(&self) -> RecordFields {
        RecordFields(self.fields.values())
    }

    pub fn field_by_name(&self, name: &str) -> Option<&FieldSchema> {
        self.fields.get(name)
    }
}

impl FieldSchema {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> Option<&str> {
        if let Some(ref doc) = self.doc {
            Some(doc.as_str())
        } else {
            None
        }
    }

    pub fn field_type(&self) -> &SchemaRef {
        &self.field_type
    }

    pub fn default(&self) -> Option<&serde_json::Value> {
        if let Some(ref default) = self.default {
            Some(default)
        } else {
            None
        }
    }
}

impl EnumSchema {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> Option<&str> {
        if let Some(ref doc) = self.doc {
            Some(doc.as_str())
        } else {
            None
        }
    }

    pub fn symbols(&self) -> &[String] {
        &self.symbols
    }
}

impl FixedSchema {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> Option<&str> {
        if let Some(ref doc) = self.doc {
            Some(doc.as_str())
        } else {
            None
        }
    }

    pub fn size(&self) -> i32 {
        self.size
    }
}

fn full_name<'a>(namespace: Option<&'a str>,
                 obj: &'a collections::BTreeMap<String, serde_json::Value>)
                 -> Result<(Option<&'a str>, String)> {
    let namespace = try!(obj.get("namespace")
        .map(|v| {
            v.as_str()
                .map(Some)
                .ok_or(Error::from(ErrorKind::InvalidSchema))
                .chain_err(|| ErrorKind::FieldTypeMismatch("namespace", "string"))
        })
        .unwrap_or(Ok(namespace)));

    let name = try!(obj.get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or(Error::from(ErrorKind::InvalidSchema))
        .chain_err(|| ErrorKind::RequiredFieldMissing("name")));

    if let Some(ns) = namespace {
        Ok((Some(ns), format!("{}.{}", ns, name)))
    } else {
        Ok((namespace, name.to_owned()))
    }
}

fn registered_schema(registry: &collections::HashMap<String, SchemaId>,
                     namespace: Option<&str>,
                     name: &str)
                     -> Result<SchemaRef> {
    match registry.get(name) {
        Some(id) => Ok(SchemaRef::Indirect(*id)),
        None => {
            match namespace.and_then(|ns| registry.get(&format!("{}.{}", ns, name))) {
                Some(id) => Ok(SchemaRef::Indirect(*id)),
                None => Err(ErrorKind::NoSuchType(name.to_owned()).into()),
            }
        },
    }
}

fn primitive_schema(name: &str) -> Option<SchemaRef> {
    match name {
        "null" => Some(SchemaRef::Direct(Schema::Null)),
        "boolean" => Some(SchemaRef::Direct(Schema::Boolean)),
        "int" => Some(SchemaRef::Direct(Schema::Int)),
        "long" => Some(SchemaRef::Direct(Schema::Long)),
        "float" => Some(SchemaRef::Direct(Schema::Float)),
        "double" => Some(SchemaRef::Direct(Schema::Double)),
        "bytes" => Some(SchemaRef::Direct(Schema::Bytes)),
        "string" => Some(SchemaRef::Direct(Schema::String)),
        _ => None,
    }
}

#[cfg(test)]
mod test {

    use serde_json;
    use super::*;

    #[test]
    fn parse_record_schema() {
        let schema = serde_json::from_str(r#"
          {
            "namespace": "example.avro",
            "type": "record",
            "name": "Record",
            "fields": [
              {"name": "null", "type": "null"},
              {"name": "boolean", "type": "boolean"},
              {"name": "int", "type": "int"},
              {"name": "long", "type": "long"},
              {"name": "float", "type": "float"},
              {"name": "double", "type": "double"},
              {"name": "bytes", "type": "bytes"},
              {"name": "string", "type": "string"},
              {
                "name": "record",
                "type": {
                  "type": "record",
                  "name": "SubRecord",
                  "fields": [
                    {"name": "null", "type": "null"},
                    {"name": "boolean", "type": "boolean"},
                    {"name": "int", "type": "int"},
                    {"name": "long", "type": "long"},
                    {"name": "float", "type": "float"},
                    {"name": "double", "type": "double"},
                    {"name": "bytes", "type": "bytes"},
                    {"name": "string", "type": "string"},
                    {
                      "name": "enum",
                      "type": {
                        "type": "enum",
                        "name": "SubEnum",
                        "symbols": ["A", "B"]
                      }
                    },
                    {
                      "name": "array",
                      "type": {
                        "type": "array",
                        "items": "string"
                      }
                    },
                    {
                      "name": "map",
                      "type": {
                        "type": "map",
                        "values": "string"
                      }
                    },
                    {
                      "name": "union",
                      "type": ["null", "string", "int"]
                    },
                    {
                      "name": "fixed",
                      "type": {
                        "namespace": "example.avro",
                        "type": "fixed",
                        "name": "SubId",
                        "size": 32
                      }
                    }
                  ]
                }
              },
              {
                "name": "enum",
                "type": {
                  "type": "enum",
                  "name": "Enum",
                  "symbols": ["A", "B"]
                }
              },
              {
                "name": "array",
                "type": {
                  "type": "array",
                  "items": "string"
                }
              },
              {
                "name": "map",
                "type": {
                  "type": "map",
                  "values": "string"
                }
              },
              {
                "name": "union",
                "type": ["null", "string", "int"]
              },
              {
                "name": "fixed",
                "type": {
                  "namespace": "example.avro",
                  "type": "fixed",
                  "name": "Id",
                  "size": 32
                }
              }
            ]
          }
        "#)
            .unwrap();
        let (schema_registry, _) = SchemaRegistry::from_json(&schema).unwrap();

        println!("{:?}", schema_registry);

        match schema_registry.schema_by_name("example.avro.Record") {
            Some(&Schema::Record(ref record)) => {
                let fields = record.fields().collect::<Vec<_>>();
                assert_eq!("null", fields[0].name());
                assert_eq!("boolean", fields[1].name());
                assert_eq!("int", fields[2].name());
                assert_eq!("long", fields[3].name());
                assert_eq!("float", fields[4].name());
                assert_eq!("double", fields[5].name());
                assert_eq!("bytes", fields[6].name());
                assert_eq!("string", fields[7].name());
                assert_eq!("record", fields[8].name());
                assert_eq!("enum", fields[9].name());
                assert_eq!("array", fields[10].name());
                assert_eq!("map", fields[11].name());
                assert_eq!("union", fields[12].name());
                assert_eq!("fixed", fields[13].name());

                // TODO: expand this test
            },
            _ => unreachable!(),
        }
    }

    #[test]
    fn parse_recursive_record_schema() {
        let schema = serde_json::from_str(r#"
          {
            "namespace": "example.avro",
            "type": "record",
            "name": "User",
            "fields": [
              {"name": "parent", "type": "User"}
            ]
          }
        "#)
            .unwrap();
        let (schema_registry, _) = SchemaRegistry::from_json(&schema).unwrap();

        match schema_registry.schema_by_name("example.avro.User") {
            Some(&Schema::Record(ref record)) => {
                assert_eq!("example.avro.User", record.name());
                match record.field_by_name("parent")
                    .unwrap()
                    .field_type()
                    .resolve(&schema_registry) {
                    Schema::Record(ref record) => {
                        assert_eq!("example.avro.User", record.name());
                    },
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }

    #[test]
    fn parse_recursive_qualified_record_schema() {
        let schema = serde_json::from_str(r#"
          {
            "namespace": "example.avro",
            "type": "record",
            "name": "User",
            "fields": [
              {"name": "parent", "type": "example.avro.User"}
            ]
          }
        "#)
            .unwrap();
        let (schema_registry, _) = SchemaRegistry::from_json(&schema).unwrap();

        match schema_registry.schema_by_name("example.avro.User") {
            Some(&Schema::Record(ref record)) => {
                assert_eq!("example.avro.User", record.name());
                match record.field_by_name("parent")
                    .unwrap()
                    .field_type()
                    .resolve(&schema_registry) {
                    Schema::Record(ref record) => {
                        assert_eq!("example.avro.User", record.name());
                    },
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }

    #[test]
    fn parse_enum_schema() {
        let schema = serde_json::from_str(r#"
          {
            "namespace": "example.avro",
            "type": "enum",
            "name": "User",
            "symbols": ["Adam", "Eve"]
          }
        "#)
            .unwrap();
        let (schema_registry, _) = SchemaRegistry::from_json(&schema).unwrap();

        match schema_registry.schema_by_name("example.avro.User") {
            Some(&Schema::Enum(ref enu)) => {
                assert_eq!("example.avro.User", enu.name());
                assert_eq!(&["Adam".to_owned(), "Eve".to_owned()], enu.symbols());
            },
            _ => unreachable!(),
        }
    }

    #[test]
    fn parse_fixed_schema() {
        let schema = serde_json::from_str(r#"
          {
            "namespace": "example.avro",
            "type": "fixed",
            "name": "Id",
            "size": 32
          }
        "#)
            .unwrap();
        let (schema_registry, _) = SchemaRegistry::from_json(&schema).unwrap();

        match schema_registry.schema_by_name("example.avro.Id") {
            Some(&Schema::Fixed(ref fixed)) => {
                assert_eq!("example.avro.Id", fixed.name());
                assert_eq!(32, fixed.size());
            },
            _ => unreachable!(),
        }
    }
}
