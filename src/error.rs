use std::io;
use std::result;
use std::string;

use serde_json;

pub type Result<A> = result::Result<A, Error>;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    FromUtf8(string::FromUtf8Error),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        match e {
            serde_json::Error::IoError(e) => Error::IO(e),
            serde_json::Error::FromUtf8Error(e) => Error::FromUtf8(e),
            serde_json::Error::SyntaxError(_, _, _) => unimplemented!(),
        }
    }
}
