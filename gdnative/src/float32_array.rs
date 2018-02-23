use sys;
use get_api;
use Variant;
use GodotType;
use VariantArray;

pub struct Float32Array(pub(crate) sys::godot_pool_real_array);

impl Float32Array {
    pub fn new() -> Self { Float32Array::default() }

    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut result = sys::godot_pool_real_array::default();
            (get_api().godot_pool_real_array_new_with_array)(&mut result, &array.0);
            Float32Array(result)
        }
    }

    pub fn push(&mut self, val: f32) {
        unsafe {
            (get_api().godot_pool_real_array_append)(&mut self.0, val);
        }
    }

    pub fn push_array(&mut self, array: &Float32Array) {
        unsafe {
            (get_api().godot_pool_real_array_append_array)(&mut self.0, &array.0);
        }
    }

    // TODO(error handling)
    pub fn insert(&mut self, offset: i32, val: f32) -> bool {
        unsafe {
            let status = (get_api().godot_pool_real_array_insert)(&mut self.0, offset, val);
            status != sys::godot_error::GODOT_OK
        }
    }

    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_pool_real_array_invert)(&mut self.0)
        }
    }

    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_pool_real_array_remove)(&mut self.0, idx);
        }
    }

    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_pool_real_array_resize)(&mut self.0, size);
        }
    }

    pub fn get(&self, idx: i32) -> f32 {
        unsafe {
            (get_api().godot_pool_real_array_get)(&self.0, idx)
        }
    }

    pub fn set(&mut self, idx: i32, val: f32) {
        unsafe {
            (get_api().godot_pool_real_array_set)(&mut self.0, idx, val);
        }
    }

    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_pool_real_array_size)(&self.0)
        }
    }
}

impl_basic_traits!(
    for Float32Array as godot_pool_real_array {
        Drop => godot_pool_real_array_destroy;
        Clone => godot_pool_real_array_new_copy;
        Default => godot_pool_real_array_new;
    }
);

impl GodotType for Float32Array {
    fn to_variant(&self) -> Variant { Variant::from_float32_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.try_to_float32_array() }
}
