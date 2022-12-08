//! Provides types to interact with the Godot `Object` class hierarchy
//!
//! This module contains wrappers and helpers to interact with Godot objects.
//! In Godot, classes stand in an inheritance relationship, with the root at `Object`.
//!
//! If you are looking for how to manage user-defined types (native scripts),
//! check out the [`export`][crate::export] module.

use std::borrow::Borrow;
use std::ffi::CString;
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

use bounds::{
    AssumeSafeLifetime, LifetimeConstraint, MemorySpec, PtrWrapper, RefImplBound, SafeAsRaw,
    SafeDeref,
};
use memory::{ManuallyManaged, Memory, RefCounted};
use ownership::{NonUniqueOwnership, Ownership, Shared, ThreadLocal, Unique};

use crate::export::NativeClass;
use crate::private::{get_api, ManuallyManagedClassPlaceholder, ReferenceCountedClassPlaceholder};
use crate::sys;

pub use as_arg::*;
pub use instance::*;
pub use new_ref::NewRef;
pub use raw::RawObject;

pub mod bounds;
pub mod memory;
pub mod ownership;

mod as_arg;
mod instance;
mod new_ref;
mod raw;

/// Trait for Godot API objects. This trait is sealed, and implemented for generated wrapper
/// types.
///
/// Bare `GodotObject` references, like `&Node`, can be used safely, but do not track thread
/// access states, which limits their usefulness to some extent. It's not, for example, possible
/// to pass a `&Node` into an API method because it might have came from a `Unique` reference.
/// As such, it's usually better to use `Ref` and `TRef`s whenever possible.
///
/// For convenience. it's possible to use bare references as `owner` arguments in exported
/// methods when using NativeScript, but the limitations above should be kept in mind. See
/// the `OwnerArg` for more information.
///
/// IF it's ever needed to obtain persistent references out of bare references, the `assume_`
/// methods can be used.
pub unsafe trait GodotObject: Sized + crate::private::godot_object::Sealed {
    /// The memory management kind of this type. This modifies the behavior of the
    /// [`Ref`](struct.Ref.html) smart pointer. See its type-level documentation for more
    /// information.
    type Memory: Memory;

    fn class_name() -> &'static str;

    /// Creates an explicitly null reference of `Self` as a method argument. This makes type
    /// inference easier for the compiler compared to `Option`.
    #[inline]
    fn null() -> Null<Self> {
        Null::null()
    }

    /// Creates a new instance of `Self` using a zero-argument constructor, as a `Unique`
    /// reference.
    #[inline]
    fn new() -> Ref<Self, Unique>
    where
        Self: Instanciable,
    {
        Ref::new()
    }

    /// Performs a dynamic reference downcast to target type.
    ///
    /// The `cast` method can only be used for downcasts. For statically casting to a
    /// supertype, use `upcast` instead.
    ///
    /// This method is only for conversion between engine types. To downcast to a `NativeScript`
    /// type from its base type, see `Ref::cast_instance` and `TRef::cast_instance`.
    #[inline]
    fn cast<T>(&self) -> Option<&T>
    where
        T: GodotObject + SubClass<Self>,
    {
        self.as_raw().cast().map(T::cast_ref)
    }

    /// Performs a static reference upcast to a supertype that is guaranteed to be valid.
    ///
    /// This is guaranteed to be a no-op at runtime.
    #[inline(always)]
    fn upcast<T>(&self) -> &T
    where
        T: GodotObject,
        Self: SubClass<T>,
    {
        unsafe { T::cast_ref(self.as_raw().cast_unchecked()) }
    }

    /// Creates a reference to `Self` given a `RawObject` reference. This is an internal
    /// interface,
    #[doc(hidden)]
    #[inline]
    fn cast_ref(raw: &RawObject<Self>) -> &Self {
        unsafe { &*(raw as *const _ as *const _) }
    }

    /// Casts `self` to `RawObject`. This is an internal interface.
    #[doc(hidden)]
    #[inline]
    fn as_raw(&self) -> &RawObject<Self> {
        unsafe { &*(self as *const _ as *const _) }
    }

    /// Casts `self` to a raw pointer. This is an internal interface.
    #[doc(hidden)]
    #[inline]
    fn as_ptr(&self) -> *mut sys::godot_object {
        self.as_raw().sys().as_ptr()
    }

