# Enum `RenameRule` -- path: `specta_serde::inflection::RenameRule`

The different possible ways to change case of fields in a struct, or variants in an enum.

```rust
pub enum RenameRule {
	/// Don't apply a default rename rule.
	None,
	/// Rename direct children to "lowercase" style.
	LowerCase,
	/// Rename direct children to "UPPERCASE" style.
	UpperCase,
	/// Rename direct children to "PascalCase" style, as typically used for
	/// enum variants.
	PascalCase,
	/// Rename direct children to "camelCase" style.
	CamelCase,
	/// Rename direct children to "snake_case" style, as commonly used for
	/// fields.
	SnakeCase,
	/// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
	/// used for constants.
	ScreamingSnakeCase,
	/// Rename direct children to "kebab-case" style.
	KebabCase,
	/// Rename direct children to "SCREAMING-KEBAB-CASE" style.
	ScreamingKebabCase,
}
```

## Methods

```rust
/**
`from_str` -- Parse serde's `rename_all` / `rename_all_fields` rule string.
*/
pub fn from_str(rename_all_str: &str) -> Result<Self, ParseError<''_>>
/**
`apply_to_variant` -- Apply a renaming rule to an enum variant, returning the version expected in the source.
*/
pub fn apply_to_variant(self: Self, variant: &str) -> String
/**
`apply_to_field` -- Apply a renaming rule to a struct field, returning the version expected in the source.
*/
pub fn apply_to_field(self: Self, field: &str) -> String
/**
`or` -- Returns the `RenameRule` if it is not `None`, `rule_b` otherwise.
*/
pub fn or(self: Self, rule_b: Self) -> Self
```

