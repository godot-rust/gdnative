use std::iter::{Extend, FromIterator};
use std::marker::PhantomData;

use gdnative_impl_proc_macros::doc_variant_collection_safety;

use crate::private::get_api;
use crate::sys;

use crate::core_types::OwnedToVariant;
use crate::core_types::ToVariant;
use crate::core_types::Variant;
use crate::NewRef;

use crate::thread_access::*;

use std::fmt;

/// A reference-counted `Variant` vector. Godot's generic array data type.
/// Negative indices can be used to count from the right.
///
/// Generic methods on this type performs `Variant` conversion every time. This could
/// be significant for complex structures. Users may convert arguments to `Variant`s before
/// calling to avoid this behavior if necessary.
///
/// # Safety
///
/// This is a reference-counted collection with "interior mutability" in Rust parlance.
/// To enforce that the official [thread-safety guidelines][thread-safety] are
/// followed this type uses the *typestate* pattern. The typestate `Access` tracks
/// whether there is thread-local or unique access (where pretty much all operations are safe)
/// or whether the value might be "shared", in which case not all operations are
/// safe.
///
/// [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html
pub struct VariantArray<Access: ThreadAccess = Shared> {
    sys: sys::godot_array,

    /// Marker preventing the compiler from incorrectly deriving `Send` and `Sync`.
    _marker: PhantomData<Access>,
}

