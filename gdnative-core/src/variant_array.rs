use std::iter::{Extend, FromIterator};
use std::marker::PhantomData;

use crate::private::get_api;
use crate::sys;

use crate::RefCounted;
use crate::ToVariant;
use crate::Variant;

use crate::thread_access::*;

use std::fmt;

/// A reference-counted `Variant` vector. Godot's generic array data type.
/// Negative indices can be used to count from the right.
///
/// # Safety
///
/// This is a reference-counted collection with "interior mutability" in Rust parlance.
/// To enforce that the official [thread-safety guidelines][thread-safety] are
/// followed this type uses the *typestate* pattern. The typestate `Access` tracks
/// whether there is "unique" access (where pretty much all operations are safe)
/// or whether the value might be "shared", in which case not all operations are
/// safe.
///
/// [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html
pub struct VariantArray<Access: ThreadAccess = Shared> {
    sys: sys::godot_array,

    /// Marker preventing the compiler from incorrectly deriving `Send` and `Sync`.
    _marker: PhantomData<Access>,
}

impl<Access: ThreadAccess> VariantArray<Access> {
    /// Sets the value of the element at the given offset.
    #[inline]
    pub fn set(&self, idx: i32, val: &Variant) {
        unsafe { (get_api().godot_array_set)(self.sys_mut(), idx, val.sys()) }
    }

    /// Returns a copy of the element at the given offset.
    #[inline]
    pub fn get(&self, idx: i32) -> Variant {
        unsafe { Variant((get_api().godot_array_get)(self.sys(), idx)) }
    }

    /// Returns a reference to the element at the given offset.
    ///
    /// # Safety
    ///
    /// The returned reference is invalidated if the same container is mutated through another
    /// reference.
    ///
    /// `Variant` is reference-counted and thus cheaply cloned. Consider using `get` instead.
    #[inline]
    pub unsafe fn get_ref(&self, idx: i32) -> &Variant {
        Variant::cast_ref((get_api().godot_array_operator_index_const)(
            self.sys(),
            idx,
        ))
    }

    /// Returns a mutable reference to the element at the given offset.
    ///
    /// # Safety
    ///
    /// The returned reference is invalidated if the same container is mutated through another
    /// reference. It is possible to create two mutable references to the same memory location
    /// if the same `idx` is provided, causing undefined behavior.
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut_ref(&self, idx: i32) -> &mut Variant {
        Variant::cast_mut_ref((get_api().godot_array_operator_index)(self.sys_mut(), idx))
    }

    #[inline]
    pub fn count(&self, val: &Variant) -> i32 {
        unsafe { (get_api().godot_array_count)(self.sys(), val.sys()) }
    }

