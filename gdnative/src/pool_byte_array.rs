use sys;
use get_api;
use Variant;
use GodotType;
use Array;

pub struct PoolByteArray(pub(crate) sys::godot_pool_byte_array);

impl PoolByteArray {
    pub fn new() -> Self { PoolByteArray::default() }

    pub fn from_array(array: &Array) -> Self {
        unsafe {
            let mut result = sys::godot_pool_byte_array::default();
            (get_api().godot_pool_byte_array_new_with_array)(&mut result, &array.0);
            PoolByteArray(result)
        }
    }

    pub fn push_byte(&mut self, byte: u8) {
        unsafe {
            // TODO: what's the difference between append and push_back.
            (get_api().godot_pool_byte_array_append)(&mut self.0, byte);
        }
    }

    pub fn push_byte_array(&mut self, bytes: &PoolByteArray) {
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
    for PoolByteArray as godot_pool_byte_array {
        Drop => godot_pool_byte_array_destroy;
        Clone => godot_pool_byte_array_new_copy;
        Default => godot_pool_byte_array_new;
    }
);

impl GodotType for PoolByteArray {
    fn to_variant(&self) -> Variant { Variant::from_pool_byte_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.to_pool_byte_array() }
}
