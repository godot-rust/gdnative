#![macro_use]

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

/// Creates a [`Site`][crate::log::Site] value from the current position in code,
/// optionally with a function path for identification.
///
/// # Examples
///
/// ```ignore
/// use gdnative::log;
///
/// // WARN: <unset>: foo At: path/to/file.rs:123
/// log::warn(log::godot_site!(), "foo");
///
/// // WARN: Foo::my_func: bar At: path/to/file.rs:123
/// log::error(log::godot_site!(Foo::my_func), "bar");
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
        impl_basic_trait_as_sys!(PartialEq for $Type as $GdType : $gd_method);
        impl Eq for $Type {}
    };

    (
        Ord for $Type:ty as $GdType:ident : $gd_method:ident
    ) => {
        impl PartialOrd for $Type {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for $Type {
            #[inline]
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                let op_less = get_api().$gd_method;
                if unsafe { op_less(&self.0, &other.0) } {
                    std::cmp::Ordering::Less
                } else if unsafe { op_less(&other.0, &self.0) } {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            }
        }
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

#[doc(hidden)]
#[macro_export]
macro_rules! godot_test_impl {
    ( $( $test_name:ident $body:block $($attrs:tt)* )* ) => {
        $(
            $($attrs)*
            #[doc(hidden)]
            #[inline]
            #[must_use]
            pub fn $test_name() -> bool {
                let str_name = stringify!($test_name);
                println!("   -- {}", str_name);

                let ok = ::std::panic::catch_unwind(
                    || $body
                ).is_ok();

                if !ok {
                    if ::std::panic::catch_unwind(|| {
                        $crate::godot_error!("   !! Test {} failed", str_name);
                    }).is_err() {
                        eprintln!("   !! Test {} failed", str_name);
                        eprintln!("   !! And failed to call Godot API to log error message");
                    }
                }

                ok
            }
        )*
    }
}

/// Declares a test to be run with the Godot engine (i.e. not a pure Rust unit test).
///
/// Creates a wrapper function that catches panics, prints errors and returns true/false.
/// To be manually invoked in higher-level test routine.
///
/// This macro is designed to be used within the current crate only, hence the #[cfg] attribute.
#[doc(hidden)]
macro_rules! godot_test {
    ($($test_name:ident $body:block)*) => {
        $(
            godot_test_impl!($test_name $body #[cfg(feature = "gd-test")]);
        )*
    }
}

/// Declares a test to be run with the Godot engine (i.e. not a pure Rust unit test).
///
/// Creates a wrapper function that catches panics, prints errors and returns true/false.
/// To be manually invoked in higher-level test routine.
///
/// This macro is designed to be used within the `test` crate, hence the method is always declared (not only in certain features).
#[doc(hidden)]
#[macro_export]
macro_rules! godot_itest {
    ($($test_name:ident $body:block)*) => {
        $(
            $crate::godot_test_impl!($test_name $body);
        )*
    }
}
