use byteorder;
use error::{self, Error, ErrorKind};
use flate2;
use header;
use schema;
use serde;
use serde_json;
use snap;
use std::borrow;
use std::io;

pub struct Deserializer<'a, R>
    where R: io::BufRead
{
    input: R,
    registry: borrow::Cow<'a, schema::SchemaRegistry>,
    schema: borrow::Cow<'a, schema::Schema>,
}
struct DeserializerImpl<'a, R>
    where R: io::BufRead + 'a
{
    input: &'a mut R,
    registry: &'a schema::SchemaRegistry,
    schema: &'a schema::Schema,
}

enum Codec {
    Null,
    Deflate,
    Snappy,
}

pub struct Blocks<R>
    where R: io::Read
{
    input: R,
    codec: Codec,
    sync_marker: Vec<u8>,
    current_block: io::Cursor<Vec<u8>>,
}

struct RecordVisitor<'a, R>
    where R: io::BufRead + 'a
{
    input: &'a mut R,
    registry: &'a schema::SchemaRegistry,
    fields: schema::RecordFields<'a>,
    field: Option<&'a schema::FieldSchema>,
}

struct FieldNameDeserializer<'a>(&'a str);

enum BlockRemainder {
    Start,
    Count(usize),
    End,
}

struct ArrayVisitor<'a, R>
    where R: io::BufRead + 'a
{
    input: &'a mut R,
    registry: &'a schema::SchemaRegistry,
    elem_schema: &'a schema::Schema,
    remainder: BlockRemainder,
}

struct MapVisitor<'a, R>
    where R: io::BufRead + 'a
{
    input: &'a mut R,
    registry: &'a schema::SchemaRegistry,
    value_schema: &'a schema::Schema,
    remainder: BlockRemainder,
}

impl<'a, R> Deserializer<'a, R>
    where R: io::BufRead
{
    pub fn new(input: R,
               registry: &'a schema::SchemaRegistry,
               schema: &'a schema::Schema)
               -> Deserializer<'a, R> {
        Deserializer::new_cow(input,
                              borrow::Cow::Borrowed(registry),
                              borrow::Cow::Borrowed(schema))
    }

    fn new_cow(input: R,
               registry: borrow::Cow<'a, schema::SchemaRegistry>,
               schema: borrow::Cow<'a, schema::Schema>)
               -> Deserializer<'a, R> {
        Deserializer {
            input: input,
            registry: registry,
            schema: schema,
        }
    }
}

impl<'a, R> Deserializer<'a, io::BufReader<Blocks<R>>>
    where R: io::Read
{
    // TODO: this uses a ridiculous number of buffers... We can cut that down significantly
    pub fn from_container(input: R) -> error::Result<Deserializer<'static, io::BufReader<Blocks<io::BufReader<R>>>>> {
        use serde::de::Deserialize;

        let mut input = io::BufReader::new(input);

        let header = {
            debug!("Parsing container header");
            let mut header_de =
                Deserializer::new(&mut input, &schema::EMPTY_REGISTRY, &schema::FILE_HEADER);
            try!(header::Header::deserialize(&mut header_de))
        };
        debug!("Container header: {:?}", header);

        if &[b'O', b'b', b'j', 1] != &*header.magic {
            Err(ErrorKind::BadFileMagic(header.magic.to_vec()).into())
        } else {
            let codec = try!(Codec::parse(header.meta.get("avro.codec").map(AsRef::as_ref)));
            let schema_data = try!(header.meta
                .get("avro.schema")
                .ok_or(Error::from(ErrorKind::NoSchema)));

            let schema_json = try!(serde_json::from_slice(&schema_data));
            let mut registry = schema::SchemaRegistry::new();

            let root_schema = try!(try!(registry.add_json(&schema_json))
                    .ok_or(Error::from(ErrorKind::NoRootType)))
                .into_resolved(&registry);

            let blocks = Blocks::new(input, codec, header.sync.to_vec());
            let registry_cow = borrow::Cow::Owned(registry);
            let schema_cow = borrow::Cow::Owned(root_schema);
            Ok(Deserializer::new_cow(io::BufReader::new(blocks), registry_cow, schema_cow))
        }
    }
}

