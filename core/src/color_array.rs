use sys;
use get_api;
use Variant;
use ToVariant;
use VariantArray;
use Color;

use std::mem::transmute;

/// A reference-counted vector of `ColorArray` that uses Godot's pool allocator.
pub struct ColorArray(pub(crate) sys::godot_pool_color_array);

impl ColorArray {
    /// Creates an empty `ColorArray`.
    pub fn new() -> Self { ColorArray::default() }

    /// Creates an array by trying to convert each variant.
    ///
    /// See `Variant::to_color_array`.
    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut result = sys::godot_pool_color_array::default();
            (get_api().godot_pool_color_array_new_with_array)(&mut result, &array.0);
            ColorArray(result)
        }
    }

    /// Appends an element at the end of the array
    pub fn push(&mut self, color: &Color) {
        unsafe {
            (get_api().godot_pool_color_array_append)(&mut self.0, transmute(color));
        }
    }

    /// Appends a `ColorArray` at the end of this array.
    pub fn push_array(&mut self, array: &ColorArray) {
        unsafe {
            (get_api().godot_pool_color_array_append_array)(&mut self.0, transmute(array));
        }
    }

    // TODO(error handling)
    /// Insert a new int at a given position in the array.
    pub fn insert(&mut self, offset: i32, color: &Color) -> bool {
        unsafe {
            let status = (get_api().godot_pool_color_array_insert)(&mut self.0, offset, transmute(color));
            status != sys::godot_error::GODOT_OK
        }
    }

    /// Inverts the order of the elements in the array.
    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_pool_color_array_invert)(&mut self.0)
        }
    }

    /// Removes an element at the given offset.
    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_pool_color_array_remove)(&mut self.0, idx);
        }
    }

    /// Changes the size of the array, possibly removing elements or pushing default values.
    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_pool_color_array_resize)(&mut self.0, size);
        }
    }

    /// Returns a copy of the element at the given offset.
    pub fn get(&self, idx: i32) -> Color {
        unsafe {
            transmute((get_api().godot_pool_color_array_get)(&self.0, idx))
        }
    }

    /// Sets the value of the element at the given offset.
    pub fn set(&mut self, idx: i32, color: &Color) {
        unsafe {
            (get_api().godot_pool_color_array_set)(&mut self.0, idx, transmute(color));
        }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_pool_color_array_size)(&self.0)
        }
    }

    #[doc(hidden)]
    pub fn sys(&self) -> *const sys::godot_pool_color_array {
        &self.0
    }

    #[doc(hidden)]
    pub fn from_sys(sys: sys::godot_pool_color_array) -> Self {
        ColorArray(sys)
    }

    impl_common_methods! {
        /// Creates a new reference to this array.
        pub fn new_ref(&self) -> ColorArray : godot_pool_color_array_new_copy;
    }
}

impl_basic_traits!(
    for ColorArray as godot_pool_color_array {
        Drop => godot_pool_color_array_destroy;
        Default => godot_pool_color_array_new;
    }
);

impl ToVariant for ColorArray {
    fn to_variant(&self) -> Variant { Variant::from_color_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.try_to_color_array() }
}
