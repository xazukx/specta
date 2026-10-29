use std::{borrow::Cow, fmt};

use serde::ser::{self, Impossible, Serialize};

use crate::{
    Config, Error, ErrorKind, ValueType,
    decimal::{Decimal, FLOAT64_LEN},
    table::{self, StringTable},
    value_type::{FLAG_HAS_KEY_TABLE, FLAG_HAS_VALUE_TABLE, MAGIC, SHORT_MAX, VERSION},
    varint,
};

/// A serde [`Serializer`](serde::Serializer) that produces a Specta Binary document.
///
/// Serialization is buffered: the body is written first so the key and value tables can be
/// built from the strings it uses, then [`Serializer::into_bytes`] prepends the header and tables.
///
/// ```
/// use serde::Serialize;
///
/// let mut serializer = specta_binary::Serializer::new(&specta_binary::Config::default());
/// vec![1, 2].serialize(&mut serializer)?;
/// assert_eq!(serializer.into_bytes(), specta_binary::to_vec(&vec![1, 2])?);
/// # Ok::<(), specta_binary::Error>(())
/// ```
#[derive(Debug)]
pub struct Serializer {
    body: Vec<u8>,
    keys: Option<StringTable>,
    values: Option<StringTable>,
}

impl Serializer {
    pub fn new(config: &Config) -> Self {
        Self {
            body: Vec::new(),
            keys: config.key_table.then(StringTable::default),
            values: config.value_table.then(StringTable::default),
        }
    }

    /// Assemble the document: header, key table and value table (each only if it isn't empty),
    /// then the body.
    pub fn into_bytes(self) -> Vec<u8> {
        let keys = self.keys.and_then(StringTable::into_ordered);
        let values = self.values.and_then(StringTable::into_ordered);
        let tables = keys.iter().chain(&values);

        let mut doc = Vec::with_capacity(
            MAGIC.len()
                + 2
                + tables.clone().map(|t| table::encoded_len(t)).sum::<usize>()
                + self.body.len(),
        );
        doc.extend_from_slice(&MAGIC);
        doc.push(VERSION);
        doc.push(
            keys.as_ref().map_or(0, |_| FLAG_HAS_KEY_TABLE)
                | values.as_ref().map_or(0, |_| FLAG_HAS_VALUE_TABLE),
        );
        tables.for_each(|t| table::write(&mut doc, t));
        doc.extend_from_slice(&self.body);
        doc
    }

    fn write_int(&mut self, value: i64) {
        if let Ok(value) = i16::try_from(value) {
            self.body.push(ValueType::Int16 as u8);
            self.body.extend_from_slice(&value.to_le_bytes());
        } else if let Ok(value) = i32::try_from(value) {
            self.body.push(ValueType::Int32 as u8);
            self.body.extend_from_slice(&value.to_le_bytes());
        } else {
            self.body.push(ValueType::Int64 as u8);
            self.body.extend_from_slice(&value.to_le_bytes());
        }
    }

    fn write_wide_int<T: TryInto<i64> + ToString + Copy>(&mut self, value: T) -> Result<(), Error> {
        let narrowed = value
            .try_into()
            .map_err(|_| ErrorKind::IntegerTooLarge(value.to_string()))?;
        self.write_int(narrowed);
        Ok(())
    }

    fn write_header(&mut self, short: ValueType, long: ValueType, len: usize) -> Result<(), Error> {
        let (header, n) = header(short, long, len)?;
        self.body.extend_from_slice(&header[..n]);
        Ok(())
    }

    /// Write an `f32` or `f64` as a `Decimal` if that takes at most as many bytes as a
    /// `Float64`, and as a `Float64` otherwise (including `NaN` and `±Infinity`).
    fn write_float<F: fmt::LowerExp + Into<f64> + Copy>(&mut self, value: F) {
        match Decimal::shortest(value).filter(|decimal| decimal.encoded_len() <= FLOAT64_LEN) {
            Some(decimal) => {
                self.body.push(ValueType::Decimal as u8);
                varint::write(&mut self.body, decimal.head());
                varint::write64(&mut self.body, decimal.mantissa);
            }
            None => {
                self.body.push(ValueType::Float64 as u8);
                self.body.extend_from_slice(&value.into().to_le_bytes());
            }
        }
    }