    /// Creates a persistent reference to the same Godot object with shared thread access.
    ///
    /// # Safety
    ///
    /// There must not be any `Unique` or `ThreadLocal` references of the object when this
    /// is called. This causes undefined behavior otherwise.
    #[inline]
    unsafe fn assume_shared(&self) -> Ref<Self, Shared>
    where
        Self: Sized,
    {
        Ref::from_sys(self.as_raw().sys())
    }

    /// Creates a persistent reference to the same Godot object with thread-local thread access.
    ///
    /// # Safety
    ///
    /// There must not be any `Unique` or `Shared` references of the object when this
    /// is called. This causes undefined behavior otherwise.
    #[inline]
    unsafe fn assume_thread_local(&self) -> Ref<Self, ThreadLocal>
    where
        Self: Sized + GodotObject<Memory = RefCounted>,
    {
        Ref::from_sys(self.as_raw().sys())
    }

    /// Creates a persistent reference to the same Godot object with unique access.
    ///
    /// # Safety
    ///
    /// **Use with care.** `Unique` is a very strong assumption that can easily be
    /// violated. Only use this when you are **absolutely** sure you have the only reference.
    ///
    /// There must be no other references of the object when this is called. This causes
    /// undefined behavior otherwise.
    #[inline]
    unsafe fn assume_unique(&self) -> Ref<Self, Unique>
    where
        Self: Sized,
    {
        Ref::from_sys(self.as_raw().sys())
    }

    /// Recovers a instance ID previously returned by `Object::get_instance_id` if the object is
    /// still alive. See also `TRef::try_from_instance_id`.
    ///
    /// # Safety
    ///
    /// During the entirety of `'a`, the thread from which `try_from_instance_id` is called must
    /// have exclusive access to the underlying object, if it is still alive.
    #[inline]
    unsafe fn try_from_instance_id<'a>(id: i64) -> Option<TRef<'a, Self, Shared>> {
        TRef::try_from_instance_id(id)
    }

    /// Recovers a instance ID previously returned by `Object::get_instance_id` if the object is
    /// still alive, and panics otherwise. This does **NOT** guarantee that the resulting
    /// reference is safe to use.
    ///
    /// # Panics
    ///
    /// Panics if the given id refers to a destroyed object. For a non-panicking version, see
    /// `try_from_instance_id`.
    ///
    /// # Safety
    ///
    /// During the entirety of `'a`, the thread from which `try_from_instance_id` is called must
    /// have exclusive access to the underlying object, if it is still alive.
    #[inline]
    unsafe fn from_instance_id<'a>(id: i64) -> TRef<'a, Self, Shared> {
        TRef::from_instance_id(id)
    }
}

/// Marker trait for API types that are subclasses of another type. This trait is implemented
/// by the bindings generator, and has no public interface. Users should not attempt to
/// implement this trait.
pub unsafe trait SubClass<A: GodotObject>: GodotObject {}
unsafe impl<T: GodotObject> SubClass<T> for T {}

/// GodotObjects that have a zero argument constructor.
pub trait Instanciable: GodotObject {
    fn construct() -> Ref<Self, Unique>;
}

/// Manually managed Godot classes implementing `queue_free`. This trait has no public
/// interface. See `Ref::queue_free`.
pub trait QueueFree: GodotObject {
    /// Deallocate the object in the near future.
    ///
    /// # Safety
    ///
    /// When this function is dequeued no references to this
    /// object must be held and dereferenced.
    #[doc(hidden)]
    unsafe fn godot_queue_free(sys: *mut sys::godot_object);
}

