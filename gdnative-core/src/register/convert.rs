use std::{
    ffi::{c_int, c_void},
    mem::transmute,
    ptr::NonNull,
};

use crate::{
    core_types::Variant,
    object::{ownership::Shared, GodotObject, Ref, TRef},
};

#[inline]
pub(crate) fn alloc_data<T: 'static>(data: T) -> *mut c_void {
    Box::into_raw(Box::new(data)).cast::<c_void>()
}

#[inline]
pub(crate) unsafe fn unalloc_data<T: 'static>(data: *mut c_void) -> T {
    // Move a value from a Box back to the stack by dereferencing
    // https://doc.rust-lang.org/std/boxed/index.html
    // https://github.com/rust-lang/rust/issues/80437
    *Box::from_raw(data.cast::<T>())
}

pub(crate) unsafe extern "C" fn free_data<T: 'static>(data: *mut c_void) {
    drop(unalloc_data::<T>(data))
}

#[inline]
pub(crate) unsafe fn as_class<'a, Class, T>(user_data: &'a *mut c_void) -> &'a T
where
    &'static Class: AsRef<T> + 'static,
{
    let Some(user_data) = user_data.cast::<Class>().as_ref() else {
        panic!(
            "user data pointer for {} is null (did the constructor fail?)",
            std::any::type_name::<Class>(),
        );
    };
    let user_data = user_data.as_ref();
    transmute::<&T, &'a T>(user_data)
}

#[inline]
pub(crate) unsafe fn as_variant_ref<'a>(value: &'a *mut sys::godot_variant) -> &'a Variant {
    // Trust Godot and will not check
    NonNull::new_unchecked(*value).cast::<Variant>().as_ref()
}

#[inline]
pub(crate) unsafe fn as_variant_args<'a>(
    args: &'a *mut *mut sys::godot_variant,
    num_args: c_int,
) -> &'a [&'a Variant] {
    // Trust Godot and will not check
    std::slice::from_raw_parts(args.cast::<&Variant>(), num_args as usize)
}

pub(crate) struct FunctionWithSite<F: 'static + std::panic::RefUnwindSafe> {
    pub site: crate::log::Site<'static>,
    pub function: F,
}

impl<F: 'static + std::panic::RefUnwindSafe> FunctionWithSite<F> {
    #[inline]
    pub(crate) unsafe fn as_self<'a>(method_data: &'a *mut c_void) -> &'a Self {
        // If the creation of method_data failed, then the method will not be registered
        NonNull::new_unchecked(*method_data).cast().as_ref()
    }
}

pub trait FromInstancePtr<'a>: Sized {
    // The reason for no type constraints is to allow smart pointer replacement from external libraries
    type RawBase;

    unsafe fn from_instance_ptr<T>(user_data: &'a T, base: &'a *mut sys::godot_object) -> Self;
}

pub trait FromBasePtr<'a>: FromInstancePtr<'a> {
    unsafe fn from_base_ptr(base_ptr: &'a *mut sys::godot_object) -> Self;
}

impl<'a, B: GodotObject> FromInstancePtr<'a> for Ref<B, Shared> {
    type RawBase = B;

    #[inline]
    unsafe fn from_instance_ptr<T>(_user_data: &'a T, base: &'a *mut sys::godot_object) -> Self {
        Self::from_base_ptr(base)
    }
}

impl<'a, B: GodotObject> FromBasePtr<'a> for Ref<B, Shared> {
    unsafe fn from_base_ptr(base: &'a *mut sys::godot_object) -> Self {
        // Trust Godot and will not check
        let base = NonNull::new_unchecked(*base);
        Ref::from_sys(base)
    }
}

impl<'a, B: GodotObject> FromInstancePtr<'a> for TRef<'a, B, Shared> {
    type RawBase = B;

    #[inline]
    unsafe fn from_instance_ptr<T>(_user_data: &'a T, base: &'a *mut sys::godot_object) -> Self {
        Self::from_base_ptr(base)
    }
}

impl<'a, B: GodotObject> FromBasePtr<'a> for TRef<'a, B, Shared> {
    unsafe fn from_base_ptr(base: &'a *mut sys::godot_object) -> Self {
        // Trust Godot and will not check
        let base = NonNull::new_unchecked(*base);
        Ref::<B, Shared>::from_sys(base).assume_safe_unchecked()
    }
}

impl<'a, B: GodotObject> FromInstancePtr<'a> for &'a B {
    type RawBase = B;

    #[inline]
    unsafe fn from_instance_ptr<T>(_user_data: &'a T, base: &'a *mut sys::godot_object) -> Self {
        Self::from_base_ptr(base)
    }
}

impl<'a, B: GodotObject> FromBasePtr<'a> for &'a B {
    unsafe fn from_base_ptr(base: &'a *mut sys::godot_object) -> Self {
        // Trust Godot and will not check
        let base = NonNull::new_unchecked(*base);
        Ref::<B, Shared>::from_sys(base)
            .assume_safe_unchecked()
            .as_ref()
    }
}

// TODO: FromInstancePtr for Instance
// TODO: FromInstancePtr for TInstance