    /// Write a string value, as an id into the value table when it is on or inline otherwise.
    /// `owned` is only called when a new string is added to the table.
    fn write_str(
        &mut self,
        value: &str,
        owned: impl FnOnce() -> Cow<'static, str>,
    ) -> Result<(), Error> {
        let Some(values) = &mut self.values else {
            self.write_header(
                ValueType::ShortStringUtf8,
                ValueType::StringUtf8,
                value.len(),
            )?;
            self.body.extend_from_slice(value.as_bytes());
            return Ok(());
        };
        len_u32(value.len())?;
        let id = values
            .intern(value, owned)
            .ok_or(ErrorKind::TooManyValues)?;
        self.body.push(ValueType::InternedString as u8);
        varint::write(&mut self.body, id);
        Ok(())
    }

    /// Write an object key, as a key id when the key table is on or inline otherwise.
    /// `owned` is only called when a new key is added to the table.
    fn write_key(
        &mut self,
        key: &str,
        owned: impl FnOnce() -> Cow<'static, str>,
    ) -> Result<(), Error> {
        let len = len_u32(key.len())?;
        match &mut self.keys {
            Some(keys) => {
                let id = keys.intern(key, owned).ok_or(ErrorKind::TooManyKeys)?;
                varint::write(&mut self.body, id);
            }
            None => {
                varint::write(&mut self.body, len);
                self.body.extend_from_slice(key.as_bytes());
            }
        }
        Ok(())
    }

    /// Read back the key written at `at`. Only used to name map entries in error paths.
    fn key_at(&self, at: usize) -> Option<&str> {
        let input = self.body.get(at..)?;
        let (n, used) = varint::read(input).ok()?;
        match &self.keys {
            Some(keys) => keys.get(n),
            None => std::str::from_utf8(input.get(used..used + n as usize)?).ok(),
        }
    }

    /// Write the `{ "Variant": ` prefix of an externally tagged variant.
    fn write_variant_key(&mut self, variant: &'static str) -> Result<(), Error> {
        self.body.extend([ValueType::ShortObject as u8, 1]);
        self.write_key(variant, || Cow::Borrowed(variant))
    }

    fn begin(
        &mut self,
        container: Container,
        len: Option<usize>,
        variant: Option<&'static str>,
    ) -> Result<Compound<'_>, Error> {
        // When the length is unknown, write a short header for 0 elements and patch it in `end`.
        let declared = len.unwrap_or(0);
        let at = self.body.len();
        let (short, long) = container.tags();
        self.write_header(short, long, declared)?;
        Ok(Compound {
            header_len: self.body.len() - at,
            ser: self,
            container,
            at,
            declared,
            count: 0,
            key_at: 0,
            variant,
        })
    }
}

/// Encode a tag and length, picking the short variant for lengths ≤ 255.
fn header(
    short: ValueType,
    long: ValueType,
    len: usize,
) -> Result<([u8; 1 + varint::MAX_LEN], usize), Error> {
    let mut buf = [0; 1 + varint::MAX_LEN];
    if len <= SHORT_MAX {
        buf[0] = short as u8;
        buf[1] = len as u8;
        return Ok((buf, 2));
    }
    let (len, n) = varint::encode(len_u32(len)?);
    buf[0] = long as u8;
    buf[1..=n].copy_from_slice(&len[..n]);
    Ok((buf, 1 + n))
}

fn len_u32(len: usize) -> Result<u32, Error> {
    u32::try_from(len).map_err(|_| ErrorKind::LengthTooLarge(len).into())
}

#[derive(Debug, Clone, Copy)]
enum Container {
    Array,
    Object,
}

impl Container {
    fn tags(self) -> (ValueType, ValueType) {
        match self {
            Self::Array => (ValueType::ShortArray, ValueType::Array),
            Self::Object => (ValueType::ShortObject, ValueType::Object),
        }
    }
}