/// A polymorphic smart pointer for Godot objects whose behavior changes depending on the
/// memory management method of the underlying type and the thread access status.
///
/// # Manually-managed types
///
/// `Shared` references to manually-managed types, like `Ref<Node, Shared>`, act like raw
/// pointers. They are safe to alias, can be sent between threads, and can also be taken as
/// method arguments (converted from `Variant`). They can't be used directly. Instead, it's
/// required to obtain a safe view first. See the "Obtaining a safe view" section below for
/// more information.
///
/// `ThreadLocal` references to manually-managed types cannot normally be obtained, since
/// it does not add anything over `Shared` ones.
///
/// `Unique` references to manually-managed types, like `Ref<Node, Unique>`, can't be aliased
/// or sent between threads, but can be used safely. However, they *won't* be automatically
/// freed on drop, and are *leaked* if not passed to the engine or freed manually with `free`.
/// `Unique` references can be obtained through constructors safely, or `assume_unique` in
/// unsafe contexts.
///
/// # Reference-counted types
///
/// `Shared` references to reference-counted types, like `Ref<Reference, Shared>`, act like
/// `Arc` smart pointers. New references can be created with `Clone`, and they can be sent
/// between threads. The pointer is presumed to be always valid. As such, more operations
/// are available even when thread safety is not assumed. However, API methods still can't be
/// used directly, and users are required to obtain a safe view first. See the "Obtaining a
/// safe view" section below for more information.
///
/// `ThreadLocal` reference to reference-counted types, like `Ref<Reference, ThreadLocal>`, add
/// the ability to call API methods safely. Unlike `Unique` references, it's unsafe to convert
/// them to `Shared` because there might be other `ThreadLocal` references in existence.
///
/// # Obtaining a safe view
///
/// In a lot of cases, references obtained from the engine as return values or arguments aren't
/// safe to use, due to lack of pointer validity and thread safety guarantees in the API. As
/// such, it's usually required to use `unsafe` code to obtain safe views of the same object
/// before API methods can be called. The ways to cast between different reference types are as
/// follows:
///
/// | From | To | Method | Note |
/// | - | - | - | - |
/// | `Unique` | `&'a T` | `Deref` (API methods can be called directly) / `as_ref` | - |
/// | `ThreadLocal` | `&'a T` | `Deref` (API methods can be called directly) / `as_ref` | Only if `T` is a reference-counted type. |
/// | `Shared` | `&'a T` | `unsafe assume_safe::<'a>` | The underlying object must be valid, and exclusive to this thread during `'a`. |
/// | `Unique` | `ThreadLocal` | `into_thread_local` | - |
/// | `Unique` | `Shared` | `into_shared` | - |
/// | `Shared` | `ThreadLocal` | `unsafe assume_thread_local` | The reference must be local to the current thread. |
/// | `Shared` / `ThreadLocal` | `Unique` | `unsafe assume_unique` | The reference must be unique. |
/// | `ThreadLocal` | `Shared` | `unsafe assume_unique().into_shared()` | The reference must be unique. |
///
/// # Using as method arguments or return values
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
/// # Conditional trait implementations
///
/// Many trait implementations for `Ref` are conditional, dependent on the type parameters.
/// When viewing rustdoc documentation, you may expand the documentation on their respective
/// `impl` blocks  for more detailed explanations of the trait bounds.
pub struct Ref<T: GodotObject, Own: Ownership = Shared> {
    ptr: <T::Memory as MemorySpec>::PtrWrapper,
    _marker: PhantomData<(*const T, Own)>,
}

/// `Ref` is `Send` if the thread access is `Shared` or `Unique`.
unsafe impl<T: GodotObject, Own: Ownership + Send> Send for Ref<T, Own> {}

/// `Ref` is `Sync` if the thread access is `Shared`.
unsafe impl<T: GodotObject, Own: Ownership + Sync> Sync for Ref<T, Own> {}

/// `Ref` is `Copy` if the underlying object is manually-managed, and the access is not
/// `Unique`.
impl<T, Own> Copy for Ref<T, Own>
where
    T: GodotObject<Memory = ManuallyManaged>,
    Own: NonUniqueOwnership,
{
}

/// `Ref` is `Clone` if the access is not `Unique`.
impl<T, Own> Clone for Ref<T, Own>
where
    T: GodotObject,
    Own: NonUniqueOwnership,
{
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Ref::from_sys(self.ptr.as_non_null()) }
    }
}

impl<T: GodotObject + Instanciable> Ref<T, Unique> {
    /// Creates a new instance of `T`.
    ///
    /// The lifetime of the returned object is *not* automatically managed if `T` is a manually-
    /// managed type.
    #[inline]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        T::construct()
    }
}

impl<T: GodotObject> Ref<T, Unique> {
    /// Creates a new instance of a sub-class of `T` by its class name. Returns `None` if the
    /// class does not exist, cannot be constructed, has a different `Memory` from, or is not
    /// a sub-class of `T`.
    ///
    /// The lifetime of the returned object is *not* automatically managed if `T` is a manually-
    /// managed type. This means that if `Object` is used as the type parameter, any `Reference`
    /// objects created, if returned, will be leaked. As a result, such calls will return `None`.
    /// Casting between `Object` and `Reference` is possible on `TRef` and bare references.
    #[inline]
    pub fn by_class_name(class_name: &str) -> Option<Self> {
        unsafe {
            // Classes with NUL-bytes in their names can not exist
            let class_name = CString::new(class_name).ok()?;
            let ctor = (get_api().godot_get_class_constructor)(class_name.as_ptr())?;
            let ptr = NonNull::new(ctor() as *mut sys::godot_object)?;
            <T::Memory as MemorySpec>::impl_from_maybe_ref_counted(ptr)
        }
    }
}

