use error;
use serde;
use serde_avro;
use std::io;
use value;

pub struct AvroSource<R>(serde_avro::de::Deserializer<'static, io::BufReader<serde_avro::de::Blocks<io::BufReader<R>>>>) where R: io::Read;

#[inline]
pub fn source<R>(input: R) -> error::Result<AvroSource<R>> where R: io::Read {
    let de = try!(serde_avro::de::Deserializer::from_container(input));
    Ok(AvroSource(de))
}

impl<R> value::Source for AvroSource<R> where R: io::Read {
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match serde::Deserialize::deserialize(&mut self.0) {
            Ok(v) => Ok(Some(v)),
            Err(e) => match *e.kind() {
                serde_avro::error::ErrorKind::EndOfStream => Ok(None),
                _ => Err(error::Error::from(e))
            },
        }
    }
}
