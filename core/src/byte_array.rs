use sys;
use get_api;
use Variant;
use ToVariant;
use VariantArray;

/// A reference-counted vector of bytes that uses Godot's pool allocator.
pub struct ByteArray(pub(crate) sys::godot_pool_byte_array);

impl ByteArray {
    /// Creates an empty array.
    pub fn new() -> Self { ByteArray::default() }

    /// Creates an array by trying to convert each variant.
    ///
    /// When no viable conversion exists, the default value `0` is pushed.
    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut result = sys::godot_pool_byte_array::default();
            (get_api().godot_pool_byte_array_new_with_array)(&mut result, &array.0);
            ByteArray(result)
        }
    }

    /// Appends a byte to the end of the array.
    pub fn push(&mut self, byte: u8) {
        unsafe {
            (get_api().godot_pool_byte_array_append)(&mut self.0, byte);
        }
    }

    /// Appends each byte to the end of the array.
    pub fn push_array(&mut self, bytes: &ByteArray) {
        unsafe {
            (get_api().godot_pool_byte_array_append_array)(&mut self.0, &bytes.0);
        }
    }

    // TODO(error handling)
    /// Inserts a byte at the given offset.
    pub fn insert(&mut self, offset: i32, byte: u8) -> bool {
        unsafe {
            let status = (get_api().godot_pool_byte_array_insert)(&mut self.0, offset, byte);
            status != sys::godot_error::GODOT_OK
        }
    }

    /// Inverts the order of the elements in the array.
    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_pool_byte_array_invert)(&mut self.0)
        }
    }

    /// Removes an element at the given offset.
    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_pool_byte_array_remove)(&mut self.0, idx);
        }
    }

    /// Changes the size of the array, possibly removing elements or pushing default values.
    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_pool_byte_array_resize)(&mut self.0, size);
        }
    }

    /// Returns a copy of the byte at the given offset.
    pub fn get(&self, idx: i32) -> u8 {
        unsafe {
            (get_api().godot_pool_byte_array_get)(&self.0, idx)
        }
    }

    /// Sets the value of the byte at the given offset.
    pub fn set(&mut self, idx: i32, byte: u8) {
        unsafe {
            (get_api().godot_pool_byte_array_set)(&mut self.0, idx, byte);
        }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_pool_byte_array_size)(&self.0)
        }
    }

    #[doc(hidden)]
    pub fn sys(&self) -> *const sys::godot_pool_byte_array {
        &self.0
    }

    #[doc(hidden)]
    pub fn from_sys(sys: sys::godot_pool_byte_array) -> Self {
        ByteArray(sys)
    }

    impl_common_methods! {
        /// Creates a new reference to this array.
        pub fn new_ref(& self) -> ByteArray : godot_pool_byte_array_new_copy;
    }
}

impl_basic_traits!(
    for ByteArray as godot_pool_byte_array {
        Drop => godot_pool_byte_array_destroy;
        Default => godot_pool_byte_array_new;
    }
);

impl ToVariant for ByteArray {
    fn to_variant(&self) -> Variant { Variant::from_byte_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.try_to_byte_array() }
}
