use std::sync::{Mutex, OnceLock, PoisonError};

use crate::{Constant, Constants, Type, Types};

// Global type store for collecting custom types to export.
//
// We intentionally store functions over a `Types` directly to ensure any internal panics aren't done in CTOR.
#[allow(clippy::type_complexity)]
static TYPES: OnceLock<Mutex<Vec<fn(&mut Types)>>> = OnceLock::new();

// Global constant store for collecting constants to export.
#[allow(clippy::type_complexity)]
static CONSTANTS: OnceLock<Mutex<Vec<fn(&mut Constants)>>> = OnceLock::new();

/// Get the global type store containing all automatically collected types.
///
/// All types with the [`Type`](macro@crate::Type) macro will automatically be registered here unless they have been explicitly disabled with `#[specta(collect = false)]`.
///
/// Note that when enabling the `collect` feature, you will not be able to enable the `unsafe_code` lint as [`ctor`] (which is used internally) is marked unsafe.
///
pub fn collect() -> Types {
    let types = TYPES
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner);

    let mut map = Types::default();
    for export in types.iter() {
        export(&mut map);
    }
    map
}

/// Get the global constant store containing all automatically collected constants.
///
/// All constants annotated with `#[specta_const]` will automatically be registered here unless they have been explicitly disabled with `#[specta_const(collect = false)]`.
///
pub fn collect_constants() -> Constants {
    let constants = CONSTANTS
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner);

    let mut map = Constants::default();
    for export in constants.iter() {
        export(&mut map);
    }
    map
}

#[doc(hidden)]
pub mod internal {
    use super::*;

    // Called within ctor functions to register a type.
    pub fn register<T: Type>() {
        TYPES
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(|types| {
                // The side-effect of this is registering the type.
                T::definition(types);
            });
    }

    // Called within ctor functions to register a constant.
    pub fn register_constant<C: Constant>() {
        CONSTANTS
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(|constants| {
                C::register(constants);
            });
    }

    // We expose this for the macros
    #[cfg(feature = "collect")]
    pub use ctor;
}
