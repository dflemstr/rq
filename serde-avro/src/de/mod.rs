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

pub struct Deserializer<'de, R>
    where R: io::Read + read::Limit
{
    input: R,
    registry: borrow::Cow<'de, schema::SchemaRegistry>,
    schema: borrow::Cow<'de, schema::Schema>,
}

struct DeserializerImpl<'de, R>
    where R: io::Read + 'de
{
    input: &'de mut R,
    registry: &'de schema::SchemaRegistry,
    schema: &'de schema::Schema,
}

struct RecordVisitor<'de, R>
    where R: io::Read + 'de
{
    input: &'de mut R,
    registry: &'de schema::SchemaRegistry,
    fields: schema::RecordFields<'de>,
    field: Option<&'de schema::FieldSchema>,
}

struct FieldNameDeserializer<'de>(&'de str);

enum BlockRemainder {
    Start,
    Count(usize),
    End,
}

struct ArrayVisitor<'de, R>
    where R: io::Read + 'de
{
    input: &'de mut R,
    registry: &'de schema::SchemaRegistry,
    elem_schema: &'de schema::Schema,
    remainder: BlockRemainder,
}

struct MapVisitor<'de, R>
    where R: io::Read + 'de
{
    input: &'de mut R,
    registry: &'de schema::SchemaRegistry,
    value_schema: &'de schema::Schema,
    remainder: BlockRemainder,
}

impl<'de, R> Deserializer<'de, R>
    where R: io::Read + read::Limit
{
    pub fn new(input: R,
               registry: &'de schema::SchemaRegistry,
               schema: &'de schema::Schema)
               -> Deserializer<'de, R> {
        Deserializer::new_cow(input,
                              borrow::Cow::Borrowed(registry),
                              borrow::Cow::Borrowed(schema))
    }

    fn new_cow(input: R,
               registry: borrow::Cow<'de, schema::SchemaRegistry>,
               schema: borrow::Cow<'de, schema::Schema>)
               -> Deserializer<'de, R> {
        Deserializer {
            input: input,
            registry: registry,
            schema: schema,
        }
    }
}

impl<'de, R> Deserializer<'de, read::Blocks<R>>
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
            header::Header::deserialize(&mut header_de)?
        };
        debug!("Container header: {:?}", header);

        if &[b'O', b'b', b'j', 1] != &*header.magic {
            Err(ErrorKind::BadFileMagic(header.magic.to_vec()).into())
        } else {
            let codec = read::Codec::parse(header.meta.get("avro.codec").map(AsRef::as_ref))?;
            let schema_data = header.meta
                .get("avro.schema")
                .ok_or(Error::from(ErrorKind::NoSchema))?;

            let schema_json = serde_json::from_slice(&schema_data)?;
            let mut registry = schema::SchemaRegistry::new();

            let root_schema = registry.add_json(&schema_json)?
                .ok_or(Error::from(ErrorKind::NoRootType))?
                .into_resolved(&registry);

            let blocks = read::Blocks::new(input, codec, header.sync.to_vec());
            let registry_cow = borrow::Cow::Owned(registry);
            let schema_cow = borrow::Cow::Owned(root_schema);
            Ok(Deserializer::new_cow(blocks, registry_cow, schema_cow))
        }
    }
}

impl<'de, 'b, R> serde::Deserializer<'de> for &'de mut Deserializer<'de, R>
    where R: io::Read + read::Limit
{
    type Error = error::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de>
    {
        if !self.input.take_limit()? {
            bail!(error::ErrorKind::EndOfStream)
        }

        DeserializerImpl::new(&mut self.input, &*self.registry, &*self.schema).deserialize(visitor)
    }
}

impl<'de, R> DeserializerImpl<'de, R>
    where R: io::Read
{
    pub fn new(input: &'de mut R,
               registry: &'de schema::SchemaRegistry,
               schema: &'de schema::Schema)
               -> DeserializerImpl<'de, R> {
        DeserializerImpl {
            input: input,
            registry: registry,
            schema: schema,
        }
    }

    fn deserialize<V>(&mut self, visitor: V) -> Result<V::Value, error::Error>
        where V: serde::de::Visitor<'de>
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


impl<'de, 'b, R> serde::Deserializer<'de> for &'de mut DeserializerImpl<'de, R>
    where R: io::Read
{
    type Error = error::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de>
    {
        self.deserialize(visitor)
    }
}

impl<'de, R> RecordVisitor<'de, R>
    where R: io::Read
{
    fn new(input: &'de mut R,
           registry: &'de schema::SchemaRegistry,
           fields: schema::RecordFields<'de>)
           -> RecordVisitor<'de, R> {
        RecordVisitor {
            input: input,
            registry: registry,
            fields: fields,
            field: None,
        }
    }
}

impl<'de, R> serde::de::MapAccess<'de> for RecordVisitor<'de, R>
    where R: io::Read
{
    type Error = error::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> error::Result<Option<K::Value>>
        where K: serde::de::DeserializeSeed<'de>
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

    fn next_value_seed<V>(&mut self, seed: V) -> error::Result<V::Value>
        where V: serde::de::DeserializeSeed<'de>
    {
        let field = self.field.take().expect("visit_value called before visit_field");
        let schema = field.field_type().resolve(&*self.registry);
        seed.deserialize(&mut DeserializerImpl::new(self.input, &*self.registry, &schema))
    }
}

impl<'de> serde::Deserializer<'de> for FieldNameDeserializer<'de> {
    type Error = error::Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de>
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

impl<'de, R> ArrayVisitor<'de, R>
    where R: io::Read
{
    fn new(input: &'de mut R,
           registry: &'de schema::SchemaRegistry,
           elem_schema: &'de schema::Schema)
           -> ArrayVisitor<'de, R> {
        ArrayVisitor {
            input: input,
            registry: registry,
            elem_schema: elem_schema,
            remainder: BlockRemainder::Start,
        }
    }
}

impl<'de, R> serde::de::SeqAccess<'de> for ArrayVisitor<'de, R>
    where R: io::Read
{
    type Error = error::Error;

    fn next_element_seed<V>(&mut self, seed: V) -> error::Result<Option<V::Value>>
        where V: serde::de::DeserializeSeed<'de>
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

impl<'de, R> MapVisitor<'de, R>
    where R: io::Read
{
    fn new(input: &'de mut R,
           registry: &'de schema::SchemaRegistry,
           value_schema: &'de schema::Schema)
           -> MapVisitor<'de, R> {
        MapVisitor {
            input: input,
            registry: registry,
            value_schema: value_schema,
            remainder: BlockRemainder::Start,
        }
    }
}

impl<'de, R> serde::de::MapAccess<'de> for MapVisitor<'de, R>
    where R: io::Read
{
    type Error = error::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> error::Result<Option<K::Value>>
        where K: serde::de::DeserializeSeed<'de>
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

    fn next_value_seed<V>(&mut self, seed: V) -> error::Result<V::Value>
        where V: serde::de::DeserializeSeed<'de>
    {
        seed.deserialize(&mut DeserializerImpl::new(self.input, self.registry, &self.value_schema))
    }
}
