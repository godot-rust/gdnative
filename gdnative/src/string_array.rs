use sys;
use get_api;
use Variant;
use GodotType;
use VariantArray;
use GodotString;

/// A vector of `GodotString` that uses Godot's pool allocator.
pub struct StringArray(pub(crate) sys::godot_pool_string_array);

impl StringArray {
    pub fn new() -> Self { StringArray::default() }

    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut result = sys::godot_pool_string_array::default();
            (get_api().godot_pool_string_array_new_with_array)(&mut result, &array.0);
            StringArray(result)
        }
    }

    pub fn push(&mut self, s: &GodotString) {
        unsafe {
            (get_api().godot_pool_string_array_append)(&mut self.0, &s.0);
        }
    }

    pub fn push_string_array(&mut self, strings: &StringArray) {
        unsafe {
            (get_api().godot_pool_string_array_append_array)(&mut self.0, &strings.0);
        }
    }

    // TODO(error handling)
    pub fn insert(&mut self, offset: i32, string: &GodotString) -> bool {
        unsafe {
            let status = (get_api().godot_pool_string_array_insert)(&mut self.0, offset, &string.0);
            status != sys::godot_error::GODOT_OK
        }
    }

    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_pool_string_array_invert)(&mut self.0)
        }
    }

    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_pool_string_array_remove)(&mut self.0, idx);
        }
    }

    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_pool_string_array_resize)(&mut self.0, size);
        }
    }

    pub fn get(&self, idx: i32) -> GodotString {
        unsafe {
            GodotString((get_api().godot_pool_string_array_get)(&self.0, idx))
        }
    }

    pub fn set(&mut self, idx: i32, string: &GodotString) {
        unsafe {
            (get_api().godot_pool_string_array_set)(&mut self.0, idx, &string.0);
        }
    }

    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_pool_string_array_size)(&self.0)
        }
    }
}

impl_basic_traits!(
    for StringArray as godot_pool_string_array {
        Drop => godot_pool_string_array_destroy;
        Clone => godot_pool_string_array_new_copy;
        Default => godot_pool_string_array_new;
    }
);

impl GodotType for StringArray {
    fn to_variant(&self) -> Variant { Variant::from_string_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.try_to_string_array() }
}
