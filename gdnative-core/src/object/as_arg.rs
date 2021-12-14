use crate::object::ownership::{Ownership, Shared, Unique};
use crate::object::{GodotObject, Null, Ref, SubClass, TRef};

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

impl<'a, T> private::Sealed for Null<T> {}
impl<'a, T: GodotObject> private::Sealed for TRef<'a, T, Shared> {}
impl<T: GodotObject, Own: Ownership> private::Sealed for Ref<T, Own> {}
impl<'a, T: GodotObject> private::Sealed for &'a Ref<T, Shared> {}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Null

impl<'a, T: GodotObject> AsArg<T> for Null<T> {
    #[inline]
    fn as_arg_ptr(&self) -> *mut sys::godot_object {
        std::ptr::null_mut()
    }
}

impl<'a, T: GodotObject> AsVariant for Null<T> {
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
