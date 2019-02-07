use std::io;
use std::mem;
use std::vec;

use serde;
use serde_hjson;

use error;
use value;

pub struct HjsonSource(serde_hjson::StreamDeserializer<value::Value, vec::IntoIter<u8>>);

pub struct HjsonSink<W>(Option<serde_hjson::Serializer<W, Formatter>>)
where
    W: io::Write;

struct Formatter {
    current_indent: usize,
    current_is_array: bool,
    stack: Vec<bool>,
    at_colon: bool,
    braces_same_line: bool,
}

#[inline]
pub fn source<R>(r: R) -> error::Result<HjsonSource>
where
    R: io::Read,
{
    let bytes = r.bytes().collect::<io::Result<Vec<u8>>>()?;
    Ok(HjsonSource(serde_hjson::StreamDeserializer::new(
        bytes.into_iter(),
    )))
}

#[inline]
pub fn sink<W>(w: W) -> HjsonSink<W>
where
    W: io::Write,
{
    HjsonSink(Some(serde_hjson::Serializer::with_formatter(
        w,
        Formatter::new(),
    )))
}

impl value::Source for HjsonSource {
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match self.0.next() {
            Some(Ok(v)) => Ok(Some(v)),
            Some(Err(e)) => Err(error::Error::from(e)),
            None => Ok(None),
        }
    }
}

impl<W> value::Sink for HjsonSink<W>
where
    W: io::Write,
{
    #[inline]
    fn write(&mut self, v: value::Value) -> error::Result<()> {
        if let Some(ref mut w) = self.0 {
            serde::Serialize::serialize(&v, w)?;
        }

        // Some juggling required here to get the underlying writer temporarily, to write a newline.
        let mut w = mem::replace(&mut self.0, None).unwrap().into_inner();
        let result = w.write_all(&[10]);
        mem::replace(
            &mut self.0,
            Some(serde_hjson::Serializer::with_formatter(w, Formatter::new())),
        );

        result.map_err(From::from)
    }
}

impl Formatter {
    fn new() -> Self {
        Formatter {
            current_indent: 0,
            current_is_array: false,
            stack: Vec::new(),
            at_colon: false,
            braces_same_line: false,
        }
    }
}

impl serde_hjson::ser::Formatter for Formatter {
    fn open<W>(&mut self, writer: &mut W, ch: u8) -> serde_hjson::Result<()>
    where
        W: io::Write,
    {
        if self.current_indent > 0 && !self.current_is_array && !self.braces_same_line {
            self.newline(writer, 0)?;
        } else {
            self.start_value(writer)?;
        }
        self.current_indent += 1;
        self.stack.push(self.current_is_array);
        self.current_is_array = ch == b'[';
        writer.write_all(&[ch]).map_err(From::from)
    }

    fn comma<W>(&mut self, writer: &mut W, _: bool) -> serde_hjson::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(b"\n")?;
        indent(writer, self.current_indent)
    }

    fn colon<W>(&mut self, writer: &mut W) -> serde_hjson::Result<()>
    where
        W: io::Write,
    {
        self.at_colon = !self.braces_same_line;
        writer
            .write_all(if self.braces_same_line { b": " } else { b":" })
            .map_err(From::from)
    }

    fn close<W>(&mut self, writer: &mut W, ch: u8) -> serde_hjson::Result<()>
    where
        W: io::Write,
    {
        self.current_indent -= 1;
        self.current_is_array = self.stack.pop().unwrap();
        writer.write(b"\n")?;
        indent(writer, self.current_indent)?;
        writer.write_all(&[ch]).map_err(From::from)
    }

    fn newline<W>(&mut self, writer: &mut W, add_indent: i32) -> serde_hjson::Result<()>
    where
        W: io::Write,
    {
        self.at_colon = false;
        writer.write_all(b"\n")?;
        let ii = self.current_indent as i32 + add_indent;
        indent(writer, if ii < 0 { 0 } else { ii as usize })
    }

    fn start_value<W>(&mut self, writer: &mut W) -> serde_hjson::Result<()>
    where
        W: io::Write,
    {
        if self.at_colon {
            self.at_colon = false;
            writer.write_all(b" ")?
        }
        Ok(())
    }
}

fn indent<W>(wr: &mut W, n: usize) -> serde_hjson::Result<()>
where
    W: io::Write,
{
    for _ in 0..n {
        wr.write_all(b"  ")?;
    }

    Ok(())
}
