use sys;
use get_api;
use Variant;
use GodotType;

/// An array of `Variant`. Godot's generic array data type.
/// Negative indices can be used to count from the right.
pub struct VariantArray(pub(crate) sys::godot_array);

impl VariantArray {
    /// Creates an empty `VariantArray`.
    pub fn new() -> Self { VariantArray::default() }

    /// Sets the value of the element at the given offset.
    pub fn set(&mut self, idx: i32, val: &Variant) {
        unsafe {
            (get_api().godot_array_set)(&mut self.0, idx, &val.0)
        }
    }

    /// Returns a copy of the element at the given offset.
    pub fn get_val(&mut self, idx: i32) -> Variant {
        unsafe {
            Variant((get_api().godot_array_get)(&self.0, idx))
        }
    }

    /// Returns a reference to the element at the given offset.
    pub fn get_ref(&self, idx: i32) -> &Variant {
        unsafe {
            Variant::cast_ref(
                (get_api().godot_array_operator_index_const)(&self.0, idx)
            )
        }
    }

    /// Returns a mutable reference to the element at the given offset.
    pub fn get_mut_ref(&mut self, idx: i32) -> &mut Variant {
        unsafe {
            Variant::cast_mut_ref((get_api().godot_array_operator_index)(&mut self.0, idx))
        }
    }

    pub fn count(&mut self, val: &Variant) -> i32 {
        unsafe {
            (get_api().godot_array_count)(&mut self.0, &val.0)
        }
    }

    /// Clears the array, resizing to 0.
    pub fn clear(&mut self) {
        unsafe {
            (get_api().godot_array_clear)(&mut self.0);
        }
    }

    pub fn remove(&mut self, idx: i32) {
        unsafe {
            (get_api().godot_array_remove)(&mut self.0, idx)
        }
    }

    pub fn erase(&mut self, val: &Variant) {
        unsafe {
            (get_api().godot_array_erase)(&mut self.0, &val.0)
        }
    }

    /// Returns `true` if the `VariantArray` contains no elements.
    pub fn is_empty(&self) -> bool {
        unsafe {
            (get_api().godot_array_empty)(&self.0)
        }
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> i32 {
        unsafe {
            (get_api().godot_array_size)(&self.0)
        }
    }

    /// Appends an element at the end of the array.
    pub fn push(&mut self, val: &Variant) {
        unsafe {
            (get_api().godot_array_push_back)(&mut self.0, &val.0);
        }
    }

    /// Removes an element at the end of the array.
    pub fn pop(&mut self) -> Variant {
        unsafe {
            Variant((get_api().godot_array_pop_back)(&mut   self.0))
        }
    }

    /// Appends an element to the front of the array.
    pub fn push_front(&mut self, val: &Variant) {
        unsafe {
            (get_api().godot_array_push_front)(&mut self.0, &val.0);
        }
    }

    /// Removes an element at the front of the array.
    pub fn pop_front(&mut self) -> Variant {
        unsafe {
            Variant((get_api().godot_array_pop_front)(&mut self.0))
        }
    }

    /// Insert a new int at a given position in the array.
    pub fn insert(&mut self, at: i32, val: &Variant) {
        unsafe {
            (get_api().godot_array_insert)(&mut self.0, at, &val.0)
        }
    }

    /// Searches the array for a value and returns its index.
    /// Pass an initial search index as the second argument.
    /// Returns `-1` if value is not found.
    pub fn find(&self, what: &Variant, from: i32) -> i32 {
        unsafe {
            (get_api().godot_array_find)(&self.0, &what.0, from)
        }
    }

    /// Returns true if the `VariantArray` contains the specified value.
    pub fn contains(&self, what: &Variant) -> bool {
        unsafe {
            (get_api().godot_array_has)(&self.0, &what.0)
        }
    }

    pub fn resize(&mut self, size: i32) {
        unsafe {
            (get_api().godot_array_resize)(&mut self.0, size)
        }
    }

    /// Searches the array in reverse order.
    /// Pass an initial search index as the second argument.
    /// If negative, the start index is considered relative to the end of the array.
    pub fn rfind(&self, what: &Variant, from: i32) -> i32 {
        unsafe {
            (get_api().godot_array_rfind)(&self.0, &what.0, from)
        }
    }

    /// Searches the array in reverse order for a value.
    /// Returns its index or `-1` if not found.
    pub fn find_last(&self, what: &Variant) -> i32 {
        unsafe {
            (get_api().godot_array_find_last)(&self.0, &what.0)
        }
    }

    /// Inverts the order of the elements in the array.
    pub fn invert(&mut self) {
        unsafe {
            (get_api().godot_array_invert)(&mut self.0)
        }
    }

    /// Return a hashed i32 value representing the array contents.
    pub fn hash(&self) -> i32 {
        unsafe {
            (get_api().godot_array_hash)(&self.0)
        }
    }

    pub fn sort(&mut self) {
        unsafe {
            (get_api().godot_array_sort)(&mut self.0)
        }
    }

    // TODO
    // pub fn sort_custom(&mut self, obj: ?, s: ?) {
    //     unimplemented!()
    // }

    // pub fn bsearch(&mut self, val: (), before: bool) -> i32 {
    //     unsafe {
    //         (get_api().godot_array_bsearch)(&mut self.0, val, before)
    //     }
    // }

    // pub fn bsearch_custom(&mut self, val: ?, obj: ?, s: ?, before: bool) -> i32 {
    //     unimplemented!();
    // }
}

impl_basic_traits!(
    for VariantArray as godot_array {
        Drop => godot_array_destroy;
        Clone => godot_array_new_copy;
        Default => godot_array_new;
    }
);

impl GodotType for VariantArray {
    fn to_variant(&self) -> Variant { Variant::from_array(self) }
    fn from_variant(variant: &Variant) -> Option<Self> { variant.try_to_array() }
}
