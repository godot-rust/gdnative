use std::ptr;
use libc;
use crate::sys;
use crate::ObjectMethodTable;

/// Internal details.
pub unsafe trait GodotObject {
    fn class_name() -> &'static str;
    #[doc(hidden)]
    unsafe fn to_sys(&self) -> *mut sys::godot_object;
    #[doc(hidden)]
    unsafe fn from_sys(obj: *mut sys::godot_object) -> Self;
}

// This function assumes the godot_object is reference counted.
pub unsafe fn add_ref(obj: *mut sys::godot_object) {
    use crate::ReferenceMethodTable;
    use std::ptr;
    let api = crate::get_api();
    let addref_method = ReferenceMethodTable::unchecked_get().reference;
    let mut argument_buffer = [ptr::null() as *const libc::c_void; 0];
    let mut ok = false;
    let ok_ptr = &mut ok as *mut bool;
    (api.godot_method_bind_ptrcall)(
        addref_method,
        obj,
        argument_buffer.as_mut_ptr() as *mut _,
        ok_ptr as *mut _
    );

    // If this assertion blows up it means there is a reference counting bug
    // and we tried to increment the ref count of a dead object (who's ref
    // count is equal to zero).
    debug_assert!(ok);
}

// This function assumes the godot_object is reference counted.
pub unsafe fn unref(obj: *mut sys::godot_object) -> bool {
    use crate::ReferenceMethodTable;
    use std::ptr;
    let unref_method = ReferenceMethodTable::unchecked_get().unreference;
    let mut argument_buffer = [ptr::null() as *const libc::c_void; 0];
    let mut last_reference = false;
    let ret_ptr = &mut last_reference as *mut bool;
    (crate::get_api().godot_method_bind_ptrcall)(
        unref_method,
        obj,
        argument_buffer.as_mut_ptr() as *mut _,
        ret_ptr as *mut _
    );

    last_reference
}

// This function assumes the godot_object is reference counted.
pub unsafe fn init_ref_count(obj: *mut sys::godot_object) {
    use crate::ReferenceMethodTable;
    use std::ptr;
    let init_method = ReferenceMethodTable::unchecked_get().init_ref;
    let mut argument_buffer = [ptr::null() as *const libc::c_void; 0];
    let mut ok = false;
    let ret_ptr = &mut ok as *mut bool;
    (crate::get_api().godot_method_bind_ptrcall)(
        init_method,
        obj,
        argument_buffer.as_mut_ptr() as *mut _,
        ret_ptr as *mut _
    );

    debug_assert!(ok);
}

pub fn is_class(obj: *mut sys::godot_object, class_name: &str) -> bool {
    unsafe {
        let api = crate::get_api();
        let method_bind = ObjectMethodTable::get(api).is_class;

        let mut class_name = (api.godot_string_chars_to_utf8_with_len)(
            class_name.as_ptr() as *const _,
            class_name.len() as _
        );

        let mut argument_buffer = [ptr::null() as *const libc::c_void; 1];
        argument_buffer[0] = (&class_name) as *const _ as *const _;

        let mut ret = false;
        let ret_ptr = &mut ret as *mut _;
        (api.godot_method_bind_ptrcall)(
            method_bind,
            obj,
            argument_buffer.as_mut_ptr() as *mut _,
            ret_ptr as *mut _
        );

        (api.godot_string_destroy)(&mut class_name);

        ret
    }
}

pub fn godot_cast<T>(from: *mut sys::godot_object) -> Option<T>
where
    T: GodotObject,
{
    unsafe {
        if !is_class(from, T::class_name()) {
            return None;
        }

        Some(T::from_sys(from))
    }
}
