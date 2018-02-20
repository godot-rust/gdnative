use sys;
use get_api;
use Variant;
use GodotType;
use Array;
use Vector3;

use std::mem::transmute;

pub struct PoolVector3Array(pub(crate) sys::godot_pool_vector3_array);

impl PoolVector3Array {
    pub fn new() -> Self { PoolVector3Array::default() }

    pub fn from_array(array: &Array) -> Self {
        unsafe {
            let mut result = sys::godot_pool_vector3_array::default();
            (get_api().godot_pool_vector3_array_new_with_array)(&mut result, &array.0);
            PoolVector3Array(result)
        }
    }

    pub fn push(&mut self, vector: &Vector3) {
        unsafe {
            (get_api().godot_pool_vector3_array_append)(&mut self.0, transmute(vector));
        }
    }

    pub fn push_array(&mut self, vectors: &PoolVector3Array) {
        unsafe {
            (get_api().godot_pool_vector3_array_append_array)(&mut self.0, transmute(vectors));
        }
    }

    // TODO(error handling)
    pub fn insert(&mut self, offset: i32, vector: &Vector3) -> bool {
        unsafe {
            let status = (get_api().godot_pool_vector3_array_insert)(&mut self.0, offset, transmute(vector));
            status != sys::godot_error::GODOT_OK
        }
    }

    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_pool_vector3_array_invert)(&mut self.0)
        }
    }

    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_pool_vector3_array_remove)(&mut self.0, idx);
        }
    }

    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_pool_vector3_array_resize)(&mut self.0, size);
        }
    }

    pub fn get(&self, idx: i32) -> Vector3 {
        unsafe {
            transmute((get_api().godot_pool_vector3_array_get)(&self.0, idx))
        }
    }

    pub fn set(&mut self, idx: i32, vector: &Vector3) {
        unsafe {
            (get_api().godot_pool_vector3_array_set)(&mut self.0, idx, transmute(vector));
        }
    }

    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_pool_vector3_array_size)(&self.0)
        }
    }
}

impl_basic_traits!(
    for PoolVector3Array as godot_pool_vector3_array {
        Drop => godot_pool_vector3_array_destroy;
        Clone => godot_pool_vector3_array_new_copy;
        Default => godot_pool_vector3_array_new;
    }
);

impl GodotType for PoolVector3Array {
    fn to_variant(&self) -> Variant { Variant::from_pool_vector3_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.to_pool_vector3_array() }
}
