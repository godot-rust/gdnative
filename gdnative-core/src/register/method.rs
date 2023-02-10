use std::{
    ffi::{c_int, c_void},
    panic::RefUnwindSafe,
};

use crate::core_types::Variant;

use super::convert::{as_class, as_variant_args, FromInstancePtr, FunctionWithSite};

// TODO: struct Method register

unsafe extern "C" fn call_method<Class, F, AsClass, Base>(
    base: *mut sys::godot_object,
    method_data: *mut c_void,
    user_data: *mut c_void,
    num_args: c_int,
    args: *mut *mut sys::godot_variant,
) -> sys::godot_variant
where
    &'static Class: AsRef<AsClass> + 'static,
    F: Fn(&AsClass, Base, &[&Variant]) -> Result<Variant, Box<dyn std::error::Error>>
        + 'static
        + RefUnwindSafe,
    Base: for<'a> FromInstancePtr<'a>,
{
    let FunctionWithSite { site, function } = FunctionWithSite::<F>::as_self(&method_data);

    std::panic::catch_unwind(move || {
        crate::private::assert_main_thread();

        let user_data = as_class::<Class, AsClass>(&user_data);
        let base = Base::from_instance_ptr(user_data, &base);
        let args = as_variant_args(&args, num_args);

        function(user_data, base, args)
    })
    .unwrap_or_else(|e| {
        crate::log::error(*site, "Method panicked");
        crate::private::print_panic_error(e);
        Ok(Variant::nil())
    })
    .unwrap_or_else(|e| {
        crate::log::error(*site, format!("{e:?}"));
        Variant::nil()
    })
    .leak()
}