/// An array or object being serialized. Its header is fixed up in `end` if the
/// number of elements differs from the length given up front.
#[derive(Debug)]
pub struct Compound<'a> {
    ser: &'a mut Serializer,
    container: Container,
    /// Where the header starts in the body, and how many bytes it takes.
    at: usize,
    header_len: usize,
    /// The count written into the header.
    declared: usize,
    count: usize,
    /// Where the current map key starts in the body.
    key_at: usize,
    /// For tuple and struct variants, the variant name wrapping this container.
    variant: Option<&'static str>,
}

impl Compound<'_> {
    fn element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value
            .serialize(&mut *self.ser)
            .map_err(|err| self.in_variant(err.in_index(self.count)))?;
        self.count += 1;
        Ok(())
    }

    fn field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) -> Result<(), Error> {
        self.ser.write_key(key, || Cow::Borrowed(key))?;
        value
            .serialize(&mut *self.ser)
            .map_err(|err| self.in_variant(err.in_key(key)))?;
        self.count += 1;
        Ok(())
    }

    fn in_variant(&self, err: Error) -> Error {
        match self.variant {
            Some(variant) => err.in_key(variant),
            None => err,
        }
    }

    fn finish(self) -> Result<(), Error> {
        if self.count == self.declared {
            return Ok(());
        }
        let (short, long) = self.container.tags();
        let (header, n) = header(short, long, self.count)?;
        let range = self.at..self.at + self.header_len;
        if n == self.header_len {
            self.ser.body[range].copy_from_slice(&header[..n]);
        } else {
            self.ser.body.splice(range, header[..n].iter().copied());
        }
        Ok(())
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;

    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        self.body.extend([ValueType::Bool as u8, u8::from(v)]);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        self.write_int(v.into());
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        self.write_int(v.into());
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        self.write_int(v.into());
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        self.write_int(v);
        Ok(())
    }

    fn serialize_i128(self, v: i128) -> Result<(), Error> {
        self.write_wide_int(v)
    }

    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        self.write_int(v.into());
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        self.write_int(v.into());
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        self.write_int(v.into());
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        self.write_wide_int(v)
    }

    fn serialize_u128(self, v: u128) -> Result<(), Error> {
        self.write_wide_int(v)
    }

    fn serialize_f32(self, v: f32) -> Result<(), Error> {
        self.write_float(v);
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<(), Error> {
        self.write_float(v);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<(), Error> {
        let mut buf = [0; 4];
        let v = &*v.encode_utf8(&mut buf);
        self.write_str(v, || Cow::Owned(v.to_owned()))
    }

    fn serialize_str(self, v: &str) -> Result<(), Error> {
        self.write_str(v, || Cow::Owned(v.to_owned()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<(), Error> {
        self.write_header(ValueType::ShortByteArray, ValueType::ByteArray, v.len())?;
        self.body.extend_from_slice(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<(), Error> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<(), Error> {
        self.body.push(ValueType::Null as u8);
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        self.write_str(variant, || Cow::Borrowed(variant))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.write_variant_key(variant)?;
        value.serialize(self).map_err(|err| err.in_key(variant))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Compound<'a>, Error> {
        self.begin(Container::Array, len, None)
    }

    fn serialize_tuple(self, len: usize) -> Result<Compound<'a>, Error> {
        self.begin(Container::Array, Some(len), None)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Compound<'a>, Error> {
        self.begin(Container::Array, Some(len), None)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Compound<'a>, Error> {
        self.write_variant_key(variant)?;
        self.begin(Container::Array, Some(len), Some(variant))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Compound<'a>, Error> {
        self.begin(Container::Object, len, None)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Compound<'a>, Error> {
        self.begin(Container::Object, Some(len), None)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Compound<'a>, Error> {
        self.write_variant_key(variant)?;
        self.begin(Container::Object, Some(len), Some(variant))
    }
}

impl ser::SerializeSeq for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.element(value)
    }

    fn end(self) -> Result<(), Error> {
        self.finish()
    }
}

impl ser::SerializeTuple for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.element(value)
    }

    fn end(self) -> Result<(), Error> {
        self.finish()
    }
}

impl ser::SerializeTupleStruct for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.element(value)
    }

    fn end(self) -> Result<(), Error> {
        self.finish()
    }
}

