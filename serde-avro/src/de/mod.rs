use byteorder;
use error::{self, Error, ErrorKind};
use header;
use schema;
use serde;
use serde_json;
use std::borrow;
use std::io;

pub mod read;
mod util;

pub struct Deserializer<'a, R>
    where R: io::Read + read::Limit
{
    input: R,
    registry: borrow::Cow<'a, schema::SchemaRegistry>,
    schema: borrow::Cow<'a, schema::Schema>,
}

struct DeserializerImpl<'a, R>
    where R: io::Read + 'a
{
    input: &'a mut R,
    registry: &'a schema::SchemaRegistry,
    schema: &'a schema::Schema,
}

struct RecordVisitor<'a, R>
    where R: io::Read + 'a
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
    where R: io::Read + 'a
{
    input: &'a mut R,
    registry: &'a schema::SchemaRegistry,
    elem_schema: &'a schema::Schema,
    remainder: BlockRemainder,
}

struct MapVisitor<'a, R>
    where R: io::Read + 'a
{
    input: &'a mut R,
    registry: &'a schema::SchemaRegistry,
    value_schema: &'a schema::Schema,
    remainder: BlockRemainder,
}

impl<'a, R> Deserializer<'a, R>
    where R: io::Read + read::Limit
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

impl<'a, R> Deserializer<'a, read::Blocks<R>>
    where R: io::Read
{
    // TODO: this uses a ridiculous number of buffers... We can cut that down significantly
    pub fn from_container(mut input: R) -> error::Result<Deserializer<'static, read::Blocks<R>>> {
        use serde::de::Deserialize;

        let header = {
            debug!("Parsing container header");
            let direct = read::Direct::new(&mut input, 1);
            let mut header_de =
                Deserializer::new(direct, &schema::EMPTY_REGISTRY, &schema::FILE_HEADER);
            try!(header::Header::deserialize(&mut header_de))
        };
        debug!("Container header: {:?}", header);

        if &[b'O', b'b', b'j', 1] != &*header.magic {
            Err(ErrorKind::BadFileMagic(header.magic.to_vec()).into())
        } else {
            let codec = try!(read::Codec::parse(header.meta.get("avro.codec").map(AsRef::as_ref)));
            let schema_data = try!(header.meta
                .get("avro.schema")
                .ok_or(Error::from(ErrorKind::NoSchema)));

            let schema_json = try!(serde_json::from_slice(&schema_data));
            let mut registry = schema::SchemaRegistry::new();

            let root_schema = try!(try!(registry.add_json(&schema_json))
                    .ok_or(Error::from(ErrorKind::NoRootType)))
                .into_resolved(&registry);

            let blocks = read::Blocks::new(input, codec, header.sync.to_vec());
            let registry_cow = borrow::Cow::Owned(registry);
            let schema_cow = borrow::Cow::Owned(root_schema);
            Ok(Deserializer::new_cow(blocks, registry_cow, schema_cow))
        }
    }
}

impl<'a, R> serde::Deserializer for Deserializer<'a, R>
    where R: io::Read + read::Limit
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
        if !try!(self.input.take_limit()) {
            return Err(serde::de::Error::end_of_stream());
        }

        DeserializerImpl::new(&mut self.input, &*self.registry, &*self.schema).deserialize(visitor)
    }
}

impl<'a, R> DeserializerImpl<'a, R>
    where R: io::Read
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
                let v = try!(util::read_int(self.input));
                debug!("Deserializing int {:?}", v);
                visitor.visit_i32(v)
            },
            Schema::Long => {
                let v = try!(util::read_long(self.input));
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
                let len = try!(util::read_long(self.input));

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
                let len = try!(util::read_long(self.input));

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
                let v = try!(util::read_int(self.input));
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
                let variant = try!(util::read_long(self.input));
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
    where R: io::Read
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

impl<'a, R> RecordVisitor<'a, R>
    where R: io::Read
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
    where R: io::Read
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
                let n = try!(util::read_block_size(reader));
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
    where R: io::Read
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
    where R: io::Read
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
    where R: io::Read
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
    where R: io::Read
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