/// Method for references that can be safely used.
impl<T: GodotObject, Own: Ownership> Ref<T, Own>
where
    RefImplBound: SafeDeref<T::Memory, Own>,
{
    /// Returns a safe temporary reference that tracks thread access.
    ///
    /// `Ref<T, Own>` can be safely dereferenced if either:
    ///
    /// - `T` is reference-counted and `Ownership` is not `Shared`,
    /// - or, `T` is manually-managed and `Ownership` is `Unique`.
    #[inline]
    pub fn as_ref(&self) -> TRef<'_, T, Own> {
        RefImplBound::impl_as_ref(self)
    }
}

/// `Ref<T, Own>` can be safely dereferenced if either:
///
/// - `T` is reference-counted and `Ownership` is not `Shared`,
/// - or, `T` is manually-managed and `Ownership` is `Unique`.
impl<T: GodotObject, Own: Ownership> Deref for Ref<T, Own>
where
    RefImplBound: SafeDeref<T::Memory, Own>,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        RefImplBound::impl_as_ref(self).obj
    }
}

/// `Ref<T, Own>` can be safely dereferenced if either:
///
/// - `T` is reference-counted and `Ownership` is not `Shared`,
/// - or, `T` is manually-managed and `Ownership` is `Unique`.
impl<T: GodotObject, Own: Ownership> Borrow<T> for Ref<T, Own>
where
    RefImplBound: SafeDeref<T::Memory, Own>,
{
    #[inline]
    fn borrow(&self) -> &T {
        RefImplBound::impl_as_ref(self).obj
    }
}

/// Methods for references that point to valid objects, but are not necessarily safe to use.
///
/// - All `Ref`s to reference-counted types always point to valid objects.
/// - `Ref` to manually-managed types are only guaranteed to be valid if `Unique`.
impl<T: GodotObject, Own: Ownership> Ref<T, Own>
where
    RefImplBound: SafeAsRaw<T::Memory, Own>,
{
    /// Cast to a `RawObject` reference safely. This is an internal interface.
    #[inline]
    #[doc(hidden)]
    pub fn as_raw(&self) -> &RawObject<T> {
        unsafe { self.as_raw_unchecked() }
    }

    /// Performs a dynamic reference cast to target type, keeping the reference count.
    /// Shorthand for `try_cast().ok()`.
    ///
    /// The `cast` method can only be used for downcasts. For statically casting to a
    /// supertype, use `upcast` instead.
    ///
    /// This is only possible between types with the same `Memory`s, since otherwise the
    /// reference can get leaked. Casting between `Object` and `Reference` is possible on
    /// `TRef` and bare references.
    #[inline]
    pub fn cast<U>(self) -> Option<Ref<U, Own>>
    where
        U: GodotObject<Memory = T::Memory> + SubClass<T>,
    {
        self.try_cast().ok()
    }

    /// Performs a static reference upcast to a supertype, keeping the reference count.
    /// This is guaranteed to be valid.
    ///
    /// This is only possible between types with the same `Memory`s, since otherwise the
    /// reference can get leaked. Casting between `Object` and `Reference` is possible on
    /// `TRef` and bare references.
    #[inline]
    pub fn upcast<U>(self) -> Ref<U, Own>
    where
        U: GodotObject<Memory = T::Memory>,
        T: SubClass<U>,
    {
        unsafe { self.cast_unchecked() }
    }

    /// Performs a dynamic reference cast to target type, keeping the reference count.
    ///
    /// This is only possible between types with the same `Memory`s, since otherwise the
    /// reference can get leaked. Casting between `Object` and `Reference` is possible on
    /// `TRef` and bare references.
    ///
    /// # Errors
    ///
    /// Returns `Err(self)` if the cast failed.
    #[inline]
    pub fn try_cast<U>(self) -> Result<Ref<U, Own>, Self>
    where
        U: GodotObject<Memory = T::Memory> + SubClass<T>,
    {
        if self.as_raw().is_class::<U>() {
            Ok(unsafe { self.cast_unchecked() })
        } else {
            Err(self)
        }
    }

    /// Performs an unchecked cast.
    unsafe fn cast_unchecked<U>(self) -> Ref<U, Own>
    where
        U: GodotObject<Memory = T::Memory>,
    {
        let ret = Ref::move_from_sys(self.ptr.as_non_null());
        std::mem::forget(self);
        ret
    }

    /// Performs a downcast to a `NativeClass` instance, keeping the reference count.
    /// Shorthand for `try_cast_instance().ok()`.
    ///
    /// The resulting `Instance` is not necessarily safe to use directly.
    #[inline]
    pub fn cast_instance<C>(self) -> Option<Instance<C, Own>>
    where
        C: NativeClass<Base = T>,
    {
        self.try_cast_instance().ok()
    }

    /// Performs a downcast to a `NativeClass` instance, keeping the reference count.
    ///
    /// # Errors
    ///
    /// Returns `Err(self)` if the cast failed.
    #[inline]
    pub fn try_cast_instance<C>(self) -> Result<Instance<C, Own>, Self>
    where
        C: NativeClass<Base = T>,
    {
        Instance::try_from_base(self)
    }
}

