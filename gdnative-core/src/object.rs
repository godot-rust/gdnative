use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::{self, NonNull};

use crate::private::get_api;
use crate::sys;
use crate::GodotString;
use crate::ObjectMethodTable;
use crate::ToVariant;

#[cfg(feature = "nativescript")]
use crate::nativescript::{Instance, NativeClass, RefInstance};

/// Trait for Godot API objects. This trait is sealed, and implemented for generated wrapper
/// types.
///
/// # Remarks
///
/// The `cast` method on Godot object types is only for conversion between engine types.
/// To downcast a `NativeScript` type from its base type, see `Instance::try_from_base`.
pub unsafe trait GodotObject:
    Sized + ToVariant + crate::private::godot_object::Sealed
{
    /// The "persistent" form of this type. For reference-counted classes, this is `Ref<Self>`.
    /// For manually-managed classes, this is `Ptr<Self>` instead.
    type PersistentRef: PersistentRef<Target = Self>;

    fn class_name() -> &'static str;

    /// Performs a dynamic reference cast to target type.
    #[inline]
    fn cast<T: GodotObject>(&self) -> Option<&T> {
        self.as_raw().cast().map(T::cast_ref)
    }

    /// Convenience method to downcast to `RefInstance` where `self` is the base object.
    #[inline]
    #[cfg(feature = "nativescript")]
    fn cast_instance<T>(&self) -> Option<RefInstance<'_, T>>
    where
        T: NativeClass<Base = Self>,
    {
        RefInstance::try_from_base(self)
    }

    /// Creates a reference to `Self` given a `RawObject` reference.
    #[doc(hidden)]
    #[inline]
    fn cast_ref(raw: &RawObject<Self>) -> &Self {
        unsafe { &*(raw as *const _ as *const _) }
    }

    #[doc(hidden)]
    #[inline]
    fn as_raw(&self) -> &RawObject<Self> {
        unsafe { &*(self as *const _ as *const _) }
    }

    #[doc(hidden)]
    #[inline]
    fn as_ptr(&self) -> *mut sys::godot_object {
        self.as_raw().sys().as_ptr()
    }

    /// Creates a wrapper around the same Godot object that has `'static` lifetime.
    ///
    /// Most Godot APIs expect object arguments with `'static` lifetime. This method may be used
    /// to produce a `'static` wrapper given a reference. For reference-counted types, or classes
    /// that extend `Reference`, this increments the reference count. For manually-managed types,
    /// including all classes that inherit `Node`, this creates an alias.
    #[inline]
    fn claim(&self) -> Self::PersistentRef
    where
        Self: Sized,
    {
        unsafe { Self::PersistentRef::from_sys(self.as_raw().sys()) }
    }
}

/// Marker trait for reference-counted Godot API objects.
pub unsafe trait RefCounted: GodotObject {}

/// Marker trait for manually-managed Godot API objects.
pub unsafe trait ManuallyManaged: GodotObject {}

/// Trait for persistent references to Godot API objects. This trait is sealed and meant to be
/// an internal interface.
pub trait PersistentRef: Clone + Debug + ToVariant + private::Sealed {
    type Target;

    #[doc(hidden)]
    fn sys(&self) -> *mut sys::godot_object;

    /// Convert to a `RawObject` reference.
    ///
    /// # Safety
    ///
    /// `self` must point to a valid object of the correct type.
    #[doc(hidden)]
    unsafe fn as_raw(&self) -> &RawObject<Self::Target>;

    /// Convert from a raw pointer, incrementing the reference counter if reference-counted.
    ///
    /// # Safety
    ///
    /// `obj` must point to a valid object of the correct type.
    #[doc(hidden)]
    unsafe fn from_sys(obj: NonNull<sys::godot_object>) -> Self;

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
    unsafe fn move_from_sys(obj: NonNull<sys::godot_object>) -> Self;

    /// Convert from a pointer returned from a constructor of a reference-counted type. For
    /// non-reference-counted types, its behavior should be exactly the same as `from_sys`.
    ///
    /// # Safety
    ///
    /// `obj` must point to a valid object of the correct type, and must be the only reference.
    #[doc(hidden)]
    unsafe fn init_from_sys(obj: NonNull<sys::godot_object>) -> Self;
}

/// GodotObjects that have a zero argument constructor.
pub trait Instanciable: GodotObject {
    fn construct() -> Self::PersistentRef;
}

/// Manually managed Godot classes implementing `queue_free`. This trait has no public
/// interface. See `Ptr::queue_free`.
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

