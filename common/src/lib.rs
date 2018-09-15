#![allow(non_snake_case)] // because of the generated bindings.

extern crate gdnative_core;
pub extern crate libc;

pub use gdnative_core::*;

use std::sync::{Once, ONCE_INIT};
use std::ops::*;

include!(concat!(env!("OUT_DIR"), "/common_types.rs"));

impl NativeScript {
    /// Try to down-cast from a `NativeScript` reference.
    pub fn to_rust_script<T: NativeClass>(&self) -> Option<NativeRef<T>> {
        unsafe {
            // TODO: There's gotta be a better way.
            let class = self.get_class_name();
            let gd_name = GodotString::from_str(T::class_name());

            if class != gd_name {
                return None;
            }

            return Some(NativeRef::from_sys(self.this));
        }
    }

    /// Up-cast to a `NativeScript` reference.
    pub fn from_rust_script<T: NativeClass>(script: NativeRef<T>) -> NativeScript {
        unsafe {
            NativeScript::from_sys(script.sys())
        }
    }
}
