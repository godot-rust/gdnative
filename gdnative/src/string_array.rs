use sys;
use get_api;
use Variant;
use GodotType;
use VariantArray;
use GodotString;

/// A vector of `GodotString` that uses Godot's pool allocator.
pub struct StringArray(pub(crate) sys::godot_pool_string_array);

impl StringArray {
    /// Creates an empty `StringArray`.
    pub fn new() -> Self { StringArray::default() }

    /// Creates an array by trying to convert each variant.
    ///
    /// See `Variant::to_string_array`.
    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut result = sys::godot_pool_string_array::default();
            (get_api().godot_pool_string_array_new_with_array)(&mut result, &array.0);
            StringArray(result)
        }
    }

    /// Appends an element at the end of the array.
    pub fn push(&mut self, s: &GodotString) {
        unsafe {
            (get_api().godot_pool_string_array_append)(&mut self.0, &s.0);
        }
    }

    /// Appends a `StringArray` at the end of this array.
    pub fn push_string_array(&mut self, strings: &StringArray) {
        unsafe {
            (get_api().godot_pool_string_array_append_array)(&mut self.0, &strings.0);
        }
    }

    // TODO(error handling)
    /// Insert a new `GodotString` at a given position in the array.
    pub fn insert(&mut self, offset: i32, string: &GodotString) -> bool {
        unsafe {
            let status = (get_api().godot_pool_string_array_insert)(&mut self.0, offset, &string.0);
            status != sys::godot_error::GODOT_OK
        }
    }

    /// Inverts the order of the elements in the array.
    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_pool_string_array_invert)(&mut self.0)
        }
    }

    /// Removes an element at the given offset.
    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_pool_string_array_remove)(&mut self.0, idx);
        }
    }

    /// Changes the size of the array, possibly removing elements or pushing default values.
    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_pool_string_array_resize)(&mut self.0, size);
        }
    }

    /// Returns a copy of the element at the given offset.
    pub fn get(&self, idx: i32) -> GodotString {
        unsafe {
            GodotString((get_api().godot_pool_string_array_get)(&self.0, idx))
        }
    }

    /// Sets the value of the element at the given offset.
    pub fn set(&mut self, idx: i32, string: &GodotString) {
        unsafe {
            (get_api().godot_pool_string_array_set)(&mut self.0, idx, &string.0);
        }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_pool_string_array_size)(&self.0)
        }
    }

    impl_common_methods! {
        /// Creates a new reference to this array.
        pub fn new_ref(&self) -> StringArray : godot_pool_string_array_new_copy;
    }
}

impl_basic_traits!(
    for StringArray as godot_pool_string_array {
        Drop => godot_pool_string_array_destroy;
        Default => godot_pool_string_array_new;
    }
);

impl GodotType for StringArray {
    fn to_variant(&self) -> Variant { Variant::from_string_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.try_to_string_array() }
}
