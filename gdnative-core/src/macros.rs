#![macro_use]

/// Declare the API endpoint to initialize the gdnative API on startup.
///
/// By default this declares an extern function named `godot_gdnative_init`.
/// This can be overridden, for example:
///
/// ```ignore
/// // Declares an extern function named custom_gdnative_init instead of
/// // godot_gdnative_init.
/// godot_gdnative_init!(my_init_callback as custom_gdnative_init);
/// ```
///
/// Overriding the default entry point names can be useful if several gdnative
/// libraries are linked statically  to avoid name clashes.
#[macro_export]
macro_rules! godot_gdnative_init {
    () => {
        fn godot_gdnative_init_empty(_options: *mut $crate::sys::godot_gdnative_init_options) {}
        $crate::godot_gdnative_init!(godot_gdnative_init_empty);
    };
    (_ as $fn_name:ident) => {
        fn godot_gdnative_init_empty(_options: *mut $crate::sys::godot_gdnative_init_options) {}
        $crate::godot_gdnative_init!(godot_gdnative_init_empty as $fn_name);
    };
    ($callback:ident) => {
        $crate::godot_gdnative_init!($callback as godot_gdnative_init);
    };
    ($callback:ident as $fn_name:ident) => {
        #[no_mangle]
        #[doc(hidden)]
        #[allow(unused_unsafe)]
        pub unsafe extern "C" fn $fn_name(options: *mut $crate::sys::godot_gdnative_init_options) {
            if !$crate::private::bind_api(options) {
                // Can't use godot_error here because the API is not bound.
                // Init errors should be reported by bind_api.
                return;
            }

            let __result = ::std::panic::catch_unwind(|| $callback(options));
            if __result.is_err() {
                $crate::godot_error!("gdnative-core: gdnative_init callback panicked");
            }
        }
    };
}

/// Declare the API endpoint invoked during shutdown.
///
/// By default this declares an extern function named `godot_gdnative_terminate`.
/// This can be overridden, for example:
///
/// ```ignore
/// // Declares an extern function named custom_gdnative_terminate instead of
/// // godot_gdnative_terminate.
/// godot_gdnative_terminate!(my_shutdown_callback as custom_gdnative_terminate);
/// ```
///
/// Overriding the default entry point names can be useful if several gdnative
/// libraries are linked statically  to avoid name clashes.
#[macro_export]
macro_rules! godot_gdnative_terminate {
    () => {
        fn godot_gdnative_terminate_empty(
            _options: *mut $crate::sys::godot_gdnative_terminate_options,
        ) {
        }
        $crate::godot_gdnative_terminate!(godot_gdnative_terminate_empty);
    };
    ($callback:ident) => {
        $crate::godot_gdnative_terminate!($callback as godot_gdnative_terminate);
    };
    (_ as $fn_name:ident) => {
        fn godot_gdnative_terminate_empty(
            _options: *mut $crate::sys::godot_gdnative_terminate_options,
        ) {
        }
        $crate::godot_gdnative_terminate!(godot_gdnative_terminate_empty as $fn_name);
    };
    ($callback:ident as $fn_name:ident) => {
        #[no_mangle]
        #[doc(hidden)]
        #[allow(unused_unsafe)]
        pub unsafe extern "C" fn $fn_name(
            options: *mut $crate::sys::godot_gdnative_terminate_options,
        ) {
            if !$crate::private::is_api_bound() {
                return;
            }

            let __result = ::std::panic::catch_unwind(|| $callback(options));
            if __result.is_err() {
                $crate::godot_error!("gdnative-core: nativescript_init callback panicked");
            }

            $crate::private::cleanup_internal_state();
        }
    };
}

