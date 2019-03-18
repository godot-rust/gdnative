use crate::sys;
use crate::get_api;
use crate::Variant;
use crate::ToVariant;
use crate::VariantArray;

/// A reference-counted vector of `i32` that uses Godot's pool allocator.
pub struct Int32Array(pub(crate) sys::godot_pool_int_array);

impl Int32Array {
    /// Creates an empty `Int32Array`.
    pub fn new() -> Self { Int32Array::default() }

    /// Creates an array by trying to convert each variant.
    ///
    /// See `Variant::to_int32_array`.
    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut result = sys::godot_pool_int_array::default();
            (get_api().godot_pool_int_array_new_with_array)(&mut result, &array.0);
            Int32Array(result)
        }
    }

    /// Appends an element at the end of the array.
    pub fn push(&mut self, val: i32) {
        unsafe {
            (get_api().godot_pool_int_array_append)(&mut self.0, val);
        }
    }

    /// Appends an `Int32Array` at the end of this array.
    pub fn push_array(&mut self, array: &Int32Array) {
        unsafe {
            (get_api().godot_pool_int_array_append_array)(&mut self.0, &array.0);
        }
    }

    // TODO(error handling)
    /// Insert a new int at a given position in the array.
    pub fn insert(&mut self, offset: i32, val: i32) -> bool {
        unsafe {
            let status = (get_api().godot_pool_int_array_insert)(&mut self.0, offset, val);
            status != sys::godot_error_GODOT_OK
        }
    }

    /// Inverts the order of the elements in the array.
    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_pool_int_array_invert)(&mut self.0)
        }
    }

    /// Removes an element at the given offset.
    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_pool_int_array_remove)(&mut self.0, idx);
        }
    }

    /// Changes the size of the array, possibly removing elements or pushing default values.
    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_pool_int_array_resize)(&mut self.0, size);
        }
    }

    /// Returns a copy of the element at the given offset.
    pub fn get(&self, idx: i32) -> i32 {
        unsafe {
            (get_api().godot_pool_int_array_get)(&self.0, idx)
        }
    }

    /// Sets the value of the element at the given offset.
    pub fn set(&mut self, idx: i32, val: i32) {
        unsafe {
            (get_api().godot_pool_int_array_set)(&mut self.0, idx, val);
        }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_pool_int_array_size)(&self.0)
        }
    }

    #[doc(hidden)]
    pub fn sys(&self) -> *const sys::godot_pool_int_array {
        &self.0
    }

    #[doc(hidden)]
    pub fn from_sys(sys: sys::godot_pool_int_array) -> Self {
        Int32Array(sys)
    }

    impl_common_methods! {
        /// Creates a new reference to this array.
        pub fn new_ref(& self) -> Int32Array : godot_pool_int_array_new_copy;
    }
}

impl_basic_traits!(
    for Int32Array as godot_pool_int_array {
        Drop => godot_pool_int_array_destroy;
        Default => godot_pool_int_array_new;
    }
);

impl ToVariant for Int32Array {
    fn to_variant(&self) -> Variant { Variant::from_int32_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.try_to_int32_array() }
}
