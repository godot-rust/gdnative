use std::any::TypeId;
use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;

use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::export::NativeClass;
use crate::init::InitLevel;

static CLASS_REGISTRY: Lazy<RwLock<HashMap<TypeId, ClassInfo>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Clone, Debug)]
pub(crate) struct ClassInfo {
    pub name: Cow<'static, str>,
    pub init_level: InitLevel,
}

/// Access the [`ClassInfo`] of the class `C`.
#[inline]
pub(crate) fn with_class_info<C: NativeClass, F, R>(f: F) -> Option<R>
where
    F: FnOnce(&ClassInfo) -> R,
{
    CLASS_REGISTRY.read().get(&TypeId::of::<C>()).map(f)
}

/// Returns the NativeScript name of the class `C` if it is registered.
/// Can also be used to validate whether or not `C` has been added using `InitHandle::add_class<C>()`
#[inline]
pub(crate) fn class_name<C: NativeClass>() -> Option<Cow<'static, str>> {
    with_class_info::<C, _, _>(|i| i.name.clone())
}

/// Returns the NativeScript name of the class `C` if it is registered, or a best-effort description
/// of the type otherwise.
///
/// The returned string should only be used for diagnostic purposes, not for types that the user
/// has to name explicitly. The format is not guaranteed.
#[inline]
pub(crate) fn class_name_or_default<C: NativeClass>() -> Cow<'static, str> {
    class_name::<C>().unwrap_or_else(|| Cow::Borrowed(std::any::type_name::<C>()))
}

/// Registers the class `C` in the class registry, using a custom name at the given level.
/// Returns `Ok(true)` if FFI registration needs to be performed. `Ok(false)` if the class has
/// already been registered on another level.
/// Returns an error with the old `ClassInfo` if a conflicting entry for `C` was already added.
#[inline]
pub(crate) fn register_class_as<C: NativeClass>(
    name: Cow<'static, str>,
    init_level: InitLevel,
) -> Result<bool, RegisterError> {
    let type_id = TypeId::of::<C>();
    let mut registry = CLASS_REGISTRY.write();
    match registry.entry(type_id) {
        Entry::Vacant(entry) => {
            entry.insert(ClassInfo { name, init_level });
            Ok(true)
        }
        Entry::Occupied(entry) => {
            let class_info = entry.get();
            let kind = if class_info.name != name {
                Some(RegisterErrorKind::ConflictingName)
            } else if class_info.init_level.intersects(init_level) {
                Some(RegisterErrorKind::AlreadyOnSameLevel)
            } else {
                None
            };

            if let Some(kind) = kind {
                Err(RegisterError {
                    class_info: class_info.clone(),
                    type_name: std::any::type_name::<C>(),
                    kind,
                })
            } else {
                Ok(false)
            }
        }
    }
}

#[inline]
#[allow(dead_code)] // Currently unused on platforms with inventory support
pub(crate) fn types_with_init_level(allow: InitLevel, deny: InitLevel) -> Vec<Cow<'static, str>> {
    let registry = CLASS_REGISTRY.read();
    let mut list = registry
        .values()
        .filter(|class_info| {
            class_info.init_level.intersects(allow) && !class_info.init_level.intersects(deny)
        })
        .map(|class_info| class_info.name.clone())
        .collect::<Vec<_>>();

    list.sort_unstable();
    list
}

#[derive(Debug)]
pub(crate) struct RegisterError {
    pub type_name: &'static str,
    pub class_info: ClassInfo,
    pub kind: RegisterErrorKind,
}

#[derive(Debug)]
pub(crate) enum RegisterErrorKind {
    ConflictingName,
    AlreadyOnSameLevel,
}

impl fmt::Display for RegisterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            RegisterErrorKind::ConflictingName => {
                write!(
                    f,
                    "`{}` has already been registered as `{}`",
                    self.type_name, self.class_info.name
                )
            }
            RegisterErrorKind::AlreadyOnSameLevel => {
                write!(f, "`{}` has already been registered", self.type_name)
            }
        }
    }
}

/// Clears the registry
#[inline]
pub(crate) fn cleanup() {
    CLASS_REGISTRY.write().clear();
}