/// Declare the API endpoint to initialize nativescript classes on startup.
///
/// By default this declares an extern function named `godot_nativescript_init`.
/// This can be overridden, for example:
///
/// ```ignore
/// // Declares an extern function named custom_nativescript_init instead of
/// // godot_nativescript_init.
/// godot_gdnative_terminate!(my_registration_callback as custom_nativescript_init);
/// ```
///
/// Overriding the default entry point names can be useful if several gdnative
/// libraries are linked statically  to avoid name clashes.
#[macro_export]
macro_rules! godot_nativescript_init {
    () => {
        fn godot_nativescript_init_empty(_init: $crate::init::InitHandle) {}
        $crate::godot_nativescript_init!(godot_nativescript_init_empty);
    };
    ($callback:ident) => {
        $crate::godot_nativescript_init!($callback as godot_nativescript_init);
    };
    (_ as $fn_name:ident) => {
        fn godot_nativescript_init_empty(_init: $crate::init::InitHandle) {}
        $crate::godot_nativescript_init!(godot_nativescript_init_empty as $fn_name);
    };
    ($callback:ident as $fn_name:ident) => {
        #[no_mangle]
        #[doc(hidden)]
        #[allow(unused_unsafe)]
        pub unsafe extern "C" fn $fn_name(handle: *mut $crate::libc::c_void) {
            if !$crate::private::is_api_bound() {
                return;
            }

            let __result = ::std::panic::catch_unwind(|| {
                $callback($crate::init::InitHandle::new(handle));
            });

            if __result.is_err() {
                $crate::godot_error!("gdnative-core: nativescript_init callback panicked");
            }
        }
    };
}

/// Print a message using the engine's logging system (visible in the editor).
#[macro_export]
macro_rules! godot_print {
    ($($args:tt)*) => ({
        let msg = format!($($args)*);

        #[allow(unused_unsafe)]
        unsafe {
            let msg = $crate::GodotString::from_str(msg);
            ($crate::private::get_api().godot_print)(&msg.to_sys() as *const _);
        }
    });
}

/// Prints and returns the value of a given expression for quick and dirty debugging,
/// using the engine's logging system (visible in the editor).
///
/// This behaves similarly to the `std::dbg!` macro.
#[macro_export]
macro_rules! godot_dbg {
    () => {
        $crate::godot_print!("[{}:{}]", ::std::file!(), ::std::line!());
    };
    ($val:expr) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::godot_print!("[{}:{}] {} = {:#?}",
                    ::std::file!(), ::std::line!(), ::std::stringify!($val), &tmp);
                tmp
            }
        }
    };
    // Trailing comma with single argument is ignored
    ($val:expr,) => { $crate::godot_dbg!($val) };
    ($($val:expr),+ $(,)?) => {
        ($($crate::godot_dbg!($val)),+,)
    };
}

/// Print a warning using the engine's logging system (visible in the editor).
///
/// # Guarantees
///
/// It's guaranteed that the expansion result of this macro may *only* panic if:
///
/// - Any of the arguments for the message panicked in `fmt`.
/// - The formatted message contains the NUL byte (`\0`) anywhere.
#[macro_export]
macro_rules! godot_warn {
    ($($args:tt)*) => ({
        let msg = format!($($args)*);
        let line = line!();
        let file = file!();
        #[allow(unused_unsafe)]
        unsafe {
            let msg = ::std::ffi::CString::new(msg).unwrap();
            let file = ::std::ffi::CString::new(file).unwrap();
            let func = b"<native>\0";
            ($crate::private::get_api().godot_print_warning)(
                msg.as_ptr() as *const _,
                func.as_ptr() as *const _,
                file.as_ptr() as *const _,
                line as _,
            );
        }
    })
}

/// Print an error using the engine's logging system (visible in the editor).
///
/// # Guarantees
///
/// It's guaranteed that the expansion result of this macro may *only* panic if:
///
/// - Any of the arguments for the message panicked in `fmt`.
/// - The formatted message contains the NUL byte (`\0`) anywhere.
#[macro_export]
macro_rules! godot_error {
    ($($args:tt)*) => ({
        let msg = format!($($args)*);
        let line = line!();
        let file = file!();
        #[allow(unused_unsafe)]
        unsafe {
            let msg = ::std::ffi::CString::new(msg).unwrap();
            let file = ::std::ffi::CString::new(file).unwrap();
            let func = b"<native>\0";
            ($crate::private::get_api().godot_print_error)(
                msg.as_ptr() as *const _,
                func.as_ptr() as *const _,
                file.as_ptr() as *const _,
                line as _,
            );
        }
    })
}

