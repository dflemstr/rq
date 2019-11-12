use crate::error;
use crate::value;
use std::io;

#[derive(Debug)]
pub struct Source<R>(io::Lines<io::BufReader<R>>)
where
    R: io::Read;

#[derive(Debug)]
pub struct Sink<W>(io::LineWriter<W>)
where
    W: io::Write;

#[inline]
pub fn source<R>(r: R) -> Source<R>
where
    R: io::Read,
{
    use std::io::BufRead;
    Source(io::BufReader::new(r).lines())
}

#[inline]
pub fn sink<W>(w: W) -> Sink<W>
where
    W: io::Write,
{
    Sink(io::LineWriter::new(w))
}

impl<R> value::Source for Source<R>
where
    R: io::Read,
{
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match self.0.next() {
            Some(Ok(v)) => Ok(Some(value::Value::String(v))),
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
        use std::io::Write;
        match value {
            value::Value::String(s) => {
                self.0.write_all(s.as_bytes())?;
                self.0.write_all(b"\n")?;
                Ok(())
            }
            value::Value::Bytes(b) => {
                self.0.write_all(&b)?;
                self.0.write_all(b"\n")?;
                Ok(())
            }
            value::Value::Char(c) => {
                writeln!(self.0, "{}", c)?;
                Ok(())
            }
            x => Err(error::Error::Format {
                msg: format!("raw can only output strings, bytes and chars, got: {:?}", x),
            }),
        }
    }
}
