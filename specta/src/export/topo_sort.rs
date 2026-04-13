use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    Types,
    datatype::{DataType, Fields, NamedDataType, Reference},
};

/// Topologically sort types so dependencies are emitted before the types that reference them.
pub fn topological_sort_types<'a>(
    ndts: Vec<&'a NamedDataType>,
    types: &Types,
) -> Vec<&'a NamedDataType> {
    if ndts.len() <= 1 {
        return ndts;
    }

    // Map each NamedDataType pointer to its index in the vec
    let ptr_to_index: HashMap<*const NamedDataType, usize> = ndts
        .iter()
        .enumerate()
        .map(|(i, ndt)| (*ndt as *const NamedDataType, i))
        .collect();

    // Build adjacency list: adj[j] contains i means type i depends on type j
    let n = ndts.len();
    let mut in_degree = vec![0usize; n];
    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];

    for (i, ndt) in ndts.iter().enumerate() {
        let mut dep_indices = HashSet::new();
        let mut visited_inline = HashSet::new();
        collect_type_deps(
            ndt.ty(),
            types,
            &ptr_to_index,
            &mut dep_indices,
            &mut visited_inline,
        );
        for j in dep_indices {
            if i != j {
                adj[j].push(i);
                in_degree[i] += 1;
            }
        }
    }

    // Kahn's algorithm
    let mut queue: VecDeque<usize> = in_degree
        .iter()
        .enumerate()
        .filter(|&(_, d)| *d == 0)
        .map(|(i, _)| i)
        .collect();

    let mut sorted = Vec::with_capacity(n);
    while let Some(node) = queue.pop_front() {
        sorted.push(node);
        for &neighbor in &adj[node] {
            in_degree[neighbor] -= 1;
            if in_degree[neighbor] == 0 {
                queue.push_back(neighbor);
            }
        }
    }

    // If cycles exist, append remaining nodes in original order
    if sorted.len() < n {
        let in_sorted: HashSet<usize> = sorted.iter().copied().collect();
        for i in 0..n {
            if !in_sorted.contains(&i) {
                sorted.push(i);
            }
        }
    }

    sorted.into_iter().map(|i| ndts[i]).collect()
}

/// Walk a DataType tree and collect indices of named types it depends on.
fn collect_type_deps(
    dt: &DataType,
    types: &Types,
    ptr_to_index: &HashMap<*const NamedDataType, usize>,
    deps: &mut HashSet<usize>,
    visited_inline: &mut HashSet<*const NamedDataType>,
) {
    match dt {
        DataType::Primitive(_) | DataType::Constant(_) => {}
        DataType::List(l) => collect_type_deps(l.ty(), types, ptr_to_index, deps, visited_inline),
        DataType::Map(m) => {
            collect_type_deps(m.key_ty(), types, ptr_to_index, deps, visited_inline);
            collect_type_deps(m.value_ty(), types, ptr_to_index, deps, visited_inline);
        }
        DataType::Nullable(inner) => {
            collect_type_deps(inner, types, ptr_to_index, deps, visited_inline)
        }
        DataType::Struct(st) => {
            collect_fields_deps(st.fields(), types, ptr_to_index, deps, visited_inline)
        }
        DataType::Enum(e) => {
            for (_, variant) in e.variants() {
                collect_fields_deps(variant.fields(), types, ptr_to_index, deps, visited_inline);
            }
        }
        DataType::Tuple(t) => {
            for elem in t.elements() {
                collect_type_deps(elem, types, ptr_to_index, deps, visited_inline);
            }
        }
        DataType::Reference(r) => match r {
            Reference::Named(nr) => {
                if let Some(referenced_ndt) = nr.get(types) {
                    let ptr = referenced_ndt as *const NamedDataType;
                    if nr.inline() {
                        // Inline references are expanded in place; recurse into the
                        // referenced type's body to capture transitive dependencies.
                        if visited_inline.insert(ptr) {
                            collect_type_deps(
                                referenced_ndt.ty(),
                                types,
                                ptr_to_index,
                                deps,
                                visited_inline,
                            );
                        }
                    } else if let Some(&idx) = ptr_to_index.get(&ptr) {
                        deps.insert(idx);
                    }
                }
                // Walk generic argument types as they are rendered inline.
                for (_, generic_dt) in nr.generics() {
                    collect_type_deps(generic_dt, types, ptr_to_index, deps, visited_inline);
                }
            }
            Reference::Generic(_) | Reference::Opaque(_) => {}
        },
    }
}

fn collect_fields_deps(
    fields: &Fields,
    types: &Types,
    ptr_to_index: &HashMap<*const NamedDataType, usize>,
    deps: &mut HashSet<usize>,
    visited_inline: &mut HashSet<*const NamedDataType>,
) {
    match fields {
        Fields::Unit => {}
        Fields::Unnamed(unnamed) => {
            for field in unnamed.fields() {
                if let Some(ty) = field.ty() {
                    collect_type_deps(ty, types, ptr_to_index, deps, visited_inline);
                }
            }
        }
        Fields::Named(named) => {
            for (_, field) in named.fields() {
                if let Some(ty) = field.ty() {
                    collect_type_deps(ty, types, ptr_to_index, deps, visited_inline);
                }
            }
        }
    }
}
