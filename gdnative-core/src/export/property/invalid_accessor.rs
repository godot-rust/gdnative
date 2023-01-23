//! Accessors indicating that no valid accessors are present.

use std::mem;

use crate::core_types::Variant;
use crate::export::{class_registry, NativeClass};

use super::accessor::{RawGetter, RawSetter};

/// Default setter used for a new property indicating that no valid setter is present. Outputs errors when invoked.
#[derive(Debug)]
pub struct InvalidSetter<'l> {
    property_name: &'l str,
}

/// Default getter used for a new property indicating that no valid getter is present. Outputs errors when invoked.
#[derive(Debug)]
pub struct InvalidGetter<'l> {
    property_name: &'l str,
}

impl<'l> InvalidSetter<'l> {
    #[inline]
    pub fn new(property_name: &'l str) -> Self {
        InvalidSetter { property_name }
    }
}

impl<'l> InvalidGetter<'l> {
    #[inline]
    pub fn new(property_name: &'l str) -> Self {
        InvalidGetter { property_name }
    }
}

#[derive(Debug)]
struct InvalidAccessorData {
    class_name: String,
    property_name: String,
}

extern "C" fn invalid_setter(
    _this: *mut sys::godot_object,
    data: *mut libc::c_void,
    _class: *mut libc::c_void,
    _val: *mut sys::godot_variant,
) {
    let InvalidAccessorData {
        class_name,
        property_name,
    } = unsafe { &*(data as *const InvalidAccessorData) };
    godot_error!(
        "property {} on native class {} does not have a setter",
        property_name,
        class_name
    );
}

extern "C" fn invalid_getter(
    _this: *mut sys::godot_object,
    data: *mut libc::c_void,
    _class: *mut libc::c_void,
) -> sys::godot_variant {
    let InvalidAccessorData {
        class_name,
        property_name,
    } = unsafe { &*(data as *const InvalidAccessorData) };
    godot_error!(
        "property {} on native class {} does not have a getter",
        property_name,
        class_name
    );
    Variant::nil().leak()
}

extern "C" fn invalid_free_func(data: *mut libc::c_void) {
    let data = unsafe { Box::from_raw(data as *mut InvalidAccessorData) };
    mem::drop(data)
}

unsafe impl<'l, C: NativeClass, T> RawSetter<C, T> for InvalidSetter<'l> {
    #[inline]
    unsafe fn into_godot_function(self) -> sys::godot_property_set_func {
        let mut set = sys::godot_property_set_func::default();

        let data = Box::new(InvalidAccessorData {
            class_name: class_registry::class_name_or_default::<C>().into_owned(),
            property_name: self.property_name.to_string(),
        });

        set.method_data = Box::into_raw(data) as *mut _;
        set.set_func = Some(invalid_setter);
        set.free_func = Some(invalid_free_func);
        set
    }
}

unsafe impl<'l, C: NativeClass, T> RawGetter<C, T> for InvalidGetter<'l> {
    #[inline]
    unsafe fn into_godot_function(self) -> sys::godot_property_get_func {
        let mut get = sys::godot_property_get_func::default();

        let data = Box::new(InvalidAccessorData {
            class_name: class_registry::class_name_or_default::<C>().into_owned(),
            property_name: self.property_name.to_string(),
        });

        get.method_data = Box::into_raw(data) as *mut _;
        get.get_func = Some(invalid_getter);
        get.free_func = Some(invalid_free_func);
        get
    }
}
