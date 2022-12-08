use std::iter::{Extend, FromIterator};
use std::marker::PhantomData;

use crate::private::get_api;
use crate::sys;

use crate::core_types::{GodotString, OwnedToVariant, ToVariantEq, Variant, VariantArray};
use crate::object::NewRef;
use std::fmt;

use crate::object::ownership::*;

/// A reference-counted `Dictionary` of `Variant` key-value pairs.
///
/// Generic methods on this type performs `Variant` conversion every time. This could
/// be significant for complex structures. Users may convert arguments to `Variant`s before
/// calling to avoid this behavior if necessary.
///
/// # Safety
///
/// This is a reference-counted collection with "interior mutability" in Rust parlance.
/// To enforce that the official [thread-safety guidelines][thread-safety] are
/// followed this type uses the *typestate* pattern. The typestate `Ownership` tracks
/// whether there is thread-local or unique access (where pretty much all operations are safe)
/// or whether the value might be "shared", in which case not all operations are
/// safe.
///
/// [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html
pub struct Dictionary<Own: Ownership = Shared> {
    sys: sys::godot_dictionary,

    /// Marker preventing the compiler from incorrectly deriving `Send` and `Sync`.
    _marker: PhantomData<Own>,
}

/// Operations allowed on all Dictionaries at any point in time.
impl<Own: Ownership> Dictionary<Own> {
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
    pub fn contains<K>(&self, key: K) -> bool
    where
        K: OwnedToVariant + ToVariantEq,
    {
        unsafe { (get_api().godot_dictionary_has)(self.sys(), key.owned_to_variant().sys()) }
    }

    /// Returns true if the `Dictionary` has all of the keys in the given array.
    #[inline]
    pub fn contains_all<ArrayOws: Ownership>(&self, keys: &VariantArray<ArrayOws>) -> bool {
        unsafe { (get_api().godot_dictionary_has_all)(self.sys(), keys.sys()) }
    }

    /// Returns a copy of the value corresponding to the key if it exists.
    #[inline]
    pub fn get<K>(&self, key: K) -> Option<Variant>
    where
        K: OwnedToVariant + ToVariantEq,
    {
        let key = key.owned_to_variant();
        if self.contains(&key) {
            // This should never return the default Nil, but there isn't a safe way to otherwise check
            // if the entry exists in a single API call.
            Some(self.get_or_nil(key))
        } else {
            None
        }
    }

    /// Returns a copy of the value corresponding to the key, or `default` if it doesn't exist
    #[inline]
    pub fn get_or<K, D>(&self, key: K, default: D) -> Variant
    where
        K: OwnedToVariant + ToVariantEq,
        D: OwnedToVariant,
    {
        unsafe {
            Variant((get_api().godot_dictionary_get_with_default)(
                self.sys(),
                key.owned_to_variant().sys(),
                default.owned_to_variant().sys(),
            ))
        }
    }

    /// Returns a copy of the element corresponding to the key, or `Nil` if it doesn't exist.
    /// Shorthand for `self.get_or(key, Variant::nil())`.
    #[inline]
    pub fn get_or_nil<K>(&self, key: K) -> Variant
    where
        K: OwnedToVariant + ToVariantEq,
    {
        self.get_or(key, Variant::nil())
    }

    /// Update an existing element corresponding to the key.
    ///
    /// # Panics
    ///
    /// Panics if the entry for `key` does not exist.
    #[inline]
    pub fn update<K, V>(&self, key: K, val: V)
    where
        K: OwnedToVariant + ToVariantEq,
        V: OwnedToVariant,
    {
        let key = key.owned_to_variant();
        assert!(self.contains(&key), "Can only update entries that exist");

        unsafe {
            (get_api().godot_dictionary_set)(
                self.sys_mut(),
                key.sys(),
                val.owned_to_variant().sys(),
            )
        }
    }

    /// Returns a reference to the value corresponding to the key, inserting a `Nil` value first if
    /// it does not exist.
    ///
    /// # Safety
    ///
    /// The returned reference is invalidated if the same container is mutated through another
    /// reference, and other references may be invalidated if the entry does not already exist
    /// (which causes this function to insert `Nil` and thus possibly re-allocate).
    ///
    /// `Variant` is reference-counted and thus cheaply cloned. Consider using `get` instead.
    #[inline]
    pub unsafe fn get_ref<K>(&self, key: K) -> &Variant
    where
        K: OwnedToVariant + ToVariantEq,
    {
        Variant::cast_ref((get_api().godot_dictionary_operator_index_const)(
            self.sys(),
            key.owned_to_variant().sys(),
        ))
    }

    /// Returns a mutable reference to the value corresponding to the key, inserting a `Nil` value
    /// first if it does not exist.
    ///
    /// # Safety
    ///
    /// The returned reference is invalidated if the same container is mutated through another
    /// reference, and other references may be invalidated if the `key` does not already exist
    /// (which causes this function to insert `Nil` and thus possibly re-allocate). It is also
    /// possible to create two mutable references to the same memory location if the same `key`
    /// is provided, causing undefined behavior.
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut_ref<K>(&self, key: K) -> &mut Variant
    where
        K: OwnedToVariant + ToVariantEq,
    {
        Variant::cast_mut_ref((get_api().godot_dictionary_operator_index)(
            self.sys_mut(),
            key.owned_to_variant().sys(),
        ))
    }

