# Functions in specta_serde

```rust
/**
`apply` -- Applies serde transformations in unified mode.

Unified mode produces a single transformed type graph that must satisfy both
serialization and deserialization behavior. This is the simplest mode and is
usually what exporters want when serde behavior is symmetric.

Returns [`ResolvedTypes`] because serde rewrites may
alter type shapes.

Returns an [`Error`] when serde metadata introduces phase-only differences
that cannot be represented as one shape (for example `#[serde(other)]`,
identifier enums, asymmetric conversion attributes, `skip_serializing_if`,
or explicit [`Phased`] overrides).

Use [`apply_phases`] when your serialize and deserialize wire shapes differ.
*/
pub fn apply(types: specta::Types) -> error::Result<specta::ResolvedTypes>
/**
`apply_phases` -- Applies serde transformations in split-phase mode.

Phase mode preserves directional differences by rewriting affected named
types into paired `*_Serialize` and `*_Deserialize` types and then updating
references accordingly. This allows exporters to represent serde behavior
that is asymmetric between serialization and deserialization.

Returns [`ResolvedTypes`] because serde rewrites may
alter type shapes.

Use this when working with deserialize-widening attributes like
`#[serde(other)]`/identifier enums, asymmetric conversion attributes, or
explicit [`Phased`] overrides.
*/
pub fn apply_phases(types: specta::Types) -> error::Result<specta::ResolvedTypes>
/**
`select_phase_datatype` -- Rewrites a [`DataType`] to the requested directional shape after [`apply_phases`].

This is useful for exporter integrations that need deserialize-specific input
types and serialize-specific output types while still exporting against the
resolved type graph returned by [`apply_phases`].

# Examples

```rust
use serde::{Deserialize, Serialize};
use specta::{Type, Types, datatype::{DataType, Reference}};
use specta_serde::{Phase, Phased, apply_phases, select_phase_datatype};

#[derive(Type, Serialize, Deserialize)]
#[serde(untagged)]
enum OneOrManyString {
    One(String),
    Many(Vec<String>),
}

#[derive(Type, Serialize, Deserialize)]
struct Filters {
    #[specta(type = Phased<Vec<String>, OneOrManyString>)]
    tags: Vec<String>,
}

let mut types = Types::default();
let dt = Filters::definition(&mut types);
let resolved = apply_phases(types)?;

let serialize = select_phase_datatype(&dt, &resolved, Phase::Serialize);
let deserialize = select_phase_datatype(&dt, &resolved, Phase::Deserialize);

let DataType::Reference(Reference::Named(serialize_reference)) = &serialize else {
    panic!("expected named serialize reference");
};
let DataType::Reference(Reference::Named(deserialize_reference)) = &deserialize else {
    panic!("expected named deserialize reference");
};

assert_eq!(
    serialize_reference.get(resolved.as_types()).unwrap().name(),
    "Filters_Serialize"
);
assert_eq!(
    deserialize_reference.get(resolved.as_types()).unwrap().name(),
    "Filters_Deserialize"
);
# Ok::<(), specta_serde::Error>(())
```
*/
pub fn select_phase_datatype(dt: &specta::datatype::DataType, types: &specta::ResolvedTypes, phase: Phase) -> specta::datatype::DataType
/**
`validate` -- Validates whether a given [`DataType`] is a valid Serde-type.

When using [`apply`]/[`apply_phases`] all [`NamedDataType`]s are validated automatically, however if you need to export a [`DataType`] directly this is required to validate the top-level type.

For example if you try and export `HashMap<InvalidKey, MyGenericType<()>>`, [`apply`]/[`apply_phases`] can validate `MyGenericType` but it doesn't see the top-level `HashMap`'s generics so it can't validate them.

This is *only* required if your using the primitives from your language exporter.
*/
pub fn validate(dt: &specta::datatype::DataType, types: &specta::ResolvedTypes) -> error::Result<()>
```

