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

extern crate ansi_term;
extern crate avro_rs;
extern crate dtoa;
#[macro_use]
extern crate failure;
extern crate glob;
extern crate itoa;
#[macro_use]
extern crate log;
extern crate csv;
extern crate ordered_float;
#[cfg(feature = "js")]
#[macro_use]
extern crate pest;
extern crate protobuf;
extern crate rmpv;
extern crate serde;
extern crate serde_cbor;
extern crate serde_hjson;
extern crate serde_json;
extern crate serde_protobuf;
extern crate serde_yaml;
extern crate toml;
#[cfg(feature = "js")]
extern crate v8;
extern crate xdg_basedir;
extern crate yaml_rust;

pub mod config;
pub mod error;
pub mod proto_index;
#[cfg(feature = "js")]
pub mod query;
pub mod value;

#[cfg(feature = "js")]
pub fn run_query<I, O>(query: &query::Query, source: I, mut sink: O) -> error::Result<()>
where
    I: value::Source,
    O: value::Sink,
{
    let query_context = query::Context::new();

    let mut results = query.evaluate(&query_context, source)?;

    while let Some(result) = value::Source::read(&mut results)? {
        sink.write(result)?;
    }

    Ok(())
}
