use crate::access::{Aligned, MaybeUnaligned};
use crate::private::get_api;
use crate::sys;
use crate::VariantArray;
use crate::Vector2;

use std::fmt;
use std::mem::transmute;

/// A reference-counted vector of `Vector2` that uses Godot's pool allocator.
pub struct Vector2Array(pub(crate) sys::godot_pool_vector2_array);

pub type Read<'a> = Aligned<ReadGuard<'a>>;
pub type Write<'a> = Aligned<WriteGuard<'a>>;

impl Vector2Array {
    /// Creates an empty array.
    pub fn new() -> Self {
        Vector2Array::default()
    }

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

    /// Appends a vector to the end of the array.
    pub fn push(&mut self, vector: &Vector2) {
        unsafe {
            (get_api().godot_pool_vector2_array_append)(&mut self.0, transmute(vector));
        }
    }

    /// Appends each vector to the end of the array.
    pub fn push_array(&mut self, vectors: &Vector2Array) {
        unsafe {
            (get_api().godot_pool_vector2_array_append_array)(&mut self.0, transmute(vectors));
        }
    }

    // TODO(error handling)
    /// Inserts a vector at the given offset.
    pub fn insert(&mut self, offset: i32, vector: &Vector2) -> bool {
        unsafe {
            let status =
                (get_api().godot_pool_vector2_array_insert)(&mut self.0, offset, transmute(vector));
            status != sys::godot_error_GODOT_OK
        }
    }

    /// Inverts the order of the elements in the array.
    pub fn invert(&mut self) {
        unsafe { (get_api().godot_pool_vector2_array_invert)(&mut self.0) }
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

    /// Returns a copy of the element at the given offset.
    pub fn get(&self, idx: i32) -> Vector2 {
        unsafe { transmute((get_api().godot_pool_vector2_array_get)(&self.0, idx)) }
    }

    /// Sets the value of the element at the given offset.
    pub fn set(&mut self, idx: i32, vector: &Vector2) {
        unsafe {
            (get_api().godot_pool_vector2_array_set)(&mut self.0, idx, transmute(vector));
        }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> i32 {
        unsafe { (get_api().godot_pool_vector2_array_size)(&self.0) }
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
    pub fn sys(&self) -> *const sys::godot_pool_vector2_array {
        &self.0
    }

    #[doc(hidden)]
    pub fn from_sys(sys: sys::godot_pool_vector2_array) -> Self {
        Vector2Array(sys)
    }

    impl_common_methods! {
        pub fn new_ref(&self) -> Vector2Array : godot_pool_vector2_array_new_copy;
    }
}

impl fmt::Debug for Vector2Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.read().iter()).finish()
    }
}

impl_basic_traits!(
    for Vector2Array as godot_pool_vector2_array {
        Drop => godot_pool_vector2_array_destroy;
        Default => godot_pool_vector2_array_new;
    }
);

define_access_guard! {
    pub struct ReadGuard<'a> : sys::godot_pool_vector2_array_read_access {
        access = godot_pool_vector2_array_read(*const sys::godot_pool_vector2_array),
        len = godot_pool_vector2_array_size,
    }
    Guard<Target=Vector2> => godot_pool_vector2_array_read_access_ptr -> *const sys::godot_vector2;
    Drop => godot_pool_vector2_array_read_access_destroy;
    Clone => godot_pool_vector2_array_read_access_copy;
}

define_access_guard! {
    pub struct WriteGuard<'a> : sys::godot_pool_vector2_array_write_access {
        access = godot_pool_vector2_array_write(*mut sys::godot_pool_vector2_array),
        len = godot_pool_vector2_array_size,
    }
    Guard<Target=Vector2> + WritePtr => godot_pool_vector2_array_write_access_ptr -> *mut sys::godot_vector2;
    Drop => godot_pool_vector2_array_write_access_destroy;
}

godot_test!(
    test_vector2_array_access {
        let mut arr = Vector2Array::new();
        arr.push(&Vector2::new(1.0, 2.0));
        arr.push(&Vector2::new(3.0, 4.0));
        arr.push(&Vector2::new(5.0, 6.0));

        let original_read = {
            let read = arr.read();
            assert_eq!(&[
                Vector2::new(1.0, 2.0),
                Vector2::new(3.0, 4.0),
                Vector2::new(5.0, 6.0),
            ], read.as_slice());
            read.clone()
        };

        let mut cow_arr = arr.new_ref();

        {
            let mut write = cow_arr.write();
            assert_eq!(3, write.len());
            for s in write.as_mut_slice() {
                s.x += 1.0;
            }
        }

        assert_eq!(Vector2::new(2.0, 2.0), cow_arr.get(0));
        assert_eq!(Vector2::new(4.0, 4.0), cow_arr.get(1));
        assert_eq!(Vector2::new(6.0, 6.0), cow_arr.get(2));

        // the write shouldn't have affected the original array
        assert_eq!(&[
            Vector2::new(1.0, 2.0),
            Vector2::new(3.0, 4.0),
            Vector2::new(5.0, 6.0),
        ], original_read.as_slice());
    }
);

godot_test!(
    test_vector2_array_debug {
        let mut arr = Vector2Array::new();
        arr.push(&Vector2::new(1.0, 2.0));
        arr.push(&Vector2::new(3.0, 4.0));
        arr.push(&Vector2::new(5.0, 6.0));

        assert_eq!(format!("{:?}", arr), "[(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)]");
    }
);
