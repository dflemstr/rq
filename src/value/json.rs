

use ansi_term;
use dtoa;

use error;
use itoa;
use serde;
use serde_json;
use std::io;
use std::str;
use value;

pub struct JsonSource<R>(serde_json::StreamDeserializer<value::Value, io::Bytes<R>>)
    where R: io::Read;

pub struct JsonSink<W, F>(W, F)
    where W: io::Write,
          F: Clone + serde_json::ser::Formatter;

#[derive(Clone, Debug)]
pub struct ReadableFormatter {
    current_indent: usize,
    is_in_object_key: bool,
    has_value: bool,

    null_style: ansi_term::Style,

    true_style: ansi_term::Style,
    false_style: ansi_term::Style,

    number_style: ansi_term::Style,

    string_quote_style: ansi_term::Style,
    string_char_style: ansi_term::Style,
    string_escape_style: ansi_term::Style,

    array_bracket_style: ansi_term::Style,
    array_comma_style: ansi_term::Style,

    object_brace_style: ansi_term::Style,
    object_colon_style: ansi_term::Style,
    object_comma_style: ansi_term::Style,
    object_key_quote_style: ansi_term::Style,
    object_key_char_style: ansi_term::Style,
    object_key_escape_style: ansi_term::Style,
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
          F: Clone + serde_json::ser::Formatter
{
    #[inline]
    fn write(&mut self, v: value::Value) -> error::Result<()> {
        {
            let mut serializer = serde_json::ser::Serializer::with_formatter(&mut self.0,
                                                                             self.1.clone());
            try!(serde::Serialize::serialize(&v, &mut serializer));
        }
        try!(self.0.write_all(b"\n"));
        Ok(())
    }
}

impl ReadableFormatter {
    fn new() -> ReadableFormatter {
        use ansi_term::{Colour, Style};

        ReadableFormatter {
            current_indent: 0,
            is_in_object_key: false,
            has_value: false,

            null_style: Colour::Black.dimmed().bold().italic(),

            true_style: Colour::Green.bold().italic(),
            false_style: Colour::Red.bold().italic(),

            number_style: Colour::Blue.normal(),

            string_quote_style: Colour::Green.dimmed(),
            string_char_style: Colour::Green.normal(),
            string_escape_style: Colour::Green.dimmed(),

            array_bracket_style: Style::default().bold(),
            array_comma_style: Style::default().bold(),

            object_brace_style: Style::default().bold(),
            object_colon_style: Style::default().bold(),
            object_comma_style: Style::default().bold(),
            object_key_quote_style: Colour::Blue.dimmed(),
            object_key_char_style: Colour::Blue.normal(),
            object_key_escape_style: Colour::Blue.dimmed(),
        }
    }
}

impl serde_json::ser::Formatter for ReadableFormatter {
    /// Writes a `null` value to the specified writer.
    #[inline]
    fn write_null<W>(&mut self, writer: &mut W) -> serde_json::Result<()>
        where W: io::Write
    {
        write!(writer, "{}", self.null_style.paint("null")).map_err(From::from)
    }

    /// Writes a `true` or `false` value to the specified writer.
    #[inline]
    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> serde_json::Result<()>
        where W: io::Write
    {
        let s = if value {
            self.true_style.paint("true")
        } else {
            self.false_style.paint("false")
        };
        write!(writer, "{}", s).map_err(From::from)
    }

    /// Writes an integer value like `-123` to the specified writer.
    #[inline]
    fn write_integer<W, I>(&mut self, writer: &mut W, value: I) -> serde_json::Result<()>
        where W: io::Write,
              I: itoa::Integer
    {
        try!(write!(writer, "{}", self.number_style.prefix()));
        try!(itoa::write(writer, value));
        try!(write!(writer, "{}", self.number_style.suffix()));
        Ok(())
    }

    /// Writes a floating point value like `-31.26e+12` to the
    /// specified writer.
    #[inline]
    fn write_floating<W, F>(&mut self, writer: &mut W, value: F) -> serde_json::Result<()>
        where W: io::Write,
              F: dtoa::Floating
    {
        try!(write!(writer, "{}", self.number_style.prefix()));
        try!(dtoa::write(writer, value));
        try!(write!(writer, "{}", self.number_style.suffix()));
        Ok(())
    }

    /// Called before each series of `write_string_fragment` and
    /// `write_char_escape`.  Writes a `"` to the specified writer.
    #[inline]
    fn begin_string<W>(&mut self, writer: &mut W) -> serde_json::Result<()>
        where W: io::Write
    {
        let style = if self.is_in_object_key {
            self.object_key_quote_style
        } else {
            self.string_quote_style
        };

        write!(writer, "{}", style.paint("\"")).map_err(From::from)
    }

    /// Called after each series of `write_string_fragment` and
    /// `write_char_escape`.  Writes a `"` to the specified writer.
    #[inline]
    fn end_string<W>(&mut self, writer: &mut W) -> serde_json::Result<()>
        where W: io::Write
    {
        let style = if self.is_in_object_key {
            self.object_key_quote_style
        } else {
            self.string_quote_style
        };

        write!(writer, "{}", style.paint("\"")).map_err(From::from)
    }

