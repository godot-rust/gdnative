use std::iter::{Extend, FromIterator};

use crate::private::get_api;
use crate::sys;
use crate::GodotString;

use crate::RefCounted;
use crate::ToVariant;
use crate::ToVariantEq;
use crate::Variant;
use crate::VariantArray;
use std::fmt;

/// A reference-counted `Dictionary` of `Variant` key-value pairs.
///
/// # Safety
///
/// This is a reference-counted collection with "interior mutability" in Rust parlance. Its use
/// must follow the official [thread-safety guidelines][thread-safety]. Specifically, it is
/// undefined behavior to pass an instance to Rust code without locking a mutex if there are
/// references to it on other threads.
///
/// [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html
pub struct Dictionary(pub(crate) sys::godot_dictionary);

impl Dictionary {
    /// Creates an empty `Dictionary`.
    #[inline]
    pub fn new() -> Self {
        Dictionary::default()
    }

    /// Returns `true` if the `Dictionary` contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        unsafe { (get_api().godot_dictionary_empty)(&self.0) }
    }

    /// Returns the number of elements in the `Dictionary`.
    #[inline]
    pub fn len(&self) -> i32 {
        unsafe { (get_api().godot_dictionary_size)(&self.0) }
    }

    /// Clears the `Dictionary`, removing all key-value pairs.
    #[inline]
    pub fn clear(&mut self) {
        unsafe { (get_api().godot_dictionary_clear)(&mut self.0) }
    }

    /// Returns true if the `Dictionary` contains the specified key.
    #[inline]
    pub fn contains(&self, key: &Variant) -> bool {
        unsafe { (get_api().godot_dictionary_has)(&self.0, &key.0) }
    }

    /// Returns true if the `Dictionary` has all of the keys in the given array.
    #[inline]
    pub fn contains_all(&self, keys: &VariantArray) -> bool {
        unsafe { (get_api().godot_dictionary_has_all)(&self.0, &keys.0) }
    }

    /// Erase a key-value pair in the `Dictionary` by the specified key.
    #[inline]
    pub fn erase(&mut self, key: &Variant) {
        unsafe { (get_api().godot_dictionary_erase)(&mut self.0, &key.0) }
    }

    /// Returns a copy of the value corresponding to the key.
    #[inline]
    pub fn get(&self, key: &Variant) -> Variant {
        unsafe { Variant((get_api().godot_dictionary_get)(&self.0, &key.0)) }
    }

    /// Sets a value to the element corresponding to the key.
    #[inline]
    pub fn set(&mut self, key: &Variant, val: &Variant) {
        unsafe { (get_api().godot_dictionary_set)(&mut self.0, &key.0, &val.0) }
    }

    /// Returns a reference to the value corresponding to the key.
    #[inline]
    pub fn get_ref(&self, key: &Variant) -> &Variant {
        unsafe {
            Variant::cast_ref((get_api().godot_dictionary_operator_index_const)(
                &self.0, &key.0,
            ))
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    #[inline]
    pub fn get_mut_ref(&mut self, key: &Variant) -> &mut Variant {
        unsafe {
            Variant::cast_mut_ref((get_api().godot_dictionary_operator_index)(
                &mut self.0,
                &key.0,
            ))
        }
    }

    /// Returns a GodotString of the `Dictionary`.
    #[inline]
    pub fn to_json(&self) -> GodotString {
        unsafe { GodotString((get_api().godot_dictionary_to_json)(&self.0)) }
    }

    /// Returns an array of the keys in the `Dictionary`.
    #[inline]
    pub fn keys(&self) -> VariantArray {
        unsafe { VariantArray((get_api().godot_dictionary_keys)(&self.0)) }
    }

    /// Returns an array of the values in the `Dictionary`.
    #[inline]
    pub fn values(&self) -> VariantArray {
        unsafe { VariantArray((get_api().godot_dictionary_values)(&self.0)) }
    }

    #[inline]
    pub fn get_next(&self, key: &Variant) -> &Variant {
        unsafe { Variant::cast_ref((get_api().godot_dictionary_next)(&self.0, &key.0)) }
    }

    /// Return a hashed i32 value representing the dictionary's contents.
    #[inline]
    pub fn hash(&self) -> i32 {
        unsafe { (get_api().godot_dictionary_hash)(&self.0) }
    }

    /// Returns an iterator through all key-value pairs in the `Dictionary`.
    ///
    /// `Dictionary` is reference-counted and have interior mutability in Rust parlance.
    /// Modifying the same underlying collection while observing the safety assumptions will
    /// not violate memory safely, but may lead to surprising behavior in the iterator.
    #[inline]
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }

    #[doc(hidden)]
    #[inline]
    pub fn sys(&self) -> *const sys::godot_dictionary {
        &self.0
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_sys(sys: sys::godot_dictionary) -> Self {
        Dictionary(sys)
    }
}

impl RefCounted for Dictionary {
    impl_common_methods! {
        #[inline]
        fn new_ref(&self) -> Dictionary : godot_dictionary_new_copy;
    }
}

impl_basic_traits!(
    for Dictionary as godot_dictionary {
        Drop => godot_dictionary_destroy;
        Default => godot_dictionary_new;
        Eq => godot_dictionary_operator_equal;
    }
);

impl fmt::Debug for Dictionary {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.to_json().to_string().fmt(f)
    }
}

/// Iterator through all key-value pairs in a `Dictionary`.
///
/// This struct is created by the `iter` method on `Dictionary`.
#[derive(Debug)]
pub struct Iter {
    dic: Dictionary,
    last_key: Option<Variant>,
}

impl Iter {
    fn new(dic: &Dictionary) -> Self {
        Iter {
            dic: dic.new_ref(),
            last_key: None,
        }
    }
}

impl Iterator for Iter {
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

impl<K, V> FromIterator<(K, V)> for Dictionary
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

impl FromIterator<(Variant, Variant)> for Dictionary {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (Variant, Variant)>>(iter: I) -> Self {
        let mut dic = Dictionary::new();
        dic.extend(iter);
        dic
    }
}

impl<K, V> Extend<(K, V)> for Dictionary
where
    K: ToVariantEq,
    V: ToVariant,
{
    #[inline]
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.set(&key.to_variant(), &value.to_variant());
        }
    }
}

impl Extend<(Variant, Variant)> for Dictionary {
    #[inline]
    fn extend<I: IntoIterator<Item = (Variant, Variant)>>(&mut self, iter: I) {
        for (key, value) in iter {
            self.set(&key.to_variant(), &value.to_variant());
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

    let mut dict = Dictionary::new();

    dict.set(&foo, &x);
    dict.set(&bar, &y);

    assert!(dict.contains(&foo));
    assert!(dict.contains(&bar));
    assert!(!dict.contains(&nope));

    let mut keys_array = dict.keys();
    let baz = Variant::from_str("baz");
    keys_array.push(&baz);
    dict.set(&baz, &x);

    assert!(dict.contains_all(&keys_array));

    dict.erase(&baz);

    assert!(!dict.contains_all(&keys_array));

    let variant = Variant::from_dictionary(&dict);
    assert!(variant.get_type() == VariantType::Dictionary);

    let dict2 = dict.new_ref();
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
