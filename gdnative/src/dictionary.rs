use sys;
use get_api;
use Variant;
use VariantArray;
use GodotString;
use GodotType;

pub struct Dictionary(pub(crate) sys::godot_dictionary);

impl Dictionary {
    pub fn new() -> Self { Dictionary::default() }

    pub fn is_empty(&self) -> bool {
        unsafe {
            (get_api().godot_dictionary_empty)(&self.0)
        }
    }

    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_dictionary_size)(&self.0)
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            (get_api().godot_dictionary_clear)(&mut self.0)
        }
    }

    pub fn contains(&self, key: &Variant) -> bool {
        unsafe {
            (get_api().godot_dictionary_has)(&self.0, &key.0)
        }
    }

    pub fn contains_all(&self, keys: &VariantArray) -> bool {
        unsafe {
            (get_api().godot_dictionary_has_all)(&self.0, &keys.0)
        }
    }

    pub fn erase(&mut self, key: &Variant) {
        unsafe {
            (get_api().godot_dictionary_erase)(&mut self.0, &key.0)
        }
    }

    pub fn get(&self, key: &Variant) -> Variant {
        unsafe {
            Variant((get_api().godot_dictionary_get)(&self.0, &key.0))
        }
    }

    pub fn set(&mut self, key: &Variant, val: &Variant) {
        unsafe {
            (get_api().godot_dictionary_set)(&mut self.0, &key.0, &val.0)
        }
    }

    pub fn get_ref(&self, key: &Variant) -> &Variant {
        unsafe {
            Variant::cast_ref((get_api().godot_dictionary_operator_index_const)(&self.0, &key.0))
        }
    }

    pub fn get_mut_ref(&mut self, key: &Variant) -> &mut Variant {
        unsafe {
            Variant::cast_mut_ref((get_api().godot_dictionary_operator_index)(&mut self.0, &key.0))
        }
    }

    pub fn to_json(&self) -> GodotString {
        unsafe {
            GodotString((get_api().godot_dictionary_to_json)(&self.0))
        }
    }

    pub fn keys(&self) -> VariantArray {
        unsafe {
            VariantArray((get_api().godot_dictionary_keys)(&self.0))
        }
    }

    pub fn values(&self) -> VariantArray {
        unsafe {
            VariantArray((get_api().godot_dictionary_values)(&self.0))
        }
    }

    pub fn get_next(&self, key: &Variant) -> &Variant {
        unsafe {
            Variant::cast_ref((get_api().godot_dictionary_next)(&self.0, &key.0))
        }
    }

    pub fn hash(&self) -> i32 {
        unsafe {
            (get_api().godot_dictionary_hash)(&self.0)
        }
    }
}

impl_basic_traits!(
    for Dictionary as godot_dictionary {
        Drop => godot_dictionary_destroy;
        Clone => godot_dictionary_new_copy;
        Default => godot_dictionary_new;
        Eq => godot_dictionary_operator_equal;
    }
);

impl GodotType for Dictionary {
    fn to_variant(&self) -> Variant { Variant::from_dictionary(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.to_dictionary() }
}

godot_test!(test_dictionary {
    use VariantType;
    let foo = Variant::from_str("foo");
    let bar = Variant::from_str("bar");
    let nope = Variant::from_str("nope");

    let mut dict = Dictionary::new();

    dict.set(&foo, &Variant::from_i64(42));
    dict.set(&bar, &Variant::from_i64(1337));

    assert!(dict.contains(&foo));
    assert!(dict.contains(&bar));
    assert!(!dict.contains(&nope));

    let variant = Variant::from_dictionary(&dict);
    assert!(variant.get_type() == VariantType::Dictionary);

    let dict_clone = dict.clone();
    assert!(dict == dict_clone);
    assert!(dict_clone.contains(&foo));
    assert!(dict_clone.contains(&bar));

    if let Some(dic_variant) = variant.to_dictionary() {
        assert!(dic_variant == dict);
    } else {
        panic!("variant should be a Dictionary");
    }
});
