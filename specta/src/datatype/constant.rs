use std::borrow::Cow;

/// A compile-time constant value that can be exported by language exporters.
///
/// This represents the runtime value of a Rust constant, allowing exporters
/// to emit `export const X = ... as const;` style declarations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstantValue {
    /// A string constant.
    String(Cow<'static, str>),
    /// A signed integer constant.
    Integer(i128),
    /// An unsigned integer constant.
    UnsignedInteger(u128),
    /// A floating-point constant stored as raw bits for Eq/Hash compatibility.
    Float(FloatBits),
    /// A boolean constant.
    Bool(bool),
    /// A byte array constant.
    Bytes(Cow<'static, [u8]>),
    /// A null/unit constant.
    Null,
}

/// Wrapper around f64 bits that implements Eq and Hash.
#[derive(Debug, Clone, Copy)]
pub struct FloatBits(u64);

impl FloatBits {
    /// Create from an f64 value.
    pub fn from_f64(v: f64) -> Self {
        Self(v.to_bits())
    }

    /// Create from an f32 value.
    pub fn from_f32(v: f32) -> Self {
        Self((v as f64).to_bits())
    }

    /// Get the f64 value.
    pub fn to_f64(self) -> f64 {
        f64::from_bits(self.0)
    }
}

impl PartialEq for FloatBits {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for FloatBits {}

impl std::hash::Hash for FloatBits {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl From<&'static str> for ConstantValue {
    fn from(s: &'static str) -> Self {
        Self::String(Cow::Borrowed(s))
    }
}

impl From<String> for ConstantValue {
    fn from(s: String) -> Self {
        Self::String(Cow::Owned(s))
    }
}

impl From<bool> for ConstantValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}
