use std::fmt;

use gdnative_impl_proc_macros as macros;

use crate::access::{Aligned, MaybeUnaligned};
use crate::private::get_api;
use crate::{Color, GodotString, VariantArray, Vector2, Vector2Godot, Vector3, Vector3Godot};

/// A reference-counted typed vector using Godot's pool allocator, generic over possible
/// element types.
pub struct TypedArray<T: Element> {
    inner: T::SysArray,
}

/// A RAII read access for Godot typed arrays.
pub type Read<'a, T> = Aligned<ReadGuard<'a, T>>;

/// A RAII write access for Godot typed arrays. This will only lock the CoW container once,
/// as opposed to every time with methods like `push`.
pub type Write<'a, T> = Aligned<WriteGuard<'a, T>>;

impl<T: Element> Drop for TypedArray<T> {
    fn drop(&mut self) {
        unsafe {
            (T::destroy_fn(get_api()))(self.sys_mut());
        }
    }
}

impl<T: Element> Default for TypedArray<T> {
    fn default() -> Self {
        TypedArray::new()
    }
}

impl<T: Element + fmt::Debug> fmt::Debug for TypedArray<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.read().iter()).finish()
    }
}

impl<T: Element> TypedArray<T> {
    /// Creates an empty array.
    pub fn new() -> Self {
        unsafe {
            let mut inner = T::SysArray::default();
            (T::new_fn(get_api()))(&mut inner);
            TypedArray { inner }
        }
    }

    /// Creates from a `VariantArray` by making a best effort to convert each variant.
    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut inner = T::SysArray::default();
            (T::new_with_array_fn(get_api()))(&mut inner, &array.0);
            TypedArray { inner }
        }
    }

    /// Creates a new reference to this reference-counted instance.
    pub fn new_ref(&self) -> Self {
        unsafe {
            let mut inner = T::SysArray::default();
            (T::new_copy_fn(get_api()))(&mut inner, self.sys());
            TypedArray { inner }
        }
    }

    /// Appends an element to the end of the array.
    ///
    /// Calling `push` triggers copy-on-write behavior. To insert a large number of elements,
    /// consider using `resize` and `write`.
    #[inline]
    pub fn push(&mut self, val: T) {
        self.push_ref(&val)
    }

    /// Appends an element to the end of the array by reference.
    ///
    /// Calling `push` triggers copy-on-write behavior. To insert a large number of elements,
    /// consider using `resize` and `write`.
    #[inline]
    pub fn push_ref(&mut self, val: &T) {
        unsafe {
            (T::append_fn(get_api()))(self.sys_mut(), T::element_to_sys_ref(val));
        }
    }

    /// Copies and appends all values in `src` to the end of the array.
    #[inline]
    pub fn append(&mut self, src: &Self) {
        unsafe {
            (T::append_array_fn(get_api()))(self.sys_mut(), src.sys());
        }
    }

    /// Inserts an element at the given offset and returns `true` if successful.
    #[inline]
    pub fn insert(&mut self, offset: i32, val: T) -> bool {
        self.insert_ref(offset, &val)
    }

    /// Inserts an element by reference at the given offset and returns `true` if successful.
    #[inline]
    pub fn insert_ref(&mut self, offset: i32, val: &T) -> bool {
        unsafe {
            let status =
                (T::insert_fn(get_api()))(self.sys_mut(), offset, T::element_to_sys_ref(val));
            status != sys::godot_error_GODOT_OK
        }
    }

    /// Inverts the order of the elements in the array.
    #[inline]
    pub fn invert(&mut self) {
        unsafe { (T::invert_fn(get_api()))(self.sys_mut()) }
    }

    /// Removes an element at the given offset.
    #[inline]
    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (T::remove_fn(get_api()))(self.sys_mut(), idx);
        }
    }

    /// Changes the size of the array, possibly removing elements or pushing default values.
    #[inline]
    pub fn resize(&mut self, size: i32) {
        unsafe {
            (T::resize_fn(get_api()))(self.sys_mut(), size);
        }
    }

    /// Returns a copy of the element at the given offset.
    #[inline]
    pub fn get(&self, idx: i32) -> T {
        unsafe { T::element_from_sys(T::get_fn(get_api())(self.sys(), idx)) }
    }

    /// Sets the value of the element at the given offset.
    #[inline]
    pub fn set(&mut self, idx: i32, val: T) {
        self.set_ref(idx, &val)
    }

    /// Sets the value of the element at the given offset by reference.
    #[inline]
    pub fn set_ref(&mut self, idx: i32, val: &T) {
        unsafe {
            (T::set_fn(get_api()))(self.sys_mut(), idx, T::element_to_sys_ref(val));
        }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> i32 {
        unsafe { (T::size_fn(get_api()))(self.sys()) }
    }

    /// Returns `true` if the container is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn read(&self) -> Read<'_, T> {
        unsafe {
            MaybeUnaligned::new(ReadGuard::new(self.sys()))
                .try_into_aligned()
                .expect("Pool array access should be aligned. This indicates a bug in Godot")
        }
    }

    pub fn write(&mut self) -> Write<'_, T> {
        unsafe {
            MaybeUnaligned::new(WriteGuard::new(self.sys() as *mut _))
                .try_into_aligned()
                .expect("Pool array access should be aligned. This indicates a bug in Godot")
        }
    }

    #[doc(hidden)]
    pub fn sys(&self) -> *const T::SysArray {
        &self.inner
    }

    #[doc(hidden)]
    pub fn sys_mut(&mut self) -> *mut T::SysArray {
        &mut self.inner
    }

    #[doc(hidden)]
    pub fn from_sys(sys: T::SysArray) -> Self {
        TypedArray { inner: sys }
    }
}

