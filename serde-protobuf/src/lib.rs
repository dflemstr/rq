//! Support for [Google protocol buffers][1] in combination with `serde`.
//!
//! The crate is split up into several logical parts.
//!
//!   * The [`descriptor`](descriptor/index.html) module provides an API for managing dynamically
//!     loaded protocol buffer schemata.
//!   * The [`value`](value/index.html) module provides structs that can hold any raw protocol
//!     buffer decoded data (but the representation is heavily coupled with a schema).
//!   * The [`de`](de/index.html) module can be used to deserialize binary encoded protocol buffer
//!     messages given some schema descriptors.
//!
//! Serialization is not yet implemented in this version.
//!
//! [1]: https://developers.google.com/protocol-buffers/
#[macro_use]
extern crate error_chain;
extern crate linked_hash_map;
#[macro_use]
extern crate log;
extern crate protobuf;
#[macro_use]
extern crate serde;

pub mod de;
pub mod descriptor;
pub mod error;
pub mod ser;
pub mod value;

pub use error::Error;
