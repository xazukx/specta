//! This module contains functions that are public for the sole reason of the macros.
//!
//! They will not be documented and may go through breaking changes without a major version bump!
//!
//! DO NOT USE THEM! You have been warned!

#[cfg(feature = "function")]
pub use paste::paste;

// --- Constant value conversion traits (used by #[derive(Constant)] macro) ---

use crate::datatype::{ConstantValue, FloatBits};
use std::borrow::Cow;

/// Trait to convert a Rust value into a [`ConstantValue`] using the default/inferred format.
pub trait IntoConstantValue {
    fn into_constant_value(&self) -> ConstantValue;
}

/// Trait to convert a Rust value into a [`ConstantValue`] forced as a string.
pub trait IntoConstantValueAsString {
    fn into_constant_value_as_string(&self) -> ConstantValue;
}

/// Trait to convert a Rust value into a [`ConstantValue`] forced as bytes.
pub trait IntoConstantValueAsBytes {
    fn into_constant_value_as_bytes(&self) -> ConstantValue;
}

// --- IntoConstantValue implementations ---

impl IntoConstantValue for &str {
    fn into_constant_value(&self) -> ConstantValue {
        ConstantValue::String(Cow::Owned((*self).to_owned()))
    }
}

impl IntoConstantValue for str {
    fn into_constant_value(&self) -> ConstantValue {
        ConstantValue::String(Cow::Owned(self.to_owned()))
    }
}

impl IntoConstantValue for String {
    fn into_constant_value(&self) -> ConstantValue {
        ConstantValue::String(Cow::Owned(self.clone()))
    }
}

impl IntoConstantValue for bool {
    fn into_constant_value(&self) -> ConstantValue {
        ConstantValue::Bool(*self)
    }
}

impl IntoConstantValue for () {
    fn into_constant_value(&self) -> ConstantValue {
        ConstantValue::Null
    }
}

macro_rules! impl_into_constant_signed {
    ($($ty:ty),*) => {
        $(
            impl IntoConstantValue for $ty {
                fn into_constant_value(&self) -> ConstantValue {
                    ConstantValue::Integer(*self as i128)
                }
            }

            impl IntoConstantValueAsString for $ty {
                fn into_constant_value_as_string(&self) -> ConstantValue {
                    ConstantValue::String(Cow::Owned(self.to_string()))
                }
            }
        )*
    };
}

macro_rules! impl_into_constant_unsigned {
    ($($ty:ty),*) => {
        $(
            impl IntoConstantValue for $ty {
                fn into_constant_value(&self) -> ConstantValue {
                    ConstantValue::UnsignedInteger(*self as u128)
                }
            }

            impl IntoConstantValueAsString for $ty {
                fn into_constant_value_as_string(&self) -> ConstantValue {
                    ConstantValue::String(Cow::Owned(self.to_string()))
                }
            }
        )*
    };
}

impl_into_constant_signed!(i8, i16, i32, i64, i128, isize);
impl_into_constant_unsigned!(u8, u16, u32, u64, u128, usize);

impl IntoConstantValue for f32 {
    fn into_constant_value(&self) -> ConstantValue {
        ConstantValue::Float(FloatBits::from_f32(*self))
    }
}

impl IntoConstantValue for f64 {
    fn into_constant_value(&self) -> ConstantValue {
        ConstantValue::Float(FloatBits::from_f64(*self))
    }
}

impl IntoConstantValueAsString for f32 {
    fn into_constant_value_as_string(&self) -> ConstantValue {
        ConstantValue::String(Cow::Owned(self.to_string()))
    }
}

impl IntoConstantValueAsString for f64 {
    fn into_constant_value_as_string(&self) -> ConstantValue {
        ConstantValue::String(Cow::Owned(self.to_string()))
    }
}

impl IntoConstantValueAsString for &str {
    fn into_constant_value_as_string(&self) -> ConstantValue {
        ConstantValue::String(Cow::Owned((*self).to_owned()))
    }
}

impl IntoConstantValueAsString for str {
    fn into_constant_value_as_string(&self) -> ConstantValue {
        ConstantValue::String(Cow::Owned(self.to_owned()))
    }
}

impl IntoConstantValueAsString for String {
    fn into_constant_value_as_string(&self) -> ConstantValue {
        ConstantValue::String(Cow::Owned(self.clone()))
    }
}

impl IntoConstantValueAsString for bool {
    fn into_constant_value_as_string(&self) -> ConstantValue {
        ConstantValue::String(Cow::Owned(self.to_string()))
    }
}

// Byte slices: default -> Bytes, string -> try UTF-8, bytes -> Bytes
impl<const N: usize> IntoConstantValue for [u8; N] {
    fn into_constant_value(&self) -> ConstantValue {
        ConstantValue::Bytes(Cow::Owned(self.to_vec()))
    }
}

impl<const N: usize> IntoConstantValue for &[u8; N] {
    fn into_constant_value(&self) -> ConstantValue {
        ConstantValue::Bytes(Cow::Owned(self.to_vec()))
    }
}

impl IntoConstantValue for [u8] {
    fn into_constant_value(&self) -> ConstantValue {
        ConstantValue::Bytes(Cow::Owned(self.to_vec()))
    }
}

impl<const N: usize> IntoConstantValueAsString for [u8; N] {
    fn into_constant_value_as_string(&self) -> ConstantValue {
        ConstantValue::String(Cow::Owned(String::from_utf8_lossy(self).into_owned()))
    }
}

impl<const N: usize> IntoConstantValueAsString for &[u8; N] {
    fn into_constant_value_as_string(&self) -> ConstantValue {
        ConstantValue::String(Cow::Owned(String::from_utf8_lossy(*self).into_owned()))
    }
}

impl IntoConstantValueAsString for [u8] {
    fn into_constant_value_as_string(&self) -> ConstantValue {
        ConstantValue::String(Cow::Owned(String::from_utf8_lossy(self).into_owned()))
    }
}

impl<const N: usize> IntoConstantValueAsBytes for [u8; N] {
    fn into_constant_value_as_bytes(&self) -> ConstantValue {
        ConstantValue::Bytes(Cow::Owned(self.to_vec()))
    }
}

impl<const N: usize> IntoConstantValueAsBytes for &[u8; N] {
    fn into_constant_value_as_bytes(&self) -> ConstantValue {
        ConstantValue::Bytes(Cow::Owned(self.to_vec()))
    }
}

impl IntoConstantValueAsBytes for [u8] {
    fn into_constant_value_as_bytes(&self) -> ConstantValue {
        ConstantValue::Bytes(Cow::Owned(self.to_vec()))
    }
}

#[cfg(feature = "function")]
mod functions {
    use std::borrow::Cow;

    use crate::{Types, datatype::Deprecated, datatype::Function, function::SpectaFn};

    #[doc(hidden)]
    /// A helper for exporting a command to a [`CommandDataType`].
    /// You shouldn't use this directly and instead should use [`fn_datatype!`](crate::fn_datatype).
    pub fn get_fn_datatype<TMarker, T: SpectaFn<TMarker>>(
        _: T,
        asyncness: bool,
        name: Cow<'static, str>,
        types: &mut Types,
        fields: &[Cow<'static, str>],
        docs: Cow<'static, str>,
        deprecated: Option<Deprecated>,
        no_return_type: bool,
    ) -> Function {
        T::to_datatype(
            asyncness,
            name,
            types,
            fields,
            docs,
            deprecated,
            no_return_type,
        )
    }
}
#[cfg(feature = "function")]
pub use functions::*;
