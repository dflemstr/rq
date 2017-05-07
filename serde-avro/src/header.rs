use serde_bytes;
use std::collections;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Header {
    pub magic: serde_bytes::ByteBuf,
    pub meta: collections::HashMap<String, serde_bytes::ByteBuf>,
    pub sync: serde_bytes::ByteBuf,
}
