#![feature(advanced_slice_patterns, plugin, slice_patterns)]
#![plugin(clippy, mod_path)]

extern crate glob;
#[macro_use]
extern crate log;
extern crate protobuf;
extern crate serde;
extern crate serde_cbor;
extern crate serde_json;
extern crate xdg_basedir;

pub mod config;
pub mod error;
pub mod proto_index;
pub mod query;
pub mod value;
