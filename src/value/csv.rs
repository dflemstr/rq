use csv;
use crate::error;
use ordered_float;
use std::fmt;
use std::io;
use crate::value;

pub struct CsvSource<R>(csv::StringRecordsIntoIter<R>)
where
    R: io::Read;

pub struct CsvSink<W>(csv::Writer<W>)
where
    W: io::Write;

#[inline]
pub fn source<R>(r: R) -> CsvSource<R>
where
    R: io::Read,
{
    CsvSource(
        csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(r)
            .into_records(),
    )
}

#[inline]
pub fn sink<W>(w: W) -> CsvSink<W>
where
    W: io::Write,
{
    CsvSink(csv::Writer::from_writer(w))
}

impl<R> value::Source for CsvSource<R>
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

impl<W> value::Sink for CsvSink<W>
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
        value::Value::Bool(v) => Ok(format!("{}", v)),

        value::Value::I8(v) => Ok(format!("{}", v)),
        value::Value::I16(v) => Ok(format!("{}", v)),
        value::Value::I32(v) => Ok(format!("{}", v)),
        value::Value::I64(v) => Ok(format!("{}", v)),

        value::Value::U8(v) => Ok(format!("{}", v)),
        value::Value::U16(v) => Ok(format!("{}", v)),
        value::Value::U32(v) => Ok(format!("{}", v)),
        value::Value::U64(v) => Ok(format!("{}", v)),

        value::Value::F32(ordered_float::OrderedFloat(v)) => Ok(format!("{}", v)),
        value::Value::F64(ordered_float::OrderedFloat(v)) => Ok(format!("{}", v)),

        value::Value::Char(v) => Ok(format!("{}", v)),
        value::Value::String(v) => Ok(format!("{}", v)),
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

impl<R> fmt::Debug for CsvSource<R>
where
    R: io::Read,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CsvSource").finish()
    }
}

impl<W> fmt::Debug for CsvSink<W>
where
    W: io::Write,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CsvSink").finish()
    }
}
