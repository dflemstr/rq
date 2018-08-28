use error;
use std::io;
use value;

pub struct RawSource<R>(io::Lines<io::BufReader<R>>) where R: io::Read;

pub struct RawSink<W>(io::LineWriter<W>) where W: io::Write;

#[inline]
pub fn source<R>(r: R) -> RawSource<R> where R: io::Read
{
    use std::io::BufRead;
    RawSource(io::BufReader::new(r).lines())
}

#[inline]
pub fn sink<W>(w: W) -> RawSink<W> where W: io::Write
{
    RawSink(io::LineWriter::new(w))
}

impl<R> value::Source for RawSource<R> where R: io::Read
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

impl<W> value::Sink for RawSink<W> where W: io::Write
{
    #[inline]
    fn write(&mut self, value: value::Value) -> error::Result<()> {
        use std::io::Write;
        match value {
            value::Value::String(s) => {
                self.0.write(s.as_bytes())?;
                self.0.write(b"\n")?;
                Ok(())
            }
            value::Value::Bytes(b) => {
                self.0.write(&b)?;
                self.0.write(b"\n")?;
                Ok(())
            }
            x => bail!("raw can only output strings and bytes, got: {:?}", x)
        }
    }
}
