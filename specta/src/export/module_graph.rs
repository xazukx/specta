use std::{borrow::Cow, collections::BTreeMap};

use crate::{Constants, NamedConstant, Types, datatype::NamedDataType};

/// A node in the module tree. Represents a Rust module that contains
/// types, constants, and child modules.
pub struct Module<'a> {
    /// Types belonging directly to this module.
    pub types: Vec<&'a NamedDataType>,
    /// Constants belonging directly to this module.
    pub constants: Vec<&'a NamedConstant>,
    /// Child modules keyed by segment name.
    pub children: BTreeMap<&'a str, Module<'a>>,
    /// Full Rust module path (e.g., `"shared::item"`).
    pub module_path: Cow<'static, str>,
}

/// Build a module tree from a flat collection of types and constants.
pub fn build_module_graph<'a>(types: &'a Types, constants: &'a Constants) -> Module<'a> {
    let mut root = Module {
        types: Default::default(),
        constants: Default::default(),
        children: Default::default(),
        module_path: Default::default(),
    };

    for ndt in types.into_unsorted_iter() {
        let path = ndt.module_path();

        if path.is_empty() {
            root.types.push(ndt);
        } else {
            let mut current = &mut root;
            let mut current_path = String::new();
            for segment in path.split("::") {
                if !current_path.is_empty() {
                    current_path.push_str("::");
                }
                current_path.push_str(segment);

                current = current.children.entry(segment).or_insert_with(|| Module {
                    types: Default::default(),
                    constants: Default::default(),
                    children: Default::default(),
                    module_path: current_path.clone().into(),
                });
            }

            current.types.push(ndt);
        }
    }

    for constant in constants.iter() {
        let path = &constant.module_path;

        if path.is_empty() {
            root.constants.push(constant);
        } else {
            let mut current = &mut root;
            let mut current_path = String::new();
            for segment in path.split("::") {
                if !current_path.is_empty() {
                    current_path.push_str("::");
                }
                current_path.push_str(segment);

                current = current.children.entry(segment).or_insert_with(|| Module {
                    types: Default::default(),
                    constants: Default::default(),
                    children: Default::default(),
                    module_path: current_path.clone().into(),
                });
            }

            current.constants.push(constant);
        }
    }

    root
}