/// A pointer to a manually-managed Godot object. This is semantically equivalent to a non-null
/// raw pointer, except that it also implements `Send` and `Sync`.
///
/// It's impossible to call API methods directly on `Ptr`s. In order to obtain a safe view
/// to the underlying object, see the `assume_safe` and `assume_safe_during` methods and follow
/// the safety guidelines there.
pub struct Ptr<T: GodotObject + ManuallyManaged> {
    ptr: NonNull<sys::godot_object>,
    _marker: PhantomData<*const T>,
}

unsafe impl<T: GodotObject + ManuallyManaged> Send for Ptr<T> {}
unsafe impl<T: GodotObject + ManuallyManaged> Sync for Ptr<T> {}

impl<T: GodotObject + ManuallyManaged> Copy for Ptr<T> {}
impl<T: GodotObject + ManuallyManaged> Clone for Ptr<T> {
    #[inline]
    fn clone(&self) -> Self {
        Ptr {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T: GodotObject<PersistentRef = Self> + ManuallyManaged + Instanciable> Ptr<T> {
    /// Creates a new instance of `T`.
    ///
    /// The lifetime of the returned object is *not* automatically managed.
    ///
    /// Immediately after creation, the object is owned by the caller, and can be
    /// passed to the engine (in which case the engine will be responsible for
    /// destroying the object) or destroyed manually using `Ptr::free`, or preferably
    /// `Ptr::queue_free` if it is a `Node`.
    #[inline]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        T::construct()
    }
}

impl<T: GodotObject + ManuallyManaged> Ptr<T> {
    unsafe fn as_raw<'a>(self) -> &'a RawObject<T> {
        RawObject::from_sys_ref_unchecked(self.ptr)
    }

    /// Returns the underlying raw pointer. This is an internal interface.
    #[doc(hidden)]
    #[inline]
    pub fn as_ptr(self) -> *mut sys::godot_object {
        self.ptr.as_ptr()
    }

    /// Returns `true` if the pointer currently points to a valid object of the correct type.
    /// **This does NOT guarantee that it's safe to use this pointer.**
    ///
    /// # Safety
    ///
    /// This thread must have exclusive access to the object during the call.
    #[inline]
    pub unsafe fn is_instance_sane(self) -> bool {
        let api = get_api();
        if !(api.godot_is_instance_valid)(self.as_ptr()) {
            return false;
        }

        self.as_raw().is_class::<T>()
    }

    /// Assume that `self` is safe to use during the `'a` lifetime. This lifetime is unbounded
    /// and inferred by the compiler unless given explicitly.
    ///
    /// This is guaranteed to be a no-op at runtime if `debug_assertions` is disabled. Runtime
    /// sanity checks may be added in debug builds to help catch bugs.
    ///
    /// # Safety
    ///
    /// It's safe to call `assume_safe` only if:
    ///
    /// 1. During the entirety of `'a`, the underlying object will always be valid.
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
    pub unsafe fn assume_safe<'a>(self) -> &'a T {
        debug_assert!(
            self.is_instance_sane(),
            "assume_safe called on an invalid pointer"
        );
        self.assume_safe_unchecked::<'a>()
    }

    /// Assume that `self` is safe to use during the `'a` lifetime, if a sanity check using
    /// `is_instance_sane` passed. This lifetime is unbounded and inferred by the compiler
    /// unless given explicitly.
    ///
    /// # Safety
    ///
    /// The same safety constraints as `assume_safe` applies. **The sanity check does NOT
    /// guarantee that the operation is safe.**
    #[inline]
    pub unsafe fn assume_safe_if_sane<'a>(self) -> Option<&'a T> {
        if self.is_instance_sane() {
            Some(self.assume_safe_unchecked())
        } else {
            None
        }
    }

    #[inline(always)]
    unsafe fn assume_safe_unchecked<'a>(self) -> &'a T {
        T::cast_ref(self.as_raw())
    }

    /// Assume that `self` is safe to use during the lifetime of the input reference. The
    /// input reference is unused and may be anything. This is a convenience wrapper around
    /// `assume_safe`.
    ///
    /// This is guaranteed to be a no-op at runtime if `debug_assertions` is disabled. Runtime
    /// sanity checks may be added in debug builds to help catch bugs.
    ///
    /// # Safety
    ///
    /// The same safety constraints as `assume_safe` applies.
    #[inline(always)]
    pub unsafe fn assume_safe_during<'a, L: 'a>(self, _lifetime: &'a L) -> &'a T {
        self.assume_safe::<'a>()
    }

    /// Assume that `self` is safe to use during the lifetime of the input reference, if a
    /// sanity check using `is_instance_sane` passed. The input reference is unused and may
    /// be anything. This is a convenience wrapper around `assume_safe_if_sane`.
    ///
    /// # Safety
    ///
    /// The same safety constraints as `assume_safe` applies. **The sanity check does NOT
    /// guarantee that the operation is safe.**
    #[inline(always)]
    pub unsafe fn assume_safe_during_if_sane<'a, L: 'a>(self, _lifetime: &'a L) -> Option<&'a T> {
        self.assume_safe_if_sane::<'a>()
    }

    /// Manually frees the object.
    ///
    /// # Safety
    ///
    /// During the call, the underlying object must be valid, and this thread must have
    /// exclusive access to the object. This pointer must never be used again.
    #[inline]
    pub unsafe fn free(self) {
        self.as_raw().free();
    }
}

impl<T: GodotObject + ManuallyManaged + QueueFree> Ptr<T> {
    /// Queues the object for deallocation in the near future. This is preferable for `Node`s
    /// compared to `Ptr::free`.
    ///
    /// # Safety
    ///
    /// During the call, the underlying object must be valid, and this thread must have
    /// exclusive access to the object. This pointer must never be used again.
    #[inline]
    pub unsafe fn queue_free(self) {
        T::godot_queue_free(self.as_ptr())
    }
}

impl<T: GodotObject + ManuallyManaged> private::Sealed for Ptr<T> {}
impl<T: GodotObject + ManuallyManaged> PersistentRef for Ptr<T> {
    type Target = T;

    #[inline]
    fn sys(&self) -> *mut sys::godot_object {
        self.ptr.as_ptr()
    }

    #[inline]
    unsafe fn as_raw(&self) -> &RawObject<Self::Target> {
        RawObject::from_sys_ref_unchecked(self.ptr)
    }

    #[inline]
    unsafe fn from_sys(obj: NonNull<sys::godot_object>) -> Self {
        Ptr {
            ptr: obj,
            _marker: PhantomData,
        }
    }

    #[inline]
    unsafe fn move_from_sys(obj: NonNull<sys::godot_object>) -> Self {
        Self::from_sys(obj)
    }

    #[inline]
    unsafe fn init_from_sys(obj: NonNull<sys::godot_object>) -> Self {
        Self::from_sys(obj)
    }
}

impl<T: GodotObject + ManuallyManaged> Eq for Ptr<T> {}
impl<T: GodotObject + ManuallyManaged> PartialEq for Ptr<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<T: GodotObject + ManuallyManaged> Ord for Ptr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.cmp(&other.ptr)
    }
}

impl<T: GodotObject + ManuallyManaged> PartialOrd for Ptr<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ptr.partial_cmp(&other.ptr)
    }
}