macro_rules! impl_basic_trait_as_sys {
    (
        Drop for $Type:ty as $GdType:ident : $gd_method:ident
    ) => {
        impl Drop for $Type {
            #[inline]
            fn drop(&mut self) {
                unsafe { (get_api().$gd_method)(self.sys_mut()) }
            }
        }
    };

    (
        Clone for $Type:ty as $GdType:ident : $gd_method:ident
    ) => {
        impl Clone for $Type {
            #[inline]
            fn clone(&self) -> Self {
                unsafe {
                    let mut result = sys::$GdType::default();
                    (get_api().$gd_method)(&mut result, self.sys());
                    <$Type>::from_sys(result)
                }
            }
        }
    };

    (
        Default for $Type:ty as $GdType:ident : $gd_method:ident
    ) => {
        impl Default for $Type {
            #[inline]
            fn default() -> Self {
                unsafe {
                    let mut gd_val = sys::$GdType::default();
                    (get_api().$gd_method)(&mut gd_val);
                    <$Type>::from_sys(gd_val)
                }
            }
        }
    };

    (
        PartialEq for $Type:ty as $GdType:ident : $gd_method:ident
    ) => {
        impl PartialEq for $Type {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                unsafe { (get_api().$gd_method)(self.sys(), other.sys()) }
            }
        }
    };

    (
        Eq for $Type:ty as $GdType:ident : $gd_method:ident
    ) => {
        impl PartialEq for $Type {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                unsafe { (get_api().$gd_method)(self.sys(), other.sys()) }
            }
        }
        impl Eq for $Type {}
    };

    (
        RefCounted for $Type:ty as $GdType:ident : $gd_method:ident
    ) => {
        impl RefCounted for $Type {
            #[inline]
            fn new_ref(&self) -> $Type {
                unsafe {
                    let mut result = Default::default();
                    (get_api().$gd_method)(&mut result, self.sys());
                    <$Type>::from_sys(result)
                }
            }
        }
    };
}

macro_rules! impl_basic_traits_as_sys {
    (
        for $Type:ty as $GdType:ident {
            $( $Trait:ident => $gd_method:ident; )*
        }
    ) => (
        $(
            impl_basic_trait_as_sys!(
                $Trait for $Type as $GdType : $gd_method
            );
        )*
    )
}

