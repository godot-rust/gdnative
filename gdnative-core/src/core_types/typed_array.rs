use std::convert::TryFrom;
use std::fmt;
use std::iter::{Extend, FromIterator};

use gdnative_impl_proc_macros as macros;

use crate::core_types::access::{Aligned, MaybeUnaligned};
use crate::core_types::{Color, GodotString, VariantArray, Vector2, Vector3};
use crate::object::NewRef;
use crate::private::get_api;

/// A reference-counted CoW typed vector using Godot's pool allocator, generic over possible
/// element types.
///
/// `TypedArray` unifies all the different `Pool*Array` types exported by Godot. It can be used
/// in exported Rust methods as parameter and return types, as well as in exported properties.
/// However, it is limited to the element types, for which a `Pool*Array` exists in GDScript,
/// i.e. it cannot contain user-defined types.
/// If you need other types, look into [`VariantArray`](struct.VariantArray.html) or directly use
/// `Vec<T>` for type safety.
///
/// This type is CoW. The `Clone` implementation of this type creates a new reference without
/// copying the contents.
///
/// When using this type, it's generally better to perform mutations in batch using `write`,
/// or the `append` methods, as opposed to `push` or `set`, because the latter ones trigger
/// CoW behavior each time they are called.
pub struct TypedArray<T: Element> {
    inner: T::SysArray,
}

/// A RAII read access for Godot typed arrays.
pub type Read<'a, T> = Aligned<ReadGuard<'a, T>>;

/// A RAII write access for Godot typed arrays. This will only lock the CoW container once,
/// as opposed to every time with methods like `push`.
pub type Write<'a, T> = Aligned<WriteGuard<'a, T>>;

impl<T: Element> Drop for TypedArray<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            (T::destroy_fn(get_api()))(self.sys_mut());
        }
    }
}

impl<T: Element> Default for TypedArray<T> {
    #[inline]
    fn default() -> Self {
        TypedArray::new()
    }
}

impl<T: Element + fmt::Debug> fmt::Debug for TypedArray<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.read().iter()).finish()
    }
}

impl<T: Element> Clone for TypedArray<T> {
    #[inline]
    fn clone(&self) -> Self {
        self.new_ref()
    }
}

impl<T: Element> NewRef for TypedArray<T> {
    /// Creates a new reference to this reference-counted instance.
    #[inline]
    fn new_ref(&self) -> Self {
        unsafe {
            let mut inner = T::SysArray::default();
            (T::new_copy_fn(get_api()))(&mut inner, self.sys());
            TypedArray { inner }
        }
    }
}

impl<T: Element> TypedArray<T> {
    /// Creates an empty array.
    #[inline]
    pub fn new() -> Self {
        unsafe {
            let mut inner = T::SysArray::default();
            (T::new_fn(get_api()))(&mut inner);
            TypedArray { inner }
        }
    }

    /// Creates from a `VariantArray` by making a best effort to convert each variant.
    #[inline]
    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut inner = T::SysArray::default();
            (T::new_with_array_fn(get_api()))(&mut inner, array.sys());
            TypedArray { inner }
        }
    }

    /// Creates a `TypedArray` moving elements from `src`.
    #[inline]
    pub fn from_vec(mut src: Vec<T>) -> Self {
        let mut arr = Self::new();
        arr.append_vec(&mut src);
        arr
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

    /// Moves all the elements from `src` into `self`, leaving `src` empty.
    ///
    /// # Panics
    ///
    /// If the resulting length would not fit in `i32`.
    #[inline]
    pub fn append_vec(&mut self, src: &mut Vec<T>) {
        let start = self.len() as usize;
        let new_len = start + src.len();
        self.resize(i32::try_from(new_len).expect("new length should fit in i32"));

        let mut write = self.write();
        let mut drain = src.drain(..);
        for dst in &mut write[start..] {
            *dst = drain.next().unwrap();
        }

        assert!(drain.next().is_none());
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
    #[inline]
    pub fn len(&self) -> i32 {
        unsafe { (T::size_fn(get_api()))(self.sys()) }
    }

    /// Returns `true` if the container is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a RAII read access into this array.
    #[inline]
    pub fn read(&self) -> Read<'_, T> {
        unsafe {
            MaybeUnaligned::new(ReadGuard::new(self.sys()))
                .try_into_aligned()
                .expect("Pool array access should be aligned. This indicates a bug in Godot")
        }
    }

    /// Returns a RAII write access into this array. This triggers CoW once per lock, instead
    /// of once each mutation.
    #[inline]
    pub fn write(&mut self) -> Write<'_, T> {
        unsafe {
            MaybeUnaligned::new(WriteGuard::new(self.sys() as *mut _))
                .try_into_aligned()
                .expect("Pool array access should be aligned. This indicates a bug in Godot")
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const T::SysArray {
        &self.inner
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys_mut(&mut self) -> *mut T::SysArray {
        &mut self.inner
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(sys: T::SysArray) -> Self {
        TypedArray { inner: sys }
    }
}

impl<T: Element + Copy> TypedArray<T> {
    /// Creates a new `TypedArray` by copying from `src`.
    ///
    /// # Panics
    ///
    /// If the length of `src` does not fit in `i32`.
    #[inline]
    pub fn from_slice(src: &[T]) -> Self {
        let mut arr = Self::new();
        arr.append_slice(src);
        arr
    }

    /// Copies and appends all values in `src` to the end of the array.
    ///
    /// # Panics
    ///
    /// If the resulting length would not fit in `i32`.
    #[inline]
    pub fn append_slice(&mut self, src: &[T]) {
        let start = self.len() as usize;
        let new_len = start + src.len();
        self.resize(i32::try_from(new_len).expect("new length should fit in i32"));

        let mut write = self.write();
        write[start..].copy_from_slice(src)
    }
}

// `FromIterator` and `Extend` implementations collect into `Vec` first, because Rust `Vec`s
// are better at handling unknown lengths than the Godot arrays (`push` CoWs every time!)

impl<T: Element> FromIterator<T> for TypedArray<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec = iter.into_iter().collect::<Vec<_>>();
        Self::from_vec(vec)
    }
}

impl<T: Element> Extend<T> for TypedArray<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut vec = iter.into_iter().collect::<Vec<_>>();
        self.append_vec(&mut vec);
    }
}

