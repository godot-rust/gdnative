use sys;
use get_api;
use Variant;
use GodotType;
use Array;
use Vector2;

use std::mem::transmute;

pub struct PoolVector2Array(pub(crate) sys::godot_pool_vector2_array);

impl PoolVector2Array {
    pub fn new() -> Self { PoolVector2Array::default() }

    pub fn from_array(array: &Array) -> Self {
        unsafe {
            let mut result = sys::godot_pool_vector2_array::default();
            (get_api().godot_pool_vector2_array_new_with_array)(&mut result, &array.0);
            PoolVector2Array(result)
        }
    }

    pub fn push(&mut self, vector: &Vector2) {
        unsafe {
            (get_api().godot_pool_vector2_array_append)(&mut self.0, transmute(vector));
        }
    }

    pub fn push_array(&mut self, vectors: &PoolVector2Array) {
        unsafe {
            (get_api().godot_pool_vector2_array_append_array)(&mut self.0, transmute(vectors));
        }
    }

    // TODO(error handling)
    pub fn insert(&mut self, offset: i32, vector: &Vector2) -> bool {
        unsafe {
            let status = (get_api().godot_pool_vector2_array_insert)(&mut self.0, offset, transmute(vector));
            status != sys::godot_error::GODOT_OK
        }
    }

    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_pool_vector2_array_invert)(&mut self.0)
        }
    }

    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_pool_vector2_array_remove)(&mut self.0, idx);
        }
    }

    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_pool_vector2_array_resize)(&mut self.0, size);
        }
    }

    pub fn get(&self, idx: i32) -> Vector2 {
        unsafe {
            transmute((get_api().godot_pool_vector2_array_get)(&self.0, idx))
        }
    }

    pub fn set(&mut self, idx: i32, vector: &Vector2) {
        unsafe {
            (get_api().godot_pool_vector2_array_set)(&mut self.0, idx, transmute(vector));
        }
    }

    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_pool_vector2_array_size)(&self.0)
        }
    }
}

impl_basic_traits!(
    for PoolVector2Array as godot_pool_vector2_array {
        Drop => godot_pool_vector2_array_destroy;
        Clone => godot_pool_vector2_array_new_copy;
        Default => godot_pool_vector2_array_new;
    }
);

impl GodotType for PoolVector2Array {
    fn to_variant(&self) -> Variant { Variant::from_pool_vector2_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.to_pool_vector2_array() }
}
