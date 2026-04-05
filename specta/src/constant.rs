use std::borrow::Cow;

use crate::{Constants, datatype::ConstantValue};

/// Trait for types that represent a compile-time constant exportable to TypeScript.
pub trait Constant {
    /// The name to use in the TypeScript export.
    fn name() -> Cow<'static, str>;

    /// The resolved constant value.
    fn value() -> ConstantValue;

    /// Documentation comments on this constant.
    fn docs() -> Cow<'static, str> {
        Cow::Borrowed("")
    }

    /// Whether this constant is deprecated.
    fn deprecated() -> Option<crate::datatype::Deprecated> {
        None
    }

    /// The Rust module path where this constant is defined.
    fn module_path() -> Cow<'static, str> {
        Cow::Borrowed("")
    }

    /// Register this constant into a [`Constants`] collection.
    fn register(constants: &mut Constants) {
        constants.push(NamedConstant {
            name: Self::name(),
            value: Self::value(),
            docs: Self::docs(),
            deprecated: Self::deprecated(),
            module_path: Self::module_path(),
        });
    }
}

/// A named constant that can be exported by language exporters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedConstant {
    /// Export name (the TypeScript variable name).
    pub name: Cow<'static, str>,
    /// The resolved constant value.
    pub value: ConstantValue,
    /// Documentation comments.
    pub docs: Cow<'static, str>,
    /// Deprecation metadata.
    pub deprecated: Option<crate::datatype::Deprecated>,
    /// Source module path.
    pub module_path: Cow<'static, str>,
}
