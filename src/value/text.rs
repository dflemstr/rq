use error;
use std::io;
use value;

pub struct TextSource<R>(io::Lines<io::BufReader<R>>) where R: io::Read;

pub struct TextSink<W>(io::LineWriter<W>) where W: io::Write;

#[inline]
pub fn source<R>(r: R) -> TextSource<R> where R: io::Read
{
    use std::io::BufRead;
    TextSource(io::BufReader::new(r).lines())
}

#[inline]
pub fn sink<W>(w: W) -> TextSink<W> where W: io::Write
{
    TextSink(io::LineWriter::new(w))
}

impl<R> value::Source for TextSource<R> where R: io::Read
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

impl<W> value::Sink for TextSink<W> where W: io::Write
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
            _ => bail!("text can only output strings")
        }
    }
}