impl ser::SerializeTupleVariant for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        self.element(value)
    }

    fn end(self) -> Result<(), Error> {
        self.finish()
    }
}

impl ser::SerializeMap for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Error> {
        self.key_at = self.ser.body.len();
        key.serialize(MapKeySerializer(self.ser))?;
        self.count += 1;
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Error> {
        value
            .serialize(&mut *self.ser)
            .map_err(|err| match self.ser.key_at(self.key_at) {
                Some(key) => err.in_key(key),
                None => err,
            })
    }

    fn end(self) -> Result<(), Error> {
        self.finish()
    }
}

impl ser::SerializeStruct for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.field(key, value)
    }

    fn end(self) -> Result<(), Error> {
        self.finish()
    }
}

impl ser::SerializeStructVariant for Compound<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.field(key, value)
    }

    fn end(self) -> Result<(), Error> {
        self.finish()
    }
}

/// Writes map keys. Like `serde_json`, integer, `bool` and `char` keys become strings,
/// and unit variants and newtype structs are written as their string form.
struct MapKeySerializer<'a>(&'a mut Serializer);

impl MapKeySerializer<'_> {
    fn write_display(self, value: impl ToString) -> Result<(), Error> {
        let key = value.to_string();
        self.0.write_key(&key, || Cow::Owned(key.clone()))
    }
}

fn invalid_key() -> Error {
    ErrorKind::InvalidMapKey.into()
}

impl ser::Serializer for MapKeySerializer<'_> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_str(self, v: &str) -> Result<(), Error> {
        self.0.write_key(v, || Cow::Owned(v.to_owned()))
    }

    fn serialize_char(self, v: char) -> Result<(), Error> {
        self.serialize_str(v.encode_utf8(&mut [0; 4]))
    }

    fn serialize_bool(self, v: bool) -> Result<(), Error> {
        let key = if v { "true" } else { "false" };
        self.0.write_key(key, || Cow::Borrowed(key))
    }

    fn serialize_i8(self, v: i8) -> Result<(), Error> {
        self.write_display(v)
    }

    fn serialize_i16(self, v: i16) -> Result<(), Error> {
        self.write_display(v)
    }

    fn serialize_i32(self, v: i32) -> Result<(), Error> {
        self.write_display(v)
    }

    fn serialize_i64(self, v: i64) -> Result<(), Error> {
        self.write_display(v)
    }

    fn serialize_i128(self, v: i128) -> Result<(), Error> {
        self.write_display(v)
    }

    fn serialize_u8(self, v: u8) -> Result<(), Error> {
        self.write_display(v)
    }

    fn serialize_u16(self, v: u16) -> Result<(), Error> {
        self.write_display(v)
    }

    fn serialize_u32(self, v: u32) -> Result<(), Error> {
        self.write_display(v)
    }

    fn serialize_u64(self, v: u64) -> Result<(), Error> {
        self.write_display(v)
    }

    fn serialize_u128(self, v: u128) -> Result<(), Error> {
        self.write_display(v)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Error> {
        self.0.write_key(variant, || Cow::Borrowed(variant))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        value.serialize(self)
    }

    fn serialize_f32(self, _v: f32) -> Result<(), Error> {
        Err(invalid_key())
    }

    fn serialize_f64(self, _v: f64) -> Result<(), Error> {
        Err(invalid_key())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<(), Error> {
        Err(invalid_key())
    }

    fn serialize_none(self) -> Result<(), Error> {
        Err(invalid_key())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<(), Error> {
        Err(invalid_key())
    }

    fn serialize_unit(self) -> Result<(), Error> {
        Err(invalid_key())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Error> {
        Err(invalid_key())
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<(), Error> {
        Err(invalid_key())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        Err(invalid_key())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Error> {
        Err(invalid_key())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Error> {
        Err(invalid_key())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Error> {
        Err(invalid_key())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        Err(invalid_key())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Error> {
        Err(invalid_key())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Error> {
        Err(invalid_key())
    }
}
