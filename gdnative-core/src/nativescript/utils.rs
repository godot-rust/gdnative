use std::ffi::CString;

use crate::private::get_api;

/// Register data for Godot's built-in profiler.
///
/// The `signature` needs to be formatted as `"::<LINE-NUMBER>::<FUNCTION>"` for the
/// profiler to accept the data (although the line number does not seem to play role).
#[inline]
pub fn profiling_add_data(signature: &str, time_in_usec: u64) {
    let c_string = CString::new(signature);

    if let Ok(c_string) = c_string {
        unsafe {
            let api = get_api();
            (api.godot_nativescript_profiling_add_data)(c_string.as_ptr(), time_in_usec);
        }
    }
}
