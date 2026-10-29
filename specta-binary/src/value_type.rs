use std::fmt;

/// The two magic bytes every document starts with (`"SB"`).
pub const MAGIC: [u8; 2] = *b"SB";
/// The format version written by this crate. Readers reject any other version.
pub const VERSION: u8 = 0x01;
/// Header flag: a key table follows the header and object keys are key ids.
pub const FLAG_HAS_KEY_TABLE: u8 = 0b0000_0001;
/// Header flag: a value table follows the header (after the key table, if any), and
/// `InternedString` values are ids into it.
pub const FLAG_HAS_VALUE_TABLE: u8 = 0b0000_0010;
/// Every header flag this version understands. Any other bit set is an error.
pub const KNOWN_FLAGS: u8 = FLAG_HAS_KEY_TABLE | FLAG_HAS_VALUE_TABLE;

/// The largest length or count that uses the short (1-byte length) tag variants.
pub(crate) const SHORT_MAX: usize = u8::MAX as usize;

/// The one-byte tag written before every value.
///
/// The gaps between the values are reserved for future tags.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    Null = 0,
    Bool = 5,
    Int16 = 10,
    Int32 = 15,
    Int64 = 20,
    Decimal = 25,
    Float64 = 30,
    StringUtf8 = 35,
    ShortStringUtf8 = 40,
    InternedString = 45,
    ByteArray = 50,
    ShortByteArray = 55,
    Array = 60,
    ShortArray = 65,
    Object = 70,
    ShortObject = 75,
}

impl ValueType {
    /// Every tag, in ascending order.
    pub const ALL: [ValueType; 16] = [
        Self::Null,
        Self::Bool,
        Self::Int16,
        Self::Int32,
        Self::Int64,
        Self::Decimal,
        Self::Float64,
        Self::StringUtf8,
        Self::ShortStringUtf8,
        Self::InternedString,
        Self::ByteArray,
        Self::ShortByteArray,
        Self::Array,
        Self::ShortArray,
        Self::Object,
        Self::ShortObject,
    ];

    /// Whether this is a short variant, whose length is a single `u8` instead of a varint.
    pub const fn is_short(self) -> bool {
        matches!(
            self,
            Self::ShortStringUtf8 | Self::ShortByteArray | Self::ShortArray | Self::ShortObject
        )
    }

    /// The tag's name, matching the key in the TypeScript runtime's `ValueType` object.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Null => "Null",
            Self::Bool => "Bool",
            Self::Int16 => "Int16",
            Self::Int32 => "Int32",
            Self::Int64 => "Int64",
            Self::Decimal => "Decimal",
            Self::Float64 => "Float64",
            Self::StringUtf8 => "StringUtf8",
            Self::ShortStringUtf8 => "ShortStringUtf8",
            Self::InternedString => "InternedString",
            Self::ByteArray => "ByteArray",
            Self::ShortByteArray => "ShortByteArray",
            Self::Array => "Array",
            Self::ShortArray => "ShortArray",
            Self::Object => "Object",
            Self::ShortObject => "ShortObject",
        }
    }
}

impl TryFrom<u8> for ValueType {
    /// The unknown tag byte.
    type Error = u8;

    fn try_from(byte: u8) -> Result<Self, u8> {
        Ok(match byte {
            0 => Self::Null,
            5 => Self::Bool,
            10 => Self::Int16,
            15 => Self::Int32,
            20 => Self::Int64,
            25 => Self::Decimal,
            30 => Self::Float64,
            35 => Self::StringUtf8,
            40 => Self::ShortStringUtf8,
            45 => Self::InternedString,
            50 => Self::ByteArray,
            55 => Self::ShortByteArray,
            60 => Self::Array,
            65 => Self::ShortArray,
            70 => Self::Object,
            75 => Self::ShortObject,
            _ => return Err(byte),
        })
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
