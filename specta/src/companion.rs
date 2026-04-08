/// A trait implemented by generated field enums.
///
/// Each variant of the field enum corresponds to a non-skipped field of the
/// original struct, with names derived from the serde-resolved field names.
pub trait CompanionField: Copy + Clone + Eq + core::hash::Hash {
    /// The serde-resolved field name (e.g. `"user_name"`).
    fn name(&self) -> &'static str;

    /// The Rust type as a string (e.g. `"String"`).
    fn type_str(&self) -> &'static str;

    /// Human-readable title. Defaults to [`name()`](CompanionField::name).
    fn title(&self) -> &'static str {
        self.name()
    }

    /// Ordering hint for UI rendering. Defaults to `0`.
    fn order(&self) -> isize {
        0
    }
}

/// A trait implemented by generated value enums.
///
/// Each variant wraps the value of a non-skipped field, enabling dynamic
/// field access and updates at runtime.
pub trait CompanionValue {
    /// The field name this value corresponds to.
    fn field_name(&self) -> &'static str;

    /// The Rust type name of the inner value.
    fn type_name(&self) -> &'static str;
}

/// The main companion trait, implemented on the original struct.
///
/// Provides dynamic field access, mutation, and enumeration through the
/// generated `F` (field) and `V` (value) companion types.
pub trait TypeCompanion<F, V>
where
    F: Copy + 'static,
{
    /// Returns the value of a specific field.
    fn value(&self, field: F) -> V;

    /// Updates the value of a specific field.
    fn update(&mut self, value: V);

    /// Returns an array of all field enum variants.
    fn fields() -> &'static [F];

    /// Returns a vector of all field values.
    fn as_values(&self) -> Vec<V>;
}
