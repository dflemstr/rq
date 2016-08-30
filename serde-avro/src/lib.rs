#![cfg_attr(feature = "serde_macros", feature(plugin, custom_derive))]
#![cfg_attr(feature = "serde_macros", plugin(serde_macros))]

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
extern crate serde;
extern crate serde_json;
extern crate snap;

#[macro_use]
mod forward;

mod header {
    #[cfg(feature = "serde_macros")]
    include!("header.in.rs");

    #[cfg(feature = "serde_codegen")]
    include!(concat!(env!("OUT_DIR"), "/header.rs"));
}

pub mod de;
pub mod error;
pub mod schema;
