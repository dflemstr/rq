use crate::error;
use crate::value;
use csv;
use ordered_float;
use std::fmt;
use std::io;

pub struct Source<R>(csv::StringRecordsIntoIter<R>)
where
    R: io::Read;

pub struct Sink<W>(csv::Writer<W>)
where
    W: io::Write;

#[inline]
pub fn source<R>(r: R) -> Source<R>
where
    R: io::Read,
{
    Source(
        csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(r)
            .into_records(),
    )
}

#[inline]
pub fn sink<W>(w: W) -> Sink<W>
where
    W: io::Write,
{
    Sink(csv::Writer::from_writer(w))
}

impl<R> value::Source for Source<R>
where
    R: io::Read,
{
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match self.0.next() {
            Some(Ok(v)) => Ok(Some(value::Value::Sequence(
                v.iter()
                    .map(|s| value::Value::String(s.to_string()))
                    .collect(),
            ))),
            Some(Err(e)) => Err(error::Error::from(e)),
            None => Ok(None),
        }
    }
}

impl<W> value::Sink for Sink<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, value: value::Value) -> error::Result<()> {
        match value {
            value::Value::Sequence(seq) => {
                let record: Vec<String> = seq
                    .into_iter()
                    .map(value_to_csv)
                    .collect::<error::Result<Vec<_>>>()?;
                self.0.write_record(record)?;
                Ok(())
            }
            x => Err(error::Error::Format {
                msg: format!("csv can only output sequences, got: {:?}", x),
            }),
        }
    }
}

fn value_to_csv(value: value::Value) -> error::Result<String> {
    match value {
        value::Value::Unit => Err(error::Error::Format {
            msg: "csv cannot output nested Unit".to_owned(),
        }),
        value::Value::Bool(v) => Ok(v.to_string()),

        value::Value::I8(v) => Ok(v.to_string()),
        value::Value::I16(v) => Ok(v.to_string()),
        value::Value::I32(v) => Ok(v.to_string()),
        value::Value::I64(v) => Ok(v.to_string()),

        value::Value::U8(v) => Ok(v.to_string()),
        value::Value::U16(v) => Ok(v.to_string()),
        value::Value::U32(v) => Ok(v.to_string()),
        value::Value::U64(v) => Ok(v.to_string()),

        value::Value::F32(ordered_float::OrderedFloat(v)) => Ok(v.to_string()),
        value::Value::F64(ordered_float::OrderedFloat(v)) => Ok(v.to_string()),

        value::Value::Char(v) => Ok(v.to_string()),
        value::Value::String(v) => Ok(v.to_string()),
        value::Value::Bytes(_) => Err(error::Error::Format {
            msg: "csv cannot output nested bytes".to_owned(),
        }),

        value::Value::Sequence(_) => Err(error::Error::Format {
            msg: "csv cannot output nested sequences".to_owned(),
        }),
        value::Value::Map(_) => Err(error::Error::Format {
            msg: "csv cannot output nested maps".to_owned(),
        }),
    }
}

impl<R> fmt::Debug for Source<R>
where
    R: io::Read,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CsvSource").finish()
    }
}

impl<W> fmt::Debug for Sink<W>
where
    W: io::Write,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CsvSink").finish()
    }
}
