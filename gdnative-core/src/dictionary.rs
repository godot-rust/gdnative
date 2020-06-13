use std::iter::{Extend, FromIterator};
use std::marker::PhantomData;

use crate::private::get_api;
use crate::sys;
use crate::GodotString;

use crate::RefCounted;
use crate::ToVariant;
use crate::ToVariantEq;
use crate::Variant;
use crate::VariantArray;
use std::fmt;

use crate::thread_access::*;

/// A reference-counted `Dictionary` of `Variant` key-value pairs.
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
pub struct Dictionary<Access: ThreadAccess = Shared> {
    sys: sys::godot_dictionary,

    /// Marker preventing the compiler from incorrectly deriving `Send` and `Sync`.
    _marker: PhantomData<Access>,
}

/// Operations allowed on all Dictionaries at any point in time.
impl<Access: ThreadAccess> Dictionary<Access> {
    /// Returns `true` if the `Dictionary` contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        unsafe { (get_api().godot_dictionary_empty)(self.sys()) }
    }

    /// Returns the number of elements in the `Dictionary`.
    #[inline]
    pub fn len(&self) -> i32 {
        unsafe { (get_api().godot_dictionary_size)(self.sys()) }
    }

    /// Returns true if the `Dictionary` contains the specified key.
    #[inline]
    pub fn contains(&self, key: &Variant) -> bool {
        unsafe { (get_api().godot_dictionary_has)(self.sys(), key.sys()) }
    }

    /// Returns true if the `Dictionary` has all of the keys in the given array.
    #[inline]
    pub fn contains_all<ArrAccess: ThreadAccess>(&self, keys: &VariantArray<ArrAccess>) -> bool {
        unsafe { (get_api().godot_dictionary_has_all)(self.sys(), keys.sys()) }
    }

    /// Returns a copy of the value corresponding to the key.
    #[inline]
    pub fn get(&self, key: &Variant) -> Variant {
        unsafe { Variant((get_api().godot_dictionary_get)(self.sys(), key.sys())) }
    }

    /// Update an existing element corresponding ot the key.
    ///
    /// # Panics
    ///
    /// Panics if the entry for `key` does not exist.
    #[inline]
    pub fn update(&self, key: &Variant, val: &Variant) {
        assert!(self.contains(key), "Can only update entries that exist");
        unsafe { (get_api().godot_dictionary_set)(self.sys_mut(), key.sys(), val.sys()) }
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Safety
    ///
    /// The returned reference is invalidated if the same container is mutated through another
    /// reference.
    ///
    /// `Variant` is reference-counted and thus cheaply cloned. Consider using `get` instead.
    #[inline]
    pub unsafe fn get_ref(&self, key: &Variant) -> &Variant {
        Variant::cast_ref((get_api().godot_dictionary_operator_index_const)(
            self.sys(),
            key.sys(),
        ))
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// # Safety
    ///
    /// The returned reference is invalidated if the same container is mutated through another
    /// reference. It is possible to create two mutable references to the same memory location
    /// if the same `key` is provided, causing undefined behavior.
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut_ref(&self, key: &Variant) -> &mut Variant {
        Variant::cast_mut_ref((get_api().godot_dictionary_operator_index)(
            self.sys_mut(),
            key.sys(),
        ))
    }

    /// Returns a GodotString of the `Dictionary`.
    #[inline]
    pub fn to_json(&self) -> GodotString {
        unsafe { GodotString((get_api().godot_dictionary_to_json)(self.sys())) }
    }

    /// Returns an array of the keys in the `Dictionary`.
    #[inline]
    pub fn keys(&self) -> VariantArray<Shared> {
        unsafe { VariantArray::from_sys((get_api().godot_dictionary_keys)(self.sys())) }
    }

    /// Returns an array of the values in the `Dictionary`.
    #[inline]
    pub fn values(&self) -> VariantArray {
        unsafe { VariantArray::from_sys((get_api().godot_dictionary_values)(self.sys())) }
    }

    #[inline]
    pub fn get_next(&self, key: &Variant) -> &Variant {
        unsafe { Variant::cast_ref((get_api().godot_dictionary_next)(self.sys(), key.sys())) }
    }

    /// Return a hashed i32 value representing the dictionary's contents.
    #[inline]
    pub fn hash(&self) -> i32 {
        unsafe { (get_api().godot_dictionary_hash)(self.sys()) }
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_dictionary {
        &self.sys
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys_mut(&self) -> *mut sys::godot_dictionary {
        &self.sys as *const _ as *mut _
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(sys: sys::godot_dictionary) -> Self {
        Dictionary {
            sys,
            _marker: PhantomData,
        }
    }
}

/// Operations allowed on Dictionaries that might be shared.
impl Dictionary<Shared> {
    /// Assume the dictionary is only used/referenced by a single thread.
    ///
    ///
    #[inline]
    pub unsafe fn assume_unique(self) -> Dictionary<Unique> {
        Dictionary::<Unique> {
            sys: self.sys,
            _marker: PhantomData,
        }
    }

    /// Returns an iterator through all key-value pairs in the `Dictionary`.
    ///
    /// `Dictionary` is reference-counted and have interior mutability in Rust parlance.
    /// Modifying the same underlying collection while observing the safety assumptions will
    /// not violate memory safely, but may lead to surprising behavior in the iterator.
    #[inline]
    pub fn iter(&self) -> IterShared {
        IterShared::new(self)
    }

    /// Create a copy of the dictionary.
    ///
    /// This creates a new dictionary and is **not** a cheap reference count
    /// increment.
    #[inline]
    pub fn duplicate(&self) -> Dictionary<Unique> {
        let d = Dictionary::new();
        for (k, v) in self {
            d.insert(&k, &v);
        }
        d
    }
}

/// Operations allowed on non-shared Dictionaries.
impl Dictionary<Unique> {
    /// Creates an empty `Dictionary`.
    #[inline]
    pub fn new() -> Self {
        unsafe {
            let mut sys = sys::godot_dictionary::default();
            (get_api().godot_dictionary_new)(&mut sys);
            Self::from_sys(sys)
        }
    }

    /// Put this dictionary under the "shared" access type.
    #[inline]
    pub fn into_shared(self) -> Dictionary<Shared> {
        Dictionary::<Shared> {
            sys: self.sys,
            _marker: PhantomData,
        }
    }

    /// Returns an iterator through all key-value pairs in the `Dictionary`.
    ///
    /// `Dictionary` is reference-counted and have interior mutability in Rust parlance.
    /// Modifying the same underlying collection while observing the safety assumptions will
    /// not violate memory safely, but may lead to surprising behavior in the iterator.
    #[inline]
    pub fn into_iter(self) -> IntoIterUnique {
        IntoIterator::into_iter(self)
    }

    /// Returns an iterator through all key-value pairs in the `Dictionary`.
    ///
    /// `Dictionary` is reference-counted and have interior mutability in Rust parlance.
    /// Modifying the same underlying collection while observing the safety assumptions will
    /// not violate memory safely, but may lead to surprising behavior in the iterator.
    #[inline]
    pub fn iter(&self) -> IterUnique {
        IterUnique::new(self)
    }

    /// Create a copy of the dictionary.
    ///
    /// This creates a new dictionary and is **not** a cheap reference count
    /// increment.
    #[inline]
    pub fn duplicate(&self) -> Dictionary<Unique> {
        let d = Dictionary::new();
        for (k, v) in self {
            d.insert(&k, &v);
        }
        d
    }

    #[inline]
    /// Inserts or updates the value of the element corresponding to the key.
    pub fn insert(&self, key: &Variant, val: &Variant) {
        unsafe { (get_api().godot_dictionary_set)(self.sys_mut(), key.sys(), val.sys()) }
    }

    /// Erase a key-value pair in the `Dictionary` by the specified key.
    #[inline]
    pub fn erase(&self, key: &Variant) {
        unsafe { (get_api().godot_dictionary_erase)(self.sys_mut(), key.sys()) }
    }

    /// Clears the `Dictionary`, removing all key-value pairs.
    #[inline]
    pub fn clear(&self) {
        unsafe { (get_api().godot_dictionary_clear)(self.sys_mut()) }
    }
}

impl<Access: ThreadAccess> Drop for Dictionary<Access> {
    #[inline]
    fn drop(&mut self) {
        unsafe { (get_api().godot_dictionary_destroy)(self.sys_mut()) }
    }
}

impl Default for Dictionary<Unique> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Dictionary<Shared> {
    #[inline]
    fn default() -> Self {
        Dictionary::<Unique>::default().into_shared()
    }
}

impl RefCounted for Dictionary<Shared> {
    #[inline]
    fn new_ref(&self) -> Self {
        unsafe {
            let mut result = Default::default();
            (get_api().godot_dictionary_new_copy)(&mut result, self.sys());
            Self::from_sys(result)
        }
    }
}

impl<A: ThreadAccess, B: ThreadAccess> PartialEq<Dictionary<A>> for Dictionary<B> {
    fn eq(&self, other: &Dictionary<A>) -> bool {
        unsafe { (get_api().godot_dictionary_operator_equal)(self.sys(), other.sys()) }
    }
}

impl From<Dictionary<Unique>> for Dictionary<Shared> {
    #[inline]
    fn from(dict: Dictionary<Unique>) -> Self {
        dict.into_shared()
    }
}

impl<Access: ThreadAccess> fmt::Debug for Dictionary<Access> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.to_json().to_string().fmt(f)
    }
}

/// Iterator through all key-value pairs in a shared `Dictionary`.
///
/// This struct is created by the `iter` method on `Dictionary<Shared>`.
#[derive(Debug)]
pub struct IterShared {
    dic: Dictionary<Shared>,
    last_key: Option<Variant>,
}

impl IterShared {
    /// Create an Iterator from a shared Dictionary.
    fn new(dic: &Dictionary<Shared>) -> Self {
        IterShared {
            dic: dic.new_ref(),
            last_key: None,
        }
    }
}

impl Iterator for IterShared {
    type Item = (Variant, Variant);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let last_ptr = self
            .last_key
            .as_ref()
            .map_or(std::ptr::null(), Variant::sys);
        let next_ptr = unsafe { (get_api().godot_dictionary_next)(self.dic.sys(), last_ptr) };

        if next_ptr.is_null() {
            None
        } else {
            let key = Variant::cast_ref(next_ptr).clone();
            let value = self.dic.get(&key);
            self.last_key = Some(key.clone());
            Some((key, value))
        }
    }
}

impl<'a> IntoIterator for &'a Dictionary<Shared> {
    type Item = (Variant, Variant);
    type IntoIter = IterShared;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator through all key-value pairs in a unique `Dictionary`.
///
/// This struct is created by the `into_iter` method on `Dictionary<Unique>`.
/// This iterator consumes the unique dictionary.
#[derive(Debug)]
pub struct IntoIterUnique {
    dic: Dictionary<Unique>,
    last_key: Option<Variant>,
}

impl IntoIterUnique {
    /// Create an Iterator by consuming a unique Dictionary.
    fn new(dic: Dictionary<Unique>) -> Self {
        IntoIterUnique {
            dic,
            last_key: None,
        }
    }
}

impl Iterator for IntoIterUnique {
    type Item = (Variant, Variant);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let last_ptr = self
            .last_key
            .as_ref()
            .map_or(std::ptr::null(), Variant::sys);
        let next_ptr = unsafe { (get_api().godot_dictionary_next)(self.dic.sys(), last_ptr) };

        if next_ptr.is_null() {
            None
        } else {
            let key = Variant::cast_ref(next_ptr).clone();
            let value = self.dic.get(&key);
            self.last_key = Some(key.clone());
            Some((key, value))
        }
    }
}

impl IntoIterator for Dictionary<Unique> {
    type Item = (Variant, Variant);
    type IntoIter = IntoIterUnique;
    fn into_iter(self) -> Self::IntoIter {
        IntoIterUnique::new(self)
    }
}

/// Iterator through all key-value pairs in a unique `Dictionary`.
///
/// This struct is created by the `iter` method on `Dictionary<Unique>`.
#[derive(Debug)]
pub struct IterUnique<'a> {
    dic: &'a Dictionary<Unique>,
    last_key: Option<Variant>,
}

