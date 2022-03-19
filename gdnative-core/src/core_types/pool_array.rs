use std::fmt;
use std::iter::{Extend, FromIterator};

use gdnative_impl_proc_macros as macros;

use crate::core_types::access::{Aligned, MaybeUnaligned};
use crate::core_types::{Color, GodotString, VariantArray, Vector2, Vector3};
use crate::object::NewRef;
use crate::private::get_api;

/// A RAII read access for Godot pool arrays.
pub type Read<'a, T> = Aligned<ReadGuard<'a, T>>;

/// A RAII write access for Godot pool arrays. This will only lock the CoW container once,
/// as opposed to every time with methods like `push()`.
pub type Write<'a, T> = Aligned<WriteGuard<'a, T>>;

/// A reference-counted CoW typed vector using Godot's pool allocator, generic over possible
/// element types.
///
/// `PoolArray` unifies all the different `Pool*Array` types exported by Godot. It can be used
/// in exported Rust methods as parameter and return types, as well as in exported properties.
/// However, it is limited to the element types, for which a `Pool*Array` exists in GDScript,
/// i.e. it cannot contain user-defined types.
/// If you need other types, look into [`VariantArray`](struct.VariantArray.html) or directly use
/// `Vec<T>` for type safety.
///
/// This type is CoW (copy-on-write). The `Clone` implementation of this type creates a new
/// reference without copying the contents.
///
/// If you need to read elements, e.g. for iteration or conversion to another collection,
/// the [`read()`][Self::read] method provides a view that dereferences to `&[T]`.
/// Analogously, [`write()`][Self::write] provides a writable view that dereferences to `&mut [T]`.
///
/// For element mutations, it's usually recommended to do process them in batch using
/// [`write()`][Self::write] or the [`append()`][Self::append] methods, as opposed to
/// [`push()`][Self::push] or [`set()`][Self::set], because the latter ones trigger
/// CoW behavior each time they are called.
pub struct PoolArray<T: PoolElement> {
    inner: T::SysArray,
}

impl<T: PoolElement> PoolArray<T> {
    /// Creates an empty array.
    #[inline]
    pub fn new() -> Self {
        unsafe {
            let mut inner = T::SysArray::default();
            (T::new_fn(get_api()))(&mut inner);
            PoolArray { inner }
        }
    }

    /// Creates from a `VariantArray` by making a best effort to convert each variant.
    #[inline]
    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut inner = T::SysArray::default();
            (T::new_with_array_fn(get_api()))(&mut inner, array.sys());
            PoolArray { inner }
        }
    }

    /// Creates a `PoolArray` moving elements from `src`.
    ///
    /// If your source type isn't precisely a `Vec<T>`, keep in mind that `PoolElement` implements the
    /// `FromIterator` trait, which allows it to be constructed from iterators, typically through `collect()`.
    ///
    /// For example:
    /// ```no_run
    /// // Int32Array is a type alias for PoolArray<i32>
    /// use gdnative::core_types::Int32Array;
    ///
    /// // Collect from range
    /// let arr = (0..4).collect::<Int32Array>();
    ///
    /// // Type conversion
    /// let vec: Vec<u32> = vec![1, 1, 2, 3, 5]; // note: unsigned
    /// let arr = vec.iter().map(|&e| e as i32).collect::<Int32Array>();
    /// ```
    #[inline]
    pub fn from_vec(mut src: Vec<T>) -> Self {
        let mut arr = Self::new();
        arr.append_vec(&mut src);
        arr
    }

    /// Copies all elements to a `Vec`, leaving this instance untouched.
    ///
    /// Equivalent to `self.read().to_vec()`. Only use this if your destination type is precisely
    /// a `Vec<T>`. Otherwise, call [`read()`][Self::read] which can be used as a slice.
    ///
    #[inline]
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let guard = self.read();
        guard.to_vec()
    }

    /// Appends an element to the end of the array.
    ///
    /// Calling `push()` triggers copy-on-write behavior. To insert a large number of elements,
    /// consider using [`append()`][Self::append], [`resize()`][Self::resize] or [`write()`][Self::write].
    #[inline]
    pub fn push(&mut self, val: T) {
        self.push_ref(&val)
    }

    /// Appends an element to the end of the array by reference.
    ///
    /// Calling `push_ref()` triggers copy-on-write behavior. To insert a large number of elements,
    /// consider using [`append()`][Self::append], [`resize()`][Self::resize] or [`write()`][Self::write].
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
    #[allow(clippy::iter_with_drain)] // "`drain(..)` used on a `Vec`"; suggests `into_iter()` but we don't have the vec by value
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

    /// Returns a scoped read-only view into this array.
    ///
    /// The returned read guard implements `Deref` with target type `[T]`, i.e. can be dereferenced to `&[T]`.
    /// This means all non-mutating (`&self`) slice methods can be used, see [here](struct.Aligned.html#deref-methods).
    #[inline]
    pub fn read(&self) -> Read<'_, T> {
        unsafe {
            MaybeUnaligned::new(ReadGuard::new(self.sys()))
                .try_into_aligned()
                .expect("Pool array access should be aligned. This indicates a bug in Godot")
        }
    }

    /// Returns a scoped read-write view into this array. This triggers CoW once per lock, instead
    /// of once each mutation.
    ///
    /// The returned write guard implements `DerefMut` with target type `[T]`, i.e. can be dereferenced to `&mut [T]`.
    /// This means all mutating and read-only slice methods can be used, see [here](struct.Aligned.html#deref-methods).
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
        PoolArray { inner: sys }
    }
}

