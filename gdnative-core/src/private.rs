use std::ffi::CString;

use crate::core_types::GodotString;
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
    #[cfg(feature = "nativescript")]
    {
        NativeScriptMethodTable::get(get_api());
    }

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

        // Godot 4 is not yet supported
        if type_ as crate::sys::GDNATIVE_API_TYPES == crate::sys::GDNATIVE_API_TYPES_GDNATIVE_CORE
            && version.major == 1
            && version.minor == 3
        {
            return Err(sys::InitError::Generic{ message: "GodotEngine v4.* is not yet supported. See https://github.com/godot-rust/godot-rust/issues/396".into() });
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
/// `gdnative_init` and before `gdnative_terminate`. **Calling this function when the API is
/// not bound will lead to an abort**, since in most cases there is simply no point to continue
/// if `get_api` failed. This allows it to be used in FFI contexts without a `catch_unwind`.
#[inline]
pub fn get_api() -> &'static sys::GodotApi {
    unsafe { GODOT_API.as_ref().unwrap_or_else(|| std::process::abort()) }
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
    #[cfg(feature = "nativescript")]
    {
        crate::nativescript::type_tag::cleanup();
        crate::nativescript::class_registry::cleanup();
    }
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
                let message = CString::new(format!("{}", api_type)).unwrap();
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

pub mod godot_object {
    pub trait Sealed {}
}

pub(crate) struct ManuallyManagedClassPlaceholder;

unsafe impl crate::object::GodotObject for ManuallyManagedClassPlaceholder {
    type RefKind = crate::object::memory::ManuallyManaged;

    fn class_name() -> &'static str {
        "Object"
    }
}

impl godot_object::Sealed for ManuallyManagedClassPlaceholder {}

pub(crate) struct ReferenceCountedClassPlaceholder;

unsafe impl crate::object::GodotObject for ReferenceCountedClassPlaceholder {
    type RefKind = crate::object::memory::RefCounted;

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
// nativescript module without making this macro public.
#[cfg(feature = "nativescript")]
make_method_table!(struct NativeScriptMethodTable for NativeScript {
    set_class_name,
    set_library,
    new,
});

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Helper structs for init/terminate

pub struct TerminateInfo {
    in_editor: bool,
}

impl TerminateInfo {
    #[inline]
    #[doc(hidden)] // avoids clippy warning: unsafe function's docs miss `# Safety` section
    pub unsafe fn new(options: *mut crate::sys::godot_gdnative_terminate_options) -> Self {
        assert!(!options.is_null(), "options were NULL");

        let crate::sys::godot_gdnative_terminate_options { in_editor } = *options;

        Self { in_editor }
    }

    /// Returns `true` if the library is loaded in the Godot Editor.
    #[inline]
    pub fn in_editor(&self) -> bool {
        self.in_editor
    }
}

pub struct InitializeInfo {
    in_editor: bool,
    active_library_path: GodotString,
    options: *mut crate::sys::godot_gdnative_init_options,
}

impl InitializeInfo {
    /// Returns true if the library is loaded in the Godot Editor.
    #[inline]
    pub fn in_editor(&self) -> bool {
        self.in_editor
    }

    /// Returns a path to the library relative to the project.
    ///
    /// Example: `res://../../target/debug/libhello_world.dylib`
    #[inline]
    pub fn active_library_path(&self) -> &GodotString {
        &self.active_library_path
    }

    /// # Safety
    ///
    /// Will `panic!()` if options is NULL or invalid.
    #[inline]
    #[doc(hidden)]
    pub unsafe fn new(options: *mut crate::sys::godot_gdnative_init_options) -> Self {
        assert!(!options.is_null(), "options were NULL");
        let crate::sys::godot_gdnative_init_options {
            in_editor,
            active_library_path,
            ..
        } = *options;

        let active_library_path =
            crate::core_types::GodotString::clone_from_sys(*active_library_path);

        Self {
            in_editor,
            active_library_path,
            options,
        }
    }

    #[inline]
    pub fn report_loading_error<T>(&self, message: T)
    where
        T: std::fmt::Display,
    {
        let crate::sys::godot_gdnative_init_options {
            report_loading_error,
            gd_native_library,
            ..
        } = unsafe { *self.options };

        if let Some(report_loading_error_fn) = report_loading_error {
            // Add the trailing zero and convert Display => String
            let message = format!("{}\0", message);

            // Convert to FFI compatible string
            let message = std::ffi::CStr::from_bytes_with_nul(message.as_bytes())
                .expect("message should not have a NULL");

            unsafe {
                report_loading_error_fn(gd_native_library, message.as_ptr());
            }
        }
    }
}
