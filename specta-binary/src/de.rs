use serde::de::{
    self, DeserializeSeed, IntoDeserializer, Visitor, value::BorrowedStrDeserializer,
};

use crate::{
    Config, Error, ErrorKind, ValueType,
    value_type::{FLAG_HAS_KEY_TABLE, KNOWN_FLAGS, MAGIC, VERSION},
    varint,
};

/// A serde [`Deserializer`](serde::Deserializer) over a Specta Binary document.
///
/// Strings, bytes and object keys are borrowed from the input where the target type allows it.
///
/// ```
/// use serde::Deserialize;
///
/// let bytes = specta_binary::to_vec(&("a", 1))?;
/// let mut deserializer = specta_binary::Deserializer::from_slice(&bytes)?;
/// let value = <(&str, u8)>::deserialize(&mut deserializer)?;
/// deserializer.end()?;
/// assert_eq!(value, ("a", 1));
/// # Ok::<(), specta_binary::Error>(())
/// ```
#[derive(Debug)]
pub struct Deserializer<'de> {
    input: &'de [u8],
    pos: usize,
    /// The key table, if the document has one.
    keys: Option<Vec<&'de str>>,
    depth: usize,
    max_depth: usize,
}

type Result<T, E = Error> = std::result::Result<T, E>;

impl<'de> Deserializer<'de> {
    /// Read the header and key table of `input`.
    pub fn from_slice(input: &'de [u8]) -> Result<Self> {
        Self::from_slice_with(input, &Config::default())
    }

    /// Read the header and key table of `input`. Only [`Config::max_depth`] applies.
    pub fn from_slice_with(input: &'de [u8], config: &Config) -> Result<Self> {
        let mut de = Self {
            input,
            pos: 0,
            keys: None,
            depth: 0,
            max_depth: config.max_depth,
        };

        if de.read_slice(MAGIC.len())? != MAGIC {
            return Err(Error::at(ErrorKind::InvalidMagic, 0));
        }
        let version = de.read_u8()?;
        if version != VERSION {
            return Err(Error::at(ErrorKind::UnsupportedVersion(version), 2));
        }
        let flags = de.read_u8()?;
        if flags & !KNOWN_FLAGS != 0 {
            return Err(Error::at(ErrorKind::UnknownFlags(flags), 3));
        }
        if flags & FLAG_HAS_KEY_TABLE != 0 {
            let count = de.read_len()?;
            let keys = (0..count)
                .map(|_| de.read_len().and_then(|len| de.read_str(len)))
                .collect::<Result<_>>()?;
            de.keys = Some(keys);
        }
        Ok(de)
    }

    /// Check that the whole input has been consumed. Call this after deserializing the root value.
    pub fn end(&self) -> Result<()> {
        match self.pos == self.input.len() {
            true => Ok(()),
            false => Err(Error::at(ErrorKind::TrailingBytes, self.pos)),
        }
    }

    /// The current byte offset into the input.
    pub fn offset(&self) -> usize {
        self.pos
    }

    fn remaining(&self) -> usize {
        self.input.len() - self.pos
    }