impl<T: Element + PartialEq> PartialEq for TypedArray<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        let left = self.read();
        let right = other.read();

        left.as_slice() == right.as_slice()
    }
}
impl<T: Element + Eq> Eq for TypedArray<T> {}

/// RAII read guard.
pub struct ReadGuard<'a, T: Element> {
    access: *mut T::SysReadAccess,
    len: usize,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T: Element> ReadGuard<'a, T> {
    #[inline]
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

unsafe impl<'a, T: Element> crate::core_types::access::Guard for ReadGuard<'a, T> {
    type Target = T;

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn read_ptr(&self) -> *const Self::Target {
        unsafe {
            let orig_ptr: *const T::SysTy = (T::read_access_ptr_fn(get_api()))(self.access);
            orig_ptr as *const Self::Target
        }
    }
}

impl<'a, T: Element> Drop for ReadGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            (T::read_access_destroy_fn(get_api()))(self.access);
        }
    }
}

impl<'a, T: Element> Clone for ReadGuard<'a, T> {
    #[inline]
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
    #[inline]
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

unsafe impl<'a, T: Element> crate::core_types::access::Guard for WriteGuard<'a, T> {
    type Target = T;

    #[inline]
    fn len(&self) -> usize {
        self.len
    }
    #[inline]
    fn read_ptr(&self) -> *const Self::Target {
        unsafe {
            let orig_ptr: *const T::SysTy = (T::write_access_ptr_fn(get_api()))(self.access);
            orig_ptr as *const Self::Target
        }
    }
}

unsafe impl<'a, T: Element> crate::core_types::access::WritePtr for WriteGuard<'a, T> {}

impl<'a, T: Element> Drop for WriteGuard<'a, T> {
    #[inline]
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

#[cfg(feature = "serde")]
mod serialize {
    use super::*;
    use serde::{
        de::{SeqAccess, Visitor},
        ser::SerializeSeq,
        Deserialize, Deserializer, Serialize, Serializer,
    };
    use std::fmt::Formatter;
    use std::marker::PhantomData;

    impl<T: Serialize + Element> Serialize for TypedArray<T> {
        #[inline]
        fn serialize<S>(&self, ser: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
        {
            let read = self.read();
            let mut ser = ser.serialize_seq(Some(read.len()))?;
            for e in read.iter() {
                ser.serialize_element(e)?
            }
            ser.end()
        }
    }

    impl<'de, T: Deserialize<'de> + Element> Deserialize<'de> for TypedArray<T> {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
        where
            D: Deserializer<'de>,
        {
            struct TypedArrayVisitor<T>(PhantomData<T>);
            impl<'de, T: Deserialize<'de> + Element> Visitor<'de> for TypedArrayVisitor<T> {
                type Value = TypedArray<T>;

                fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                    formatter.write_str(std::any::type_name::<Self::Value>())
                }

                fn visit_seq<A>(
                    self,
                    mut seq: A,
                ) -> Result<Self::Value, <A as SeqAccess<'de>>::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut vec = seq.size_hint().map_or_else(Vec::new, Vec::with_capacity);
                    while let Some(val) = seq.next_element::<T>()? {
                        vec.push(val);
                    }
                    Ok(Self::Value::from_vec(vec))
                }
            }

            deserializer.deserialize_seq(TypedArrayVisitor::<T>(PhantomData))
        }
    }
}
