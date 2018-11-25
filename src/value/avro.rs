use error;
use serde;
use serde_avro;
use avro_rs;
use std;
use std::io;
use value;

pub struct AvroSource<R>(serde_avro::de::Deserializer<'static, serde_avro::de::read::Blocks<R>>)
    where R: io::Read;

pub struct AvroSink<'a, W>(avro_rs::Writer<'a, W>) where W: io::Write;

#[inline]
pub fn source<R>(input: R) -> error::Result<AvroSource<R>>
    where R: io::Read
{
    let de = serde_avro::de::Deserializer::from_container(input)?;
    Ok(AvroSource(de))
}

#[inline]
pub fn sink<W>(schema: &avro_rs::Schema, w: W) -> error::Result<AvroSink<W>> where W: io::Write
{
    Ok(AvroSink(avro_rs::Writer::new(schema, w)))
}

impl<R> value::Source for AvroSource<R>
    where R: io::Read
{
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match serde::Deserialize::deserialize(&mut self.0) {
            Ok(v) => Ok(Some(v)),
            Err(e) => {
                match *e.kind() {
                    serde_avro::error::ErrorKind::EndOfStream => Ok(None),
                    _ => Err(error::Error::from(e)),
                }
            },
        }
    }
}

impl<'a, W> value::Sink for AvroSink<'a, W> where W: io::Write
{
    #[inline]
    fn write(&mut self, value: value::Value) -> error::Result<()> {
        self.0.append(value_to_avro(value)?)?;
        Ok(())
    }
}

fn value_to_avro(value: value::Value) -> error::Result<avro_rs::types::Value> {
    use avro_rs::types::Value;
    match value {
        value::Value::Unit => Ok(Value::Null),
        value::Value::Bool(v) => Ok(Value::Boolean(v)),

        value::Value::I8(v) => Ok(Value::Int(v as i32)),
        value::Value::I16(v) => Ok(Value::Int(v as i32)),
        value::Value::I32(v) => Ok(Value::Int(v as i32)),
        value::Value::I64(v) => Ok(Value::Long(v)),

        value::Value::U8(v) => Ok(Value::Int(v as i32)),
        value::Value::U16(v) => Ok(Value::Int(v as i32)),
        value::Value::U32(v) => Ok(Value::Long(v as i64)),
        value::Value::U64(v) =>
            if v <= std::i64::MAX as u64 {
                Ok(Value::Long(v as i64))
            } else {
                bail!("Avro output does not support unsigned 64 bit integer: {}", v)
            },

        value::Value::F32(ordered_float::OrderedFloat(v)) => Ok(Value::Float(v)),
        value::Value::F64(ordered_float::OrderedFloat(v)) => Ok(Value::Double(v)),

        value::Value::Char(v) => Ok(Value::String(format!("{}", v))),
        value::Value::String(v) => Ok(Value::String(v)),
        value::Value::Bytes(v) => Ok(Value::Bytes(v)),

        value::Value::Sequence(v) => {
            Ok(Value::Array(v.into_iter().map(|v| value_to_avro(v)).collect::<error::Result<Vec<_>>>()?))
        },
        value::Value::Map(v) => {
            Ok(Value::Record(v.into_iter()
                .map(|(k, v)| {
                    match (value_to_string(k), value_to_avro(v)) {
                        (Ok(k), Ok(v)) => Ok((k, v)),
                        (Ok(_), Err(e)) => Err(e),
                        (Err(e), Ok(_)) => Err(e),
                        (Err(_), Err(e)) => Err(e)
                    }
                })
                .collect::<error::Result<Vec<_>>>()?))
        }
    }
}

fn value_to_string(value: value::Value) -> error::Result<String> {
    match value {
        value::Value::Char(v) => Ok(format!("{}", v)),
        value::Value::String(v) => Ok(v),
        x => bail!("Avro can only output string keys, got: {:?}", x)
    }
}

impl<'a, W> Drop for AvroSink<'a, W> where W: io::Write
{
    fn drop(&mut self) {
        match self.0.flush() {
            Ok(_) => (),
            Err(error) => panic!(error)
        }
    }
}