    fn read_slice(&mut self, len: usize) -> Result<&'de [u8]> {
        let bytes = self
            .input
            .get(self.pos..)
            .and_then(|rest| rest.get(..len))
            .ok_or_else(|| Error::at(ErrorKind::UnexpectedEof, self.pos))?;
        self.pos += len;
        Ok(bytes)
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        let at = self.pos;
        self.read_slice(N)?
            .try_into()
            .map_err(|_| Error::at(ErrorKind::UnexpectedEof, at))
    }

    fn read_u8(&mut self) -> Result<u8> {
        self.read_array::<1>().map(|[byte]| byte)
    }

    fn read_varint(&mut self) -> Result<u32> {
        let rest = self.input.get(self.pos..).unwrap_or_default();
        match varint::read(rest) {
            Ok((value, used)) => {
                self.pos += used;
                Ok(value)
            }
            Err(varint::ReadError::Eof) => Err(Error::at(ErrorKind::UnexpectedEof, self.pos)),
            Err(varint::ReadError::Overflow) => {
                Err(Error::at(ErrorKind::VarintOverflow, self.pos))
            }
        }
    }

    /// Check a length or count read at `at` against the bytes that are left, before anything is allocated.
    fn check_len(&self, len: u32, at: usize) -> Result<usize> {
        let remaining = self.remaining();
        match len as usize <= remaining {
            true => Ok(len as usize),
            false => Err(Error::at(
                ErrorKind::LengthOutOfBounds { len, remaining },
                at,
            )),
        }
    }

    fn read_len(&mut self) -> Result<usize> {
        let at = self.pos;
        let len = self.read_varint()?;
        self.check_len(len, at)
    }

    /// Read the length or count following `tag`: one byte for short tags, a varint otherwise.
    fn read_len_of(&mut self, tag: ValueType) -> Result<usize> {
        if !tag.is_short() {
            return self.read_len();
        }
        let at = self.pos;
        let len = self.read_u8()?;
        self.check_len(len.into(), at)
    }

    fn read_str(&mut self, len: usize) -> Result<&'de str> {
        let at = self.pos;
        std::str::from_utf8(self.read_slice(len)?)
            .map_err(|_| Error::at(ErrorKind::InvalidUtf8, at))
    }

    fn peek_tag(&self) -> Result<ValueType> {
        let byte = *self
            .input
            .get(self.pos)
            .ok_or_else(|| Error::at(ErrorKind::UnexpectedEof, self.pos))?;
        ValueType::try_from(byte).map_err(|byte| Error::at(ErrorKind::UnknownTag(byte), self.pos))
    }

    fn read_tag(&mut self) -> Result<ValueType> {
        let tag = self.peek_tag()?;
        self.pos += 1;
        Ok(tag)
    }

    fn read_bool(&mut self) -> Result<bool> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            byte => Err(Error::at(ErrorKind::InvalidBool(byte), self.pos - 1)),
        }
    }

    fn read_i32(&mut self) -> Result<i32> {
        self.read_array().map(i32::from_le_bytes)
    }

    fn read_i64(&mut self) -> Result<i64> {
        self.read_array().map(i64::from_le_bytes)
    }

    fn read_f32(&mut self) -> Result<f32> {
        self.read_array().map(f32::from_le_bytes)
    }

    fn read_f64(&mut self) -> Result<f64> {
        self.read_array().map(f64::from_le_bytes)
    }

    fn read_key(&mut self) -> Result<&'de str> {
        let at = self.pos;
        let n = self.read_varint()?;
        match &self.keys {
            Some(keys) => keys.get(n as usize).copied().ok_or_else(|| {
                Error::at(
                    ErrorKind::KeyIdOutOfRange {
                        id: n,
                        count: keys.len(),
                    },
                    at,
                )
            }),
            None => {
                let len = self.check_len(n, at)?;
                self.read_str(len)
            }
        }
    }

    /// Read an `Int32` or `Int64`, returning it with the offset of its tag.
    fn read_integer(&mut self, expected: &'static str) -> Result<(i64, usize)> {
        let at = self.pos;
        match self.read_tag()? {
            ValueType::Int32 => Ok((self.read_i32()?.into(), at)),
            ValueType::Int64 => Ok((self.read_i64()?, at)),
            found => Err(mismatch(expected, found, at)),
        }
    }

    fn read_byte_element(&mut self) -> Result<u8> {
        let (value, at) = self.read_integer("u8")?;
        u8::try_from(value).map_err(|_| {
            Error::at(
                ErrorKind::IntegerOutOfRange { value, target: "u8" },
                at,
            )
        })
    }

    fn enter(&mut self, at: usize) -> Result<()> {
        if self.depth >= self.max_depth {
            return Err(Error::at(
                ErrorKind::DepthLimitExceeded(self.max_depth),
                at,
            ));
        }
        self.depth += 1;
        Ok(())
    }

    fn visit_array<V: Visitor<'de>>(&mut self, count: usize, at: usize, visitor: V) -> Result<V::Value> {
        self.enter(at)?;
        let mut access = SeqAccess {
            de: self,
            remaining: count,
            index: 0,
        };
        let value = visitor.visit_seq(&mut access);
        let remaining = access.remaining;
        self.depth -= 1;
        let value = value?;
        match remaining {
            0 => Ok(value),
            n => Err(Error::at(ErrorKind::TrailingElements(n), self.pos)),
        }
    }

    fn visit_object<V: Visitor<'de>>(&mut self, count: usize, at: usize, visitor: V) -> Result<V::Value> {
        self.enter(at)?;
        let mut access = MapAccess {
            de: self,
            remaining: count,
            key: "",
        };
        let value = visitor.visit_map(&mut access);
        let remaining = access.remaining;
        self.depth -= 1;
        let value = value?;
        match remaining {
            0 => Ok(value),
            n => Err(Error::at(ErrorKind::TrailingElements(n), self.pos)),
        }
    }

    /// Skip over a value without visiting it, still enforcing every limit.
    fn skip(&mut self) -> Result<()> {
        let at = self.pos;
        match self.read_tag()? {
            ValueType::Null => {}
            ValueType::Bool => {
                self.read_bool()?;
            }
            ValueType::Int32 | ValueType::Float32 => {
                self.read_slice(4)?;
            }
            ValueType::Int64 | ValueType::Float64 => {
                self.read_slice(8)?;
            }
            tag @ (ValueType::StringUtf8 | ValueType::ShortStringUtf8) => {
                let len = self.read_len_of(tag)?;
                self.read_str(len)?;
            }
            tag @ (ValueType::ByteArray | ValueType::ShortByteArray) => {
                let len = self.read_len_of(tag)?;
                self.read_slice(len)?;
            }
            tag @ (ValueType::Array | ValueType::ShortArray) => {
                let count = self.read_len_of(tag)?;
                self.enter(at)?;
                let result =
                    (0..count).try_for_each(|index| self.skip().map_err(|err| err.in_index(index)));
                self.depth -= 1;
                result?;
            }
            tag @ (ValueType::Object | ValueType::ShortObject) => {
                let count = self.read_len_of(tag)?;
                self.enter(at)?;
                let result = (0..count).try_for_each(|_| {
                    let key = self.read_key()?;
                    self.skip().map_err(|err| err.in_key(key))
                });
                self.depth -= 1;
                result?;
            }
        }
        Ok(())
    }
}

