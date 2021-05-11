use crate::api::{Api, GodotClass};
use std::collections::HashSet;

/// Find set of classes that only depend on each other.
///
/// Mostly useful to get "self-contained" bindings for classes
/// of interest.
///
/// If many classes should be checked, a previous result can be
/// passed to avoid unnecessary checks.
#[allow(clippy::implicit_hasher)]
pub fn strongly_connected_components(
    api: &Api,
    class: &str,
    visited: Option<HashSet<String>>,
) -> HashSet<String> {
    let mut visited = visited.unwrap_or_default();

    if let Some(class) = api.find_class(class) {
        visit(api, class, &mut visited);
    }

    visited
}

fn visit(api: &Api, class: &GodotClass, visited: &mut HashSet<String>) {
    visited.insert(class.name.clone());

    let mut to_visit = HashSet::new();
    to_visit.extend(base_classes(api, class));
    to_visit.extend(referenced_classes(api, class));

    for v in to_visit {
        // already seen, don't insert again.
        if visited.contains(&v) {
            continue;
        }

        if let Some(class) = api.find_class(&v) {
            visit(api, class, visited);
        }
    }
}

fn base_classes(api: &Api, class: &GodotClass) -> HashSet<String> {
    let mut bases = HashSet::new();

    if !class.base_class.is_empty() {
        if let Some(class) = api.find_class(&class.base_class) {
            bases.insert(class.name.clone());
            bases.extend(base_classes(api, class));
        }
    }

    bases
}

fn referenced_classes(api: &Api, class: &GodotClass) -> HashSet<String> {
    let mut classes = HashSet::new();

    for method in &class.methods {
        // return
        if let Some(ret_class) = api.find_class(&method.return_type) {
            classes.insert(ret_class.name.clone());
        }

        for arg in &method.arguments {
            if let Some(ty) = api.find_class(&arg.ty) {
                classes.insert(ty.name.clone());
            }
        }
    }

    classes
}