    /// Returns `true` if the `VariantArray` contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        unsafe { (get_api().godot_array_empty)(self.sys()) }
    }

    /// Returns the number of elements in the array.
    #[inline]
    pub fn len(&self) -> i32 {
        unsafe { (get_api().godot_array_size)(self.sys()) }
    }

    /// Searches the array for a value and returns its index.
    /// Pass an initial search index as the second argument.
    /// Returns `-1` if value is not found.
    #[inline]
    pub fn find(&self, what: &Variant, from: i32) -> i32 {
        unsafe { (get_api().godot_array_find)(self.sys(), what.sys(), from) }
    }

    /// Returns true if the `VariantArray` contains the specified value.
    #[inline]
    pub fn contains(&self, what: &Variant) -> bool {
        unsafe { (get_api().godot_array_has)(self.sys(), what.sys()) }
    }

    /// Searches the array in reverse order.
    /// Pass an initial search index as the second argument.
    /// If negative, the start index is considered relative to the end of the array.
    #[inline]
    pub fn rfind(&self, what: &Variant, from: i32) -> i32 {
        unsafe { (get_api().godot_array_rfind)(self.sys(), what.sys(), from) }
    }

    /// Searches the array in reverse order for a value.
    /// Returns its index or `-1` if not found.
    #[inline]
    pub fn find_last(&self, what: &Variant) -> i32 {
        unsafe { (get_api().godot_array_find_last)(self.sys(), what.sys()) }
    }

    /// Inverts the order of the elements in the array.
    #[inline]
    pub fn invert(&self) {
        unsafe { (get_api().godot_array_invert)(self.sys_mut()) }
    }

    /// Return a hashed i32 value representing the array contents.
    #[inline]
    pub fn hash(&self) -> i32 {
        unsafe { (get_api().godot_array_hash)(self.sys()) }
    }

    #[inline]
    pub fn sort(&self) {
        unsafe { (get_api().godot_array_sort)(self.sys_mut()) }
    }

    /// Create a copy of the array.
    ///
    /// This creates a new array and is **not** a cheap reference count
    /// increment.
    #[inline]
    pub fn duplicate(&self) -> VariantArray<Unique> {
        unsafe {
            let sys = (get_api().godot_array_duplicate)(self.sys(), false);
            VariantArray::<Unique>::from_sys(sys)
        }
    }

    // TODO
    // pub fn sort_custom(&mut self, obj: ?, s: ?) {
    //     unimplemented!()
    // }

    // pub fn bsearch(&mut self, val: (), before: bool) -> i32 {
    //     unsafe {
    //         (get_api().godot_array_bsearch)(self.sys_mut(), val, before)
    //     }
    // }

    // pub fn bsearch_custom(&mut self, val: ?, obj: ?, s: ?, before: bool) -> i32 {
    //     unimplemented!();
    // }

    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_array {
        &self.sys
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys_mut(&self) -> *mut sys::godot_array {
        &self.sys as *const _ as *mut _
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(sys: sys::godot_array) -> Self {
        VariantArray {
            sys,
            _marker: PhantomData,
        }
    }
}

impl VariantArray<Unique> {
    /// Creates an empty `VariantArray`.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn into_shared(self) -> VariantArray<Shared> {
        VariantArray::<Shared> {
            sys: self.sys,
            _marker: PhantomData,
        }
    }

    /// Clears the array, resizing to 0.
    #[inline]
    pub fn clear(&self) {
        unsafe {
            (get_api().godot_array_clear)(self.sys_mut());
        }
    }

    /// Removes the element at `idx`.
    #[inline]
    pub fn remove(&self, idx: i32) {
        unsafe { (get_api().godot_array_remove)(self.sys_mut(), idx) }
    }

    /// Removed the first occurrence of `val`.
    #[inline]
    pub fn erase(&self, val: &Variant) {
        unsafe { (get_api().godot_array_erase)(self.sys_mut(), val.sys()) }
    }

    #[inline]
    pub fn resize(&self, size: i32) {
        unsafe { (get_api().godot_array_resize)(self.sys_mut(), size) }
    }

    /// Appends an element at the end of the array.
    #[inline]
    pub fn push(&self, val: &Variant) {
        unsafe {
            (get_api().godot_array_push_back)(self.sys_mut(), val.sys());
        }
    }

    /// Removes an element at the end of the array.
    #[inline]
    pub fn pop(&self) -> Variant {
        unsafe { Variant((get_api().godot_array_pop_back)(self.sys_mut())) }
    }

    /// Appends an element to the front of the array.
    #[inline]
    pub fn push_front(&self, val: &Variant) {
        unsafe {
            (get_api().godot_array_push_front)(self.sys_mut(), val.sys());
        }
    }

    /// Removes an element at the front of the array.
    #[inline]
    pub fn pop_front(&self) -> Variant {
        unsafe { Variant((get_api().godot_array_pop_front)(self.sys_mut())) }
    }

    /// Insert a new int at a given position in the array.
    #[inline]
    pub fn insert(&self, at: i32, val: &Variant) {
        unsafe { (get_api().godot_array_insert)(self.sys_mut(), at, val.sys()) }
    }

    /// Returns an iterator through all values in the `VariantArray`.
    ///
    /// `VariantArray` is reference-counted and have interior mutability in Rust parlance.
    /// Modifying the same underlying collection while observing the safety assumptions will
    /// not violate memory safely, but may lead to surprising behavior in the iterator.
    #[inline]
    pub fn iter(&self) -> IterUnique {
        self.into_iter()
    }
}

impl VariantArray<Shared> {
    /// Create a new shared array.
    #[inline]
    pub fn new_shared() -> Self {
        VariantArray::<Unique>::new().into_shared()
    }

    /// Assume the array is only referenced a single time to get a `Unique`
    /// version of the array.
    ///
    /// # Safety
    ///
    /// By calling this function it is assumed that only a single
    /// reference to the array exists, on the current or any other thread.
    #[inline]
    pub unsafe fn assume_unique(self) -> VariantArray<Unique> {
        VariantArray::<Unique> {
            sys: self.sys,
            _marker: PhantomData,
        }
    }

    /// Returns an iterator through all values in the `VariantArray`.
    ///
    /// `VariantArray` is reference-counted and have interior mutability in Rust parlance.
    /// Modifying the same underlying collection while observing the safety assumptions will
    /// not violate memory safely, but may lead to surprising behavior in the iterator.
    #[inline]
    pub fn iter(&self) -> IterShared {
        self.into_iter()
    }
}

impl Default for VariantArray<Unique> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Default for VariantArray<Shared> {
    #[inline]
    fn default() -> Self {
        VariantArray::new_shared()
    }
}

impl RefCounted for VariantArray<Shared> {
    #[inline]
    fn new_ref(&self) -> Self {
        unsafe {
            let mut result = Default::default();
            (get_api().godot_array_new_copy)(&mut result, self.sys());
            Self::from_sys(result)
        }
    }
}

