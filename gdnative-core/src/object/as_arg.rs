use crate::export::user_data::Map;
use crate::export::NativeClass;
use crate::object::ownership::{Ownership, Shared, Unique};
use crate::object::{GodotObject, Instance, Null, Ref, SubClass, TInstance, TRef};

/// Trait for safe conversion from Godot object references into API method arguments. This is
/// a sealed trait with no public interface.
///
/// In order to enforce thread safety statically, the ability to be passed to the engine is only
/// given to some reference types. Specifically, they are:
///
/// - All *owned* `Ref<T, Unique>` references. The `Unique` access is lost if passed into a
///   method.
/// - Owned and borrowed `Shared` references, including temporary ones (`TRef`).
///
/// It's unsound to pass `ThreadLocal` references to the engine because there is no guarantee
/// that the reference will stay on the same thread.
///
/// To explicitly pass a null reference to the engine, use `Null::null` or `GodotObject::null`.
pub trait AsArg<T>: private::Sealed {
    #[doc(hidden)]
    fn as_arg_ptr(&self) -> *mut sys::godot_object;

    #[doc(hidden)]
    #[inline]
    unsafe fn to_arg_variant(&self) -> crate::core_types::Variant {
        crate::core_types::Variant::from_object_ptr(self.as_arg_ptr())
    }
}

/// Trait for safe conversion from Godot object references into Variant. This is
/// a sealed trait with no public interface.
///
/// Used for `Variant` methods and implementations as a trait bound to improve type inference.
pub trait AsVariant: AsArg<<Self as AsVariant>::Target> {
    type Target;
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Sealed

mod private {
    pub trait Sealed {}
}

// Null
impl<T> private::Sealed for Null<T> {}

// Temporary references (shared ownership)
impl<'a, T: GodotObject> private::Sealed for TRef<'a, T, Shared> {}
impl<'a, T: GodotObject> private::Sealed for &'a Ref<T, Shared> {}
impl<'a, T: NativeClass> private::Sealed for TInstance<'a, T, Shared> {}
impl<'a, T: NativeClass> private::Sealed for &'a Instance<T, Shared> {}

// Persistent references (any ownership)
impl<T: GodotObject, Own: Ownership> private::Sealed for Ref<T, Own> {}
impl<T: NativeClass, Own: Ownership> private::Sealed for Instance<T, Own> {}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Null

impl<T: GodotObject> AsArg<T> for Null<T> {
    #[inline]
    fn as_arg_ptr(&self) -> *mut sys::godot_object {
        std::ptr::null_mut()
    }
}

impl<T: GodotObject> AsVariant for Null<T> {
    type Target = T;
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// TRef

impl<'a, T, U> AsArg<U> for TRef<'a, T, Shared>
where
    T: GodotObject + SubClass<U>,
    U: GodotObject,
{
    #[inline]
    fn as_arg_ptr(&self) -> *mut sys::godot_object {
        self.as_ptr()
    }
}

impl<'a, T: GodotObject> AsVariant for TRef<'a, T, Shared> {
    type Target = T;
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Ref

impl<T, U> AsArg<U> for Ref<T, Shared>
where
    T: GodotObject + SubClass<U>,
    U: GodotObject,
{
    #[inline]
    fn as_arg_ptr(&self) -> *mut sys::godot_object {
        self.as_ptr()
    }
}

impl<T, U> AsArg<U> for Ref<T, Unique>
where
    T: GodotObject + SubClass<U>,
    U: GodotObject,
{
    #[inline]
    fn as_arg_ptr(&self) -> *mut sys::godot_object {
        self.as_ptr()
    }
}

impl<T: GodotObject> AsVariant for Ref<T, Unique> {
    type Target = T;
}

impl<'a, T, U> AsArg<U> for &'a Ref<T, Shared>
where
    T: GodotObject + SubClass<U>,
    U: GodotObject,
{
    #[inline]
    fn as_arg_ptr(&self) -> *mut sys::godot_object {
        self.as_ptr()
    }
}

impl<T: GodotObject> AsVariant for Ref<T, Shared> {
    type Target = T;
}

impl<'a, T: GodotObject> AsVariant for &'a Ref<T, Shared> {
    type Target = T;
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// TInstance

impl<'a, T, U> AsArg<U> for TInstance<'a, T, Shared>
where
    T: NativeClass,
    T::Base: GodotObject + SubClass<U>,
    T::UserData: Map,
    U: GodotObject,
{
    #[inline]
    fn as_arg_ptr(&self) -> *mut sys::godot_object {
        self.as_base_ptr()
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Instance

impl<T, U, Own: Ownership> AsArg<U> for Instance<T, Own>
where
    T: NativeClass,
    T::Base: GodotObject + SubClass<U>,
    T::UserData: Map,
    U: GodotObject,
{
    #[inline]
    fn as_arg_ptr(&self) -> *mut sys::godot_object {
        self.as_base_ptr()
    }
}

impl<'a, T, U> AsArg<U> for &'a Instance<T, Shared>
where
    T: NativeClass,
    T::Base: GodotObject + SubClass<U>,
    T::UserData: Map,
    U: GodotObject,
{
    #[inline]
    fn as_arg_ptr(&self) -> *mut sys::godot_object {
        self.as_base_ptr()
    }
}