impl<T: GodotObject + ManuallyManaged> Hash for Ptr<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.ptr.as_ptr() as usize)
    }
}

impl<T: GodotObject + ManuallyManaged> Debug for Ptr<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({:p})", T::class_name(), self.ptr)
    }
}

/// A reference to a reference-counted Godot object. This is semantically analogous to a `Rc`
/// wrapper.
///
/// It's possible to call API methods in safe contexts directly on `Ref`.
pub struct Ref<T: GodotObject + RefCounted> {
    ptr: NonNull<sys::godot_object>,
    _marker: PhantomData<*const T>,
}

impl<T: GodotObject + RefCounted> Deref for Ref<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { T::cast_ref(self.as_raw()) }
    }
}

impl<T: GodotObject + RefCounted> AsRef<T> for Ref<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        unsafe { T::cast_ref(self.as_raw()) }
    }
}

impl<T: GodotObject + RefCounted> Clone for Ref<T> {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Self::from_sys(self.ptr) }
    }
}

impl<T: GodotObject + RefCounted> Drop for Ref<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.as_raw().unref_and_free_if_last();
        }
    }
}

impl<T: GodotObject<PersistentRef = Self> + RefCounted + Instanciable> Ref<T> {
    /// Creates a new instance of `T`.
    ///
    /// The returned object is automatically managed for reference-counted types.
    #[inline]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        T::construct()
    }
}

impl<T: GodotObject<PersistentRef = Self> + RefCounted> Ref<T> {
    /// Performs a dynamic reference cast to target type, keeping the reference count.
    #[inline]
    pub fn cast<U: GodotObject>(self) -> Option<Ref<U>>
    where
        U: GodotObject + RefCounted,
    {
        unsafe {
            if self.as_raw().is_class::<U>() {
                let ret = Some(Ref::move_from_sys(self.ptr));
                std::mem::forget(self);
                ret
            } else {
                None
            }
        }
    }

