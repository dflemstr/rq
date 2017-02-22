use std::io;

use ordered_float;
use rmpv;

use error;
use value;

pub struct MessagePackSource<R>(R) where R: io::Read;

pub struct MessagePackSink<W>(W) where W: io::Write;

#[inline]
pub fn source<R>(r: R) -> MessagePackSource<R>
    where R: io::Read
{
    MessagePackSource(r)
}

#[inline]
pub fn sink<W>(w: W) -> MessagePackSink<W>
    where W: io::Write
{
    MessagePackSink(w)
}

impl<R> value::Source for MessagePackSource<R>
    where R: io::Read
{
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        use rmpv::decode::value::Error;

        match rmpv::decode::value::read_value(&mut self.0) {
            Ok(v) => Ok(Some(value_from_message_pack(v))),
            Err(Error::InvalidMarkerRead(ref e)) if e.kind() == io::ErrorKind::UnexpectedEof => {
                Ok(None)
            },
            Err(e) => Err(error::ErrorKind::MessagePackDecode(e).into()),
        }
    }
}

impl<W> value::Sink for MessagePackSink<W>
    where W: io::Write
{
    #[inline]
    fn write(&mut self, v: value::Value) -> error::Result<()> {
        rmpv::encode::write_value(&mut self.0, &value_to_message_pack(v)).map_err(From::from)
    }
}

fn value_from_message_pack(value: rmpv::Value) -> value::Value {
    use rmpv::Value;
    match value {
        Value::Nil => value::Value::Unit,
        Value::Boolean(v) => value::Value::Bool(v),
        Value::U64(v) => value::Value::U64(v),
        Value::I64(v) => value::Value::I64(v),
        Value::F32(v) => value::Value::from_f32(v),
        Value::F64(v) => value::Value::from_f64(v),
        Value::String(v) => value::Value::String(v),
        Value::Binary(v) => value::Value::Bytes(v),
        Value::Array(v) => {
            value::Value::Sequence(v.into_iter().map(value_from_message_pack).collect())
        },
        Value::Map(v) => {
            value::Value::Map(v.into_iter()
                .map(|(k, v)| (value_from_message_pack(k), value_from_message_pack(v)))
                .collect())
        },
        Value::Ext(_, v) => value::Value::Bytes(v),
    }
}

fn value_to_message_pack(value: value::Value) -> rmpv::Value {
    use rmpv::Value;
    match value {
        value::Value::Unit => Value::Nil,
        value::Value::Bool(v) => Value::Boolean(v),

        value::Value::I8(v) => Value::I64(v as i64),
        value::Value::I16(v) => Value::I64(v as i64),
        value::Value::I32(v) => Value::I64(v as i64),
        value::Value::I64(v) => Value::I64(v),

        value::Value::U8(v) => Value::U64(v as u64),
        value::Value::U16(v) => Value::U64(v as u64),
        value::Value::U32(v) => Value::U64(v as u64),
        value::Value::U64(v) => Value::U64(v),

        value::Value::F32(ordered_float::OrderedFloat(v)) => Value::F32(v),
        value::Value::F64(ordered_float::OrderedFloat(v)) => Value::F64(v),

        value::Value::Char(v) => Value::String(format!("{}", v)),
        value::Value::String(v) => Value::String(v),
        value::Value::Bytes(v) => Value::Binary(v),

        value::Value::Sequence(v) => {
            Value::Array(v.into_iter().map(value_to_message_pack).collect())
        },
        value::Value::Map(v) => {
            Value::Map(v.into_iter()
                .map(|(k, v)| (value_to_message_pack(k), value_to_message_pack(v)))
                .collect())
        },
    }
}
