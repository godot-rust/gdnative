use sys;
use get_api;
use Variant;
use GodotType;
use VariantArray;
use Color;

use std::mem::transmute;

pub struct ColorArray(pub(crate) sys::godot_pool_color_array);

impl ColorArray {
    pub fn new() -> Self { ColorArray::default() }

    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut result = sys::godot_pool_color_array::default();
            (get_api().godot_pool_color_array_new_with_array)(&mut result, &array.0);
            ColorArray(result)
        }
    }

    pub fn push(&mut self, color: &Color) {
        unsafe {
            (get_api().godot_pool_color_array_append)(&mut self.0, transmute(color));
        }
    }

    pub fn push_array(&mut self, array: &ColorArray) {
        unsafe {
            (get_api().godot_pool_color_array_append_array)(&mut self.0, transmute(array));
        }
    }

    // TODO(error handling)
    pub fn insert(&mut self, offset: i32, color: &Color) -> bool {
        unsafe {
            let status = (get_api().godot_pool_color_array_insert)(&mut self.0, offset, transmute(color));
            status != sys::godot_error::GODOT_OK
        }
    }

    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_pool_color_array_invert)(&mut self.0)
        }
    }

    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_pool_color_array_remove)(&mut self.0, idx);
        }
    }

    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_pool_color_array_resize)(&mut self.0, size);
        }
    }

    pub fn get(&self, idx: i32) -> Color {
        unsafe {
            transmute((get_api().godot_pool_color_array_get)(&self.0, idx))
        }
    }

    pub fn set(&mut self, idx: i32, color: &Color) {
        unsafe {
            (get_api().godot_pool_color_array_set)(&mut self.0, idx, transmute(color));
        }
    }

    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_pool_color_array_size)(&self.0)
        }
    }
}

impl_basic_traits!(
    for ColorArray as godot_pool_color_array {
        Drop => godot_pool_color_array_destroy;
        Clone => godot_pool_color_array_new_copy;
        Default => godot_pool_color_array_new;
    }
);

impl GodotType for ColorArray {
    fn to_variant(&self) -> Variant { Variant::from_color_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.try_to_color_array() }
}