    /// Returns a GodotString of the `Dictionary`.
    #[inline]
    pub fn to_json(&self) -> GodotString {
        unsafe { GodotString((get_api().godot_dictionary_to_json)(self.sys())) }
    }

    /// Returns an array of the keys in the `Dictionary`.
    #[inline]
    pub fn keys(&self) -> VariantArray<Unique> {
        unsafe { VariantArray::<Unique>::from_sys((get_api().godot_dictionary_keys)(self.sys())) }
    }

    /// Returns an array of the values in the `Dictionary`.
    #[inline]
    pub fn values(&self) -> VariantArray<Unique> {
        unsafe { VariantArray::<Unique>::from_sys((get_api().godot_dictionary_values)(self.sys())) }
    }

    /// Return a hashed i32 value representing the dictionary's contents.
    #[inline]
    pub fn hash(&self) -> i32 {
        unsafe { (get_api().godot_dictionary_hash)(self.sys()) }
    }

    /// Returns an iterator through all key-value pairs in the `Dictionary`.
    ///
    /// `Dictionary` is reference-counted and have interior mutability in Rust parlance.
    /// Modifying the same underlying collection while observing the safety assumptions will
    /// not violate memory safely, but may lead to surprising behavior in the iterator.
    #[inline]
    pub fn iter(&self) -> Iter<Own> {
        Iter::new(self)
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

    unsafe fn cast_access<A: Ownership>(self) -> Dictionary<A> {
        let sys = self.sys;
        std::mem::forget(self);
        Dictionary::from_sys(sys)
    }
}

/// Operations allowed on Dictionaries that might be shared between different threads.
impl Dictionary<Shared> {
    /// Create a new shared dictionary.
    #[inline]
    pub fn new_shared() -> Self {
        Dictionary::<Unique>::new().into_shared()
    }
}

/// Operations allowed on Dictionaries that may only be shared on the current thread.
impl Dictionary<ThreadLocal> {
    /// Create a new thread-local dictionary.
    #[inline]
    pub fn new_thread_local() -> Self {
        Dictionary::<Unique>::new().into_thread_local()
    }
}

/// Operations allowed on Dictionaries that are not unique.
impl<Own: NonUniqueOwnership> Dictionary<Own> {
    /// Assume that this is the only reference to this dictionary, on which
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
    pub unsafe fn assume_unique(self) -> Dictionary<Unique> {
        self.cast_access()
    }
}

/// Operations allowed on Dictionaries that can only be referenced to from the current thread.
impl<Own: LocalThreadOwnership> Dictionary<Own> {
    #[inline]
    /// Inserts or updates the value of the element corresponding to the key.
    pub fn insert<K, V>(&self, key: K, val: V)
    where
        K: OwnedToVariant + ToVariantEq,
        V: OwnedToVariant,
    {
        unsafe {
            (get_api().godot_dictionary_set)(
                self.sys_mut(),
                key.owned_to_variant().sys(),
                val.owned_to_variant().sys(),
            )
        }
    }

    /// Erase a key-value pair in the `Dictionary` by the specified key.
    #[inline]
    pub fn erase<K>(&self, key: K)
    where
        K: OwnedToVariant + ToVariantEq,
    {
        unsafe { (get_api().godot_dictionary_erase)(self.sys_mut(), key.owned_to_variant().sys()) }
    }

    /// Clears the `Dictionary`, removing all key-value pairs.
    #[inline]
    pub fn clear(&self) {
        unsafe { (get_api().godot_dictionary_clear)(self.sys_mut()) }
    }
}

/// Operations allowed on unique Dictionaries.
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
        unsafe { self.cast_access() }
    }

    /// Put this dictionary under the "thread-local" access type.
    #[inline]
    pub fn into_thread_local(self) -> Dictionary<ThreadLocal> {
        unsafe { self.cast_access() }
    }
}

impl<Own: Ownership> Drop for Dictionary<Own> {
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

impl Default for Dictionary<ThreadLocal> {
    #[inline]
    fn default() -> Self {
        Dictionary::<Unique>::default().into_thread_local()
    }
}

impl<Own: NonUniqueOwnership> NewRef for Dictionary<Own> {
    #[inline]
    fn new_ref(&self) -> Self {
        unsafe {
            let mut result = Default::default();
            (get_api().godot_dictionary_new_copy)(&mut result, self.sys());
            Self::from_sys(result)
        }
    }
}

impl From<Dictionary<Unique>> for Dictionary<Shared> {
    #[inline]
    fn from(dict: Dictionary<Unique>) -> Self {
        dict.into_shared()
    }
}

impl From<Dictionary<Unique>> for Dictionary<ThreadLocal> {
    #[inline]
    fn from(dict: Dictionary<Unique>) -> Self {
        dict.into_thread_local()
    }
}

impl<Own: Ownership> fmt::Debug for Dictionary<Own> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.to_json().to_string().fmt(f)
    }
}

