use crate::access::{Aligned, MaybeUnaligned};
use crate::get_api;
use crate::sys;
use crate::GodotString;
use crate::VariantArray;

/// A vector of `GodotString` that uses Godot's pool allocator.
pub struct StringArray(pub(crate) sys::godot_pool_string_array);

pub type Read<'a> = Aligned<ReadGuard<'a>>;
pub type Write<'a> = Aligned<WriteGuard<'a>>;

impl StringArray {
    /// Creates an empty `StringArray`.
    pub fn new() -> Self {
        StringArray::default()
    }

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
            status != sys::godot_error_GODOT_OK
        }
    }

    /// Inverts the order of the elements in the array.
    pub fn invert(&mut self) {
        unsafe { (get_api().godot_pool_string_array_invert)(&mut self.0) }
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
        unsafe { GodotString((get_api().godot_pool_string_array_get)(&self.0, idx)) }
    }

    /// Sets the value of the element at the given offset.
    pub fn set(&mut self, idx: i32, string: &GodotString) {
        unsafe {
            (get_api().godot_pool_string_array_set)(&mut self.0, idx, &string.0);
        }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> i32 {
        unsafe { (get_api().godot_pool_string_array_size)(&self.0) }
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
    pub fn sys(&self) -> *const sys::godot_pool_string_array {
        &self.0
    }

    #[doc(hidden)]
    pub fn from_sys(sys: sys::godot_pool_string_array) -> Self {
        StringArray(sys)
    }

    impl_common_methods! {
        pub fn new_ref(&self) -> StringArray : godot_pool_string_array_new_copy;
    }
}

impl_basic_traits!(
    for StringArray as godot_pool_string_array {
        Drop => godot_pool_string_array_destroy;
        Default => godot_pool_string_array_new;
    }
);

define_access_guard! {
    pub struct ReadGuard<'a> : sys::godot_pool_string_array_read_access {
        access = godot_pool_string_array_read(*const sys::godot_pool_string_array),
        len = godot_pool_string_array_size,
    }
    Guard<Target=GodotString> => godot_pool_string_array_read_access_ptr -> *const sys::godot_string;
    Drop => godot_pool_string_array_read_access_destroy;
    Clone => godot_pool_string_array_read_access_copy;
}

define_access_guard! {
    pub struct WriteGuard<'a> : sys::godot_pool_string_array_write_access {
        access = godot_pool_string_array_write(*mut sys::godot_pool_string_array),
        len = godot_pool_string_array_size,
    }
    Guard<Target=GodotString> + WritePtr => godot_pool_string_array_write_access_ptr -> *mut sys::godot_string;
    Drop => godot_pool_string_array_write_access_destroy;
}

godot_test!(
    test_string_array_access {
        let mut arr = StringArray::new();
        arr.push(&GodotString::from("foo"));
        arr.push(&GodotString::from("bar"));
        arr.push(&GodotString::from("baz"));

        let original_read = {
            let read = arr.read();
            assert_eq!(&[
                GodotString::from("foo"),
                GodotString::from("bar"),
                GodotString::from("baz"),
            ], read.as_slice());
            read.clone()
        };

        let mut cow_arr = arr.new_ref();

        {
            let mut write = cow_arr.write();
            assert_eq!(3, write.len());
            for s in write.as_mut_slice() {
                *s = s.to_uppercase();
            }
        }

        assert_eq!(GodotString::from("FOO"), cow_arr.get(0));
        assert_eq!(GodotString::from("BAR"), cow_arr.get(1));
        assert_eq!(GodotString::from("BAZ"), cow_arr.get(2));

        // the write shouldn't have affected the original array
        assert_eq!(&[
            GodotString::from("foo"),
            GodotString::from("bar"),
            GodotString::from("baz"),
        ], original_read.as_slice());
    }
);