impl<'a, R> serde::Deserializer for Deserializer<'a, R>
    where R: io::BufRead
{
    type Error = error::Error;

    forward_to_deserialize! {
        deserialize_bool,
        deserialize_f64, deserialize_f32,
        deserialize_u8, deserialize_u16, deserialize_u32, deserialize_u64, deserialize_usize,
        deserialize_i8, deserialize_i16, deserialize_i32, deserialize_i64, deserialize_isize,
        deserialize_char, deserialize_str, deserialize_string,
        deserialize_ignored_any,
        deserialize_bytes,
        deserialize_unit_struct, deserialize_unit,
        deserialize_seq, deserialize_seq_fixed_size,
        deserialize_map, deserialize_newtype_struct, deserialize_struct_field,
        deserialize_tuple,
        deserialize_enum,
        deserialize_struct, deserialize_tuple_struct,
        deserialize_option
    }

    #[inline]
    fn deserialize<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor
    {
        DeserializerImpl::new(&mut self.input, &*self.registry, &*self.schema).deserialize(visitor)
    }
}

impl<'a, R> DeserializerImpl<'a, R>
    where R: io::BufRead
{
    pub fn new(input: &'a mut R,
               registry: &'a schema::SchemaRegistry,
               schema: &'a schema::Schema)
               -> DeserializerImpl<'a, R> {
        DeserializerImpl {
            input: input,
            registry: registry,
            schema: schema,
        }
    }

    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, error::Error>
        where V: serde::de::Visitor
    {
        use schema::Schema;
        use byteorder::ReadBytesExt;

        if try!(self.input.fill_buf()).is_empty() {
            return Err(serde::de::Error::end_of_stream());
        }

        match *self.schema {
            Schema::Null => {
                debug!("Deserializing null");
                visitor.visit_unit()
            },
            Schema::Boolean => {
                let v = try!(self.input.read_u8());
                debug!("Deserializing boolean {:?}", v);
                // TODO: if v is not in [0, 1], report error
                visitor.visit_bool(v != 0)
            },
            Schema::Int => {
                let v = try!(read_int(self.input));
                debug!("Deserializing int {:?}", v);
                visitor.visit_i32(v)
            },
            Schema::Long => {
                let v = try!(read_long(self.input));
                debug!("Deserializing long {:?}", v);
                visitor.visit_i64(v)
            },
            Schema::Float => {
                let v = try!(self.input.read_f32::<byteorder::LittleEndian>());
                debug!("Deserializing float {:?}", v);
                visitor.visit_f32(v)
            },
            Schema::Double => {
                let v = try!(self.input.read_f64::<byteorder::LittleEndian>());
                debug!("Deserializing double {:?}", v);
                visitor.visit_f64(v)
            },
            Schema::Bytes => {
                let len = try!(read_long(self.input));

                if len < 0 {
                    Err(ErrorKind::NegativeLength.into())
                } else {
                    let mut result = vec![0; len as usize];
                    try!(self.input.read_exact(&mut result));
                    debug!("Deserializing bytes {:?}", result);
                    visitor.visit_byte_buf(result)
                }
            },
            Schema::String => {
                let len = try!(read_long(self.input));

                if len < 0 {
                    Err(ErrorKind::NegativeLength.into())
                } else {
                    let mut buffer = vec![0; len as usize];
                    try!(self.input.read_exact(&mut buffer));
                    let result = try!(String::from_utf8(buffer));
                    debug!("Deserializing string {:?}", result);
                    visitor.visit_string(result)
                }
            },
            Schema::Record(ref inner) => {
                debug!("Deserializing record of type {:?}", inner.name());
                let fields = inner.fields();
                visitor.visit_map(RecordVisitor::new(self.input, &*self.registry, fields))
            },
            Schema::Enum(ref inner) => {
                debug!("Deserializing enum of type {:?}", inner.name());
                let v = try!(read_int(self.input));
                visitor.visit_str(inner.symbols()[v as usize].as_str())
            },
            Schema::Array(ref inner) => {
                debug!("Deserializing array");
                let elem_schema = inner.resolve(&self.registry);
                visitor.visit_seq(ArrayVisitor::new(self.input, &*self.registry, elem_schema))
            },
            Schema::Map(ref inner) => {
                debug!("Deserializing map");
                let value_schema = inner.resolve(&self.registry);
                visitor.visit_map(MapVisitor::new(self.input, &*self.registry, value_schema))
            },
            Schema::Union(ref inner) => {
                debug!("Deserializing union");
                let variant = try!(read_long(self.input));
                let schema = inner[variant as usize].resolve(&self.registry);
                DeserializerImpl::new(self.input, self.registry, &schema).deserialize(visitor)
            },
            Schema::Fixed(ref inner) => {
                debug!("Deserializing fixed of size {}", inner.size());
                let mut buffer = vec![0; inner.size() as usize];
                try!(self.input.read_exact(&mut buffer));
                visitor.visit_byte_buf(buffer)
            },
        }
    }
}


