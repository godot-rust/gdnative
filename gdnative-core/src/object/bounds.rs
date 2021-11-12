//! Various traits to verify memory policy, ownership policy or lifetime bounds
//!
//! The symbols defined in this module are internal and used to enhance type safety.
//! You typically will not need to work with them.

use crate::object::memory::*;
use crate::object::ownership::*;
use crate::object::*;

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Implementation for Memory policy

/// Specialization trait depending on [`Memory`]. This is an internal interface.
pub trait MemorySpec: Sized {
    /// Pointer wrapper that may be `Drop` or not.
    #[doc(hidden)]
    type PtrWrapper: PtrWrapper;

    #[doc(hidden)]
    unsafe fn impl_from_maybe_ref_counted<T: GodotObject<Memory = Self>>(
        ptr: NonNull<sys::godot_object>,
    ) -> Option<Ref<T, Unique>>
    where
        Self: Memory;

    #[doc(hidden)]
    unsafe fn impl_assume_safe<'a, T: GodotObject<Memory = Self>>(
        this: &Ref<T, Shared>,
    ) -> TRef<'a, T, Shared>
    where
        Self: Memory;

    #[doc(hidden)]
    unsafe fn impl_assume_unique<T: GodotObject<Memory = Self>>(
        this: Ref<T, Shared>,
    ) -> Ref<T, Unique>
    where
        Self: Memory;

    #[doc(hidden)]
    unsafe fn maybe_add_ref<T: GodotObject<Memory = Self>>(raw: &RawObject<T>)
    where
        Self: Memory;

    #[doc(hidden)]
    unsafe fn maybe_init_ref<T: GodotObject<Memory = Self>>(raw: &RawObject<T>)
    where
        Self: Memory;
}

impl MemorySpec for ManuallyManaged {
    type PtrWrapper = Forget;

    #[inline(always)]
    unsafe fn impl_from_maybe_ref_counted<T: GodotObject<Memory = Self>>(
        ptr: NonNull<sys::godot_object>,
    ) -> Option<Ref<T, Unique>> {
        if RawObject::<ReferenceCountedClassPlaceholder>::try_from_sys_ref(ptr).is_some() {
            drop(Ref::<ReferenceCountedClassPlaceholder, Unique>::init_from_sys(ptr));
            None
        } else {
            let obj = Ref::<ManuallyManagedClassPlaceholder, Unique>::init_from_sys(ptr);

            if obj.as_raw().is_class::<T>() {
                Some(obj.cast_unchecked())
            } else {
                obj.free();
                None
            }
        }
    }

    #[inline(always)]
    unsafe fn impl_assume_safe<'a, T: GodotObject<Memory = Self>>(
        this: &Ref<T, Shared>,
    ) -> TRef<'a, T, Shared> {
        debug_assert!(
            this.is_instance_sane(),
            "assume_safe called on an invalid pointer"
        );
        this.assume_safe_unchecked()
    }

    #[inline(always)]
    unsafe fn impl_assume_unique<T: GodotObject<Memory = Self>>(
        this: Ref<T, Shared>,
    ) -> Ref<T, Unique> {
        debug_assert!(
            this.is_instance_sane(),
            "assume_unique called on an invalid pointer"
        );
        this.cast_access()
    }

    #[inline]
    unsafe fn maybe_add_ref<T: GodotObject<Memory = Self>>(_raw: &RawObject<T>) {}
    #[inline]
    unsafe fn maybe_init_ref<T: GodotObject<Memory = Self>>(_raw: &RawObject<T>) {}
}

impl MemorySpec for RefCounted {
    type PtrWrapper = UnRef;

    #[inline(always)]
    unsafe fn impl_from_maybe_ref_counted<T: GodotObject<Memory = Self>>(
        ptr: NonNull<sys::godot_object>,
    ) -> Option<Ref<T, Unique>> {
        if RawObject::<ReferenceCountedClassPlaceholder>::try_from_sys_ref(ptr).is_some() {
            let obj = Ref::<ReferenceCountedClassPlaceholder, Unique>::init_from_sys(ptr);

            if obj.as_raw().is_class::<T>() {
                Some(obj.cast_unchecked())
            } else {
                None
            }
        } else {
            RawObject::<ManuallyManagedClassPlaceholder>::from_sys_ref_unchecked(ptr).free();
            None
        }
    }

