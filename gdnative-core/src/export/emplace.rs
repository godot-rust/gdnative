//! Support code for script emplacement.

use std::any::Any;
use std::cell::RefCell;

use crate::export::class_registry;

use super::NativeClass;

thread_local! {
    static CELL: RefCell<Option<Box<dyn Any>>> = RefCell::default();
}

/// Place a script to be taken by the emplacement constructor. Must be called
/// directly before `NativeScript::_new` for intended behavior.
///
/// # Panics
///
/// If there is already a value placed for this thread, or if the thread is
/// exiting. This is always a bug in the bindings.
pub fn place<T: NativeClass>(script: T) {
    CELL.with(|f| {
        if f.replace(Some(Box::new(script))).is_some() {
            panic!(
                "there is already a value in the emplacement cell (this is a bug in the bindings)"
            );
        }
    });
}

/// Take the script stored for emplacement and return it. Returns `None` if
/// there is no value in store.
///
/// # Panics
///
/// If there is a value in store but it is of the incorrect type. This is always
/// a bug in the bindings.
pub fn take<T: NativeClass>() -> Option<T> {
    CELL.with(|f| f.borrow_mut().take())
        .map(|script| match script.downcast() {
            Ok(script) => *script,
            Err(any) => panic!(
                "expecting {} in the emplacement cell, got {:?} (this is a bug in the bindings)",
                class_registry::class_name_or_default::<T>(),
                (*any).type_id(),
            ),
        })
}
