use crate::access::{Aligned, MaybeUnaligned};
use crate::private::get_api;
use crate::sys;
use crate::VariantArray;

/// A reference-counted vector of bytes that uses Godot's pool allocator.
pub struct ByteArray(pub(crate) sys::godot_pool_byte_array);

pub type Read<'a> = Aligned<ReadGuard<'a>>;
pub type Write<'a> = Aligned<WriteGuard<'a>>;

impl ByteArray {
    /// Creates an empty array.
    pub fn new() -> Self {
        ByteArray::default()
    }

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
            status != sys::godot_error_GODOT_OK
        }
    }

    /// Inverts the order of the elements in the array.
    pub fn invert(&mut self) {
        unsafe { (get_api().godot_pool_byte_array_invert)(&mut self.0) }
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
        unsafe { (get_api().godot_pool_byte_array_get)(&self.0, idx) }
    }

    /// Sets the value of the byte at the given offset.
    pub fn set(&mut self, idx: i32, byte: u8) {
        unsafe {
            (get_api().godot_pool_byte_array_set)(&mut self.0, idx, byte);
        }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> i32 {
        unsafe { (get_api().godot_pool_byte_array_size)(&self.0) }
    }

    pub fn read<'a>(&'a self) -> Read<'a> {
        unsafe {
            MaybeUnaligned::new(ReadGuard::new(self.sys()))
                .try_into_aligned()
                .expect("Pool array access should be aligned. This indicates a bug in Godot")
        }
    }

    pub fn write<'a>(&'a mut self) -> Write<'a> {
        unsafe {
            MaybeUnaligned::new(WriteGuard::new(self.sys() as *mut _))
                .try_into_aligned()
                .expect("Pool array access should be aligned. This indicates a bug in Godot")
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
        pub fn new_ref(& self) -> ByteArray : godot_pool_byte_array_new_copy;
    }
}

impl_basic_traits!(
    for ByteArray as godot_pool_byte_array {
        Drop => godot_pool_byte_array_destroy;
        Default => godot_pool_byte_array_new;
    }
);

define_access_guard! {
    pub struct ReadGuard<'a> : sys::godot_pool_byte_array_read_access {
        access = godot_pool_byte_array_read(*const sys::godot_pool_byte_array),
        len = godot_pool_byte_array_size,
    }
    Guard<Target=u8> => godot_pool_byte_array_read_access_ptr -> *const u8;
    Drop => godot_pool_byte_array_read_access_destroy;
    Clone => godot_pool_byte_array_read_access_copy;
}

define_access_guard! {
    pub struct WriteGuard<'a> : sys::godot_pool_byte_array_write_access {
        access = godot_pool_byte_array_write(*mut sys::godot_pool_byte_array),
        len = godot_pool_byte_array_size,
    }
    Guard<Target=u8> + WritePtr => godot_pool_byte_array_write_access_ptr -> *mut u8;
    Drop => godot_pool_byte_array_write_access_destroy;
}

godot_test!(
    test_byte_array_access {
        let mut arr = ByteArray::new();
        for i in 0..8 {
            arr.push(i);
        }

        let original_read = {
            let read = arr.read();
            assert_eq!(&[0, 1, 2, 3, 4, 5, 6, 7], read.as_slice());
            read.clone()
        };

        let mut cow_arr = arr.new_ref();

        {
            let mut write = cow_arr.write();
            assert_eq!(8, write.len());
            for i in write.as_mut_slice() {
                *i *= 2;
            }
        }

        for i in 0..8 {
            assert_eq!(i * 2, cow_arr.get(i as i32));
        }

        // the write shouldn't have affected the original array
        assert_eq!(&[0, 1, 2, 3, 4, 5, 6, 7], original_read.as_slice());
    }
);
