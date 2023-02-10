use std::{ffi::c_void, panic::RefUnwindSafe};

use super::convert::{alloc_data, unalloc_data, FromBasePtr, FunctionWithSite};

// TODO: struct ClassHandle
// TODO: struct Class register

unsafe extern "C" fn call_create<F, Class, Base>(
    base: *mut sys::godot_object,
    method_data: *mut c_void,
) -> *mut c_void
where
    F: Fn(Base) -> Class + 'static + RefUnwindSafe,
    Class: 'static,
    Base: for<'a> FromBasePtr<'a>,
{
    let FunctionWithSite { site, function } = FunctionWithSite::<F>::as_self(&method_data);

    std::panic::catch_unwind(move || {
        crate::private::assert_main_thread();

        let base = Base::from_base_ptr(&base);

        // TODO: emplace create
        let user_data = function(base);
        alloc_data(user_data)
    })
    .unwrap_or_else(|e| {
        crate::log::error(*site, "Class create panicked");
        crate::private::print_panic_error(e);
        std::ptr::null_mut()
    })
}

unsafe extern "C" fn call_destroy<F, Class, Base>(
    base: *mut sys::godot_object,
    method_data: *mut c_void,
    user_data: *mut c_void,
) where
    F: Fn(Class, Base) + 'static + RefUnwindSafe,
    Class: 'static,
    Base: for<'a> FromBasePtr<'a>,
{
    let FunctionWithSite { site, function } = FunctionWithSite::<F>::as_self(&method_data);

    std::panic::catch_unwind(move || {
        if user_data.is_null() {
            return; // Nothing to destroy has been created
        }

        crate::private::assert_main_thread();

        let user_data = unalloc_data::<Class>(user_data);
        let base = Base::from_base_ptr(&base);

        function(user_data, base);
    })
    .unwrap_or_else(|e| {
        crate::log::error(*site, "Class create panicked");
        crate::private::print_panic_error(e);
    });
}
