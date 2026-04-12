# Enum `DataType` -- path: `specta::datatype::DataType`

Runtime type-erased representation of a Rust type.

A language exporter takes this general format and converts it into a language specific syntax.

```rust
pub enum DataType {
	/// A primitive scalar type like integers, floats, booleans, chars, or strings.
	/// [Primitive](./primitive/Primitive.md)
	Primitive(Primitive),
	/// A sequential collection type.
	/// [List](./list/List.md)
	List(List),
	/// A map/dictionary type.
	/// [Map](./map/Map.md)
	Map(Map),
	/// A struct type with named, unnamed, or unit fields.
	/// [Struct](./struct/Struct.md)
	Struct(Struct),
	/// An enum type.
	/// [Enum](./enum/Enum.md)
	Enum(Enum),
	/// A tuple type.
	/// [Tuple](./tuple/Tuple.md)
	Tuple(Tuple),
	/// A nullable wrapper around another type.
	/// [DataType](./DataType.md)
	Nullable(Box<DataType>),
	/// A reference to another named or opaque type.
	/// [Reference](./reference/Reference.md)
	Reference(Reference),
	/// A compile-time constant value used as a type (e.g. a string literal type).
	/// 
	/// This will typically not be constructed directly in most languages and exists
	/// for outputting tagged enums in languages like TypeScript that accept constant
	/// values as types (e.g. `"variant_name"` as a type).
	/// [ConstantValue](./constant/ConstantValue.md)
	Constant(ConstantValue),
}
```
