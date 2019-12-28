use crate::access::{Aligned, MaybeUnaligned};
use crate::get_api;
use crate::sys;
use crate::Color;
use crate::ToVariant;
use crate::Variant;
use crate::VariantArray;

use std::mem::transmute;

/// A reference-counted vector of `ColorArray` that uses Godot's pool allocator.
pub struct ColorArray(pub(crate) sys::godot_pool_color_array);

pub type Read<'a> = Aligned<ReadGuard<'a>>;
pub type Write<'a> = Aligned<WriteGuard<'a>>;

impl ColorArray {
    /// Creates an empty `ColorArray`.
    pub fn new() -> Self {
        ColorArray::default()
    }

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
            let status =
                (get_api().godot_pool_color_array_insert)(&mut self.0, offset, transmute(color));
            status != sys::godot_error_GODOT_OK
        }
    }

    /// Inverts the order of the elements in the array.
    pub fn invert(&mut self) {
        unsafe { (get_api().godot_pool_color_array_invert)(&mut self.0) }
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
        unsafe { transmute((get_api().godot_pool_color_array_get)(&self.0, idx)) }
    }

    /// Sets the value of the element at the given offset.
    pub fn set(&mut self, idx: i32, color: &Color) {
        unsafe {
            (get_api().godot_pool_color_array_set)(&mut self.0, idx, transmute(color));
        }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> i32 {
        unsafe { (get_api().godot_pool_color_array_size)(&self.0) }
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
    fn to_variant(&self) -> Variant {
        Variant::from_color_array(self)
    }
}

define_access_guard! {
    pub struct ReadGuard<'a> : sys::godot_pool_color_array_read_access {
        access = godot_pool_color_array_read(*const sys::godot_pool_color_array),
        len = godot_pool_color_array_size,
    }
    Guard<Target=Color> => godot_pool_color_array_read_access_ptr -> *const sys::godot_color;
    Drop => godot_pool_color_array_read_access_destroy;
    Clone => godot_pool_color_array_read_access_copy;
}

define_access_guard! {
    pub struct WriteGuard<'a> : sys::godot_pool_color_array_write_access {
        access = godot_pool_color_array_write(*mut sys::godot_pool_color_array),
        len = godot_pool_color_array_size,
    }
    Guard<Target=Color> + WritePtr => godot_pool_color_array_write_access_ptr -> *mut sys::godot_color;
    Drop => godot_pool_color_array_write_access_destroy;
}

godot_test!(
    test_color_array_access {
        let mut arr = ColorArray::new();
        arr.push(&Color::rgb(1.0, 0.0, 0.0));
        arr.push(&Color::rgb(0.0, 1.0, 0.0));
        arr.push(&Color::rgb(0.0, 0.0, 1.0));

        let original_read = {
            let read = arr.read();
            assert_eq!(&[
                Color::rgb(1.0, 0.0, 0.0),
                Color::rgb(0.0, 1.0, 0.0),
                Color::rgb(0.0, 0.0, 1.0),
            ], read.as_slice());
            read.clone()
        };

        let mut cow_arr = arr.new_ref();

        {
            let mut write = cow_arr.write();
            assert_eq!(3, write.len());
            for i in write.as_mut_slice() {
                i.b = 1.0;
            }
        }

        assert_eq!(Color::rgb(1.0, 0.0, 1.0), cow_arr.get(0));
        assert_eq!(Color::rgb(0.0, 1.0, 1.0), cow_arr.get(1));
        assert_eq!(Color::rgb(0.0, 0.0, 1.0), cow_arr.get(2));

        // the write shouldn't have affected the original array
        assert_eq!(&[
            Color::rgb(1.0, 0.0, 0.0),
            Color::rgb(0.0, 1.0, 0.0),
            Color::rgb(0.0, 0.0, 1.0),
        ], original_read.as_slice());
    }
);
