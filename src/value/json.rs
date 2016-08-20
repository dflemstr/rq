use std::char;
use std::io;

use ansi_term;
use serde;
use serde_json;

use error;
use value;

pub struct JsonSource<R>(serde_json::StreamDeserializer<value::Value, io::Bytes<R>>)
    where R: io::Read;

pub struct JsonSink<W, F>(W, F)
    where W: io::Write,
          F: FormatterClone + serde_json::ser::Formatter;

// TODO: this is needed until https://github.com/serde-rs/json/pull/139 is merged
pub trait FormatterClone {
    fn clone_formatter(&self) -> Self;
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Context {
  Object,
  Array,
}

#[derive(Clone, Debug)]
pub struct ReadableFormatter {
    current_indent: usize,
    context: Context,

    bracket_style: ansi_term::Style,
    colon_style: ansi_term::Style,
    comma_style: ansi_term::Style,

    field_style: ansi_term::Style,
    value_style: ansi_term::Style,
}

#[inline]
pub fn source<R>(r: R) -> JsonSource<R>
    where R: io::Read
{
    JsonSource(serde_json::StreamDeserializer::new(r.bytes()))
}

#[inline]
pub fn sink_compact<W>(w: W) -> JsonSink<W, serde_json::ser::CompactFormatter>
    where W: io::Write
{
    JsonSink(w, serde_json::ser::CompactFormatter)
}

#[inline]
pub fn sink_readable<W>(w: W) -> JsonSink<W, ReadableFormatter>
    where W: io::Write
{
    JsonSink(w, ReadableFormatter::new())
}

impl<R> value::Source for JsonSource<R>
    where R: io::Read
{
    #[inline]
    fn read(&mut self) -> error::Result<Option<value::Value>> {
        match self.0.next() {
            Some(Ok(v)) => Ok(Some(v)),
            Some(Err(e)) => Err(error::Error::from(e)),
            None => Ok(None),
        }
    }
}

impl<W, F> value::Sink for JsonSink<W, F>
    where W: io::Write,
          F: FormatterClone + serde_json::ser::Formatter
{
    #[inline]
    fn write(&mut self, v: value::Value) -> error::Result<()> {
        {
            let mut serializer =
                serde_json::ser::Serializer::with_formatter(&mut self.0, self.1.clone_formatter());
            try!(serde::Serialize::serialize(&v, &mut serializer));
        }
        try!(self.0.write_all(b"\n"));
        Ok(())
    }
}

impl FormatterClone for serde_json::ser::CompactFormatter {
    fn clone_formatter(&self) -> Self {
        serde_json::ser::CompactFormatter
    }
}

impl ReadableFormatter {
    fn new() -> ReadableFormatter {
        ReadableFormatter {
            current_indent: 0,
            context: Context::Array,

            bracket_style: ansi_term::Colour::White.bold(),
            colon_style: ansi_term::Colour::White.bold(),
            comma_style: ansi_term::Colour::White.bold(),

            field_style: ansi_term::Colour::Blue.bold(),
            value_style: ansi_term::Colour::Green.normal(),
        }
    }
}

impl serde_json::ser::Formatter for ReadableFormatter {
    fn open<W>(&mut self, writer: &mut W, ch: u8) -> serde_json::error::Result<()>
        where W: io::Write
    {
        self.current_indent += 1;
        // TODO: optimize this maybe?
        let char_str = format!("{}", ch as char);
        try!(write!(writer, "{}", self.bracket_style.paint(char_str)));

        self.context = match ch as char {
            '{' =>  Context::Object,
            '[' => Context::Array,
            _ => Context::Array,
        };

        Ok(())
    }

    fn comma<W>(&mut self, writer: &mut W, first: bool) -> serde_json::error::Result<()>
        where W: io::Write
    {
        if !first {
            try!(write!(writer, "{}", self.comma_style.paint(",")));
        }
        try!(writer.write_all(b"\n"));

        try!(indent(writer, self.current_indent));

        match self.context {
            Context::Object => try!(write!(writer, "{}", self.field_style.prefix())),
            Context::Array => try!(write!(writer, "{}", self.value_style.prefix())),
        }

        Ok(())
    }

    fn colon<W>(&mut self, writer: &mut W) -> serde_json::error::Result<()>
        where W: io::Write
    {
        try!(write!(writer, "{}", self.field_style.suffix()));
        try!(write!(writer, "{}", self.colon_style.paint(":")));
        try!(write!(writer, " "));

        try!(write!(writer, "{}", self.value_style.prefix()));

        Ok(())
    }

    fn close<W>(&mut self, writer: &mut W, ch: u8) -> serde_json::error::Result<()>
        where W: io::Write
    {
        self.current_indent -= 1;

        match self.context {
            Context::Object => try!(write!(writer, "{}", self.field_style.suffix())),
            Context::Array => try!(write!(writer, "{}", self.value_style.suffix())),
        }

        try!(writer.write(b"\n"));
        try!(indent(writer, self.current_indent));

        // TODO: optimize this maybe?
        let char_str = format!("{}", ch as char);
        try!(write!(writer, "{}", self.bracket_style.paint(char_str)));

        Ok(())
    }
}

impl FormatterClone for ReadableFormatter {
    fn clone_formatter(&self) -> Self {
        self.clone()
    }
}

fn indent<W>(wr: &mut W, n: usize) -> serde_json::error::Result<()>
    where W: io::Write
{
    for _ in 0..n {
        try!(wr.write_all(b"  "));
    }

    Ok(())
}