/// Operations allowed on all arrays at any point in time.
impl<Access: ThreadAccess> VariantArray<Access> {
    /// Sets the value of the element at the given offset.
    #[inline]
    pub fn set<T: OwnedToVariant>(&self, idx: i32, val: T) {
        unsafe { (get_api().godot_array_set)(self.sys_mut(), idx, val.owned_to_variant().sys()) }
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
    pub fn count<T: ToVariant>(&self, val: T) -> i32 {
        unsafe { (get_api().godot_array_count)(self.sys(), val.to_variant().sys()) }
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
    pub fn find<T: ToVariant>(&self, what: T, from: i32) -> i32 {
        unsafe { (get_api().godot_array_find)(self.sys(), what.to_variant().sys(), from) }
    }

    /// Returns true if the `VariantArray` contains the specified value.
    #[inline]
    pub fn contains<T: ToVariant>(&self, what: T) -> bool {
        unsafe { (get_api().godot_array_has)(self.sys(), what.to_variant().sys()) }
    }

    /// Searches the array in reverse order.
    /// Pass an initial search index as the second argument.
    /// If negative, the start index is considered relative to the end of the array.
    #[inline]
    pub fn rfind<T: ToVariant>(&self, what: T, from: i32) -> i32 {
        unsafe { (get_api().godot_array_rfind)(self.sys(), what.to_variant().sys(), from) }
    }

    /// Searches the array in reverse order for a value.
    /// Returns its index or `-1` if not found.
    #[inline]
    pub fn find_last<T: ToVariant>(&self, what: T) -> i32 {
        unsafe { (get_api().godot_array_find_last)(self.sys(), what.to_variant().sys()) }
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

    /// Create a deep copy of the array.
    ///
    /// This creates a new array and is **not** a cheap reference count
    /// increment.
    #[inline]
    pub fn duplicate_deep(&self) -> VariantArray<Unique> {
        unsafe {
            let sys = (get_api().godot_array_duplicate)(self.sys(), true);
            VariantArray::<Unique>::from_sys(sys)
        }
    }

    /// Returns an iterator through all values in the `VariantArray`.
    ///
    /// `VariantArray` is reference-counted and have interior mutability in Rust parlance.
    /// Modifying the same underlying collection while observing the safety assumptions will
    /// not violate memory safely, but may lead to surprising behavior in the iterator.
    #[inline]
    pub fn iter(&self) -> Iter<'_, Access> {
        self.into_iter()
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

    unsafe fn cast_access<A: ThreadAccess>(self) -> VariantArray<A> {
        let sys = self.sys;
        std::mem::forget(self);
        VariantArray::from_sys(sys)
    }
}

/// Operations allowed on Dictionaries that can only be referenced to from the current thread.
impl<Access: LocalThreadAccess> VariantArray<Access> {
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
    pub fn erase<T: ToVariant>(&self, val: T) {
        unsafe { (get_api().godot_array_erase)(self.sys_mut(), val.to_variant().sys()) }
    }

    /// Resizes the array, filling with `Nil` if necessary.
    #[inline]
    pub fn resize(&self, size: i32) {
        unsafe { (get_api().godot_array_resize)(self.sys_mut(), size) }
    }

    /// Appends an element at the end of the array.
    #[inline]
    pub fn push<T: OwnedToVariant>(&self, val: T) {
        unsafe {
            (get_api().godot_array_push_back)(self.sys_mut(), val.owned_to_variant().sys());
        }
    }

    /// Removes an element at the end of the array.
    #[inline]
    pub fn pop(&self) -> Variant {
        unsafe { Variant((get_api().godot_array_pop_back)(self.sys_mut())) }
    }

    /// Appends an element to the front of the array.
    #[inline]
    pub fn push_front<T: OwnedToVariant>(&self, val: T) {
        unsafe {
            (get_api().godot_array_push_front)(self.sys_mut(), val.owned_to_variant().sys());
        }
    }

    /// Removes an element at the front of the array.
    #[inline]
    pub fn pop_front(&self) -> Variant {
        unsafe { Variant((get_api().godot_array_pop_front)(self.sys_mut())) }
    }

    /// Insert a new int at a given position in the array.
    #[inline]
    pub fn insert<T: OwnedToVariant>(&self, at: i32, val: T) {
        unsafe { (get_api().godot_array_insert)(self.sys_mut(), at, val.owned_to_variant().sys()) }
    }
}

/// Operations allowed on non-unique arrays.
impl<Access: NonUniqueThreadAccess> VariantArray<Access> {
    /// Assume that this is the only reference to this array, on which
    /// operations that change the container size can be safely performed.
    ///
    /// # Safety
    ///
    /// It isn't thread-safe to perform operations that change the container
    /// size from multiple threads at the same time.
    /// Creating multiple `Unique` references to the same collections, or
    /// violating the thread-safety guidelines in non-Rust code will cause
    /// undefined behavior.
    #[inline]
    pub unsafe fn assume_unique(self) -> VariantArray<Unique> {
        self.cast_access()
    }
}

/// Operations allowed on unique arrays.
impl VariantArray<Unique> {
    /// Creates an empty `VariantArray`.
    #[inline]
    pub fn new() -> Self {
        unsafe {
            let mut sys = sys::godot_array::default();
            (get_api().godot_array_new)(&mut sys);
            Self::from_sys(sys)
        }
    }

    /// Put this array under the "shared" access type.
    #[inline]
    pub fn into_shared(self) -> VariantArray<Shared> {
        unsafe { self.cast_access() }
    }

    /// Put this array under the "thread-local" access type.
    #[inline]
    pub fn into_thread_local(self) -> VariantArray<ThreadLocal> {
        unsafe { self.cast_access() }
    }
}

/// Operations allowed on arrays that might be shared between different threads.
impl VariantArray<Shared> {
    /// Create a new shared array.
    #[inline]
    pub fn new_shared() -> Self {
        VariantArray::<Unique>::new().into_shared()
    }

    /// Clears the array, resizing to 0.
    ///
    #[doc_variant_collection_safety]
    #[inline]
    pub unsafe fn clear(&self) {
        (get_api().godot_array_clear)(self.sys_mut());
    }

    /// Removes the element at `idx`.
    ///
    #[doc_variant_collection_safety]
    #[inline]
    pub unsafe fn remove(&self, idx: i32) {
        (get_api().godot_array_remove)(self.sys_mut(), idx)
    }

    /// Removed the first occurrence of `val`.
    ///
    #[doc_variant_collection_safety]
    #[inline]
    pub unsafe fn erase<T: ToVariant>(&self, val: T) {
        (get_api().godot_array_erase)(self.sys_mut(), val.to_variant().sys())
    }

    /// Resizes the array, filling with `Nil` if necessary.
    ///
    #[doc_variant_collection_safety]
    #[inline]
    pub unsafe fn resize(&self, size: i32) {
        (get_api().godot_array_resize)(self.sys_mut(), size)
    }

    /// Appends an element at the end of the array.
    ///
    #[doc_variant_collection_safety]
    #[inline]
    pub unsafe fn push<T: OwnedToVariant>(&self, val: T) {
        (get_api().godot_array_push_back)(self.sys_mut(), val.owned_to_variant().sys());
    }

    /// Removes an element at the end of the array.
    ///
    #[doc_variant_collection_safety]
    #[inline]
    pub unsafe fn pop(&self) -> Variant {
        Variant((get_api().godot_array_pop_back)(self.sys_mut()))
    }

    /// Appends an element to the front of the array.
    ///
    #[doc_variant_collection_safety]
    #[inline]
    pub unsafe fn push_front<T: OwnedToVariant>(&self, val: T) {
        (get_api().godot_array_push_front)(self.sys_mut(), val.owned_to_variant().sys());
    }

    /// Removes an element at the front of the array.
    ///
    #[doc_variant_collection_safety]
    #[inline]
    pub unsafe fn pop_front(&self) -> Variant {
        Variant((get_api().godot_array_pop_front)(self.sys_mut()))
    }

    /// Insert a new int at a given position in the array.
    ///
    #[doc_variant_collection_safety]
    #[inline]
    pub unsafe fn insert<T: OwnedToVariant>(&self, at: i32, val: T) {
        (get_api().godot_array_insert)(self.sys_mut(), at, val.owned_to_variant().sys())
    }
}

/// Operations allowed on Dictionaries that may only be shared on the current thread.
impl VariantArray<ThreadLocal> {
    /// Create a new thread-local array.
    #[inline]
    pub fn new_thread_local() -> Self {
        VariantArray::<Unique>::new().into_thread_local()
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

impl Default for VariantArray<ThreadLocal> {
    #[inline]
    fn default() -> Self {
        VariantArray::new_thread_local()
    }
}

impl<Access: NonUniqueThreadAccess> NewRef for VariantArray<Access> {
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

impl<Access: ThreadAccess> fmt::Debug for VariantArray<Access> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

// #[derive(Debug)]
pub struct Iter<'a, Access: ThreadAccess> {
    arr: &'a VariantArray<Access>,
    range: std::ops::Range<i32>,
}

impl<'a, Access: ThreadAccess> Iterator for Iter<'a, Access> {
    type Item = Variant;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|idx| self.arr.get(idx))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        if !self.arr.is_empty() {
            Some(self.arr.get(self.arr.len() - 1))
        } else {
            None
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        use std::convert::TryFrom;
        let n = i32::try_from(n).ok()?;

        if self.arr.len() > n {
            Some(self.arr.get(n))
        } else {
            None
        }
    }
}

impl<'a, Access: ThreadAccess> IntoIterator for &'a VariantArray<Access> {
    type Item = Variant;
    type IntoIter = Iter<'a, Access>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            range: 0..self.len(),
            arr: self,
        }
    }
}

// #[derive(Debug)]
pub struct IntoIter {
    arr: VariantArray<Unique>,
    range: std::ops::Range<i32>,
}

impl Iterator for IntoIter {
    type Item = Variant;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|idx| self.arr.get(idx))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        if !self.arr.is_empty() {
            Some(self.arr.get(self.arr.len() - 1))
        } else {
            None
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        use std::convert::TryFrom;
        let n = i32::try_from(n).ok()?;

        if self.arr.len() > n {
            Some(self.arr.get(n))
        } else {
            None
        }
    }
}

impl IntoIterator for VariantArray<Unique> {
    type Item = Variant;
    type IntoIter = IntoIter;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
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

impl<T: ToVariant, Access: LocalThreadAccess> Extend<T> for VariantArray<Access> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for elem in iter {
            self.push(&elem.to_variant());
        }
    }
}

impl Clone for VariantArray {
    fn clone(&self) -> Self {
        self.new_ref()
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

    let array4 = VariantArray::new(); // []
    let array5 = VariantArray::new(); // []
    array4.push(&foo); // [&foo]
    array4.push(&bar); // [&foo, &bar]
    array5.push(array4); // [[&foo, &bar]]

    let array6 = array5.duplicate_deep(); // [[&foo, &bar]]
    unsafe { array5.get(0).to_array().assume_unique().pop(); } // [[&foo]]

    assert!(!array5.get(0).to_array().contains(&bar));
    assert!(array6.get(0).to_array().contains(&bar));
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