impl<'a> IterUnique<'a> {
    /// Create an Iterator from a unique Dictionary.
    fn new(dic: &'a Dictionary<Unique>) -> Self {
        IterUnique {
            dic,
            last_key: None,
        }
    }
}

impl<'a> Iterator for IterUnique<'a> {
    type Item = (Variant, Variant);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let last_ptr = self
            .last_key
            .as_ref()
            .map_or(std::ptr::null(), Variant::sys);
        let next_ptr = unsafe { (get_api().godot_dictionary_next)(self.dic.sys(), last_ptr) };

        if next_ptr.is_null() {
            None
        } else {
            let key = Variant::cast_ref(next_ptr).clone();
            let value = self.dic.get(&key);
            self.last_key = Some(key.clone());
            Some((key, value))
        }
    }
}

impl<'a> IntoIterator for &'a Dictionary<Unique> {
    type Item = (Variant, Variant);
    type IntoIter = IterUnique<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K, V> FromIterator<(K, V)> for Dictionary<Unique>
where
    K: ToVariantEq,
    V: ToVariant,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut dic = Dictionary::new();
        dic.extend(iter);
        dic
    }
}

impl FromIterator<(Variant, Variant)> for Dictionary<Unique> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (Variant, Variant)>>(iter: I) -> Self {
        let mut dic = Dictionary::new();
        dic.extend(iter);
        dic
    }
}