    /// Performs a downcast to a `NativeClass` instance, keeping the reference count.
    #[inline]
    pub fn cast_instance<C>(self) -> Option<Instance<C>>
    where
        C: NativeClass<Base = T>,
    {
        Instance::try_from_ref_base(self)
    }
}

impl<T: GodotObject + RefCounted> private::Sealed for Ref<T> {}
impl<T: GodotObject + RefCounted> PersistentRef for Ref<T> {
    type Target = T;

    #[inline]
    fn sys(&self) -> *mut sys::godot_object {
        self.ptr.as_ptr()
    }

    #[inline]
    unsafe fn as_raw(&self) -> &RawObject<Self::Target> {
        RawObject::from_sys_ref_unchecked(self.ptr)
    }

    #[inline]
    unsafe fn from_sys(obj: NonNull<sys::godot_object>) -> Self {
        let ret = Self::move_from_sys(obj);
        ret.as_raw().add_ref();
        ret
    }

    #[inline]
    unsafe fn move_from_sys(obj: NonNull<sys::godot_object>) -> Self {
        Ref {
            ptr: obj,
            _marker: PhantomData,
        }
    }

    #[inline]
    unsafe fn init_from_sys(obj: NonNull<sys::godot_object>) -> Self {
        let ret = Self::move_from_sys(obj);
        ret.as_raw().init_ref_count();
        ret
    }
}

impl<T: GodotObject + RefCounted> Debug for Ref<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({:p})", T::class_name(), self.ptr)
    }
}

/// An opaque struct representing Godot objects. This should never be created on the stack.
///
/// This is an internal interface. Users are expected to use references to named generated types
/// instead.
#[repr(C)]
pub struct RawObject<T> {
    _opaque: [u8; 0],
    _marker: PhantomData<T>,
}

impl<T: GodotObject> RawObject<T> {
    /// Creates a typed reference from a pointer, without checking the type of the pointer.
    ///
    /// # Safety
    ///
    /// The `obj` pointer must be pointing to a valid Godot object of type `T` during the
    /// entirety of `'a`.
    #[inline]
    pub unsafe fn from_sys_ref_unchecked<'a>(obj: NonNull<sys::godot_object>) -> &'a Self {
        &*(obj.as_ptr() as *mut Self)
    }

    /// Creates a typed reference from a pointer if the pointer is pointing to an object of
    /// the correct type. Returns `None` otherwise.
    ///
    /// # Safety
    ///
    /// The `obj` pointer must be pointing to a valid Godot object during the entirety of `'a`.
    #[inline]
    pub unsafe fn try_from_sys_ref<'a>(obj: NonNull<sys::godot_object>) -> Option<&'a Self> {
        if ptr_is_class(obj.as_ptr(), T::class_name()) {
            Some(Self::from_sys_ref_unchecked(obj))
        } else {
            None
        }
    }

    /// Casts a reference to this opaque object to `*const sys::godot_object`.
    #[inline]
    pub fn sys(&self) -> NonNull<sys::godot_object> {
        // SAFETY: references should never be null
        unsafe { NonNull::new_unchecked(self as *const _ as *mut _) }
    }

    /// Checks whether the object is of a certain Godot class.
    #[inline]
    pub fn is_class<U: GodotObject>(&self) -> bool {
        self.is_class_by_name(U::class_name())
    }

    /// Checks whether the object is of a certain Godot class by name.
    #[inline]
    pub fn is_class_by_name(&self, class_name: &str) -> bool {
        unsafe { ptr_is_class(self.sys().as_ptr(), class_name) }
    }

    /// Returns the class name of this object dynamically using `Object::get_class`.
    #[inline]
    pub fn class_name(&self) -> String {
        let api = crate::private::get_api();
        let get_class_method = ObjectMethodTable::get(api).get_class;
        let mut argument_buffer = [ptr::null() as *const libc::c_void; 0];
        let mut class_name = sys::godot_string::default();
        let ret_ptr = &mut class_name as *mut sys::godot_string;

        unsafe {
            (api.godot_method_bind_ptrcall)(
                get_class_method,
                self.sys().as_ptr(),
                argument_buffer.as_mut_ptr() as *mut _,
                ret_ptr as *mut _,
            );
        }

        let string = GodotString::from_sys(class_name);
        string.to_string()
    }

    /// Attempt to cast a Godot object to a different class type.
    #[inline]
    pub fn cast<U>(&self) -> Option<&RawObject<U>>
    where
        U: GodotObject,
    {
        unsafe { RawObject::try_from_sys_ref(self.sys()) }
    }

