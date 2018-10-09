use serde;
use std::{collections};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Header {
    pub magic: serde::bytes::ByteBuf,
    pub meta: collections::HashMap<String, serde::bytes::ByteBuf>,
    pub sync: serde::bytes::ByteBuf,
}

impl Header {

    /// Returns a copy of the given header's name field as a String.
    pub fn name(&self) -> String {
        let raw = self.get("name").expect("No name present in Avro schema!");
        String::from_utf8(raw.to_vec()).expect("Failed to decode name!")
    }

    /// Returns a copy of the raw bytes stored at the given key
    /// for the given header.
    pub fn get(&self, key: &str) -> Option<serde::bytes::ByteBuf> {
        if let Some(bytes) = self.meta.get(key) {
            Some(bytes.clone())
        } else {
            None
        }
    }
}