/// Methods for references that can't be used directly, and have to be assumed safe `unsafe`ly.
impl<T: GodotObject> Ref<T, Shared> {
    /// Assume that `self` is safe to use, returning a reference that can be used to call API
    /// methods.
    ///
    /// This is guaranteed to be a no-op at runtime if `debug_assertions` is disabled. Runtime
    /// sanity checks may be added in debug builds to help catch bugs.
    ///
    /// # Safety
    ///
    /// Suppose that the lifetime of the returned reference is `'a`. It's safe to call
    /// `assume_safe` only if:
    ///
    /// 1. During the entirety of `'a`, the underlying object will always be valid.
    ///
    ///     *This is always true for reference-counted types.* For them, the `'a` lifetime will
    ///     be constrained to the lifetime of `&self`.
    ///
    ///     This means that any methods called on the resulting reference will not free it,
    ///     unless it's the last operation within the lifetime.
    ///
    ///     If any script methods are called, the code ran as a consequence will also not free
    ///     it. This can happen via virtual method calls on other objects, or signals connected
    ///     in a non-deferred way.
    ///
    /// 2. During the entirety of 'a, the thread from which `assume_safe` is called has
    ///    exclusive access to the underlying object.
    ///
    ///     This is because all Godot objects have "interior mutability" in Rust parlance,
    ///     and can't be shared across threads. The best way to guarantee this is to follow
    ///     the official [thread-safety guidelines][thread-safety] across the codebase.
    ///
    /// Failure to satisfy either of the conditions will lead to undefined behavior.
    ///
    /// [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html
    #[inline(always)]
    pub unsafe fn assume_safe<'a, 'r>(&'r self) -> TRef<'a, T, Shared>
    where
        AssumeSafeLifetime<'a, 'r>: LifetimeConstraint<T::Memory>,
    {
        T::Memory::impl_assume_safe(self)
    }

    /// Assume that `self` is the unique reference to the underlying object.
    ///
    /// This is guaranteed to be a no-op at runtime if `debug_assertions` is disabled. Runtime
    /// sanity checks may be added in debug builds to help catch bugs.
    ///
    /// # Safety
    ///
    /// Calling `assume_unique` when `self` isn't the unique reference is instant undefined
    /// behavior. This is a much stronger assumption than `assume_safe` and should be used with
    /// care.
    #[inline(always)]
    pub unsafe fn assume_unique(self) -> Ref<T, Unique> {
        T::Memory::impl_assume_unique(self)
    }
}

/// Extra methods with explicit sanity checks for manually-managed unsafe references.
impl<T: GodotObject<Memory = ManuallyManaged>> Ref<T, Shared> {
    /// Returns `true` if the pointer currently points to a valid object of the correct type.
    /// **This does NOT guarantee that it's safe to use this pointer.**
    ///
    /// # Safety
    ///
    /// This thread must have exclusive access to the object during the call.
    #[inline]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub unsafe fn is_instance_sane(&self) -> bool {
        let api = get_api();
        if !(api.godot_is_instance_valid)(self.as_ptr()) {
            return false;
        }

        self.as_raw_unchecked().is_class::<T>()
    }