macro_rules! godot_test {
    ($($test_name:ident $body:block)*) => {
        $(
            #[cfg(feature = "gd_test")]
            #[doc(hidden)]
            #[inline]
            pub fn $test_name() -> bool {
                let str_name = stringify!($test_name);
                println!("   -- {}", str_name);

                let ok = ::std::panic::catch_unwind(
                    || $body
                ).is_ok();

                if !ok {
                    $crate::godot_error!("   !! Test {} failed", str_name);
                }

                ok
            }
        )*
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! godot_wrap_method_parameter_count {
    () => {
        0
    };
    ($name:ident, $($other:ident,)*) => {
        1 + $crate::godot_wrap_method_parameter_count!($($other,)*)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! godot_wrap_method_inner {
    (
        $type_name:ty,
        $map_method:ident,
        fn $method_name:ident(
            $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
            $(, #[opt] $opt_pname:ident : $opt_pty:ty)*
        ) -> $retty:ty
    ) => {
        {
            #[allow(unused_unsafe, unused_variables, unused_assignments, unused_mut)]
            #[allow(clippy::transmute_ptr_to_ptr)]
            unsafe extern "C" fn method(
                this: *mut $crate::sys::godot_object,
                method_data: *mut $crate::libc::c_void,
                user_data: *mut $crate::libc::c_void,
                num_args: $crate::libc::c_int,
                args: *mut *mut $crate::sys::godot_variant
            ) -> $crate::sys::godot_variant {

                use std::panic::{self, AssertUnwindSafe};
                use $crate::Instance;

                if user_data.is_null() {
                    $crate::godot_error!(
                        "gdnative-core: user data pointer for {} is null (did the constructor fail?)",
                        stringify!($type_name),
                    );
                    return $crate::Variant::new().forget();
                }

                let __catch_result = panic::catch_unwind(move || {
                    let __instance: Instance<$type_name> = Instance::from_raw(this, user_data);

                    let num_args = num_args as isize;

                    let num_required_params = $crate::godot_wrap_method_parameter_count!($($pname,)*);
                    if num_args < num_required_params {
                        $crate::godot_error!("Incorrect number of parameters: required {} but got {}", num_required_params, num_args);
                        return $crate::Variant::new();
                    }

                    let num_optional_params = $crate::godot_wrap_method_parameter_count!($($opt_pname,)*);
                    let num_max_params = num_required_params + num_optional_params;
                    if num_args > num_max_params {
                        $crate::godot_error!("Incorrect number of parameters: expected at most {} but got {}", num_max_params, num_args);
                        return $crate::Variant::new();
                    }

                    let mut offset = 0;
                    $(
                        let _variant: &$crate::Variant = ::std::mem::transmute(&mut **(args.offset(offset)));
                        let $pname = match <$pty as $crate::FromVariant>::from_variant(_variant) {
                            Ok(val) => val,
                            Err(err) => {
                                $crate::godot_error!(
                                    "Cannot convert argument #{idx} ({name}) to {ty}: {err} (non-primitive types may impose structural checks)",
                                    idx = offset + 1,
                                    name = stringify!($pname),
                                    ty = stringify!($pty),
                                    err = err,
                                );
                                return $crate::Variant::new();
                            },
                        };

                        offset += 1;
                    )*

                    $(
                        let $opt_pname = if offset < num_args {
                            let _variant: &$crate::Variant = ::std::mem::transmute(&mut **(args.offset(offset)));

                            let $opt_pname = match <$opt_pty as $crate::FromVariant>::from_variant(_variant) {
                                Ok(val) => val,
                                Err(err) => {
                                    $crate::godot_error!(
                                        "Cannot convert argument #{idx} ({name}) to {ty}: {err} (non-primitive types may impose structural checks)",
                                        idx = offset + 1,
                                        name = stringify!($opt_pname),
                                        ty = stringify!($opt_pty),
                                        err = err,
                                    );
                                    return $crate::Variant::new();
                                },
                            };

                            offset += 1;

                            $opt_pname
                        }
                        else {
                            <$opt_pty as ::std::default::Default>::default()
                        };
                    )*

                    let __ret = __instance
                        .$map_method(|__rust_val, $owner| {
                            let ret = __rust_val.$method_name($owner, $($pname,)* $($opt_pname,)*);
                            <$retty as $crate::ToVariant>::to_variant(&ret)
                        })
                        .unwrap_or_else(|err| {
                            $crate::godot_error!("gdnative-core: method call failed with error: {:?}", err);
                            $crate::godot_error!("gdnative-core: check module level documentation on gdnative::user_data for more information");
                            $crate::Variant::new()
                        });

                    std::mem::drop(__instance);

                    __ret
                });

                __catch_result
                    .unwrap_or_else(|_err| {
                        $crate::godot_error!("gdnative-core: method panicked (check stderr for output)");
                        $crate::Variant::new()
                    })
                    .forget()
            }

            method
        }
    };
}

/// Convenience macro to wrap an object's method into a function pointer
/// that can be passed to the engine when registering a class.
#[macro_export]
macro_rules! godot_wrap_method {
    // mutable
    (
        $type_name:ty,
        fn $method_name:ident(
            &mut $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
            $(,#[opt] $opt_pname:ident : $opt_pty:ty)*
            $(,)?
        ) -> $retty:ty
    ) => {
        $crate::godot_wrap_method_inner!(
            $type_name,
            map_mut,
            fn $method_name(
                $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
                $(,#[opt] $opt_pname : $opt_pty)*
            ) -> $retty
        )
    };
    // immutable
    (
        $type_name:ty,
        fn $method_name:ident(
            & $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
            $(,#[opt] $opt_pname:ident : $opt_pty:ty)*
            $(,)?
        ) -> $retty:ty
    ) => {
        $crate::godot_wrap_method_inner!(
            $type_name,
            map,
            fn $method_name(
                $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
                $(,#[opt] $opt_pname : $opt_pty)*
            ) -> $retty
        )
    };
    // mutable without return type
    (
        $type_name:ty,
        fn $method_name:ident(
            &mut $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
            $(,#[opt] $opt_pname:ident : $opt_pty:ty)*
            $(,)?
        )
    ) => {
        $crate::godot_wrap_method!(
            $type_name,
            fn $method_name(
                &mut $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
                $(,#[opt] $opt_pname : $opt_pty)*
            ) -> ()
        )
    };
    // immutable without return type
    (
        $type_name:ty,
        fn $method_name:ident(
            & $self:ident,
            $owner:ident : $owner_ty:ty
            $(,$pname:ident : $pty:ty)*
            $(,#[opt] $opt_pname:ident : $opt_pty:ty)*
            $(,)?
        )
    ) => {
        $crate::godot_wrap_method!(
            $type_name,
            fn $method_name(
                & $self,
                $owner: $owner_ty
                $(,$pname : $pty)*
                $(,#[opt] $opt_pname : $opt_pty)*
            ) -> ()
        )
    };
}
