use sys;
use get_api;
use Variant;
use GodotType;
use VariantArray;
use Vector2;

use std::mem::transmute;

/// A vector of `Vector2` that uses Godot's pool allocator.
pub struct Vector2Array(pub(crate) sys::godot_pool_vector2_array);

impl Vector2Array {
    /// Creates an empty array.
    pub fn new() -> Self { Vector2Array::default() }

    /// Creates an array by trying to convert each variant
    ///
    /// See `Variant::to_vector2`.
    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut result = sys::godot_pool_vector2_array::default();
            (get_api().godot_pool_vector2_array_new_with_array)(&mut result, &array.0);
            Vector2Array(result)
        }
    }

    // Appends a vector to the end of the array.
    pub fn push(&mut self, vector: &Vector2) {
        unsafe {
            (get_api().godot_pool_vector2_array_append)(&mut self.0, transmute(vector));
        }
    }

    // Appends each vector to the end of the array.
    pub fn push_array(&mut self, vectors: &Vector2Array) {
        unsafe {
            (get_api().godot_pool_vector2_array_append_array)(&mut self.0, transmute(vectors));
        }
    }

    // TODO(error handling)
    /// Inserts a vector at the given offset.
    pub fn insert(&mut self, offset: i32, vector: &Vector2) -> bool {
        unsafe {
            let status = (get_api().godot_pool_vector2_array_insert)(&mut self.0, offset, transmute(vector));
            status != sys::godot_error::GODOT_OK
        }
    }

    /// Inverts the order of the elements in the array.
    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_pool_vector2_array_invert)(&mut self.0)
        }
    }

    /// Removes an element at the given offset.
    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_pool_vector2_array_remove)(&mut self.0, idx);
        }
    }

    /// Changes the size of the array, possibly removing elements or pushing default values.
    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_pool_vector2_array_resize)(&mut self.0, size);
        }
    }

    /// Gets a copy of the element at the given offset.
    pub fn get(&self, idx: i32) -> Vector2 {
        unsafe {
            transmute((get_api().godot_pool_vector2_array_get)(&self.0, idx))
        }
    }

    /// Sets the value of the element at the given offset.
    pub fn set(&mut self, idx: i32, vector: &Vector2) {
        unsafe {
            (get_api().godot_pool_vector2_array_set)(&mut self.0, idx, transmute(vector));
        }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_pool_vector2_array_size)(&self.0)
        }
    }
}

impl_basic_traits!(
    for Vector2Array as godot_pool_vector2_array {
        Drop => godot_pool_vector2_array_destroy;
        Clone => godot_pool_vector2_array_new_copy;
        Default => godot_pool_vector2_array_new;
    }
);

impl GodotType for Vector2Array {
    fn to_variant(&self) -> Variant { Variant::from_vector2_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.to_vector2_array() }
}