    /// Assume that `self` is safe to use, if a sanity check using `is_instance_sane` passed.
    ///
    /// # Safety
    ///
    /// The same safety constraints as `assume_safe` applies. **The sanity check does NOT
    /// guarantee that the operation is safe.**
    #[inline]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub unsafe fn assume_safe_if_sane<'a>(&self) -> Option<TRef<'a, T, Shared>> {
        if self.is_instance_sane() {
            Some(self.assume_safe_unchecked())
        } else {
            None
        }
    }

    /// Assume that `self` is the unique reference to the underlying object, if a sanity check
    /// using `is_instance_sane` passed.
    ///
    /// # Safety
    ///
    /// Calling `assume_unique_if_sane` when `self` isn't the unique reference is instant
    /// undefined behavior. This is a much stronger assumption than `assume_safe` and should be
    /// used with care.
    #[inline]
    pub unsafe fn assume_unique_if_sane(self) -> Option<Ref<T, Unique>> {
        if self.is_instance_sane() {
            Some(self.cast_access())
        } else {
            None
        }
    }
}

/// Methods for conversion from `Shared` to `ThreadLocal` access. This is only available for
/// reference-counted types.
impl<T: GodotObject<Memory = RefCounted>> Ref<T, Shared> {
    /// Assume that all references to the underlying object is local to the current thread.
    ///
    /// This is guaranteed to be a no-op at runtime.
    ///
    /// # Safety
    ///
    /// Calling `assume_thread_local` when there are references on other threads is instant
    /// undefined behavior. This is a much stronger assumption than `assume_safe` and should
    /// be used with care.
    #[inline(always)]
    pub unsafe fn assume_thread_local(self) -> Ref<T, ThreadLocal> {
        self.cast_access()
    }
}

/// Methods for conversion from `Unique` to `ThreadLocal` access. This is only available for
/// reference-counted types.
impl<T: GodotObject<Memory = RefCounted>> Ref<T, Unique> {
    /// Convert to a thread-local reference.
    ///
    /// This is guaranteed to be a no-op at runtime.
    #[inline(always)]
    pub fn into_thread_local(self) -> Ref<T, ThreadLocal> {
        unsafe { self.cast_access() }
    }
}

/// Methods for conversion from `Unique` to `Shared` access.
impl<T: GodotObject> Ref<T, Unique> {
    /// Convert to a shared reference.
    ///
    /// This is guaranteed to be a no-op at runtime.
    #[inline(always)]
    pub fn into_shared(self) -> Ref<T, Shared> {
        unsafe { self.cast_access() }
    }
}

/// Methods for freeing `Unique` references to manually-managed objects.
impl<T: GodotObject<Memory = ManuallyManaged>> Ref<T, Unique> {
    /// Manually frees the object.
    ///
    /// Manually-managed objects are not free-on-drop *even when the access is unique*, because
    /// it's impossible to know whether methods take "ownership" of them or not. It's up to the
    /// user to decide when they should be freed.
    ///
    /// This is only available for `Unique` references. If you have a `Ref` with another access,
    /// and you are sure that it is unique, use `assume_unique` to convert it to a `Unique` one.
    #[inline]
    pub fn free(self) {
        unsafe {
            self.as_raw().free();
        }
    }
}

/// Methods for freeing `Unique` references to manually-managed objects.
impl<T: GodotObject<Memory = ManuallyManaged> + QueueFree> Ref<T, Unique> {
    /// Queues the object for deallocation in the near future. This is preferable for `Node`s
    /// compared to `Ref::free`.
    ///
    /// This is only available for `Unique` references. If you have a `Ref` with another access,
    /// and you are sure that it is unique, use `assume_unique` to convert it to a `Unique` one.
    #[inline]
    pub fn queue_free(self) {
        unsafe { T::godot_queue_free(self.as_ptr()) }
    }
}

/// Reference equality.
impl<T: GodotObject, Own: Ownership> Eq for Ref<T, Own> {}

/// Reference equality.
impl<T, Own, RhsOws> PartialEq<Ref<T, RhsOws>> for Ref<T, Own>
where
    T: GodotObject,
    Own: Ownership,
    RhsOws: Ownership,
{
    #[inline]
    fn eq(&self, other: &Ref<T, RhsOws>) -> bool {
        self.ptr.as_non_null() == other.ptr.as_non_null()
    }
}

/// Ordering of the raw pointer value.
impl<T: GodotObject, Own: Ownership> Ord for Ref<T, Own> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.as_non_null().cmp(&other.ptr.as_non_null())
    }
}

/// Ordering of the raw pointer value.
impl<T: GodotObject, Own: Ownership> PartialOrd for Ref<T, Own> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ptr.as_non_null().partial_cmp(&other.ptr.as_non_null())
    }
}

