use std::{fmt, io};

use crate::ValueType;

/// An error from serializing or deserializing Specta Binary.
///
/// Deserialization errors carry the path to the value that failed (like `$.users[3].email`)
/// and the byte offset where it starts, so they read like
/// `at $.users[3].email: expected string, found Int32 (offset 412)`.
#[derive(Debug)]
pub struct Error(Box<Inner>);

#[derive(Debug)]
struct Inner {
    kind: ErrorKind,
    /// While an error bubbles up this holds the path relative to the current value.
    /// [`Error::rooted`] prefixes it with `$` once it reaches the root.
    path: String,
    rooted: bool,
    offset: Option<usize>,
}

/// What went wrong.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ErrorKind {
    /// A custom error from a `Serialize` or `Deserialize` implementation.
    #[error("{0}")]
    Message(String),
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("invalid magic bytes, expected \"SB\"")]
    InvalidMagic,
    #[error("unsupported version {0}")]
    UnsupportedVersion(u8),
    #[error("unknown header flags {0:#04x}")]
    UnknownFlags(u8),
    #[error("unknown value type tag {0:#04x}")]
    UnknownTag(u8),
    #[error("invalid bool byte {0:#04x}")]
    InvalidBool(u8),
    #[error("varint is too long or overflows")]
    VarintOverflow,
    #[error("length {len} exceeds the {remaining} remaining bytes")]
    LengthOutOfBounds { len: u32, remaining: usize },
    #[error("key id {id} is out of range for a key table of {count} keys")]
    KeyIdOutOfRange { id: u32, count: usize },
    #[error("value id {id} is out of range for a value table of {count} values")]
    ValueIdOutOfRange { id: u32, count: usize },
    #[error("invalid UTF-8")]
    InvalidUtf8,
    #[error("maximum nesting depth of {0} exceeded")]
    DepthLimitExceeded(usize),
    #[error("trailing bytes after the root value")]
    TrailingBytes,
    #[error("{0} unexpected trailing elements")]
    TrailingElements(usize),
    #[error("expected {expected}, found {found}")]
    TypeMismatch {
        expected: &'static str,
        found: ValueType,
    },
    #[error("expected an enum object with exactly 1 entry, found {0} entries")]
    InvalidEnumObject(usize),
    #[error("integer {value} is out of range for {target}")]
    IntegerOutOfRange { value: i64, target: &'static str },
    #[error("integer {0} does not fit in i64")]
    IntegerTooLarge(String),
    #[error("map keys must be strings, integers, bools or chars")]
    InvalidMapKey,
    #[error("length {0} does not fit in 32 bits")]
    LengthTooLarge(usize),
    #[error("declared a length of {declared} but serialized {actual} elements")]
    LengthMismatch { declared: usize, actual: usize },
    #[error("too many distinct object keys for the key table")]
    TooManyKeys,
    #[error("too many distinct strings for the value table")]
    TooManyValues,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self(Box::new(Inner {
            kind,
            path: String::new(),
            rooted: false,
            offset: None,
        }))
    }

    pub(crate) fn at(kind: ErrorKind, offset: usize) -> Self {
        Self::new(kind).or_offset(offset)
    }

    /// What went wrong.
    pub fn kind(&self) -> &ErrorKind {
        &self.0.kind
    }

    /// The path to the value that failed, like `$.users[3].email`.
    /// `None` for errors outside the root value, such as an invalid header.
    pub fn path(&self) -> Option<&str> {
        self.0.rooted.then_some(self.0.path.as_str())
    }

    /// The byte offset in the document where the failing value (or header field) starts.
    /// Always `None` for serialization errors.
    pub fn offset(&self) -> Option<usize> {
        self.0.offset
    }

    /// Set the offset unless a more precise one was already recorded deeper down.
    pub(crate) fn or_offset(mut self, offset: usize) -> Self {
        self.0.offset.get_or_insert(offset);
        self
    }

    pub(crate) fn in_key(mut self, key: &str) -> Self {
        // Matches `/^[A-Za-z_$][A-Za-z0-9_$]*$/` in the TypeScript runtime.
        let is_ident = key.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '$')
            && key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
        let segment = if is_ident {
            format!(".{key}")
        } else {
            format!("[{key:?}]")
        };
        self.0.path.insert_str(0, &segment);
        self
    }

    pub(crate) fn in_index(mut self, index: usize) -> Self {
        self.0.path.insert_str(0, &format!("[{index}]"));
        self
    }

    pub(crate) fn rooted(mut self) -> Self {
        if !self.0.rooted {
            self.0.path.insert(0, '$');
            self.0.rooted = true;
        }
        self
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self::new(kind)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::new(ErrorKind::Io(err))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = self.path() {
            write!(f, "at {path}: ")?;
        }
        write!(f, "{}", self.0.kind)?;
        if let Some(offset) = self.0.offset {
            write!(f, " (offset {offset})")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0.kind {
            ErrorKind::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::new(ErrorKind::Message(msg.to_string()))
    }
}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::new(ErrorKind::Message(msg.to_string()))
    }
}