fn mismatch(expected: &'static str, found: ValueType, at: usize) -> Error {
    Error::at(ErrorKind::TypeMismatch { expected, found }, at)
}

macro_rules! deserialize_narrow_int {
    ($($method:ident => $visit:ident($ty:ty)),* $(,)?) => {$(
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            let (value, at) = self.read_integer(stringify!($ty))?;
            let value = <$ty>::try_from(value).map_err(|_| {
                Error::at(ErrorKind::IntegerOutOfRange { value, target: stringify!($ty) }, at)
            })?;
            visitor.$visit(value)
        }
    )*};
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let at = self.pos;
        match self.read_tag()? {
            ValueType::Null => visitor.visit_unit(),
            ValueType::Bool => visitor.visit_bool(self.read_bool()?),
            ValueType::Int32 => visitor.visit_i32(self.read_i32()?),
            ValueType::Int64 => visitor.visit_i64(self.read_i64()?),
            ValueType::Float32 => visitor.visit_f32(self.read_f32()?),
            ValueType::Float64 => visitor.visit_f64(self.read_f64()?),
            tag @ (ValueType::StringUtf8 | ValueType::ShortStringUtf8) => {
                let len = self.read_len_of(tag)?;
                visitor.visit_borrowed_str(self.read_str(len)?)
            }
            tag @ (ValueType::ByteArray | ValueType::ShortByteArray) => {
                let len = self.read_len_of(tag)?;
                visitor.visit_borrowed_bytes(self.read_slice(len)?)
            }
            tag @ (ValueType::Array | ValueType::ShortArray) => {
                let count = self.read_len_of(tag)?;
                self.visit_array(count, at, visitor)
            }
            tag @ (ValueType::Object | ValueType::ShortObject) => {
                let count = self.read_len_of(tag)?;
                self.visit_object(count, at, visitor)
            }
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let at = self.pos;
        match self.read_tag()? {
            ValueType::Bool => visitor.visit_bool(self.read_bool()?),
            found => Err(mismatch("bool", found, at)),
        }
    }

    deserialize_narrow_int! {
        deserialize_i8 => visit_i8(i8),
        deserialize_i16 => visit_i16(i16),
        deserialize_i32 => visit_i32(i32),
        deserialize_u8 => visit_u8(u8),
        deserialize_u16 => visit_u16(u16),
        deserialize_u32 => visit_u32(u32),
        deserialize_u64 => visit_u64(u64),
        deserialize_u128 => visit_u128(u128),
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(self.read_integer("i64")?.0)
    }

    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i128(self.read_integer("i128")?.0.into())
    }

    // Any numeric tag can feed a float, because JavaScript can't tell `1.0` from `1`.
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let at = self.pos;
        let value = match self.read_tag()? {
            ValueType::Float32 => self.read_f32()?,
            ValueType::Float64 => self.read_f64()? as f32,
            ValueType::Int32 => self.read_i32()? as f32,
            ValueType::Int64 => self.read_i64()? as f32,
            found => return Err(mismatch("f32", found, at)),
        };
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let at = self.pos;
        let value = match self.read_tag()? {
            ValueType::Float64 => self.read_f64()?,
            ValueType::Float32 => self.read_f32()?.into(),
            ValueType::Int32 => self.read_i32()?.into(),
            ValueType::Int64 => self.read_i64()? as f64,
            found => return Err(mismatch("f64", found, at)),
        };
        visitor.visit_f64(value)
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let at = self.pos;
        match self.read_tag()? {
            tag @ (ValueType::StringUtf8 | ValueType::ShortStringUtf8) => {
                let len = self.read_len_of(tag)?;
                visitor.visit_borrowed_str(self.read_str(len)?)
            }
            found => Err(mismatch("string", found, at)),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    // Accepts an `Array` of integers in 0..=255 too, since the TS encoder writes `number[]` that way.
    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let at = self.pos;
        match self.read_tag()? {
            tag @ (ValueType::ByteArray | ValueType::ShortByteArray) => {
                let len = self.read_len_of(tag)?;
                visitor.visit_borrowed_bytes(self.read_slice(len)?)
            }
            tag @ (ValueType::Array | ValueType::ShortArray) => {
                let count = self.read_len_of(tag)?;
                let bytes = (0..count)
                    .map(|index| self.read_byte_element().map_err(|err| err.in_index(index)))
                    .collect::<Result<Vec<_>>>()?;
                visitor.visit_byte_buf(bytes)
            }
            found => Err(mismatch("bytes", found, at)),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.peek_tag()? {
            ValueType::Null => {
                self.pos += 1;
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let at = self.pos;
        match self.read_tag()? {
            ValueType::Null => visitor.visit_unit(),
            found => Err(mismatch("null", found, at)),
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    // Accepts a `ByteArray` too, since the TS encoder writes `Uint8Array` that way.
    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let at = self.pos;
        match self.read_tag()? {
            tag @ (ValueType::Array | ValueType::ShortArray) => {
                let count = self.read_len_of(tag)?;
                self.visit_array(count, at, visitor)
            }
            tag @ (ValueType::ByteArray | ValueType::ShortByteArray) => {
                let len = self.read_len_of(tag)?;
                let mut access = BytesAccess {
                    bytes: self.read_slice(len)?.iter(),
                    index: 0,
                };
                let value = visitor.visit_seq(&mut access)?;
                match access.bytes.len() {
                    0 => Ok(value),
                    n => Err(Error::at(ErrorKind::TrailingElements(n), self.pos)),
                }
            }
            found => Err(mismatch("array", found, at)),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let at = self.pos;
        match self.read_tag()? {
            tag @ (ValueType::Object | ValueType::ShortObject) => {
                let count = self.read_len_of(tag)?;
                self.visit_object(count, at, visitor)
            }
            found => Err(mismatch("object", found, at)),
        }
    }

    // Like `serde_json`, a struct can also be read from an array of its fields in order.
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        match self.peek_tag()? {
            ValueType::Array | ValueType::ShortArray => self.deserialize_seq(visitor),
            _ => self.deserialize_map(visitor),
        }
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let at = self.pos;
        match self.read_tag()? {
            tag @ (ValueType::StringUtf8 | ValueType::ShortStringUtf8) => {
                let len = self.read_len_of(tag)?;
                visitor.visit_enum(BorrowedStrDeserializer::new(self.read_str(len)?))
            }
            tag @ (ValueType::Object | ValueType::ShortObject) => {
                match self.read_len_of(tag)? {
                    1 => {}
                    count => return Err(Error::at(ErrorKind::InvalidEnumObject(count), at)),
                }
                self.enter(at)?;
                let value = visitor.visit_enum(EnumAccess { de: self });
                self.depth -= 1;
                value
            }
            found => Err(mismatch("enum", found, at)),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.skip()?;
        visitor.visit_unit()
    }
}

struct SeqAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    remaining: usize,
    index: usize,
}

impl<'de> de::SeqAccess<'de> for SeqAccess<'_, 'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        let (at, index) = (self.de.pos, self.index);
        self.index += 1;
        seed.deserialize(&mut *self.de)
            .map(Some)
            .map_err(|err| err.or_offset(at).in_index(index))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

/// Reads a `ByteArray` as a sequence of `u8`.
struct BytesAccess<'de> {
    bytes: std::slice::Iter<'de, u8>,
    index: usize,
}

impl<'de> de::SeqAccess<'de> for BytesAccess<'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        let Some(&byte) = self.bytes.next() else {
            return Ok(None);
        };
        let index = self.index;
        self.index += 1;
        seed.deserialize(IntoDeserializer::<Error>::into_deserializer(byte))
            .map(Some)
            .map_err(|err| err.in_index(index))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.bytes.len())
    }
}

