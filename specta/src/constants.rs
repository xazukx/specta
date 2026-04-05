use crate::{Constant, constant::NamedConstant};

/// A collection of constants to export, analogous to [`crate::Types`] for type declarations.
///
/// # Examples
///
/// ```ignore
/// use specta::Constants;
///
/// let constants = Constants::default()
///     .register::<MyApiVersion>()
///     .register::<MaxRetries>();
/// ```
#[derive(Default, Debug, Clone)]
pub struct Constants(Vec<NamedConstant>);

impl Constants {
    /// Register a [`Constant`] with the collection.
    pub fn register<C: Constant>(mut self) -> Self {
        C::register(&mut self);
        self
    }

    /// Register a [`Constant`] with the collection by mutable reference.
    pub fn register_mut<C: Constant>(&mut self) -> &mut Self {
        C::register(self);
        self
    }

    /// Push a named constant directly.
    pub fn push(&mut self, constant: NamedConstant) {
        self.0.push(constant);
    }

    /// Get the number of constants in the collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return a sorted iterator over the constants (sorted by name).
    pub fn into_sorted_iter(&self) -> impl ExactSizeIterator<Item = &NamedConstant> {
        let mut v: Vec<&NamedConstant> = self.0.iter().collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v.into_iter()
    }

    /// Return an unsorted iterator over the constants.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &NamedConstant> {
        self.0.iter()
    }
}
