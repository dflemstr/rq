use std::io;

use serde_json;

use error;
use value;

pub struct JsonValues<Iter>
    where Iter: Iterator<Item = io::Result<u8>>
{
    deserializer: serde_json::de::Deserializer<Iter>,
}

impl<Iter> JsonValues<Iter>
    where Iter: Iterator<Item = io::Result<u8>>
{
    pub fn new(iter: Iter) -> JsonValues<Iter> {
        JsonValues { deserializer: serde_json::Deserializer::new(iter) }
    }
}

impl<Iter> Iterator for JsonValues<Iter>
    where Iter: Iterator<Item = io::Result<u8>>
{
    type Item = error::Result<value::Value>;

    fn next(&mut self) -> Option<Self::Item> {
        use serde::de::Deserialize;
        use serde_json::error::Error::*;
        use serde_json::error::ErrorCode::*;

        match serde_json::Value::deserialize(&mut self.deserializer) {
            Ok(v) => Some(Ok(json_to_value(v))),
            Err(SyntaxError(EOFWhileParsingList, _, _)) => None,
            Err(SyntaxError(EOFWhileParsingObject, _, _)) => None,
            Err(SyntaxError(EOFWhileParsingString, _, _)) => None,
            Err(SyntaxError(EOFWhileParsingValue, _, _)) => None,
            Err(e) => Some(Err(error::Error::from(e))),
        }
    }
}

fn json_to_value(json: serde_json::Value) -> value::Value {
    match json {
        serde_json::Value::Null => value::Value::Unit,
        serde_json::Value::Bool(v) => value::Value::Bool(v),
        serde_json::Value::I64(v) => value::Value::I64(v),
        serde_json::Value::U64(v) => value::Value::U64(v),
        serde_json::Value::F64(v) => value::Value::F64(v),
        serde_json::Value::String(v) => value::Value::String(v),
        serde_json::Value::Array(v) => {
            value::Value::Sequence(v.into_iter().map(json_to_value).collect())
        },
        serde_json::Value::Object(v) => {
            value::Value::Map(v.into_iter()
                               .map(|(k, v)| (k, json_to_value(v)))
                               .collect())
        },
    }
}
