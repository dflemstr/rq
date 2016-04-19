use std::io;

use serde_cbor;

use error;
use value;

pub struct CborValues<R>
    where R: io::Read
{
    deserializer: serde_cbor::de::Deserializer<R>,
}

impl<R> CborValues<R>
    where R: io::Read
{
    pub fn new(reader: R) -> CborValues<R> {
        CborValues { deserializer: serde_cbor::de::Deserializer::new(reader) }
    }
}

impl<R> Iterator for CborValues<R>
    where R: io::Read
{
    type Item = error::Result<value::Value>;

    fn next(&mut self) -> Option<Self::Item> {
        use serde::de::Deserialize;
        use serde_cbor::error::Error::*;

        match serde_cbor::Value::deserialize(&mut self.deserializer) {
            Ok(v) => Some(Ok(cbor_to_value(v))),
            Err(Eof) => None,
            Err(e) => Some(Err(error::Error::from(e))),
        }
    }
}

fn cbor_to_value(cbor: serde_cbor::Value) -> value::Value {
    match cbor {
        serde_cbor::Value::Null => value::Value::Unit,
        serde_cbor::Value::Bool(v) => value::Value::Bool(v),
        serde_cbor::Value::I64(v) => value::Value::I64(v),
        serde_cbor::Value::U64(v) => value::Value::U64(v),
        serde_cbor::Value::F64(v) => value::Value::F64(v),
        serde_cbor::Value::String(v) => value::Value::String(v),
        serde_cbor::Value::Array(v) => {
            value::Value::Sequence(v.into_iter().map(cbor_to_value).collect())
        },
        serde_cbor::Value::Object(v) => {
            value::Value::Map(v.into_iter()
                               .map(|(k, v)| (cbor_key_to_string(k), cbor_to_value(v)))
                               .collect())
        },
        serde_cbor::Value::Bytes(b) => value::Value::Bytes(b),
    }
}

fn cbor_key_to_string(key: serde_cbor::ObjectKey) -> String {
    match key {
        serde_cbor::ObjectKey::String(s) => s,
        _ => unimplemented!(),
    }
}