impl<Access: ThreadAccess> Drop for VariantArray<Access> {
    #[inline]
    fn drop(&mut self) {
        unsafe { (get_api().godot_array_destroy)(self.sys_mut()) }
    }
}

impl fmt::Debug for VariantArray<Unique> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl fmt::Debug for VariantArray<Shared> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

// #[derive(Debug)]
pub struct IterShared {
    arr: VariantArray<Shared>,
    range: std::ops::Range<i32>,
}

impl Iterator for IterShared {
    type Item = Variant;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|idx| self.arr.get(idx))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<'a> IntoIterator for &'a VariantArray<Shared> {
    type Item = Variant;
    type IntoIter = IterShared;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterShared {
            arr: self.new_ref(),
            range: 0..self.len(),
        }
    }
}

// #[derive(Debug)]
pub struct IterUnique<'a> {
    arr: &'a VariantArray<Unique>,
    range: std::ops::Range<i32>,
}

impl<'a> Iterator for IterUnique<'a> {
    type Item = Variant;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|idx| self.arr.get(idx))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl<'a> IntoIterator for &'a VariantArray<Unique> {
    type Item = Variant;
    type IntoIter = IterUnique<'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterUnique {
            range: 0..self.len(),
            arr: self,
        }
    }
}

// #[derive(Debug)]
pub struct IntoIterUnique {
    arr: VariantArray<Unique>,
    range: std::ops::Range<i32>,
}

impl Iterator for IntoIterUnique {
    type Item = Variant;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|idx| self.arr.get(idx))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl IntoIterator for VariantArray<Unique> {
    type Item = Variant;
    type IntoIter = IntoIterUnique;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIterUnique {
            range: 0..self.len(),
            arr: self,
        }
    }
}

impl<T: ToVariant> FromIterator<T> for VariantArray<Unique> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut arr = Self::new();
        arr.extend(iter);
        arr
    }
}

impl<T: ToVariant> Extend<T> for VariantArray<Unique> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for elem in iter {
            self.push(&elem.to_variant());
        }
    }
}

godot_test!(test_array {
    let foo = Variant::from_str("foo");
    let bar = Variant::from_str("bar");
    let nope = Variant::from_str("nope");

    let array = VariantArray::new(); // []

    assert!(array.is_empty());
    assert_eq!(array.len(), 0);

    array.push(&foo); // [&foo]
    array.push(&bar); // [&foo, &bar]

    assert_eq!(array.len(), 2);

    assert!(array.contains(&foo));
    assert!(array.contains(&bar));
    assert!(!array.contains(&nope));

    array.set(0, &bar); // [&bar, &bar]
    array.set(1, &foo); // [&bar, &foo]

    assert_eq!(&array.get(0), &bar);
    assert_eq!(&array.get(1), &foo);

    array.pop(); // [&bar]
    array.pop(); // []

    let x = Variant::from_i64(42);
    let y = Variant::from_i64(1337);
    let z = Variant::from_i64(512);

    array.insert(0, &x); // [&x]
    array.insert(0, &y); // [&y, &x]
    array.push_front(&z); // [&y, &x]
    array.push_front(&z); // [&z, &z, &y, &x]

    assert_eq!(array.find(&y, 0), 2);
    assert_eq!(array.find_last(&z), 1);
    assert_eq!(array.find(&nope, 0), -1);

    array.invert(); // [&x, &y, &z, &z]

    assert_eq!(&array.get(0), &x);

    array.pop_front(); // [&y, &z, &z]
    array.pop_front(); // [&z, &z]

    assert_eq!(&array.get(0), &z);

    array.resize(0); // []
    assert!(array.is_empty());

    array.push(&foo); // [&foo]
    array.push(&bar); // [&foo, &bar]

    let array2 = array.duplicate();
    assert!(array2.contains(&foo));
    assert!(array2.contains(&bar));
    assert!(!array2.contains(&nope));

    let array3 = VariantArray::new(); // []

    array3.push(&Variant::from_i64(42));
    array3.push(&Variant::from_i64(1337));
    array3.push(&Variant::from_i64(512));

    assert_eq!(
        &[42, 1337, 512],
        array3.iter().map(|v| v.try_to_i64().unwrap()).collect::<Vec<_>>().as_slice(),
    );
});

godot_test!(
    test_array_debug {
        let arr = VariantArray::new(); // []
        arr.push(&Variant::from_str("hello world"));
        arr.push(&Variant::from_bool(true));
        arr.push(&Variant::from_i64(42));

        assert_eq!(format!("{:?}", arr), "[GodotString(hello world), Bool(True), I64(42)]");
    }
);

// TODO: clear arrays without affecting clones
//godot_test!(test_array_clone_clear {
//    let foo = Variant::from_str("foo");
//    let mut array = VariantArray::new();
//
//    array.push(&foo);
//    let array_clone = array.clone();
//    array.clear();
//
//    assert!(array.is_empty());
//    assert!(!array_clone.is_empty());
//});
