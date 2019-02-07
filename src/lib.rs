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
#[cfg(feature = "js")]
#[macro_use]
extern crate pest;

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
