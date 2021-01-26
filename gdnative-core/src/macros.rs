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
        fn godot_gdnative_init_empty(_options: &$crate::InitializeInfo) {}
        $crate::godot_gdnative_init!(godot_gdnative_init_empty);
    };
    (_ as $fn_name:ident) => {
        fn godot_gdnative_init_empty(_options: &$crate::InitializeInfo) {}
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

            let __result = ::std::panic::catch_unwind(|| {
                let callback_options = $crate::InitializeInfo::new(options);
                $callback(&callback_options)
            });
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
        fn godot_gdnative_terminate_empty(_term_info: &$crate::TerminateInfo) {}
        $crate::godot_gdnative_terminate!(godot_gdnative_terminate_empty);
    };
    ($callback:ident) => {
        $crate::godot_gdnative_terminate!($callback as godot_gdnative_terminate);
    };
    (_ as $fn_name:ident) => {
        fn godot_gdnative_terminate_empty(_term_info: &$crate::TerminateInfo) {}
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

            let __result = ::std::panic::catch_unwind(|| {
                let term_info = $crate::TerminateInfo::new(options);
                $callback(&term_info)
            });
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
        $crate::log::print(::std::format_args!($($args)*));
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

/// Creates a `gdnative::log::Site` value from the current position in code,
/// optionally with a function path for identification.
///
/// # Examples
///
/// ```ignore
/// // WARN: <unset>: foo At: path/to/file.rs:123
/// gdnative::log::warn(godot_site!(), "foo");
/// // WARN: Foo::my_func: bar At: path/to/file.rs:123
/// gdnative::log::error(godot_site!(Foo::my_func), "bar");
/// ```
#[macro_export]
macro_rules! godot_site {
    () => {{
        // SAFETY: I guess we can assume that all sane file systems don't allow
        // NUL-bytes in paths?
        #[allow(unused_unsafe)]
        let site: $crate::log::Site<'static> = unsafe {
            let file = ::std::ffi::CStr::from_bytes_with_nul_unchecked(
                ::std::concat!(::std::file!(), "\0").as_bytes(),
            );
            let func = ::std::ffi::CStr::from_bytes_with_nul_unchecked(b"<unset>\0");
            $crate::log::Site::new(file, func, ::std::line!())
        };

        site
    }};
    ($($path:tt)+) => {{
        // SAFETY: I guess we can assume that all sane file systems don't allow
        // NUL-bytes in paths?
        #[allow(unused_unsafe)]
        let site: $crate::log::Site<'static> = unsafe {
            let file = ::std::ffi::CStr::from_bytes_with_nul_unchecked(
                ::std::concat!(::std::file!(), "\0").as_bytes(),
            );
            let func = ::std::ffi::CStr::from_bytes_with_nul_unchecked(
                ::std::concat!(::std::stringify!($($path)+), "\0").as_bytes(),
            );
            $crate::log::Site::new(file, func, ::std::line!())
        };

        site
    }};
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
        $crate::log::warn($crate::godot_site!(), ::std::format_args!($($args)*));
    });
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
        $crate::log::error($crate::godot_site!(), ::std::format_args!($($args)*));
    });
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
        NewRef for $Type:ty as $GdType:ident : $gd_method:ident
    ) => {
        impl NewRef for $Type {
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
