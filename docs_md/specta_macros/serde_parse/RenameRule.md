# Enum `RenameRule` -- path: `specta_macros::serde_parse::RenameRule`

Supported serde rename rules.

```rust
pub enum RenameRule {
	Lower,
	Upper,
	Pascal,
	Camel,
	Snake,
	ScreamingSnake,
	Kebab,
	ScreamingKebab,
}
```

## Methods

```rust

pub fn parse(lit: &LitStr) -> Result<Self>
/**
`apply` -- Apply this rename rule to a Rust identifier name.
*/
pub fn apply(self: Self, name: &str) -> String

pub fn as_str(self: Self) -> &str
```

