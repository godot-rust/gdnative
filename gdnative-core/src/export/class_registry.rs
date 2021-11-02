use crate::export::NativeClass;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::any::TypeId;
use std::collections::HashSet;

static CLASS_REGISTRY: Lazy<RwLock<HashSet<TypeId>>> = Lazy::new(|| RwLock::new(HashSet::new()));

/// Can be used to validate whether or not `C` has been added using `InitHandle::add_class<C>()`
/// Returns true if added otherwise false.
#[inline]
pub(crate) fn is_class_registered<C: NativeClass>() -> bool {
    let type_id = TypeId::of::<C>();
    CLASS_REGISTRY.read().contains(&type_id)
}

/// Registers the class `C` in the class registry.
/// Returns `true` if `C` was not already present in the list.
/// Returns `false` if `C` was already added.
#[inline]
pub(crate) fn register_class<C: NativeClass>() -> bool {
    let type_id = TypeId::of::<C>();
    CLASS_REGISTRY.write().insert(type_id)
}

/// Clears the registry
#[inline]
pub(crate) fn cleanup() {
    CLASS_REGISTRY.write().clear();
}
