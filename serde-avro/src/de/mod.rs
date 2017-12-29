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
    header: borrow::Cow<'a, header::Header>,
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
               header: &'a header::Header,
               registry: &'a schema::SchemaRegistry,
               schema: &'a schema::Schema)
               -> Deserializer<'a, R> {
        Deserializer::new_cow(input,
                              borrow::Cow::Borrowed(header),
                              borrow::Cow::Borrowed(registry),
                              borrow::Cow::Borrowed(schema))
    }

    fn new_cow(input: R,
               header: borrow::Cow<'a, header::Header>,
               registry: borrow::Cow<'a, schema::SchemaRegistry>,
               schema: borrow::Cow<'a, schema::Schema>)
               -> Deserializer<'a, R> {
        Deserializer {
            input: input,
            header: header,
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
            let default_header = Default::default();
            let mut header_de =
                Deserializer::new(direct, &default_header, &schema::EMPTY_REGISTRY, &schema::FILE_HEADER);
            header::Header::deserialize(&mut header_de)?
        };
        debug!("Container header: {:?}", header);

        if &[b'O', b'b', b'j', 1] != &*header.magic {
            Err(ErrorKind::BadFileMagic(header.magic.to_vec()).into())
        } else {
            let codec = read::Codec::parse(header.meta.get("avro.codec").map(AsRef::as_ref))?;
            let schema_data = header
                .get("avro.schema")
                .ok_or(Error::from(ErrorKind::NoSchema))?;

            let schema_json = serde_json::from_slice(&schema_data)?;
            let mut registry = schema::SchemaRegistry::new();

            let root_schema = registry.add_json(&schema_json)?
                .ok_or(Error::from(ErrorKind::NoRootType))?
                .into_resolved(&registry);

            let blocks = read::Blocks::new(input, codec, header.clone().sync.to_vec());
            let header_cow = borrow::Cow::Owned(header);
            let registry_cow = borrow::Cow::Owned(registry);
            let schema_cow = borrow::Cow::Owned(root_schema);
            Ok(Deserializer::new_cow(blocks, header_cow, registry_cow, schema_cow))
        }
    }

    /// Returns the name assigned to the given Avro schema.
    pub fn name(&self) -> String {
        self.header.name()
    }
}

impl<'a, 'b, R> serde::Deserializer for &'b mut Deserializer<'a, R>
    where R: io::Read + read::Limit
{
    type Error = error::Error;

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct struct_field tuple enum ignored_any
    }

    #[inline]
    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor
    {
        if !self.input.take_limit()? {
            bail!(error::ErrorKind::EndOfStream)
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

    fn deserialize<V>(&mut self, visitor: V) -> Result<V::Value, error::Error>
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
                let v = self.input.read_u8()?;
                debug!("Deserializing boolean {:?}", v);
                // TODO: if v is not in [0, 1], report error
                visitor.visit_bool(v != 0)
            },
            Schema::Int => {
                let v = util::read_int(self.input)?;
                debug!("Deserializing int {:?}", v);
                visitor.visit_i32(v)
            },
            Schema::Long => {
                let v = util::read_long(self.input)?;
                debug!("Deserializing long {:?}", v);
                visitor.visit_i64(v)
            },
            Schema::Float => {
                let v = self.input.read_f32::<byteorder::LittleEndian>()?;
                debug!("Deserializing float {:?}", v);
                visitor.visit_f32(v)
            },
            Schema::Double => {
                let v = self.input.read_f64::<byteorder::LittleEndian>()?;
                debug!("Deserializing double {:?}", v);
                visitor.visit_f64(v)
            },
            Schema::Bytes => {
                let len = util::read_long(self.input)?;

                if len < 0 {
                    Err(ErrorKind::NegativeLength.into())
                } else {
                    let mut result = vec![0; len as usize];
                    self.input.read_exact(&mut result)?;
                    debug!("Deserializing bytes {:?}", result);
                    visitor.visit_byte_buf(result)
                }
            },
            Schema::String => {
                let len = util::read_long(self.input)?;

                if len < 0 {
                    Err(ErrorKind::NegativeLength.into())
                } else {
                    let mut buffer = vec![0; len as usize];
                    self.input.read_exact(&mut buffer)?;
                    let result = String::from_utf8(buffer)?;
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
                let v = util::read_int(self.input)?;
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
                let variant = util::read_long(self.input)?;
                let schema = inner[variant as usize].resolve(&self.registry);
                DeserializerImpl::new(self.input, self.registry, &schema).deserialize(visitor)
            },
            Schema::Fixed(ref inner) => {
                debug!("Deserializing fixed of size {}", inner.size());
                let mut buffer = vec![0; inner.size() as usize];
                self.input.read_exact(&mut buffer)?;
                visitor.visit_byte_buf(buffer)
            },
        }
    }
}


impl<'a, 'b, R> serde::Deserializer for &'b mut DeserializerImpl<'a, R>
    where R: io::Read
{
    type Error = error::Error;

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct struct_field tuple enum ignored_any
    }

    #[inline]
    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
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

    fn visit_key_seed<K>(&mut self, seed: K) -> error::Result<Option<K::Value>>
        where K: serde::de::DeserializeSeed
    {
        if let Some(f) = self.fields.next() {
            self.field = Some(f);
            debug!("Deserializing field {:?}", f.name());
            let k = seed.deserialize(FieldNameDeserializer(f.name()))?;
            Ok(Some(k))
        } else {
            Ok(None)
        }
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> error::Result<V::Value>
        where V: serde::de::DeserializeSeed
    {
        let field = self.field.take().expect("visit_value called before visit_field");
        let schema = field.field_type().resolve(&*self.registry);
        seed.deserialize(&mut DeserializerImpl::new(self.input, &*self.registry, &schema))
    }
}

impl<'a> serde::Deserializer for FieldNameDeserializer<'a> {
    type Error = error::Error;

    forward_to_deserialize! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct struct_field tuple enum ignored_any
    }

    #[inline]
    fn deserialize<V>(self, visitor: V) -> Result<V::Value, Self::Error>
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
                let n = util::read_block_size(reader)?;
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

    fn visit_seed<V>(&mut self, seed: V) -> error::Result<Option<V::Value>>
        where V: serde::de::DeserializeSeed
    {
        if self.remainder.next(self.input)? {
            debug!("Deserializing array element");
            let mut de = DeserializerImpl::new(self.input, self.registry, &self.elem_schema);
            let v = seed.deserialize(&mut de)?;
            Ok(Some(v))
        } else {
            Ok(None)
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

    fn visit_key_seed<K>(&mut self, seed: K) -> error::Result<Option<K::Value>>
        where K: serde::de::DeserializeSeed
    {
        if self.remainder.next(&mut self.input)? {
            let schema = schema::Schema::String;
            let mut de = DeserializerImpl::new(self.input, self.registry, &schema);
            let k = seed.deserialize(&mut de)?;
            Ok(Some(k))
        } else {
            Ok(None)
        }
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> error::Result<V::Value>
        where V: serde::de::DeserializeSeed
    {
        seed.deserialize(&mut DeserializerImpl::new(self.input, self.registry, &self.value_schema))
    }
}