impl<'a, R> serde::Deserializer for DeserializerImpl<'a, R>
    where R: io::BufRead
{
    type Error = error::Error;

    forward_to_deserialize! {
        deserialize_bool,
        deserialize_f64, deserialize_f32,
        deserialize_u8, deserialize_u16, deserialize_u32, deserialize_u64, deserialize_usize,
        deserialize_i8, deserialize_i16, deserialize_i32, deserialize_i64, deserialize_isize,
        deserialize_char, deserialize_str, deserialize_string,
        deserialize_ignored_any,
        deserialize_bytes,
        deserialize_unit_struct, deserialize_unit,
        deserialize_seq, deserialize_seq_fixed_size,
        deserialize_map, deserialize_newtype_struct, deserialize_struct_field,
        deserialize_tuple,
        deserialize_enum,
        deserialize_struct, deserialize_tuple_struct,
        deserialize_option
    }

    #[inline]
    fn deserialize<V>(&mut self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor
    {
        self.deserialize(visitor)
    }
}

impl Codec {
    fn parse(codec: Option<&[u8]>) -> error::Result<Codec> {
        match codec {
            None | Some(b"null") => Ok(Codec::Null),
            Some(b"deflate") => Ok(Codec::Deflate),
            Some(b"snappy") => Ok(Codec::Snappy),
            Some(codec) => {
                Err(ErrorKind::UnsupportedCodec(String::from_utf8_lossy(codec).into_owned()).into())
            },
        }
    }
}

impl<R> Blocks<R>
    where R: io::Read
{
    fn new(input: R, codec: Codec, sync_marker: Vec<u8>) -> Blocks<R> {
        Blocks {
            input: input,
            codec: codec,
            sync_marker: sync_marker,
            current_block: io::Cursor::new(Vec::new()),
        }
    }

    fn fill_buffer(&mut self) -> io::Result<()> {
        use std::io::Read;

        let mut buffer = self.current_block.get_mut();
        buffer.clear();

        let obj_count = match read_long(&mut self.input) {
            Ok(c) => c,
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        };

        let compressed_size = try!(read_long(&mut self.input));
        debug!("Loading block with compressed size {} containing {} objects",
               compressed_size,
               obj_count);

        match self.codec {
            Codec::Null => {
                let mut limited = (&mut self.input).take(compressed_size as u64);
                buffer.reserve(compressed_size as usize);
                try!(limited.read_to_end(buffer));
            },
            Codec::Deflate => {
                let limited = (&mut self.input).take(compressed_size as u64);
                let mut reader = flate2::read::DeflateDecoder::new(limited);
                try!(reader.read_to_end(buffer));
            },
            Codec::Snappy => {
                {
                    let limited = (&mut self.input).take(compressed_size as u64);
                    let mut reader = snap::Reader::new(limited);
                    try!(reader.read_to_end(buffer));
                }
                // Skip CRC checksum for now
                try!(self.input.read_exact(&mut vec![0; 4]));
            },
        }

        let mut sync_marker = vec![0; 16];
        try!(self.input.read_exact(&mut sync_marker));

        if self.sync_marker != sync_marker {
            Err(io::Error::new(io::ErrorKind::InvalidInput, "bad snappy sync marker"))
        } else {
            Ok(())
        }
    }
}

impl<R> io::Read for Blocks<R>
    where R: io::Read
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.current_block.position() as usize == self.current_block.get_ref().len() {
            try!(self.fill_buffer());
            self.current_block.set_position(0)
        }

        self.current_block.read(buf)
    }
}

