use crate::access::{Aligned, MaybeUnaligned};
use crate::private::get_api;
use crate::sys;
use crate::VariantArray;

use std::fmt;

/// A reference-counted vector of `f32` that uses Godot's pool allocator.
pub struct Float32Array(pub(crate) sys::godot_pool_real_array);

pub type Read<'a> = Aligned<ReadGuard<'a>>;
pub type Write<'a> = Aligned<WriteGuard<'a>>;

impl Float32Array {
    /// Creates an empty `Float32Array`.
    pub fn new() -> Self {
        Float32Array::default()
    }

    /// Creates an array by trying to convert each variant.
    ///
    /// See `Variant::to_float32_array`.
    pub fn from_variant_array(array: &VariantArray) -> Self {
        unsafe {
            let mut result = sys::godot_pool_real_array::default();
            (get_api().godot_pool_real_array_new_with_array)(&mut result, &array.0);
            Float32Array(result)
        }
    }

    /// Appends an element at the end of the array.
    pub fn push(&mut self, val: f32) {
        unsafe {
            (get_api().godot_pool_real_array_append)(&mut self.0, val);
        }
    }

    /// Appends a `Float32Array` at the end of this array.
    pub fn push_array(&mut self, array: &Float32Array) {
        unsafe {
            (get_api().godot_pool_real_array_append_array)(&mut self.0, &array.0);
        }
    }

    // TODO(error handling)
    /// Insert a new f32 at a given position in the array.
    pub fn insert(&mut self, offset: i32, val: f32) -> bool {
        unsafe {
            let status = (get_api().godot_pool_real_array_insert)(&mut self.0, offset, val);
            status != sys::godot_error_GODOT_OK
        }
    }

    /// Inverts the order of the elements in the array.
    pub fn invert(&mut self) {
        unsafe { (get_api().godot_pool_real_array_invert)(&mut self.0) }
    }

    /// Removes an element at the given offset.
    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_pool_real_array_remove)(&mut self.0, idx);
        }
    }

    /// Changes the size of the array, possibly removing elements or pushing default values.
    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_pool_real_array_resize)(&mut self.0, size);
        }
    }

    /// Returns a copy of the element at the given offset.
    pub fn get(&self, idx: i32) -> f32 {
        unsafe { (get_api().godot_pool_real_array_get)(&self.0, idx) }
    }

    /// Sets the value of the element at the given offset.
    pub fn set(&mut self, idx: i32, val: f32) {
        unsafe {
            (get_api().godot_pool_real_array_set)(&mut self.0, idx, val);
        }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> i32 {
        unsafe { (get_api().godot_pool_real_array_size)(&self.0) }
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
    pub fn sys(&self) -> *const sys::godot_pool_real_array {
        &self.0
    }

    #[doc(hidden)]
    pub fn from_sys(sys: sys::godot_pool_real_array) -> Self {
        Float32Array(sys)
    }

    impl_common_methods! {
        pub fn new_ref(&self) -> Float32Array : godot_pool_real_array_new_copy;
    }
}

impl fmt::Debug for Float32Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.read().iter()).finish()
    }
}

impl_basic_traits!(
    for Float32Array as godot_pool_real_array {
        Drop => godot_pool_real_array_destroy;
        Default => godot_pool_real_array_new;
    }
);

define_access_guard! {
    pub struct ReadGuard<'a> : sys::godot_pool_real_array_read_access {
        access = godot_pool_real_array_read(*const sys::godot_pool_real_array),
        len = godot_pool_real_array_size,
    }
    Guard<Target=f32> => godot_pool_real_array_read_access_ptr -> *const f32;
    Drop => godot_pool_real_array_read_access_destroy;
    Clone => godot_pool_real_array_read_access_copy;
}

define_access_guard! {
    pub struct WriteGuard<'a> : sys::godot_pool_real_array_write_access {
        access = godot_pool_real_array_write(*mut sys::godot_pool_real_array),
        len = godot_pool_real_array_size,
    }
    Guard<Target=f32> + WritePtr => godot_pool_real_array_write_access_ptr -> *mut f32;
    Drop => godot_pool_real_array_write_access_destroy;
}

godot_test!(
    test_float32_array_access {
        let mut arr = Float32Array::new();
        for i in 0..8 {
            arr.push(i as f32);
        }

        let original_read = {
            let read = arr.read();
            assert_eq!(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0], read.as_slice());
            read.clone()
        };

        let mut cow_arr = arr.new_ref();

        {
            let mut write = cow_arr.write();
            assert_eq!(8, write.len());
            for i in write.as_mut_slice() {
                *i *= 2.0;
            }
        }

        for i in 0..8 {
            assert_eq!(i as f32 * 2.0, cow_arr.get(i as i32));
        }

        // the write shouldn't have affected the original array
        assert_eq!(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0], original_read.as_slice());
    }
);

godot_test!(
    test_float32_array_debug {
        let mut arr = Float32Array::new();
        for i in 0..8 {
            arr.push(i as f32);
        }

        assert_eq!(format!("{:?}", arr), "[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]");
    }
);