unsafe fn iter_next<Own: Ownership>(
    dic: &Dictionary<Own>,
    last_key: &mut Option<Variant>,
) -> Option<(Variant, Variant)> {
    let last_ptr = last_key.as_ref().map_or(std::ptr::null(), Variant::sys);
    let next_ptr = (get_api().godot_dictionary_next)(dic.sys(), last_ptr);

    if next_ptr.is_null() {
        None
    } else {
        let key = Variant::cast_ref(next_ptr).clone();
        let value = dic.get_or_nil(&key);
        *last_key = Some(key.clone());
        Some((key, value))
    }
}

/// Iterator through all key-value pairs in a unique `Dictionary`.
///
/// This struct is created by the `iter` method on `Dictionary<Unique>`.
#[derive(Debug)]
pub struct Iter<'a, Own: Ownership> {
    dic: &'a Dictionary<Own>,
    last_key: Option<Variant>,
}

impl<'a, Own: Ownership> Iter<'a, Own> {
    /// Create an Iterator from a unique Dictionary.
    fn new(dic: &'a Dictionary<Own>) -> Self {
        Iter {
            dic,
            last_key: None,
        }
    }
}

impl<'a, Own: Ownership> Iterator for Iter<'a, Own> {
    type Item = (Variant, Variant);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { iter_next(self.dic, &mut self.last_key) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, usize::try_from(self.dic.len()).ok())
    }
}

impl<'a, Own: Ownership> IntoIterator for &'a Dictionary<Own> {
    type Item = (Variant, Variant);
    type IntoIter = Iter<'a, Own>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator through all key-value pairs in a unique `Dictionary`.
///
/// This struct is created by the `into_iter` method on `Dictionary<Unique>`.
/// This iterator consumes the unique dictionary.
#[derive(Debug)]
pub struct IntoIter {
    dic: Dictionary<Unique>,
    last_key: Option<Variant>,
}

impl IntoIter {
    /// Create an Iterator by consuming a unique Dictionary.
    fn new(dic: Dictionary<Unique>) -> Self {
        IntoIter {
            dic,
            last_key: None,
        }
    }
}

impl Iterator for IntoIter {
    type Item = (Variant, Variant);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        unsafe { iter_next(&self.dic, &mut self.last_key) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, usize::try_from(self.dic.len()).ok())
    }
}

impl IntoIterator for Dictionary<Unique> {
    type Item = (Variant, Variant);
    type IntoIter = IntoIter;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<K, V> FromIterator<(K, V)> for Dictionary<Unique>
where
    K: ToVariantEq + OwnedToVariant,
    V: OwnedToVariant,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut dic = Dictionary::new();
        dic.extend(iter);
        dic
    }
}

impl<K, V, Own: LocalThreadOwnership> Extend<(K, V)> for Dictionary<Own>
where
    K: ToVariantEq + OwnedToVariant,
    V: OwnedToVariant,
{
    #[inline]
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(&key.owned_to_variant(), &value.owned_to_variant());
        }
    }
}

godot_test!(test_dictionary {
    use std::collections::HashSet;

    use crate::core_types::VariantType;
    let foo = Variant::new("foo");
    let bar = Variant::new("bar");
    let nope = Variant::new("nope");

    let x = Variant::new(42);
    let y = Variant::new(1337);

    let dict = Dictionary::new();

    dict.insert(&foo, &x);
    dict.insert(&bar, &y);

    assert!(dict.contains(&foo));
    assert!(dict.contains(&bar));
    assert!(!dict.contains(&nope));

    let keys_array = dict.keys();
    let baz = Variant::new("baz");
    keys_array.push(&baz);
    dict.insert(&baz, &x);

    assert!(dict.contains_all(&keys_array));

    dict.erase(&baz);

    assert!(!dict.contains_all(&keys_array));

    let variant = Variant::new(&dict.duplicate().into_shared());
    assert!(variant.get_type() == VariantType::Dictionary);

    let dict2 = dict.duplicate();
    assert!(dict2.contains(&foo));
    assert!(dict2.contains(&bar));

    if let Ok(dic_variant) = variant.try_to::<Dictionary>() {
        assert!(dic_variant.len() == dict.len());
    } else {
        panic!("variant should be a Dictionary");
    }

    let mut iter_keys = HashSet::new();
    let expected_keys = ["foo", "bar"].iter().map(|&s| s.to_string()).collect::<HashSet<_>>();
    for (key, value) in &dict {
        assert_eq!(Some(value), dict.get(&key));
        assert!(
            iter_keys.insert(key.to_string()) ,
            "key is already contained in set: {key:?}"
        );
    }
    assert_eq!(expected_keys, iter_keys);
});

godot_test!(test_dictionary_clone_clear {
    let foo = Variant::new("foo");
    let bar = Variant::new("bar");
    let dict = Dictionary::new();

    dict.insert(&foo, &bar);
    let dict_clone = dict.duplicate();
    dict.clear();

    assert!(dict.is_empty());
    assert!(!dict_clone.is_empty());
});