impl<'a, R> RecordVisitor<'a, R>
    where R: io::BufRead
{
    fn new(input: &'a mut R,
           registry: &'a schema::SchemaRegistry,
           fields: schema::RecordFields<'a>)
           -> RecordVisitor<'a, R> {
        RecordVisitor {
            input: input,
            registry: registry,
            fields: fields,
            field: None,
        }
    }
}

impl<'a, R> serde::de::MapVisitor for RecordVisitor<'a, R>
    where R: io::BufRead
{
    type Error = error::Error;

    fn visit_key<K>(&mut self) -> error::Result<Option<K>>
        where K: serde::de::Deserialize
    {
        if let Some(f) = self.fields.next() {
            self.field = Some(f);
            debug!("Deserializing field {:?}", f.name());
            let k = try!(K::deserialize(&mut FieldNameDeserializer(f.name())));
            Ok(Some(k))
        } else {
            Ok(None)
        }
    }

    fn visit_value<V>(&mut self) -> error::Result<V>
        where V: serde::de::Deserialize
    {
        let field = self.field.take().expect("visit_value called before visit_field");
        let schema = field.field_type().resolve(&*self.registry);
        V::deserialize(&mut DeserializerImpl::new(self.input, &*self.registry, &schema))
    }

    fn end(&mut self) -> error::Result<()> {
        if self.fields.len() > 0 {
            // TODO: make custom error type
            Err(serde::de::Error::invalid_length(self.fields.len()))
        } else {
            Ok(())
        }
    }
}

impl<'a> serde::Deserializer for FieldNameDeserializer<'a> {
    type Error = error::Error;

    forward_to_deserialize! {
        deserialize_bool,
        deserialize_f64, deserialize_f32,
        deserialize_u8, deserialize_u16, deserialize_u32, deserialize_u64, deserialize_usize,
        deserialize_i8, deserialize_i16, deserialize_i32, deserialize_i64, deserialize_isize,
        deserialize_char, deserialize_str, deserialize_string,
        deserialize_ignored_any,
        deserialize_bytes,
        deserialize_unit_struct, deserialize_unit,
        deserialize_seq, deserialize_seq_fixed_size,
        deserialize_map, deserialize_newtype_struct, deserialize_struct_field,
        deserialize_tuple,
        deserialize_enum,
        deserialize_struct, deserialize_tuple_struct,
        deserialize_option
    }

    #[inline]
    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor
    {
        visitor.visit_str(self.0)
    }
}

impl BlockRemainder {
    fn next<R: io::Read>(&mut self, reader: &mut R) -> error::Result<bool> {
        match *self {
            BlockRemainder::Start |
            BlockRemainder::Count(0) => {
                let n = try!(read_block_size(reader));
                if n == 0 {
                    *self = BlockRemainder::End;
                    Ok(false)
                } else {
                    *self = BlockRemainder::Count(n - 1);
                    Ok(true)
                }
            },
            BlockRemainder::Count(n) => {
                if n == 0 {
                    *self = BlockRemainder::End;
                    Ok(false)
                } else {
                    *self = BlockRemainder::Count(n - 1);
                    Ok(true)
                }
            },
            BlockRemainder::End => Ok(false),
        }
    }
}

impl<'a, R> ArrayVisitor<'a, R>
    where R: io::BufRead
{
    fn new(input: &'a mut R,
           registry: &'a schema::SchemaRegistry,
           elem_schema: &'a schema::Schema)
           -> ArrayVisitor<'a, R> {
        ArrayVisitor {
            input: input,
            registry: registry,
            elem_schema: elem_schema,
            remainder: BlockRemainder::Start,
        }
    }
}

