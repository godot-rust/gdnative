use std::ffi::CString;
use std::panic::{catch_unwind, UnwindSafe};

use crate::sys;

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Unsafe helpers for sys

static mut GODOT_API: Option<sys::GodotApi> = None;
static mut GDNATIVE_LIBRARY_SYS: Option<*mut sys::godot_object> = None;

/// Binds the API struct from `gdnative_init_options`. Returns `true` on success.
///
/// # Safety
///
/// This is intended to be an internal interface.
#[inline]
pub unsafe fn bind_api(options: *mut sys::godot_gdnative_init_options) -> bool {
    if let Err(err) = check_api_compatibility(options) {
        report_init_error(options, err);
        return false;
    }

    let api = match sys::GodotApi::from_raw((*options).api_struct) {
        Ok(api) => api,
        Err(e) => {
            report_init_error(options, e);
            return false;
        }
    };

    GODOT_API = Some(api);
    GDNATIVE_LIBRARY_SYS = Some((*options).gd_native_library);

    ObjectMethodTable::get(get_api());
    ReferenceMethodTable::get(get_api());
    NativeScriptMethodTable::get(get_api());

    true
}

unsafe fn check_api_compatibility(
    options: *const sys::godot_gdnative_init_options,
) -> Result<(), sys::InitError> {
    use crate::sys::godot_gdnative_api_struct as api_struct;
    let mut api: *const api_struct = (*options).api_struct as *const api_struct;

    // Check for unsupported versions
    loop {
        let sys::godot_gdnative_api_struct {
            type_,
            version,
            next,
        } = *api;

        // Note: this code ensured that Godot 4 (which was back then GDNative 1.3) wasn't actually used.
        // Godot uses now GDExtension, so this no longer applies. Keep this around in case old Godot 4 versions still
        // need to be detected in the future. Now just check against minor version 1.
        // See also: gdnative-sys/build.rs:485
        // See also: https://github.com/godot-rust/godot-rust/issues/904

        // Godot 4 is not yet supported
        if type_ as crate::sys::GDNATIVE_API_TYPES == crate::sys::GDNATIVE_API_TYPES_GDNATIVE_CORE
            && version.major != 1
        //  && version.major == 1 && version.minor == 3   (old check)
        {
            return Err(sys::InitError::Generic {
                message: "GDNative major version 1 expected".into(),
            });
            //return Err(sys::InitError::Generic{ message: "GodotEngine v4.* is not yet supported. See https://github.com/godot-rust/godot-rust/issues/396".into() });
        }

        api = next;
        if api.is_null() {
            break;
        }
    }
    Ok(())
}

/// Returns a reference to the current API struct.
///
/// This function is intended to be part of the internal API. It should only be called after
/// `gdnative_init` and before `gdnative_terminate`.
///
/// # Panics
///
/// **Calling this function when the API is not bound will panic**. Note that it will abort
/// directly during tests (i.e. in unit testing or enable `feature = gd-test`). Since in
/// most cases there is simply no point to continue if `get_api` failed. This allows it to
/// be used in FFI contexts without a `catch_unwind`. (Unwinding across FFI boundary is an
/// undefined behavior.)
///
/// In testing environment, this function use `Option::expect` because unwinding in this
/// scenario should be safe.
///
/// See more: https://github.com/godot-rust/godot-rust/pull/929
#[inline]
pub fn get_api() -> &'static sys::GodotApi {
    const ERR_MSG: &str = "
    This code requires the Godot engine to be running and the GDNative initialization \
    to have completed. It cannot execute as a standalone Rust program.
    
    Hint: If you encounter this issue during unit testing, you might \
    need to use the godot_test! macro, and invoke the test functions in test/src/lib.rs.
    ";

    // Unwinding during tests should be safe and provide more ergonomic UI.
    #[cfg(any(test, feature = "gd-test"))]
    unsafe {
        return GODOT_API.as_ref().expect(ERR_MSG);
    }

    // Abort directly to avoid undefined behaviors.
    #[cfg(not(any(test, feature = "gd-test")))]
    unsafe {
        return GODOT_API.as_ref().unwrap_or_else(|| {
            eprintln!("{}", ERR_MSG);
            std::process::abort();
        });
    }
}

/// Returns a reference to the current API struct if it is bounds, or `None` otherwise.
#[inline]
pub(crate) fn try_get_api() -> Option<&'static sys::GodotApi> {
    unsafe { GODOT_API.as_ref() }
}

/// Returns whether the API is bound.
///
/// This is intended to be an internal interface.
#[inline]
pub fn is_api_bound() -> bool {
    unsafe { GODOT_API.is_some() }
}

/// Returns a pointer to the `GDNativeLibrary` object for the current library.
///
/// This is intended to be an internal interface.
#[inline]
pub fn get_gdnative_library_sys() -> *mut sys::godot_object {
    unsafe { GDNATIVE_LIBRARY_SYS.expect("GDNativeLibrary not bound") }
}