impl<T: PoolElement + Copy> PoolArray<T> {
    /// Creates a new `PoolArray` by copying from `src`.
    ///
    /// Equivalent to a new object created with [`new()`][Self::new], followed by a subsequent [`append_slice()`][Self::append_slice].
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

impl<T: PoolElement> Drop for PoolArray<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            (T::destroy_fn(get_api()))(self.sys_mut());
        }
    }
}

impl<T: PoolElement> Default for PoolArray<T> {
    #[inline]
    fn default() -> Self {
        PoolArray::new()
    }
}

impl<T: PoolElement + fmt::Debug> fmt::Debug for PoolArray<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.read().iter()).finish()
    }
}

impl<T: PoolElement> Clone for PoolArray<T> {
    #[inline]
    fn clone(&self) -> Self {
        self.new_ref()
    }
}

impl<T: PoolElement> NewRef for PoolArray<T> {
    /// Creates a new reference to this reference-counted instance.
    #[inline]
    fn new_ref(&self) -> Self {
        unsafe {
            let mut inner = T::SysArray::default();
            (T::new_copy_fn(get_api()))(&mut inner, self.sys());
            PoolArray { inner }
        }
    }
}

// `FromIterator` and `Extend` implementations collect into `Vec` first, because Rust `Vec`s
// are better at handling unknown lengths than the Godot arrays (`push` CoWs every time!)

impl<T: PoolElement> FromIterator<T> for PoolArray<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec = iter.into_iter().collect::<Vec<_>>();
        Self::from_vec(vec)
    }
}

impl<T: PoolElement> Extend<T> for PoolArray<T> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let mut vec = iter.into_iter().collect::<Vec<_>>();
        self.append_vec(&mut vec);
    }
}

impl<T: PoolElement + PartialEq> PartialEq for PoolArray<T> {
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
impl<T: PoolElement + Eq> Eq for PoolArray<T> {}

/// RAII read guard.
pub struct ReadGuard<'a, T: PoolElement> {
    access: *mut T::SysReadAccess,
    len: usize,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T: PoolElement> ReadGuard<'a, T> {
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

unsafe impl<'a, T: PoolElement> crate::core_types::access::Guard for ReadGuard<'a, T> {
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

impl<'a, T: PoolElement> Drop for ReadGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            (T::read_access_destroy_fn(get_api()))(self.access);
        }
    }
}

impl<'a, T: PoolElement> Clone for ReadGuard<'a, T> {
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
pub struct WriteGuard<'a, T: PoolElement> {
    access: *mut T::SysWriteAccess,
    len: usize,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T: PoolElement> WriteGuard<'a, T> {
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

unsafe impl<'a, T: PoolElement> crate::core_types::access::Guard for WriteGuard<'a, T> {
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

unsafe impl<'a, T: PoolElement> crate::core_types::access::WritePtr for WriteGuard<'a, T> {}

impl<'a, T: PoolElement> Drop for WriteGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            (T::write_access_destroy_fn(get_api()))(self.access);
        }
    }
}

macros::decl_typed_array_element! {
    /// Trait for element types that can be contained in `PoolArray`. This trait is sealed
    /// and has no public interface.
    pub trait PoolElement: private::Sealed { .. }
}

macros::impl_typed_array_element! {
    impl PoolElement for u8 => byte { .. }
}
macros::impl_typed_array_element! {
    impl PoolElement for i32 => int { .. }
}
macros::impl_typed_array_element! {
    impl PoolElement for f32 => real { .. }
}
macros::impl_typed_array_element! {
    impl PoolElement for GodotString
        as sys::godot_string
        ref *const sys::godot_string
        => string
    { .. }
}
macros::impl_typed_array_element! {
    impl PoolElement for Vector2
        as sys::godot_vector2
        ref *const sys::godot_vector2
        => vector2
    { .. }
}
macros::impl_typed_array_element! {
    impl PoolElement for Vector3
        as sys::godot_vector3
        ref *const sys::godot_vector3
        => vector3
    { .. }
}
macros::impl_typed_array_element! {
    impl PoolElement for Color
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

    impl<T: Serialize + PoolElement> Serialize for PoolArray<T> {
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

    impl<'de, T: Deserialize<'de> + PoolElement> Deserialize<'de> for PoolArray<T> {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
        where
            D: Deserializer<'de>,
        {
            struct TypedArrayVisitor<T>(PhantomData<T>);
            impl<'de, T: Deserialize<'de> + PoolElement> Visitor<'de> for TypedArrayVisitor<T> {
                type Value = PoolArray<T>;

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