struct MapAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    remaining: usize,
    /// The most recently read key, used to name the entry in error paths.
    key: &'de str,
}

impl<'de> de::MapAccess<'de> for MapAccess<'_, 'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        let at = self.de.pos;
        self.key = self.de.read_key()?;
        seed.deserialize(KeyDeserializer(self.key))
            .map(Some)
            .map_err(|err| err.or_offset(at).in_key(self.key))
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        let at = self.de.pos;
        seed.deserialize(&mut *self.de)
            .map_err(|err| err.or_offset(at).in_key(self.key))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

/// The `{ "Variant": content }` form of an externally tagged enum.
struct EnumAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> de::EnumAccess<'de> for EnumAccess<'a, 'de> {
    type Error = Error;
    type Variant = VariantAccess<'a, 'de>;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        let at = self.de.pos;
        let variant = self.de.read_key()?;
        let value = seed
            .deserialize(KeyDeserializer(variant))
            .map_err(|err| err.or_offset(at))?;
        Ok((value, VariantAccess { de: self.de, variant }))
    }
}

struct VariantAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    variant: &'de str,
}

impl VariantAccess<'_, '_> {
    fn locate(&self, at: usize) -> impl FnOnce(Error) -> Error + '_ {
        move |err| err.or_offset(at).in_key(self.variant)
    }
}

