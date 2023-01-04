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

            // Compatibility warning if using in-house Godot version (not applicable for custom ones)
            #[cfg(not(feature = "custom-godot"))]
            {
                use $crate::core_types::Variant;

                let engine = gdnative::api::Engine::godot_singleton();
                let info = engine.get_version_info();

                if info.get("major").expect("major version") != Variant::new(3)
                || info.get("minor").expect("minor version") != Variant::new(5)
                || info.get("patch").expect("patch version") < Variant::new(1) {
                    let string = info.get("string").expect("version str").to::<String>().expect("version str type");
                    $crate::log::godot_warn!(
                        "This godot-rust version is only compatible with Godot >= 3.5.1 and < 3.6; detected version {}.\n\
                        GDNative mismatches may lead to subtle bugs, undefined behavior or crashes at runtime.\n\
                	    Apply the 'custom-godot' feature if you want to use current godot-rust with another Godot engine version.",
                        string
                    );
                }
            }

            $crate::private::report_panics("nativescript_init", || {
                $crate::init::auto_register($crate::init::InitHandle::new(handle, $crate::init::InitLevel::AUTO));
                $callback($crate::init::InitHandle::new(handle, $crate::init::InitLevel::USER));

                $crate::init::diagnostics::missing_suggested_diagnostics();
            });
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

            $crate::private::report_panics("gdnative_init", || {
                let init_info = $crate::init::InitializeInfo::new(options);
                $callback(&init_info)
            });
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

            $crate::private::report_panics("gdnative_terminate", || {
                let term_info = $crate::init::TerminateInfo::new(options);
                $callback(&term_info)
            });

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