impl<'a, R> serde::de::SeqVisitor for ArrayVisitor<'a, R>
    where R: io::BufRead
{
    type Error = error::Error;

    fn visit<V>(&mut self) -> error::Result<Option<V>>
        where V: serde::de::Deserialize
    {
        if try!(self.remainder.next(self.input)) {
            debug!("Deserializing array element");
            let mut de = DeserializerImpl::new(self.input, self.registry, &self.elem_schema);
            let v = try!(V::deserialize(&mut de));
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }

    fn end(&mut self) -> error::Result<()> {
        match self.remainder {
            BlockRemainder::End => Ok(()),
            BlockRemainder::Count(n) => Err(serde::de::Error::invalid_length(n)),
            BlockRemainder::Start => panic!("seq visitor end() called before any call to visit()"),
        }
    }
}

impl<'a, R> MapVisitor<'a, R>
    where R: io::BufRead
{
    fn new(input: &'a mut R,
           registry: &'a schema::SchemaRegistry,
           value_schema: &'a schema::Schema)
           -> MapVisitor<'a, R> {
        MapVisitor {
            input: input,
            registry: registry,
            value_schema: value_schema,
            remainder: BlockRemainder::Start,
        }
    }
}

impl<'a, R> serde::de::MapVisitor for MapVisitor<'a, R>
    where R: io::BufRead
{
    type Error = error::Error;

    fn visit_key<K>(&mut self) -> error::Result<Option<K>>
        where K: serde::de::Deserialize
    {
        if try!(self.remainder.next(&mut self.input)) {
            let schema = schema::Schema::String;
            let mut de = DeserializerImpl::new(self.input, self.registry, &schema);
            let k = try!(K::deserialize(&mut de));
            Ok(Some(k))
        } else {
            Ok(None)
        }
    }

    fn visit_value<V>(&mut self) -> error::Result<V>
        where V: serde::de::Deserialize
    {
        V::deserialize(&mut DeserializerImpl::new(self.input, self.registry, &self.value_schema))
    }

    fn end(&mut self) -> error::Result<()> {
        match self.remainder {
            BlockRemainder::End => Ok(()),
            BlockRemainder::Count(n) => Err(serde::de::Error::invalid_length(n)),
            BlockRemainder::Start => {
                panic!("map visitor end() called before any call to visit_key()")
            },
        }
    }
}

fn read_block_size<R: io::Read>(reader: &mut R) -> error::Result<usize> {
    let n = try!(read_long(reader));
    let n = if n < 0 {
        try!(read_long(reader)); // discard
        n.abs()
    } else {
        n
    };
    Ok(n as usize)
}

fn read_int<R: io::Read>(reader: &mut R) -> error::Result<i32> {
    let v = try!(read_long(reader));
    if v < (i32::min_value() as i64) || v > (i32::max_value() as i64) {
        Err(ErrorKind::IntegerOverflow.into())
    } else {
        Ok(v as i32)
    }
}

fn read_long<R: io::Read>(reader: &mut R) -> io::Result<i64> {
    let unsigned = try!(decode_var_len_u64(reader));
    Ok(decode_zig_zag(unsigned))
}

// Taken from the rust-avro functions with the same name...
// TODO: credit this when creating an ATTRIBUTIONS file or something

fn decode_var_len_u64<R: io::Read>(reader: &mut R) -> io::Result<u64> {
    use byteorder::ReadBytesExt;

    let mut num = 0;
    let mut i = 0;
    loop {
        let byte = try!(reader.read_u8());

        if i >= 9 && byte & 0b1111_1110 != 0 {
            // 10th byte
            return Err(io::Error::new(io::ErrorKind::InvalidData, "integer overflow"));
        }
        num |= (byte as u64 & 0b0111_1111) << (i * 7);
        if byte & 0b1000_0000 == 0 {
            break;
        }
        i += 1;
    }
    Ok(num)
}

fn decode_zig_zag(num: u64) -> i64 {
    if num & 1 == 1 {
        !(num >> 1) as i64
    } else {
        (num >> 1) as i64
    }
}