impl<K, V> Extend<(K, V)> for Dictionary<Unique>
where
    K: ToVariantEq,
    V: ToVariant,
{
    #[inline]
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(&key.to_variant(), &value.to_variant());
        }
    }
}

impl Extend<(Variant, Variant)> for Dictionary<Unique> {
    #[inline]
    fn extend<I: IntoIterator<Item = (Variant, Variant)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(&key.to_variant(), &value.to_variant());
        }
    }
}

godot_test!(test_dictionary {
    use std::collections::HashSet;

    use crate::VariantType;
    let foo = Variant::from_str("foo");
    let bar = Variant::from_str("bar");
    let nope = Variant::from_str("nope");

    let x = Variant::from_i64(42);
    let y = Variant::from_i64(1337);

    let dict = Dictionary::<Unique>::new();

    dict.insert(&foo, &x);
    dict.insert(&bar, &y);

    assert!(dict.contains(&foo));
    assert!(dict.contains(&bar));
    assert!(!dict.contains(&nope));

    let keys_array = unsafe { dict.keys().assume_unique() };
    let baz = Variant::from_str("baz");
    keys_array.push(&baz);
    dict.insert(&baz, &x);

    assert!(dict.contains_all(&keys_array));

    dict.erase(&baz);

    assert!(!dict.contains_all(&keys_array));

    let variant = Variant::from_dictionary(&dict.duplicate().into_shared());
    assert!(variant.get_type() == VariantType::Dictionary);

    let dict2 = dict.duplicate();
    assert!(dict == dict2);
    assert!(dict2.contains(&foo));
    assert!(dict2.contains(&bar));

    if let Some(dic_variant) = variant.try_to_dictionary() {
        assert!(dic_variant == dict);
    } else {
        panic!("variant should be a Dictionary");
    }

    let mut iter_keys = HashSet::new();
    let expected_keys = ["foo", "bar"].iter().map(|&s| s.to_string()).collect::<HashSet<_>>();
    for (key, value) in dict.iter() {
        assert_eq!(value, dict.get(&key));
        if !iter_keys.insert(key.to_string()) {
            panic!("key is already contained in set: {:?}", key);
        }
    }
    assert_eq!(expected_keys, iter_keys);
});

// TODO: clear dictionaries without affecting clones
//godot_test!(test_dictionary_clone_clear {
//    let foo = Variant::from_str("foo");
//    let bar = Variant::from_str("bar");
//    let mut dict = Dictionary::new();
//
//    dict.set(&foo, &bar);
//    let dict_clone = dict.clone();
//    dict.clear();
//
//    assert!(dict.is_empty());
//    assert!(!dict_clone.is_empty());
//});