    /// Writes a string fragment that doesn't need any escaping to the
    /// specified writer.
    #[inline]
    fn write_string_fragment<W>(&mut self,
                                writer: &mut W,
                                fragment: &[u8])
                                -> serde_json::Result<()>
        where W: io::Write
    {
        let style = if self.is_in_object_key {
            self.object_key_char_style
        } else {
            self.string_char_style
        };

        let s = unsafe { str::from_utf8_unchecked(fragment) };
        write!(writer, "{}", style.paint(s)).map_err(From::from)
    }

    /// Writes a character escape code to the specified writer.
    #[inline]
    fn write_char_escape<W>(&mut self,
                            writer: &mut W,
                            char_escape: serde_json::ser::CharEscape)
                            -> serde_json::Result<()>
        where W: io::Write
    {
        use serde_json::ser::CharEscape::*;

        let style = if self.is_in_object_key {
            self.object_key_escape_style
        } else {
            self.string_escape_style
        };

        let s = match char_escape {
            Quote => "\\\"",
            ReverseSolidus => "\\\\",
            Solidus => "\\/",
            Backspace => "\\b",
            FormFeed => "\\f",
            LineFeed => "\\n",
            CarriageReturn => "\\r",
            Tab => "\\t",
            AsciiControl(byte) => {
                static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
                let bytes = &[b'\\',
                              b'u',
                              b'0',
                              b'0',
                              HEX_DIGITS[(byte >> 4) as usize],
                              HEX_DIGITS[(byte & 0xF) as usize]];
                let s = unsafe { str::from_utf8_unchecked(bytes) };

                // Need to return early because of allocated String
                return write!(writer, "{}", style.paint(s)).map_err(From::from);
            },
        };

        write!(writer, "{}", style.paint(s)).map_err(From::from)
    }

    /// Called before every array.  Writes a `[` to the specified
    /// writer.
    #[inline]
    fn begin_array<W>(&mut self, writer: &mut W) -> serde_json::Result<()>
        where W: io::Write
    {
        self.current_indent += 1;
        self.has_value = false;

        write!(writer, "{}", self.array_bracket_style.paint("[")).map_err(From::from)
    }

    /// Called after every array.  Writes a `]` to the specified
    /// writer.
    #[inline]
    fn end_array<W>(&mut self, writer: &mut W) -> serde_json::Result<()>
        where W: io::Write
    {
        self.current_indent -= 1;

        if self.has_value {
            try!(write!(writer, "\n"));
            try!(indent(writer, self.current_indent));
        }

        write!(writer, "{}", self.array_bracket_style.paint("]")).map_err(From::from)
    }

    /// Called before every array value.  Writes a `,` if needed to
    /// the specified writer.
    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> serde_json::Result<()>
        where W: io::Write
    {
        if !first {
            try!(write!(writer, "{}", self.array_comma_style.paint(",")));
        }

        try!(write!(writer, "\n"));
        try!(indent(writer, self.current_indent));
        Ok(())
    }

    /// Called after every array value.
    #[inline]
    fn end_array_value<W>(&mut self, _writer: &mut W) -> serde_json::Result<()>
        where W: io::Write
    {
        self.has_value = true;
        Ok(())
    }

    /// Called before every object.  Writes a `{` to the specified
    /// writer.
    #[inline]
    fn begin_object<W>(&mut self, writer: &mut W) -> serde_json::Result<()>
        where W: io::Write
    {
        self.current_indent += 1;
        self.has_value = false;

        write!(writer, "{}", self.object_brace_style.paint("{")).map_err(From::from)
    }

    /// Called after every object.  Writes a `}` to the specified
    /// writer.
    #[inline]
    fn end_object<W>(&mut self, writer: &mut W) -> serde_json::Result<()>
        where W: io::Write
    {
        self.current_indent -= 1;

        if self.has_value {
            try!(write!(writer, "\n"));
            try!(indent(writer, self.current_indent));
        }

        write!(writer, "{}", self.object_brace_style.paint("}")).map_err(From::from)
    }

    /// Called before every object key.
    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> serde_json::Result<()>
        where W: io::Write
    {
        self.is_in_object_key = true;

        if !first {
            try!(write!(writer, "{}", self.object_comma_style.paint(",")));
        }

        try!(write!(writer, "\n"));
        try!(indent(writer, self.current_indent));
        Ok(())
    }

    /// Called after every object key.  A `:` should be written to the
    /// specified writer by either this method or
    /// `begin_object_value`.
    #[inline]
    fn end_object_key<W>(&mut self, _writer: &mut W) -> serde_json::Result<()>
        where W: io::Write
    {
        self.is_in_object_key = false;
        Ok(())
    }

    /// Called before every object value.  A `:` should be written to
    /// the specified writer by either this method or
    /// `end_object_key`.
    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> serde_json::Result<()>
        where W: io::Write
    {
        write!(writer, "{}", self.object_colon_style.paint(": ")).map_err(From::from)
    }

    /// Called after every object value.
    #[inline]
    fn end_object_value<W>(&mut self, _writer: &mut W) -> serde_json::Result<()>
        where W: io::Write
    {
        self.has_value = true;
        Ok(())
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
