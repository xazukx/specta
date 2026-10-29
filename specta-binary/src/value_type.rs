use std::fmt;

/// The two magic bytes every document starts with (`"SB"`).
pub const MAGIC: [u8; 2] = *b"SB";
/// The format version written by this crate. Readers reject any other version.
pub const VERSION: u8 = 0x01;
/// Header flag: a key table follows the header and object keys are key ids.
pub const FLAG_HAS_KEY_TABLE: u8 = 0b0000_0001;
/// Every header flag this version understands. Any other bit set is an error.
pub const KNOWN_FLAGS: u8 = FLAG_HAS_KEY_TABLE;

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
    Int32 = 10,
    Int64 = 15,
    Float32 = 20,
    Float64 = 25,
    StringUtf8 = 30,
    ShortStringUtf8 = 35,
    ByteArray = 40,
    ShortByteArray = 45,
    Array = 50,
    ShortArray = 55,
    Object = 60,
    ShortObject = 65,
}

impl ValueType {
    /// Every tag, in ascending order.
    pub const ALL: [ValueType; 14] = [
        Self::Null,
        Self::Bool,
        Self::Int32,
        Self::Int64,
        Self::Float32,
        Self::Float64,
        Self::StringUtf8,
        Self::ShortStringUtf8,
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
            Self::Int32 => "Int32",
            Self::Int64 => "Int64",
            Self::Float32 => "Float32",
            Self::Float64 => "Float64",
            Self::StringUtf8 => "StringUtf8",
            Self::ShortStringUtf8 => "ShortStringUtf8",
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
            10 => Self::Int32,
            15 => Self::Int64,
            20 => Self::Float32,
            25 => Self::Float64,
            30 => Self::StringUtf8,
            35 => Self::ShortStringUtf8,
            40 => Self::ByteArray,
            45 => Self::ShortByteArray,
            50 => Self::Array,
            55 => Self::ShortArray,
            60 => Self::Object,
            65 => Self::ShortObject,
            _ => return Err(byte),
        })
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
