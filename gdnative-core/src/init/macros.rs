#![macro_use]

/// Declare the API endpoint to initialize export classes on startup.
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
        fn godot_gdnative_init_empty(_options: &$crate::init::InitializeInfo) {}
        $crate::init::godot_gdnative_init!(godot_gdnative_init_empty);
    };
    (_ as $fn_name:ident) => {
        fn godot_gdnative_init_empty(_options: &$crate::init::InitializeInfo) {}
        $crate::init::godot_gdnative_init!(godot_gdnative_init_empty as $fn_name);
    };
    ($callback:ident) => {
        $crate::init::godot_gdnative_init!($callback as godot_gdnative_init);
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
                let callback_options = $crate::init::InitializeInfo::new(options);
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
        fn godot_gdnative_terminate_empty(_term_info: &$crate::init::TerminateInfo) {}
        $crate::init::godot_gdnative_terminate!(godot_gdnative_terminate_empty);
    };
    ($callback:ident) => {
        $crate::init::godot_gdnative_terminate!($callback as godot_gdnative_terminate);
    };
    (_ as $fn_name:ident) => {
        fn godot_gdnative_terminate_empty(_term_info: &$crate::init::TerminateInfo) {}
        $crate::init::godot_gdnative_terminate!(godot_gdnative_terminate_empty as $fn_name);
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
                let term_info = $crate::init::TerminateInfo::new(options);
                $callback(&term_info)
            });
            if __result.is_err() {
                $crate::godot_error!("gdnative-core: nativescript_init callback panicked");
            }

            $crate::private::cleanup_internal_state();
        }
    };
}

/// Declare all the API endpoints necessary to initialize a NativeScript library.
///
/// `godot_init!(init)` is a shorthand for:
///
/// ```ignore
/// godot_gdnative_init!();
/// godot_nativescript_init!(init);
/// godot_gdnative_terminate!();
/// ```
///
/// This declares three extern functions, named `godot_gdnative_init`,
/// `godot_nativescript_init`, and `godot_gdnative_terminate`. If you need different prefixes
/// to avoid name clashes when multiple GDNative libraries are linked statically, please use
/// the respective macros directly.
#[macro_export]
macro_rules! godot_init {
    ($callback:ident) => {
        $crate::init::godot_gdnative_init!();
        $crate::init::godot_nativescript_init!($callback);
        $crate::init::godot_gdnative_terminate!();
    };
}