/// Performs library-wide cleanup during `terminate`.
///
/// # Safety
///
/// This is intended to be an internal interface.
#[inline]
pub unsafe fn cleanup_internal_state() {
    crate::export::type_tag::cleanup();
    crate::export::class_registry::cleanup();

    GODOT_API = None;
}

/// Reports an `InitError` to Godot.
#[inline]
unsafe fn report_init_error(
    options: *const sys::godot_gdnative_init_options,
    error: sys::InitError,
) {
    match error {
        sys::InitError::VersionMismatch {
            api_type,
            want,
            got,
        } => {
            if let Some(f) = (*options).report_version_mismatch {
                let message = CString::new(format!("{api_type}")).unwrap();
                f((*options).gd_native_library, message.as_ptr(), want, got);
            }
        }
        sys::InitError::Generic { message } => {
            if let Some(f) = (*options).report_loading_error {
                let message = CString::new(message).unwrap();
                f((*options).gd_native_library, message.as_ptr());
            }
        }
    }
}

#[inline]
pub fn report_panics(context: &str, callback: impl FnOnce() + UnwindSafe) {
    let __result = catch_unwind(callback);

    if let Err(e) = __result {
        godot_error!("gdnative-core: {} callback panicked", context);
        print_panic_error(e);
    }
}

pub(crate) fn print_panic_error(err: Box<dyn std::any::Any + Send>) {
    if let Some(s) = err.downcast_ref::<String>() {
        godot_error!("Panic message: {}", s);
    } else if let Some(s) = err.downcast_ref::<&'static str>() {
        godot_error!("Panic message: {}", s);
    } else {
        godot_error!("Panic message unknown, type {:?}", err.type_id());
    }
}

/// Plugin type to be used by macros for auto class registration.
pub struct AutoInitPlugin {
    pub f: fn(init_handle: crate::init::InitHandle),
}

#[cfg(feature = "inventory")]
pub mod inventory {
    pub use inventory::{collect, submit};

    inventory::collect!(super::AutoInitPlugin);
}

#[cfg(not(feature = "inventory"))]
pub mod inventory {
    pub use crate::_inventory_discard as submit;
    pub use crate::_inventory_discard as collect;

    #[macro_export]
    #[doc(hidden)]
    macro_rules! _inventory_discard {
        ($($tt:tt)*) => {};
    }
}

pub mod godot_object {
    pub trait Sealed {}
}

pub mod mixin {
    pub trait Sealed {}

    pub struct Opaque {
        _private: (),
    }
}

pub(crate) struct ManuallyManagedClassPlaceholder;

unsafe impl crate::object::GodotObject for ManuallyManagedClassPlaceholder {
    type Memory = crate::object::memory::ManuallyManaged;

    fn class_name() -> &'static str {
        "Object"
    }
}

impl godot_object::Sealed for ManuallyManagedClassPlaceholder {}

pub(crate) struct ReferenceCountedClassPlaceholder;

unsafe impl crate::object::GodotObject for ReferenceCountedClassPlaceholder {
    type Memory = crate::object::memory::RefCounted;

    fn class_name() -> &'static str {
        "Reference"
    }
}

impl godot_object::Sealed for ReferenceCountedClassPlaceholder {}

macro_rules! make_method_table {
    (struct $tablename:ident for $class:ident { $($methods:ident,)* }) => {
        pub(crate) struct $tablename {
            $(pub(crate) $methods: *mut sys::godot_method_bind,)*
        }

        impl $tablename {
            unsafe fn get_mut() -> &'static mut Self {
                static mut TABLE: $tablename = $tablename {
                    $($methods: std::ptr::null_mut(),)*
                };

                &mut TABLE
            }

            #[inline]
            pub(crate) fn get(api: &sys::GodotApi) -> &'static Self {
                unsafe {
                    let table = Self::get_mut();
                    static INIT: std::sync::Once = std::sync::Once::new();
                    INIT.call_once(|| {
                        Self::init(table, api);
                    });

                    table
                }
            }

            #[inline(never)]
            fn init(table: &mut Self, api: &sys::GodotApi) {
                const CLASS_NAME: *const libc::c_char = concat!(stringify!($class), "\0").as_ptr() as *const libc::c_char;

                unsafe {
                    $(table.$methods = (api.godot_method_bind_get_method)(CLASS_NAME, concat!(stringify!($methods), "\0").as_ptr() as *const libc::c_char);)*
                }
            }
        }
    };
}

make_method_table!(struct ObjectMethodTable for Object {
    get_class,
    is_class,
});

make_method_table!(struct ReferenceMethodTable for Reference {
    reference,
    unreference,
    init_ref,
});

// Add this one here too. It's not easy to use this macro from the
// export module without making this macro public.
make_method_table!(struct NativeScriptMethodTable for NativeScript {
    set_class_name,
    set_library,
    new,
});
