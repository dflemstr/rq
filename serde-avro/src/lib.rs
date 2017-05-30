#![recursion_limit = "1024"]

extern crate byteorder;
extern crate crc;
#[macro_use]
extern crate error_chain;
extern crate flate2;
#[macro_use]
extern crate lazy_static;
extern crate linked_hash_map;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_bytes;
extern crate snap;

mod header;
pub mod de;
pub mod error;
pub mod schema;
