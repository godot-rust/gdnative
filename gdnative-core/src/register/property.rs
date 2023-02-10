use std::{ffi::c_void, panic::RefUnwindSafe};

use crate::core_types::Variant;

use super::convert::{as_class, as_variant_ref, FromInstancePtr, FunctionWithSite};

// TODO: struct Property register

unsafe extern "C" fn call_setter<Class, F, AsClass, Base>(
    base: *mut sys::godot_object,
    method_data: *mut c_void,
    user_data: *mut c_void,
    value: *mut sys::godot_variant,
) where
    &'static Class: AsRef<AsClass> + 'static,
    F: Fn(&AsClass, Base, &Variant) -> Result<(), Box<dyn std::error::Error>>
        + 'static
        + RefUnwindSafe,
    Base: for<'a> FromInstancePtr<'a>,
{
    let FunctionWithSite { site, function } = FunctionWithSite::<F>::as_self(&method_data);

    std::panic::catch_unwind(move || {
        crate::private::assert_main_thread();

        let user_data = as_class::<Class, AsClass>(&user_data);
        let base = Base::from_instance_ptr(user_data, &base);
        let value = as_variant_ref(&value);

        function(user_data, base, value)
    })
    .unwrap_or_else(|e| {
        crate::log::error(*site, "Setter panicked");
        crate::private::print_panic_error(e);
        Ok(())
    })
    .unwrap_or_else(|e| {
        crate::log::error(*site, format!("{e:?}"));
    });
}

unsafe extern "C" fn call_getter<Class, F, AsClass, Base>(
    base: *mut sys::godot_object,
    method_data: *mut c_void,
    user_data: *mut c_void,
) -> sys::godot_variant
where
    &'static Class: AsRef<AsClass> + 'static,
    F: Fn(&AsClass, Base) -> Result<Variant, Box<dyn std::error::Error>> + 'static + RefUnwindSafe,
    Base: for<'a> FromInstancePtr<'a>,
{
    let FunctionWithSite { site, function } = FunctionWithSite::<F>::as_self(&method_data);

    std::panic::catch_unwind(move || {
        crate::private::assert_main_thread();

        let user_data = as_class::<Class, AsClass>(&user_data);
        let base = Base::from_instance_ptr(user_data, &base);

        function(user_data, base)
    })
    .unwrap_or_else(|e| {
        crate::log::error(*site, "Getter panicked");
        crate::private::print_panic_error(e);
        Ok(Variant::nil())
    })
    .unwrap_or_else(|e| {
        crate::log::error(*site, format!("{e:?}"));
        Variant::nil()
    })
    .leak()
}
