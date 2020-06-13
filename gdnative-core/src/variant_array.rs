use std::iter::{Extend, FromIterator};
use std::marker::PhantomData;

use crate::private::get_api;
use crate::sys;

use crate::RefCounted;
use crate::ToVariant;
use crate::Variant;

use std::fmt;

/// A reference-counted `Variant` vector. Godot's generic array data type.
/// Negative indices can be used to count from the right.
///
/// # Safety
///
/// This is a reference-counted collection with "interior mutability" in Rust parlance. Its use
/// must follow the official [thread-safety guidelines][thread-safety]. Specifically, it is
/// undefined behavior to pass an instance to Rust code without locking a mutex if there are
/// references to it on other threads.
///
/// [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html
pub struct VariantArray {
    sys: sys::godot_array,

    /// Marker preventing the compiler from incorrectly deriving `Send` and `Sync`.
    _marker: PhantomData<*const ()>,
}

impl VariantArray {
    /// Creates an empty `VariantArray`.
    #[inline]
    pub fn new() -> Self {
        VariantArray::default()
    }

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

    #[inline]
    pub fn resize(&self, size: i32) {
        unsafe { (get_api().godot_array_resize)(self.sys_mut(), size) }
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

    /// Returns an iterator through all values in the `VariantArray`.
    ///
    /// `VariantArray` is reference-counted and have interior mutability in Rust parlance.
    /// Modifying the same underlying collection while observing the safety assumptions will
    /// not violate memory safely, but may lead to surprising behavior in the iterator.
    #[inline]
    pub fn iter(&self) -> Iter {
        Iter {
            arr: self.new_ref(),
            range: 0..self.len(),
        }
    }

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

impl_basic_traits_as_sys!(
    for VariantArray as godot_array {
        Drop => godot_array_destroy;
        Default => godot_array_new;
        RefCounted => godot_array_new_copy;
    }
);

impl fmt::Debug for VariantArray {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[derive(Debug)]
pub struct Iter {
    arr: VariantArray,
    range: std::ops::Range<i32>,
}

impl Iterator for Iter {
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

impl<T: ToVariant> FromIterator<T> for VariantArray {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut arr = Self::new();
        arr.extend(iter);
        arr
    }
}

impl<T: ToVariant> Extend<T> for VariantArray {
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

    let array2 = array.new_ref();
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
