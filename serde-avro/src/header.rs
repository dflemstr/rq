use serde;
use std::collections;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Header {
    pub magic: serde::bytes::ByteBuf,
    pub meta: collections::HashMap<String, serde::bytes::ByteBuf>,
    pub sync: serde::bytes::ByteBuf,
}
