//! Types related to working with [`DataType`]. Exposed for advanced users.

mod attributes;
mod constant;
mod r#enum;
mod fields;
mod function;
mod list;
mod map;
mod named;
mod primitive;
mod reference;
mod r#struct;
mod tuple;

pub use attributes::Attributes;
pub use constant::{ConstantValue, FloatBits};
pub use r#enum::{Enum, Variant, VariantBuilder};
pub use fields::{Field, Fields, NamedFields, StructBuilder, UnnamedFields};
pub use function::Function;
pub use list::List;
pub use map::Map;
pub use named::{Deprecated, NamedDataType};
pub use primitive::Primitive;
pub use reference::{GenericReference, NamedReference, OpaqueReference, Reference};
pub use r#struct::Struct;
pub use tuple::Tuple;

/// Opaque marker representing a dynamically-typed "any" value.
///
/// Used as `DataType::Reference(Reference::opaque(AnyValue))` by types like
/// `serde_json::Value` whose concrete shape is not known at compile time.
/// Language exporters should map this to their native "any" representation
/// (e.g. `any` in TypeScript, `z.any()` in Zod, `{}` in JSON Schema).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnyValue;

pub(crate) use reference::NamedId;

/// Runtime type-erased representation of a Rust type.
///
/// A language exporter takes this general format and converts it into a language specific syntax.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataType {
    /// A primitive scalar type like integers, floats, booleans, chars, or strings.
    Primitive(Primitive),
    /// A sequential collection type.
    List(List),
    /// A map/dictionary type.
    Map(Map),
    /// A struct type with named, unnamed, or unit fields.
    Struct(Struct),
    /// An enum type.
    Enum(Enum),
    /// A tuple type.
    Tuple(Tuple),
    /// A nullable wrapper around another type.
    Nullable(Box<DataType>),
    /// A reference to another named or opaque type.
    Reference(Reference),
    /// A compile-time constant value used as a type (e.g. a string literal type).
    ///
    /// This will typically not be constructed directly in most languages and exists
    /// for outputting tagged enums in languages like TypeScript that accept constant
    /// values as types (e.g. `"variant_name"` as a type).
    Constant(ConstantValue),
}