    #[inline(always)]
    unsafe fn impl_assume_safe<'a, T: GodotObject<Memory = Self>>(
        this: &Ref<T, Shared>,
    ) -> TRef<'a, T, Shared> {
        this.assume_safe_unchecked()
    }

    #[inline(always)]
    unsafe fn impl_assume_unique<T: GodotObject<Memory = Self>>(
        this: Ref<T, Shared>,
    ) -> Ref<T, Unique> {
        this.cast_access()
    }

    #[inline]
    unsafe fn maybe_add_ref<T: GodotObject<Memory = Self>>(raw: &RawObject<T>) {
        raw.add_ref();
    }

    #[inline]
    unsafe fn maybe_init_ref<T: GodotObject<Memory = Self>>(raw: &RawObject<T>) {
        raw.init_ref_count();
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Drop strategy

/// Specialization trait for `Drop` behavior.
pub trait PtrWrapper {
    fn new(ptr: NonNull<sys::godot_object>) -> Self;
    fn as_non_null(&self) -> NonNull<sys::godot_object>;

    #[inline]
    fn as_ptr(&self) -> *mut sys::godot_object {
        self.as_non_null().as_ptr()
    }
}

/// Simply releases the held object without deallocating it.
#[derive(Copy, Clone)]
pub struct Forget(NonNull<sys::godot_object>);
impl PtrWrapper for Forget {
    #[inline]
    fn new(ptr: NonNull<sys::godot_object>) -> Self {
        Forget(ptr)
    }

    #[inline]
    fn as_non_null(&self) -> NonNull<sys::godot_object> {
        self.0
    }
}

/// Decrements the reference count on the held object, deallocating it if it's the last ref.
pub struct UnRef(NonNull<sys::godot_object>);
impl PtrWrapper for UnRef {
    #[inline]
    fn new(ptr: NonNull<sys::godot_object>) -> Self {
        UnRef(ptr)
    }

    #[inline]
    fn as_non_null(&self) -> NonNull<sys::godot_object> {
        self.0
    }
}
impl Drop for UnRef {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let raw = RawObject::<ReferenceCountedClassPlaceholder>::from_sys_ref_unchecked(self.0);
            raw.unref_and_free_if_last();
        }
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// assume_safe and lifetime bounds

/// Trait for constraining `assume_safe` lifetimes to the one of `&self` when `T` is
/// reference-counted. This is an internal interface.
pub trait LifetimeConstraint<Kind: Memory> {}

/// Type used to check lifetime constraint depending on `Memory`. Internal interface.
#[doc(hidden)]
pub struct AssumeSafeLifetime<'a, 'r> {
    _marker: PhantomData<(&'a (), &'r ())>,
}

impl<'a, 'r> LifetimeConstraint<ManuallyManaged> for AssumeSafeLifetime<'a, 'r> {}
impl<'a, 'r: 'a> LifetimeConstraint<RefCounted> for AssumeSafeLifetime<'a, 'r> {}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// SafeDeref, SafeAsRaw

/// Trait for combinations of `Memory` and `Ownership` that can be dereferenced safely.
/// This is an internal interface.
pub unsafe trait SafeDeref<Kind: Memory, Own: Ownership> {
    /// Returns a safe reference to the underlying object.
    #[doc(hidden)]
    fn impl_as_ref<T: GodotObject<Memory = Kind>>(this: &Ref<T, Own>) -> TRef<'_, T, Own>;
}

/// Trait for persistent `Ref`s that point to valid objects. This is an internal interface.
pub unsafe trait SafeAsRaw<Kind: Memory, Own: Ownership> {
    /// Returns a raw reference to the underlying object.
    #[doc(hidden)]
    fn impl_as_raw<T: GodotObject<Memory = Kind>>(this: &Ref<T, Own>) -> &RawObject<T>;
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// RefImplBound

/// Struct to be used for various `Ref` trait bounds.
pub struct RefImplBound {
    _private: (),
}

unsafe impl SafeDeref<ManuallyManaged, Unique> for RefImplBound {
    #[inline]
    fn impl_as_ref<T: GodotObject<Memory = ManuallyManaged>>(
        this: &Ref<T, Unique>,
    ) -> TRef<'_, T, Unique> {
        unsafe { this.assume_safe_unchecked() }
    }
}

unsafe impl<Own: LocalThreadOwnership> SafeDeref<RefCounted, Own> for RefImplBound {
    #[inline]
    fn impl_as_ref<T: GodotObject<Memory = RefCounted>>(this: &Ref<T, Own>) -> TRef<'_, T, Own> {
        unsafe { this.assume_safe_unchecked() }
    }
}

unsafe impl SafeAsRaw<ManuallyManaged, Unique> for RefImplBound {
    #[inline]
    fn impl_as_raw<T: GodotObject<Memory = ManuallyManaged>>(
        this: &Ref<T, Unique>,
    ) -> &RawObject<T> {
        unsafe { this.as_raw_unchecked() }
    }
}

unsafe impl<Own: Ownership> SafeAsRaw<RefCounted, Own> for RefImplBound {
    #[inline]
    fn impl_as_raw<T: GodotObject<Memory = RefCounted>>(this: &Ref<T, Own>) -> &RawObject<T> {
        unsafe { this.as_raw_unchecked() }
    }
}
