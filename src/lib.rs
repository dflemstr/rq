//! `rq` (Record Query) is a library and command-line tool for manipulating structured (record)
//! data.

// For pest parser generation
#![recursion_limit = "1024"]
#![deny(warnings)]
#![deny(clippy::all)]
#![deny(
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate pest;

pub mod config;
pub mod error;
pub mod proto_index;
pub mod value;

pub const VERSION: &str = env!("VERGEN_GIT_SEMVER");

#[doc(hidden)]
#[deprecated(since = "1.0.1", note = "use VERSION instead")]
pub const GIT_VERSION: &str = VERSION;