/// Hashes the raw pointer.
impl<T: GodotObject, Own: Ownership> Hash for Ref<T, Own> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.ptr.as_ptr() as usize)
    }
}

impl<T: GodotObject, Own: Ownership> Debug for Ref<T, Own> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({:p})", T::class_name(), self.ptr.as_ptr())
    }
}

impl<T: GodotObject, Own: Ownership> Ref<T, Own> {
    /// Convert to a nullable raw pointer.
    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *mut sys::godot_object {
        self.ptr.as_ptr()
    }

    /// Convert to a nullable raw pointer.
    #[doc(hidden)]
    #[inline]
    pub fn as_ptr(&self) -> *mut sys::godot_object {
        self.ptr.as_ptr()
    }

    /// Convert to a `RawObject` reference.
    ///
    /// # Safety
    ///
    /// `self` must point to a valid object of the correct type.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn as_raw_unchecked<'a>(&self) -> &'a RawObject<T> {
        RawObject::from_sys_ref_unchecked(self.ptr.as_non_null())
    }

    /// Convert from a pointer returned from a ptrcall. For reference-counted types, this takes
    /// the ownership of the returned reference, in Rust parlance. For non-reference-counted
    /// types, its behavior should be exactly the same as `from_sys`. This is needed for
    /// reference-counted types to be properly freed, since any references returned from
    /// ptrcalls are leaked in the process of being cast into a pointer.
    ///
    /// # Safety
    ///
    /// `obj` must point to a valid object of the correct type.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn move_from_sys(obj: NonNull<sys::godot_object>) -> Self {
        Ref {
            ptr: <T::Memory as MemorySpec>::PtrWrapper::new(obj),
            _marker: PhantomData,
        }
    }

    /// Convert from a raw pointer, incrementing the reference counter if reference-counted.
    ///
    /// # Safety
    ///
    /// `obj` must point to a valid object of the correct type.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn from_sys(obj: NonNull<sys::godot_object>) -> Self {
        let ret = Self::move_from_sys(obj);
        <T::Memory as MemorySpec>::maybe_add_ref(ret.as_raw_unchecked());
        ret
    }

    /// Convert from a pointer returned from a constructor of a reference-counted type. For
    /// non-reference-counted types, its behavior should be exactly the same as `from_sys`.
    ///
    /// # Safety
    ///
    /// `obj` must point to a valid object of the correct type, and must be the only reference.
    #[doc(hidden)]
    #[inline]
    pub unsafe fn init_from_sys(obj: NonNull<sys::godot_object>) -> Self {
        let ret = Self::move_from_sys(obj);
        <T::Memory as MemorySpec>::maybe_init_ref(ret.as_raw_unchecked());
        ret
    }

    /// Casts the access type of `self` to `TargetAccess`, moving the reference.
    ///
    /// # Safety
    ///
    /// The cast must be valid.
    unsafe fn cast_access<TargetOws: Ownership>(self) -> Ref<T, TargetOws> {
        let ret = Ref::move_from_sys(self.ptr.as_non_null());
        std::mem::forget(self);
        ret
    }

    /// Assume that the reference is safe in an `unsafe` context even if it can be used safely.
    /// For internal use in macros.
    ///
    /// This is guaranteed to be a no-op at runtime.
    ///
    /// # Safety
    ///
    /// The same safety constraints as `assume_safe` applies.
    #[doc(hidden)]
    #[inline(always)]
    pub unsafe fn assume_safe_unchecked<'a>(&self) -> TRef<'a, T, Own> {
        TRef::new(T::cast_ref(self.as_raw_unchecked()))
    }
}

/// A temporary safe pointer to Godot objects that tracks thread access status. `TRef` can be
/// coerced into bare references with `Deref`.
///
/// See the type-level documentation on `Ref` for detailed documentation on the reference
/// system of `godot-rust`.
///
/// # Using as method arguments or return values
///
/// `TRef<T, Shared>` can be passed into methods.
///
/// # Using as `owner` arguments in NativeScript methods
///
/// It's possible to use `TRef` as the `owner` argument in NativeScript methods. This can make
/// passing `owner` to methods easier.
pub struct TRef<'a, T: GodotObject, Own: Ownership = Shared> {
    obj: &'a T,
    _marker: PhantomData<Own>,
}

impl<'a, T: GodotObject, Own: Ownership> Copy for TRef<'a, T, Own> {}
impl<'a, T: GodotObject, Own: Ownership> Clone for TRef<'a, T, Own> {
    #[inline]
    fn clone(&self) -> Self {
        TRef::new(self.obj)
    }
}

