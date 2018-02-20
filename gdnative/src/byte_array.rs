use sys;
use get_api;
use Variant;
use GodotType;
use VariantArray;

pub struct ByteArray(pub(crate) sys::godot_pool_byte_array);

impl ByteArray {
    pub fn new() -> Self { ByteArray::default() }

    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut result = sys::godot_pool_byte_array::default();
            (get_api().godot_pool_byte_array_new_with_array)(&mut result, &array.0);
            ByteArray(result)
        }
    }

    pub fn push(&mut self, byte: u8) {
        unsafe {
            (get_api().godot_pool_byte_array_append)(&mut self.0, byte);
        }
    }

    pub fn push_array(&mut self, bytes: &ByteArray) {
        unsafe {
            (get_api().godot_pool_byte_array_append_array)(&mut self.0, &bytes.0);
        }
    }

    // TODO(error handling)
    pub fn insert_byte(&mut self, offset: i32, byte: u8) -> bool {
        unsafe {
            let status = (get_api().godot_pool_byte_array_insert)(&mut self.0, offset, byte);
            status != sys::godot_error::GODOT_OK
        }
    }

    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_pool_byte_array_invert)(&mut self.0)
        }
    }

    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_pool_byte_array_remove)(&mut self.0, idx);
        }
    }

    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_pool_byte_array_resize)(&mut self.0, size);
        }
    }

    pub fn get(&self, idx: i32) -> u8 {
        unsafe {
            (get_api().godot_pool_byte_array_get)(&self.0, idx)
        }
    }

    pub fn set(&mut self, idx: i32, byte: u8) {
        unsafe {
            (get_api().godot_pool_byte_array_set)(&mut self.0, idx, byte);
        }
    }

    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_pool_byte_array_size)(&self.0)
        }
    }
}

impl_basic_traits!(
    for ByteArray as godot_pool_byte_array {
        Drop => godot_pool_byte_array_destroy;
        Clone => godot_pool_byte_array_new_copy;
        Default => godot_pool_byte_array_new;
    }
);

impl GodotType for ByteArray {
    fn to_variant(&self) -> Variant { Variant::from_byte_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.to_byte_array() }
}
