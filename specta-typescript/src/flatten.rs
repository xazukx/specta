//! Shared handling of `#[serde(flatten)]` members, used by both the TypeScript and Zod renderers.
//!
//! `#[serde(flatten)]` on an `Option<T>` contributes nothing when the value is `None` and merges
//! `T`'s fields into the parent object when it is `Some`. Rendering it as a plain intersection
//! member is wrong in both directions: serde never emits a bare `null` for the whole value, and the
//! legitimate `None` output (the base fields alone) does not satisfy an intersection that requires
//! `T`'s fields. Instead each optional part doubles the number of shapes the parent can take, so
//! the parent is rendered as a union over every subset of the optional parts.

use specta::datatype::DataType;

/// A `#[serde(flatten)]`ed member of an object type.
pub(crate) struct FlattenedPart {
    /// The already-rendered type (or Zod schema) of the part.
    pub(crate) rendered: String,
    /// Whether serde may omit the part entirely, which is the case for `#[serde(flatten)]` on an
    /// `Option<T>`.
    pub(crate) optional: bool,
}

/// Peels the `Nullable` layers off a `#[serde(flatten)]`ed field's type.
///
/// The flattened part is the inner type, and `None` makes it absent rather than `null`.
/// `Option<Option<T>>` is flattened identically to `Option<T>`, hence the loop.
pub(crate) fn peel_nullable(ty: &DataType) -> (&DataType, bool) {
    let mut ty = ty;
    let mut optional = false;

    while let DataType::Nullable(inner) = ty {
        ty = inner;
        optional = true;
    }

    (ty, optional)
}

/// Removes duplicate flattened parts while preserving source order.
///
/// A part that is flattened both optionally and mandatorily is mandatory overall.
pub(crate) fn dedup(parts: Vec<FlattenedPart>) -> Vec<FlattenedPart> {
    let mut deduped = Vec::<FlattenedPart>::with_capacity(parts.len());

    for part in parts {
        match deduped
            .iter_mut()
            .find(|other| other.rendered == part.rendered)
        {
            Some(other) => other.optional &= part.optional,
            None => deduped.push(part),
        }
    }

    deduped
}

/// Enumerates the shapes serde can produce for `base` intersected with `parts`: one shape per
/// subset of the optional parts, each listing the rendered members that are present in it.
///
/// Shapes are ordered most-present first, so one optional part yields `[[base, part], [base]]` and
/// two independent ones `[[base, a, b], [base, b], [base, a], [base]]`. Mandatory parts are in
/// every shape, and members keep their source order within a shape.
pub(crate) fn shapes<'a>(base: Option<&'a str>, parts: &'a [FlattenedPart]) -> Vec<Vec<&'a str>> {
    let optional = parts
        .iter()
        .enumerate()
        .filter(|(_, part)| part.optional)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();

    (0..1usize << optional.len())
        .rev()
        .map(|present| {
            base.into_iter()
                .chain(parts.iter().enumerate().filter_map(|(i, part)| {
                    optional
                        .iter()
                        .position(|&j| j == i)
                        .is_none_or(|bit| present & (1 << bit) != 0)
                        .then_some(part.rendered.as_str())
                }))
                .collect()
        })
        .collect()
}