impl<'de> de::VariantAccess<'de> for VariantAccess<'_, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        let at = self.de.pos;
        de::Deserialize::deserialize(&mut *self.de).map_err(self.locate(at))
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        let at = self.de.pos;
        seed.deserialize(&mut *self.de).map_err(self.locate(at))
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        let at = self.de.pos;
        de::Deserializer::deserialize_seq(&mut *self.de, visitor).map_err(self.locate(at))
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let at = self.de.pos;
        de::Deserializer::deserialize_struct(&mut *self.de, "", fields, visitor)
            .map_err(self.locate(at))
    }
}

/// Deserializes an object key. Like `serde_json`'s map keys, a string key can feed
/// an integer, float, `bool` or `char` target by parsing it.
struct KeyDeserializer<'de>(&'de str);

macro_rules! deserialize_parsed_key {
    ($($method:ident => $visit:ident),* $(,)?) => {$(
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            match self.0.parse() {
                Ok(value) => visitor.$visit(value),
                // Let the visitor report the type mismatch.
                Err(_) => visitor.visit_borrowed_str(self.0),
            }
        }
    )*};
}

impl<'de> de::Deserializer<'de> for KeyDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_borrowed_str(self.0)
    }

    deserialize_parsed_key! {
        deserialize_bool => visit_bool,
        deserialize_i8 => visit_i8,
        deserialize_i16 => visit_i16,
        deserialize_i32 => visit_i32,
        deserialize_i64 => visit_i64,
        deserialize_i128 => visit_i128,
        deserialize_u8 => visit_u8,
        deserialize_u16 => visit_u16,
        deserialize_u32 => visit_u32,
        deserialize_u64 => visit_u64,
        deserialize_u128 => visit_u128,
        deserialize_f32 => visit_f32,
        deserialize_f64 => visit_f64,
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_enum(BorrowedStrDeserializer::new(self.0))
    }

    serde::forward_to_deserialize_any! {
        char str string bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}