impl<'a, T: GodotObject, Own: Ownership> Debug for TRef<'a, T, Own> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({:p})", T::class_name(), self.obj)
    }
}

impl<'a, T: GodotObject, Own: Ownership> Deref for TRef<'a, T, Own> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.obj
    }
}

impl<'a, T: GodotObject, Own: Ownership> AsRef<T> for TRef<'a, T, Own> {
    #[inline]
    fn as_ref(&self) -> &T {
        self.obj
    }
}

impl<'a, T: GodotObject, Own: Ownership> Borrow<T> for TRef<'a, T, Own> {
    #[inline]
    fn borrow(&self) -> &T {
        self.obj
    }
}

impl<'a, T: GodotObject, Own: Ownership> TRef<'a, T, Own> {
    pub(crate) fn new(obj: &'a T) -> Self {
        TRef {
            obj,
            _marker: PhantomData,
        }
    }

    /// Returns the underlying reference without thread access.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(self) -> &'a T {
        self.obj
    }

    /// Performs a dynamic reference cast to target type, keeping the thread access info.
    #[inline]
    pub fn cast<U>(self) -> Option<TRef<'a, U, Own>>
    where
        U: GodotObject + SubClass<T>,
    {
        self.obj.cast().map(TRef::new)
    }

    /// Performs a static reference upcast to a supertype that is guaranteed to be valid,
    /// keeping the thread access info.
    ///
    /// This is guaranteed to be a no-op at runtime.
    #[inline(always)]
    pub fn upcast<U>(&self) -> TRef<'a, U, Own>
    where
        U: GodotObject,
        T: SubClass<U>,
    {
        TRef::new(self.obj.upcast())
    }

    /// Convenience method to downcast to `TInstance` where `self` is the base object.
    #[inline]
    pub fn cast_instance<C>(self) -> Option<TInstance<'a, C, Own>>
    where
        C: NativeClass<Base = T>,
    {
        TInstance::try_from_base(self)
    }
}

impl<'a, Kind, T, Own> TRef<'a, T, Own>
where
    Kind: Memory,
    T: GodotObject<Memory = Kind>,
    Own: NonUniqueOwnership,
{
    /// Persists this reference into a persistent `Ref` with the same thread access.
    ///
    /// This is only available for non-`Unique` accesses.
    #[inline]
    pub fn claim(self) -> Ref<T, Own> {
        unsafe { Ref::from_sys(self.obj.as_raw().sys()) }
    }
}

impl<'a, T: GodotObject> TRef<'a, T, Shared> {
    /// Recovers a instance ID previously returned by `Object::get_instance_id` if the object is
    /// still alive.
    ///
    /// # Safety
    ///
    /// During the entirety of `'a`, the thread from which `try_from_instance_id` is called must
    /// have exclusive access to the underlying object, if it is still alive.
    #[inline]
    pub unsafe fn try_from_instance_id(id: i64) -> Option<Self> {
        let api = get_api();
        let ptr = NonNull::new((api.godot_instance_from_id)(id as sys::godot_int))?;
        let raw = RawObject::try_from_sys_ref(ptr)?;
        Some(TRef::new(T::cast_ref(raw)))
    }

    /// Recovers a instance ID previously returned by `Object::get_instance_id` if the object is
    /// still alive, and panics otherwise. This does **NOT** guarantee that the resulting
    /// reference is safe to use.
    ///
    /// # Panics
    ///
    /// Panics if the given id refers to a destroyed object. For a non-panicking version, see
    /// `try_from_instance_id`.
    ///
    /// # Safety
    ///
    /// During the entirety of `'a`, the thread from which `try_from_instance_id` is called must
    /// have exclusive access to the underlying object, if it is still alive.
    #[inline]
    pub unsafe fn from_instance_id(id: i64) -> Self {
        Self::try_from_instance_id(id).expect("instance should be alive")
    }
}

/// Represents an explicit null reference in method arguments. This works around type inference
/// issues with `Option`. You may create `Null`s with `Null::null` or `GodotObject::null`.
pub struct Null<T>(PhantomData<T>);

impl<T: GodotObject> Null<T> {
    /// Creates an explicit null reference that can be used as a method argument.
    // TODO(#997) consider something more idiomatic, like module::null::<T>(), similar to std::ptr::null()
    #[inline]
    #[allow(clippy::self_named_constructors)]
    pub fn null() -> Self {
        Null(PhantomData)
    }
}
