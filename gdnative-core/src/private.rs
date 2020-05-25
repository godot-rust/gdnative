use std::ffi::CString;

use crate::sys;

static mut GODOT_API: Option<sys::GodotApi> = None;
static mut GDNATIVE_LIBRARY_SYS: Option<*mut sys::godot_object> = None;

/// Binds the API struct from `gdnative_init_options`. Returns `true` on success.
///
/// This is intended to be an internal interface.
#[inline]
pub unsafe fn bind_api(options: *mut sys::godot_gdnative_init_options) -> bool {
    let api = match sys::GodotApi::from_raw((*options).api_struct) {
        Ok(api) => api,
        Err(e) => {
            report_init_error(options, e);
            return false;
        }
    };

    GODOT_API = Some(api);
    GDNATIVE_LIBRARY_SYS = Some((*options).gd_native_library);

    // Force the initialization of the method table of common types. This way we can
    // assume that if the api object is alive we can fetch the method of these types
    // without checking for initialization.
    crate::ReferenceMethodTable::get(get_api());

    true
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
/// This is intended to be an internal interface.
#[inline]
pub unsafe fn cleanup_internal_state() {
    crate::type_tag::cleanup();
    GODOT_API = None;
}

/// Reports an `InitError` to Godot.
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
                f(
                    (*options).gd_native_library,
                    CString::new(format!("{}", api_type)).unwrap().as_ptr(),
                    want,
                    got,
                );
            }
        }
        sys::InitError::Generic { message } => {
            if let Some(f) = (*options).report_loading_error {
                f(
                    (*options).gd_native_library,
                    CString::new(message).unwrap().as_ptr(),
                );
            }
        }
    }
}