/// RAII read guard.
pub struct ReadGuard<'a, T: Element> {
    access: *mut T::SysReadAccess,
    len: usize,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T: Element> ReadGuard<'a, T> {
    unsafe fn new(arr: *const T::SysArray) -> Self {
        let len = (T::size_fn(get_api()))(arr) as usize;
        let access = (T::read_fn(get_api()))(arr);

        Self {
            access,
            len,
            _marker: std::marker::PhantomData,
        }
    }
}

unsafe impl<'a, T: Element> crate::access::Guard for ReadGuard<'a, T> {
    type Target = T;
    fn len(&self) -> usize {
        self.len
    }
    fn read_ptr(&self) -> *const Self::Target {
        unsafe {
            let orig_ptr: *const T::SysTy = (T::read_access_ptr_fn(get_api()))(self.access);
            orig_ptr as *const Self::Target
        }
    }
}

impl<'a, T: Element> Drop for ReadGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            (T::read_access_destroy_fn(get_api()))(self.access);
        }
    }
}

impl<'a, T: Element> Clone for ReadGuard<'a, T> {
    fn clone(&self) -> Self {
        let access = unsafe { (T::read_access_copy_fn(get_api()))(self.access) };

        Self {
            access,
            len: self.len,
            _marker: std::marker::PhantomData,
        }
    }
}

/// RAII write guard.
pub struct WriteGuard<'a, T: Element> {
    access: *mut T::SysWriteAccess,
    len: usize,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T: Element> WriteGuard<'a, T> {
    unsafe fn new(arr: *mut T::SysArray) -> Self {
        let len = (T::size_fn(get_api()))(arr) as usize;
        let access = (T::write_fn(get_api()))(arr);

        Self {
            access,
            len,
            _marker: std::marker::PhantomData,
        }
    }
}

unsafe impl<'a, T: Element> crate::access::Guard for WriteGuard<'a, T> {
    type Target = T;
    fn len(&self) -> usize {
        self.len
    }
    fn read_ptr(&self) -> *const Self::Target {
        unsafe {
            let orig_ptr: *const T::SysTy = (T::write_access_ptr_fn(get_api()))(self.access);
            orig_ptr as *const Self::Target
        }
    }
}

unsafe impl<'a, T: Element> crate::access::WritePtr for WriteGuard<'a, T> {}

impl<'a, T: Element> Drop for WriteGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            (T::write_access_destroy_fn(get_api()))(self.access);
        }
    }
}

macros::decl_typed_array_element! {
    /// Trait for element types that can be contained in `TypedArray`. This trait is sealed
    /// and has no public interface.
    pub trait Element: private::Sealed { .. }
}

macros::impl_typed_array_element! {
    impl Element for u8 => byte { .. }
}
macros::impl_typed_array_element! {
    impl Element for i32 => int { .. }
}
macros::impl_typed_array_element! {
    impl Element for f32 => real { .. }
}
macros::impl_typed_array_element! {
    impl Element for GodotString
        as sys::godot_string
        ref *const sys::godot_string
        => string
    { .. }
}
macros::impl_typed_array_element! {
    impl Element for Vector2
        as sys::godot_vector2
        ref *const sys::godot_vector2
        => vector2
    { .. }
}
macros::impl_typed_array_element! {
    impl Element for Vector3
        as sys::godot_vector3
        ref *const sys::godot_vector3
        => vector3
    { .. }
}
macros::impl_typed_array_element! {
    impl Element for Color
        as sys::godot_color
        ref *const sys::godot_color
        => color
    { .. }
}

mod private {
    pub trait Sealed {}
}
