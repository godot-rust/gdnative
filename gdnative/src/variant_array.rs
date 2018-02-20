use sys;
use get_api;
use Variant;
use GodotType;

/// An array of `Variant`.
pub struct VariantArray(pub(crate) sys::godot_array);

impl VariantArray {
    pub fn new() -> Self { VariantArray::default() }

    pub fn set(&mut self, idx: i32, val: &Variant) {
        unsafe {
            (get_api().godot_array_set)(&mut self.0, idx, &val.0)
        }
    }

    pub fn get_val(&mut self, idx: i32) -> Variant {
        unsafe {
            Variant((get_api().godot_array_get)(&self.0, idx))
        }
    }

    pub fn get_ref(&self, idx: i32) -> &Variant {
        unsafe {
            Variant::cast_ref(
                (get_api().godot_array_operator_index_const)(&self.0, idx)
            )
        }
    }

    pub fn get_mut_ref(&mut self, idx: i32) -> &mut Variant {
        unsafe {
            Variant::cast_mut_ref((get_api().godot_array_operator_index)(&mut self.0, idx))
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            (get_api().godot_array_clear)(&mut self.0);
        }
    }

    pub fn is_empty(&self) {
        unsafe {
            (get_api().godot_array_empty)(&self.0);
        }
    }

    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_array_size)(&self.0)
        }
    }

    pub fn push(&mut self, val: &Variant) {
        unsafe {
            (get_api().godot_array_push_back)(&mut self.0, &val.0);
        }
    }

    pub fn pop(&mut self) -> Variant {
        unsafe {
            Variant((get_api().godot_array_pop_back)(&mut   self.0))
        }
    }

    pub fn push_front(&mut self, val: &Variant) {
        unsafe {
            (get_api().godot_array_push_front)(&mut self.0, &val.0);
        }
    }

    pub fn pop_front(&mut self) -> Variant {
        unsafe {
            Variant((get_api().godot_array_pop_front)(&mut self.0))
        }
    }

    pub fn insert(&mut self, at: i32, val: &Variant) {
        unsafe {
            (get_api().godot_array_insert)(&mut self.0, at, &val.0)
        }
    }

    pub fn find(&self, what: &Variant, from: i32) -> i32 {
        unsafe {
            (get_api().godot_array_find)(&self.0, &what.0, from)
        }
    }

    pub fn contains(&self, what: &Variant) -> bool {
        unsafe {
            (get_api().godot_array_has)(&self.0, &what.0)
        }
    }

    pub fn rfind(&self, what: &Variant, from: i32) -> i32 {
        unsafe {
            (get_api().godot_array_rfind)(&self.0, &what.0, from)
        }
    }

    pub fn find_last(&self, what: &Variant) -> i32 {
        unsafe {
            (get_api().godot_array_find_last)(&self.0, &what.0)
        }
    }

    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_array_invert)(&mut self.0)
        }
    }

    pub fn hash(&self) -> i32 {
        unsafe {
            (get_api().godot_array_hash)(&self.0)
        }
    }
}

impl_basic_traits!(
    for VariantArray as godot_array {
        Drop => godot_array_destroy;
        Clone => godot_array_new_copy;
        Default => godot_array_new;
    }
);

impl GodotType for VariantArray {
    fn to_variant(&self) -> Variant { Variant::from_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.to_array() }
}
