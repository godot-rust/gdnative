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