    /// Attempt to cast a Godot object to a different class type without checking the type at
    /// runtime.
    ///
    /// # Safety
    ///
    /// The types must be compatible.
    #[inline]
    pub unsafe fn cast_unchecked<U>(&self) -> &RawObject<U>
    where
        U: GodotObject,
    {
        RawObject::from_sys_ref_unchecked(self.sys())
    }

    /// Free the underlying object.
    ///
    /// # Safety
    ///
    /// Further operations must not be performed on the same reference.
    #[inline]
    pub unsafe fn free(&self) {
        (get_api().godot_object_destroy)(self.sys().as_ptr());
    }
}

impl<T: GodotObject + RefCounted> RawObject<T> {
    /// Increase the reference count of the object.
    #[inline]
    pub fn add_ref(&self) {
        use crate::ReferenceMethodTable;

        let api = crate::private::get_api();
        let addref_method = ReferenceMethodTable::get(api).reference;
        let mut argument_buffer = [ptr::null() as *const libc::c_void; 0];
        let mut ok = false;
        let ok_ptr = &mut ok as *mut bool;

        unsafe {
            (api.godot_method_bind_ptrcall)(
                addref_method,
                self.sys().as_ptr(),
                argument_buffer.as_mut_ptr() as *mut _,
                ok_ptr as *mut _,
            );
        }

        // If this assertion blows up it means there is a reference counting bug
        // and we tried to increment the ref count of a dead object (who's ref
        // count is equal to zero).
        debug_assert!(ok);
    }

    /// Decrease the reference count of the object. Returns `true` if this is the last
    /// reference.
    ///
    /// # Safety
    ///
    /// Further operations must not be performed on the same reference if this is the last
    /// reference.
    #[inline]
    pub unsafe fn unref(&self) -> bool {
        use crate::ReferenceMethodTable;

        let api = crate::private::get_api();
        let unref_method = ReferenceMethodTable::get(api).unreference;
        let mut argument_buffer = [ptr::null() as *const libc::c_void; 0];
        let mut last_reference = false;
        let ret_ptr = &mut last_reference as *mut bool;
        (api.godot_method_bind_ptrcall)(
            unref_method,
            self.sys().as_ptr(),
            argument_buffer.as_mut_ptr() as *mut _,
            ret_ptr as *mut _,
        );

        last_reference
    }

    /// Decrease the reference count of the object. Frees the object and returns `true` if this
    /// is the last reference.
    ///
    /// # Safety
    ///
    /// Further operations must not be performed on the same reference if this is the last
    /// reference.
    #[inline]
    pub unsafe fn unref_and_free_if_last(&self) -> bool {
        let last_reference = self.unref();

        if last_reference {
            self.free();
        }

        last_reference
    }

    /// Initialize the reference count of the object.
    ///
    /// # Safety
    ///
    /// This function assumes that no other references are held at the time.
    #[inline]
    pub unsafe fn init_ref_count(&self) {
        use crate::ReferenceMethodTable;

        let obj = self.sys().as_ptr();

        let api = crate::private::get_api();
        let init_method = ReferenceMethodTable::get(api).init_ref;
        let mut argument_buffer = [ptr::null() as *const libc::c_void; 0];
        let mut ok = false;
        let ret_ptr = &mut ok as *mut bool;
        (api.godot_method_bind_ptrcall)(
            init_method,
            obj,
            argument_buffer.as_mut_ptr() as *mut _,
            ret_ptr as *mut _,
        );

        debug_assert!(ok);
    }
}

impl<T: GodotObject> Debug for RawObject<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({:p})", T::class_name(), self.sys())
    }
}

/// Checks whether the raw object pointer is of a certain Godot class.
///
/// # Safety
///
/// The `obj` pointer must be pointing to a valid Godot object.
#[inline]
unsafe fn ptr_is_class(obj: *mut sys::godot_object, class_name: &str) -> bool {
    let api = crate::private::get_api();
    let method_bind = ObjectMethodTable::get(api).is_class;

    let mut class_name = (api.godot_string_chars_to_utf8_with_len)(
        class_name.as_ptr() as *const _,
        class_name.len() as _,
    );

    let mut argument_buffer = [ptr::null() as *const libc::c_void; 1];
    argument_buffer[0] = (&class_name) as *const _ as *const _;

    let mut ret = false;
    let ret_ptr = &mut ret as *mut _;
    (api.godot_method_bind_ptrcall)(
        method_bind,
        obj,
        argument_buffer.as_mut_ptr() as *mut _,
        ret_ptr as *mut _,
    );

    (api.godot_string_destroy)(&mut class_name);

    ret
}

mod private {
    pub trait Sealed {}
}
